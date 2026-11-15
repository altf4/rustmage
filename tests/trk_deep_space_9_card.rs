//! Deep Space 9's exact ETB/upkeep batches and live Station bonus.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

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
    panic!("Deep Space 9 resolution budget");
}

#[test]
fn enters_and_upkeep_exile_separate_top_four_batches_then_enter_only_permanents() {
    let mut game = scenario(Player::P0);
    game.state.players[0].library.clear();
    let second_land = put(&mut game, Player::P0, "Plains", Zone::Library);
    let second_spell = put(&mut game, Player::P0, "Shock", Zone::Library);
    let third_land = put(&mut game, Player::P0, "Mountain", Zone::Library);
    let third_spell = put(&mut game, Player::P0, "Opt", Zone::Library);
    let first_land = put(&mut game, Player::P0, "Forest", Zone::Library);
    let creature = put(&mut game, Player::P0, "Grizzly Bears", Zone::Library);
    let first_spell = put(&mut game, Player::P0, "Shock", Zone::Library);
    let other_land = put(&mut game, Player::P0, "Evolving Wilds", Zone::Library);
    let station = put(&mut game, Player::P0, "Deep Space 9", Zone::Hand);
    cast(&mut game, station, None);
    drain(&mut game);
    for card in [first_land, creature, other_land] {
        assert_eq!(game.object_view(card).zone, Zone::Battlefield);
    }
    assert_eq!(game.object_view(first_spell).zone, Zone::Exile);
    for card in [second_land, second_spell, third_land, third_spell] {
        assert_eq!(game.object_view(card).zone, Zone::Library);
    }
    let checkpoint = game.snapshot();
    let mut replay = game.clone();
    for branch in [&mut game, &mut replay] {
        branch.begin_step(Step::Upkeep);
        branch.prepare_triggers();
        drain(branch);
    }
    assert_eq!(game.snapshot(), replay.snapshot());
    replay.restore(&checkpoint).unwrap();
    assert_eq!(replay.snapshot(), checkpoint);
    for card in [second_land, third_land] {
        assert_eq!(game.object_view(card).zone, Zone::Battlefield);
    }
    for card in [second_spell, third_spell] {
        assert_eq!(game.object_view(card).zone, Zone::Exile);
    }
    assert!(game.state.players[0].library.is_empty());
}

#[test]
fn station_taps_only_another_own_creature_and_grants_live_nine_charge_bonus() {
    let mut game = scenario(Player::P0);
    let station = put(&mut game, Player::P0, "Deep Space 9", Zone::Battlefield);
    let own = put(&mut game, Player::P0, "Bear Cub", Zone::Battlefield);
    let donors: Vec<_> = (0..3)
        .map(|_| put(&mut game, Player::P0, "Hill Giant", Zone::Battlefield))
        .collect();
    let enemy = put(&mut game, Player::P1, "Grizzly Bears", Zone::Battlefield);
    assert_eq!(stats(&game, own), (2, 2));
    for (index, donor) in donors.into_iter().enumerate() {
        act(
            &mut game,
            Action::AnnounceActivate {
                card: station,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        if index == 0 {
            let enemy_ref = game.object_ref(enemy);
            let station_ref = game.object_ref(station);
            reject(
                &mut game,
                Action::ChooseAdditionalCost { object: enemy_ref },
            );
            reject(
                &mut game,
                Action::ChooseAdditionalCost {
                    object: station_ref,
                },
            );
        }
        let donor_ref = game.object_ref(donor);
        act(
            &mut game,
            Action::ChooseAdditionalCost { object: donor_ref },
        );
        pay(&mut game);
        drain(&mut game);
        assert_eq!(
            game.counter_count(station, CounterKind::Charge),
            (index as u64 + 1) * 3
        );
        assert_eq!(stats(&game, own), if index == 2 { (5, 5) } else { (2, 2) });
    }
    assert_eq!(stats(&game, enemy), (2, 2));
    game.move_object(station, Zone::Graveyard);
    assert_eq!(stats(&game, own), (2, 2));
}
