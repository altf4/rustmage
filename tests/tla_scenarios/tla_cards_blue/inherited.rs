use super::*;

#[test]
fn accumulate_wisdom_privately_selects_one_or_puts_all_three_in_hand_with_three_lessons() {
    let p = Player::P0;
    let mut game = scenario(p);
    let spell = put(&mut game, p, "Accumulate Wisdom", Zone::Hand);
    let deepest = game.state.players[p.index()].library[4];
    let top =
        ["Island", "Short Bow", "Bear Cub"].map(|name| put(&mut game, p, name, Zone::Library));
    for _ in 0..2 {
        put(&mut game, p, "It'll Quench Ya!", Zone::Graveyard);
    }
    put(&mut game, p.other(), "It'll Quench Ya!", Zone::Graveyard);
    let mut learned = game.clone();
    put(&mut learned, p, "Firebending Lesson", Zone::Graveyard);
    cast(&mut learned, spell, None);
    settle(&mut learned);
    for card in top {
        assert_eq!(learned.object_view(card).zone, Zone::Hand);
    }
    assert_eq!(learned.state.players[p.index()].cards_drawn_this_turn, 0);

    cast(&mut game, spell, None);
    pass_pair(&mut game);
    assert!(game.observe(p.other()).library_selection.is_none());
    assert_eq!(
        game.observe(p).library_selection.unwrap().remaining.len(),
        3
    );
    reject(&mut game, Action::SelectLibrary { card: deepest });
    act(&mut game, Action::SelectLibrary { card: top[1] });
    if game
        .legal_actions()
        .contains(&Action::FinishLibrarySelection)
    {
        act(&mut game, Action::FinishLibrarySelection);
    }
    for card in [top[0], top[2]] {
        act(&mut game, Action::OrderLibraryBottom { card });
    }
    assert_eq!(game.object_view(top[1]).zone, Zone::Hand);
    assert_eq!(
        &game.state.players[p.index()].library[..2],
        &[top[2], top[0]]
    );
    assert_eq!(game.state.players[p.index()].cards_drawn_this_turn, 0);
}

#[test]
fn agna_qela_checks_basic_land_entry_and_has_separate_mana_and_loot_abilities() {
    let p = Player::P1;
    let mut game = scenario(p);
    let land = put(&mut game, p, "Agna Qel'a", Zone::Hand);
    let drawn = game.top_library_card(p).unwrap();
    let mut no_basic = game.clone();
    act(&mut no_basic, Action::PlayLand { card: land });
    assert!(no_basic.object_view(land).tapped);

    put(&mut game, p, "Island", Zone::Battlefield);
    act(&mut game, Action::PlayLand { card: land });
    assert!(!game.object_view(land).tapped);
    let mut mana = game.clone();
    act(
        &mut mana,
        Action::ActivateMana {
            card: land,
            ability: 0,
            payment: [0; 6],
            counter_payment: Counters::default(),
            production: [0, 1, 0, 0, 0, 0],
        },
    );
    assert_eq!(mana.state.players[p.index()].mana[1], 9);
    game.state.players[p.index()].mana = [0, 1, 0, 0, 0, 2];
    act(&mut game, activate(land, 1));
    pay(&mut game);
    pass_pair(&mut game);
    assert_eq!(game.object_view(drawn).zone, Zone::Hand);
    act(&mut game, Action::Discard { card: drawn });
    assert_eq!(game.object_view(drawn).zone, Zone::Graveyard);
    assert!(game.object_view(land).tapped);
}

