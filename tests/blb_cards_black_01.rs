//! Bloomburrow life, offspring, and graveyard-history cards.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn life(game: &Game, player: Player) -> i128 {
    game.state.players[player.index()].life
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn order_triggers(game: &mut Game) {
    while matches!(game.state.pending, Pending::OrderTriggers { .. }) {
        choose(game, |action| matches!(action, Action::PutTrigger { .. }));
    }
}

fn available(game: &Game, source: CardId) -> bool {
    game.legal_actions().iter().any(|action| {
        matches!(action, Action::Activate { card, .. } | Action::AnnounceActivate { card, .. } if *card == source)
    })
}

fn activate(game: &mut Game, source: CardId, target: Option<Target>) {
    choose(game, |action| {
        matches!(action, Action::Activate { card, target: chosen, .. } if *card == source && *chosen == target)
            || matches!(action, Action::AnnounceActivate { card, .. } if *card == source)
    });
    if matches!(game.state.pending, Pending::Targets { .. }) {
        act(
            game,
            Action::ChooseTarget {
                target: target.unwrap(),
            },
        );
        if game.legal_actions().contains(&Action::FinishTargets) {
            act(game, Action::FinishTargets);
        }
    }
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(game);
    }
}

#[test]
fn agate_blade_assassin_drains_the_defending_player_when_attacking_a_planeswalker() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let assassin = put(&mut game, player, "Agate-Blade Assassin", Zone::Battlefield);
    let walker = put(
        &mut game,
        opponent,
        "Ajani, Caller of the Pride",
        Zone::Battlefield,
    );
    game.add_counter(walker, CounterKind::Loyalty, 4);
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    let defender = game.object_ref(walker);
    act(
        &mut game,
        Action::AttackPlaneswalker {
            card: assassin,
            defender,
        },
    );
    act(&mut game, Action::FinishAttackers);
    settle(&mut game);
    assert_eq!([life(&game, player), life(&game, opponent)], [21, 19]);
    assert_eq!(
        game.object_view(walker).counters.get(CounterKind::Loyalty),
        4
    );
}

#[test]
fn bonecache_overseer_requires_three_own_departures_or_a_committed_food_sacrifice() {
    let player = Player::P1;
    let mut game = scenario(player);
    let overseer = put(&mut game, player, "Bonecache Overseer", Zone::Battlefield);
    for owner in [player, player, player.other()] {
        let card = put(&mut game, owner, "Forest", Zone::Graveyard);
        game.move_object(card, Zone::Hand);
    }
    assert!(!available(&game, overseer));
    let food = put(&mut game, player, "Gingerbrute", Zone::Battlefield);
    let mut destroyed = game.clone();
    destroyed.move_object(food, Zone::Graveyard);
    assert!(!available(&destroyed, overseer));
    let mut food_branch = game.clone();
    food_branch.sacrifice_batch(&[(player, food)]);
    let third = put(&mut game, player, "Forest", Zone::Graveyard);
    game.move_object(third, Zone::Exile);
    for branch in [&mut game, &mut food_branch] {
        assert!(available(branch, overseer));
        activate(branch, overseer, None);
        settle(branch);
        assert!(branch.object_view(overseer).tapped);
        assert_eq!(life(branch, player), 19);
        assert_eq!(
            branch.state.players[player.index()].cards_drawn_this_turn,
            1
        );
    }
}

#[test]
fn darkstar_augur_offspring_reveals_the_new_top_on_each_upkeep_trigger() {
    let player = Player::P0;
    let mut game = scenario(player);
    let parent = put(&mut game, player, "Darkstar Augur", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: parent });
    act(&mut game, Action::ChooseOffspring { pay: true });
    pay(&mut game);
    settle(&mut game);
    let child = game
        .state
        .objects
        .iter()
        .position(|object| object.is_copy_token())
        .unwrap() as CardId;
    assert_eq!(game.object_view(parent).card, game.object_view(child).card);
    assert_eq!(stats(&game, parent), (2, 3));
    assert_eq!(stats(&game, child), (1, 1));
    assert!(game.object_view(child).keywords.contains(Keywords::FLYING));
    let land = put(&mut game, player, "Forest", Zone::Library);
    let expensive = put(&mut game, player, "Serra Angel", Zone::Library);
    game.state.active = player.other();
    game.state.priority = player.other();
    game.state.step = Step::End;
    pass_pair(&mut game);
    assert_eq!(game.state.step, Step::Upkeep);
    order_triggers(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(expensive).zone, Zone::Hand);
    assert_eq!(game.object_view(land).zone, Zone::Hand);
    assert_eq!(life(&game, player), 15);
}

