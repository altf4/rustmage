//! Common Goal: target count, opposing restriction, and damage from each source.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn setup() -> (Game, CardId, CardId, CardId, CardId) {
    let mut game = scenario(Player::P0);
    let spell = put(&mut game, Player::P0, "Common Goal", Zone::Hand);
    let first = put(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let second = put(&mut game, Player::P0, "Bear Cub", Zone::Battlefield);
    let victim = put(&mut game, Player::P1, "Hill Giant", Zone::Battlefield);
    (game, spell, first, second, victim)
}

#[test]
fn common_goal_two_sources_each_deal_their_power() {
    let (mut game, spell, first, second, victim) = setup();
    act(&mut game, Action::AnnounceCast { card: spell });
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(victim),
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(first),
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(second),
        },
    );
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(first),
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(victim),
        },
    );
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(victim).zone, Zone::Graveyard);
    assert_eq!(game.object_view(first).zone, Zone::Battlefield);
    assert_eq!(game.object_view(second).zone, Zone::Battlefield);
}

#[test]
fn common_goal_one_source_is_legal_and_does_not_use_other_creature() {
    let (mut game, spell, first, second, victim) = setup();
    act(&mut game, Action::AnnounceCast { card: spell });
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(first),
        },
    );
    act(&mut game, Action::FinishTargets);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(second),
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(victim),
        },
    );
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(victim).zone, Zone::Battlefield);
    assert_eq!(game.object_view(victim).damage, 2);
}
