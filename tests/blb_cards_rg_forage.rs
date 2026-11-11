//! Forage routes, exact graveyard links, and Squirrel or Food qualification.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn food(game: &mut Game, player: Player) -> CardId {
    game.create_tokens(player, Card::from_catalogue_id(5246), 1, false);
    assert!(!game.advance_entries());
    game.controlled_token_ids(player).last().unwrap()
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn drain(game: &mut Game, choice: impl Fn(&Action) -> bool) {
    for _ in 0..48 {
        if game.state.pending == Pending::Priority
            && game.state.stack.is_empty()
            && game.state.pending_triggers.is_empty()
        {
            return;
        }
        choose(game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. }) || choice(action)
        });
    }
    panic!("forage and entry decisions did not settle");
}

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn forage_pick(game: &Game, card: CardId) -> Action {
    Action::ChooseForage {
        object: game.object_ref(card),
    }
}

fn activate(game: &mut Game, source: CardId, slot: u16, chosen: Option<CardId>) {
    choose(game, |action| match action {
        Action::Activate {
            card,
            ability,
            target,
            ..
        } => *card == source && *ability == slot && *target == chosen.map(Target::Card),
        Action::AnnounceActivate { card, ability, .. } => *card == source && *ability == slot,
        _ => false,
    });
    if let Some(chosen) = chosen
        && matches!(game.state.pending, Pending::Targets { .. })
    {
        act(game, target(chosen));
    }
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(game);
    }
}

#[test]
fn cache_grab_creates_food_for_either_squirrel_condition() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let land = put(&mut game, player, "Forest", Zone::Library);
    let instant = put(&mut game, player, "Shock", Zone::Library);
    let squirrel = put(&mut game, player, "Bakersbane Duo", Zone::Library);
    put(&mut game, player, "Mountain", Zone::Library);
    let spell = put(&mut game, player, "Cache Grab", Zone::Hand);
    cast(&mut game, spell, None);
    pass_pair(&mut game);
    let milled = |game: &Game, card| Action::SelectMilled {
        object: game.object_ref(card),
    };
    let wrong_type = milled(&game, instant);
    reject(&mut game, wrong_type);
    assert!(game.observe(opponent).milled_choice.is_none());
    assert!(game.observe(player).milled_choice.is_some());

    let mut return_squirrel = game.clone();
    let action = milled(&return_squirrel, squirrel);
    act(&mut return_squirrel, action);
    assert_eq!(return_squirrel.object_view(squirrel).zone, Zone::Hand);
    assert_eq!(return_squirrel.controlled_token_ids(player).count(), 1);

    let mut keep_squirrel = game.clone();
    put(
        &mut keep_squirrel,
        player,
        "Bakersbane Duo",
        Zone::Battlefield,
    );
    let action = milled(&keep_squirrel, land);
    act(&mut keep_squirrel, action);
    assert_eq!(keep_squirrel.object_view(land).zone, Zone::Hand);
    assert_eq!(keep_squirrel.controlled_token_ids(player).count(), 1);

    act(&mut game, Action::FinishMilledSelection);
    assert_eq!(game.object_view(squirrel).zone, Zone::Graveyard);
    assert_eq!(game.controlled_token_ids(player).count(), 0);
}

#[test]
fn dreyleader_counts_other_squirrels_and_food_and_gains_one_per_entrant() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    put(&mut game, opponent, "Bakersbane Duo", Zone::Battlefield);
    food(&mut game, opponent);
    put(&mut game, player, "Bakersbane Duo", Zone::Battlefield);
    let early_food = food(&mut game, player);
    let leader = put(&mut game, player, "Honored Dreyleader", Zone::Hand);
    cast(&mut game, leader, None);
    pass_pair(&mut game);
    game.move_object(early_food, Zone::Graveyard);
    settle(&mut game);
    assert_eq!(stats(&game, leader), (2, 2));
    assert!(game.keywords(leader).contains(Keywords::TRAMPLE));

    let duo = put(&mut game, player, "Bakersbane Duo", Zone::Hand);
    cast(&mut game, duo, None);
    drain(&mut game, |_| false);
    assert_eq!(stats(&game, leader), (4, 4));
}

