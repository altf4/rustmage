use super::*;

#[test]
fn aang_at_the_crossroads_looks_privately_then_returns_as_a_land_creature_provider() {
    let player = Player::P1;
    let mut game = scenario(player);
    let own_land = put(&mut game, player, "Island", Zone::Battlefield);
    let enemy_land = put(&mut game, player.other(), "Mountain", Zone::Battlefield);
    let enemy = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let eligible = put(&mut game, player, "Bear Cub", Zone::Library);
    let oversized = put(&mut game, player, "Earthshaker Dreadmaw", Zone::Library);
    let spell = put(&mut game, player, "Opt", Zone::Library);
    let front = put(&mut game, player, "Aang, at the Crossroads", Zone::Hand);
    cast(&mut game, front, None);
    pass_pair(&mut game);
    pass_pair(&mut game);
    assert!(game.observe(player).library_selection.is_some());
    assert!(game.observe(player.other()).library_selection.is_none());
    for card in [oversized, spell] {
        reject(&mut game, Action::SelectLibrary { card });
    }
    let mut declined = game.clone();
    act(&mut declined, Action::FinishLibrarySelection);
    drain(&mut declined);
    assert_eq!(declined.object_view(eligible).zone, Zone::Library);
    act(&mut game, Action::SelectLibrary { card: eligible });
    if game
        .legal_actions()
        .contains(&Action::FinishLibrarySelection)
    {
        act(&mut game, Action::FinishLibrarySelection);
    }
    drain(&mut game);
    assert_eq!(game.object_view(eligible).zone, Zone::Battlefield);
    assert!(game.state.players[player.index()].library[..4].contains(&oversized));
    assert!(game.state.players[player.index()].library[..4].contains(&spell));
    assert!(game.keywords(front).contains(Keywords::FLYING));

    game.move_object(enemy, Zone::Graveyard);
    drain(&mut game);
    assert!(
        game.observe(player)
            .delayed_steps
            .is_none_or(|rows| rows.is_empty())
    );
    let exact = game.object_ref(front);
    game.move_object(eligible, Zone::Hand);
    drain(&mut game);
    assert_eq!(game.observe(player).delayed_steps.unwrap().len(), 1);
    game.state.active = player.other();
    game.state.priority = player.other();
    game.begin_step(Step::Upkeep);
    drain(&mut game);
    assert_eq!(game.object_ref(front), exact);
    assert_eq!(game.object_view(front).name, "Aang, Destined Savior");
    assert_eq!(stats(&game, front), (4, 4));

    game.state.active = player;
    game.state.priority = player;
    game.begin_step(Step::BeginCombat);
    game.prepare_triggers();
    reject(&mut game, target(enemy_land));
    act(&mut game, target(own_land));
    drain(&mut game);
    assert_eq!(stats(&game, own_land), (2, 2));
    assert!(
        game.keywords(own_land)
            .contains(Keywords::VIGILANCE.union(Keywords::HASTE))
    );
    assert!(!game.keywords(front).contains(Keywords::VIGILANCE));
}

