use super::*;

#[test]
fn each_avatar_dual_enters_tapped_produces_both_colors_and_can_be_sacrificed_to_draw() {
    let player = Player::P0;
    for (name, colors) in [
        ("Airship Engine Room", [1, 3]),
        ("Boiling Rock Prison", [2, 3]),
        ("Foggy Bottom Swamp", [2, 4]),
        ("Kyoshi Village", [0, 4]),
        ("Meditation Pools", [1, 4]),
        ("Misty Palms Oasis", [0, 2]),
        ("North Pole Gates", [0, 1]),
        ("Omashu City", [3, 4]),
        ("Serpent's Pass", [1, 2]),
        ("Sun-Blessed Peak", [0, 3]),
    ] {
        let mut game = scenario(player);
        let land = put(&mut game, player, name, Zone::Hand);
        act(&mut game, Action::PlayLand { card: land });
        assert!(game.object_view(land).tapped, "{name}");
        reject(&mut game, activation(land, 1));
        game.state.objects[land as usize].tapped = false;
        for color in 0..6 {
            let mut production = [0; 6];
            production[color] = 1;
            let action = Action::ActivateMana {
                card: land,
                ability: 0,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production,
            };
            let mut branch = game.clone();
            if colors.contains(&color) {
                branch.state.players[player.index()].mana = [0; 6];
                act(&mut branch, action);
                assert_eq!(branch.state.players[player.index()].mana, production);
                assert!(branch.object_view(land).tapped);
            } else {
                reject(&mut branch, action);
            }
        }
        game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 4];
        act(&mut game, activation(land, 1));
        pay(&mut game);
        assert_eq!(game.object_view(land).zone, Zone::Graveyard);
        assert_eq!(game.state.players[player.index()].mana, [0; 6]);
        drain(&mut game);
        assert_eq!(draws(&game, player), 1, "{name}");
    }
}

#[test]
fn swift_savior_airbends_a_spell_then_la_boosts_only_tapped_creatures() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(opponent);
    let swift = put(&mut game, player, "Aang, Swift Savior", Zone::Hand);
    let opposing_spell = put(&mut game, opponent, "Opt", Zone::Hand);
    let tapped = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let untapped = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let hostile = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    cast(&mut game, opposing_spell, None);
    act(&mut game, Action::Pass);
    cast(&mut game, swift, None);
    pass_pair(&mut game);
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Spell(opposing_spell),
        },
    );
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    drain(&mut game);
    assert_eq!(game.object_view(opposing_spell).zone, Zone::Exile);
    assert!(game.keywords(swift).contains(Keywords::FLYING));
    let announced = Action::AnnounceAirbend {
        card: opposing_spell,
        face: CastFace::Normal,
        flash_surcharge: false,
    };
    assert!(game.legal_actions().contains(&announced));

    game.state.active = player;
    game.state.priority = player;
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 8];
    act(&mut game, activation(swift, 0));
    pay(&mut game);
    drain(&mut game);
    assert_eq!(game.object_view(swift).name, "Aang and La, Ocean's Fury");
    assert!(
        game.keywords(swift)
            .contains(Keywords::REACH.union(Keywords::TRAMPLE))
    );
    game.state.objects[swift as usize].summoning_sick = false;
    game.state.objects[tapped as usize].tapped = true;
    game.state.objects[hostile as usize].tapped = true;
    attack(&mut game, &[swift]);
    drain(&mut game);
    assert_eq!(stats(&game, swift), (6, 6));
    assert_eq!(stats(&game, tapped), (3, 3));
    assert_eq!(stats(&game, untapped), (2, 2));
    assert_eq!(stats(&game, hostile), (2, 2));
}

#[test]
fn abandon_attachments_draws_only_after_the_optional_discard() {
    let player = Player::P1;
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Abandon Attachments", Zone::Hand);
    let discard = put(&mut game, player, "Bear Cub", Zone::Hand);
    cast(&mut game, spell, None);
    pass_pair(&mut game);
    let mut decline = game.clone();
    act(&mut decline, Action::DeclineEffectPayment);
    drain(&mut decline);
    assert_eq!(draws(&decline, player), 0);
    assert_eq!(decline.object_view(discard).zone, Zone::Hand);
    assert!(game.observe(player.other()).effect_choice.is_none());
    act(&mut game, Action::Discard { card: discard });
    drain(&mut game);
    assert_eq!(draws(&game, player), 2);
    assert_eq!(game.object_view(discard).zone, Zone::Graveyard);
}

