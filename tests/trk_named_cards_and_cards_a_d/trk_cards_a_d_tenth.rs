//! Star Trek A–D delayed death and distinct-land sorcery scenarios.
use crate::support::*;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};

fn owned(game: &mut Game, name: &str) -> CardId {
    spawn(game, Player::P0, Card::from_name(name).unwrap(), Zone::Hand)
}

#[test]
fn a_good_day_to_die_draws_only_when_its_exact_target_dies_this_turn() {
    let mut game = scenario(Player::P0);
    let target = put(&mut game, Player::P1, "Bear Cub", Zone::Battlefield);
    let spell = owned(&mut game, "A Good Day to Die");
    cast(&mut game, spell, Some(Target::Card(target)));
    settle(&mut game);
    assert_eq!(stats(&game, target), (4, 2));
    assert_eq!(game.hand_count(Player::P0), 0);
    let kill = put(&mut game, Player::P0, "Shock", Zone::Hand);
    cast(&mut game, kill, Some(Target::Card(target)));
    settle(&mut game);
    assert_eq!(game.object_view(target).zone, Zone::Graveyard);
    assert_eq!(game.hand_count(Player::P0), 1);
}

#[test]
fn deep_space_scan_counts_distinct_lands_and_grants_temporary_unlimited_hand() {
    let mut game = scenario(Player::P0);
    for land in ["Forest", "Island", "Mountain"] {
        put(&mut game, Player::P0, land, Zone::Battlefield);
    }
    put(&mut game, Player::P1, "Plains", Zone::Battlefield);
    let spell = owned(&mut game, "Deep Space Scan");
    cast(&mut game, spell, None);
    settle(&mut game);
    assert_eq!(game.state.players[0].cards_drawn_this_turn, 3);
    assert_eq!(game.hand_count(Player::P0), 3);
    assert_eq!(game.maximum_hand_size(Player::P0), None);
    assert_ne!(game.maximum_hand_size(Player::P1), None);
}