#[test]
fn avatar_aang_combines_all_four_bends_and_offers_the_reverse_transform() {
    let player = Player::P0;
    let mut game = scenario(player);
    let aang = put(&mut game, player, "Avatar Aang", Zone::Battlefield);
    let swift = put(&mut game, player, "Aang, Swift Savior", Zone::Hand);
    let bumi = put(&mut game, player, "Bumi, Unleashed", Zone::Hand);
    let land = put(&mut game, player, "Forest", Zone::Battlefield);
    let victim = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    cast(&mut game, bumi, None);
    pass_pair(&mut game);
    act(&mut game, target(land));
    drain(&mut game);
    assert_eq!(draws(&game, player), 1);
    cast(&mut game, swift, None);
    pass_pair(&mut game);
    act(&mut game, target(victim));
    drain(&mut game);
    assert_eq!(game.object_view(victim).zone, Zone::Exile);
    assert_eq!(draws(&game, player), 2);
    assert_eq!(stats(&game, aang), (4, 4));
    attack(&mut game, &[aang]);
    drain(&mut game);
    assert_eq!(game.state.players[player.index()].combat_red_mana, 2);
    assert_eq!(draws(&game, player), 3);
    assert_eq!(game.object_view(aang).name, "Avatar Aang");
    act(&mut game, activation(swift, 0));
    pay(&mut game);
    drain(&mut game);
    assert_eq!(draws(&game, player), 4);
    assert_eq!(game.object_view(aang).name, "Aang, Master of Elements");
    assert_eq!(stats(&game, aang), (6, 6));
    assert_eq!(
        game.state.players[player.index()].turn_facts.bent_elements,
        0b1111
    );

    game.state.step = Step::Main2;
    let giant = put(&mut game, player, "Hill Giant", Zone::Hand);
    game.state.players[player.index()].combat_red_mana = 0;
    game.state.players[player.index()].mana = [0; 6];
    cast(&mut game, giant, None);
    drain(&mut game);
    assert_eq!(game.object_view(giant).zone, Zone::Battlefield);

    game.state.active = player.other();
    game.state.priority = player.other();
    game.begin_step(Step::Upkeep);
    game.prepare_triggers();
    pass_pair(&mut game);
    let mut declined = game.clone();
    act(&mut declined, Action::ChooseOptional { accept: false });
    drain(&mut declined);
    assert_eq!(declined.object_view(aang).name, "Aang, Master of Elements");
    assert_eq!(life(&declined, player), 20);
    act(&mut game, Action::ChooseOptional { accept: true });
    drain(&mut game);
    assert_eq!(game.object_view(aang).name, "Avatar Aang");
    assert_eq!(stats(&game, aang), (8, 8));
    assert_eq!(draws(&game, player), 8);
    assert_eq!(life(&game, player), 24);
    assert_eq!(life(&game, player.other()), 16);
}

#[test]
fn bounty_hunters_use_nonland_death_power_and_ruffians_register_their_own_return() {
    let player = Player::P0;
    let mut game = scenario(player);
    put(
        &mut game,
        player,
        "Beifong's Bounty Hunters",
        Zone::Battlefield,
    );
    let land = put(&mut game, player, "Forest", Zone::Battlefield);
    let second_land = put(&mut game, player, "Island", Zone::Battlefield);
    let animated = put(&mut game, player, "Mountain", Zone::Battlefield);
    let cub = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let ruffians = put(
        &mut game,
        player,
        "Earth Village Ruffians",
        Zone::Battlefield,
    );
    animate(&mut game, animated, Some((2, 2)));
    game.move_object(animated, Zone::Graveyard);
    drain(&mut game);
    assert_eq!(game.counter_count(land, CounterKind::PlusOne), 0);
    game.add_counter(cub, CounterKind::PlusOne, 3);
    game.move_object(cub, Zone::Graveyard);
    game.prepare_triggers();
    act(&mut game, target(land));
    drain(&mut game);
    assert_eq!(stats(&game, land), (5, 5));

    game.move_object(ruffians, Zone::Exile);
    drain(&mut game);
    assert_eq!(game.counter_count(second_land, CounterKind::PlusOne), 0);
    game.move_object(ruffians, Zone::Battlefield);
    game.move_object(ruffians, Zone::Graveyard);
    game.prepare_triggers();
    for land in [second_land, land] {
        if matches!(game.state.pending, Pending::OrderTriggers { .. }) {
            choose(&mut game, |action| {
                matches!(action, Action::PutTrigger { .. })
            });
        }
        act(&mut game, target(land));
    }
    drain(&mut game);
    let counters = [
        game.counter_count(land, CounterKind::PlusOne) - 5,
        game.counter_count(second_land, CounterKind::PlusOne),
    ];
    assert!(counters == [2, 3] || counters == [3, 2]);
    let old_land = game.object_ref(second_land);
    game.move_object(second_land, Zone::Exile);
    drain(&mut game);
    assert_ne!(game.object_ref(second_land), old_land);
    assert!(game.object_view(second_land).tapped);
    assert_eq!(game.object_view(second_land).kinds, 2);
}

#[test]
fn bitter_work_draws_once_for_qualifying_attackers_and_exhausts_only_on_its_turn() {
    let player = Player::P0;
    let mut game = scenario(player);
    let bitter = put(&mut game, player, "Bitter Work", Zone::Battlefield);
    let first = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let second = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let land = put(&mut game, player, "Forest", Zone::Battlefield);
    game.add_counter(first, CounterKind::PlusOne, 1);
    game.add_counter(second, CounterKind::PlusOne, 2);
    attack(&mut game, &[first, second]);
    drain(&mut game);
    assert_eq!(draws(&game, player), 1);

    let announce = activation(bitter, 0);
    let mut off_turn = game.clone();
    off_turn.state.active = player.other();
    reject(&mut off_turn, announce.clone());
    act(&mut game, announce.clone());
    act(&mut game, target(land));
    pay(&mut game);
    drain(&mut game);
    assert_eq!(stats(&game, land), (4, 4));
    assert_eq!(game.object_view(bitter).exhausted_abilities, 1);
    game.finish_cleanup();
    assert_eq!(game.object_view(bitter).exhausted_abilities, 1);
    reject(&mut game, announce);
}

