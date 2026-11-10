use super::*;

#[test]
fn rent_taps_a_creature_and_a_treasure_or_sacrifices_without_a_partial_payment() {
    let player = Player::P0;
    let mut game = scenario(player);
    let rent = put(&mut game, player, "Rent Is Due", Zone::Hand);
    cast(&mut game, rent, None);
    settle(&mut game);
    let crook = put(&mut game, player, "Common Crook", Zone::Battlefield);
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, shock, Some(Target::Card(crook)));
    settle(&mut game);
    let treasure = game
        .state
        .objects
        .iter()
        .enumerate()
        .find_map(|(id, object)| {
            (object.zone == Zone::Battlefield && object.is_token()).then_some(id as CardId)
        })
        .unwrap();
    assert!(game.object_view(treasure).has_subtype("Treasure"));
    let creature = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let artifact = put(&mut game, player, "Swiftfoot Boots", Zone::Battlefield);
    let enemy = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    game.state.objects[creature as usize].summoning_sick = true;

    let mut insufficient = game.clone();
    insufficient.state.objects[treasure as usize].tapped = true;
    start_step(&mut insufficient, Step::End);
    settle(&mut insufficient);
    assert_eq!(insufficient.object_view(rent).zone, Zone::Graveyard);
    assert!(!insufficient.object_view(creature).tapped);
    assert_eq!(insufficient.hand_count(player), 0);

    start_step(&mut game, Step::End);
    pass_pair(&mut game);
    for ineligible in [artifact, enemy] {
        let object = game.object_ref(ineligible);
        reject(&mut game, Action::Sacrifice { object });
    }
    let object = game.object_ref(creature);
    act(&mut game, Action::Sacrifice { object });
    assert!(!game.object_view(creature).tapped);
    reject(&mut game, Action::Sacrifice { object });
    let object = game.object_ref(treasure);
    act(&mut game, Action::Sacrifice { object });
    assert!(game.object_view(creature).tapped && game.object_view(treasure).tapped);
    assert_eq!(game.object_view(rent).zone, Zone::Battlefield);
    assert_eq!(game.hand_count(player), 1);
}

#[test]
fn rhino_rampage_queues_its_artifact_target_only_after_excess_fight_damage() {
    let player = Player::P0;
    for (recipient, color, excess) in [("Hill Giant", 3, false), ("Grizzly Bears", 4, true)] {
        let mut game = scenario(player);
        let own = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        let enemy = put(&mut game, player.other(), recipient, Zone::Battlefield);
        let artifact = put(
            &mut game,
            player.other(),
            "Swiftfoot Boots",
            Zone::Battlefield,
        );
        let artifact_creature = put(
            &mut game,
            player.other(),
            "Oasis Gardener",
            Zone::Battlefield,
        );
        let spell = put(&mut game, player, "Rhino's Rampage", Zone::Hand);
        game.state.players[player.index()].mana = [0; 6];
        game.state.players[player.index()].mana[color] = 1;
        act(&mut game, Action::AnnounceCast { card: spell });
        choose(&mut game, |action| {
            matches!(action, Action::ChooseSpellCost { .. })
        });
        reject(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        target(&mut game, Target::Card(own));
        target(&mut game, Target::Card(enemy));
        pay(&mut game);
        assert_eq!(game.state.stack.len(), 1);
        pass_pair(&mut game);
        if excess {
            reject(
                &mut game,
                Action::ChooseTarget {
                    target: Target::Card(artifact_creature),
                },
            );
            target(&mut game, Target::Card(artifact));
        } else {
            assert!(game.state.stack.is_empty());
        }
        settle(&mut game);
        assert_eq!(game.object_view(enemy).zone, Zone::Graveyard);
        assert_eq!(
            game.object_view(artifact).zone,
            if excess {
                Zone::Graveyard
            } else {
                Zone::Battlefield
            }
        );
        assert_eq!(game.object_view(artifact_creature).zone, Zone::Battlefield);
    }
}

#[test]
fn rhino_attacks_untapped_on_the_turn_he_is_cast_and_draws_only_with_a_qualifying_cast() {
    let player = Player::P0;
    for cast_rhino in [false, true] {
        let mut game = scenario(player);
        let rhino = if cast_rhino {
            let rhino = put(&mut game, player, "Rhino, Barreling Brute", Zone::Hand);
            cast(&mut game, rhino, None);
            settle(&mut game);
            rhino
        } else {
            let bear = put(&mut game, player, "Grizzly Bears", Zone::Hand);
            cast(&mut game, bear, None);
            settle(&mut game);
            put(
                &mut game,
                player,
                "Rhino, Barreling Brute",
                Zone::Battlefield,
            )
        };
        declare(&mut game, &[rhino]);
        settle(&mut game);
        assert!(!game.object_view(rhino).tapped);
        assert!(game.object_view(rhino).keywords.contains(Keywords::TRAMPLE));
        assert_eq!(game.hand_count(player), usize::from(cast_rhino));
    }
}

#[test]
fn police_captain_enters_with_a_counter_and_uses_its_departure_plus_one_count() {
    let player = Player::P0;
    let mut game = scenario(player);
    let captain = put(&mut game, player, "Selfless Police Captain", Zone::Hand);
    let bearer = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let enemy = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    cast(&mut game, captain, None);
    settle(&mut game);
    assert_eq!(game.counter_count(captain, CounterKind::PlusOne), 1);
    game.add_counter(captain, CounterKind::PlusOne, 2);
    game.add_counter(captain, CounterKind::Stun, 1);
    let bounce = put(&mut game, player, "Unsummon", Zone::Hand);
    cast(&mut game, bounce, Some(Target::Card(captain)));
    pass_pair(&mut game);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(enemy),
        },
    );
    target(&mut game, Target::Card(bearer));
    settle(&mut game);
    assert_eq!(game.object_view(captain).zone, Zone::Hand);
    assert_eq!(game.counter_count(bearer, CounterKind::PlusOne), 3);
    assert_eq!(game.counter_count(bearer, CounterKind::Stun), 0);
}

