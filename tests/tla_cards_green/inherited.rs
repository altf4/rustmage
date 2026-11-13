use crate::{reach, support::*};
use rustmage_core::{cards::*, engine::*};

fn activate(card: CardId) -> Action {
    Action::AnnounceActivate {
        card,
        ability: 0,
        counter_payment: Counters::default(),
    }
}

fn token_count(game: &Game, subtype: &str) -> usize {
    (0..game.state.objects.len())
        .map(|id| game.object_view(id as CardId))
        .filter(|object| {
            object.is_token() && object.zone == Zone::Battlefield && object.has_subtype(subtype)
        })
        .count()
}

#[test]
fn invasion_tactics_grows_the_current_group_and_draws_once_for_two_allies() {
    let player = Player::P1;
    let mut game = scenario(player);
    let first = put(&mut game, player, "Haru, Hidden Talent", Zone::Battlefield);
    let second = put(&mut game, player, "Great Divide Guide", Zone::Battlefield);
    let enchantment = put(&mut game, player, "Invasion Tactics", Zone::Hand);
    cast(&mut game, enchantment, None);
    settle(&mut game);
    assert_eq!(stats(&game, first), (3, 3));
    assert_eq!(stats(&game, second), (4, 5));
    let later = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    assert_eq!(stats(&game, later), (2, 2));
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    for card in [first, second, later] {
        act(&mut game, Action::Attack { card });
    }
    act(&mut game, Action::FinishAttackers);
    for _ in 0..24 {
        if game.state.step == Step::Main2 {
            break;
        }
        choose(&mut game, |action| {
            matches!(action, Action::Pass | Action::FinishBlockers)
        });
    }
    assert_eq!(game.state.step, Step::Main2);
    assert_eq!(game.state.players[player.other().index()].life, 11);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
}

#[test]
fn leaves_from_the_vine_resolves_all_three_chapters() {
    let player = Player::P0;
    let mut game = scenario(player);
    let saga = put(&mut game, player, "Leaves from the Vine", Zone::Hand);
    let first = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let second = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let lesson = put(&mut game, player, "Firebending Lesson", Zone::Library);
    let top = put(&mut game, player, "Island", Zone::Library);
    let other = put(&mut game, player, "Mountain", Zone::Library);
    cast(&mut game, saga, None);
    settle(&mut game);
    assert_eq!(token_count(&game, "Food"), 1);
    for card in [lesson, top, other] {
        assert_eq!(game.object_view(card).zone, Zone::Graveyard);
    }
    game.begin_step(Step::Main1);
    game.prepare_triggers();
    reach(&mut game, |g| {
        matches!(g.state.pending, Pending::Targets { .. })
    });
    for card in [first, second] {
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(card),
            },
        );
    }
    settle(&mut game);
    for card in [first, second] {
        assert_eq!(game.counter_count(card, CounterKind::PlusOne), 1);
    }
    let mut none = game.clone();
    none.move_object(lesson, Zone::Exile);
    for branch in [&mut game, &mut none] {
        branch.begin_step(Step::Main1);
        branch.prepare_triggers();
        settle(branch);
        assert_eq!(branch.object_view(saga).zone, Zone::Graveyard);
    }
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
    assert_eq!(none.state.players[player.index()].cards_drawn_this_turn, 0);
}

