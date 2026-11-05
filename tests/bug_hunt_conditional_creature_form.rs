mod support;
use rustmage as _;
use rustmage_core::{cards::CounterKind, engine::*};
use support::*;

#[test]
fn conditional_creature_form_replaces_planeswalker_type_until_opponents_turn() {
    let player = Player::P0;
    let mut game = scenario(player);
    let kaito = put(&mut game, player, "Kaito, Bane of Nightmares", Zone::Hand);
    let pyroclasm = put(&mut game, player, "Pyroclasm", Zone::Hand);
    cast(&mut game, kaito, None);
    settle(&mut game);
    let creature = game.object_view(kaito);
    assert_eq!(creature.kinds, 1);
    assert!(creature.has_subtype("Ninja"));
    assert!(!creature.has_subtype("Kaito"));
    assert_eq!(stats(&game, kaito), (3, 4));
    assert_eq!(game.counter_count(kaito, CounterKind::Loyalty), 4);

    cast(&mut game, pyroclasm, None);
    settle(&mut game);
    let view = game.object_view(kaito);
    assert_eq!((view.zone, view.damage), (Zone::Battlefield, 2));
    assert_eq!(game.counter_count(kaito, CounterKind::Loyalty), 4);

    game.state.active = player.other();
    let planeswalker = game.object_view(kaito);
    assert_eq!(planeswalker.kinds, 64);
    assert!(planeswalker.has_subtype("Kaito"));
    assert!(!planeswalker.has_subtype("Ninja"));
}

#[test]
fn conditional_artifact_creature_form_retains_spacecraft_subtype() {
    let player = Player::P0;
    let mut game = scenario(player);
    let gunship = put(&mut game, player, "Warmaker Gunship", Zone::Battlefield);
    assert_eq!(game.object_view(gunship).kinds, 16);
    game.add_counter(gunship, CounterKind::Charge, 6);
    let creature = game.object_view(gunship);
    assert_eq!(creature.kinds, 17);
    assert!(creature.has_subtype("Spacecraft"));
}
