use super::*;

#[test]
fn rioter_taps_an_ally_to_exile_and_firebends_to_cast_only_an_owned_linked_ally() {
    let p = Player::P0;
    let mut game = scenario(p);
    let rioter = put(&mut game, p, "Boiling Rock Rioter", Zone::Battlefield);
    let ally = put(&mut game, p, "First-Time Flyer", Zone::Battlefield);
    let non_ally = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    let valid = put(&mut game, p, "Forecasting Fortune Teller", Zone::Graveyard);
    let foreign = put(
        &mut game,
        p.other(),
        "Forecasting Fortune Teller",
        Zone::Graveyard,
    );
    game.state.objects[ally as usize].summoning_sick = true;
    for (card, contributor) in [(valid, ally), (foreign, rioter)] {
        act(&mut game, activate(rioter, 0));
        act(&mut game, target(card));
        let object = game.object_ref(non_ally);
        reject(&mut game, Action::ChooseAdditionalCost { object });
        let object = game.object_ref(contributor);
        act(&mut game, Action::ChooseAdditionalCost { object });
        if game
            .legal_actions()
            .contains(&Action::FinishAdditionalCostGroup)
        {
            act(&mut game, Action::FinishAdditionalCostGroup);
        }
        pay(&mut game);
        settle(&mut game);
        assert!(game.object_view(contributor).tapped);
        assert_eq!(game.object_view(card).zone, Zone::Exile);
    }
    game.state.objects[rioter as usize].tapped = false;
    game.state.players[p.index()].mana = [0, 1, 0, 0, 0, 0];
    declare_attackers(&mut game, &[rioter]);
    let cast_trigger = game
        .state
        .ordering_triggers
        .iter()
        .find(|trigger| trigger.kind == (StackKind::TriggeredAbility { ability: 1 }))
        .unwrap()
        .id;
    act(&mut game, Action::PutTrigger { id: cast_trigger });
    reach(&mut game, |g| {
        g.state.pending == Pending::ResolutionCasts { player: p }
    });
    assert_eq!(game.state.players[p.index()].combat_red_mana, 1);
    let foreign_cast = Action::CastExiledDuringResolution {
        object: game.object_ref(foreign),
        face: CastFace::Normal,
    };
    reject(&mut game, foreign_cast);
    let cast = Action::CastExiledDuringResolution {
        object: game.object_ref(valid),
        face: CastFace::Normal,
    };
    act(&mut game, cast);
    if game
        .legal_actions()
        .iter()
        .any(|a| matches!(a, Action::PayCosts { .. }))
    {
        pay(&mut game);
    }
    if game
        .legal_actions()
        .contains(&Action::FinishResolutionCasts)
    {
        act(&mut game, Action::FinishResolutionCasts);
    }
    settle(&mut game);
    assert_eq!(game.object_view(valid).zone, Zone::Battlefield);
    assert_eq!(game.state.players[p.index()].combat_red_mana, 0);
}

#[test]
fn joo_dee_surviels_before_copying_and_requires_a_sacrifice_at_sorcery_speed() {
    let p = Player::P1;
    let mut game = scenario(p);
    let joo = put(&mut game, p, "Joo Dee, One of Many", Zone::Battlefield);
    let identity = game.object_view(joo).card;
    let artifact = put(&mut game, p, "Short Bow", Zone::Battlefield);
    let next = game.top_library_card(p).unwrap();
    let action = activate(joo, 0);
    let mut offturn = game.clone();
    offturn.state.active = p.other();
    reject(&mut offturn, action.clone());
    act(&mut game, action);
    pay(&mut game);
    assert!(game.object_view(joo).tapped);
    pass_pair(&mut game);
    assert_eq!(token_count(&game, identity), 0);
    assert!(game.observe(p.other()).library_choice.is_none());
    act(
        &mut game,
        Action::ArrangeLibrary {
            card: next,
            destination: LibraryDestination::Graveyard,
        },
    );
    assert_eq!(game.object_view(next).zone, Zone::Graveyard);
    assert_eq!(token_count(&game, identity), 1);
    reject(&mut game, Action::FinishEffectChoice);
    sacrifice(&mut game, artifact);
    assert_eq!(game.object_view(artifact).zone, Zone::Graveyard);
}

