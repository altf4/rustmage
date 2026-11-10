//! Definition-level scenarios for the remaining admitted EOE cards in collectors 133–198.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn resolve_top(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn until(game: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..96 {
        if ready(game) {
            return;
        }
        choose(game, |action| {
            matches!(
                action,
                Action::Pass
                    | Action::PutTrigger { .. }
                    | Action::FinishAttackers
                    | Action::FinishBlockers
            )
        });
    }
    panic!("scenario did not reach its stop condition");
}

fn settle_ordered(game: &mut Game) {
    until(game, |game| {
        game.state.pending == Pending::Priority
            && game.state.stack.is_empty()
            && game.state.pending_triggers.is_empty()
    });
}

fn target(game: &mut Game, target: Target) {
    act(game, Action::ChooseTarget { target });
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(game, Action::FinishTargets);
    }
}

fn activation(card: CardId) -> Action {
    Action::AnnounceActivate {
        card,
        ability: 0,
        counter_payment: Counters::default(),
    }
}

fn token_count(game: &Game, card: Card) -> usize {
    game.state
        .objects
        .iter()
        .filter(|object| {
            object.zone == Zone::Battlefield && object.is_token() && object.card == card
        })
        .count()
}

#[test]
fn blooming_stinger_keeps_its_deathtouch_and_grants_another_controlled_creature_deathtouch() {
    let player = Player::P1;
    let mut game = scenario(player);
    let stinger = put(&mut game, player, "Blooming Stinger", Zone::Hand);
    let ally = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let enemy = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    cast(&mut game, stinger, None);
    resolve_top(&mut game);
    for invalid in [stinger, enemy] {
        reject(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(invalid),
            },
        );
    }
    target(&mut game, Target::Card(ally));
    settle(&mut game);
    for id in [stinger, ally] {
        assert!(game.object_view(id).keywords.contains(Keywords::DEATHTOUCH));
    }
    let turn = game.state.turn;
    until(&mut game, |game| game.state.turn == turn + 1);
    assert!(
        game.object_view(stinger)
            .keywords
            .contains(Keywords::DEATHTOUCH)
    );
    assert!(
        !game
            .object_view(ally)
            .keywords
            .contains(Keywords::DEATHTOUCH)
    );
}

#[test]
fn drix_fatemaker_warps_and_can_counter_an_opponent_but_grants_only_own_marked_creatures_trample() {
    let player = Player::P0;
    let mut game = scenario(player);
    let drix = put(&mut game, player, "Drix Fatemaker", Zone::Hand);
    let ally = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let enemy = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    game.add_counter(ally, CounterKind::PlusOne, 1);
    act(&mut game, Action::AnnounceWarp { card: drix });
    pay(&mut game);
    resolve_top(&mut game);
    target(&mut game, Target::Card(enemy));
    settle(&mut game);
    assert!(game.object_view(drix).cast_for_warp);
    assert_eq!(game.counter_count(enemy, CounterKind::PlusOne), 1);
    assert!(!game.object_view(enemy).keywords.contains(Keywords::TRAMPLE));
    assert!(!game.object_view(drix).keywords.contains(Keywords::TRAMPLE));
    assert!(game.object_view(ally).keywords.contains(Keywords::TRAMPLE));
    until(&mut game, |game| game.object_view(drix).zone == Zone::Exile);
    assert!(!game.object_view(ally).keywords.contains(Keywords::TRAMPLE));
}

