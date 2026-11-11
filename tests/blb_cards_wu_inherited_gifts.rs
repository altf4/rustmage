mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn begin(game: &mut Game, card: CardId, promise: bool) {
    let beneficiary = promise.then(|| game.actor().unwrap().other());
    act(game, Action::AnnounceCast { card });
    act(game, Action::ChooseGift { beneficiary });
}

fn pick(game: &mut Game, target: Target) {
    act(game, Action::ChooseTarget { target });
}

fn finish(game: &mut Game) {
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(game, Action::FinishTargets);
    }
    pay(game);
    settle(game);
}

fn gift_tokens(game: &Game, player: Player, subtype: &str, tapped: bool) -> usize {
    game.observe(player)
        .objects
        .iter()
        .filter(|object| {
            object.zone == Zone::Battlefield
                && object.controller == player
                && object.is_token()
                && object.has_subtype(subtype)
                && object.tapped == tapped
        })
        .count()
}

#[test]
fn crumb_gives_food_and_indestructible_only_after_resolving_a_promised_gift() {
    let player = Player::P0;
    for promise in [false, true] {
        let mut game = scenario(player);
        let own = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
        let spell = put(&mut game, player, "Crumb and Get It", Zone::Hand);
        begin(&mut game, spell, promise);
        reject(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        pick(&mut game, Target::Card(own));
        pay(&mut game);
        let mut stale = game.clone();
        stale.move_object(own, Zone::Exile);
        settle(&mut stale);
        assert_eq!(gift_tokens(&stale, player.other(), "Food", false), 0);
        settle(&mut game);
        assert_eq!(stats(&game, own), (4, 4));
        assert_eq!(
            game.object_view(own)
                .keywords
                .contains(Keywords::INDESTRUCTIBLE),
            promise
        );
        assert_eq!(
            gift_tokens(&game, player.other(), "Food", false),
            usize::from(promise)
        );
    }
}

#[test]
fn dawns_truce_protects_player_and_permanents_and_its_card_gift_adds_indestructible() {
    let player = Player::P1;
    for promise in [false, true] {
        let mut game = scenario(player);
        let own = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        let artifact = put(&mut game, player, "Swiftfoot Boots", Zone::Battlefield);
        let truce = put(&mut game, player, "Dawn's Truce", Zone::Hand);
        let shock = put(&mut game, player.other(), "Shock", Zone::Hand);
        begin(&mut game, truce, promise);
        finish(&mut game);
        for id in [own, artifact] {
            let keywords = game.object_view(id).keywords;
            assert!(keywords.contains(Keywords::HEXPROOF));
            assert_eq!(keywords.contains(Keywords::INDESTRUCTIBLE), promise);
        }
        assert_eq!(
            game.state.players[player.other().index()].cards_drawn_this_turn,
            u64::from(promise)
        );
        if game.actor() == Some(player) {
            act(&mut game, Action::Pass);
        }
        act(&mut game, Action::AnnounceCast { card: shock });
        for target in [Target::Player(player), Target::Card(own)] {
            reject(&mut game, Action::ChooseTarget { target });
        }
        pick(&mut game, Target::Player(player.other()));
        finish(&mut game);
        assert_eq!(game.state.players[player.index()].life, 20);
        assert_eq!(game.state.players[player.other().index()].life, 18);
    }
}

#[test]
fn dewdrop_cure_can_return_the_third_cheap_creature_only_with_a_card_gift() {
    let player = Player::P0;
    for promise in [false, true] {
        let mut game = scenario(player);
        let cheap = ["Llanowar Elves", "Bear Cub", "Grizzly Bears"]
            .map(|name| put(&mut game, player, name, Zone::Graveyard));
        let large = put(&mut game, player, "Hill Giant", Zone::Graveyard);
        let spell = put(&mut game, player, "Dewdrop Cure", Zone::Hand);
        begin(&mut game, spell, promise);
        reject(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(large),
            },
        );
        for card in &cheap[..2] {
            pick(&mut game, Target::Card(*card));
        }
        if promise {
            pick(&mut game, Target::Card(cheap[2]));
        } else {
            reject(
                &mut game,
                Action::ChooseTarget {
                    target: Target::Card(cheap[2]),
                },
            );
        }
        finish(&mut game);
        for card in &cheap[..2] {
            assert_eq!(game.object_view(*card).zone, Zone::Battlefield);
        }
        assert_eq!(
            game.object_view(cheap[2]).zone,
            if promise {
                Zone::Battlefield
            } else {
                Zone::Graveyard
            }
        );
        assert_eq!(
            game.state.players[player.other().index()].cards_drawn_this_turn,
            u64::from(promise)
        );
    }
}

#[test]
fn flood_maw_bounces_a_noncreature_only_with_the_tapped_fish_gift() {
    let player = Player::P0;
    let opponent = player.other();
    for promise in [false, true] {
        let mut game = scenario(player);
        let bear = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
        let artifact = put(&mut game, opponent, "Swiftfoot Boots", Zone::Battlefield);
        let land = put(&mut game, opponent, "Forest", Zone::Battlefield);
        let spell = put(&mut game, player, "Into the Flood Maw", Zone::Hand);
        begin(&mut game, spell, promise);
        reject(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(land),
            },
        );
        let chosen = if promise { artifact } else { bear };
        if !promise {
            reject(
                &mut game,
                Action::ChooseTarget {
                    target: Target::Card(artifact),
                },
            );
        }
        pick(&mut game, Target::Card(chosen));
        finish(&mut game);
        assert_eq!(game.object_view(chosen).zone, Zone::Hand);
        assert_eq!(
            gift_tokens(&game, opponent, "Fish", true),
            usize::from(promise)
        );
    }
}

#[test]
fn long_rivers_pull_gifts_a_card_to_counter_a_noncreature_spell() {
    let player = Player::P1;
    let opponent = player.other();
    for promise in [false, true] {
        let mut game = scenario(opponent);
        let pull = put(&mut game, player, "Long River's Pull", Zone::Hand);
        let original = put(
            &mut game,
            opponent,
            if promise {
                "Divination"
            } else {
                "Grizzly Bears"
            },
            Zone::Hand,
        );
        cast(&mut game, original, None);
        if game.actor() == Some(opponent) {
            act(&mut game, Action::Pass);
        }
        begin(&mut game, pull, promise);
        pick(&mut game, Target::Spell(original));
        finish(&mut game);
        assert_eq!(game.object_view(original).zone, Zone::Graveyard);
        assert_eq!(
            game.state.players[opponent.index()].cards_drawn_this_turn,
            u64::from(promise)
        );
    }
}

#[test]
fn mind_spiral_draws_for_its_player_and_taps_and_stuns_only_on_the_gift_route() {
    let player = Player::P0;
    let opponent = player.other();
    for promise in [false, true] {
        let mut game = scenario(player);
        let enemy = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
        let own = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let spiral = put(&mut game, player, "Mind Spiral", Zone::Hand);
        begin(&mut game, spiral, promise);
        pick(&mut game, Target::Player(player));
        if promise {
            reject(
                &mut game,
                Action::ChooseTarget {
                    target: Target::Card(own),
                },
            );
            pick(&mut game, Target::Card(enemy));
        }
        finish(&mut game);
        assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 3);
        assert_eq!(game.object_view(enemy).tapped, promise);
        assert_eq!(
            game.counter_count(enemy, CounterKind::Stun),
            u64::from(promise)
        );
        assert_eq!(
            gift_tokens(&game, opponent, "Fish", true),
            usize::from(promise)
        );
    }
}
