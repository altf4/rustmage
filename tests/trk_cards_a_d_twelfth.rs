//! Deanna Troi's entry replacement applies to other controlled creatures only.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

#[test]
fn deanna_troi_gives_other_controlled_creature_entries_one_counter() {
    let mut game = scenario(Player::P0);
    let deanna = spawn(
        &mut game,
        Player::P0,
        Card::from_name("Deanna Troi, Psychic Counsel").unwrap(),
        Zone::Hand,
    );
    cast(&mut game, deanna, None);
    settle(&mut game);
    assert_eq!(game.counter_count(deanna, CounterKind::PlusOne), 0);
    let ally = put(&mut game, Player::P0, "Grizzly Bears", Zone::Hand);
    cast(&mut game, ally, None);
    settle(&mut game);
    assert_eq!(game.counter_count(ally, CounterKind::PlusOne), 1);
    assert_eq!(stats(&game, ally), (3, 3));
}
