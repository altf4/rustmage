//! Inherited artifact effects, payments and reattachment.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn announce(game: &mut Game, card: CardId) {
    act(
        game,
        Action::AnnounceActivate {
            card,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
}

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

#[test]
fn barkform_harvester_has_reach_and_all_types_and_bottoms_only_its_owners_graveyard_card() {
    let player = Player::P0;
    let mut game = scenario(player);
    let harvester = put(&mut game, player, "Barkform Harvester", Zone::Battlefield);
    let ours = put(&mut game, player, "Opt", Zone::Graveyard);
    let theirs = put(&mut game, player.other(), "Opt", Zone::Graveyard);
    assert!(game.keywords(harvester).contains(Keywords::REACH));
    for subtype in ["Rabbit", "Otter", "Dragon"] {
        assert!(game.has_subtype(harvester, subtype));
    }
    announce(&mut game, harvester);
    reject(&mut game, target(theirs));
    act(&mut game, target(ours));
    act(
        &mut game,
        Action::PayCosts {
            payment: [0, 0, 0, 0, 0, 2],
        },
    );
    settle(&mut game);
    assert_eq!(game.object_view(ours).zone, Zone::Library);
    assert_eq!(
        game.state.players[player.index()].library.first(),
        Some(&ours)
    );
    assert_eq!(game.object_view(theirs).zone, Zone::Graveyard);
    assert!(!game.object_view(harvester).tapped);
}

#[test]
fn sharepot_creates_food_and_sacrifices_at_sorcery_speed_to_destroy_a_nonland() {
    let player = Player::P1;
    let mut game = scenario(player);
    let pot = put(&mut game, player, "Bumbleflower's Sharepot", Zone::Hand);
    let permanent = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let land = put(&mut game, player.other(), "Forest", Zone::Battlefield);
    cast(&mut game, pot, None);
    settle(&mut game);
    let food = game.controlled_token_ids(player).next().unwrap();
    assert!(game.has_subtype(food, "Food"));
    let activate = Action::AnnounceActivate {
        card: pot,
        ability: 0,
        counter_payment: Counters::default(),
    };
    game.state.step = Step::BeginCombat;
    reject(&mut game, activate.clone());
    game.state.step = Step::Main1;
    act(&mut game, activate);
    reject(&mut game, target(land));
    act(&mut game, target(permanent));
    act(
        &mut game,
        Action::PayCosts {
            payment: [0, 0, 0, 0, 0, 5],
        },
    );
    assert_eq!(game.object_view(pot).zone, Zone::Graveyard);
    settle(&mut game);
    assert_eq!(game.object_view(permanent).zone, Zone::Graveyard);
    assert_eq!(game.object_view(food).zone, Zone::Battlefield);
}

#[test]
fn inherited_equipment_reassigns_printed_keywords_and_stats_on_paid_equip() {
    let player = Player::P0;
    let specifications = [
        (
            "Short Bow",
            1,
            (3, 3),
            Keywords::REACH.union(Keywords::VIGILANCE),
        ),
        (
            "Sword of Vengeance",
            3,
            (4, 2),
            Keywords::FIRST_STRIKE
                .union(Keywords::VIGILANCE)
                .union(Keywords::TRAMPLE)
                .union(Keywords::HASTE),
        ),
    ];
    for (name, cost, with_gear, keywords) in specifications {
        let mut game = scenario(player);
        let equipment = put(&mut game, player, name, Zone::Battlefield);
        let first = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let second = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
        for card in [first, second] {
            announce(&mut game, equipment);
            reject(&mut game, target(enemy));
            act(&mut game, target(card));
            act(
                &mut game,
                Action::PayCosts {
                    payment: [0, 0, 0, 0, 0, cost],
                },
            );
            settle(&mut game);
            assert_eq!(
                game.object_view(equipment).attached_to,
                Some(game.object_ref(card))
            );
            assert_eq!(stats(&game, card), with_gear);
            assert!(game.keywords(card).contains(keywords));
        }
        assert_eq!(stats(&game, first), (2, 2));
        assert!(!game.keywords(first).intersects(keywords));
    }
}

#[test]
fn three_tree_mascot_filters_one_mana_once_each_turn_without_tapping() {
    let player = Player::P1;
    let mut game = scenario(player);
    let mascot = put(&mut game, player, "Three Tree Mascot", Zone::Battlefield);
    assert!(game.has_subtype(mascot, "Frog") && game.has_subtype(mascot, "Lizard"));
    game.state.objects[mascot as usize].summoning_sick = true;
    let filter = Action::ActivateMana {
        card: mascot,
        ability: 0,
        payment: [0, 0, 0, 0, 0, 1],
        counter_payment: Counters::default(),
        production: [0, 1, 0, 0, 0, 0],
    };
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 2];
    act(&mut game, filter.clone());
    assert_eq!(game.state.players[player.index()].mana, [0, 1, 0, 0, 0, 1]);
    assert!(!game.object_view(mascot).tapped);
    reject(&mut game, filter.clone());
    game.finish_cleanup();
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 1];
    act(&mut game, filter);
    assert_eq!(game.state.players[player.index()].mana, [0, 1, 0, 0, 0, 0]);
}
