//! Branching Timeline's distinct-land count and forced revealed permanents.
use crate::support::*;

use rustmage as _;
use rustmage_core::engine::*;

#[test]
fn branching_timeline_puts_all_revealed_permanents_onto_battlefield_tapped() {
    let mut game = scenario(Player::P0);
    game.state.players[0].library.clear();
    put(&mut game, Player::P0, "Forest", Zone::Battlefield);
    put(&mut game, Player::P0, "Forest", Zone::Battlefield);
    put(&mut game, Player::P0, "Island", Zone::Battlefield);
    put(&mut game, Player::P0, "Mountain", Zone::Battlefield);
    put(&mut game, Player::P0, "Plains", Zone::Battlefield);

    let below = put(&mut game, Player::P0, "Plains", Zone::Library);
    let land = put(&mut game, Player::P0, "Island", Zone::Library);
    let creature = put(&mut game, Player::P0, "Grizzly Bears", Zone::Library);
    let other_instant = put(&mut game, Player::P0, "Opt", Zone::Library);
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
    let revealed = game.observe(Player::P1).library_selection.unwrap();
    let ids: Vec<_> = revealed
        .remaining
        .iter()
        .map(|o| o.reference.card)
        .collect();
    assert_eq!(ids.len(), 4);
    for id in [land, creature, other_instant, instant] {
        assert!(ids.contains(&id));
    }
    assert!(!ids.contains(&below));
    reject(&mut game, Action::FinishLibrarySelection);
    assert!(
        !game
            .legal_actions()
            .contains(&Action::SelectLibrary { card: instant })
    );
    let checkpoint = game.snapshot();
    let revision = game.revision();
    let select = Action::SelectLibrary { card: land };
    let index = game
        .legal_actions()
        .iter()
        .position(|a| *a == select)
        .unwrap();
    let mut indexed = game.clone();
    act(&mut game, select.clone());
    indexed.apply_index(Player::P0, revision, index).unwrap();
    assert_eq!(game.snapshot(), indexed.snapshot());
    indexed.restore(&checkpoint).unwrap();
    assert_eq!(
        indexed.apply_index(Player::P0, revision, index),
        Err(Error::StaleRevision)
    );
    act(&mut indexed, select);
    assert_eq!(game.snapshot(), indexed.snapshot());
    reject(&mut game, Action::FinishLibrarySelection);
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
    assert_eq!(game.object_view(other_instant).zone, Zone::Library);
    assert_eq!(game.object_view(below).zone, Zone::Library);
    assert_eq!(game.state.players[0].library.len(), 3);
    let mut bottom = game.state.players[0].library[..2].to_vec();
    bottom.sort_unstable();
    let mut wanted = vec![instant, other_instant];
    wanted.sort_unstable();
    assert_eq!(bottom, wanted);
    assert_eq!(game.state.players[0].library[2], below);
}
