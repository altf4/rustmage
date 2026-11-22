//! E–M printed interactions exercised through the owned game API.
use crate::support::*;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};

fn settle(game: &mut Game) {
    for _ in 0..48 {
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

#[test]
fn field_medic_counter_vigilance_expires_and_draw_checks_live_counters() {
    for mode in [0, 1] {
        let player = Player::P0;
        let mut game = scenario(player);
        let target = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        if mode == 1 {
            game.add_counter(target, CounterKind::PlusOne, 1);
        }
        let medic = put(&mut game, player, "Federation Field Medic", Zone::Hand);
        cast(&mut game, medic, None);
        for _ in 0..12 {
            if game.legal_actions().contains(&Action::ChooseMode { mode }) {
                break;
            }
            choose(&mut game, |action| {
                matches!(action, Action::Pass | Action::PutTrigger { .. })
            });
        }
        reject(&mut game, Action::ChooseMode { mode: 2 });
        act(&mut game, Action::ChooseMode { mode });
        if mode == 0 {
            act(
                &mut game,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
            settle(&mut game);
            assert_eq!(game.counter_count(target, CounterKind::PlusOne), 1);
            assert!(game.keywords(target).contains(Keywords::VIGILANCE));
            game.state.step = Step::Cleanup;
            game.finish_cleanup();
            assert!(!game.keywords(target).contains(Keywords::VIGILANCE));
            assert_eq!(game.counter_count(target, CounterKind::PlusOne), 1);
        } else {
            let before = game.hand_count(player);
            settle(&mut game);
            assert_eq!(game.hand_count(player), before + 1);
        }
    }
}

#[test]
fn hemmer_grants_own_turn_spacecraft_hexproof_and_loots_only_on_tap() {
    let player = Player::P0;
    let mut game = scenario(player);
    let hemmer = put(
        &mut game,
        player,
        "Hemmer, Fixing What Is Broken",
        Zone::Battlefield,
    );
    let spacecraft = spawn(
        &mut game,
        player,
        Card::from_name("H.M.S. Bounty, Unlikely Timeship").unwrap(),
        Zone::Battlefield,
    );
    let enemy = spawn(
        &mut game,
        player.other(),
        Card::from_name("H.M.S. Bounty, Unlikely Timeship").unwrap(),
        Zone::Battlefield,
    );
    assert!(game.keywords(spacecraft).contains(Keywords::HEXPROOF));
    assert!(!game.keywords(enemy).contains(Keywords::HEXPROOF));
    game.state.active = player.other();
    assert!(!game.keywords(spacecraft).contains(Keywords::HEXPROOF));
    game.state.active = player;
    let hand_before = game.hand_count(player);
    game.tap_objects_as(player, &[hemmer]);
    for _ in 0..12 {
        if game
            .legal_actions()
            .iter()
            .any(|a| matches!(a, Action::Discard { .. }))
        {
            break;
        }
        choose(&mut game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    assert_eq!(game.hand_count(player), hand_before + 1);
    choose(&mut game, |action| matches!(action, Action::Discard { .. }));
    settle(&mut game);
    assert_eq!(game.hand_count(player), hand_before);
}

#[test]
fn data_grows_from_any_players_noncreature_spell_and_pays_distributed_counters() {
    let player = Player::P0;
    let mut game = scenario(player);
    let data = put(
        &mut game,
        player,
        "Lieutenant Commander Data",
        Zone::Battlefield,
    );
    let bearer = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let victim = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    game.state.active = player.other();
    game.state.priority = player.other();
    let shock = put(&mut game, player.other(), "Shock", Zone::Hand);
    cast(&mut game, shock, Some(Target::Card(victim)));
    settle(&mut game);
    assert_eq!(game.counter_count(data, CounterKind::PlusOne), 1);

    game.state.active = player;
    game.state.priority = player;
    game.add_counter(bearer, CounterKind::PlusOne, 2);
    let before = game.snapshot();
    let action = Action::AnnounceActivate {
        card: data,
        ability: 0,
        counter_payment: Counters::default(),
    };
    let mut branch = game.clone();
    act(&mut branch, action.clone());
    branch.restore(&before).unwrap();
    assert_eq!(branch.snapshot(), before);
    act(&mut game, action);
    let victim_ref = game.object_ref(victim);
    reject(
        &mut game,
        Action::ChooseCreatureCounterCost {
            object: victim_ref,
            counter: CounterKind::PlusOne,
        },
    );
    for id in [data, bearer, bearer] {
        let object = game.object_ref(id);
        act(
            &mut game,
            Action::ChooseCreatureCounterCost {
                object,
                counter: CounterKind::PlusOne,
            },
        );
    }
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.counter_count(data, CounterKind::PlusOne), 0);
    assert_eq!(game.counter_count(bearer, CounterKind::PlusOne), 0);
    assert_eq!(game.hand_count(player), 1);
}