#[test]
fn origin_of_metalbending_and_pillar_launch_keep_their_distinct_effects() {
    let player = Player::P1;
    let mut game = scenario(player);
    let creature = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let artifact = put(&mut game, player, "Short Bow", Zone::Battlefield);
    let modal = put(&mut game, player, "Origin of Metalbending", Zone::Hand);
    let mut destroy = game.clone();
    for (branch, mode, target) in [(&mut destroy, 0, artifact), (&mut game, 1, creature)] {
        act(branch, Action::AnnounceCast { card: modal });
        act(branch, Action::ChooseMode { mode });
        act(
            branch,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        pay(branch);
        settle(branch);
    }
    assert_eq!(destroy.object_view(artifact).zone, Zone::Graveyard);
    assert_eq!(game.counter_count(creature, CounterKind::PlusOne), 1);
    assert!(game.keywords(creature).contains(Keywords::INDESTRUCTIBLE));
    game.state.objects[creature as usize].tapped = true;
    let launch = put(&mut game, player, "Pillar Launch", Zone::Hand);
    cast(&mut game, launch, Some(Target::Card(creature)));
    settle(&mut game);
    assert_eq!(stats(&game, creature), (5, 5));
    assert!(game.keywords(creature).contains(Keywords::REACH));
    assert!(!game.object_view(creature).tapped);
    let victim = put(
        &mut game,
        player.other(),
        "Shivan Dragon",
        Zone::Battlefield,
    );
    let rebuke = put(&mut game, player, "Rocky Rebuke", Zone::Hand);
    announce(
        &mut game,
        rebuke,
        &[Target::Card(creature), Target::Card(victim)],
    );
    settle(&mut game);
    assert_eq!(game.object_view(victim).zone, Zone::Graveyard);
    assert_eq!(game.state.objects[creature as usize].damage, 0);
}

#[test]
fn shared_roots_and_forestcycling_use_distinct_search_filters_and_zones() {
    let player = Player::P0;
    let mut game = scenario(player);
    let forest = put(&mut game, player, "Forest", Zone::Library);
    let dual = put(&mut game, player, "Temple Garden", Zone::Library);
    let moose = put(&mut game, player, "Saber-Tooth Moose-Lion", Zone::Hand);
    let mut cycling = game.clone();
    act(&mut cycling, activate(moose));
    pay(&mut cycling);
    reach(&mut cycling, |g| {
        g.legal_actions()
            .contains(&Action::ChooseSearch { card: dual })
    });
    act(&mut cycling, Action::ChooseSearch { card: dual });
    act(&mut cycling, Action::FinishSearch);
    settle(&mut cycling);
    assert_eq!(cycling.object_view(moose).zone, Zone::Graveyard);
    assert_eq!(cycling.object_view(dual).zone, Zone::Hand);

    let roots = put(&mut game, player, "Shared Roots", Zone::Hand);
    cast(&mut game, roots, None);
    reach(&mut game, |g| {
        g.legal_actions()
            .contains(&Action::ChooseSearch { card: forest })
    });
    reject(&mut game, Action::ChooseSearch { card: dual });
    act(&mut game, Action::ChooseSearch { card: forest });
    act(&mut game, Action::FinishSearch);
    settle(&mut game);
    assert!(game.object_view(forest).tapped);
    game.state.objects[forest as usize].tapped = false;
    let mana = game.state.players[player.index()].mana[4];
    act(&mut game, Action::TapMana { card: forest });
    assert_eq!(game.state.players[player.index()].mana[4], mana + 1);

    game.move_object(moose, Zone::Battlefield);
    assert!(game.keywords(moose).contains(Keywords::REACH));
}

#[test]
fn true_ancestry_investigates_without_a_target_and_turtle_duck_sets_only_base_power() {
    let player = Player::P1;
    let mut game = scenario(player);
    let duck = put(&mut game, player, "Turtle-Duck", Zone::Graveyard);
    let spell = put(&mut game, player, "True Ancestry", Zone::Hand);
    let mut no_target = game.clone();
    announce(&mut no_target, spell, &[]);
    settle(&mut no_target);
    assert_eq!(token_count(&no_target, "Clue"), 1);
    assert_eq!(no_target.object_view(duck).zone, Zone::Graveyard);
    cast(&mut game, spell, Some(Target::Card(duck)));
    settle(&mut game);
    assert_eq!(game.object_view(duck).zone, Zone::Hand);
    assert_eq!(token_count(&game, "Clue"), 1);
    cast(&mut game, duck, None);
    settle(&mut game);
    game.add_counter(duck, CounterKind::PlusOne, 1);
    act(&mut game, activate(duck));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(stats(&game, duck), (5, 5));
    assert!(game.keywords(duck).contains(Keywords::TRAMPLE));
}
