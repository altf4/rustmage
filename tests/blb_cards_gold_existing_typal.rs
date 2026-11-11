//! Inherited Bloomburrow typal abilities, hybrid costs, and rule histories.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn next(game: &mut Game) {
    choose(game, |action| {
        matches!(action, Action::Pass | Action::PutTrigger { .. })
    });
}

fn idle(game: &mut Game) {
    for _ in 0..40 {
        if game.state.stack.is_empty()
            && game.state.pending_triggers.is_empty()
            && game.state.ordering_triggers.is_empty()
            && game.state.pending == Pending::Priority
        {
            return;
        }
        next(game);
    }
    panic!("unresolved scenario choice in {:?}", game.state.pending);
}

fn start_step(game: &mut Game, step: Step) {
    game.begin_step(step);
    game.give_priority();
    game.prepare_triggers();
}

fn activate(game: &mut Game, card: CardId) {
    act(
        game,
        Action::AnnounceActivate {
            card,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
}

fn forage(game: &mut Game, route: ForageRoute, cards: &[CardId]) {
    act(game, Action::ChooseForageRoute { route });
    for &card in cards {
        let object = game.object_ref(card);
        act(game, Action::ChooseForage { object });
    }
}

#[test]
fn burrowguard_tracks_all_own_creatures_as_head_of_the_homestead_makes_two_rabbits() {
    let player = Player::P0;
    let mut game = scenario(player);
    let burrow = put(&mut game, player, "Burrowguard Mentor", Zone::Battlefield);
    let homestead = put(&mut game, player, "Head of the Homestead", Zone::Hand);
    put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    assert_eq!(stats(&game, burrow), (1, 1));
    assert!(game.keywords(burrow).contains(Keywords::TRAMPLE));

    game.state.players[player.index()].mana = [1, 0, 0, 0, 1, 3];
    cast(&mut game, homestead, None);
    idle(&mut game);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    let rabbits = game.controlled_token_ids(player).collect::<Vec<_>>();
    assert_eq!(rabbits.len(), 2);
    for &token in &rabbits {
        assert!(game.has_subtype(token, "Rabbit"));
        assert_eq!(stats(&game, token), (1, 1));
    }
    assert_eq!(stats(&game, burrow), (4, 4));
    game.move_object(rabbits[0], Zone::Graveyard);
    assert_eq!(stats(&game, burrow), (3, 3));
}

#[test]
fn camellia_counts_other_squirrels_including_kindred_and_makes_a_squirrel_for_sacrificing_food() {
    let player = Player::P1;
    let mut game = scenario(player);
    let camellia = put(
        &mut game,
        player,
        "Camellia, the Seedmiser",
        Zone::Battlefield,
    );
    let core = put(&mut game, player, "Firdoch Core", Zone::Battlefield);
    let squirrel = put(&mut game, player, "Barkform Harvester", Zone::Battlefield);
    let bear = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let opposing = put(
        &mut game,
        player.other(),
        "Barkform Harvester",
        Zone::Battlefield,
    );
    let food = put(&mut game, player, "Gingerbrute", Zone::Battlefield);
    assert!(!game.is_creature(core));
    for id in [camellia, core, squirrel] {
        assert!(game.keywords(id).contains(Keywords::MENACE));
    }
    for id in [bear, opposing] {
        assert!(!game.keywords(id).contains(Keywords::MENACE));
    }

    activate(&mut game, camellia);
    forage(&mut game, ForageRoute::Food, &[food]);
    act(
        &mut game,
        Action::PayCosts {
            payment: [0, 0, 0, 0, 0, 2],
        },
    );
    idle(&mut game);
    assert_eq!(game.object_view(food).zone, Zone::Graveyard);
    let created = game.controlled_token_ids(player).next().unwrap();
    assert!(game.has_subtype(created, "Squirrel"));
    assert!(game.keywords(created).contains(Keywords::MENACE));
    for id in [core, squirrel, created] {
        assert_eq!(game.counter_count(id, CounterKind::PlusOne), 1);
    }
    for id in [camellia, bear, opposing] {
        assert_eq!(game.counter_count(id, CounterKind::PlusOne), 0);
    }
}

#[test]
fn cindering_cutthroat_uses_opponent_life_loss_and_has_two_menace_payment_colors() {
    let player = Player::P0;
    for color in [2, 3] {
        let mut game = scenario(player);
        let lizard = put(&mut game, player, "Cindering Cutthroat", Zone::Hand);
        game.lose_life(player.other(), 1);
        game.gain_life(player.other(), 1);
        let mut mana = [0, 0, 0, 0, 0, 2];
        mana[color] = 1;
        game.state.players[player.index()].mana = mana;
        cast(&mut game, lizard, None);
        idle(&mut game);
        assert_eq!(game.counter_count(lizard, CounterKind::PlusOne), 1);
        mana[5] = 1;
        game.state.players[player.index()].mana = mana;
        activate(&mut game, lizard);
        choose(&mut game, |action| {
            matches!(action, Action::ChooseActivationCost { .. })
        });
        act(&mut game, Action::PayCosts { payment: mana });
        idle(&mut game);
        assert!(game.keywords(lizard).contains(Keywords::MENACE));
        game.finish_cleanup();
        assert!(!game.keywords(lizard).contains(Keywords::MENACE));
    }
}

#[test]
fn corpseberry_can_forage_with_food_or_three_grave_cards_and_counters_only_after_forage() {
    let player = Player::P1;
    let mut game = scenario(player);
    let cultivator = put(
        &mut game,
        player,
        "Corpseberry Cultivator",
        Zone::Battlefield,
    );
    let food = put(&mut game, player, "Gingerbrute", Zone::Battlefield);
    let grave = [(); 3].map(|_| put(&mut game, player, "Forest", Zone::Graveyard));
    start_step(&mut game, Step::BeginCombat);
    next(&mut game);
    next(&mut game);
    assert!(matches!(game.state.pending, Pending::Forage { .. }));

    let mut declined = game.clone();
    act(&mut declined, Action::DeclineForage);
    idle(&mut declined);
    assert_eq!(declined.counter_count(cultivator, CounterKind::PlusOne), 0);

    for (route, cards, destination) in [
        (ForageRoute::Food, [food].as_slice(), Zone::Graveyard),
        (ForageRoute::ExileThree, grave.as_slice(), Zone::Exile),
    ] {
        let mut branch = game.clone();
        forage(&mut branch, route, cards);
        idle(&mut branch);
        assert_eq!(branch.counter_count(cultivator, CounterKind::PlusOne), 1);
        assert!(
            cards
                .iter()
                .all(|&id| branch.object_view(id).zone == destination)
        );
    }
}

#[test]
fn mind_drill_gets_live_threshold_from_its_own_surveil_and_tidecaller_rechecks_threshold() {
    let player = Player::P0;
    let mut game = scenario(player);
    let mind = put(&mut game, player, "Mind Drill Assailant", Zone::Battlefield);
    let old = put(&mut game, player, "Opt", Zone::Graveyard);
    for _ in 0..5 {
        put(&mut game, player, "Forest", Zone::Graveyard);
    }
    let top = put(&mut game, player, "Island", Zone::Library);
    let tide = put(&mut game, player, "Tidecaller Mentor", Zone::Hand);
    let enemy = put(&mut game, player.other(), "Short Bow", Zone::Battlefield);
    let enemy_land = put(&mut game, player.other(), "Forest", Zone::Battlefield);
    assert_eq!(stats(&game, mind), (2, 5));
    game.state.players[player.index()].mana = [0, 1, 0, 0, 0, 2];
    activate(&mut game, mind);
    choose(&mut game, |action| {
        matches!(action, Action::ChooseActivationCost { .. })
    });
    act(
        &mut game,
        Action::PayCosts {
            payment: [0, 1, 0, 0, 0, 2],
        },
    );
    next(&mut game);
    next(&mut game);
    act(
        &mut game,
        Action::ArrangeLibrary {
            card: top,
            destination: LibraryDestination::Graveyard,
        },
    );
    assert_eq!(stats(&game, mind), (5, 5));
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);

    game.state.players[player.index()].mana = [0, 1, 1, 0, 0, 1];
    cast(&mut game, tide, None);
    next(&mut game);
    next(&mut game);
    assert!(game.keywords(tide).contains(Keywords::MENACE));
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(enemy_land),
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(enemy),
        },
    );
    let mut lost = game.clone();
    lost.move_object(old, Zone::Exile);
    idle(&mut lost);
    assert_eq!(stats(&lost, mind), (2, 5));
    assert_eq!(lost.object_view(enemy).zone, Zone::Battlefield);
    idle(&mut game);
    assert_eq!(game.object_view(enemy).zone, Zone::Hand);
}

