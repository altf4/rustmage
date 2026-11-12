use crate::support::*;
use rustmage_core::{cards::*, engine::*};

fn reach(game: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..32 {
        if ready(game) {
            return;
        }
        choose(game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("missing Earthbend choice");
}

fn target_land(game: &mut Game, land: CardId) {
    reach(game, |g| matches!(g.state.pending, Pending::Targets { .. }));
    act(
        game,
        Action::ChooseTarget {
            target: Target::Card(land),
        },
    );
}

#[test]
fn earthbending_lesson_animates_and_returns_the_exact_land_tapped() {
    let player = Player::P1;
    let mut game = scenario(player);
    let land = put(&mut game, player, "Forest", Zone::Battlefield);
    let opposing = put(&mut game, player.other(), "Island", Zone::Battlefield);
    let lesson = put(&mut game, player, "Earthbending Lesson", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: lesson });
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(opposing),
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(land),
        },
    );
    pay(&mut game);
    settle(&mut game);
    let exact = game.object_ref(land);
    assert_eq!(game.object_view(land).kinds & 3, 3);
    assert!(game.keywords(land).contains(Keywords::HASTE));
    assert_eq!(stats(&game, land), (4, 4));
    assert_eq!(game.counter_count(land, CounterKind::PlusOne), 4);

    for zone in [Zone::Graveyard, Zone::Exile] {
        let mut branch = game.clone();
        if zone == Zone::Graveyard {
            let removal = put(&mut branch, player, "Fell", Zone::Hand);
            cast(&mut branch, removal, Some(Target::Card(land)));
        } else {
            branch.move_object(land, zone);
            branch.prepare_triggers();
        }
        settle(&mut branch);
        assert_ne!(branch.object_ref(land), exact);
        assert_eq!(branch.object_view(land).zone, Zone::Battlefield);
        assert_eq!(branch.object_view(land).kinds & 3, 2);
        assert!(branch.object_view(land).tapped);
        assert_eq!(branch.counter_count(land, CounterKind::PlusOne), 0);
    }
}

#[test]
fn the_badgermoles_apply_to_current_creatures_and_the_cub_funds_a_spell() {
    let player = Player::P0;
    let mut game = scenario(player);
    let island = put(&mut game, player, "Island", Zone::Battlefield);
    let other_land = put(&mut game, player, "Island", Zone::Battlefield);
    let cub = put(&mut game, player, "Badgermole Cub", Zone::Hand);
    let badgermole = put(&mut game, player, "Badgermole", Zone::Hand);
    assert!(
        !Card::from_name("Badgermole Cub")
            .unwrap()
            .info()
            .standard_legal
    );

    cast(&mut game, cub, None);
    target_land(&mut game, island);
    settle(&mut game);
    assert_eq!(stats(&game, island), (1, 1));
    let growth = put(&mut game, player, "Giant Growth", Zone::Hand);
    game.state.players[player.index()].mana = [0; 6];
    act(&mut game, Action::AnnounceCast { card: growth });
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(cub),
        },
    );
    let tap = Action::TapMana { card: island };
    let revision = game.revision();
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == tap)
        .unwrap();
    let mut indexed = game.clone();
    act(&mut game, tap);
    indexed.apply_index(player, revision, index).unwrap();
    assert_eq!(game.snapshot(), indexed.snapshot());
    assert_eq!(game.state.players[player.index()].mana, [0, 1, 0, 0, 1, 0]);
    assert!(game.state.stack.is_empty());
    pay(&mut game);
    settle(&mut game);

    game.state.players[player.index()].mana = [8; 6];
    cast(&mut game, badgermole, None);
    target_land(&mut game, other_land);
    settle(&mut game);
    assert_eq!(stats(&game, other_land), (2, 2));
    assert!(game.keywords(island).contains(Keywords::TRAMPLE));
    assert!(game.keywords(other_land).contains(Keywords::TRAMPLE));
    assert!(!game.keywords(cub).contains(Keywords::TRAMPLE));
    game.move_object(badgermole, Zone::Exile);
    assert!(!game.keywords(island).contains(Keywords::TRAMPLE));
}