#[test]
fn shadow_discards_before_drawing_and_tracks_actual_casts_and_land_plays_from_exile() {
    let player = Player::P0;
    let mut game = scenario(player);
    let shadow = put(&mut game, player, "Shadow of the Goblin", Zone::Hand);
    cast(&mut game, shadow, None);
    settle(&mut game);

    let mut empty = game.clone();
    start_step(&mut empty, Step::Main1);
    settle(&mut empty);
    assert_eq!(empty.hand_count(player), 0);

    let discard = put(&mut game, player, "Island", Zone::Hand);
    start_step(&mut game, Step::Main1);
    pass_pair(&mut game);
    assert_eq!(game.hand_count(player), 1);
    act(&mut game, Action::Discard { card: discard });
    assert_eq!(game.object_view(discard).zone, Zone::Graveyard);
    assert_eq!(game.hand_count(player), 1);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);

    let ordinary = put(&mut game, player, "Grizzly Bears", Zone::Hand);
    cast(&mut game, ordinary, None);
    settle(&mut game);
    assert_eq!(game.state.players[player.other().index()].life, 20);
    let land = put(&mut game, player, "Mountain", Zone::Exile);
    game.grant_exile_play_permission(player, game.object_ref(land), false);
    act(&mut game, Action::PlayLand { card: land });
    settle(&mut game);
    assert_eq!(game.state.players[player.other().index()].life, 19);
    let spell = put(&mut game, player, "Shock", Zone::Exile);
    game.grant_exile_play_permission(player, game.object_ref(spell), false);
    act(&mut game, Action::AnnounceExile { card: spell });
    target(&mut game, Target::Player(player.other()));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.state.players[player.other().index()].life, 16);
}

#[test]
fn shocker_deals_damage_to_the_creature_and_its_controller_at_resolution() {
    let player = Player::P0;
    let mut game = scenario(player);
    let shocker = put(&mut game, player, "Shocker, Unshakable", Zone::Hand);
    let victim = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    cast(&mut game, shocker, None);
    pass_pair(&mut game);
    assert!(
        game.object_view(shocker)
            .keywords
            .contains(Keywords::FIRST_STRIKE)
    );
    target(&mut game, Target::Card(victim));
    game.change_controller(game.object_ref(victim), player);
    settle(&mut game);
    assert_eq!(game.object_view(victim).damage, 2);
    assert_eq!(game.state.players[player.index()].life, 18);
    assert_eq!(game.state.players[player.other().index()].life, 20);
    cleanup(&mut game);
    assert!(
        !game
            .object_view(shocker)
            .keywords
            .contains(Keywords::FIRST_STRIKE)
    );
}

#[test]
fn shriek_chooses_the_blocking_target_after_discard_and_damages_the_dying_creatures_controller() {
    let player = Player::P0;
    for discard in [false, true] {
        let mut game = scenario(player);
        let shriek = put(&mut game, player, "Shriek, Treblemaker", Zone::Hand);
        let land = put(&mut game, player, "Forest", Zone::Hand);
        let blocker = put(
            &mut game,
            player.other(),
            "Grizzly Bears",
            Zone::Battlefield,
        );
        let victim = put(
            &mut game,
            player.other(),
            "Grizzly Bears",
            Zone::Battlefield,
        );
        choose(
            &mut game,
            |action| matches!(action, Action::AnnounceCast { card } if *card == shriek),
        );
        choose(&mut game, |action| {
            matches!(action, Action::ChooseSpellCost { .. })
        });
        pay(&mut game);
        settle(&mut game);
        start_step(&mut game, Step::Main1);
        reject(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(blocker),
            },
        );
        pass_pair(&mut game);
        if discard {
            act(&mut game, Action::Discard { card: land });
            target(&mut game, Target::Card(blocker));
        } else {
            act(&mut game, Action::DeclineEffectPayment);
        }
        settle(&mut game);
        assert_eq!(
            game.object_view(blocker)
                .keywords
                .contains(Keywords::CANT_BLOCK),
            discard
        );
        let shock = put(&mut game, player, "Shock", Zone::Hand);
        cast(&mut game, shock, Some(Target::Card(victim)));
        settle(&mut game);
        assert_eq!(game.object_view(victim).zone, Zone::Graveyard);
        assert_eq!(game.state.players[player.other().index()].life, 19);
        cleanup(&mut game);
        assert!(
            !game
                .object_view(blocker)
                .keywords
                .contains(Keywords::CANT_BLOCK)
        );
    }
}

