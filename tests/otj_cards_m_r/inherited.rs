use super::*;

#[test]
fn inherited_cast_observers_use_actual_ordinals_and_the_clue_draws() {
    let player = Player::P0;
    let mut game = scenario(player);
    let malcolm = board(&mut game, player, "Malcolm, the Eyes");
    board(&mut game, player, "Rodeo Pyromancers");
    assert!(keywords(&game, malcolm).contains(Keywords::FLYING.union(Keywords::HASTE)));
    let shock = hand(&mut game, player, "Shock");
    cast(&mut game, shock, Some(Target::Player(player.other())));
    assert_eq!(game.state.players[player.index()].mana[3], 7);
    quiet(&mut game);
    assert_eq!(game.state.players[player.index()].mana[3], 9);
    assert!(tokens(&game, player, 5296).is_empty());

    for name in ["Grizzly Bears", "Bear Cub"] {
        let card = hand(&mut game, player, name);
        cast(&mut game, card, None);
        quiet(&mut game);
        assert_eq!(tokens(&game, player, 5296).len(), 1);
        assert_eq!(game.state.players[player.index()].mana[3], 9);
    }
    let clue = tokens(&game, player, 5296)[0];
    activate(&mut game, clue, 0, &[]);
    quiet(&mut game);
    assert!(tokens(&game, player, 5296).is_empty());
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
}

#[test]
fn inherited_desert_searches_choose_basic_or_desert_and_greenblade_scales_live() {
    let player = Player::P1;
    let mut game = scenario(player);
    let desert = top(&mut game, player, "Abraded Bluffs");
    let forest = top(&mut game, player, "Forest");
    let reveal = top(&mut game, player, "Island");
    let map = hand(&mut game, player, "Map the Frontier");
    cast(&mut game, map, None);
    for card in [forest, desert] {
        let choice = Action::ChooseSearch { card };
        until_action(&mut game, &choice);
        act(&mut game, choice);
    }
    if has(&game, &Action::FinishSearch) {
        act(&mut game, Action::FinishSearch);
    }
    quiet(&mut game);
    for land in [forest, desert] {
        assert_eq!(zone(&game, land), Zone::Battlefield);
        assert!(game.object_view(land).tapped);
    }
    let greenblade = hand(&mut game, player, "Outcaster Greenblade");
    cast(&mut game, greenblade, None);
    let choice = Action::ChooseSearch { card: reveal };
    until_action(&mut game, &choice);
    act(&mut game, choice);
    if has(&game, &Action::FinishSearch) {
        act(&mut game, Action::FinishSearch);
    }
    quiet(&mut game);
    assert_eq!(zone(&game, reveal), Zone::Hand);
    assert!(
        game.observe(player.other())
            .known_hand
            .contains(&game.object_ref(reveal))
    );
    assert_eq!(stats(&game, greenblade), (2, 3));
    let exact = game.object_ref(desert);
    game.change_controller(exact, player.other());
    assert_eq!(stats(&game, greenblade), (1, 2));
}

#[test]
fn inherited_mystical_tether_pays_its_flash_surcharge_and_releases_the_exact_exile() {
    let active = Player::P0;
    let player = active.other();
    let mut game = scenario(active);
    let creature = board(&mut game, active, "Hill Giant");
    let tether = hand(&mut game, player, "Mystical Tether");
    act(&mut game, Action::Pass);
    reject(&mut game, Action::AnnounceCast { card: tether });
    act(&mut game, Action::AnnounceFlashSurcharge { card: tether });
    pay(&mut game);
    let target = Action::ChooseTarget {
        target: Target::Card(creature),
    };
    until_action(&mut game, &target);
    act(&mut game, target);
    quiet(&mut game);
    let exiled = game.object_ref(creature);
    assert_eq!(zone(&game, creature), Zone::Exile);
    reach(&mut game, |g| g.actor() == Some(active));
    let disenchant = hand(&mut game, active, "Disenchant");
    cast(&mut game, disenchant, Some(Target::Card(tether)));
    quiet(&mut game);
    assert_eq!(zone(&game, tether), Zone::Graveyard);
    assert_eq!(zone(&game, creature), Zone::Battlefield);
    assert_ne!(game.object_ref(creature), exiled);
    assert_eq!(game.object_view(creature).controller, active);
}

#[test]
fn inherited_oasis_gardener_gains_life_and_basics_have_intrinsic_mana() {
    let player = Player::P0;
    let mut game = scenario(player);
    let gardener = hand(&mut game, player, "Oasis Gardener");
    cast(&mut game, gardener, None);
    quiet(&mut game);
    assert_eq!(life(&game, player), 22);
    assert!(game.object_view(gardener).summoning_sick);
    assert!(
        !game
            .legal_actions()
            .iter()
            .any(|a| matches!(a, Action::ActivateMana { card, .. } if *card == gardener))
    );
    for (name, color) in [("Plains", 0), ("Mountain", 3)] {
        let mut branch = game.clone();
        let land = hand(&mut branch, player, name);
        act(&mut branch, Action::PlayLand { card: land });
        assert!(!branch.object_view(land).tapped);
        let before = branch.state.players[player.index()].mana;
        act(&mut branch, Action::TapMana { card: land });
        assert_eq!(
            branch.state.players[player.index()].mana[color],
            before[color] + 1
        );
    }
    let turn = game.state.turn;
    reach(&mut game, |g| {
        g.state.turn > turn && g.state.active == player && g.state.step == Step::Main1
    });
    choose(
        &mut game,
        |a| matches!(a, Action::ActivateMana { card, production: [0, 1, 0, 0, 0, 0], .. } if *card == gardener),
    );
    assert_eq!(game.state.players[player.index()].mana[1], 1);
}

