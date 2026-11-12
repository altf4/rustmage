use super::*;

#[test]
fn river_spirit_requires_five_extra_scries_two_and_wards_for_two() {
    let p = Player::P0;
    let mut game = scenario(p);
    let spirit = put(&mut game, p, "Benevolent River Spirit", Zone::Hand);
    let artifact = put(&mut game, p, "Short Bow", Zone::Battlefield);
    let murder = put(&mut game, p.other(), "Murder", Zone::Hand);
    let announce = Action::AnnounceCast { card: spirit };
    game.state.players[p.index()].mana = [0, 1, 0, 0, 0, 4];
    reject(&mut game, announce.clone());
    game.state.players[p.index()].mana[1] = 2;
    act(&mut game, announce);
    act(&mut game, Action::BeginWaterbend);
    choose_waterbender(&mut game, artifact);
    pay(&mut game);
    assert!(game.object_view(artifact).tapped);
    pass_pair(&mut game);
    pass_pair(&mut game);
    finish_scry(&mut game, p, 2);
    assert_eq!(stats(&game, spirit), (4, 5));
    assert!(game.keywords(spirit).contains(Keywords::FLYING));

    act(&mut game, Action::Pass);
    cast(&mut game, murder, Some(Target::Card(spirit)));
    for _ in 0..8 {
        if matches!(game.state.pending, Pending::ResolutionMana { .. }) {
            break;
        }
        act(&mut game, Action::Pass);
    }
    assert_eq!(game.actor(), Some(p.other()));
    reject(
        &mut game,
        Action::PayResolutionMana {
            payment: [0, 0, 0, 0, 0, 1],
        },
    );
    let mut declined = game.clone();
    act(&mut declined, Action::DeclineResolutionMana);
    settle(&mut declined);
    assert_eq!(declined.object_view(spirit).zone, Zone::Battlefield);
    act(
        &mut game,
        Action::PayResolutionMana {
            payment: [0, 0, 0, 0, 0, 2],
        },
    );
    settle(&mut game);
    assert_eq!(game.object_view(spirit).zone, Zone::Graveyard);
}

#[test]
fn boomerang_draws_for_the_target_controller_and_returns_to_its_owner() {
    let p = Player::P0;
    let mut game = scenario(p);
    let spell = put(&mut game, p, "Boomerang Basics", Zone::Hand);
    let permanent = put(&mut game, p.other(), "Short Bow", Zone::Battlefield);
    let land = put(&mut game, p, "Island", Zone::Battlefield);
    act(&mut game, Action::AnnounceCast { card: spell });
    reject(&mut game, target(land));
    let mut enemy_controlled = game.clone();
    act(&mut enemy_controlled, target(permanent));
    pay(&mut enemy_controlled);
    settle(&mut enemy_controlled);
    assert_eq!(
        enemy_controlled.state.players[p.index()].cards_drawn_this_turn,
        0
    );

    game.change_controller(game.object_ref(permanent), p);
    act(&mut game, target(permanent));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(permanent).zone, Zone::Hand);
    assert_eq!(game.object_view(permanent).owner, p.other());
    assert_eq!(game.state.players[p.index()].cards_drawn_this_turn, 1);
}

#[test]
fn crashing_wave_distributes_three_stuns_among_tapped_opponents() {
    let p = Player::P1;
    let mut game = scenario(p);
    let spell = put(&mut game, p, "Crashing Wave", Zone::Hand);
    let creature = put(&mut game, p.other(), "Bear Cub", Zone::Battlefield);
    let protected = put(&mut game, p.other(), "Hill Giant", Zone::Battlefield);
    let own = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    game.state.objects[protected as usize].tapped = true;
    game.state.objects[protected as usize].gained_keywords = Keywords::HEXPROOF;
    game.state.objects[own as usize].tapped = true;
    game.state.players[p.index()].mana = [0, 2, 0, 0, 0, 1];
    act(&mut game, Action::AnnounceCast { card: spell });
    act(&mut game, Action::ChooseX { value: 1 });
    reject(&mut game, target(protected));
    act(&mut game, target(creature));
    finish_targets(&mut game);
    pay(&mut game);
    pass_pair(&mut game);
    assert!(game.object_view(creature).tapped);
    let allocation = |g: &Game, card| Action::ChooseEffectObject {
        object: g.object_ref(card),
    };
    let own_choice = allocation(&game, own);
    reject(&mut game, own_choice);
    for card in [creature, protected, protected] {
        let choice = allocation(&game, card);
        act(&mut game, choice);
    }
    assert_eq!(game.counter_count(creature, CounterKind::Stun), 1);
    assert_eq!(game.counter_count(protected, CounterKind::Stun), 2);
}