#[test]
fn silver_sable_counters_another_creature_then_targets_a_modified_ally_on_attack() {
    let player = Player::P0;
    let mut game = scenario(player);
    let sable = put(
        &mut game,
        player,
        "Silver Sable, Mercenary Leader",
        Zone::Hand,
    );
    let modified = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let plain = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    cast(&mut game, sable, None);
    pass_pair(&mut game);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(sable),
        },
    );
    target(&mut game, Target::Card(modified));
    settle(&mut game);
    assert_eq!(game.counter_count(modified, CounterKind::PlusOne), 1);

    game.state.objects[sable as usize].summoning_sick = false;
    declare(&mut game, &[sable]);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(plain),
        },
    );
    target(&mut game, Target::Card(modified));
    let mut no_counter = game.clone();
    no_counter.pay_counters(modified, &Counters::of(CounterKind::PlusOne, 1));
    settle(&mut no_counter);
    assert!(
        !no_counter
            .object_view(modified)
            .keywords
            .contains(Keywords::LIFELINK)
    );
    settle(&mut game);
    assert!(
        game.object_view(modified)
            .keywords
            .contains(Keywords::LIFELINK)
    );
    cleanup(&mut game);
    assert!(
        !game
            .object_view(modified)
            .keywords
            .contains(Keywords::LIFELINK)
    );
}

#[test]
fn spider_man_2099_counts_his_controllers_turns_and_uses_power_after_an_offhand_play() {
    let player = Player::P0;
    for offhand in [false, true] {
        let mut game = scenario(player);
        let hero = put(&mut game, player, "Spider-Man 2099", Zone::Hand);
        for turn in 1..=3 {
            game.state.players[player.index()].turns_started = turn;
            reject(&mut game, Action::AnnounceCast { card: hero });
        }
        game.state.players[player.index()].turns_started = 4;
        let land = put(
            &mut game,
            player,
            "Forest",
            if offhand { Zone::Exile } else { Zone::Hand },
        );
        if offhand {
            game.grant_exile_play_permission(player, game.object_ref(land), false);
        }
        act(&mut game, Action::PlayLand { card: land });
        cast(&mut game, hero, None);
        settle(&mut game);
        assert!(
            game.object_view(hero)
                .keywords
                .contains(Keywords::DOUBLE_STRIKE.union(Keywords::VIGILANCE))
        );
        game.add_counter(hero, CounterKind::PlusOne, 2);
        start_step(&mut game, Step::End);
        if offhand {
            target(&mut game, Target::Player(player.other()));
            let bounce = put(&mut game, player, "Unsummon", Zone::Hand);
            cast(&mut game, bounce, Some(Target::Card(hero)));
        } else {
            assert!(game.state.stack.is_empty());
        }
        settle(&mut game);
        assert_eq!(
            game.state.players[player.other().index()].life,
            if offhand { 16 } else { 20 }
        );
    }
}

#[test]
fn noir_counts_all_live_or_last_known_counters_on_the_lone_attacker() {
    let player = Player::P0;
    let mut game = scenario(player);
    let noir = put(&mut game, player, "Spider-Man Noir", Zone::Hand);
    let attacker = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    game.add_counter(attacker, CounterKind::Stun, 2);
    cast(&mut game, noir, None);
    settle(&mut game);
    assert!(game.object_view(noir).keywords.contains(Keywords::MENACE));
    declare(&mut game, &[attacker]);
    for left in [false, true] {
        let mut branch = game.clone();
        if left {
            let bounce = put(&mut branch, player, "Unsummon", Zone::Hand);
            cast(&mut branch, bounce, Some(Target::Card(attacker)));
            pass_pair(&mut branch);
        }
        pass_pair(&mut branch);
        let look = branch.observe(player).library_choice.unwrap();
        assert!(look.surveil);
        assert_eq!(look.remaining.len(), if left { 2 } else { 3 });
        assert!(branch.observe(player.other()).library_choice.is_none());
        let milled = look.remaining[0].reference.card;
        act(
            &mut branch,
            Action::ArrangeLibrary {
                card: milled,
                destination: LibraryDestination::Graveyard,
            },
        );
        for card in look.remaining.iter().skip(1) {
            act(
                &mut branch,
                Action::ArrangeLibrary {
                    card: card.reference.card,
                    destination: LibraryDestination::Top,
                },
            );
        }
        assert_eq!(branch.object_view(milled).zone, Zone::Graveyard);
        assert_eq!(
            branch.counter_count(attacker, CounterKind::PlusOne),
            u64::from(!left)
        );
    }
}
