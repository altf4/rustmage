//! Exact mandatory graveyard selection after a paid optional sacrifice.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const END_STEP_RETURN: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::End,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::MayPayCosts {
                costs: &[ResolutionCost {
                    objects: Some(ResolutionObjectCost {
                        cost: AdditionalCost {
                            kind: EffectChoiceKind::Sacrifice,
                            filter: SacrificeFilter {
                                keywords: Keywords::NONE,
                                source_only: false,
                                kinds: 0,
                                nonland: true,
                                other: true,
                                tokens: None,
                                artifact_enchantment_or_token: false,
                                subtype: None,
                            },
                        },
                        count: 3,
                    }),
                    ..ResolutionCost::FREE
                }],
                skip: 1,
            },
            Instruction::ChooseGraveyardReturnWithCounter(CounterKind::Finality),
        ],
    }],
    ..CardRules::EMPTY
};

fn watcher() -> Card {
    Card::from_name("Impact Tremors").unwrap()
}

fn start(game: &mut Game, player: Player) -> CardId {
    let source = spawn(game, player, watcher(), Zone::Battlefield);
    game.begin_step(Step::End);
    game.prepare_triggers();
    assert_eq!(game.state.stack.len(), 1);
    assert!(game.state.stack[0].targets.is_empty());
    act(game, Action::Pass);
    act(game, Action::Pass);
    source
}

fn choose_cost(game: &Game, card: CardId) -> Action {
    Action::ChooseAdditionalCost {
        object: game.object_ref(card),
    }
}

fn return_creature(game: &Game, card: CardId) -> Action {
    Action::SelectGraveyardReturn {
        object: game.object_ref(card),
    }
}

fn sacrifice(game: &mut Game, cards: [CardId; 3]) {
    for card in cards {
        act(game, choose_cost(game, card));
    }
    act(game, Action::PayResolutionMana { payment: [0; 6] });
}

struct ResetPrevention;

impl Drop for ResetPrevention {
    fn drop(&mut self) {
        zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|rule| rule.set(None));
    }
}

#[test]
fn a_freshly_sacrificed_creature_can_be_chosen_and_returns_with_finality_after_entry_choices() {
    let _rules = fixture_rules::activate(&[(watcher(), &END_STEP_RETURN)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let fresh = put(&mut game, player, "Adaptive Automaton", Zone::Battlefield);
    let bow = put(&mut game, player, "Short Bow", Zone::Battlefield);
    let tome = put(&mut game, player, "Mazemind Tome", Zone::Battlefield);
    let land = put(&mut game, player, "Forest", Zone::Battlefield);
    let old = put(&mut game, player, "Grizzly Bears", Zone::Graveyard);
    let foreign = put(&mut game, opponent, "Bear Cub", Zone::Graveyard);
    let source = start(&mut game, player);
    for invalid in [source, land, foreign] {
        let action = choose_cost(&game, invalid);
        reject(&mut game, action);
    }
    reject(&mut game, Action::PayResolutionMana { payment: [0; 6] });

    let mut declined = game.clone();
    act(&mut declined, Action::DeclineResolutionMana);
    assert_eq!(declined.state.pending, Pending::Priority);
    for card in [fresh, bow, tome] {
        assert_eq!(declined.object_view(card).zone, Zone::Battlefield);
    }
    assert_eq!(declined.object_view(old).zone, Zone::Graveyard);

    sacrifice(&mut game, [fresh, bow, tome]);
    let current = game.object_ref(fresh);
    for viewer in [player, opponent] {
        let choice = game.observe(viewer).effect_choice.unwrap();
        assert!(!choice.optional);
        assert_eq!(choice.required[player.index()], 1);
        assert_eq!(choice.candidates, [current, game.object_ref(old)]);
    }
    reject(&mut game, Action::FinishGraveyardReturn);
    for invalid in [foreign, bow, tome] {
        let action = return_creature(&game, invalid);
        reject(&mut game, action);
    }

    let snapshot = game.snapshot();
    let action = return_creature(&game, fresh);
    let mut stale = game.clone();
    stale.move_object(fresh, Zone::Exile);
    stale.move_object(fresh, Zone::Graveyard);
    reject(&mut stale, action.clone());

    let mut prevented = game.clone();
    let blocked = prevented.object_ref(old);
    let reset = ResetPrevention;
    zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|rule| rule.set(Some(blocked)));
    prevented.state.zone_rules.get_or_insert_default();
    act(
        &mut prevented,
        Action::SelectGraveyardReturn { object: blocked },
    );
    assert_eq!(prevented.state.pending, Pending::Priority);
    assert_eq!(prevented.object_ref(old), blocked);
    assert_eq!(prevented.object_view(old).zone, Zone::Graveyard);
    assert_eq!(prevented.counter_count(old, CounterKind::Finality), 0);
    drop(reset);

    let index = game
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    let mut indexed = game.clone();
    indexed.apply_index(player, game.revision(), index).unwrap();
    act(&mut game, action);
    assert_eq!(game.snapshot(), indexed.snapshot());
    for branch in [&mut game, &mut indexed] {
        assert_eq!(branch.object_ref(fresh), current);
        assert_eq!(branch.object_view(fresh).zone, Zone::Graveyard);
        choose(branch, |a| matches!(a, Action::ChooseEntryValue { .. }));
        assert_ne!(branch.object_ref(fresh), current);
        assert_eq!(branch.object_view(fresh).zone, Zone::Battlefield);
        assert_eq!(branch.counter_count(fresh, CounterKind::Finality), 1);
        assert_eq!(branch.object_view(old).zone, Zone::Graveyard);
        branch.move_object(fresh, Zone::Graveyard);
        assert_eq!(branch.object_view(fresh).zone, Zone::Exile);
    }
    assert_eq!(game.snapshot(), indexed.snapshot());
    indexed.restore(&snapshot).unwrap();
    assert_eq!(indexed.snapshot(), snapshot);
}

#[test]
fn a_paid_sacrifice_can_leave_no_creature_to_return() {
    let _rules = fixture_rules::activate(&[(watcher(), &END_STEP_RETURN)]);
    let player = Player::P1;
    let mut game = scenario(player);
    let artifacts = [(); 3].map(|_| put(&mut game, player, "Short Bow", Zone::Battlefield));
    let opponent = put(&mut game, player.other(), "Grizzly Bears", Zone::Graveyard);
    start(&mut game, player);
    sacrifice(&mut game, artifacts);
    assert_eq!(game.state.pending, Pending::Priority);
    assert!(game.observe(player).effect_choice.is_none());
    for artifact in artifacts {
        assert_eq!(game.object_view(artifact).zone, Zone::Graveyard);
    }
    assert_eq!(game.object_view(opponent).zone, Zone::Graveyard);
}
