//! Focused behavior for the inherited Malamet through Oltec cards and related spells.
mod support;
use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn activate(card: CardId, ability: u16) -> Action {
    Action::AnnounceActivate {
        card,
        ability,
        counter_payment: Counters::default(),
    }
}

fn reach(game: &mut Game, predicate: impl Fn(&Game) -> bool) {
    for _ in 0..32 {
        if predicate(game) {
            return;
        }
        choose(game, |action| {
            matches!(
                action,
                Action::Pass | Action::PutTrigger { .. } | Action::FinishBlockers
            )
        });
    }
    panic!("missing decision {:?}", game.state.pending);
}

fn reach_action(game: &mut Game, action: &Action) {
    reach(game, |game| game.legal_actions().contains(action));
}

fn attack(game: &mut Game, attackers: &[CardId]) {
    reach(game, |game| game.state.pending == Pending::Attackers);
    for &card in attackers {
        act(game, Action::Attack { card });
    }
    act(game, Action::FinishAttackers);
}

fn tokens(game: &Game, player: Player, recipe: u16) -> Vec<ObjectView> {
    game.observe(player)
        .objects
        .into_iter()
        .filter(|object| {
            object.zone == Zone::Battlefield
                && object.controller == player
                && object.card == Card::from_catalogue_id(recipe)
        })
        .collect()
}

#[test]
fn might_targets_only_a_controlled_creature_and_brawler_targets_a_current_attacker() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        put(
            &mut game,
            player,
            "Might of the Ancestors",
            Zone::Battlefield,
        );
        let brawler = put(&mut game, player, "Malamet Brawler", Zone::Battlefield);
        let ally = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let bench = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let foe = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
        reach_action(&mut game, &target(ally));
        reject(&mut game, target(foe));
        act(&mut game, target(ally));
        settle(&mut game);
        assert_eq!(stats(&game, ally), (4, 2));
        assert!(game.keywords(ally).contains(Keywords::VIGILANCE));
        attack(&mut game, &[brawler, ally]);
        assert!(game.legal_actions().contains(&target(brawler)));
        reject(&mut game, target(bench));
        act(&mut game, target(ally));
        settle(&mut game);
        assert!(game.keywords(ally).contains(Keywords::TRAMPLE));
        assert!(!game.object_view(ally).tapped);
        assert!(!game.keywords(brawler).contains(Keywords::TRAMPLE));
    }
}

#[test]
fn veteran_rechecks_the_live_four_permanent_card_threshold() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let veteran = put(&mut game, player, "Malamet Veteran", Zone::Battlefield);
        let enemy = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
        for name in ["Forest", "Swiftfoot Boots", "Bear Cub", "Shock"] {
            put(&mut game, player, name, Zone::Graveyard);
        }
        assert!(game.keywords(veteran).contains(Keywords::TRAMPLE));
        let mut three = game.clone();
        attack(&mut three, &[veteran]);
        assert!(!three.legal_actions().contains(&target(enemy)));

        let fourth = put(&mut game, player, "Island", Zone::Graveyard);
        attack(&mut game, &[veteran]);
        act(&mut game, target(enemy));
        let mut lost = game.clone();
        lost.move_object(fourth, Zone::Exile);
        settle(&mut lost);
        assert_eq!(lost.counter_count(enemy, CounterKind::PlusOne), 0);
        settle(&mut game);
        assert_eq!(game.counter_count(enemy, CounterKind::PlusOne), 1);
    }
}

