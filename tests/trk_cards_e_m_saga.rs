//! Star Trek E–M landfall, mode memory, and Saga chapters.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn owned(game: &mut Game, player: Player, name: &str, zone: Zone) -> CardId {
    spawn(game, player, Card::from_name(name).unwrap(), zone)
}
fn quiet(game: &mut Game) {
    for _ in 0..48 {
        if game.outcome().is_some() {
            return;
        }
        if game.state.pending == Pending::Priority
            && game.state.stack.is_empty()
            && game.state.pending_triggers.is_empty()
            && game.state.resolving.is_none()
        {
            return;
        }
        choose(game, |a| {
            matches!(a, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("resolution budget");
}
fn reach_action(game: &mut Game, action: &Action) {
    for _ in 0..24 {
        if game.legal_actions().contains(action) {
            return;
        }
        choose(game, |a| {
            matches!(a, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("expected action {action:?}");
}
fn count_token(game: &Game, player: Player, card: Card) -> usize {
    game.state
        .objects
        .iter()
        .filter(|o| o.zone == Zone::Battlefield && o.controller == player && o.card == card)
        .count()
}

#[test]
fn jake_investigates_only_on_second_resolution_of_exact_landfall() {
    let player = Player::P0;
    let mut game = scenario(player);
    let jake = owned(
        &mut game,
        player,
        "Jake Sisko, War Correspondent",
        Zone::Battlefield,
    );
    for index in 1..=3 {
        let land = put(&mut game, player, "Forest", Zone::Hand);
        game.move_object(land, Zone::Battlefield);
        quiet(&mut game);
        assert_eq!(game.state.players[player.index()].life, 20 + index);
        assert_eq!(
            count_token(&game, player, Card(5296)),
            usize::from(index >= 2)
        );
    }
    let checkpoint = game.snapshot();
    let mut branch = game.clone();
    branch.move_object(jake, Zone::Exile);
    branch.move_object(jake, Zone::Battlefield);
    let land = put(&mut branch, player, "Island", Zone::Hand);
    branch.move_object(land, Zone::Battlefield);
    quiet(&mut branch);
    assert_eq!(count_token(&branch, player, Card(5296)), 1);
    branch.restore(&checkpoint).unwrap();
    assert_eq!(branch.snapshot(), checkpoint);
}

#[test]
fn kobayashi_modes_have_distinct_targets_and_persistent_once_memory() {
    let player = Player::P0;
    let mut base = scenario(player);
    let source = owned(
        &mut base,
        player,
        "Kobayashi Maru Scenario",
        Zone::Battlefield,
    );
    let victim = put(&mut base, player.other(), "Bear Cub", Zone::Battlefield);
    let too_large = put(&mut base, player.other(), "Hill Giant", Zone::Battlefield);
    base.begin_step(Step::Upkeep);
    base.prepare_triggers();
    for mode in 0..4 {
        let mut game = base.clone();
        reach_action(&mut game, &Action::ChooseMode { mode });
        act(&mut game, Action::ChooseMode { mode });
        if mode == 0 {
            let target = Action::ChooseTarget {
                target: Target::Card(victim),
            };
            reach_action(&mut game, &target);
            reject(
                &mut game,
                Action::ChooseTarget {
                    target: Target::Card(too_large),
                },
            );
            act(&mut game, target);
            if game.legal_actions().contains(&Action::FinishTargets) {
                act(&mut game, Action::FinishTargets);
            }
        }
        quiet(&mut game);
        match mode {
            0 => assert_eq!(game.object_view(victim).zone, Zone::Hand),
            1 => assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 2),
            2 => assert_eq!(count_token(&game, player, Card(6008)), 2),
            3 => assert!(game.outcome().is_some()),
            _ => unreachable!(),
        }
        if mode < 3 {
            game.begin_step(Step::Upkeep);
            game.prepare_triggers();
            for _ in 0..10 {
                if game
                    .legal_actions()
                    .iter()
                    .any(|a| matches!(a, Action::ChooseMode { .. }))
                {
                    break;
                }
                choose(&mut game, |a| {
                    matches!(a, Action::Pass | Action::PutTrigger { .. })
                });
            }
            reject(&mut game, Action::ChooseMode { mode });
            assert!(
                game.legal_actions()
                    .iter()
                    .any(|a| matches!(a, Action::ChooseMode { mode: other } if *other != mode))
            );
        }
        assert_eq!(game.object_view(source).zone, Zone::Battlefield);
    }
}

#[test]
fn pale_moonlight_saga_requires_nontoken_opponent_sacrifice_and_optional_final_cost() {
    let player = Player::P0;
    let mut game = scenario(player);
    let victim = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let saga = owned(&mut game, player, "In the Pale Moonlight", Zone::Hand);
    cast(&mut game, saga, None);
    let choose_victim = Action::Sacrifice {
        object: game.object_ref(victim),
    };
    reach_action(&mut game, &choose_victim);
    act(&mut game, choose_victim);
    quiet(&mut game);
    assert_eq!(game.object_view(victim).zone, Zone::Graveyard);
    assert_eq!(game.counter_count(saga, CounterKind::Lore), 1);
    game.begin_step(Step::Main1);
    game.prepare_triggers();
    quiet(&mut game);
    assert_eq!(count_token(&game, player, Card(5296)), 1);
    let clue = game
        .state
        .objects
        .iter()
        .enumerate()
        .find(|(_, o)| o.zone == Zone::Battlefield && o.card == Card(5296))
        .unwrap()
        .0 as CardId;
    game.begin_step(Step::Main1);
    game.prepare_triggers();
    let decline_payment = Action::DeclineEffectPayment;
    reach_action(&mut game, &decline_payment);
    let mut decline = game.clone();
    act(&mut decline, decline_payment);
    quiet(&mut decline);
    assert_eq!(count_token(&decline, player, Card(6012)), 0);
    let clue_object = game.object_ref(clue);
    act(
        &mut game,
        Action::Sacrifice {
            object: clue_object,
        },
    );
    quiet(&mut game);
    assert_eq!(game.object_view(clue).zone, Zone::Outside);
    assert_eq!(count_token(&game, player, Card(6012)), 2);
    assert_eq!(game.object_view(saga).zone, Zone::Graveyard);
}
