//! Paid gameplay scenarios for Wilds of Eldraine cards from M through R.
mod support;

use rustmage as _;
use rustmage_core::engine::replacements::TokenGroupView;
use rustmage_core::{cards::*, engine::*};
use support::*;

const RAT: Card = Card::from_catalogue_id(5323);
const MONSTER: Card = Card::from_catalogue_id(5330);
const YOUNG_HERO: Card = Card::from_catalogue_id(5275);
const FOOD: Card = Card::from_catalogue_id(5246);

fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn tokens(game: &Game, controller: Player, recipe: Card) -> Vec<CardId> {
    game.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, object)| {
            (object.zone == Zone::Battlefield
                && object.controller == controller
                && object.is_token()
                && object.card == recipe)
                .then_some(id as CardId)
        })
        .collect()
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn attack(game: &mut Game, attacker: CardId) {
    game.begin_step(Step::BeginCombat);
    pass_pair(game);
    act(game, Action::Attack { card: attacker });
    act(game, Action::FinishAttackers);
}

fn finish_turn(game: &mut Game) {
    let turn = game.state.turn;
    game.begin_step(Step::End);
    for _ in 0..12 {
        if game.state.turn != turn {
            return;
        }
        assert_eq!(game.state.pending, Pending::Priority);
        act(game, Action::Pass);
    }
    panic!("turn did not reach cleanup");
}

#[test]
fn rat_out_zero_target_resolution_and_stale_target_are_distinct() {
    for player in [Player::P0, Player::P1] {
        let other = player.other();
        let mut game = scenario(other);
        let attacker = put(&mut game, other, "Hill Giant", Zone::Battlefield);
        let victim = put(&mut game, other, "Llanowar Elves", Zone::Battlefield);
        let spell = spawn(&mut game, player, named("Rat Out"), Zone::Hand);
        let bounce = put(&mut game, player, "Unsummon", Zone::Hand);
        act(&mut game, Action::Pass);
        act(&mut game, Action::AnnounceCast { card: spell });

        let checkpoint = game.snapshot();
        let finish_index = game
            .legal_actions()
            .iter()
            .position(|action| *action == Action::FinishTargets)
            .unwrap();
        let mut declined = game.clone();
        let mut indexed = game.clone();
        act(&mut declined, Action::FinishTargets);
        indexed
            .apply_index(player, game.revision(), finish_index)
            .unwrap();
        assert_eq!(declined.snapshot(), indexed.snapshot());
        indexed.restore(&checkpoint).unwrap();
        assert_eq!(indexed.snapshot(), checkpoint);
        pay(&mut declined);
        settle(&mut declined);
        let rats = tokens(&declined, player, RAT);
        assert_eq!(rats.len(), 1);
        let rat = rats[0];
        let rat_view = declined.object_view(rat);
        assert_eq!((rat_view.power, rat_view.toughness), (1, 1));
        assert_eq!(rat_view.colors, 4);
        assert!(rat_view.has_subtype("Rat"));
        assert!(rat_view.keywords.contains(Keywords::CANT_BLOCK));
        assert_eq!(stats(&declined, victim), (1, 1));
        attack(&mut declined, attacker);
        pass_pair(&mut declined);
        assert_eq!(declined.state.pending, Pending::Blockers);
        reject(
            &mut declined,
            Action::Block {
                blocker: rat,
                attacker,
            },
        );

        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(victim),
            },
        );
        pay(&mut game);
        let mut stale = game.clone();
        cast(&mut stale, bounce, Some(Target::Card(victim)));
        settle(&mut stale);
        assert_eq!(stale.object_view(victim).zone, Zone::Hand);
        assert!(tokens(&stale, player, RAT).is_empty());
        settle(&mut game);
        assert_eq!(game.object_view(victim).zone, Zone::Graveyard);
        assert_eq!(tokens(&game, player, RAT).len(), 1);
    }
}

#[test]
fn monstrous_rage_replaces_only_its_controllers_role_and_expires_the_pump() {
    for player in [Player::P0, Player::P1] {
        let other = player.other();
        let mut game = scenario(player);
        let target = put(&mut game, other, "Bear Cub", Zone::Battlefield);
        let recipient = game.object_ref(target);
        let spell = spawn(&mut game, player, named("Monstrous Rage"), Zone::Hand);
        let bounce = put(&mut game, player, "Unsummon", Zone::Hand);
        let group = [TokenGroupView {
            token: YOUNG_HERO,
            amount: 1,
            tapped: false,
        }];
        let prior =
            game.create_token_groups_attached(player, &group, None, true, Some(recipient))[0];
        let foreign =
            game.create_token_groups_attached(other, &group, None, true, Some(recipient))[0];

        cast(&mut game, spell, Some(Target::Card(target)));
        let mut stale = game.clone();
        cast(&mut stale, bounce, Some(Target::Card(target)));
        settle(&mut stale);
        assert!(tokens(&stale, player, MONSTER).is_empty());

        settle(&mut game);
        let monsters = tokens(&game, player, MONSTER);
        assert_eq!(monsters.len(), 1);
        assert_eq!(game.object_view(monsters[0]).attached_to, Some(recipient));
        assert_ne!(game.object_view(prior.card).zone, Zone::Battlefield);
        assert_eq!(game.object_view(foreign.card).attached_to, Some(recipient));
        assert_eq!(stats(&game, target), (5, 3));
        assert!(
            game.object_view(target)
                .keywords
                .contains(Keywords::TRAMPLE)
        );
        finish_turn(&mut game);
        assert_eq!(stats(&game, target), (3, 3));
        assert!(
            game.object_view(target)
                .keywords
                .contains(Keywords::TRAMPLE)
        );
    }
}