#[test]
fn inherited_oko_copies_at_combat_and_can_create_elk_or_copy_other_nonlands() {
    let player = Player::P1;
    let mut game = scenario(player);
    let bear = board(&mut game, player, "Grizzly Bears");
    board(&mut game, player, "Forest");
    let oko = hand(&mut game, player, "Oko, the Ringleader");
    cast(&mut game, oko, None);
    quiet(&mut game);
    let mut elk = game.clone();
    activate(&mut elk, oko, 1, &[]);
    quiet(&mut elk);
    assert_eq!(stats(&elk, tokens(&elk, player, 5474)[0]), (3, 3));
    assert_eq!(elk.counter_count(oko, CounterKind::Loyalty), 2);

    let mut copy_all = game.clone();
    copy_all.add_counter(oko, CounterKind::Loyalty, 3);
    activate(&mut copy_all, oko, 2, &[]);
    quiet(&mut copy_all);
    assert_eq!(copy_all.counter_count(oko, CounterKind::Loyalty), 1);
    let bears = copy_all
        .state
        .objects
        .iter()
        .filter(|o| o.zone == Zone::Battlefield && o.card == Card::GrizzlyBears)
        .count();
    assert_eq!(bears, 2);
    let forests = copy_all
        .state
        .objects
        .iter()
        .filter(|o| o.zone == Zone::Battlefield && o.card == Card::Forest)
        .count();
    assert_eq!(forests, 1);

    let copy = Action::ChooseTarget {
        target: Target::Card(bear),
    };
    until_action(&mut game, &copy);
    act(&mut game, copy);
    if has(&game, &Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    quiet(&mut game);
    assert_eq!(game.object_view(oko).card, Card::GrizzlyBears);
    assert!(keywords(&game, oko).contains(Keywords::HEXPROOF));
    assert_eq!(game.counter_count(oko, CounterKind::Loyalty), 3);
    reach(&mut game, |g| {
        g.state.active == player.other() && g.state.step == Step::Main1
    });
    assert_eq!(game.object_view(oko).name, "Oko, the Ringleader");
}

#[test]
fn inherited_outlaw_medic_and_its_aura_each_draw_on_real_death() {
    let player = Player::P0;
    let mut game = scenario(player);
    let medic = board(&mut game, player, "Outlaw Medic");
    let aura = hand(&mut game, player, "Reach for the Sky");
    cast(&mut game, aura, Some(Target::Card(medic)));
    quiet(&mut game);
    assert_eq!(stats(&game, medic), (4, 5));
    assert!(keywords(&game, medic).contains(Keywords::LIFELINK.union(Keywords::REACH)));
    let destroy = hand(&mut game, player, "Murder");
    cast(&mut game, destroy, Some(Target::Card(medic)));
    quiet(&mut game);
    assert_eq!(zone(&game, medic), Zone::Graveyard);
    assert_eq!(zone(&game, aura), Zone::Graveyard);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 2);
}

#[test]
fn inherited_outlaws_fury_grants_an_exact_top_play_and_freezes_its_creature_group() {
    let player = Player::P0;
    let mut game = scenario(player);
    let outlaw = board(&mut game, player, "Reckless Lackey");
    let creature = board(&mut game, player, "Grizzly Bears");
    let land = top(&mut game, player, "Island");
    let fury = hand(&mut game, player, "Outlaws' Fury");
    cast(&mut game, fury, None);
    quiet(&mut game);
    assert_eq!(stats(&game, outlaw), (3, 2));
    assert_eq!(stats(&game, creature), (4, 2));
    let late = board(&mut game, player, "Grizzly Bears");
    assert_eq!(stats(&game, late), (2, 2));
    assert_eq!(zone(&game, land), Zone::Exile);
    act(&mut game, Action::PlayLand { card: land });
    assert_eq!(zone(&game, land), Zone::Battlefield);
}