#[test]
fn ember_island_makes_nonlegendary_heroes_or_cowards_with_the_other_types() {
    let p = Player::P0;
    let mut game = scenario(p);
    let spell = put(&mut game, p, "Ember Island Production", Zone::Hand);
    let own = put(
        &mut game,
        p,
        "Malcolm, Alluring Scoundrel",
        Zone::Battlefield,
    );
    let enemy = put(
        &mut game,
        p.other(),
        "Malcolm, Alluring Scoundrel",
        Zone::Battlefield,
    );
    for (mode, recipient, forbidden, size, subtype) in
        [(0, own, enemy, 4, "Hero"), (1, enemy, own, 2, "Coward")]
    {
        let mut branch = game.clone();
        act(&mut branch, Action::AnnounceCast { card: spell });
        act(&mut branch, Action::ChooseMode { mode });
        reject(&mut branch, target(forbidden));
        act(&mut branch, target(recipient));
        pay(&mut branch);
        settle(&mut branch);
        let (id, _) = branch
            .state
            .objects
            .iter()
            .enumerate()
            .find(|(_, object)| object.zone == Zone::Battlefield && object.is_token())
            .unwrap();
        let copy = branch.object_view(id as CardId);
        assert_eq!((copy.power, copy.toughness), (size, size));
        assert_eq!(copy.controller, p);
        assert!(!copy.is_legendary());
        for tag in [subtype, "Siren", "Pirate"] {
            assert!(copy.has_subtype(tag));
        }
        assert!(copy.keywords.contains(Keywords::FLYING));
    }
}

#[test]
fn flexible_waterbender_keeps_counters_through_the_base_switch() {
    let p = Player::P0;
    let mut game = scenario(p);
    let source = put(&mut game, p, "Flexible Waterbender", Zone::Battlefield);
    game.add_counter(source, CounterKind::PlusOne, 1);
    assert_eq!(stats(&game, source), (3, 6));
    assert!(game.keywords(source).contains(Keywords::VIGILANCE));
    game.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
    game.state.objects[source as usize].tapped = true;
    reject(&mut game, activate(source, 0));
    game.state.players[p.index()].mana[5] = 3;
    act(&mut game, activate(source, 0));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(stats(&game, source), (6, 3));
    game.finish_cleanup();
    assert_eq!(stats(&game, source), (3, 6));
}

#[test]
fn geyser_leaper_draws_before_its_required_discard() {
    let p = Player::P1;
    let mut game = scenario(p);
    let source = put(&mut game, p, "Geyser Leaper", Zone::Battlefield);
    let drawn = game.top_library_card(p).unwrap();
    assert!(game.keywords(source).contains(Keywords::FLYING));
    game.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
    act(&mut game, activate(source, 0));
    act(&mut game, Action::BeginWaterbend);
    choose_waterbender(&mut game, source);
    pay(&mut game);
    pass_pair(&mut game);
    assert_eq!(game.object_view(drawn).zone, Zone::Hand);
    act(&mut game, Action::Discard { card: drawn });
    assert_eq!(game.object_view(drawn).zone, Zone::Graveyard);
}

#[test]
fn giant_koi_has_battlefield_unblockability_and_hand_islandcycling() {
    let p = Player::P0;
    let mut game = scenario(p);
    let koi = put(&mut game, p, "Giant Koi", Zone::Hand);
    let island = put(&mut game, p, "Meticulous Archive", Zone::Library);
    let forest = game.state.players[p.index()].library[0];
    game.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
    reject(&mut game, activate(koi, 0));
    act(&mut game, activate(koi, 1));
    pay(&mut game);
    pass_pair(&mut game);
    assert_eq!(game.object_view(koi).zone, Zone::Graveyard);
    reject(&mut game, Action::ChooseSearch { card: forest });
    act(&mut game, Action::ChooseSearch { card: island });
    if game.legal_actions().contains(&Action::FinishSearch) {
        act(&mut game, Action::FinishSearch);
    }
    assert_eq!(game.object_view(island).zone, Zone::Hand);
    game.move_object(koi, Zone::Battlefield);
    game.state.players[p.index()].mana[5] = 3;
    act(&mut game, activate(koi, 0));
    pay(&mut game);
    settle(&mut game);
    assert!(game.keywords(koi).contains(Keywords::CANT_BE_BLOCKED));
    game.finish_cleanup();
    assert!(!game.keywords(koi).contains(Keywords::CANT_BE_BLOCKED));
}