#[test]
fn monstrous_rage_remains_illegal_in_standard_decks() {
    let rage = named("Monstrous Rage");
    let mut deck = vec![Card::Forest; 60];
    deck[0] = rage;
    let result = Game::new(Config {
        decks: [deck, vec![Card::Forest; 60]],
        seed: 7,
        starting_player: Player::P0,
        deck_rules: DeckRules::Standard,
    });
    assert!(matches!(
        result,
        Err(Error::InvalidDeck(reason)) if reason.contains("not Standard legal")
            && reason.contains("Monstrous Rage")
    ));
}

#[test]
fn moonshaker_cavalry_counts_and_grants_at_resolution() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let survivor = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let witness = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let opposing = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
        let cavalry = spawn(&mut game, player, named("Moonshaker Cavalry"), Zone::Hand);
        let first_bounce = put(&mut game, player, "Unsummon", Zone::Hand);
        let second_bounce = put(&mut game, player, "Unsummon", Zone::Hand);
        let late = put(&mut game, player, "Bear Cub", Zone::Hand);

        act(&mut game, Action::AnnounceCast { card: cavalry });
        act(
            &mut game,
            Action::PayCosts {
                payment: [3, 0, 0, 0, 0, 5],
            },
        );
        pass_pair(&mut game);
        assert_eq!(game.object_view(cavalry).zone, Zone::Battlefield);
        assert_eq!(stats(&game, cavalry), (6, 6));
        assert!(
            game.object_view(cavalry)
                .keywords
                .contains(Keywords::FLYING)
        );
        assert!(
            !game
                .object_view(survivor)
                .keywords
                .contains(Keywords::FLYING)
        );
        assert_eq!(game.state.stack.len(), 1);

        let mut all_present = game.clone();
        settle(&mut all_present);
        assert_eq!(stats(&all_present, cavalry), (9, 9));
        assert_eq!(stats(&all_present, witness), (5, 5));

        cast(&mut game, first_bounce, Some(Target::Card(witness)));
        settle(&mut game);
        assert_eq!(game.object_view(witness).zone, Zone::Hand);
        assert_eq!(stats(&game, cavalry), (8, 8));
        assert_eq!(stats(&game, survivor), (4, 4));
        assert!(
            game.object_view(survivor)
                .keywords
                .contains(Keywords::FLYING)
        );
        assert_eq!(stats(&game, opposing), (2, 2));
        assert!(
            !game
                .object_view(opposing)
                .keywords
                .contains(Keywords::FLYING)
        );

        cast(&mut game, late, None);
        settle(&mut game);
        assert_eq!(stats(&game, late), (2, 2));
        assert!(!game.object_view(late).keywords.contains(Keywords::FLYING));
        cast(&mut game, second_bounce, Some(Target::Card(cavalry)));
        settle(&mut game);
        assert_eq!(stats(&game, survivor), (4, 4));
        assert!(
            game.object_view(survivor)
                .keywords
                .contains(Keywords::FLYING)
        );
        finish_turn(&mut game);
        assert_eq!(stats(&game, survivor), (2, 2));
        assert!(
            !game
                .object_view(survivor)
                .keywords
                .contains(Keywords::FLYING)
        );
    }
}

#[test]
fn hare_raising_locks_the_creature_count_and_the_hare_buffs_only_own_creature_tokens() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let hare = spawn(&mut game, player, named("Pollen-Shield Hare"), Zone::Hand);
        let target = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let rat = spawn(&mut game, player, RAT, Zone::Battlefield);
        let opposing_rat = spawn(&mut game, player.other(), RAT, Zone::Battlefield);
        let food = spawn(&mut game, player, FOOD, Zone::Battlefield);
        let bounce = put(&mut game, player, "Unsummon", Zone::Hand);

        act(&mut game, Action::AnnounceAdventure { card: hare });
        reject(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(opposing_rat),
            },
        );
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        act(
            &mut game,
            Action::PayCosts {
                payment: [0, 0, 0, 0, 1, 0],
            },
        );
        let spell = game.state.stack.last().unwrap();
        assert_eq!(spell.cast.unwrap().face, CastFace::Adventure);
        assert_eq!(spell.source.unwrap().card.name(), "Hare Raising");
        settle(&mut game);
        assert_eq!(game.object_view(hare).zone, Zone::Exile);
        assert_eq!(stats(&game, target), (4, 4));
        assert!(
            game.object_view(target)
                .keywords
                .contains(Keywords::VIGILANCE)
        );
        assert_eq!(stats(&game, rat), (1, 1));

        act(&mut game, Action::AnnounceExile { card: hare });
        act(
            &mut game,
            Action::PayCosts {
                payment: [1, 0, 0, 0, 0, 1],
            },
        );
        settle(&mut game);
        assert_eq!(game.object_view(hare).zone, Zone::Battlefield);
        assert_eq!(stats(&game, hare), (2, 2));
        assert_eq!(stats(&game, target), (4, 4));
        assert_eq!(stats(&game, rat), (2, 2));
        assert_eq!(stats(&game, opposing_rat), (1, 1));
        assert_eq!(stats(&game, food), (0, 0));

        let mut departed = game.clone();
        cast(&mut departed, bounce, Some(Target::Card(hare)));
        settle(&mut departed);
        assert_eq!(departed.object_view(hare).zone, Zone::Hand);
        assert_eq!(stats(&departed, rat), (1, 1));
        assert_eq!(stats(&departed, target), (4, 4));

        finish_turn(&mut game);
        assert_eq!(stats(&game, target), (2, 2));
        assert!(
            !game
                .object_view(target)
                .keywords
                .contains(Keywords::VIGILANCE)
        );
        assert_eq!(stats(&game, rat), (2, 2));
    }
}