#[test]
fn scythe_flashes_attaches_and_can_move_its_bonus_only_at_sorcery_speed() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player.other());
        let a = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let b = put(&mut game, player, "Hill Giant", Zone::Battlefield);
        let foe = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
        let scythe = put(&mut game, player, "Malamet Scythe", Zone::Hand);
        act(&mut game, Action::Pass);
        cast(&mut game, scythe, None);
        reach_action(&mut game, &target(a));
        reject(&mut game, target(foe));
        act(&mut game, target(a));
        settle(&mut game);
        assert_eq!(stats(&game, a), (4, 4));
        act(&mut game, Action::Pass);
        reject(&mut game, activate(scythe, 0));

        game.state.priority = player.other();
        game.state.passes = 0;
        game.begin_step(Step::End);
        reach(&mut game, |game| {
            game.state.active == player && game.state.step == Step::Main1
        });
        game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 4];
        act(&mut game, activate(scythe, 0));
        act(&mut game, target(b));
        pay(&mut game);
        settle(&mut game);
        assert_eq!(stats(&game, a), (2, 2));
        assert_eq!(stats(&game, b), (5, 5));
        assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    }
}

#[test]
fn war_scribe_grows_its_own_resolution_cohort() {
    let player = Player::P0;
    let mut game = scenario(player);
    let scribe = put(&mut game, player, "Malamet War Scribe", Zone::Hand);
    let ally = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let foe = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    cast(&mut game, scribe, None);
    settle(&mut game);
    assert_eq!(stats(&game, scribe), (6, 4));
    assert_eq!(stats(&game, ally), (4, 3));
    assert_eq!(stats(&game, foe), (2, 2));
    let late = put(&mut game, player, "Bear Cub", Zone::Hand);
    cast(&mut game, late, None);
    settle(&mut game);
    assert_eq!(stats(&game, late), (2, 2));
}

#[test]
fn typecycling_can_find_each_printed_nonbasic_land_subtype() {
    for player in [Player::P0, Player::P1] {
        for (name, land, invalid) in [
            ("Marauding Brinefang", "Watery Grave", "Mountain"),
            ("Nurturing Bristleback", "Breeding Pool", "Plains"),
            ("Rampaging Spiketail", "Blood Crypt", "Island"),
        ] {
            let mut game = scenario(player);
            let card = put(&mut game, player, name, Zone::Hand);
            let land = put(&mut game, player, land, Zone::Library);
            let invalid = put(&mut game, player, invalid, Zone::Library);
            game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 1];
            reject(&mut game, activate(card, 0));
            game.state.players[player.index()].mana[5] = 2;
            act(&mut game, activate(card, 0));
            pay(&mut game);
            assert_eq!(game.object_view(card).zone, Zone::Graveyard);
            let search = Action::ChooseSearch { card: land };
            reach_action(&mut game, &search);
            assert!(game.observe(player.other()).search_choice.is_none());
            reject(&mut game, Action::ChooseSearch { card: invalid });
            let mut missed = game.clone();
            act(&mut missed, Action::FinishSearch);
            assert_eq!(missed.object_view(land).zone, Zone::Library);
            act(&mut game, search);
            if game.legal_actions().contains(&Action::FinishSearch) {
                act(&mut game, Action::FinishSearch);
            }
            settle(&mut game);
            assert_eq!(game.object_view(land).zone, Zone::Hand);
        }
    }
}

#[test]
fn brinefang_wards_against_an_opposing_spell_for_exactly_three() {
    let player = Player::P1;
    let mut game = scenario(player.other());
    let brinefang = put(&mut game, player, "Marauding Brinefang", Zone::Battlefield);
    let spell = put(&mut game, player.other(), "Murder", Zone::Hand);
    cast(&mut game, spell, Some(Target::Card(brinefang)));
    reach_action(&mut game, &Action::DeclineResolutionMana);
    let mut decline = game.clone();
    act(&mut decline, Action::DeclineResolutionMana);
    settle(&mut decline);
    assert_eq!(decline.object_view(brinefang).zone, Zone::Battlefield);
    act(
        &mut game,
        Action::PayResolutionMana {
            payment: [0, 0, 0, 0, 0, 3],
        },
    );
    settle(&mut game);
    assert_eq!(game.object_view(brinefang).zone, Zone::Graveyard);
}