#[test]
fn moonstone_harbinger_grants_current_bats_one_bonus_for_either_first_life_change() {
    let player = Player::P0;
    for loss_first in [false, true] {
        let mut game = scenario(player);
        let bat = put(&mut game, player, "Moonstone Harbinger", Zone::Battlefield);
        let other_bat = put(&mut game, player, "Starlit Soothsayer", Zone::Battlefield);
        let opposing_bat = put(
            &mut game,
            player.other(),
            "Starlit Soothsayer",
            Zone::Battlefield,
        );
        let monolith = put(&mut game, player, "Sinister Monolith", Zone::Battlefield);
        let duo = put(&mut game, player, "Glidedive Duo", Zone::Hand);
        for lose in [loss_first, !loss_first] {
            if lose {
                activate(&mut game, monolith, None);
            } else {
                cast(&mut game, duo, None);
            }
            settle(&mut game);
            assert_eq!(stats(&game, bat), (2, 3));
            assert_eq!(stats(&game, other_bat), (3, 2));
            assert_eq!(stats(&game, opposing_bat), (2, 2));
            assert!(
                game.object_view(other_bat)
                    .keywords
                    .contains(Keywords::DEATHTOUCH)
            );
            assert!(
                !game
                    .object_view(opposing_bat)
                    .keywords
                    .contains(Keywords::DEATHTOUCH)
            );
        }
    }
}

#[test]
fn psychic_whorl_surveils_privately_even_when_the_target_has_nothing_to_discard() {
    let player = Player::P1;
    for rat in [false, true] {
        let mut game = scenario(player);
        if rat {
            put(&mut game, player, "Daggerfang Duo", Zone::Battlefield);
        }
        let whorl = put(&mut game, player, "Psychic Whorl", Zone::Hand);
        cast(&mut game, whorl, Some(Target::Player(player.other())));
        pass_pair(&mut game);
        assert_eq!(
            matches!(game.state.pending, Pending::EffectLibrary { .. }),
            rat
        );
        if rat {
            assert!(game.observe(player).library_choice.is_some());
            assert!(game.observe(player.other()).library_choice.is_none());
            for _ in 0..2 {
                choose(&mut game, |action| {
                    matches!(
                        action,
                        Action::ArrangeLibrary {
                            destination: LibraryDestination::Graveyard,
                            ..
                        }
                    )
                });
            }
        }
        settle(&mut game);
        assert_eq!(
            game.state.players[player.index()].library.len(),
            if rat { 6 } else { 8 }
        );
    }
}

#[test]
fn starlit_soothsayer_checks_its_controllers_life_during_their_end_step() {
    let player = Player::P0;
    for change in [-1, 0, 1] {
        let mut game = scenario(player);
        put(&mut game, player, "Starlit Soothsayer", Zone::Battlefield);
        put(
            &mut game,
            player.other(),
            "Starlit Soothsayer",
            Zone::Battlefield,
        );
        if change < 0 {
            let monolith = put(&mut game, player, "Sinister Monolith", Zone::Battlefield);
            activate(&mut game, monolith, None);
            settle(&mut game);
        } else if change > 0 {
            let duo = put(&mut game, player, "Glidedive Duo", Zone::Hand);
            cast(&mut game, duo, None);
            settle(&mut game);
        }
        game.state.step = Step::Main2;
        pass_pair(&mut game);
        assert_eq!(game.state.step, Step::End);
        assert_eq!(game.state.stack.len(), usize::from(change != 0));
        if change != 0 {
            pass_pair(&mut game);
            assert_eq!(game.actor(), Some(player));
            choose(&mut game, |action| {
                matches!(
                    action,
                    Action::ArrangeLibrary {
                        destination: LibraryDestination::Top,
                        ..
                    }
                )
            });
        }
        settle(&mut game);
    }
}

#[test]
fn bloomburrow_equipment_tokens_have_real_equip_cost_and_printed_effects() {
    let player = Player::P0;
    for (token, keywords) in [
        (
            Card::from_catalogue_id(5619),
            Keywords::VIGILANCE
                .union(Keywords::TRAMPLE)
                .union(Keywords::HASTE),
        ),
        (Card::from_catalogue_id(5620), Keywords::NONE),
    ] {
        let mut game = scenario(player);
        let equipment = spawn(&mut game, player, token, Zone::Battlefield);
        let bearer = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        let opposing = put(
            &mut game,
            player.other(),
            "Grizzly Bears",
            Zone::Battlefield,
        );
        assert_eq!(stats(&game, bearer), (2, 2));
        assert!(!game.legal_actions().iter().any(|action| matches!(action, Action::Activate { card, target: Some(Target::Card(target)), .. } if *card == equipment && *target == opposing)));
        game.state.players[player.index()].mana = [1, 0, 0, 0, 0, 0];
        assert!(!available(&game, equipment));
        game.state.players[player.index()].mana = [2, 0, 0, 0, 0, 0];
        activate(&mut game, equipment, Some(Target::Card(bearer)));
        settle(&mut game);
        assert_eq!(game.state.players[player.index()].mana, [0; 6]);
        assert_eq!(stats(&game, bearer), (3, 3));
        assert_eq!(game.object_view(bearer).keywords, keywords);
    }
    let mut game = scenario(player);
    let snail = spawn(
        &mut game,
        player,
        Card::from_catalogue_id(5621),
        Zone::Battlefield,
    );
    assert_eq!(stats(&game, snail), (1, 1));
    assert!(Card::from_catalogue_id(5621).has_subtype("Snail"));
}