#[test]
fn koh_switches_between_entry_and_death_linked_donors_at_one_life_each() {
    let p = Player::P0;
    let mut game = scenario(p);
    let koh = put(&mut game, p, "Koh, the Face Stealer", Zone::Hand);
    let katara = put(
        &mut game,
        p.other(),
        "Katara, Bending Prodigy",
        Zone::Battlefield,
    );
    let june = put(
        &mut game,
        p.other(),
        "June, Bounty Hunter",
        Zone::Battlefield,
    );
    let own = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    let murder = put(&mut game, p, "Murder", Zone::Hand);
    cast(&mut game, koh, None);
    reach(&mut game, |g| {
        matches!(g.state.pending, Pending::Targets { .. })
    });
    reject(&mut game, target(koh));
    act(&mut game, target(katara));
    finish_targets(&mut game);
    settle(&mut game);
    let choose_donor = |g: &mut Game, donor| {
        act(g, activate(koh, 0));
        pay(g);
        reach(g, |g| {
            matches!(g.state.pending, Pending::LinkedExile { .. })
        });
        let object = g.object_ref(donor);
        act(g, Action::ChooseLinkedExile { object });
        settle(g);
    };
    let acquired = |g: &Game, donor| Action::AnnounceBorrowedAbility {
        card: koh,
        exiled: g.object_ref(donor),
        ability: 0,
        counter_payment: Counters::default(),
    };
    choose_donor(&mut game, katara);
    assert_eq!(game.state.players[p.index()].life, 19);
    let from_katara = acquired(&game, katara);
    act(&mut game, from_katara.clone());
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.state.players[p.index()].cards_drawn_this_turn, 1);
    assert_eq!(stats(&game, koh), (6, 6));

    cast(&mut game, murder, Some(Target::Card(june)));
    reach(&mut game, |g| {
        g.legal_actions()
            .contains(&Action::ChooseOptional { accept: true })
    });
    let mut decline = game.clone();
    act(&mut decline, Action::ChooseOptional { accept: false });
    assert_eq!(decline.object_view(june).zone, Zone::Graveyard);
    act(&mut game, Action::ChooseOptional { accept: true });
    assert_eq!(game.object_view(june).zone, Zone::Exile);
    choose_donor(&mut game, june);
    assert_eq!(game.state.players[p.index()].life, 18);
    reject(&mut game, from_katara);
    let from_june = acquired(&game, june);
    act(&mut game, from_june);
    let object = game.object_ref(own);
    act(&mut game, Action::ChooseAdditionalCost { object });
    pay(&mut game);
    reach(&mut game, |g| {
        g.legal_actions()
            .contains(&Action::ChooseOptional { accept: false })
    });
    act(&mut game, Action::ChooseOptional { accept: false });
    settle(&mut game);
    assert_eq!(game.object_view(own).zone, Zone::Graveyard);
    assert_eq!(token_count(&game, Card(5296)), 1);
}

#[test]
fn phoenix_fleet_airship_copies_after_sacrifice_crews_for_one_and_counts_eight_live_names() {
    let p = Player::P0;
    let mut game = scenario(p);
    let airship = put(&mut game, p, "Phoenix Fleet Airship", Zone::Battlefield);
    let identity = game.object_view(airship).card;
    let crewer = put(&mut game, p, "Goblin Piker", Zone::Battlefield);
    assert!(game.keywords(airship).contains(Keywords::FLYING));
    assert_eq!(game.current_kinds(airship), 16);
    let mut count = game.clone();
    for _ in 0..6 {
        put(&mut count, p, "Phoenix Fleet Airship", Zone::Battlefield);
    }
    let foreign = put(
        &mut count,
        p.other(),
        "Phoenix Fleet Airship",
        Zone::Battlefield,
    );
    assert_eq!(count.current_kinds(airship), 16);
    count.change_controller(count.object_ref(foreign), p);
    assert_eq!(count.current_kinds(airship), 17);
    assert_eq!(stats(&count, airship), (4, 4));
    count.change_controller(count.object_ref(foreign), p.other());
    assert_eq!(count.current_kinds(airship), 16);

    let mut no_sacrifice = game.clone();
    start_end_step(&mut no_sacrifice, p);
    settle(&mut no_sacrifice);
    assert_eq!(token_count(&no_sacrifice, identity), 0);

    crew(&mut game, airship, 0, crewer);
    assert_eq!(game.current_kinds(airship), 17);
    let clue = spawn(&mut game, p, Card(5296), Zone::Battlefield);
    act(&mut game, activate(clue, 0));
    pay(&mut game);
    settle(&mut game);
    start_end_step(&mut game, p);
    settle(&mut game);
    assert_eq!(token_count(&game, identity), 1);
}