#[test]
fn draught_draws_and_loses_one_on_entry_and_artifact_death() {
    let player = Player::P0;
    let mut game = scenario(player);
    let draught = put(&mut game, player, "Mephitic Draught", Zone::Hand);
    cast(&mut game, draught, None);
    settle(&mut game);
    assert_eq!(game.hand_count(player), 1);
    assert_eq!(game.state.players[player.index()].life, STARTING_LIFE - 1);
    let mut exiled = game.clone();
    exiled.move_object(draught, Zone::Exile);
    exiled.prepare_triggers();
    settle(&mut exiled);
    assert_eq!(exiled.hand_count(player), 1);

    let abrade = put(&mut game, player, "Abrade", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: abrade });
    act(&mut game, Action::ChooseMode { mode: 1 });
    act(&mut game, target(draught));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(draught).zone, Zone::Graveyard);
    assert_eq!(game.hand_count(player), 2);
    assert_eq!(game.state.players[player.index()].life, STARTING_LIFE - 2);
}

#[test]
fn guidewing_dies_to_explore_with_another_controlled_creature() {
    let player = Player::P1;
    let mut game = scenario(player);
    let bird = put(&mut game, player, "Miner's Guidewing", Zone::Battlefield);
    let ally = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let foe = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let land = put(&mut game, player, "Mountain", Zone::Library);
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    assert!(
        game.keywords(bird)
            .contains(Keywords::FLYING.union(Keywords::VIGILANCE))
    );
    cast(&mut game, shock, Some(Target::Card(bird)));
    reach_action(&mut game, &target(ally));
    reject(&mut game, target(foe));
    act(&mut game, target(ally));
    settle(&mut game);
    assert_eq!(game.object_view(land).zone, Zone::Hand);
    let explore = game.observe(player.other()).last_explore.unwrap();
    assert_eq!(explore.explorer.object, game.object_ref(ally));
}

#[test]
fn mineshaft_spider_can_decline_or_mill_both_cards_with_only_one_descending() {
    let player = Player::P0;
    let mut game = scenario(player);
    let spider = put(&mut game, player, "Mineshaft Spider", Zone::Hand);
    let first = put(&mut game, player, "Bear Cub", Zone::Library);
    let second = put(&mut game, player, "Shock", Zone::Library);
    cast(&mut game, spider, None);
    let accept = Action::ChooseOptional { accept: true };
    reach_action(&mut game, &accept);
    let mut decline = game.clone();
    act(&mut decline, Action::ChooseOptional { accept: false });
    settle(&mut decline);
    assert_eq!(decline.object_view(second).zone, Zone::Library);
    act(&mut game, accept);
    settle(&mut game);
    assert!(game.keywords(spider).contains(Keywords::REACH));
    for card in [first, second] {
        assert_eq!(game.object_view(card).zone, Zone::Graveyard);
    }
    assert_eq!(
        game.observe(player).players[player.index()].descended_this_turn,
        1
    );
}

#[test]
fn mischievous_pup_flashes_and_bounces_another_controlled_permanent_to_its_owner() {
    let player = Player::P0;
    let mut game = scenario(player.other());
    let pup = put(&mut game, player, "Mischievous Pup", Zone::Hand);
    let stolen = put(
        &mut game,
        player.other(),
        "Swiftfoot Boots",
        Zone::Battlefield,
    );
    let enemy = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    game.change_controller(game.object_ref(stolen), player);
    act(&mut game, Action::Pass);
    cast(&mut game, pup, None);
    reach_action(&mut game, &target(stolen));
    reject(&mut game, target(pup));
    reject(&mut game, target(enemy));
    let mut decline = game.clone();
    act(&mut decline, Action::FinishTargets);
    settle(&mut decline);
    assert_eq!(decline.object_view(stolen).zone, Zone::Battlefield);
    act(&mut game, target(stolen));
    settle(&mut game);
    assert_eq!(game.object_view(stolen).zone, Zone::Hand);
    assert_eq!(game.object_view(stolen).owner, player.other());
}