#[test]
fn the_general_retries_after_a_decline_and_accepts_counters_on_either_side() {
    let player = Player::P1;
    let mut game = scenario(player);
    let land = put(&mut game, player, "Forest", Zone::Battlefield);
    let enemy = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let general = put(&mut game, player, "Earth Kingdom General", Zone::Hand);
    cast(&mut game, general, None);
    target_land(&mut game, land);
    let optional = Action::ChooseOptional { accept: true };
    reach(&mut game, |g| g.legal_actions().contains(&optional));
    assert_eq!(game.counter_count(land, CounterKind::PlusOne), 2);
    act(&mut game, Action::ChooseOptional { accept: false });
    settle(&mut game);

    put(&mut game, player, "Ornery Tumblewagg", Zone::Battlefield);
    game.begin_step(Step::BeginCombat);
    game.prepare_triggers();
    target_land(&mut game, enemy);
    reach(&mut game, |g| g.legal_actions().contains(&optional));
    act(&mut game, optional);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, 21);
    assert_eq!(game.counter_count(enemy, CounterKind::PlusOne), 1);

    let veil = put(&mut game, player, "Snakeskin Veil", Zone::Hand);
    cast(&mut game, veil, Some(Target::Card(general)));
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, 21);
}

#[test]
fn earth_rumble_targets_its_new_land_only_on_the_separate_fight_trigger() {
    let player = Player::P0;
    let mut game = scenario(player);
    let land = put(&mut game, player, "Forest", Zone::Battlefield);
    let opponent = put(
        &mut game,
        player.other(),
        "Llanowar Elves",
        Zone::Battlefield,
    );
    let spell = put(&mut game, player, "Earth Rumble", Zone::Hand);
    cast(&mut game, spell, Some(Target::Card(land)));
    let mut failed = game.clone();
    failed.move_object(land, Zone::Hand);
    settle(&mut failed);
    assert_eq!(failed.object_view(opponent).zone, Zone::Battlefield);

    reach(&mut game, |g| {
        matches!(g.state.pending, Pending::Targets { .. })
    });
    let mut no_fighter = game.clone();
    act(&mut no_fighter, Action::FinishTargets);
    act(
        &mut no_fighter,
        Action::ChooseTarget {
            target: Target::Card(opponent),
        },
    );
    settle(&mut no_fighter);
    assert_eq!(no_fighter.object_view(opponent).zone, Zone::Battlefield);

    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(land),
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(opponent),
        },
    );
    settle(&mut game);
    assert_eq!(game.object_view(opponent).zone, Zone::Graveyard);
    assert_eq!(game.state.objects[land as usize].damage, 1);
}

#[test]
fn haru_observes_another_friendly_ally_and_the_captives_exhaust_once() {
    let player = Player::P0;
    let mut game = scenario(player);
    let land = put(&mut game, player, "Forest", Zone::Battlefield);
    let haru = put(&mut game, player, "Haru, Hidden Talent", Zone::Hand);
    cast(&mut game, haru, None);
    settle(&mut game);
    assert_eq!(game.counter_count(land, CounterKind::PlusOne), 0);

    let opposing_ally = put(&mut game, player.other(), "Wartime Protestors", Zone::Hand);
    game.move_object(opposing_ally, Zone::Battlefield);
    game.prepare_triggers();
    settle(&mut game);
    assert_eq!(game.counter_count(land, CounterKind::PlusOne), 0);
    let ally = put(&mut game, player, "Wartime Protestors", Zone::Hand);
    game.move_object(ally, Zone::Battlefield);
    game.prepare_triggers();
    target_land(&mut game, land);
    settle(&mut game);
    assert_eq!(game.counter_count(land, CounterKind::PlusOne), 1);

    let captives = put(&mut game, player, "Rebellious Captives", Zone::Battlefield);
    let action = Action::AnnounceActivate {
        card: captives,
        ability: 0,
        counter_payment: Counters::default(),
    };
    act(&mut game, action.clone());
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(land),
        },
    );
    pay(&mut game);
    let mut illegal = game.clone();
    illegal.move_object(land, Zone::Hand);
    settle(&mut illegal);
    assert_eq!(illegal.counter_count(captives, CounterKind::PlusOne), 0);

    settle(&mut game);
    assert_eq!(game.counter_count(captives, CounterKind::PlusOne), 2);
    assert_eq!(game.counter_count(land, CounterKind::PlusOne), 3);
    reject(&mut game, action);
}

