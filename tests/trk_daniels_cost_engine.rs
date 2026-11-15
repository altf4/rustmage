//! The graveyard source goes below its owner's library before its ability resolves.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

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
    reject(&mut game, action.clone());
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
