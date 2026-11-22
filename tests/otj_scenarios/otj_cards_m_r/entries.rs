use super::*;

#[test]
fn mercenary_sources_use_the_same_token_and_tycoon_can_sacrifice_an_artifact_token() {
    let player = Player::P0;
    let mut game = scenario(player);
    let nezumi = board(&mut game, player, "Nezumi Linkbreaker");
    let shock = hand(&mut game, player, "Shock");
    cast(&mut game, shock, Some(Target::Card(nezumi)));
    quiet(&mut game);
    assert_eq!(tokens(&game, player, MERCENARY).len(), 1);

    let surprise = hand(&mut game, player, "Mourner's Surprise");
    let mut decline = game.clone();
    announce(&mut decline, surprise, &[]);
    quiet(&mut decline);
    assert_eq!(zone(&decline, nezumi), Zone::Graveyard);
    assert_eq!(tokens(&decline, player, MERCENARY).len(), 2);
    announce(&mut game, surprise, &[Target::Card(nezumi)]);
    quiet(&mut game);
    assert_eq!(zone(&game, nezumi), Zone::Hand);

    let pair = hand(&mut game, player, "Prickly Pair");
    cast(&mut game, pair, None);
    quiet(&mut game);
    assert_eq!(tokens(&game, player, MERCENARY).len(), 3);

    let tycoon = hand(&mut game, player, "Prosperity Tycoon");
    cast(&mut game, tycoon, None);
    quiet(&mut game);
    let mercenaries = tokens(&game, player, MERCENARY);
    assert_eq!(mercenaries.len(), 4);
    let token = mercenaries[0];
    assert_eq!(stats(&game, token), (1, 1));
    assert_eq!(game.object_view(token).colors, 8);
    assert!(game.object_view(token).summoning_sick);

    let treasure = spawn(
        &mut game,
        player,
        Card::from_catalogue_id(TREASURE),
        Zone::Battlefield,
    );
    announce_activation(&mut game, tycoon, 0);
    let non_token = game.object_ref(pair);
    reject(
        &mut game,
        Action::ChooseAdditionalCost { object: non_token },
    );
    finish_additional_cost(&mut game, &[treasure]);
    assert!(tokens(&game, player, TREASURE).is_empty());
    assert!(!game.object_view(tycoon).tapped);
    quiet(&mut game);
    assert!(game.object_view(tycoon).tapped);
    assert!(keywords(&game, tycoon).contains(Keywords::INDESTRUCTIBLE));

    let turn = game.state.turn;
    reach(&mut game, |g| {
        g.state.turn > turn && g.state.active == player && g.state.step == Step::Main1
    });
    activate(&mut game, token, 0, &[Target::Card(tycoon)]);
    quiet(&mut game);
    assert!(game.object_view(token).tapped);
    assert_eq!(stats(&game, tycoon), (5, 2));
}

#[test]
fn nurturing_pixie_returns_the_exact_controlled_non_faerie_or_can_decline() {
    let player = Player::P1;
    let mut game = scenario(player);
    let borrowed = board(&mut game, player.other(), "Grizzly Bears");
    let object = game.object_ref(borrowed);
    game.change_controller(object, player);
    let land = board(&mut game, player, "Plains");
    let pixie = hand(&mut game, player, "Nurturing Pixie");
    cast(&mut game, pixie, None);
    let target = Action::ChooseTarget {
        target: Target::Card(borrowed),
    };
    until_action(&mut game, &target);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(land),
        },
    );
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(pixie),
        },
    );

    let mut decline = game.clone();
    act(&mut decline, Action::FinishTargets);
    quiet(&mut decline);
    assert_eq!(counters(&decline, pixie), 0);
    assert_eq!(zone(&decline, borrowed), Zone::Battlefield);
    act(&mut game, target);
    if has(&game, &Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    quiet(&mut game);
    assert_eq!(zone(&game, borrowed), Zone::Hand);
    assert_eq!(game.object_view(borrowed).owner, player.other());
    assert_eq!(stats(&game, pixie), (2, 2));
    assert!(keywords(&game, pixie).contains(Keywords::FLYING));
}