#[test]
fn first_time_flyer_checks_its_current_controllers_graveyard_and_parrot_has_prowess() {
    let p = Player::P0;
    let mut game = scenario(p);
    let flyer = put(&mut game, p, "First-Time Flyer", Zone::Battlefield);
    let parrot = put(&mut game, p, "Iguana Parrot", Zone::Battlefield);
    let spell = put(&mut game, p, "Quick Study", Zone::Hand);
    let creature = put(&mut game, p, "Bear Cub", Zone::Hand);
    put(&mut game, p.other(), "Firebending Lesson", Zone::Graveyard);
    assert_eq!(stats(&game, flyer), (1, 2));
    assert!(game.keywords(flyer).contains(Keywords::FLYING));
    let lesson = put(&mut game, p, "Accumulate Wisdom", Zone::Graveyard);
    assert_eq!(stats(&game, flyer), (2, 3));
    game.move_object(lesson, Zone::Exile);
    assert_eq!(stats(&game, flyer), (1, 2));

    assert!(
        game.keywords(parrot)
            .contains(Keywords::FLYING.union(Keywords::VIGILANCE))
    );
    cast(&mut game, creature, None);
    settle(&mut game);
    assert_eq!(stats(&game, parrot), (2, 2));
    cast(&mut game, spell, None);
    settle(&mut game);
    assert_eq!(stats(&game, parrot), (3, 3));
    game.finish_cleanup();
    assert_eq!(stats(&game, parrot), (2, 2));
}

#[test]
fn fortune_teller_creates_a_clue_and_its_token_cost_draws_one() {
    let p = Player::P1;
    let mut game = scenario(p);
    let teller = put(&mut game, p, "Forecasting Fortune Teller", Zone::Hand);
    cast(&mut game, teller, None);
    settle(&mut game);
    assert_eq!(stats(&game, teller), (1, 3));
    let clue = token(&game, Card(5296));
    game.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
    reject(&mut game, activate(clue, 0));
    game.state.players[p.index()].mana[5] = 2;
    act(&mut game, activate(clue, 0));
    pay(&mut game);
    settle(&mut game);
    assert_ne!(game.object_view(clue).zone, Zone::Battlefield);
    assert_eq!(game.state.players[p.index()].cards_drawn_this_turn, 1);
}

#[test]
fn itll_quench_ya_counters_the_exact_spell_unless_its_caster_pays_two() {
    let p = Player::P0;
    let mut game = scenario(p);
    let divination = put(&mut game, p, "Divination", Zone::Hand);
    let answer = put(&mut game, p.other(), "It'll Quench Ya!", Zone::Hand);
    cast(&mut game, divination, None);
    act(&mut game, Action::Pass);
    cast(&mut game, answer, Some(Target::Spell(divination)));
    pass_pair(&mut game);
    assert_eq!(game.actor(), Some(p));
    reject(
        &mut game,
        Action::PayResolutionMana {
            payment: [0, 0, 0, 0, 0, 1],
        },
    );
    let mut decline = game.clone();
    act(&mut decline, Action::DeclineResolutionMana);
    settle(&mut decline);
    assert_eq!(decline.state.players[p.index()].cards_drawn_this_turn, 0);
    act(
        &mut game,
        Action::PayResolutionMana {
            payment: [0, 0, 0, 0, 0, 2],
        },
    );
    settle(&mut game);
    assert_eq!(game.state.players[p.index()].cards_drawn_this_turn, 2);
}

#[test]
fn second_draws_trigger_knowledge_seeker_otter_penguin_and_tiger_seal_once() {
    let p = Player::P1;
    let mut game = scenario(p);
    let seeker = put(&mut game, p, "Knowledge Seeker", Zone::Battlefield);
    let otter = put(&mut game, p, "Otter-Penguin", Zone::Battlefield);
    let seal = put(&mut game, p, "Tiger-Seal", Zone::Battlefield);
    for card in [seeker, seal] {
        assert!(game.keywords(card).contains(Keywords::VIGILANCE));
    }
    game.state.active = p.other();
    game.begin_step(Step::Upkeep);
    game.give_priority();
    game.prepare_triggers();
    settle(&mut game);
    assert!(!game.object_view(seal).tapped);
    game.state.active = p;
    game.begin_step(Step::Upkeep);
    game.give_priority();
    game.prepare_triggers();
    settle(&mut game);
    assert!(game.object_view(seal).tapped);
    let mut stunned = game.clone();
    stunned.add_counter(seal, CounterKind::Stun, 1);
    for branch in [&mut game, &mut stunned] {
        branch.draw_many(p, 1);
        branch.prepare_triggers();
        assert_eq!(branch.counter_count(seeker, CounterKind::PlusOne), 0);
        assert_eq!(stats(branch, otter), (2, 1));
        branch.draw_many(p, 2);
        branch.prepare_triggers();
        finish_ordered_triggers(branch);
        assert_eq!(branch.counter_count(seeker, CounterKind::PlusOne), 1);
        assert_eq!(stats(branch, otter), (3, 3));
        assert!(branch.keywords(otter).contains(Keywords::CANT_BE_BLOCKED));
    }
    assert!(!game.object_view(seal).tapped);
    assert!(stunned.object_view(seal).tapped);
    assert_eq!(stunned.counter_count(seal, CounterKind::Stun), 0);
    game.move_object(seeker, Zone::Graveyard);
    game.prepare_triggers();
    settle(&mut game);
    assert_eq!(game.object_view(token(&game, Card(5296))).controller, p);
    game.finish_cleanup();
    assert_eq!(stats(&game, otter), (2, 1));
}