#[test]
fn air_nomad_legacy_and_messenger_hawk_create_clues_and_track_live_conditions() {
    let player = Player::P0;
    let mut game = scenario(player);
    let legacy = put(&mut game, player, "Air Nomad Legacy", Zone::Hand);
    let hawk = put(&mut game, player, "Messenger Hawk", Zone::Hand);
    let ground = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let hostile = put(
        &mut game,
        player.other(),
        "Messenger Hawk",
        Zone::Battlefield,
    );
    cast(&mut game, legacy, None);
    drain(&mut game);
    cast(&mut game, hawk, None);
    drain(&mut game);
    assert_eq!(tokens(&game, player, 5296).len(), 2);
    assert!(game.keywords(hawk).contains(Keywords::FLYING));
    assert_eq!(stats(&game, hawk), (2, 3));
    assert_eq!(stats(&game, hostile), (1, 2));
    assert_eq!(stats(&game, ground), (2, 2));
    game.state.objects[ground as usize].gained_keywords = Keywords::FLYING;
    assert_eq!(stats(&game, ground), (3, 3));
    game.draw_many(player, 1);
    assert_eq!(stats(&game, hawk), (2, 3));
    game.draw_many(player, 2);
    assert_eq!(stats(&game, hawk), (4, 3));
    game.finish_cleanup();
    assert_eq!(stats(&game, hawk), (2, 3));
}

#[test]
fn cat_owl_untaps_either_side_and_soldier_counters_up_to_two_friendly_creatures() {
    let player = Player::P0;
    let mut game = scenario(player);
    let owl = put(&mut game, player, "Cat-Owl", Zone::Battlefield);
    let artifact = put(&mut game, player.other(), "Short Bow", Zone::Battlefield);
    let enemy = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let land = put(&mut game, player, "Forest", Zone::Battlefield);
    let soldier = put(&mut game, player, "Earth Kingdom Soldier", Zone::Hand);
    assert!(game.keywords(owl).contains(Keywords::FLYING));
    game.state.objects[artifact as usize].tapped = true;
    attack(&mut game, &[owl]);
    reject(&mut game, target(land));
    act(&mut game, target(artifact));
    drain(&mut game);
    assert!(!game.object_view(artifact).tapped);
    game.state.step = Step::Main2;
    cast(&mut game, soldier, None);
    pass_pair(&mut game);
    reject(&mut game, target(enemy));
    act(&mut game, target(soldier));
    act(&mut game, target(owl));
    drain(&mut game);
    assert_eq!(game.counter_count(soldier, CounterKind::PlusOne), 1);
    assert_eq!(game.counter_count(owl, CounterKind::PlusOne), 1);
    assert!(game.keywords(soldier).contains(Keywords::VIGILANCE));
}

#[test]
fn jet_counts_controlled_creatures_at_entry_and_death_can_choose_two_controllers() {
    let player = Player::P0;
    let mut game = scenario(player);
    let jet = put(&mut game, player, "Jet, Freedom Fighter", Zone::Hand);
    let friend = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let opponent = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    cast(&mut game, jet, None);
    pass_pair(&mut game);
    reject(&mut game, target(friend));
    act(&mut game, target(opponent));
    drain(&mut game);
    assert_eq!(game.object_view(opponent).damage, 2);
    game.move_object(jet, Zone::Graveyard);
    game.prepare_triggers();
    act(&mut game, target(friend));
    act(&mut game, target(opponent));
    drain(&mut game);
    assert_eq!(game.counter_count(friend, CounterKind::PlusOne), 1);
    assert_eq!(game.counter_count(opponent, CounterKind::PlusOne), 1);
}

#[test]
fn long_feng_counts_a_creature_land_once_and_poxbearers_require_death() {
    let player = Player::P1;
    let mut game = scenario(player);
    let feng = put(
        &mut game,
        player,
        "Long Feng, Grand Secretariat",
        Zone::Battlefield,
    );
    let land = put(&mut game, player, "Forest", Zone::Battlefield);
    let recipient = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let pox = put(
        &mut game,
        player,
        "Pretending Poxbearers",
        Zone::Battlefield,
    );
    let mut exile = game.clone();
    exile.move_object(pox, Zone::Exile);
    drain(&mut exile);
    assert!(tokens(&exile, player, 5272).is_empty());

    animate(&mut game, land, Some((2, 2)));
    game.move_object(land, Zone::Graveyard);
    game.prepare_triggers();
    act(&mut game, target(recipient));
    drain(&mut game);
    assert_eq!(game.counter_count(recipient, CounterKind::PlusOne), 1);
    game.move_object(pox, Zone::Graveyard);
    game.prepare_triggers();
    for _ in 0..12 {
        if game.state.stack.is_empty() && game.state.pending == Pending::Priority {
            break;
        }
        choose(&mut game, |action| {
            matches!(action, Action::PutTrigger { .. } | Action::Pass) || *action == target(feng)
        });
    }
    assert_eq!(tokens(&game, player, 5272).len(), 1);
    assert_eq!(game.counter_count(feng, CounterKind::PlusOne), 1);
}