#[test]
fn mine_raider_checks_for_another_outlaw_at_entry_and_again_on_resolution() {
    let player = Player::P0;
    for (has_outlaw, survives) in [(false, false), (true, false), (true, true)] {
        let mut game = scenario(player);
        let outlaw = has_outlaw.then(|| board(&mut game, player, "Reckless Lackey"));
        let raider = hand(&mut game, player, "Mine Raider");
        cast(&mut game, raider, None);
        if let Some(outlaw) = outlaw.filter(|_| !survives) {
            reach(&mut game, |g| {
                g.state.pending == Pending::Priority && zone(g, raider) == Zone::Battlefield
            });
            let shock = hand(&mut game, player, "Shock");
            cast(&mut game, shock, Some(Target::Card(outlaw)));
        }
        quiet(&mut game);
        assert!(keywords(&game, raider).contains(Keywords::TRAMPLE));
        assert_eq!(tokens(&game, player, TREASURE).len(), usize::from(survives));
    }
}

#[test]
fn trailblazer_checks_entry_power_while_brawler_reads_live_power_on_resolution() {
    let player = Player::P0;
    let mut game = scenario(player);
    let trailblazer = hand(&mut game, player, "Outcaster Trailblazer");
    cast(&mut game, trailblazer, None);
    let produce = Action::ChooseMana {
        production: [0, 0, 0, 0, 1, 0],
    };
    until_action(&mut game, &produce);
    let mana = game.state.players[player.index()].mana[4];
    act(&mut game, produce);
    quiet(&mut game);
    assert_eq!(game.state.players[player.index()].mana[4], mana + 1);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 0);

    let brawler = hand(&mut game, player, "Railway Brawler");
    cast(&mut game, brawler, None);
    quiet(&mut game);
    assert!(keywords(&game, brawler).contains(Keywords::REACH.union(Keywords::TRAMPLE)));
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);

    let bear = hand(&mut game, player, "Grizzly Bears");
    cast(&mut game, bear, None);
    reach(&mut game, |g| {
        g.state.pending == Pending::Priority && zone(g, bear) == Zone::Battlefield
    });
    let growth = hand(&mut game, player, "Giant Growth");
    cast(&mut game, growth, Some(Target::Card(bear)));
    quiet(&mut game);
    assert_eq!(counters(&game, bear), 5);
    assert_eq!(stats(&game, bear), (10, 10));
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
}

#[test]
fn outlaw_stitcher_counts_each_committed_spell_beyond_the_first_on_its_new_token() {
    let player = Player::P0;
    for prior_spells in [0, 2] {
        let mut game = scenario(player);
        let existing = spawn(
            &mut game,
            player,
            Card::from_catalogue_id(ZOMBIE_ROGUE),
            Zone::Battlefield,
        );
        for _ in 0..prior_spells {
            let shock = hand(&mut game, player, "Shock");
            cast(&mut game, shock, Some(Target::Player(player.other())));
            quiet(&mut game);
        }
        let stitcher = hand(&mut game, player, "Outlaw Stitcher");
        cast(&mut game, stitcher, None);
        quiet(&mut game);
        let batch = tokens(&game, player, ZOMBIE_ROGUE);
        assert_eq!(batch.len(), 2);
        assert_eq!(counters(&game, existing), 0);
        assert_eq!(counters(&game, batch[1]), prior_spells * 2);
        assert_eq!(game.object_view(batch[1]).colors, 6);
    }
}

#[test]
fn raucous_entertainer_places_counters_on_currently_controlled_entrants() {
    let player = Player::P1;
    let mut game = scenario(player);
    let old = board(&mut game, player, "Grizzly Bears");
    let entertainer = board(&mut game, player, "Raucous Entertainer");
    game.state.entered_this_turn = None;
    let entrant = hand(&mut game, player, "Bear Cub");
    cast(&mut game, entrant, None);
    quiet(&mut game);
    let borrowed = hand(&mut game, player.other(), "Hill Giant");
    game.move_object(borrowed, Zone::Battlefield);
    let object = game.object_ref(borrowed);
    game.change_controller(object, player);
    let foreign = board(&mut game, player.other(), "Grizzly Bears");
    activate(&mut game, entertainer, 0, &[]);
    quiet(&mut game);
    assert!(game.object_view(entertainer).tapped);
    for card in [entrant, borrowed] {
        assert_eq!(counters(&game, card), 1);
    }
    for card in [old, entertainer, foreign] {
        assert_eq!(counters(&game, card), 0);
    }
}

#[test]
fn rakdos_joins_up_uses_entry_counters_and_legendary_death_power() {
    let player = Player::P0;
    let mut game = scenario(player);
    let legend = graveyard(&mut game, player, "Magda, the Hoardmaster");
    let enchantment = hand(&mut game, player, "Rakdos Joins Up");
    cast(&mut game, enchantment, None);
    let bring_back = Action::ChooseTarget {
        target: Target::Card(legend),
    };
    until_action(&mut game, &bring_back);
    act(&mut game, bring_back);
    quiet(&mut game);
    assert_eq!(stats(&game, legend), (4, 4));
    let murder = hand(&mut game, player, "Murder");
    cast(&mut game, murder, Some(Target::Card(legend)));
    let damage = Action::ChooseTarget {
        target: Target::Player(player.other()),
    };
    until_action(&mut game, &damage);
    act(&mut game, damage);
    quiet(&mut game);
    assert_eq!(life(&game, player.other()), 16);
}

