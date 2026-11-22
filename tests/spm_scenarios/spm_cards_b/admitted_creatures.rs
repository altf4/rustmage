use super::*;

fn activated(card: CardId) -> Action {
    Action::AnnounceActivate {
        card,
        ability: 0,
        counter_payment: Counters::default(),
    }
}

#[test]
fn gangsters_discard_at_sorcery_speed_and_gain_temporary_power_and_menace() {
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(
        &mut game,
        player,
        "Inner Demons Gangsters",
        Zone::Battlefield,
    );
    let discard = put(&mut game, player, "Forest", Zone::Hand);
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, shock, Some(Target::Player(player.other())));
    reject(&mut game, activated(source));
    settle(&mut game);
    let mana = game.state.players[player.index()].mana;
    act(&mut game, activated(source));
    let object = game.object_ref(discard);
    act(&mut game, Action::ChooseAdditionalCost { object });
    pay(&mut game);
    assert_eq!(game.object_view(discard).zone, Zone::Graveyard);
    settle(&mut game);
    assert_eq!(stats(&game, source), (4, 4));
    assert!(game.object_view(source).keywords.contains(Keywords::MENACE));
    assert_eq!(game.state.players[player.index()].mana, mana);
    advance_until(&mut game, |g| {
        g.state.active != player && g.state.step == Step::Main1
    });
    assert_eq!(stats(&game, source), (3, 4));
    assert!(!game.object_view(source).keywords.contains(Keywords::MENACE));
}

#[test]
fn kapow_puts_a_counter_on_only_the_ally_before_the_fight() {
    let player = Player::P0;
    for invalidated in [false, true] {
        let mut game = scenario(player);
        let ally = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        let enemy = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
        let spell = put(&mut game, player, "Kapow!", Zone::Hand);
        announce(&mut game, spell, &[Target::Card(ally), Target::Card(enemy)]);
        if invalidated {
            let bounce = put(&mut game, player, "Unsummon", Zone::Hand);
            cast(&mut game, bounce, Some(Target::Card(enemy)));
        }
        settle(&mut game);
        if invalidated {
            assert_eq!(game.object_view(enemy).zone, Zone::Hand);
            assert_eq!(game.object_view(ally).zone, Zone::Battlefield);
            assert_eq!(game.counter_count(ally, CounterKind::PlusOne), 1);
            assert_eq!(game.object_view(ally).damage, 0);
        } else {
            assert_eq!(game.object_view(enemy).zone, Zone::Graveyard);
            assert_eq!(game.object_view(ally).zone, Zone::Graveyard);
        }
    }
}

#[test]
fn kraven_s_cats_pay_once_per_turn_and_their_boost_expires() {
    let player = Player::P0;
    let mut game = scenario(player);
    let cats = put(&mut game, player, "Kraven's Cats", Zone::Battlefield);
    act(&mut game, activated(cats));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(stats(&game, cats), (4, 4));
    reject(&mut game, activated(cats));
    advance_until(&mut game, |g| {
        g.state.active != player && g.state.step == Step::Main1
    });
    assert_eq!(stats(&game, cats), (2, 2));
    act(&mut game, Action::Pass);
    game.state.players[player.index()].mana = [0, 0, 0, 0, 1, 2];
    act(&mut game, activated(cats));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(stats(&game, cats), (4, 4));
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
}

#[test]
fn kraven_s_power_tracks_the_highest_controlled_permanent_mana_value() {
    let player = Player::P0;
    let mut game = scenario(player);
    let kraven = put(&mut game, player, "Kraven, Proud Predator", Zone::Hand);
    let opposing = put(
        &mut game,
        player.other(),
        "Ancient Adamantoise",
        Zone::Battlefield,
    );
    cast(&mut game, kraven, None);
    settle(&mut game);
    assert_eq!(stats(&game, kraven), (3, 4));
    assert!(
        game.object_view(kraven)
            .keywords
            .contains(Keywords::VIGILANCE)
    );
    let archive = put(&mut game, player, "Hedron Archive", Zone::Hand);
    cast(&mut game, archive, None);
    settle(&mut game);
    assert_eq!(stats(&game, kraven), (4, 4));
    assert_eq!(game.object_view(opposing).zone, Zone::Battlefield);
    let disenchant = put(&mut game, player, "Disenchant", Zone::Hand);
    cast(&mut game, disenchant, Some(Target::Card(archive)));
    settle(&mut game);
    assert_eq!(stats(&game, kraven), (3, 4));
}