#[test]
fn moonrise_cleric_flies_and_gains_life_when_declared_as_an_attacker() {
    let player = Player::P1;
    let mut game = scenario(player);
    let cleric = put(&mut game, player, "Moonrise Cleric", Zone::Hand);
    game.state.players[player.index()].mana = [1, 0, 1, 0, 0, 1];
    cast(&mut game, cleric, None);
    idle(&mut game);
    assert!(game.keywords(cleric).contains(Keywords::FLYING));
    assert_eq!(game.state.players[player.index()].life, 20);
    game.state.objects[cleric as usize].summoning_sick = false;
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    game.state.priority = player;
    act(&mut game, Action::Attack { card: cleric });
    act(&mut game, Action::FinishAttackers);
    idle(&mut game);
    assert_eq!(game.state.players[player.index()].life, 21);
}

#[test]
fn pond_prophet_draws_when_cast_with_any_hybrid_combination() {
    let player = Player::P0;
    for (blue, green) in [(0, 2), (1, 1), (2, 0)] {
        let mut game = scenario(player);
        let frog = put(&mut game, player, "Pond Prophet", Zone::Hand);
        game.state.players[player.index()].mana = [0, blue, 0, 0, green, 0];
        cast(&mut game, frog, None);
        idle(&mut game);
        assert_eq!(game.state.players[player.index()].mana, [0; 6]);
        assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
    }
}