#[test]
fn river_heralds_and_axejaw_explore_and_apply_nicanzil_and_cave_diver_payoffs() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let conductor = put(
            &mut game,
            player,
            "Nicanzil, Current Conductor",
            Zone::Battlefield,
        );
        let diver = put(&mut game, player, "Merfolk Cave-Diver", Zone::Battlefield);
        let enemy_diver = put(
            &mut game,
            player.other(),
            "Merfolk Cave-Diver",
            Zone::Battlefield,
        );
        let guide = put(&mut game, player, "River Herald Guide", Zone::Hand);
        let prior = put(&mut game, player, "Forest", Zone::Hand);
        let discovered = put(&mut game, player, "Mountain", Zone::Library);
        cast(&mut game, guide, None);
        let land = Action::PutHandLand {
            object: game.object_ref(prior),
        };
        reach_action(&mut game, &land);
        act(&mut game, land);
        settle(&mut game);
        assert!(game.keywords(guide).contains(Keywords::VIGILANCE));
        assert_eq!(game.object_view(discovered).zone, Zone::Hand);
        assert_eq!(game.object_view(prior).zone, Zone::Battlefield);
        assert!(game.object_view(prior).tapped);
        assert_eq!(stats(&game, diver), (3, 4));
        assert_eq!(stats(&game, enemy_diver), (2, 4));

        let scout = put(&mut game, player, "River Herald Scout", Zone::Hand);
        let nonland = put(&mut game, player, "Shock", Zone::Library);
        cast(&mut game, scout, None);
        let choice = Action::ChooseExplore {
            object: game.object_ref(nonland),
            graveyard: true,
        };
        reach_action(&mut game, &choice);
        act(&mut game, choice);
        while matches!(game.state.pending, Pending::OrderTriggers { .. }) {
            choose(&mut game, |a| matches!(a, Action::PutTrigger { .. }));
        }
        settle(&mut game);
        assert_eq!(game.counter_count(scout, CounterKind::PlusOne), 1);
        assert_eq!(game.counter_count(conductor, CounterKind::PlusOne), 1);
        assert_eq!(stats(&game, diver), (4, 4));
        assert!(game.keywords(diver).contains(Keywords::CANT_BE_BLOCKED));

        let axejaw = put(&mut game, player, "Pathfinding Axejaw", Zone::Hand);
        let nonland = put(&mut game, player, "Shock", Zone::Library);
        game.state.players[player.index()].mana = [8; 6];
        cast(&mut game, axejaw, None);
        let choice = Action::ChooseExplore {
            object: game.object_ref(nonland),
            graveyard: false,
        };
        reach_action(&mut game, &choice);
        act(&mut game, choice);
        while matches!(game.state.pending, Pending::OrderTriggers { .. }) {
            choose(&mut game, |a| matches!(a, Action::PutTrigger { .. }));
        }
        settle(&mut game);
        assert_eq!(stats(&game, axejaw), (5, 4));
        assert_eq!(game.object_view(nonland).zone, Zone::Library);
    }
}

#[test]
fn bristleback_and_cloud_guard_create_their_actual_token_recipes() {
    let player = Player::P0;
    let mut game = scenario(player);
    let bristleback = put(&mut game, player, "Nurturing Bristleback", Zone::Hand);
    let guard = put(&mut game, player, "Oltec Cloud Guard", Zone::Hand);
    cast(&mut game, bristleback, None);
    settle(&mut game);
    let dinosaurs = tokens(&game, player, 5327);
    assert_eq!(dinosaurs.len(), 1);
    assert_eq!((dinosaurs[0].power, dinosaurs[0].toughness), (3, 3));
    assert_eq!(dinosaurs[0].colors, 16);
    cast(&mut game, guard, None);
    settle(&mut game);
    let gnomes = tokens(&game, player, 5267);
    assert_eq!(gnomes.len(), 1);
    assert_eq!((gnomes[0].power, gnomes[0].toughness), (1, 1));
    assert_eq!(gnomes[0].kinds & 17, 17);
    assert!(game.keywords(guard).contains(Keywords::FLYING));
}