#[test]
fn lurk_triggers_at_four_including_x_and_morlun_uses_the_same_x_after_bounce() {
    let player = Player::P0;
    let mut game = scenario(player);
    let lurk = put(&mut game, player, "Lurking Lizards", Zone::Battlefield);
    let draw = put(&mut game, player, "Divination", Zone::Hand);
    assert!(game.object_view(lurk).keywords.contains(Keywords::TRAMPLE));
    cast(&mut game, draw, None);
    settle(&mut game);
    assert_eq!(game.counter_count(lurk, CounterKind::PlusOne), 0);

    let morlun = put(&mut game, player, "Morlun, Devourer of Spiders", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: morlun });
    act(&mut game, Action::ChooseX { value: 2 });
    pay(&mut game);
    pass_pair(&mut game);
    assert_eq!(game.counter_count(lurk, CounterKind::PlusOne), 1);
    pass_pair(&mut game);
    assert!(
        game.object_view(morlun)
            .keywords
            .contains(Keywords::LIFELINK)
    );
    assert_eq!(game.counter_count(morlun, CounterKind::PlusOne), 2);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(player),
        },
    );
    choose_target(&mut game, Target::Player(player.other()));
    let bounce = put(&mut game, player, "Unsummon", Zone::Hand);
    cast(&mut game, bounce, Some(Target::Card(morlun)));
    settle(&mut game);
    assert_eq!(game.object_view(morlun).zone, Zone::Hand);
    assert_eq!(game.state.players[player.index()].life, 22);
    assert_eq!(game.state.players[player.other().index()].life, 18);
}

#[test]
fn mary_jane_accepts_both_hybrid_colors_and_draws_once_for_each_turn_s_spiders() {
    let player = Player::P0;
    for color in [0, 4] {
        let mut game = scenario(player);
        let mary = put(&mut game, player, "Mary Jane Watson", Zone::Hand);
        let mut colored = [0; 6];
        colored[color] = 1;
        game.state.players[player.index()].mana = colored;
        game.state.players[player.index()].mana[5] = 1;
        act(&mut game, Action::AnnounceCast { card: mary });
        let choice = game
            .observe(player)
            .cost_choices
            .into_iter()
            .find(|view| view.cost.colored == colored)
            .unwrap();
        act(&mut game, Action::ChooseSpellCost { cost: choice.index });
        pay(&mut game);
        settle(&mut game);
        assert_eq!(game.state.players[player.index()].mana, [0; 6]);

        game.state.players[player.index()].mana[4] = 2;
        for draws in [1, 1] {
            let spider = put(&mut game, player, "Radioactive Spider", Zone::Hand);
            cast(&mut game, spider, None);
            settle(&mut game);
            assert_eq!(
                game.state.players[player.index()].cards_drawn_this_turn,
                draws
            );
        }
        let turn = game.state.turn;
        advance_until(&mut game, |g| {
            g.state.active == player && g.state.turn > turn && g.state.step == Step::Main1
        });
        let draws = game.state.players[player.index()].cards_drawn_this_turn;
        game.state.players[player.index()].mana[4] = 1;
        let spider = put(&mut game, player, "Radioactive Spider", Zone::Hand);
        cast(&mut game, spider, None);
        settle(&mut game);
        assert_eq!(
            game.state.players[player.index()].cards_drawn_this_turn,
            draws + 1
        );
    }
}