#[test]
fn professor_zei_pays_discard_then_later_can_sacrifice_to_recover_the_card() {
    let player = Player::P0;
    let mut game = scenario(player);
    let professor = put(
        &mut game,
        player,
        "Professor Zei, Anthropologist",
        Zone::Battlefield,
    );
    let discard = put(&mut game, player, "Quick Study", Zone::Hand);
    let opposing = put(&mut game, player.other(), "Opt", Zone::Graveyard);
    act(&mut game, activation(professor, 0));
    assert!(game.observe(player.other()).cast_costs.is_none());
    choose(&mut game, |action| match action {
        Action::Discard { card } => *card == discard,
        Action::ChooseAdditionalCost { object } => object.card == discard,
        _ => false,
    });
    if game
        .legal_actions()
        .iter()
        .any(|action| matches!(action, Action::PayCosts { .. }))
    {
        pay(&mut game);
    }
    drain(&mut game);
    assert!(game.object_view(professor).tapped);
    assert_eq!(draws(&game, player), 1);
    assert_eq!(game.object_view(discard).zone, Zone::Graveyard);
    game.state.objects[professor as usize].tapped = false;
    let mut wrong_turn = game.clone();
    wrong_turn.state.active = player.other();
    reject(&mut wrong_turn, activation(professor, 1));
    act(&mut game, activation(professor, 1));
    reject(&mut game, target(opposing));
    act(&mut game, target(discard));
    pay(&mut game);
    drain(&mut game);
    assert_eq!(game.object_view(professor).zone, Zone::Graveyard);
    assert_eq!(game.object_view(discard).zone, Zone::Hand);
}

#[test]
fn sun_warriors_and_the_warden_create_spendable_firebending_mana() {
    let player = Player::P0;
    let mut game = scenario(player);
    let sun = put(&mut game, player, "Sun Warriors", Zone::Battlefield);
    let warden = put(&mut game, player, "Vindictive Warden", Zone::Battlefield);
    let departed = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 2];
    assert!(game.keywords(warden).contains(Keywords::MENACE));
    attack(&mut game, &[sun, warden]);
    game.move_object(departed, Zone::Exile);
    drain(&mut game);
    assert_eq!(game.state.players[player.index()].combat_red_mana, 3);

    let mut offense = game.clone();
    act(&mut offense, activation(warden, 0));
    pay(&mut offense);
    drain(&mut offense);
    assert_eq!(life(&offense, player.other()), 19);
    act(&mut game, activation(sun, 0));
    pay(&mut game);
    drain(&mut game);
    assert_eq!(tokens(&game, player, 5272).len(), 1);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
}

#[test]
fn lion_turtle_gains_life_and_mana_and_needs_three_lessons_to_fight() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let turtle = put(&mut game, player, "The Lion-Turtle", Zone::Hand);
    let enemy = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
    let lesson = put(&mut game, player, "Firebending Lesson", Zone::Graveyard);
    put(&mut game, player, "Firebending Lesson", Zone::Graveyard);
    put(&mut game, player, "Firebending Lesson", Zone::Graveyard);
    cast(&mut game, turtle, None);
    drain(&mut game);
    assert_eq!(life(&game, player), 23);
    assert!(
        game.keywords(turtle)
            .contains(Keywords::REACH.union(Keywords::VIGILANCE))
    );
    game.state.objects[turtle as usize].summoning_sick = false;
    let mut without = game.clone();
    without.move_object(lesson, Zone::Exile);
    without.state.step = Step::Attackers;
    without.state.pending = Pending::Attackers;
    reject(&mut without, Action::Attack { card: turtle });
    let mut attack_branch = game.clone();
    attack(&mut attack_branch, &[turtle]);
    assert!(!attack_branch.object_view(turtle).tapped);

    game.state.active = opponent;
    game.state.priority = opponent;
    attack(&mut game, &[enemy]);
    pass_pair(&mut game);
    let block = Action::Block {
        blocker: turtle,
        attacker: enemy,
    };
    let mut without = game.clone();
    without.move_object(lesson, Zone::Exile);
    reject(&mut without, block.clone());
    act(&mut game, block);
    act(&mut game, Action::FinishBlockers);
    let mut mana_branch = without;
    mana_branch.state.pending = Pending::Priority;
    mana_branch.state.priority = player;
    let before = mana_branch.state.players[player.index()].mana;
    act(
        &mut mana_branch,
        Action::ActivateMana {
            card: turtle,
            ability: 0,
            payment: [0; 6],
            counter_payment: Counters::default(),
            production: [0, 0, 0, 0, 1, 0],
        },
    );
    assert_eq!(
        mana_branch.state.players[player.index()].mana[4],
        before[4] + 1
    );
}