#[test]
fn katara_can_waterbend_herself_and_must_still_be_tapped_at_resolution() {
    let p = Player::P0;
    let mut game = scenario(p);
    let source = put(&mut game, p, "Katara, Bending Prodigy", Zone::Battlefield);
    game.state.objects[source as usize].summoning_sick = true;
    game.state.players[p.index()].mana = [0, 0, 0, 0, 0, 5];
    act(&mut game, activate(source, 0));
    act(&mut game, Action::BeginWaterbend);
    choose_waterbender(&mut game, source);
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.state.players[p.index()].cards_drawn_this_turn, 1);

    let mut enemy_turn = game.clone();
    start_end_step(&mut enemy_turn, p.other());
    settle(&mut enemy_turn);
    assert_eq!(enemy_turn.counter_count(source, CounterKind::PlusOne), 0);

    start_end_step(&mut game, p);
    let mut untapped = game.clone();
    untapped.state.objects[source as usize].tapped = false;
    settle(&mut untapped);
    assert_eq!(untapped.counter_count(source, CounterKind::PlusOne), 0);
    settle(&mut game);
    assert_eq!(game.counter_count(source, CounterKind::PlusOne), 1);
}

#[test]
fn north_pole_patrol_separates_its_two_tap_abilities_and_funding() {
    let p = Player::P1;
    let mut game = scenario(p);
    let patrol = put(&mut game, p, "North Pole Patrol", Zone::Battlefield);
    let land = put(&mut game, p, "Island", Zone::Battlefield);
    let artifact = put(&mut game, p, "Short Bow", Zone::Battlefield);
    let ally = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    let enemy = put(&mut game, p.other(), "Hill Giant", Zone::Battlefield);
    game.state.objects[land as usize].tapped = true;
    let mut untap = game.clone();
    act(&mut untap, activate(patrol, 0));
    reject(&mut untap, target(patrol));
    reject(&mut untap, target(enemy));
    act(&mut untap, target(land));
    pay(&mut untap);
    settle(&mut untap);
    assert!(!untap.object_view(land).tapped && untap.object_view(patrol).tapped);

    game.state.players[p.index()].mana = [0; 6];
    reject(&mut game, activate(patrol, 1));
    game.state.players[p.index()].mana[5] = 1;
    act(&mut game, activate(patrol, 1));
    reject(&mut game, target(ally));
    act(&mut game, target(enemy));
    act(&mut game, Action::BeginWaterbend);
    let source = game.object_ref(patrol);
    reject(&mut game, Action::ChooseWaterbender { object: source });
    choose_waterbender(&mut game, artifact);
    choose_waterbender(&mut game, ally);
    pay(&mut game);
    settle(&mut game);
    assert!(
        [patrol, artifact, ally, enemy]
            .into_iter()
            .all(|id| game.object_view(id).tapped)
    );
}

#[test]
fn serpent_uses_two_independent_graveyard_thresholds() {
    let p = Player::P1;
    let mut game = scenario(p.other());
    act(&mut game, Action::Pass);
    let source = put(&mut game, p, "Serpent of the Pass", Zone::Hand);
    let lesson = put(&mut game, p, "Firebending Lesson", Zone::Graveyard);
    for _ in 0..2 {
        put(&mut game, p, "Firebending Lesson", Zone::Graveyard);
    }
    put(&mut game, p, "Short Bow", Zone::Graveyard);
    put(&mut game, p, "Bear Cub", Zone::Graveyard);
    put(&mut game, p, "Island", Zone::Graveyard);
    put(&mut game, p.other(), "Firebending Lesson", Zone::Graveyard);
    game.state.players[p.index()].mana = [0, 2, 0, 0, 0, 1];
    let announce = Action::AnnounceCast { card: source };
    let mut below = game.clone();
    below.move_object(lesson, Zone::Exile);
    put(&mut below, p, "Short Bow", Zone::Graveyard);
    reject(&mut below, announce.clone());
    let mut underfunded = game.clone();
    underfunded.state.players[p.index()].mana[5] = 0;
    reject(&mut underfunded, announce.clone());
    act(&mut game, announce);
    assert!(!game.keywords(source).contains(Keywords::FLASH));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(stats(&game, source), (6, 5));
    assert_eq!(game.state.players[p.index()].mana, [0; 6]);
}