#[test]
fn masked_meower_needs_a_discard_then_sacrifices_and_draws() {
    let player = Player::P0;
    let mut game = scenario(player);
    let cat = put(&mut game, player, "Masked Meower", Zone::Battlefield);
    assert!(game.object_view(cat).keywords.contains(Keywords::HASTE));
    reject(&mut game, activated(cat));
    let discard = put(&mut game, player, "Island", Zone::Hand);
    act(&mut game, activated(cat));
    let object = game.object_ref(discard);
    act(&mut game, Action::ChooseAdditionalCost { object });
    pay(&mut game);
    assert_eq!(game.object_view(cat).zone, Zone::Graveyard);
    assert_eq!(game.object_view(discard).zone, Zone::Graveyard);
    settle(&mut game);
    assert_eq!(game.hand_count(player), 1);
}

#[test]
fn mobster_exiles_before_a_controlled_creature_connives_even_when_exile_is_declined() {
    let player = Player::P0;
    for exile in [false, true] {
        let mut game = scenario(player);
        let ally = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        let enemy = put(
            &mut game,
            player.other(),
            "Grizzly Bears",
            Zone::Battlefield,
        );
        let grave = put(&mut game, player.other(), "Shock", Zone::Graveyard);
        let discard = put(&mut game, player, "Shock", Zone::Hand);
        let mobster = put(&mut game, player, "Mechanical Mobster", Zone::Hand);
        cast(&mut game, mobster, None);
        pass_pair(&mut game);
        if exile {
            choose_target(&mut game, Target::Card(grave));
        } else {
            act(&mut game, Action::FinishTargets);
        }
        reject(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        choose_target(&mut game, Target::Card(ally));
        pass_pair(&mut game);
        assert_eq!(
            game.object_view(grave).zone,
            if exile { Zone::Exile } else { Zone::Graveyard }
        );
        act(&mut game, Action::Discard { card: discard });
        settle(&mut game);
        assert_eq!(game.counter_count(ally, CounterKind::PlusOne), 1);
        assert_eq!(game.counter_count(mobster, CounterKind::PlusOne), 0);
    }
}

#[test]
fn merciless_enforcers_deal_damage_and_gain_life_using_the_departed_source() {
    let player = Player::P0;
    let mut game = scenario(player);
    let creature = put(&mut game, player, "Merciless Enforcers", Zone::Battlefield);
    assert!(
        game.object_view(creature)
            .keywords
            .contains(Keywords::LIFELINK)
    );
    act(&mut game, activated(creature));
    pay(&mut game);
    let bounce = put(&mut game, player, "Unsummon", Zone::Hand);
    cast(&mut game, bounce, Some(Target::Card(creature)));
    settle(&mut game);
    assert_eq!(game.object_view(creature).zone, Zone::Hand);
    assert_eq!(game.state.players[player.index()].life, 21);
    assert_eq!(game.state.players[player.other().index()].life, 19);
}

#[test]
fn mob_lookout_connives_with_either_hybrid_color_and_only_nonland_adds_a_counter() {
    let player = Player::P1;
    for (color, discard_name, expected) in [(1, "Island", 0), (2, "Shock", 1)] {
        let mut game = scenario(player);
        let lookout = put(&mut game, player, "Mob Lookout", Zone::Hand);
        let discard = put(&mut game, player, discard_name, Zone::Hand);
        let enemy = put(
            &mut game,
            player.other(),
            "Grizzly Bears",
            Zone::Battlefield,
        );
        let mut colored = [0; 6];
        colored[color] = 1;
        game.state.players[player.index()].mana = colored;
        game.state.players[player.index()].mana[5] = 1;
        act(&mut game, Action::AnnounceCast { card: lookout });
        let choice = game
            .observe(player)
            .cost_choices
            .into_iter()
            .find(|view| view.cost.colored == colored)
            .unwrap();
        act(&mut game, Action::ChooseSpellCost { cost: choice.index });
        pay(&mut game);
        pass_pair(&mut game);
        reject(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        choose_target(&mut game, Target::Card(lookout));
        pass_pair(&mut game);
        act(&mut game, Action::Discard { card: discard });
        settle(&mut game);
        assert_eq!(game.counter_count(lookout, CounterKind::PlusOne), expected);
        assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    }
}

#[test]
fn phantasm_mills_on_attack_and_wrestler_accepts_one_blocker() {
    let player = Player::P0;
    let mut game = scenario(player);
    let phantasm = put(&mut game, player, "Mysterio's Phantasm", Zone::Battlefield);
    let wrestler = put(&mut game, player, "Professional Wrestler", Zone::Hand);
    let blockers = [(); 2].map(|_| {
        put(
            &mut game,
            player.other(),
            "Grizzly Bears",
            Zone::Battlefield,
        )
    });
    cast(&mut game, wrestler, None);
    settle(&mut game);
    assert_eq!(token_refs(&game, Card(5302), player).len(), 1);
    game.state.objects[wrestler as usize].summoning_sick = false;
    let top = *game.state.players[player.index()].library.last().unwrap();
    declare(&mut game, &[phantasm, wrestler]);
    settle(&mut game);
    assert_eq!(game.object_view(top).zone, Zone::Graveyard);
    assert!(!game.object_view(phantasm).tapped);
    assert!(
        game.object_view(phantasm)
            .keywords
            .contains(Keywords::FLYING.union(Keywords::VIGILANCE))
    );
    advance_until(&mut game, |g| g.state.pending == Pending::Blockers);
    act(
        &mut game,
        Action::Block {
            blocker: blockers[0],
            attacker: wrestler,
        },
    );
    reject(
        &mut game,
        Action::Block {
            blocker: blockers[1],
            attacker: wrestler,
        },
    );
    act(&mut game, Action::FinishBlockers);
}

#[test]
fn news_helicopter_makes_a_green_white_human_citizen() {
    let player = Player::P0;
    let mut game = scenario(player);
    let helicopter = put(&mut game, player, "News Helicopter", Zone::Hand);
    cast(&mut game, helicopter, None);
    settle(&mut game);
    assert!(
        game.object_view(helicopter)
            .keywords
            .contains(Keywords::FLYING)
    );
    let tokens = token_refs(&game, Card(5314), player);
    assert_eq!(tokens.len(), 1);
    let token = game.object_view(tokens[0].card);
    assert_eq!(token.colors, 1 | 16);
    assert!(token.has_subtype("Human") && token.has_subtype("Citizen"));
    assert_eq!(stats(&game, token.id), (1, 1));
}

#[test]
fn research_team_uses_seven_mana_for_two_draws_on_the_opponent_s_turn() {
    let player = Player::P1;
    let mut game = scenario(player.other());
    let team = put(&mut game, player, "Oscorp Research Team", Zone::Battlefield);
    act(&mut game, Action::Pass);
    game.state.players[player.index()].mana = [0, 1, 0, 0, 0, 5];
    reject(&mut game, activated(team));
    game.state.players[player.index()].mana[5] = 6;
    act(&mut game, activated(team));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.hand_count(player), 2);
    assert!(!game.object_view(team).tapped);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
}

#[test]
fn prowler_ignores_itself_and_nonvillains_then_connives_when_another_villain_enters() {
    let player = Player::P0;
    let mut game = scenario(player);
    let prowler = put(&mut game, player, "Prowler, Clawed Thief", Zone::Hand);
    for name in ["Prowler, Clawed Thief", "Grizzly Bears"] {
        let card = if name == "Prowler, Clawed Thief" {
            prowler
        } else {
            put(&mut game, player, name, Zone::Hand)
        };
        cast(&mut game, card, None);
        settle(&mut game);
        assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 0);
    }
    assert!(
        game.object_view(prowler)
            .keywords
            .contains(Keywords::MENACE)
    );
    let villain = put(&mut game, player, "Inner Demons Gangsters", Zone::Hand);
    let discard = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, villain, None);
    pass_pair(&mut game);
    pass_pair(&mut game);
    act(&mut game, Action::Discard { card: discard });
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
    assert_eq!(game.counter_count(prowler, CounterKind::PlusOne), 1);
}
