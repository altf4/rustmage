use crate::{reach, support::*};
use rustmage_core::{cards::*, engine::*};

fn pick_target(game: &mut Game, target: Target) {
    act(game, Action::ChooseTarget { target });
}

fn finish_search(game: &mut Game, cards: &[CardId]) {
    for &card in cards {
        act(game, Action::ChooseSearch { card });
    }
    act(game, Action::FinishSearch);
}

fn chapter(game: &mut Game) {
    game.begin_step(Step::Main1);
    game.prepare_triggers();
}

#[test]
fn bumi_captures_the_mode_count_and_leaves_the_target_players_scry_private() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let bumi = put(&mut game, player, "Bumi, King of Three Trials", Zone::Hand);
    let land = put(&mut game, player, "Forest", Zone::Battlefield);
    put(&mut game, opponent, "Firebending Lesson", Zone::Graveyard);
    let all_modes = Action::ChooseMode { mode: 7 };
    let mut none = game.clone();
    cast(&mut none, bumi, None);
    reach(&mut none, |g| {
        g.legal_actions().contains(&Action::ChooseMode { mode: 0 })
    });
    assert_eq!(
        none.legal_actions()
            .iter()
            .filter(|action| matches!(action, Action::ChooseMode { .. }))
            .count(),
        1
    );
    act(&mut none, Action::ChooseMode { mode: 0 });
    settle(&mut none);
    assert_eq!(stats(&none, bumi), (4, 4));

    let lessons = [
        "Firebending Lesson",
        "Earthbending Lesson",
        "Cycle of Renewal",
    ]
    .map(|name| put(&mut game, player, name, Zone::Graveyard));
    cast(&mut game, bumi, None);
    reach(&mut game, |g| g.legal_actions().contains(&all_modes));
    game.move_object(lessons[0], Zone::Exile);
    act(&mut game, all_modes);
    pick_target(&mut game, Target::Player(opponent));
    pick_target(&mut game, Target::Card(land));
    reach(&mut game, |g| g.observe(opponent).library_choice.is_some());
    assert_eq!(game.actor(), Some(opponent));
    assert!(game.observe(player).library_choice.is_none());
    let rows = game.observe(opponent).library_choice.unwrap().remaining;
    assert_eq!(rows.len(), 3);
    for card in rows {
        act(
            &mut game,
            Action::ArrangeLibrary {
                card: card.id,
                destination: LibraryDestination::Bottom,
            },
        );
    }
    settle(&mut game);
    assert_eq!(stats(&game, bumi), (7, 7));
    assert_eq!(stats(&game, land), (3, 3));
}

#[test]
fn kyoshi_draws_by_power_then_bends_by_hand_size_and_grants_land_abilities() {
    let player = Player::P0;
    let mut game = scenario(player);
    let saga = put(
        &mut game,
        player,
        "The Legend of Kyoshi // Avatar Kyoshi",
        Zone::Hand,
    );
    put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let forest = put(&mut game, player, "Forest", Zone::Battlefield);
    let ordinary = put(&mut game, player, "Mountain", Zone::Battlefield);
    cast(&mut game, saga, None);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 3);
    let exact = game.object_ref(saga);
    chapter(&mut game);
    reach(&mut game, |g| {
        matches!(g.state.pending, Pending::Targets { .. })
    });
    pick_target(&mut game, Target::Card(forest));
    settle(&mut game);
    let island_forest = game.object_view(forest);
    assert!(island_forest.has_subtype("Island"));
    assert!(island_forest.has_subtype("Forest"));
    assert_eq!(stats(&game, forest), (3, 3));
    chapter(&mut game);
    settle(&mut game);
    assert_ne!(game.object_ref(saga), exact);
    assert_eq!(
        game.object_view(saga).card,
        Card::from_name("Avatar Kyoshi").unwrap()
    );
    assert_eq!(stats(&game, saga), (5, 4));
    for card in [forest, ordinary] {
        assert!(
            game.keywords(card)
                .contains(Keywords::TRAMPLE.union(Keywords::HEXPROOF))
        );
    }
    game.state.objects[saga as usize].summoning_sick = false;
    game.state.players[player.index()].mana = [0; 6];
    let activate = |production| Action::ActivateMana {
        card: saga,
        ability: 0,
        payment: [0; 6],
        counter_payment: Counters::default(),
        production,
    };
    reject(&mut game, activate([0, 2, 0, 3, 0, 0]));
    act(&mut game, activate([0, 5, 0, 0, 0, 0]));
    assert_eq!(game.state.players[player.index()].mana, [0, 5, 0, 0, 0, 0]);
}