#[test]
fn bumis_damage_inserts_a_combat_for_land_creatures_after_untapping_lands() {
    let player = Player::P0;
    let mut game = scenario(player);
    let bumi = put(&mut game, player, "Bumi, Unleashed", Zone::Hand);
    let land = put(&mut game, player, "Forest", Zone::Battlefield);
    let island = put(&mut game, player, "Island", Zone::Battlefield);
    let ordinary = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    cast(&mut game, bumi, None);
    pass_pair(&mut game);
    act(&mut game, target(land));
    drain(&mut game);
    assert_eq!(stats(&game, land), (4, 4));
    assert!(game.keywords(bumi).contains(Keywords::TRAMPLE));
    game.state.objects[bumi as usize].summoning_sick = false;
    game.state.objects[island as usize].tapped = true;
    attack(&mut game, &[bumi, land]);
    pass_pair(&mut game);
    act(&mut game, Action::FinishBlockers);
    pass_pair(&mut game);
    pass_pair(&mut game);
    drain(&mut game);
    assert_eq!(life(&game, player.other()), 11);
    assert!(!game.object_view(land).tapped);
    assert!(!game.object_view(island).tapped);
    for _ in 0..16 {
        if game.state.pending == Pending::Attackers {
            break;
        }
        act(&mut game, Action::Pass);
    }
    assert_eq!(game.state.pending, Pending::Attackers);
    reject(&mut game, Action::Attack { card: ordinary });
    act(&mut game, Action::Attack { card: land });
}

#[test]
fn cruel_administrator_gets_raid_on_entry_and_creates_a_soldier_on_attack() {
    let player = Player::P0;
    let mut game = scenario(player);
    let before = put(&mut game, player, "Cruel Administrator", Zone::Hand);
    let after = put(&mut game, player, "Cruel Administrator", Zone::Hand);
    cast(&mut game, before, None);
    drain(&mut game);
    assert_eq!(stats(&game, before), (5, 4));
    game.state.objects[before as usize].summoning_sick = false;
    attack(&mut game, &[before]);
    drain(&mut game);
    let soldiers = tokens(&game, player, 5706);
    assert_eq!(soldiers.len(), 1);
    assert_eq!(stats(&game, soldiers[0]), (2, 2));
    assert!(!game.object_view(soldiers[0]).attacking);
    game.state.step = Step::Main2;
    game.state.players[player.index()].mana = [8; 6];
    cast(&mut game, after, None);
    drain(&mut game);
    assert_eq!(stats(&game, after), (6, 5));
}

#[test]
fn dai_li_agents_can_earthbend_the_same_land_twice_and_drain_live_recipients() {
    let player = Player::P1;
    let mut game = scenario(player);
    let agents = put(&mut game, player, "Dai Li Agents", Zone::Hand);
    let land = put(&mut game, player, "Forest", Zone::Battlefield);
    let cub = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let other = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    game.add_counter(cub, CounterKind::PlusOne, 1);
    game.add_counter(other, CounterKind::PlusOne, 1);
    cast(&mut game, agents, None);
    pass_pair(&mut game);
    act(&mut game, target(land));
    act(&mut game, target(land));
    drain(&mut game);
    assert_eq!(stats(&game, land), (2, 2));
    assert_eq!(game.earthbend_views().len(), 2);
    game.state.objects[agents as usize].summoning_sick = false;
    attack(&mut game, &[agents]);
    drain(&mut game);
    assert_eq!((life(&game, player), life(&game, player.other())), (22, 18));
}

