//! Daniels's actual enter trigger and graveyard source cost.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

#[test]
fn entering_daniels_returns_only_an_owned_creature_card() {
    let mut game = scenario(Player::P0);
    let own = put(&mut game, Player::P0, "Bear Cub", Zone::Graveyard);
    let foreign = put(&mut game, Player::P1, "Hill Giant", Zone::Graveyard);
    let daniels = put(&mut game, Player::P0, "Daniels, Temporal Agent", Zone::Hand);
    cast(&mut game, daniels, None);
    for _ in 0..24 {
        if game.legal_actions().contains(&Action::ChooseTarget {
            target: Target::Card(own),
        }) {
            break;
        }
        choose(&mut game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(foreign),
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(own),
        },
    );
    if game.legal_actions().contains(&Action::FinishStackTargets) {
        act(&mut game, Action::FinishStackTargets);
    }
    settle(&mut game);
    assert_eq!(stats(&game, daniels), (3, 1));
    assert_eq!(game.object_view(daniels).zone, Zone::Battlefield);
    assert_eq!(game.object_view(own).zone, Zone::Hand);
    assert_eq!(game.object_view(foreign).zone, Zone::Graveyard);
}

#[test]
fn graveyard_daniels_pays_exact_bottom_before_gaining_life() {
    let mut game = scenario(Player::P0);
    let daniels = put(
        &mut game,
        Player::P0,
        "Daniels, Temporal Agent",
        Zone::Graveyard,
    );
    let foreign = put(
        &mut game,
        Player::P1,
        "Daniels, Temporal Agent",
        Zone::Graveyard,
    );
    let old_bottom = game.state.players[0].library[0];
    let action = Action::AnnounceActivate {
        card: daniels,
        ability: 0,
        counter_payment: Counters::default(),
    };
    reject(
        &mut game,
        Action::AnnounceActivate {
            card: foreign,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    game.state.players[0].mana = [0; 6];
    admit(&game, action.clone());
    game.state.players[0].mana = [8; 6];

    let snapshot = game.snapshot();
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
    indexed.restore(&snapshot).unwrap();
    assert_eq!(
        indexed.apply_index(Player::P0, revision, index),
        Err(Error::StaleRevision)
    );

    pay(&mut game);
    assert_eq!(game.object_view(daniels).zone, Zone::Library);
    assert_eq!(game.state.players[0].library[0], daniels);
    assert_eq!(game.state.players[0].library[1], old_bottom);
    assert_eq!(game.state.players[0].life, 20);
    settle(&mut game);
    assert_eq!(game.state.players[0].life, 22);
}