#[test]
fn thornvault_makes_mana_immediately_with_either_forage_route_and_tutors_a_squirrel() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let forager = put(&mut game, player, "Thornvault Forager", Zone::Battlefield);
    let food = food(&mut game, player);
    let grave: [CardId; 3] =
        std::array::from_fn(|_| put(&mut game, player, "Forest", Zone::Graveyard));
    game.state.players[player.index()].mana = [0; 6];
    let green = [0, 0, 0, 0, 1, 0];
    let split = [1, 0, 0, 1, 0, 0];
    let simple = Action::ActivateMana {
        card: forager,
        ability: 0,
        payment: [0; 6],
        counter_payment: Counters::default(),
        production: green,
    };
    game.state.objects[forager as usize].summoning_sick = true;
    reject(&mut game, simple.clone());
    game.state.objects[forager as usize].summoning_sick = false;
    let mut regular = game.clone();
    act(&mut regular, simple);
    assert!(regular.state.stack.is_empty());
    assert_eq!(regular.state.players[player.index()].mana, green);

    for route in [ForageRoute::Food, ForageRoute::ExileThree] {
        let mut branch = game.clone();
        choose(&mut branch, |action| {
            matches!(action, Action::AnnounceMana { card, ability: 1, production, .. }
                if *card == forager && (*production == [0; 6] || *production == split))
        });
        assert!(branch.observe(opponent).forage.is_none());
        act(&mut branch, Action::ChooseForageRoute { route });
        let food_payment = [food];
        let payments = if route == ForageRoute::Food {
            &food_payment[..]
        } else {
            &grave[..]
        };
        for &card in payments {
            let action = forage_pick(&branch, card);
            act(&mut branch, action);
        }
        pay(&mut branch);
        if matches!(branch.state.pending, Pending::EffectMana { .. }) {
            act(&mut branch, Action::ChooseMana { production: split });
        }
        assert!(branch.state.stack.is_empty());
        assert!(branch.object_view(forager).tapped);
        assert_eq!(branch.state.players[player.index()].mana, split);
        for &card in payments {
            assert_ne!(
                branch.object_view(card).zone,
                if route == ForageRoute::Food {
                    Zone::Battlefield
                } else {
                    Zone::Graveyard
                }
            );
        }
    }

    game.state.players[player.index()].mana = [0, 0, 0, 0, 1, 3];
    let squirrel = put(&mut game, player, "Bakersbane Duo", Zone::Library);
    let bear = put(&mut game, player, "Grizzly Bears", Zone::Library);
    activate(&mut game, forager, 2, None);
    pass_pair(&mut game);
    reject(&mut game, Action::ChooseSearch { card: bear });
    act(&mut game, Action::ChooseSearch { card: squirrel });
    drain(&mut game, |action| matches!(action, Action::FinishSearch));
    assert_eq!(game.object_view(squirrel).zone, Zone::Hand);
    assert_eq!(game.object_view(bear).zone, Zone::Library);
    assert!(game.object_view(forager).tapped);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
}

#[test]
fn food_is_a_token_for_druid_and_buys_curious_foragers_reflexive_return() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let druid = put(&mut game, player, "Druid of the Spade", Zone::Battlefield);
    food(&mut game, opponent);
    assert_eq!(stats(&game, druid), (2, 3));
    let food = food(&mut game, player);
    assert_eq!(stats(&game, druid), (4, 3));
    assert!(game.keywords(druid).contains(Keywords::TRAMPLE));
    let returned = put(&mut game, player, "Forest", Zone::Graveyard);
    let foreign = put(&mut game, opponent, "Forest", Zone::Graveyard);
    let forager = put(&mut game, player, "Curious Forager", Zone::Hand);
    cast(&mut game, forager, None);
    pass_pair(&mut game);
    pass_pair(&mut game);

    let mut declined = game.clone();
    act(&mut declined, Action::DeclineForage);
    assert_eq!(declined.object_view(returned).zone, Zone::Graveyard);
    assert_eq!(stats(&declined, druid), (4, 3));

    act(
        &mut game,
        Action::ChooseForageRoute {
            route: ForageRoute::Food,
        },
    );
    let pick = forage_pick(&game, food);
    act(&mut game, pick);
    if matches!(game.state.pending, Pending::Targets { .. }) {
        reject(&mut game, target(foreign));
    }
    drain(&mut game, |action| *action == target(returned));
    assert_eq!(game.object_view(returned).zone, Zone::Hand);
    assert_eq!(game.object_view(foreign).zone, Zone::Graveyard);
    assert_eq!(stats(&game, druid), (2, 3));
    assert!(!game.keywords(druid).contains(Keywords::TRAMPLE));
}

