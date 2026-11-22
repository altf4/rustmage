//! D-7 Battle Cruiser station levels and temporary Klingon Warrior token.
use crate::support::*;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};

fn drain(game: &mut Game) {
    for _ in 0..48 {
        if game.state.pending == Pending::Priority
            && game.state.stack.is_empty()
            && game.state.pending_triggers.is_empty()
            && game.state.resolving.is_none()
        {
            return;
        }
        choose(game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("resolution budget");
}

fn station(game: &mut Game, ship: CardId, creature: CardId) {
    act(
        game,
        Action::AnnounceActivate {
            card: ship,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    let object = game.object_ref(creature);
    act(game, Action::ChooseAdditionalCost { object });
    pay(game);
    drain(game);
}

fn begin_step(game: &mut Game, step: Step) {
    game.begin_step(step);
    game.advance_replacements(false);
    game.give_priority();
    drain(game);
}

fn klingons(game: &Game) -> Vec<CardId> {
    (0..game.state.objects.len())
        .map(|id| id as CardId)
        .filter(|&id| {
            game.object_view(id).zone == Zone::Battlefield
                && game.object_view(id).card == Card(6010)
        })
        .collect()
}

#[test]
fn d7_station_uses_actual_tapped_power_and_levels_are_live() {
    let mut game = scenario(Player::P0);
    let ship = put(
        &mut game,
        Player::P0,
        "D-7 Battle Cruiser",
        Zone::Battlefield,
    );
    let three = put(&mut game, Player::P0, "Hill Giant", Zone::Battlefield);
    let other_three = put(&mut game, Player::P0, "Hill Giant", Zone::Battlefield);
    let two = put(&mut game, Player::P0, "Bear Cub", Zone::Battlefield);
    assert!(!game.is_creature(ship));
    begin_step(&mut game, Step::BeginCombat);
    assert!(klingons(&game).is_empty());
    game.state.step = Step::Main1;
    station(&mut game, ship, three);
    assert_eq!(game.counter_count(ship, CounterKind::Charge), 3);
    assert!(game.object_view(three).tapped);
    assert!(!game.is_creature(ship));
    begin_step(&mut game, Step::BeginCombat);
    let created = klingons(&game);
    assert_eq!(created.len(), 1);
    assert!(
        game.object_view(created[0])
            .keywords
            .contains(Keywords::HASTE)
    );
    begin_step(&mut game, Step::End);
    assert!(klingons(&game).is_empty());

    game.state.step = Step::Main1;
    station(&mut game, ship, other_three);
    station(&mut game, ship, two);
    assert_eq!(game.counter_count(ship, CounterKind::Charge), 8);
    assert!(game.is_creature(ship));
    assert_eq!(stats(&game, ship), (3, 3));
    assert!(game.object_view(ship).keywords.contains(Keywords::FLYING));
}
