//! Triggered tap additions belong to their exact providers, including after a copy.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::engine::funding::{FundingGoal, FundingState};
use rustmage_core::{cards::*, engine::*};
use support::*;

const BLUE: Mana = [0, 1, 0, 0, 0, 0];
const GREEN: Mana = [0, 0, 0, 0, 1, 0];
const BLUE_GREEN: Mana = [0, 1, 0, 0, 1, 0];
const LISTENER: CardRules = CardRules {
    effect: Effect::Land,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCastUsingSourceMana(CastFilter::Permanent),
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};
const LAND: CardRules = CardRules {
    effect: Effect::Land,
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::AddMana(ManaProduction::Fixed(BLUE_GREEN))],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
const SPELL: CardRules = CardRules {
    effect: Effect::Spell(TargetPlan::NONE, &[]),
    ..CardRules::EMPTY
};

fn card(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn live_mana(game: &Game, player: Player, source: ObjectRef) -> Option<[u128; 6]> {
    game.state.mana_lots.as_ref()?.pools[player.index()]
        .iter()
        .find(|lot| lot.quality.producer == Some(source))
        .map(|lot| lot.amounts)
}

#[test]
fn an_aura_can_spend_its_own_triggered_mana_after_becoming_the_listener() {
    let _rules = fixture_rules::activate(&[(card("Captivating Cave"), &LISTENER)]);
    let p = Player::P0;
    let mut game = scenario(p);
    let land = put(&mut game, p, "Forest", Zone::Battlefield);
    let aura = put(&mut game, p, "Shimmerwilds Growth", Zone::Battlefield);
    let multiplier = put(&mut game, p, "Virtue of Strength", Zone::Battlefield);
    let blueprint = put(&mut game, p, "Captivating Cave", Zone::Battlefield);
    let mirror = put(&mut game, p, "Mirrorform", Zone::Hand);
    let spell = put(&mut game, p, "Spyglass Siren", Zone::Hand);
    let land_ref = game.object_ref(land);
    let aura_ref = game.object_ref(aura);
    let multiplier_ref = game.object_ref(multiplier);
    game.state.objects[aura as usize].attached_to = Some(land_ref);
    game.set_entry_value(aura_ref, EntryChoiceValue::Color(ChoiceColor::Blue));
    game.state.players[p.index()].mana = [0, 2, 0, 0, 0, 4];
    cast(&mut game, mirror, Some(Target::Card(blueprint)));
    assert_eq!(game.state.players[p.index()].mana, [0; 6]);

    let action = Action::TapMana { card: land };
    let goal = FundingGoal {
        actor: p,
        required: BLUE,
        generic: 0,
        reserve: None,
        source: None,
    };
    let world = FundingState::from_game(&game, &goal);
    let step = world
        .moves(&goal)
        .into_iter()
        .find(|step| world.action(step) == action)
        .unwrap();
    assert!(world.visit_applied(&step, &mut |next| {
        let units = |source| {
            next.lots
                .iter()
                .find(|lot| lot.quality.producer == Some(source))
                .map(|lot| lot.amounts)
        };
        assert_eq!(units(land_ref), Some([0, 0, 0, 0, 3, 0]));
        assert_eq!(units(aura_ref), Some([0, 1, 0, 0, 0, 0]));
        assert_eq!(units(multiplier_ref), None);
        true
    }));

    let saved = game.snapshot();
    let index = game
        .legal_actions()
        .iter()
        .position(|candidate| *candidate == action)
        .unwrap();
    let mut indexed = game.clone();
    let revision = game.revision();
    act(&mut game, action.clone());
    indexed.apply_index(p, revision, index).unwrap();
    assert_eq!(game.snapshot(), indexed.snapshot());
    for branch in [&mut game, &mut indexed] {
        assert_eq!(live_mana(branch, p, land_ref), Some([0, 0, 0, 0, 3, 0]));
        assert_eq!(live_mana(branch, p, aura_ref), Some([0, 1, 0, 0, 0, 0]));
        assert_eq!(live_mana(branch, p, multiplier_ref), None);
        settle(branch);
        assert_eq!(branch.object_ref(aura), aura_ref);
        assert!(branch.has_subtype(aura, "Cave"));
        cast(branch, spell, None);
        let cast = branch
            .state
            .stack
            .iter()
            .find(|item| item.kind == StackKind::Spell)
            .unwrap();
        assert_eq!(
            cast.mana_provenance.as_ref().unwrap().sources.as_slice(),
            &[ManaSourcePayment {
                source: aura_ref,
                amount: 1,
                cave: false,
            }]
        );
        settle(branch);
        assert_eq!(branch.state.players[p.index()].life, 21);
    }
    assert_eq!(game.snapshot(), indexed.snapshot());
    indexed.restore(&saved).unwrap();
    act(&mut indexed, action);
    assert_eq!(live_mana(&indexed, p, aura_ref), Some([0, 1, 0, 0, 0, 0]));
}

#[test]
fn separate_produced_type_choices_keep_both_exact_providers_on_rollback() {
    let _rules = fixture_rules::activate(&[(Card::Forest, &LAND), (card("Cancel"), &SPELL)]);
    let p = Player::P1;
    let mut game = scenario(p);
    game.state.players[p.index()].mana = [0; 6];
    let land = put(&mut game, p, "Forest", Zone::Battlefield);
    let first = put(&mut game, p, "Lavaleaper", Zone::Battlefield);
    let second = put(&mut game, p.other(), "Lavaleaper", Zone::Battlefield);
    let spell = put(&mut game, p, "Cancel", Zone::Hand);
    let first_ref = game.object_ref(first);
    let second_ref = game.object_ref(second);
    game.state.cast_rollback = Some(Box::new(cast_rollback::CastRollback::new(
        game.state.clone(),
        p,
        spell,
    )));
    act(&mut game, Action::AnnounceCast { card: spell });
    act(
        &mut game,
        Action::ActivateMana {
            card: land,
            ability: 0,
            payment: [0; 6],
            counter_payment: Counters::default(),
            production: BLUE_GREEN,
        },
    );
    for viewer in [p, p.other()] {
        assert_eq!(game.observe(viewer).mana_tap_bonus_source, Some(first_ref));
    }
    act(&mut game, Action::ChooseMana { production: GREEN });
    for viewer in [p, p.other()] {
        assert_eq!(game.observe(viewer).mana_tap_bonus_source, Some(second_ref));
    }
    assert_eq!(live_mana(&game, p, first_ref), Some(GREEN.map(u128::from)));
    assert_eq!(live_mana(&game, p, second_ref), None);
    assert!(game.state.cast_rollback.as_ref().unwrap().events.is_empty());
    reject(&mut game, Action::ChooseMana { production: GREEN });
    act(&mut game, Action::ChooseMana { production: BLUE });
    assert_eq!(live_mana(&game, p, second_ref), Some(BLUE.map(u128::from)));
    assert!(game.observe(p).mana_tap_bonus_source.is_none());
    let event = &game.state.cast_rollback.as_ref().unwrap().events[0];
    assert_eq!(event.tap_bonus, BLUE_GREEN);
    assert_eq!(event.tap_bonus_lots.len(), 2);
    game.finish_cast_rollback(false);
    let saved = game.snapshot();
    let mut indexed = game.clone();
    let action = Action::ChooseManaRollback { keep: true };
    let index = game
        .legal_actions()
        .iter()
        .position(|candidate| *candidate == action)
        .unwrap();
    let revision = game.revision();
    act(&mut game, action);
    indexed.apply_index(p, revision, index).unwrap();
    assert_eq!(game.snapshot(), indexed.snapshot());
    assert_eq!(game.state.players[p.index()].mana, [0, 2, 0, 0, 2, 0]);
    assert_eq!(live_mana(&game, p, first_ref), Some(GREEN.map(u128::from)));
    assert_eq!(live_mana(&game, p, second_ref), Some(BLUE.map(u128::from)));
    indexed.restore(&saved).unwrap();
    act(&mut indexed, Action::ChooseManaRollback { keep: false });
    assert_eq!(indexed.state.players[p.index()].mana, [0; 6]);
    assert_eq!(live_mana(&indexed, p, first_ref), None);
    assert_eq!(live_mana(&indexed, p, second_ref), None);
}