#[test]
fn valiant_pays_off_once_for_own_targets_on_seedglaive_and_guardmouse() {
    let player = Player::P1;
    let mut game = scenario(player);
    let mentor = put(&mut game, player, "Seedglaive Mentor", Zone::Battlefield);
    let guard = put(&mut game, player, "Veteran Guardmouse", Zone::Battlefield);
    assert!(
        game.keywords(mentor)
            .contains(Keywords::HASTE.union(Keywords::VIGILANCE))
    );
    let top = put(&mut game, player, "Island", Zone::Library);
    let opposing = put(&mut game, player.other(), "Giant Growth", Zone::Hand);
    act(&mut game, Action::Pass);
    cast(&mut game, opposing, Some(Target::Card(guard)));
    idle(&mut game);
    assert!(!game.keywords(guard).contains(Keywords::FIRST_STRIKE));

    for recipient in [mentor, mentor, guard] {
        let pump = put(&mut game, player, "Giant Growth", Zone::Hand);
        cast(&mut game, pump, Some(Target::Card(recipient)));
        if recipient == guard {
            next(&mut game);
            next(&mut game);
            let looked = game.observe(player).library_choice.unwrap();
            assert!(!looked.surveil);
            act(
                &mut game,
                Action::ArrangeLibrary {
                    card: top,
                    destination: LibraryDestination::Bottom,
                },
            );
        }
        idle(&mut game);
    }
    assert_eq!(game.counter_count(mentor, CounterKind::PlusOne), 1);
    assert_eq!(stats(&game, guard), (10, 10));
    assert!(game.keywords(guard).contains(Keywords::FIRST_STRIKE));
    game.finish_cleanup();
    assert_eq!(stats(&game, mentor), (4, 3));
    assert_eq!(stats(&game, guard), (3, 4));
    assert!(!game.keywords(guard).contains(Keywords::FIRST_STRIKE));
}

#[test]
fn vinereap_rewards_entry_and_death_but_vren_replaces_death_and_builds_scaling_rats() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let mentor = put(&mut game, player, "Vinereap Mentor", Zone::Hand);
    cast(&mut game, mentor, None);
    idle(&mut game);
    assert_eq!(
        game.controlled_token_ids(player)
            .filter(|&id| game.has_subtype(id, "Food"))
            .count(),
        1
    );
    let mut died = game.clone();
    died.move_object(mentor, Zone::Graveyard);
    died.prepare_triggers();
    idle(&mut died);
    assert_eq!(
        died.controlled_token_ids(player)
            .filter(|&id| died.has_subtype(id, "Food"))
            .count(),
        2
    );

    put(
        &mut game,
        opponent,
        "Vren, the Relentless",
        Zone::Battlefield,
    );
    game.move_object(mentor, Zone::Graveyard);
    assert_eq!(game.object_view(mentor).zone, Zone::Exile);
    let extra = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    game.move_object(extra, Zone::Exile);
    start_step(&mut game, Step::End);
    idle(&mut game);
    assert_eq!(
        game.controlled_token_ids(player)
            .filter(|&id| game.has_subtype(id, "Food"))
            .count(),
        1
    );
    let rats = game.controlled_token_ids(opponent).collect::<Vec<_>>();
    assert_eq!(rats.len(), 2);
    for rat in rats {
        assert!(game.has_subtype(rat, "Rat"));
        assert_eq!(stats(&game, rat), (3, 3));
    }
}

#[test]
fn valiant_observes_the_first_target_of_each_controller_on_the_same_incarnation() {
    let owner = Player::P0;
    for (name, scries) in [("Seedglaive Mentor", false), ("Veteran Guardmouse", true)] {
        let mut game = scenario(owner);
        let creature = put(&mut game, owner, name, Zone::Battlefield);
        let reference = game.object_ref(creature);
        for controller in [owner, owner.other()] {
            if controller != owner {
                game.change_controller(reference, controller);
                assert_eq!(game.object_ref(creature), reference);
            }
            if game.actor() != Some(controller) {
                act(&mut game, Action::Pass);
            }
            let top = *game.state.players[controller.index()]
                .library
                .last()
                .unwrap();
            let pump = put(&mut game, controller, "Giant Growth", Zone::Hand);
            cast(&mut game, pump, Some(Target::Card(creature)));
            if scries {
                next(&mut game);
                next(&mut game);
                assert!(game.observe(controller).library_choice.is_some());
                assert!(game.observe(controller.other()).library_choice.is_none());
                act(
                    &mut game,
                    Action::ArrangeLibrary {
                        card: top,
                        destination: LibraryDestination::Top,
                    },
                );
            }
            idle(&mut game);
        }
        if !scries {
            assert_eq!(game.counter_count(creature, CounterKind::PlusOne), 2);
        }
    }
}