#[test]
fn the_earth_king_makes_a_bear_and_remembers_the_qualifying_declared_attackers() {
    let player = Player::P1;
    let mut game = scenario(player);
    let king = put(&mut game, player, "The Earth King", Zone::Hand);
    let dragon = put(&mut game, player, "Shivan Dragon", Zone::Battlefield);
    let small = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let [first, second, third] =
        ["Island", "Mountain", "Swamp"].map(|name| put(&mut game, player, name, Zone::Library));
    cast(&mut game, king, None);
    settle(&mut game);
    let bear = (0..game.state.objects.len())
        .map(|id| game.object_view(id as CardId))
        .find(|object| {
            object.zone == Zone::Battlefield && object.is_token() && object.has_subtype("Bear")
        })
        .unwrap()
        .id;
    assert_eq!(stats(&game, bear), (4, 4));
    assert_eq!(game.object_view(bear).colors, 16);
    game.state.objects[bear as usize].summoning_sick = false;
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    for card in [bear, dragon, small] {
        act(&mut game, Action::Attack { card });
    }
    act(&mut game, Action::FinishAttackers);
    game.move_object(dragon, Zone::Exile);
    reach(&mut game, |g| {
        g.legal_actions()
            .contains(&Action::ChooseSearch { card: first })
    });
    act(&mut game, Action::ChooseSearch { card: first });
    act(&mut game, Action::ChooseSearch { card: second });
    reject(&mut game, Action::ChooseSearch { card: third });
    finish_search(&mut game, &[]);
    settle(&mut game);
    for card in [first, second] {
        assert_eq!(game.object_view(card).zone, Zone::Battlefield);
        assert!(game.object_view(card).tapped);
    }
}

#[test]
fn the_shrines_count_their_current_cohort_and_observe_each_others_entry() {
    let player = Player::P0;
    let mut game = scenario(player);
    let plaza = put(&mut game, player, "Kyoshi Island Plaza", Zone::Hand);
    let temple = put(&mut game, player, "Crescent Island Temple", Zone::Hand);
    let [first, second, third] =
        ["Island", "Mountain", "Swamp"].map(|name| put(&mut game, player, name, Zone::Library));
    let monks = |g: &Game| {
        (0..g.state.objects.len())
            .map(|id| g.object_view(id as CardId))
            .filter(|object| {
                object.zone == Zone::Battlefield && object.is_token() && object.has_subtype("Monk")
            })
            .map(|object| object.id)
            .collect::<Vec<_>>()
    };

    let mut plaza_enters = game.clone();
    plaza_enters.move_object(temple, Zone::Battlefield);
    plaza_enters.prepare_triggers();
    settle(&mut plaza_enters);
    assert_eq!(monks(&plaza_enters).len(), 1);
    cast(&mut plaza_enters, plaza, None);
    reach(&mut plaza_enters, |g| {
        g.legal_actions()
            .contains(&Action::ChooseSearch { card: first })
    });
    act(&mut plaza_enters, Action::ChooseSearch { card: first });
    act(&mut plaza_enters, Action::ChooseSearch { card: second });
    reject(&mut plaza_enters, Action::ChooseSearch { card: third });
    finish_search(&mut plaza_enters, &[]);
    reach(&mut plaza_enters, |g| {
        g.state.stack.is_empty() && g.state.pending == Pending::Priority
    });
    assert_eq!(monks(&plaza_enters).len(), 2);
    for card in [first, second] {
        assert!(plaza_enters.object_view(card).tapped);
    }

    cast(&mut game, plaza, None);
    reach(&mut game, |g| {
        g.legal_actions()
            .contains(&Action::ChooseSearch { card: first })
    });
    finish_search(&mut game, &[first]);
    settle(&mut game);
    cast(&mut game, temple, None);
    reach(&mut game, |g| {
        g.legal_actions()
            .contains(&Action::ChooseSearch { card: second })
    });
    act(&mut game, Action::ChooseSearch { card: second });
    reject(&mut game, Action::ChooseSearch { card: third });
    finish_search(&mut game, &[]);
    reach(&mut game, |g| {
        g.state.stack.is_empty() && g.state.pending == Pending::Priority
    });
    let made = monks(&game);
    assert_eq!(made.len(), 2);
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, shock, Some(Target::Player(player.other())));
    reach(&mut game, |g| {
        g.state.stack.is_empty() && g.state.pending == Pending::Priority
    });
    for monk in made {
        assert_eq!(stats(&game, monk), (2, 2));
    }
}
