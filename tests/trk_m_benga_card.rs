//! Dr. Joseph M'Benga's authored rules and targeting interactions.
mod support;

use rustmage as _;
use rustmage_core::engine::replacements::{CounterPlacement, ResumeMode};
use rustmage_core::{cards::*, engine::*};
use support::*;

fn place(game: &mut Game, card: CardId, by: Player, amount: Counters) {
    let view = game.object_view(card);
    game.stage_counter_placements(
        vec![CounterPlacement {
            object: view.reference,
            placed_by: by,
            card: view.card,
            owner: view.owner,
            controller: view.controller,
            protector: view.battle_protector,
            subtypes: view.granted_subtypes,
            kind: view.kinds,
            power: view.power,
            amount,
            effect: true,
            entry_index: None,
            private: false,
        }],
        ResumeMode::Pending,
    );
    game.advance_replacements(true);
}

#[test]
fn different_counter_kinds_double_and_only_other_countered_creatures_have_hexproof() {
    let mut game = scenario(Player::P0);
    let doctor = put(&mut game, Player::P0, "Dr. Joseph M'Benga", Zone::Battlefield);
    let own = put(&mut game, Player::P0, "Bear Cub", Zone::Battlefield);
    let enemy = put(&mut game, Player::P1, "Hill Giant", Zone::Battlefield);
    let artifact = put(&mut game, Player::P0, "Short Bow", Zone::Battlefield);
    assert_eq!(stats(&game, doctor), (3, 3));
    assert!(!game.keywords(own).contains(Keywords::HEXPROOF));

    let mut mixed = Counters::of(CounterKind::PlusOne, 1);
    mixed.add(CounterKind::Stun, 1);
    place(&mut game, own, Player::P0, mixed);
    assert_eq!(game.counter_count(own, CounterKind::PlusOne), 2);
    assert_eq!(game.counter_count(own, CounterKind::Stun), 2);
    assert!(game.keywords(own).contains(Keywords::HEXPROOF));

    place(&mut game, doctor, Player::P0, Counters::of(CounterKind::PlusOne, 1));
    assert_eq!(game.counter_count(doctor, CounterKind::PlusOne), 2);
    assert!(!game.keywords(doctor).contains(Keywords::HEXPROOF));
    place(&mut game, enemy, Player::P0, Counters::of(CounterKind::PlusOne, 1));
    place(&mut game, artifact, Player::P0, Counters::of(CounterKind::PlusOne, 1));
    assert_eq!(game.counter_count(enemy, CounterKind::PlusOne), 1);
    assert_eq!(game.counter_count(artifact, CounterKind::PlusOne), 1);
    place(&mut game, own, Player::P1, Counters::of(CounterKind::Flying, 1));
    assert_eq!(game.counter_count(own, CounterKind::Flying), 1);

    let saved = game.snapshot();
    let removal = put(&mut game, Player::P1, "Murder", Zone::Hand);
    act(&mut game, Action::Pass);
    act(&mut game, Action::AnnounceCast { card: removal });
    reject(&mut game, Action::ChooseTarget { target: Target::Card(own) });
    act(&mut game, Action::ChooseTarget { target: Target::Card(doctor) });
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(doctor).zone, Zone::Graveyard);
    assert!(!game.keywords(own).contains(Keywords::HEXPROOF));
    game.restore(&saved).unwrap();
    assert_eq!(game.object_view(doctor).zone, Zone::Battlefield);
    assert!(game.keywords(own).contains(Keywords::HEXPROOF));
}