#[test]
fn archaeologists_chooses_own_artifact_recovery_or_private_scry_three() {
    let player = Player::P0;
    let mut game = scenario(player);
    let card = put(&mut game, player, "Oltec Archaeologists", Zone::Hand);
    let artifact = put(&mut game, player, "Swiftfoot Boots", Zone::Graveyard);
    let foreign = put(
        &mut game,
        player.other(),
        "Swiftfoot Boots",
        Zone::Graveyard,
    );
    cast(&mut game, card, None);
    let recover = Action::ChooseMode { mode: 0 };
    reach_action(&mut game, &recover);
    let mut scry = game.clone();
    act(&mut scry, Action::ChooseMode { mode: 1 });
    reach(&mut scry, |game| {
        game.observe(player).library_choice.is_some()
    });
    let view = scry.observe(player).library_choice.unwrap();
    assert_eq!(view.remaining.len(), 3);
    assert!(scry.observe(player.other()).library_choice.is_none());
    for card in view.remaining {
        act(
            &mut scry,
            Action::ArrangeLibrary {
                card: card.id,
                destination: LibraryDestination::Bottom,
            },
        );
    }
    settle(&mut scry);
    assert_eq!(scry.object_view(artifact).zone, Zone::Graveyard);

    act(&mut game, recover);
    reject(&mut game, target(foreign));
    act(&mut game, target(artifact));
    settle(&mut game);
    assert_eq!(game.object_view(artifact).zone, Zone::Hand);
}

#[test]
fn out_of_air_counters_either_type_and_only_the_creature_costs_two_less() {
    for player in [Player::P0, Player::P1] {
        for (name, generic) in [("Bear Cub", 0), ("Opt", 2)] {
            let mut game = scenario(player.other());
            let spell = put(&mut game, player.other(), name, Zone::Hand);
            let counter = put(&mut game, player, "Out of Air", Zone::Hand);
            cast(&mut game, spell, None);
            act(&mut game, Action::Pass);
            act(&mut game, Action::AnnounceCast { card: counter });
            act(
                &mut game,
                Action::ChooseTarget {
                    target: Target::Spell(spell),
                },
            );
            assert_eq!(
                game.observe(player).payment_request.unwrap().generic,
                generic
            );
            act(
                &mut game,
                Action::PayCosts {
                    payment: [0, 2, 0, 0, 0, generic],
                },
            );
            settle(&mut game);
            assert_eq!(game.object_view(spell).zone, Zone::Graveyard);
        }
    }
}

#[test]
fn over_the_edge_has_distinct_destruction_and_sequential_explore_modes() {
    let player = Player::P0;
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Over the Edge", Zone::Hand);
    let artifact = put(
        &mut game,
        player.other(),
        "Swiftfoot Boots",
        Zone::Battlefield,
    );
    let creature = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let foe = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let revealed = put(&mut game, player, "Shock", Zone::Library);
    act(&mut game, Action::AnnounceCast { card: spell });
    let mut destroy = game.clone();
    act(&mut destroy, Action::ChooseMode { mode: 0 });
    act(&mut destroy, target(artifact));
    pay(&mut destroy);
    settle(&mut destroy);
    assert_eq!(destroy.object_view(artifact).zone, Zone::Graveyard);

    act(&mut game, Action::ChooseMode { mode: 1 });
    reject(&mut game, target(foe));
    act(&mut game, target(creature));
    pay(&mut game);
    for graveyard in [false, true] {
        let choice = Action::ChooseExplore {
            object: game.object_ref(revealed),
            graveyard,
        };
        reach_action(&mut game, &choice);
        act(&mut game, choice);
    }
    settle(&mut game);
    assert_eq!(game.counter_count(creature, CounterKind::PlusOne), 2);
    assert_eq!(game.object_view(revealed).zone, Zone::Graveyard);
}
