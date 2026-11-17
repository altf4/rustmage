//! The graveyard source goes below its owner's library before its ability resolves.
use crate::support::*;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::engine::zone_replacements;
use rustmage_core::{cards::*, engine::*};

const GRAVEYARD_RETURN: CardRules = CardRules {
    effect: Effect::Creature(3, 1),
    activated: &[ActivatedAbility {
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::GraveyardBottomLibrary,
        cycling: false,
        exhaust: false,
        once_per_turn: false,
        sorcery: false,
        cost: ActivationCost {
            generic: 2,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 2)],
    }],
    ..CardRules::EMPTY
};

#[test]
fn graveyard_source_is_exact_bottom_cost_with_paid_timing_and_owned_rollback() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &GRAVEYARD_RETURN)]);
    let mut game = scenario(Player::P0);
    let source = put(&mut game, Player::P0, "Grizzly Bears", Zone::Graveyard);
    let foreign = put(&mut game, Player::P1, "Grizzly Bears", Zone::Graveyard);
    let old_bottom = game.state.players[0].library[0];
    let action = Action::AnnounceActivate {
        card: source,
        ability: 0,
        counter_payment: Counters::default(),
    };
    game.state.priority = Player::P1;
    reject(&mut game, action.clone());
    game.state.priority = Player::P0;
    reject(
        &mut game,
        Action::AnnounceActivate {
            card: foreign,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    game.state.players[0].mana = [0; 6];
    admit(&game, action.clone());
    assert_eq!(game.object_view(source).zone, Zone::Graveyard);
    game.state.players[0].mana = [8; 6];

    let checkpoint = game.snapshot();
    let revision = game.revision();
    let index = game
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    let mut indexed = game.clone();
    indexed.apply_index(Player::P0, revision, index).unwrap();
    act(&mut game, action);
    assert_eq!(indexed.snapshot(), game.snapshot());
    indexed.restore(&checkpoint).unwrap();
    assert_eq!(
        indexed.apply_index(Player::P0, revision, index),
        Err(Error::StaleRevision)
    );

    pay(&mut game);
    assert_eq!(game.object_view(source).zone, Zone::Library);
    assert_eq!(game.state.players[0].library[0], source);
    assert_eq!(game.state.players[0].library[1], old_bottom);
    assert_eq!(game.state.players[0].life, 20);
    settle(&mut game);
    assert_eq!(game.state.players[0].life, 22);
    assert_eq!(game.object_view(source).zone, Zone::Library);
}

#[test]
fn prevented_source_move_cannot_be_announced_or_paid() {
    struct ResetPrevention;
    impl Drop for ResetPrevention {
        fn drop(&mut self) {
            zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|rule| rule.set(None));
        }
    }
    let _reset = ResetPrevention;
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &GRAVEYARD_RETURN)]);
    let mut game = scenario(Player::P0);
    let source = put(&mut game, Player::P0, "Grizzly Bears", Zone::Graveyard);
    let exact = game.object_ref(source);
    let action = Action::AnnounceActivate {
        card: source,
        ability: 0,
        counter_payment: Counters::default(),
    };
    zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|rule| rule.set(Some(exact)));
    reject(&mut game, action.clone());
    assert_eq!(game.object_view(source).zone, Zone::Graveyard);
    zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|rule| rule.set(None));

    act(&mut game, action);
    let pay_action = game
        .legal_actions()
        .into_iter()
        .find(|action| matches!(action, Action::PayCosts { .. }))
        .expect("the unblocked cost can be paid");
    let before = game.snapshot();
    zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|rule| rule.set(Some(exact)));
    reject(&mut game, pay_action);
    assert_eq!(game.snapshot(), before);
    assert_eq!(game.object_view(source).zone, Zone::Graveyard);
    assert!(game.state.stack.is_empty());
    zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|rule| rule.set(None));
    pay(&mut game);
    assert_eq!(game.object_view(source).zone, Zone::Library);
    settle(&mut game);
    assert_eq!(game.state.players[0].life, 22);
}