#[test]
fn inherited_rustler_rampage_can_save_peerless_ropemasters_tapped_target() {
    let player = Player::P1;
    let mut game = scenario(player);
    let own = board(&mut game, player, "Grizzly Bears");
    let target = board(&mut game, player.other(), "Hill Giant");
    game.state.objects[target as usize].tapped = true;
    let ropemaster = hand(&mut game, player, "Peerless Ropemaster");
    cast(&mut game, ropemaster, None);
    let choose_target = Action::ChooseTarget {
        target: Target::Card(target),
    };
    until_action(&mut game, &choose_target);
    act(&mut game, choose_target);
    if has(&game, &Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    let mut without = game.clone();
    quiet(&mut without);
    assert_eq!(zone(&without, target), Zone::Hand);

    let rustler = hand(&mut game, player, "Rustler Rampage");
    modal(
        &mut game,
        rustler,
        2,
        &[Target::Player(player.other()), Target::Card(own)],
    );
    quiet(&mut game);
    assert_eq!(zone(&game, target), Zone::Battlefield);
    assert!(!game.object_view(target).tapped);
    assert!(keywords(&game, own).contains(Keywords::DOUBLE_STRIKE));
}

#[test]
fn inherited_requisition_raid_combines_three_modes_for_separate_roles() {
    let player = Player::P0;
    let mut game = scenario(player);
    let artifact = board(&mut game, player.other(), "Gingerbrute");
    let enchantment = board(&mut game, player.other(), "Rakish Crew");
    let own = board(&mut game, player, "Grizzly Bears");
    let raid = hand(&mut game, player, "Requisition Raid");
    modal(
        &mut game,
        raid,
        6,
        &[
            Target::Card(artifact),
            Target::Card(enchantment),
            Target::Player(player),
        ],
    );
    quiet(&mut game);
    assert_eq!(zone(&game, artifact), Zone::Graveyard);
    assert_eq!(zone(&game, enchantment), Zone::Graveyard);
    assert_eq!(counters(&game, own), 1);
}

#[test]
fn inherited_reckless_lackey_and_redrock_sentinel_pay_costs_before_the_draw_and_treasure() {
    let player = Player::P0;
    let mut game = scenario(player);
    let lackey = board(&mut game, player, "Reckless Lackey");
    let sentinel = board(&mut game, player, "Redrock Sentinel");
    let land = board(&mut game, player, "Forest");
    assert!(keywords(&game, lackey).contains(Keywords::HASTE.union(Keywords::FIRST_STRIKE)));
    assert!(keywords(&game, sentinel).contains(Keywords::DEFENDER));
    activate(&mut game, lackey, 0, &[]);
    assert_eq!(zone(&game, lackey), Zone::Graveyard);
    assert!(tokens(&game, player, TREASURE).is_empty());
    quiet(&mut game);
    assert_eq!(tokens(&game, player, TREASURE).len(), 1);
    announce_activation(&mut game, sentinel, 0);
    finish_additional_cost(&mut game, &[land]);
    assert_eq!(zone(&game, land), Zone::Graveyard);
    assert!(game.object_view(sentinel).tapped);
    quiet(&mut game);
    assert_eq!(tokens(&game, player, TREASURE).len(), 2);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 2);
}

#[test]
fn inherited_death_and_graveyard_cards_distinguish_died_creatures_from_other_cards() {
    let player = Player::P1;
    let mut game = scenario(player);
    let own = board(&mut game, player, "Bear Cub");
    let grave = graveyard(&mut game, player, "Grizzly Bears");
    graveyard(&mut game, player, "Forest");
    graveyard(&mut game, player.other(), "Bear Cub");
    let robber = hand(&mut game, player, "Rictus Robber");
    let rise = hand(&mut game, player, "Rise of the Varmints");
    let mut no_death = game.clone();
    cast(&mut no_death, robber, None);
    quiet(&mut no_death);
    assert!(tokens(&no_death, player, ZOMBIE_ROGUE).is_empty());

    let shock = hand(&mut game, player, "Shock");
    cast(&mut game, shock, Some(Target::Card(own)));
    quiet(&mut game);
    cast(&mut game, robber, None);
    quiet(&mut game);
    assert_eq!(tokens(&game, player, ZOMBIE_ROGUE).len(), 1);
    cast(&mut game, rise, None);
    quiet(&mut game);
    assert_eq!(zone(&game, grave), Zone::Graveyard);
    let varmints = tokens(&game, player, 5289);
    assert_eq!(varmints.len(), 2);
    assert!(varmints.iter().all(|&card| stats(&game, card) == (2, 1)));
}

#[test]
fn inherited_rooftop_assassin_targets_only_an_opponents_damaged_creature() {
    let player = Player::P0;
    let mut game = scenario(player);
    let victim = board(&mut game, player.other(), "Hill Giant");
    let untouched = board(&mut game, player.other(), "Hill Giant");
    let shock = hand(&mut game, player, "Shock");
    cast(&mut game, shock, Some(Target::Card(victim)));
    quiet(&mut game);
    let assassin = hand(&mut game, player, "Rooftop Assassin");
    cast(&mut game, assassin, None);
    let target = Action::ChooseTarget {
        target: Target::Card(victim),
    };
    until_action(&mut game, &target);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(untouched),
        },
    );
    act(&mut game, target);
    quiet(&mut game);
    assert_eq!(zone(&game, victim), Zone::Graveyard);
    assert_eq!(zone(&game, untouched), Zone::Battlefield);
    assert!(
        keywords(&game, assassin).contains(
            Keywords::FLASH
                .union(Keywords::FLYING)
                .union(Keywords::LIFELINK)
        )
    );
}
