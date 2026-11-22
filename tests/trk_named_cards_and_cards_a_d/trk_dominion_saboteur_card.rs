//! Dominion Saboteur's printed optional copy choice and donor counter snapshot.
use crate::support::*;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};

fn wait_copy(game: &mut Game) {
    for _ in 0..8 {
        if matches!(game.state.pending, Pending::EntryCopy { .. }) {
            return;
        }
        assert_eq!(game.state.pending, Pending::Priority);
        act(game, Action::Pass);
    }
    panic!("Dominion Saboteur did not offer its entry copy choice");
}

#[test]
fn optional_creature_or_artifact_copy_is_nonlegendary_with_donor_counters() {
    let mut game = scenario(Player::P0);
    let angel = put(
        &mut game,
        Player::P1,
        "Giada, Font of Hope",
        Zone::Battlefield,
    );
    game.set_counters(
        angel,
        Counters::of(CounterKind::PlusOne, 2).with(CounterKind::Stun, 1),
    );
    let artifact = put(&mut game, Player::P1, "Short Bow", Zone::Battlefield);
    game.set_counters(artifact, Counters::of(CounterKind::Charge, 3));
    let land = put(&mut game, Player::P1, "Forest", Zone::Battlefield);
    let saboteur = put(&mut game, Player::P0, "Dominion Saboteur", Zone::Hand);
    cast(&mut game, saboteur, None);
    wait_copy(&mut game);

    let land_ref = game.object_ref(land);
    reject(
        &mut game,
        Action::ChooseEntryCopy {
            object: Some(land_ref),
        },
    );

    let mut declined = game.clone();
    act(&mut declined, Action::ChooseEntryCopy { object: None });
    settle(&mut declined);
    assert_eq!(declined.object_view(saboteur).name, "Dominion Saboteur");
    assert_eq!(stats(&declined, saboteur), (3, 3));
    assert_eq!(declined.counter_count(saboteur, CounterKind::PlusOne), 0);

    let mut artifact_copy = game.clone();
    let artifact_ref = artifact_copy.object_ref(artifact);
    act(
        &mut artifact_copy,
        Action::ChooseEntryCopy {
            object: Some(artifact_ref),
        },
    );
    settle(&mut artifact_copy);
    let copied_artifact = artifact_copy.object_view(saboteur);
    assert_eq!(copied_artifact.name, "Short Bow");
    assert!(!copied_artifact.is_creature());
    assert_eq!(
        artifact_copy.counter_count(saboteur, CounterKind::Charge),
        3
    );

    let angel_ref = game.object_ref(angel);
    act(
        &mut game,
        Action::ChooseEntryCopy {
            object: Some(angel_ref),
        },
    );
    settle(&mut game);
    let copied_creature = game.object_view(saboteur);
    assert_eq!(copied_creature.name, "Giada, Font of Hope");
    assert!(!copied_creature.is_legendary());
    assert_eq!(stats(&game, saboteur), (4, 4));
    assert_eq!(game.counter_count(saboteur, CounterKind::PlusOne), 2);
    assert_eq!(game.counter_count(saboteur, CounterKind::Stun), 1);
    assert!(
        game.keywords(saboteur)
            .contains(Keywords::FLYING.union(Keywords::VIGILANCE))
    );
}