#[test]
fn tolls_and_zhao_observe_distinct_actual_sacrifices_and_wandering_boosts_current_creatures() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let tolls = put(&mut game, player, "Tolls of War", Zone::Hand);
    let zhao = put(
        &mut game,
        player,
        "Zhao, Ruthless Admiral",
        Zone::Battlefield,
    );
    let wandering = put(&mut game, player, "Wandering Musicians", Zone::Battlefield);
    let first = put(&mut game, player, "Forest", Zone::Battlefield);
    let second = put(&mut game, player, "Island", Zone::Battlefield);
    let third = put(&mut game, player, "Swamp", Zone::Battlefield);
    let hostile = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    cast(&mut game, tolls, None);
    drain(&mut game);
    assert_eq!(tokens(&game, player, 5296).len(), 1);
    game.sacrifice_batch(&[(player, first), (player, second)]);
    drain(&mut game);
    assert_eq!(tokens(&game, player, 5272).len(), 1);
    assert_eq!(stats(&game, zhao), (5, 4));
    assert_eq!(stats(&game, hostile), (2, 2));
    game.state.active = opponent;
    game.sacrifice_batch(&[(player, third)]);
    drain(&mut game);
    assert_eq!(tokens(&game, player, 5272).len(), 1);
    assert_eq!(stats(&game, zhao), (6, 4));

    game.state.active = player;
    game.state.priority = player;
    attack(&mut game, &[zhao, wandering]);
    drain(&mut game);
    assert_eq!(game.state.players[player.index()].combat_red_mana, 2);
    assert_eq!(stats(&game, zhao), (7, 4));
    let late = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    assert_eq!(stats(&game, late), (2, 2));
    game.finish_cleanup();
    assert_eq!(stats(&game, zhao), (3, 4));
}

#[test]
fn uncle_iroh_reduces_only_lesson_costs_and_reinforcements_pump_other_allies() {
    let player = Player::P0;
    let mut game = scenario(player);
    let iroh = put(&mut game, player, "Uncle Iroh", Zone::Battlefield);
    let reinforces = put(
        &mut game,
        player,
        "White Lotus Reinforcements",
        Zone::Battlefield,
    );
    let hostile = put(
        &mut game,
        player.other(),
        "White Lotus Reinforcements",
        Zone::Battlefield,
    );
    let lesson = put(&mut game, player, "Accumulate Wisdom", Zone::Hand);
    let nonlesson = put(&mut game, player, "Quick Study", Zone::Hand);
    assert!(game.keywords(reinforces).contains(Keywords::VIGILANCE));
    assert_eq!(stats(&game, iroh), (5, 3));
    assert_eq!(stats(&game, reinforces), (2, 3));
    assert_eq!(stats(&game, hostile), (2, 3));
    attack(&mut game, &[iroh]);
    drain(&mut game);
    assert_eq!(game.state.players[player.index()].combat_red_mana, 1);

    game.state.step = Step::Main2;
    game.state.players[player.index()].combat_red_mana = 0;
    game.state.players[player.index()].mana = [0, 1, 0, 0, 0, 0];
    reject(&mut game, Action::AnnounceCast { card: nonlesson });
    cast(&mut game, lesson, None);
    assert_eq!(
        game.state.stack.last().unwrap().cast.unwrap().payment,
        [0, 1, 0, 0, 0, 0]
    );
    game.move_object(reinforces, Zone::Exile);
    assert_eq!(stats(&game, iroh), (4, 2));
}