#[test]
fn four_landfall_permanents_distinguish_life_stats_and_the_robot() {
    let player = Player::P0;
    let mut game = scenario(player);
    let red = put(&mut game, player, "Remnant Elemental", Zone::Battlefield);
    let green = put(&mut game, player, "Icecave Crasher", Zone::Battlefield);
    put(
        &mut game,
        player,
        "Eumidian Terrabotanist",
        Zone::Battlefield,
    );
    let engineering = put(&mut game, player, "Eusocial Engineering", Zone::Hand);
    let first = put(&mut game, player, "Forest", Zone::Hand);
    let foreign = put(&mut game, player.other(), "Island", Zone::Hand);
    act(&mut game, Action::AnnounceWarp { card: engineering });
    pay(&mut game);
    settle(&mut game);
    assert!(game.object_view(engineering).cast_for_warp);

    act(&mut game, Action::PlayLand { card: first });
    settle_ordered(&mut game);
    assert_eq!(game.state.players[player.index()].life, 21);
    assert_eq!(stats(&game, red), (2, 4));
    assert!(game.object_view(red).keywords.contains(Keywords::REACH));
    assert_eq!(stats(&game, green), (5, 4));
    assert!(game.object_view(green).keywords.contains(Keywords::TRAMPLE));
    let robot = game
        .observe(player)
        .objects
        .into_iter()
        .find(|object| {
            object.zone == Zone::Battlefield && object.is_token() && object.has_subtype("Robot")
        })
        .unwrap();
    assert_eq!(
        (
            robot.controller,
            robot.kinds,
            robot.colors,
            robot.power,
            robot.toughness
        ),
        (player, 17, 0, 2, 2)
    );

    let turn = game.state.turn;
    until(&mut game, |game| {
        game.state.turn == turn + 1 && game.state.step == Step::Main1
    });
    assert_eq!(game.object_view(engineering).zone, Zone::Exile);
    act(&mut game, Action::PlayLand { card: foreign });
    settle_ordered(&mut game);
    assert_eq!(game.state.players[player.index()].life, 21);
    assert_eq!(stats(&game, red), (0, 4));
    assert_eq!(stats(&game, green), (4, 4));
    assert_eq!(token_count(&game, Card::from_catalogue_id(5309)), 1);
}

#[test]
fn fungal_colossus_costs_less_for_distinct_controlled_land_names() {
    let player = Player::P0;
    let mut game = scenario(player);
    for name in ["Forest", "Forest", "Island"] {
        let land = put(&mut game, player, name, Zone::Battlefield);
        game.state.objects[land as usize].tapped = true;
    }
    put(&mut game, player.other(), "Swamp", Zone::Battlefield);
    let colossus = put(&mut game, player, "Fungal Colossus", Zone::Hand);
    game.state.players[player.index()].mana = [0, 0, 0, 0, 4, 0];
    reject(&mut game, Action::AnnounceCast { card: colossus });
    let third_name = put(&mut game, player, "Mountain", Zone::Battlefield);
    game.state.objects[third_name as usize].tapped = true;
    cast(&mut game, colossus, None);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    assert_eq!(game.object_mana_value(colossus), 7);
    assert_eq!(stats(&game, colossus), (5, 5));
}

#[test]
fn wayfarer_and_godmaw_each_make_a_lander_and_godmaw_boosts_current_creatures() {
    let player = Player::P0;
    let mut game = scenario(player);
    let lander = Card::from_catalogue_id(5238);
    let wayfarer = put(&mut game, player, "Galactic Wayfarer", Zone::Hand);
    let godmaw = put(&mut game, player, "Glacier Godmaw", Zone::Hand);
    let enemy = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let land = put(&mut game, player, "Forest", Zone::Hand);

    for (card, expected) in [(wayfarer, 1), (godmaw, 2)] {
        cast(&mut game, card, None);
        settle(&mut game);
        assert_eq!(token_count(&game, lander), expected);
    }
    assert!(
        game.object_view(godmaw)
            .keywords
            .contains(Keywords::TRAMPLE)
    );
    act(&mut game, Action::PlayLand { card: land });
    settle(&mut game);
    assert_eq!(stats(&game, wayfarer), (4, 4));
    assert_eq!(stats(&game, godmaw), (7, 7));
    for id in [godmaw, wayfarer] {
        assert!(
            game.object_view(id)
                .keywords
                .contains(Keywords::HASTE.union(Keywords::VIGILANCE))
        );
    }
    let newcomer = put(&mut game, player, "Grizzly Bears", Zone::Hand);
    cast(&mut game, newcomer, None);
    settle(&mut game);
    for id in [enemy, newcomer] {
        assert_eq!(stats(&game, id), (2, 2));
        assert!(!game.object_view(id).keywords.contains(Keywords::HASTE));
    }
}

