//! Branching Timeline's distinct-land count and forced revealed permanents.
mod support;

use rustmage as _;
use rustmage_core::engine::*;
use support::*;

#[test]
fn branching_timeline_puts_all_revealed_permanents_onto_battlefield_tapped() {
    let mut game = scenario(Player::P0);
    game.state.players[0].library.clear();
    put(&mut game, Player::P0, "Forest", Zone::Battlefield);
    put(&mut game, Player::P0, "Forest", Zone::Battlefield);
    put(&mut game, Player::P0, "Island", Zone::Battlefield);
    put(&mut game, Player::P0, "Mountain", Zone::Battlefield);

    let below = put(&mut game, Player::P0, "Plains", Zone::Library);
    let land = put(&mut game, Player::P0, "Island", Zone::Library);
    let creature = put(&mut game, Player::P0, "Grizzly Bears", Zone::Library);
    let instant = put(&mut game, Player::P0, "Shock", Zone::Library);
    let spell = put(&mut game, Player::P0, "Branching Timeline", Zone::Hand);
    cast(&mut game, spell, None);
    for _ in 0..24 {
        if matches!(game.state.pending, Pending::EffectLibrarySelection { .. }) {
            break;
        }
        choose(&mut game, |a| {
            matches!(a, Action::Pass | Action::PutTrigger { .. })
        });
    }
    assert!(matches!(
        game.state.pending,
        Pending::EffectLibrarySelection { .. }
    ));
    assert!(!game
        .legal_actions()
        .contains(&Action::SelectLibrary { card: instant }));
    act(&mut game, Action::SelectLibrary { card: land });
    act(&mut game, Action::SelectLibrary { card: creature });
    if game
        .legal_actions()
        .contains(&Action::FinishLibrarySelection)
    {
        act(&mut game, Action::FinishLibrarySelection);
    }
    settle(&mut game);

    assert_eq!(game.object_view(land).zone, Zone::Battlefield);
    assert_eq!(game.object_view(creature).zone, Zone::Battlefield);
    assert!(game.object_view(land).tapped);
    assert!(game.object_view(creature).tapped);
    assert_eq!(game.object_view(instant).zone, Zone::Library);
    assert_eq!(game.object_view(below).zone, Zone::Library);
    assert_eq!(game.state.players[0].library.len(), 2);
}