#[test]
fn spirit_water_revival_exiles_itself_and_applies_the_paid_branch() {
    let p = Player::P0;
    let mut game = scenario(p);
    let spell = put(&mut game, p, "Spirit Water Revival", Zone::Hand);
    let grave = put(&mut game, p, "Bear Cub", Zone::Graveyard);
    let foreign = put(&mut game, p.other(), "Hill Giant", Zone::Graveyard);
    act(&mut game, Action::AnnounceCast { card: spell });
    for (paid, draws) in [(false, 2), (true, 7)] {
        let mut branch = game.clone();
        act(&mut branch, Action::ChooseWaterbendCost { pay: paid });
        pay(&mut branch);
        settle(&mut branch);
        assert_eq!(branch.object_view(spell).zone, Zone::Exile);
        assert_eq!(branch.object_view(foreign).zone, Zone::Graveyard);
        assert_eq!(branch.object_view(grave).zone == Zone::Graveyard, !paid);
        assert_eq!(branch.state.players[p.index()].cards_drawn_this_turn, draws);
        assert_eq!(
            branch.observe(p).players[p.index()].no_maximum_hand_size,
            paid
        );
    }
}

#[test]
fn unagi_flashes_in_observes_the_second_opposing_draw_and_wards_by_waterbend() {
    let p = Player::P1;
    let mut game = scenario(p.other());
    let unagi = put(&mut game, p, "The Unagi of Kyoshi Island", Zone::Hand);
    let draw = put(&mut game, p.other(), "Quick Study", Zone::Hand);
    let third = put(&mut game, p.other(), "Quick Study", Zone::Hand);
    let removal = put(&mut game, p.other(), "Murder", Zone::Hand);
    let artifact = put(&mut game, p.other(), "Short Bow", Zone::Battlefield);
    act(&mut game, Action::Pass);
    cast(&mut game, unagi, None);
    settle(&mut game);
    assert_eq!(stats(&game, unagi), (5, 5));
    assert!(game.keywords(unagi).contains(Keywords::FLASH));
    for spell in [draw, third] {
        if game.actor() != Some(p.other()) {
            act(&mut game, Action::Pass);
        }
        cast(&mut game, spell, None);
        settle(&mut game);
        assert_eq!(game.state.players[p.index()].cards_drawn_this_turn, 2);
    }
    if game.actor() != Some(p.other()) {
        act(&mut game, Action::Pass);
    }
    cast(&mut game, removal, Some(Target::Card(unagi)));
    for _ in 0..8 {
        if matches!(game.state.pending, Pending::ResolutionMana { .. }) {
            break;
        }
        act(&mut game, Action::Pass);
    }
    assert_eq!(game.actor(), Some(p.other()));
    let mut decline = game.clone();
    act(&mut decline, Action::DeclineResolutionMana);
    settle(&mut decline);
    assert_eq!(decline.object_view(unagi).zone, Zone::Battlefield);
    act(&mut game, Action::BeginWaterbend);
    choose_waterbender(&mut game, artifact);
    reject(
        &mut game,
        Action::PayResolutionMana {
            payment: [2, 0, 0, 0, 0, 0],
        },
    );
    act(
        &mut game,
        Action::PayResolutionMana {
            payment: [3, 0, 0, 0, 0, 0],
        },
    );
    settle(&mut game);
    assert_eq!(game.object_view(unagi).zone, Zone::Graveyard);
}