#[test]
fn gene_pollinator_taps_itself_and_one_distinct_controlled_permanent_for_a_color() {
    let player = Player::P1;
    let mut game = scenario(player);
    let pollinator = put(&mut game, player, "Gene Pollinator", Zone::Battlefield);
    let land = put(&mut game, player, "Forest", Zone::Battlefield);
    let foreign = put(&mut game, player.other(), "Forest", Zone::Battlefield);
    let announce = |production| Action::AnnounceMana {
        card: pollinator,
        ability: 0,
        counter_payment: Counters::default(),
        production,
    };
    reject(&mut game, announce([0, 0, 0, 0, 0, 1]));
    let before = game.state.players[player.index()].mana[1];
    act(&mut game, announce([0, 1, 0, 0, 0, 0]));
    for invalid in [pollinator, foreign] {
        let object = game.object_ref(invalid);
        reject(&mut game, Action::ChooseAdditionalCost { object });
    }
    let object = game.object_ref(land);
    act(&mut game, Action::ChooseAdditionalCost { object });
    pay(&mut game);
    assert_eq!(game.state.players[player.index()].mana[1], before + 1);
    assert!(game.object_view(pollinator).tapped);
    assert!(game.object_view(land).tapped);
    assert!(game.state.stack.is_empty());
}

#[test]
fn germinating_wurm_gains_two_life_on_warp_and_again_on_its_later_paid_recast() {
    let player = Player::P0;
    let mut game = scenario(player);
    let wurm = put(&mut game, player, "Germinating Wurm", Zone::Hand);
    game.state.players[player.index()].mana = [0, 0, 0, 0, 2, 0];
    act(&mut game, Action::AnnounceWarp { card: wurm });
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, 22);
    let turn = game.state.turn;
    until(&mut game, |game| game.object_view(wurm).zone == Zone::Exile);
    assert!(!game.can_cast_exiled_card(player, wurm));
    until(&mut game, |game| {
        game.state.turn == turn + 2 && game.state.step == Step::Main1
    });
    game.state.players[player.index()].mana = [0, 0, 0, 0, 5, 0];
    act(&mut game, Action::AnnounceExile { card: wurm });
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, 24);
    assert!(!game.object_view(wurm).cast_for_warp);
}

#[test]
fn harmonious_grovestrider_has_land_defined_stats_in_other_zones_and_ward_two_in_play() {
    let player = Player::P0;
    let mut game = scenario(player);
    let strider = put(&mut game, player, "Harmonious Grovestrider", Zone::Hand);
    for name in ["Forest", "Forest", "Island"] {
        put(&mut game, player, name, Zone::Battlefield);
    }
    put(&mut game, player.other(), "Forest", Zone::Battlefield);
    assert_eq!(stats(&game, strider), (3, 3));
    let mut exiled = game.clone();
    exiled.move_object(strider, Zone::Exile);
    assert_eq!(stats(&exiled, strider), (3, 3));

    cast(&mut game, strider, None);
    settle(&mut game);
    let land = put(&mut game, player, "Plains", Zone::Hand);
    act(&mut game, Action::PlayLand { card: land });
    assert_eq!(stats(&game, strider), (4, 4));
    let shock = put(&mut game, player.other(), "Shock", Zone::Hand);
    act(&mut game, Action::Pass);
    cast(&mut game, shock, Some(Target::Card(strider)));
    resolve_top(&mut game);
    assert!(game.legal_actions().contains(&Action::PayResolutionMana {
        payment: [0, 0, 0, 2, 0, 0]
    }));
    act(&mut game, Action::DeclineResolutionMana);
    settle(&mut game);
    assert_eq!(game.object_view(strider).damage, 0);
}