#[test]
fn earth_king_lieutenant_and_earthen_ally_read_effective_allies_and_colors() {
    let player = Player::P0;
    let mut game = scenario(player);
    let earthen = put(&mut game, player, "Earthen Ally", Zone::Battlefield);
    let lieutenant = put(&mut game, player, "Earth King's Lieutenant", Zone::Hand);
    let swift = put(&mut game, player, "Aang, Swift Savior", Zone::Battlefield);
    let hostile = put(&mut game, player.other(), "Avatar Aang", Zone::Battlefield);
    let cub = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let land = put(&mut game, player, "Forest", Zone::Battlefield);
    assert_eq!(stats(&game, earthen), (3, 2));
    cast(&mut game, lieutenant, None);
    drain(&mut game);
    assert!(game.keywords(lieutenant).contains(Keywords::TRAMPLE));
    for card in [earthen, swift] {
        assert_eq!(game.counter_count(card, CounterKind::PlusOne), 1);
    }
    for card in [hostile, cub, lieutenant] {
        assert_eq!(game.counter_count(card, CounterKind::PlusOne), 0);
    }
    let kindred = put(&mut game, player, "Firdoch Core", Zone::Hand);
    cast(&mut game, kindred, None);
    drain(&mut game);
    assert!(!game.is_creature(kindred));
    assert!(game.has_subtype(kindred, "Ally"));
    assert_eq!(game.counter_count(lieutenant, CounterKind::PlusOne), 1);
    game.state.players[player.index()].mana = [1, 1, 1, 1, 1, 2];
    act(&mut game, activation(earthen, 0));
    act(&mut game, target(land));
    pay(&mut game);
    drain(&mut game);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    assert_eq!(stats(&game, land), (5, 5));
}

#[test]
fn earth_rumble_wrestlers_track_qualifying_lands_and_this_turns_entries() {
    let player = Player::P0;
    let mut game = scenario(player);
    let wrestlers = put(
        &mut game,
        player,
        "Earth Rumble Wrestlers",
        Zone::Battlefield,
    );
    let old = put(&mut game, player, "Forest", Zone::Battlefield);
    let land = put(&mut game, player, "Island", Zone::Hand);
    game.state.entered_this_turn = None;
    assert_eq!(stats(&game, wrestlers), (3, 4));
    assert_eq!(game.keywords(wrestlers), Keywords::REACH);
    let mut animated = game.clone();
    animate(&mut animated, old, Some((2, 2)));
    assert_eq!(stats(&animated, wrestlers), (4, 4));
    assert!(animated.keywords(wrestlers).contains(Keywords::TRAMPLE));
    act(&mut game, Action::PlayLand { card: land });
    game.move_object(land, Zone::Graveyard);
    assert_eq!(stats(&game, wrestlers), (4, 4));
    assert!(game.keywords(wrestlers).contains(Keywords::TRAMPLE));
    game.finish_cleanup();
    assert_eq!(stats(&game, wrestlers), (3, 4));
}

#[test]
fn toph_makes_only_nontoken_artifacts_lands_and_earthbends_an_exact_artifact() {
    let player = Player::P1;
    let mut game = scenario(player);
    let toph = put(
        &mut game,
        player,
        "Toph, the First Metalbender",
        Zone::Battlefield,
    );
    let equipment = put(&mut game, player, "Short Bow", Zone::Hand);
    let enemy = put(&mut game, player.other(), "Short Bow", Zone::Battlefield);
    let deduce = put(&mut game, player, "Deduce", Zone::Hand);
    reject(&mut game, Action::PlayLand { card: equipment });
    cast(&mut game, equipment, None);
    cast(&mut game, deduce, None);
    drain(&mut game);
    let clue = tokens(&game, player, 5296)[0];
    assert_eq!(game.object_view(equipment).kinds, 2 | 16);
    assert_eq!(game.object_view(enemy).kinds, 16);
    assert_eq!(game.object_view(clue).kinds, 16);
    reject(&mut game, Action::TapMana { card: equipment });

    game.begin_step(Step::End);
    game.prepare_triggers();
    reject(&mut game, target(clue));
    act(&mut game, target(equipment));
    drain(&mut game);
    assert_eq!(game.object_view(equipment).kinds, 1 | 2 | 16);
    assert_eq!(stats(&game, equipment), (2, 2));
    game.move_object(toph, Zone::Exile);
    assert_eq!(game.object_view(equipment).kinds, 1 | 2 | 16);
    let exact = game.object_ref(equipment);
    game.move_object(equipment, Zone::Graveyard);
    drain(&mut game);
    assert_ne!(game.object_ref(equipment), exact);
    assert_eq!(game.object_view(equipment).kinds, 16);
    assert!(game.object_view(equipment).tapped);
}