#[test]
fn fire_nation_drill_may_tap_then_destroy_and_strips_both_keywords_before_crewing() {
    let p = Player::P1;
    let mut game = scenario(p);
    let drill = put(&mut game, p, "The Fire Nation Drill", Zone::Hand);
    let victim = put(&mut game, p.other(), "Hill Giant", Zone::Battlefield);
    let too_big = put(&mut game, p.other(), "Canyon Crawler", Zone::Battlefield);
    let contributor = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    let protection = Keywords::HEXPROOF
        .union(Keywords::hexproof_from_color(ChoiceColor::Black))
        .union(Keywords::INDESTRUCTIBLE);
    let grant_source = game.object_ref(contributor);
    game.grant_keyword_effects(&[too_big, contributor], grant_source, protection);
    assert!(game.keywords(contributor).contains(protection));
    cast(&mut game, drill, None);
    reach(&mut game, |g| {
        g.legal_actions()
            .contains(&Action::ChooseOptional { accept: true })
    });
    let mut no_tap = game.clone();
    act(&mut no_tap, Action::ChooseOptional { accept: false });
    assert!(!no_tap.object_view(drill).tapped);
    assert_eq!(no_tap.object_view(victim).zone, Zone::Battlefield);
    act(&mut game, Action::ChooseOptional { accept: true });
    assert!(game.object_view(drill).tapped);
    reject(&mut game, target(too_big));
    act(&mut game, target(victim));
    settle(&mut game);
    assert_eq!(game.object_view(victim).zone, Zone::Graveyard);

    act(&mut game, activate(drill, 0));
    pay(&mut game);
    settle(&mut game);
    for keyword in [
        Keywords::HEXPROOF,
        Keywords::hexproof_from_color(ChoiceColor::Black),
        Keywords::INDESTRUCTIBLE,
    ] {
        assert!(!game.keywords(too_big).contains(keyword));
        assert!(game.keywords(contributor).contains(keyword));
    }
    crew(&mut game, drill, 1, contributor);
    assert_eq!(stats(&game, drill), (6, 3));
    assert!(game.keywords(drill).contains(Keywords::TRAMPLE));
    game.finish_cleanup();
    assert!(!game.keywords(too_big).contains(Keywords::HEXPROOF));
}

#[test]
fn sozin_destroys_creatures_searches_three_zones_then_firebends_to_reanimate_within_x() {
    let p = Player::P0;
    let mut game = scenario(p);
    let saga = put(
        &mut game,
        p,
        "The Rise of Sozin // Fire Lord Sozin",
        Zone::Hand,
    );
    let own = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    let enemy = put(&mut game, p.other(), "Hill Giant", Zone::Battlefield);
    let other = put(&mut game, p.other(), "Goblin Piker", Zone::Graveyard);
    let bow = put(&mut game, p.other(), "Short Bow", Zone::Battlefield);
    let selected: Vec<_> = [Zone::Graveyard, Zone::Hand, Zone::Library]
        .into_iter()
        .map(|zone| put(&mut game, p.other(), "Forest", zone))
        .collect();
    cast(&mut game, saga, None);
    settle(&mut game);
    assert_eq!(game.object_view(own).zone, Zone::Graveyard);
    assert_eq!(game.object_view(enemy).zone, Zone::Graveyard);
    assert_eq!(game.object_view(bow).zone, Zone::Battlefield);

    game.begin_step(Step::Main1);
    game.give_priority();
    game.prepare_triggers();
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(p),
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(p.other()),
        },
    );
    pass_pair(&mut game);
    act(
        &mut game,
        Action::ChooseEntryValue {
            value: EntryChoiceValue::CardName(rustmage::CardName::from_name("Forest").unwrap()),
        },
    );
    assert!(game.observe(p.other()).effect_choice.is_none());
    for &id in &selected {
        let object = game.object_ref(id);
        act(&mut game, Action::SelectResolutionExile { object });
    }
    act(&mut game, Action::FinishSearch);
    for id in selected {
        assert_eq!(game.object_view(id).zone, Zone::Exile);
    }

    let prior = game.object_ref(saga);
    game.begin_step(Step::Main1);
    game.give_priority();
    game.prepare_triggers();
    settle(&mut game);
    assert_eq!(game.object_view(saga).card, Card(5704));
    assert_ne!(game.object_ref(saga), prior);
    assert_eq!(stats(&game, saga), (5, 5));
    assert!(game.keywords(saga).contains(Keywords::MENACE));
    game.state.objects[saga as usize].summoning_sick = false;
    game.state.players[p.index()].mana = [0; 6];
    declare_attackers(&mut game, &[saga]);
    reach(&mut game, |g| {
        g.state.pending == Pending::ResolutionX { player: p }
    });
    assert_eq!(game.state.players[p.index()].combat_red_mana, 3);
    reject(&mut game, Action::ChooseX { value: 4 });
    act(&mut game, Action::ChooseX { value: 2 });
    act(
        &mut game,
        Action::PayResolutionMana {
            payment: [0, 0, 0, 2, 0, 0],
        },
    );
    reject(&mut game, target(own));
    reject(&mut game, target(enemy));
    act(&mut game, target(other));
    finish_targets(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(other).zone, Zone::Battlefield);
    assert_eq!(game.object_view(other).controller, p);
    assert_eq!(game.state.players[p.index()].combat_red_mana, 1);
}
