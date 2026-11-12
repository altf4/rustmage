//! Non-targeted return choices use exact movement receipts after replacements.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const CACHE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::Mill(PlayerSet::Controller, MillAmount::Fixed(4)),
            Instruction::SelectMilled(MilledSelectionSpec {
                filter: MilledFilter::Permanent,
                count: MilledCount::Range {
                    minimum: 0,
                    maximum: 1,
                },
                destination: MilledDestination::HandAndCountSubtype("Squirrel"),
            }),
            Instruction::GainLifeScaled(PlayerSet::Controller, AmountValue::MovedCount),
        ],
    ),
    ..CardRules::EMPTY
};
const ELDER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::Upkeep,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::ChooseObjects {
                kind: EffectChoiceKind::ReturnToHandCounting,
                players: PlayerSet::Controller,
                filter: SacrificeFilter {
                    kinds: 1,
                    nonland: false,
                    other: true,
                    source_only: false,
                    tokens: None,
                    subtype: None,
                    keywords: Keywords::NONE,
                    artifact_enchantment_or_token: false,
                },
                amount: 1,
                optional_skip: None,
            },
            Instruction::IfAmountAtLeast {
                value: AmountValue::MovedCount,
                minimum: 1,
                skip: 2,
            },
            Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::PlusOne,
                amount: 1,
            },
            Instruction::Skip(2),
            Instruction::MayIfSourceOnBattlefield { skip: 1 },
            Instruction::ReturnSourceToHand(TargetZone::Battlefield),
        ],
    }],
    ..CardRules::EMPTY
};

struct ResetPrevention;
impl Drop for ResetPrevention {
    fn drop(&mut self) {
        zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|slot| slot.set(None));
    }
}
fn prevent(object: ObjectRef) -> ResetPrevention {
    zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|slot| slot.set(Some(object)));
    ResetPrevention
}
fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}
fn pass_pair(g: &mut Game) {
    act(g, Action::Pass);
    act(g, Action::Pass);
}
fn select_milled(g: &Game, card: CardId) -> Action {
    Action::SelectMilled {
        object: g.object_ref(card),
    }
}
fn return_choice(g: &Game, card: CardId) -> Action {
    Action::ChooseEffectObject {
        object: g.object_ref(card),
    }
}

#[test]
fn milling_accepts_battles_and_only_counts_a_squirrel_that_reaches_hand() {
    let _rules = fixture_rules::activate(&[(named("Opt"), &CACHE)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let instant = put(&mut g, p, "Shock", Zone::Library);
        let battle = put(&mut g, p, "Shock", Zone::Library);
        let land = put(&mut g, p, "Forest", Zone::Library);
        let squirrel = put(&mut g, p, "Bushy Bodyguard", Zone::Library);
        let unbound = put(&mut g, p, "Forest", Zone::Graveyard);
        // Model a physical Battle card independently of Standard admission.
        let mut recipe = CopyRecipe::new(named("Shock"));
        recipe.overrides = Some(CopyOverrides {
            kinds: Some(BATTLE_KIND),
            ..CopyOverrides::NONE
        });
        g.set_copy_recipe(g.object_ref(battle), recipe);
        let spell = put(&mut g, p, "Opt", Zone::Hand);
        cast(&mut g, spell, None);
        pass_pair(&mut g);

        assert_eq!(g.current_kinds(battle), BATTLE_KIND);
        let mut candidates = g.observe(p).milled_choice.unwrap().candidates;
        candidates.sort_by_key(|object| object.card);
        assert_eq!(
            candidates,
            [
                g.object_ref(battle),
                g.object_ref(land),
                g.object_ref(squirrel)
            ]
        );
        let wrong_type = select_milled(&g, instant);
        let wrong_batch = select_milled(&g, unbound);
        reject(&mut g, wrong_type);
        reject(&mut g, wrong_batch);

        let mut battle_branch = g.clone();
        let pick = select_milled(&battle_branch, battle);
        act(&mut battle_branch, pick);
        assert_eq!(battle_branch.object_view(battle).zone, Zone::Hand);
        assert_eq!(battle_branch.state.players[p.index()].life, 20);

        let pick = select_milled(&g, squirrel);
        let mut prevented = g.clone();
        {
            let _reset = prevent(g.object_ref(squirrel));
            act(&mut prevented, pick.clone());
        }
        assert_eq!(prevented.object_view(squirrel).zone, Zone::Graveyard);
        assert_eq!(prevented.state.players[p.index()].life, 20);
        act(&mut g, pick);
        assert_eq!(g.object_view(squirrel).zone, Zone::Hand);
        assert_eq!(g.state.players[p.index()].life, 21);
    }
}

#[test]
fn chosen_non_target_return_is_mandatory_and_only_a_success_enables_the_counter() {
    let _rules = fixture_rules::activate(&[(named("Bear Cub"), &ELDER)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let elder = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let foreign_owned = put(&mut g, p.other(), "Llanowar Elves", Zone::Battlefield);
        g.state.objects[foreign_owned as usize].controller = p;
        let foe = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        g.begin_step(Step::Upkeep);
        g.prepare_triggers();
        pass_pair(&mut g);
        reject(&mut g, Action::DeclineEffectPayment);
        let self_choice = return_choice(&g, elder);
        let foe_choice = return_choice(&g, foe);
        reject(&mut g, self_choice);
        reject(&mut g, foe_choice);

        let chosen = return_choice(&g, foreign_owned);
        let index = g.legal_actions().iter().position(|a| *a == chosen).unwrap();
        let revision = g.revision();
        let before = g.snapshot();
        let mut indexed = g.clone();
        indexed.apply_index(p, revision, index).unwrap();
        assert_eq!(indexed.object_view(foreign_owned).zone, Zone::Hand);
        assert_eq!(indexed.object_view(foreign_owned).controller, p.other());
        assert_eq!(indexed.counter_count(elder, CounterKind::PlusOne), 1);
        indexed.restore(&before).unwrap();
        assert_eq!(
            indexed.apply_index(p, revision, index),
            Err(Error::StaleRevision)
        );
        act(&mut indexed, chosen.clone());
        assert_eq!(indexed.counter_count(elder, CounterKind::PlusOne), 1);

        {
            let _reset = prevent(g.object_ref(foreign_owned));
            act(&mut g, chosen);
        }
        assert_eq!(g.object_view(foreign_owned).zone, Zone::Battlefield);
        assert_eq!(g.counter_count(elder, CounterKind::PlusOne), 0);
        assert!(matches!(g.state.pending, Pending::EffectOptional { .. }));
        act(&mut g, Action::ChooseOptional { accept: true });
        settle(&mut g);
        assert_eq!(g.object_view(elder).zone, Zone::Hand);

        let mut alone = scenario(p);
        let elder = put(&mut alone, p, "Bear Cub", Zone::Battlefield);
        alone.begin_step(Step::Upkeep);
        alone.prepare_triggers();
        pass_pair(&mut alone);
        assert!(matches!(
            alone.state.pending,
            Pending::EffectOptional { .. }
        ));
        act(&mut alone, Action::ChooseOptional { accept: false });
        assert_eq!(alone.object_view(elder).zone, Zone::Battlefield);
        assert_eq!(alone.counter_count(elder, CounterKind::PlusOne), 0);
    }
}
