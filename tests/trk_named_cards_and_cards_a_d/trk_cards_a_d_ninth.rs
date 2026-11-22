//! Star Trek A–D artifact entry and modal card scenarios.
use crate::support::*;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};

fn owned(game: &mut Game, name: &str, zone: Zone) -> CardId {
    spawn(game, Player::P0, Card::from_name(name).unwrap(), zone)
}
fn drain(game: &mut Game) {
    for _ in 0..50 {
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
    panic!("resolution budget");
}
fn choose_mode(game: &mut Game, mode: u16) {
    for _ in 0..30 {
        if game.legal_actions().contains(&Action::ChooseMode { mode }) {
            act(game, Action::ChooseMode { mode });
            return;
        }
        choose(game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("mode unavailable");
}
fn borg_count(game: &Game) -> usize {
    game.state
        .objects
        .iter()
        .filter(|object| {
            object.zone == Zone::Battlefield
                && object.controller == Player::P0
                && object.card == Card(6007)
        })
        .count()
}

#[test]
fn conquest_mission_counts_nontoken_artifacts_and_switches_at_six() {
    let mut game = scenario(Player::P0);
    let mission = owned(&mut game, "Conquest Mission", Zone::Battlefield);
    let artifact = put(&mut game, Player::P0, "Tricorder", Zone::Hand);
    cast(&mut game, artifact, None);
    drain(&mut game);
    assert_eq!(game.counter_count(mission, CounterKind::Quest), 1);
    assert_eq!(borg_count(&game), 1);
    game.set_counters(mission, Counters::of(CounterKind::Quest, 5));
    let second = put(&mut game, Player::P0, "Tricorder", Zone::Hand);
    cast(&mut game, second, None);
    drain(&mut game);
    assert_eq!(game.counter_count(mission, CounterKind::Quest), 6);
    assert_eq!(borg_count(&game), 3);
}

#[test]
fn consider_the_prime_directive_observe_is_optional_and_interfere_counters() {
    let mut observe = scenario(Player::P0);
    let target = put(&mut observe, Player::P1, "Grizzly Bears", Zone::Battlefield);
    let spell = owned(&mut observe, "Consider the Prime Directive", Zone::Hand);
    act(&mut observe, Action::AnnounceCast { card: spell });
    choose_mode(&mut observe, 1);
    act(
        &mut observe,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    if observe.legal_actions().contains(&Action::FinishTargets) {
        act(&mut observe, Action::FinishTargets);
    }
    pay(&mut observe);
    drain(&mut observe);
    assert!(observe.object_view(target).tapped);
    assert_eq!(observe.counter_count(target, CounterKind::Stun), 1);
    assert_eq!(observe.hand_count(Player::P0), 1);

    let mut interfere = scenario(Player::P0);
    let pending = put(&mut interfere, Player::P0, "Shock", Zone::Hand);
    cast(&mut interfere, pending, Some(Target::Player(Player::P1)));
    let counter = owned(&mut interfere, "Consider the Prime Directive", Zone::Hand);
    act(&mut interfere, Action::AnnounceCast { card: counter });
    choose_mode(&mut interfere, 0);
    act(
        &mut interfere,
        Action::ChooseTarget {
            target: Target::Spell(pending),
        },
    );
    if interfere.legal_actions().contains(&Action::FinishTargets) {
        act(&mut interfere, Action::FinishTargets);
    }
    pay(&mut interfere);
    for _ in 0..40 {
        if interfere
            .legal_actions()
            .contains(&Action::DeclineResolutionMana)
        {
            break;
        }
        choose(&mut interfere, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    act(&mut interfere, Action::DeclineResolutionMana);
    drain(&mut interfere);
    assert_eq!(interfere.state.players[1].life, 20);
    assert_eq!(interfere.object_view(pending).zone, Zone::Graveyard);
}

#[test]
fn constable_odo_taps_and_stuns_or_investigates_twice() {
    for mode in [0, 1] {
        let mut game = scenario(Player::P0);
        let enemy = put(&mut game, Player::P1, "Grizzly Bears", Zone::Battlefield);
        let odo = owned(&mut game, "Constable Odo", Zone::Hand);
        cast(&mut game, odo, None);
        choose_mode(&mut game, mode);
        if mode == 0 {
            act(
                &mut game,
                Action::ChooseTarget {
                    target: Target::Card(enemy),
                },
            );
            if game.legal_actions().contains(&Action::FinishTargets) {
                act(&mut game, Action::FinishTargets);
            }
        }
        drain(&mut game);
        if mode == 0 {
            assert!(game.object_view(enemy).tapped);
            assert_eq!(game.counter_count(enemy, CounterKind::Stun), 2);
        } else {
            assert_eq!(
                game.state
                    .objects
                    .iter()
                    .filter(|object| object.zone == Zone::Battlefield
                        && object.controller == Player::P0
                        && object.card == Card(5296))
                    .count(),
                2
            );
        }
    }
}