#[test]
fn icetill_explorer_plays_two_lands_including_from_graveyard_and_mills_for_each() {
    let player = Player::P1;
    let mut game = scenario(player);
    put(&mut game, player, "Icetill Explorer", Zone::Battlefield);
    let hand = put(&mut game, player, "Forest", Zone::Hand);
    let grave = put(&mut game, player, "Swamp", Zone::Graveyard);
    let extra = put(&mut game, player, "Mountain", Zone::Hand);
    let second_mill = put(&mut game, player, "Island", Zone::Library);
    let first_mill = put(&mut game, player, "Plains", Zone::Library);
    for (card, milled) in [(hand, first_mill), (grave, second_mill)] {
        act(&mut game, Action::PlayLand { card });
        settle(&mut game);
        assert_eq!(game.object_view(card).zone, Zone::Battlefield);
        assert_eq!(game.object_view(milled).zone, Zone::Graveyard);
    }
    reject(&mut game, Action::PlayLand { card: extra });
}

#[test]
fn intrepid_tenderfoot_repeats_its_paid_sorcery_only_activation() {
    let player = Player::P0;
    let mut game = scenario(player);
    let tenderfoot = put(&mut game, player, "Intrepid Tenderfoot", Zone::Battlefield);
    for count in [1, 2] {
        act(&mut game, activation(tenderfoot));
        pay(&mut game);
        reject(&mut game, activation(tenderfoot));
        settle(&mut game);
        assert_eq!(game.counter_count(tenderfoot, CounterKind::PlusOne), count);
        assert!(!game.object_view(tenderfoot).tapped);
    }
}

#[test]
fn loading_zone_warps_and_doubles_each_qualifying_counter_event_once() {
    let player = Player::P0;
    let mut game = scenario(player);
    let enchantment = put(&mut game, player, "Loading Zone", Zone::Hand);
    let planet = spawn(
        &mut game,
        player,
        Card::from_name("Evendo, Waking Haven").unwrap(),
        Zone::Battlefield,
    );
    let ship = put(&mut game, player, "Warmaker Gunship", Zone::Battlefield);
    let ally = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    act(&mut game, Action::AnnounceWarp { card: enchantment });
    act(
        &mut game,
        Action::PayCosts {
            payment: [0, 0, 0, 0, 1, 0],
        },
    );
    settle(&mut game);
    assert!(game.object_view(enchantment).cast_for_warp);

    for recipient in [planet, ship, ship] {
        let drill = put(&mut game, player, "Drill Too Deep", Zone::Hand);
        act(&mut game, Action::AnnounceCast { card: drill });
        act(&mut game, Action::ChooseMode { mode: 0 });
        target(&mut game, Target::Card(recipient));
        pay(&mut game);
        settle(&mut game);
    }
    assert_eq!(game.counter_count(planet, CounterKind::Charge), 10);
    assert_eq!(game.counter_count(ship, CounterKind::Charge), 20);
    let burst = put(&mut game, player, "Biosynthic Burst", Zone::Hand);
    cast(&mut game, burst, Some(Target::Card(ally)));
    settle(&mut game);
    assert_eq!(game.counter_count(ally, CounterKind::PlusOne), 2);
    until(&mut game, |game| {
        game.object_view(enchantment).zone == Zone::Exile
    });
}

#[test]
fn meltstriders_gear_attaches_on_entry_then_moves_its_stats_and_reach_with_equip() {
    let player = Player::P0;
    let mut game = scenario(player);
    let first = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let second = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let enemy = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let gear = put(&mut game, player, "Meltstrider's Gear", Zone::Hand);
    cast(&mut game, gear, None);
    resolve_top(&mut game);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(enemy),
        },
    );
    target(&mut game, Target::Card(first));
    settle(&mut game);
    assert_eq!(stats(&game, first), (4, 3));
    assert!(game.object_view(first).keywords.contains(Keywords::REACH));

    act(&mut game, activation(gear));
    target(&mut game, Target::Card(second));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(
        game.object_view(gear).attached_to,
        Some(game.object_ref(second))
    );
    assert_eq!(stats(&game, first), (2, 2));
    assert!(!game.object_view(first).keywords.contains(Keywords::REACH));
    assert_eq!(stats(&game, second), (4, 3));
    assert!(game.object_view(second).keywords.contains(Keywords::REACH));
}