#[test]
fn rockalanche_uses_current_forests_and_flashback_exiles_the_spell() {
    let player = Player::P0;
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Rockalanche", Zone::Graveyard);
    let land = put(&mut game, player, "Mountain", Zone::Battlefield);
    put(&mut game, player.other(), "Forest", Zone::Battlefield);
    act(&mut game, Action::AnnounceFlashback { card: spell });
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(land),
        },
    );
    pay(&mut game);
    let mut zero = game.clone();
    settle(&mut zero);
    assert_eq!(zero.object_view(land).kinds & 3, 2);
    assert!(zero.object_view(land).tapped);
    assert_eq!(zero.object_view(spell).zone, Zone::Exile);

    put(&mut game, player, "Forest", Zone::Battlefield);
    put(&mut game, player, "Restless Cottage", Zone::Battlefield);
    settle(&mut game);
    assert_eq!(game.object_view(spell).zone, Zone::Exile);
    assert_eq!(stats(&game, land), (1, 1));
}

#[test]
fn the_boulder_samples_power_before_reanimating_and_toph_counts_noncreature_lands() {
    let player = Player::P1;
    let mut game = scenario(player);
    let toph = put(&mut game, player, "Toph, the Blind Bandit", Zone::Hand);
    let island = put(&mut game, player, "Island", Zone::Battlefield);
    let forest = put(&mut game, player, "Forest", Zone::Battlefield);
    game.add_counter(island, CounterKind::PlusOne, 2);
    assert_eq!(stats(&game, toph), (2, 3));
    cast(&mut game, toph, None);
    target_land(&mut game, forest);
    settle(&mut game);
    assert_eq!(stats(&game, toph), (4, 3));

    let cottage = put(&mut game, player, "Restless Cottage", Zone::Battlefield);
    choose(
        &mut game,
        |action| matches!(action, Action::Activate { card, ability: 1, .. } if *card == cottage),
    );
    settle(&mut game);
    assert_eq!(stats(&game, cottage), (4, 4));
    let boulder = put(
        &mut game,
        player,
        "The Boulder, Ready to Rumble",
        Zone::Battlefield,
    );
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    act(&mut game, Action::Attack { card: boulder });
    act(&mut game, Action::FinishAttackers);
    target_land(&mut game, cottage);
    settle(&mut game);
    assert_eq!(game.counter_count(cottage, CounterKind::PlusOne), 3);
    assert_eq!(stats(&game, toph), (7, 3));
}

#[test]
fn ba_sing_se_and_earthbender_ascension_place_counters_before_the_return() {
    let player = Player::P0;
    let mut game = scenario(player);
    let basic = put(&mut game, player, "Forest", Zone::Battlefield);
    let city = put(&mut game, player, "Ba Sing Se", Zone::Hand);
    act(&mut game, Action::PlayLand { card: city });
    assert!(!game.object_view(city).tapped);
    choose(
        &mut game,
        |action| matches!(action, Action::ActivateMana { card, ability: 0, .. } if *card == city),
    );
    assert!(game.object_view(city).tapped);
    game.state.objects[city as usize].tapped = false;
    act(
        &mut game,
        Action::AnnounceActivate {
            card: city,
            ability: 1,
            counter_payment: Counters::default(),
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(city),
        },
    );
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.counter_count(city, CounterKind::PlusOne), 2);

    let ascension = put(&mut game, player, "Earthbender Ascension", Zone::Hand);
    let found = put(&mut game, player, "Forest", Zone::Library);
    cast(&mut game, ascension, None);
    target_land(&mut game, basic);
    reach(&mut game, |g| {
        g.legal_actions()
            .contains(&Action::ChooseSearch { card: found })
    });
    act(&mut game, Action::ChooseSearch { card: found });
    act(&mut game, Action::FinishSearch);
    settle(&mut game);
    assert_eq!(game.counter_count(basic, CounterKind::PlusOne), 2);
    assert!(game.object_view(found).tapped);
    assert_eq!(game.counter_count(ascension, CounterKind::Quest), 1);
    game.add_counter(ascension, CounterKind::Quest, 2);
    game.move_object(found, Zone::Hand);
    game.move_object(found, Zone::Battlefield);
    game.prepare_triggers();
    target_land(&mut game, basic);
    settle(&mut game);
    assert_eq!(game.counter_count(ascension, CounterKind::Quest), 4);
    assert_eq!(game.counter_count(basic, CounterKind::PlusOne), 3);
    assert!(game.keywords(basic).contains(Keywords::TRAMPLE));
}