#[test]
fn ascension_counts_each_combat_source_and_targets_an_unblockable_creature() {
    let p = Player::P0;
    let mut game = scenario(p);
    let ascension = put(&mut game, p, "Waterbender Ascension", Zone::Battlefield);
    let a = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    let b = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    game.add_counter(ascension, CounterKind::Quest, 2);
    act(&mut game, activate(ascension, 0));
    act(&mut game, target(a));
    pay(&mut game);
    settle(&mut game);
    assert!(game.keywords(a).contains(Keywords::CANT_BE_BLOCKED));

    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    for card in [a, b] {
        act(&mut game, Action::Attack { card });
    }
    act(&mut game, Action::FinishAttackers);
    pass_pair(&mut game);
    act(&mut game, Action::FinishBlockers);
    for _ in 0..12 {
        if game.counter_count(ascension, CounterKind::Quest) == 4 {
            break;
        }
        choose(&mut game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    assert_eq!(game.counter_count(ascension, CounterKind::Quest), 4);
    assert_eq!(game.state.players[p.index()].cards_drawn_this_turn, 1);
    game.finish_cleanup();
    assert!(!game.keywords(a).contains(Keywords::CANT_BE_BLOCKED));
}

#[test]
fn waterbending_lesson_draws_three_before_offering_waterbend_two() {
    let p = Player::P1;
    let mut game = scenario(p);
    let spell = put(&mut game, p, "Waterbending Lesson", Zone::Hand);
    let artifact = put(&mut game, p, "Short Bow", Zone::Battlefield);
    let creature = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    let drawn = game.top_library_card(p).unwrap();
    cast(&mut game, spell, None);
    pass_pair(&mut game);
    assert_eq!(game.state.players[p.index()].cards_drawn_this_turn, 3);
    let mut decline = game.clone();
    act(&mut decline, Action::DeclineResolutionMana);
    act(&mut decline, Action::Discard { card: drawn });
    assert_eq!(decline.observe(p).players[p.index()].hand_count, 2);
    act(&mut game, Action::BeginWaterbend);
    choose_waterbender(&mut game, artifact);
    choose_waterbender(&mut game, creature);
    act(&mut game, Action::PayResolutionMana { payment: [0; 6] });
    assert_eq!(game.observe(p).players[p.index()].hand_count, 3);
    assert!(game.object_view(artifact).tapped && game.object_view(creature).tapped);
}

#[test]
fn waterbending_scroll_counts_only_current_controlled_islands() {
    let p = Player::P0;
    let mut game = scenario(p);
    let scroll = put(&mut game, p, "Waterbending Scroll", Zone::Battlefield);
    for name in ["Island", "Meticulous Archive"] {
        let land = put(&mut game, p, name, Zone::Battlefield);
        game.state.objects[land as usize].tapped = true;
    }
    put(&mut game, p.other(), "Island", Zone::Battlefield);
    game.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
    reject(&mut game, activate(scroll, 0));
    game.state.players[p.index()].mana[5] = 4;
    act(&mut game, activate(scroll, 0));
    pay(&mut game);
    settle(&mut game);
    assert!(game.object_view(scroll).tapped);
    assert_eq!(game.state.players[p.index()].cards_drawn_this_turn, 1);
    assert_eq!(game.state.players[p.index()].mana, [0; 6]);
}

#[test]
fn watery_grasp_stops_untap_and_waterbends_the_attached_creature_into_its_owners_library() {
    let p = Player::P0;
    let mut game = scenario(p);
    let aura = put(&mut game, p, "Watery Grasp", Zone::Hand);
    let creature = put(&mut game, p.other(), "Hill Giant", Zone::Battlefield);
    cast(&mut game, aura, Some(Target::Card(creature)));
    settle(&mut game);
    assert!(!game.object_view(creature).tapped);
    game.state.objects[creature as usize].tapped = true;
    game.state.active = p.other();
    game.begin_step(Step::Untap);
    assert!(game.object_view(creature).tapped);
    game.state.active = p;
    game.state.priority = p;
    game.state.step = Step::Main1;
    act(&mut game, activate(aura, 0));
    pay(&mut game);
    game.move_object(aura, Zone::Graveyard);
    settle(&mut game);
    assert_eq!(game.object_view(creature).zone, Zone::Library);
    assert!(
        game.state.players[p.other().index()]
            .library
            .contains(&creature)
    );
}

#[test]
fn yue_requires_tap_and_waterbend_then_offers_only_noncreature_spells() {
    let p = Player::P1;
    let mut game = scenario(p);
    let yue = put(&mut game, p, "Yue, the Moon Spirit", Zone::Battlefield);
    let spell = put(&mut game, p, "Divination", Zone::Hand);
    let creature = put(&mut game, p, "Bear Cub", Zone::Hand);
    assert!(
        game.keywords(yue)
            .contains(Keywords::FLYING.union(Keywords::VIGILANCE))
    );
    game.state.players[p.index()].mana = [0, 0, 0, 0, 0, 5];
    act(&mut game, activate(yue, 0));
    pay(&mut game);
    pass_pair(&mut game);
    let cast_free = |g: &Game, card| Action::CastFreeDuringResolution {
        object: g.object_ref(card),
        face: CastFace::Normal,
    };
    let forbidden = cast_free(&game, creature);
    reject(&mut game, forbidden);
    let mut declined = game.clone();
    act(&mut declined, Action::FinishResolutionCasts);
    assert_eq!(declined.object_view(spell).zone, Zone::Hand);
    let action = cast_free(&game, spell);
    act(&mut game, action);
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
    assert!(game.object_view(yue).tapped);
    assert_eq!(game.state.players[p.index()].cards_drawn_this_turn, 2);
    assert_eq!(game.state.players[p.index()].mana, [0; 6]);
}
