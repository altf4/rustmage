//! Star Trek A–D Saga, activation and landfall scenarios.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn owned(game: &mut Game, player: Player, name: &str, zone: Zone) -> CardId {
    spawn(game, player, Card::from_name(name).unwrap(), zone)
}

fn drain(game: &mut Game, targets: &[CardId]) {
    let mut next = 0;
    for _ in 0..60 {
        if game.state.pending == Pending::Priority
            && game.state.stack.is_empty()
            && game.state.pending_triggers.is_empty()
            && game.state.resolving.is_none()
        {
            return;
        }
        let legal = game.legal_actions();
        if next < targets.len() {
            let action = Action::ChooseTarget {
                target: Target::Card(targets[next]),
            };
            if legal.contains(&action) {
                act(game, action);
                next += 1;
                continue;
            }
        }
        if legal.contains(&Action::FinishTargets) {
            act(game, Action::FinishTargets);
            continue;
        }
        choose(game, |a| {
            matches!(a, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("resolution budget");
}

#[test]
fn amok_time_chapters_fight_gain_life_and_return_a_creature() {
    let mut game = scenario(Player::P0);
    let fighter = put(&mut game, Player::P0, "Hill Giant", Zone::Battlefield);
    let victim = put(&mut game, Player::P1, "Grizzly Bears", Zone::Battlefield);
    let survivor = put(&mut game, Player::P0, "Bear Cub", Zone::Graveyard);
    let saga = owned(&mut game, Player::P0, "Amok Time", Zone::Hand);
    cast(&mut game, saga, None);
    drain(&mut game, &[fighter, victim]);
    assert_eq!(game.object_view(victim).zone, Zone::Graveyard);
    assert_eq!(game.object_view(fighter).zone, Zone::Battlefield);
    assert_eq!(game.counter_count(saga, CounterKind::Lore), 1);
    game.state.turn += 2;
    game.begin_step(Step::Main1);
    game.give_priority();
    drain(&mut game, &[]);
    assert_eq!(game.state.players[0].life, 23);
    game.state.turn += 2;
    game.begin_step(Step::Main1);
    game.give_priority();
    drain(&mut game, &[survivor]);
    assert_eq!(game.object_view(survivor).zone, Zone::Hand);
    assert_eq!(game.object_view(saga).zone, Zone::Graveyard);
}

#[test]
fn dot7_untaps_any_creature_once_at_sorcery_speed() {
    let mut game = scenario(Player::P0);
    let squad = owned(
        &mut game,
        Player::P0,
        "DOT-7 Repair Squad",
        Zone::Battlefield,
    );
    let target = put(&mut game, Player::P1, "Grizzly Bears", Zone::Battlefield);
    game.state.objects[target as usize].tapped = true;
    assert!(game.object_view(squad).keywords.contains(Keywords::FLYING));
    act(
        &mut game,
        Action::AnnounceActivate {
            card: squad,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    pay(&mut game);
    drain(&mut game, &[]);
    assert!(!game.object_view(target).tapped);
    assert!(
        !game.legal_actions().iter().any(
            |a| matches!(a, Action::AnnounceActivate { card, ability: 0, .. } if *card == squad)
        )
    );
}

#[test]
fn tendi_untaps_another_players_permanent_and_surveils() {
    let mut game = scenario(Player::P0);
    let tendi = owned(
        &mut game,
        Player::P0,
        "D'Vana Tendi, Optimistic Ensign",
        Zone::Battlefield,
    );
    game.state.objects[tendi as usize].summoning_sick = false;
    let target = put(&mut game, Player::P1, "Grizzly Bears", Zone::Battlefield);
    game.state.objects[target as usize].tapped = true;
    act(
        &mut game,
        Action::AnnounceActivate {
            card: tendi,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    pay(&mut game);
    drain(&mut game, &[]);
    assert!(!game.object_view(target).tapped);
    assert!(game.object_view(tendi).tapped);
    let top = put(&mut game, Player::P0, "Forest", Zone::Library);
    act(
        &mut game,
        Action::AnnounceActivate {
            card: tendi,
            ability: 1,
            counter_payment: Counters::default(),
        },
    );
    pay(&mut game);
    for _ in 0..32 {
        if matches!(game.state.pending, Pending::EffectLibrary { .. }) {
            break;
        }
        choose(&mut game, |a| {
            matches!(a, Action::Pass | Action::PutTrigger { .. })
        });
    }
    assert!(game.observe(Player::P0).library_choice.is_some());
    assert!(game.observe(Player::P1).library_choice.is_none());
    act(
        &mut game,
        Action::ArrangeLibrary {
            card: top,
            destination: LibraryDestination::Graveyard,
        },
    );
    drain(&mut game, &[]);
    assert!(game.state.players[0].scried_or_surveilled_this_turn);
    assert_eq!(game.object_view(top).zone, Zone::Graveyard);
}

#[test]
fn booker_landfall_makes_worm_and_pumps_owned_creatures() {
    let mut game = scenario(Player::P0);
    let booker = owned(
        &mut game,
        Player::P0,
        "Cleveland Booker, Empath",
        Zone::Battlefield,
    );
    let enemy = put(&mut game, Player::P1, "Grizzly Bears", Zone::Battlefield);
    let land = put(&mut game, Player::P0, "Forest", Zone::Hand);
    act(&mut game, Action::PlayLand { card: land });
    drain(&mut game, &[]);
    let worm = game
        .state
        .objects
        .iter()
        .enumerate()
        .find_map(|(i, o)| {
            (o.zone == Zone::Battlefield && o.controller == Player::P0 && o.card == Card(6014))
                .then_some(i as CardId)
        })
        .unwrap();
    assert_eq!(stats(&game, worm), (2, 2));
    act(
        &mut game,
        Action::AnnounceActivate {
            card: booker,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    pay(&mut game);
    drain(&mut game, &[]);
    assert_eq!(stats(&game, booker), (4, 2));
    assert_eq!(stats(&game, worm), (4, 2));
    assert_eq!(stats(&game, enemy), (2, 2));
}
