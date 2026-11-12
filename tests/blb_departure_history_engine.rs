//! Graveyard departures and Food sacrifices retain committed turn history.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const REQUIREMENT: StatValue = StatValue::Sum(&[
    StatValue::AtLeast {
        value: &StatValue::CardsLeftGraveyardThisTurn,
        minimum: 3,
    },
    StatValue::FoodSacrificedThisTurn,
]);

const OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[ActivatedAbility {
        guard: ActivationGuard::StatAtLeast {
            value: REQUIREMENT,
            minimum: 1,
        },
        cost: ActivationCost {
            tap: true,
            life: 1,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::Draw(PlayerSet::Controller, 1)],
        origin: ActivationOrigin::Battlefield,
        exhaust: false,
        cycling: false,
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};

fn activation(action: &Action, source: CardId) -> bool {
    matches!(*action,
        Action::Activate { card, .. } | Action::AnnounceActivate { card, .. } if card == source
    )
}

fn available(game: &Game, source: CardId) -> bool {
    game.legal_actions()
        .iter()
        .any(|action| activation(action, source))
}

#[test]
fn repeated_departures_from_the_same_graveyard_count_without_other_seat_entries() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &OBSERVER)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let first = put(&mut game, player, "Hill Giant", Zone::Graveyard);
    let opposing = put(&mut game, player.other(), "Hill Giant", Zone::Graveyard);
    game.move_object(first, Zone::Hand);
    game.move_object(first, Zone::Graveyard);
    game.move_object(first, Zone::Exile);
    game.move_object(opposing, Zone::Hand);

    let observer = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    assert!(!available(&game, observer));

    let third = put(&mut game, player, "Healer's Hawk", Zone::Graveyard);
    game.move_object(third, Zone::Hand);
    assert!(available(&game, observer));
    choose(&mut game, |action| activation(action, observer));
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(&mut game);
    }
    settle(&mut game);
    assert!(game.object_view(observer).tapped);
    assert_eq!(game.state.players[player.index()].life, 19);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
}

#[test]
fn sacrificing_a_food_counts_even_if_it_is_exiled_instead_of_dying() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &OBSERVER)]);
    let player = Player::P1;
    let mut game = scenario(player);
    let observer = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let other_food = put(&mut game, player.other(), "Gingerbrute", Zone::Battlefield);
    game.sacrifice_batch(&[(player.other(), other_food)]);
    assert!(!available(&game, observer));

    let food = put(&mut game, player, "Gingerbrute", Zone::Battlefield);
    game.add_counter(food, CounterKind::Finality, 1);
    let mut destroyed = game.clone();
    destroyed.move_object(food, Zone::Graveyard);
    assert!(!available(&destroyed, observer));

    game.sacrifice_batch(&[(player, food)]);
    assert_eq!(game.object_view(food).zone, Zone::Exile);
    assert!(available(&game, observer));
    assert_eq!(
        game.state.players[player.index()].cards_left_graveyard_this_turn,
        0
    );
}