#[test]
fn octopus_form_untaps_its_target_unless_rowdy_snowballers_stunned_it() {
    let p = Player::P0;
    let mut game = scenario(p);
    let subject = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    let snow = put(&mut game, p.other(), "Rowdy Snowballers", Zone::Hand);
    let spell = put(&mut game, p, "Octopus Form", Zone::Hand);
    let their_other = put(&mut game, p.other(), "Hill Giant", Zone::Battlefield);
    game.state.active = p.other();
    game.state.priority = p.other();
    cast(&mut game, snow, None);
    reach(&mut game, |g| {
        matches!(g.state.pending, Pending::Targets { .. })
    });
    reject(&mut game, target(their_other));
    act(&mut game, target(subject));
    settle(&mut game);
    assert!(game.object_view(subject).tapped);
    assert_eq!(game.counter_count(subject, CounterKind::Stun), 1);

    act(&mut game, Action::Pass);
    act(&mut game, Action::AnnounceCast { card: spell });
    reject(&mut game, target(their_other));
    act(&mut game, target(subject));
    pay(&mut game);
    settle(&mut game);
    assert!(game.object_view(subject).tapped);
    assert_eq!(game.counter_count(subject, CounterKind::Stun), 0);
    assert_eq!(stats(&game, subject), (3, 3));
    assert!(game.keywords(subject).contains(Keywords::HEXPROOF));
    game.finish_cleanup();
    assert_eq!(stats(&game, subject), (2, 2));
    assert!(!game.keywords(subject).contains(Keywords::HEXPROOF));
}

#[test]
fn wan_shi_tong_flashes_with_x_then_observes_opponents_searching_their_own_library() {
    let p = Player::P1;
    let mut game = scenario(p.other());
    let owl = put(&mut game, p, "Wan Shi Tong, Librarian", Zone::Hand);
    let search = put(&mut game, p.other(), "Evolving Wilds", Zone::Battlefield);
    act(&mut game, Action::Pass);
    act(&mut game, Action::AnnounceCast { card: owl });
    act(&mut game, Action::ChooseX { value: 3 });
    pay(&mut game);
    settle(&mut game);
    assert!(
        game.keywords(owl).contains(
            Keywords::FLASH
                .union(Keywords::FLYING)
                .union(Keywords::VIGILANCE)
        )
    );
    assert_eq!(game.counter_count(owl, CounterKind::PlusOne), 3);
    assert_eq!(game.state.players[p.index()].cards_drawn_this_turn, 1);

    if game.actor() != Some(p.other()) {
        act(&mut game, Action::Pass);
    }
    act(&mut game, activate(search, 0));
    pay(&mut game);
    pass_pair(&mut game);
    act(&mut game, Action::FinishSearch);
    settle(&mut game);
    assert_eq!(game.counter_count(owl, CounterKind::PlusOne), 4);
    assert_eq!(game.state.players[p.index()].cards_drawn_this_turn, 2);
}