#[test]
fn rakish_crew_drains_for_its_exact_outlaw_death() {
    let player = Player::P1;
    let mut game = scenario(player);
    let crew = hand(&mut game, player, "Rakish Crew");
    cast(&mut game, crew, None);
    quiet(&mut game);
    let mercenary = tokens(&game, player, MERCENARY)[0];
    let normal = board(&mut game, player, "Grizzly Bears");
    for (victim, expected) in [(normal, 20), (mercenary, 19)] {
        let shock = hand(&mut game, player, "Shock");
        cast(&mut game, shock, Some(Target::Card(victim)));
        quiet(&mut game);
        assert_eq!(life(&game, player.other()), expected);
    }
    assert_eq!(life(&game, player), 21);
}

#[test]
fn rakdos_the_muscle_locks_the_cost_and_exiles_exact_cards_for_flexible_plays() {
    let player = Player::P0;
    let mut game = scenario(player);
    let rakdos = board(&mut game, player, "Rakdos, the Muscle");
    let paid = board(&mut game, player, "Grizzly Bears");
    let other = board(&mut game, player, "Bear Cub");
    let land = top(&mut game, player.other(), "Forest");
    let shock = top(&mut game, player.other(), "Shock");
    let retained = game.state.players[player.other().index()].library[5];
    announce_activation(&mut game, rakdos, 0);
    finish_additional_cost(&mut game, &[paid]);
    assert_eq!(zone(&game, paid), Zone::Graveyard);
    let target = Action::ChooseTarget {
        target: Target::Player(player.other()),
    };
    until_action(&mut game, &target);
    act(&mut game, target);
    quiet(&mut game);
    assert!(game.object_view(rakdos).tapped);
    assert!(
        keywords(&game, rakdos).contains(
            Keywords::FLYING
                .union(Keywords::TRAMPLE)
                .union(Keywords::INDESTRUCTIBLE)
        )
    );
    assert_eq!(zone(&game, other), Zone::Battlefield);
    assert_eq!(zone(&game, retained), Zone::Library);
    assert_eq!(zone(&game, shock), Zone::Exile);
    assert_eq!(zone(&game, land), Zone::Exile);
    reject(
        &mut game,
        Action::AnnounceActivate {
            card: rakdos,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 1];
    act(&mut game, Action::AnnounceExile { card: shock });
    bind_targets(&mut game, &[Target::Player(player.other())]);
    pay(&mut game);
    quiet(&mut game);
    assert_eq!(life(&game, player.other()), 18);
    act(&mut game, Action::PlayLand { card: land });
    assert_eq!(zone(&game, land), Zone::Battlefield);
    assert_eq!(game.object_view(land).controller, player);
}

#[test]
fn ruthless_lawbringer_sacrifices_before_the_independent_destruction_trigger() {
    let player = Player::P0;
    let mut game = scenario(player);
    let sacrifice = board(&mut game, player, "Grizzly Bears");
    let land = board(&mut game, player, "Plains");
    let victim = board(&mut game, player.other(), "Grizzly Bears");
    let lawbringer = hand(&mut game, player, "Ruthless Lawbringer");
    cast(&mut game, lawbringer, None);
    let exact = game.object_ref(sacrifice);
    let cost = Action::Sacrifice { object: exact };
    until_action(&mut game, &cost);
    let self_target = game.object_ref(lawbringer);
    reject(
        &mut game,
        Action::Sacrifice {
            object: self_target,
        },
    );
    let mut declined = game.clone();
    act(&mut declined, Action::DeclineEffectPayment);
    quiet(&mut declined);
    assert_eq!(zone(&declined, sacrifice), Zone::Battlefield);
    assert_eq!(zone(&declined, victim), Zone::Battlefield);
    act(&mut game, cost);
    assert_eq!(zone(&game, sacrifice), Zone::Graveyard);
    let target = Action::ChooseTarget {
        target: Target::Card(victim),
    };
    until_action(&mut game, &target);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(land),
        },
    );
    act(&mut game, target);
    assert_eq!(zone(&game, victim), Zone::Battlefield);
    quiet(&mut game);
    assert_eq!(zone(&game, victim), Zone::Graveyard);
}