#[test]
fn rootweaver_makes_each_color_and_sentries_draw_only_after_forage() {
    let player = Player::P1;
    let mut game = scenario(player);
    let rootweaver = put(
        &mut game,
        player,
        "Three Tree Rootweaver",
        Zone::Battlefield,
    );
    game.state.players[player.index()].mana = [0; 6];
    for color in 0..6 {
        let mut production = [0; 6];
        production[color] = 1;
        let mut branch = game.clone();
        let action = Action::ActivateMana {
            card: rootweaver,
            ability: 0,
            payment: [0; 6],
            counter_payment: Counters::default(),
            production,
        };
        if color < 5 {
            act(&mut branch, action);
            assert_eq!(branch.state.players[player.index()].mana, production);
            assert!(branch.state.stack.is_empty());
        } else {
            reject(&mut branch, action);
        }
    }
    game.state.players[player.index()].mana = [8; 6];
    let library = put(&mut game, player, "Shock", Zone::Library);
    let grave: [CardId; 3] =
        std::array::from_fn(|_| put(&mut game, player, "Forest", Zone::Graveyard));
    let sentries = put(&mut game, player, "Treetop Sentries", Zone::Hand);
    cast(&mut game, sentries, None);
    pass_pair(&mut game);
    pass_pair(&mut game);
    let mut declined = game.clone();
    act(&mut declined, Action::DeclineForage);
    assert_eq!(declined.object_view(library).zone, Zone::Library);
    act(
        &mut game,
        Action::ChooseForageRoute {
            route: ForageRoute::ExileThree,
        },
    );
    for card in grave {
        let action = forage_pick(&game, card);
        act(&mut game, action);
        assert_eq!(
            game.object_view(card).zone,
            if card == grave[2] {
                Zone::Exile
            } else {
                Zone::Graveyard
            }
        );
    }
    assert_eq!(game.object_view(library).zone, Zone::Hand);
    assert!(game.keywords(sentries).contains(Keywords::REACH));
}

#[test]
fn keen_eyed_curator_checks_distinct_types_on_exact_linked_cards() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let curator = put(&mut game, player, "Keen-Eyed Curator", Zone::Battlefield);
    let land = put(&mut game, opponent, "Forest", Zone::Graveyard);
    let instant = put(&mut game, player, "Shock", Zone::Graveyard);
    let creature = put(&mut game, opponent, "Grizzly Bears", Zone::Graveyard);
    let artifact = put(&mut game, player, "Candy Trail", Zone::Graveyard);
    put(&mut game, player, "Mountain", Zone::Exile);
    for card in [land, instant, creature] {
        activate(&mut game, curator, 0, Some(card));
        settle(&mut game);
        assert_eq!(game.object_view(card).zone, Zone::Exile);
        assert_eq!(stats(&game, curator), (3, 3));
        assert!(!game.keywords(curator).contains(Keywords::TRAMPLE));
    }
    activate(&mut game, curator, 0, Some(artifact));
    settle(&mut game);
    assert_eq!(stats(&game, curator), (7, 7));
    assert!(game.keywords(curator).contains(Keywords::TRAMPLE));

    game.move_object(artifact, Zone::Graveyard);
    game.move_object(artifact, Zone::Exile);
    assert_eq!(stats(&game, curator), (3, 3));
    assert!(!game.keywords(curator).contains(Keywords::TRAMPLE));
}
