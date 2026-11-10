use super::*;

#[test]
fn iron_spider_adds_group_counters_and_pays_from_uncrewed_artifacts() {
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(
        &mut game,
        player,
        "Iron Spider, Stark Upgrade",
        Zone::Battlefield,
    );
    let car = put(&mut game, player, "Voyager Glidecar", Zone::Battlefield);
    let bears = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let boots = put(&mut game, player, "Swiftfoot Boots", Zone::Battlefield);
    assert!(
        game.object_view(source)
            .keywords
            .contains(Keywords::VIGILANCE)
    );
    assert!(!game.is_creature(car));
    announce_ability(&mut game, source, 0);
    pay(&mut game);
    settle(&mut game);
    assert!(game.object_view(source).tapped);
    assert_eq!(
        [source, car, bears, boots].map(|id| game.counter_count(id, CounterKind::PlusOne)),
        [1, 1, 0, 0]
    );

    announce_ability(&mut game, source, 1);
    for id in [car, source] {
        let object = game.object_ref(id);
        act(
            &mut game,
            Action::ChooseCreatureCounterCost {
                object,
                counter: CounterKind::PlusOne,
            },
        );
    }
    pay(&mut game);
    assert!(game.state.stack.last().unwrap().targets.is_empty());
    settle(&mut game);
    assert_eq!(game.hand_count(player), 1);
    assert_eq!(game.counter_count(car, CounterKind::PlusOne), 0);
    assert_eq!(game.counter_count(source, CounterKind::PlusOne), 0);
}

#[test]
fn maximum_carnage_requires_an_attack_adds_red_and_deals_five() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let bear = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let saga = put(&mut game, player, "Maximum Carnage", Zone::Hand);
    cast(&mut game, saga, None);
    settle(&mut game);
    assert_eq!(game.counter_count(saga, CounterKind::Lore), 1);
    advance_until(&mut game, |g| g.state.pending == Pending::Attackers);
    reject(&mut game, Action::FinishAttackers);
    act(&mut game, Action::Attack { card: bear });
    act(&mut game, Action::FinishAttackers);
    let turn = game.state.turn;
    advance_until(&mut game, |g| {
        g.state.active == player && g.state.turn > turn && g.state.step == Step::Main1
    });
    settle(&mut game);
    assert_eq!(game.counter_count(saga, CounterKind::Lore), 2);
    assert_eq!(game.state.players[player.index()].mana, [0, 0, 0, 3, 0, 0]);
    let turn = game.state.turn;
    advance_until(&mut game, |g| {
        g.state.active == player && g.state.turn > turn && g.state.step == Step::Main1
    });
    let before = game.state.players[opponent.index()].life;
    settle(&mut game);
    assert_eq!(game.state.players[opponent.index()].life, before - 5);
    assert_eq!(game.object_view(saga).zone, Zone::Graveyard);
}

#[test]
fn mister_negative_offers_exchange_and_draws_the_life_he_lost() {
    let player = Player::P1;
    for accept in [false, true] {
        let mut game = scenario(player);
        let opponent = player.other();
        game.state.players[opponent.index()].life = 16;
        let card = put(&mut game, player, "Mister Negative", Zone::Hand);
        cast(&mut game, card, None);
        pass_pair(&mut game);
        assert!(
            game.object_view(card)
                .keywords
                .contains(Keywords::VIGILANCE.union(Keywords::LIFELINK))
        );
        reject(
            &mut game,
            Action::ChooseTarget {
                target: Target::Player(player),
            },
        );
        choose_target(&mut game, Target::Player(opponent));
        pass_pair(&mut game);
        act(&mut game, Action::ChooseOptional { accept });
        settle(&mut game);
        assert_eq!(
            game.state.players[player.index()].life,
            if accept { 16 } else { 20 }
        );
        assert_eq!(
            game.state.players[opponent.index()].life,
            if accept { 20 } else { 16 }
        );
        assert_eq!(game.hand_count(player), if accept { 4 } else { 0 });
    }
}

#[test]
fn mysterio_creates_illusions_for_nontoken_villains_and_exiles_his_batch() {
    let player = Player::P0;
    let mut game = scenario(player);
    put(
        &mut game,
        player,
        "Inner Demons Gangsters",
        Zone::Battlefield,
    );
    put(
        &mut game,
        player.other(),
        "Inner Demons Gangsters",
        Zone::Battlefield,
    );
    put(&mut game, player, "Doubling Season", Zone::Battlefield);
    let card = put(
        &mut game,
        player,
        "Mysterio, Master of Illusion",
        Zone::Hand,
    );
    cast(&mut game, card, None);
    settle(&mut game);
    let illusions = token_refs(&game, Card(5545), player);
    assert_eq!(illusions.len(), 4);
    for object in &illusions {
        let view = game.object_view(object.card);
        assert_eq!(view.colors, 2);
        assert_eq!(stats(&game, object.card), (3, 3));
        assert!(game.has_subtype(object.card, "Illusion"));
        assert!(game.has_subtype(object.card, "Villain"));
    }

    let unsummon = put(&mut game, player, "Unsummon", Zone::Hand);
    cast(&mut game, unsummon, Some(Target::Card(card)));
    pass_pair(&mut game);
    assert_eq!(game.object_view(card).zone, Zone::Hand);
    assert_eq!(token_refs(&game, Card(5545), player), illusions);
    settle(&mut game);
    assert!(token_refs(&game, Card(5545), player).is_empty());
    assert!(
        illusions
            .iter()
            .all(|object| game.object_view(object.card).zone == Zone::Outside)
    );
}

#[test]
fn oscorp_is_a_tapped_mayhem_land_with_three_mana_colors() {
    let player = Player::P0;
    let mut game = scenario(player);
    let card = put(&mut game, player, "Oscorp Industries", Zone::Hand);
    let milled = put(&mut game, player, "Oscorp Industries", Zone::Graveyard);
    let land = put(&mut game, player, "Island", Zone::Hand);
    discard_with_meower(&mut game, player, card);
    reject(&mut game, Action::PlayLand { card: milled });
    act(&mut game, Action::PlayLand { card });
    settle(&mut game);
    assert!(game.object_view(card).tapped);
    assert_eq!(game.state.players[player.index()].life, 18);
    reject(&mut game, Action::PlayLand { card: land });

    let turn = game.state.turn;
    advance_until(&mut game, |g| {
        g.state.active == player && g.state.turn > turn && g.state.step == Step::Main1
    });
    assert!(!game.object_view(card).tapped);
    for color in [1, 2, 3] {
        let mut branch = game.clone();
        let mut expected = [0; 6];
        expected[color] = 1;
        choose(&mut branch, |action| {
            matches!(action,
                Action::ActivateMana { card: source, production, .. }
                if *source == card && *production == expected
            )
        });
        assert_eq!(branch.state.players[player.index()].mana, expected);
    }
}

#[test]
fn prison_break_mayhem_reanimates_an_owned_creature_with_one_extra_counter() {
    let player = Player::P1;
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Prison Break", Zone::Hand);
    let owned = put(
        &mut game,
        player,
        "Selfless Police Captain",
        Zone::Graveyard,
    );
    let enemy = put(&mut game, player.other(), "Grizzly Bears", Zone::Graveyard);
    discard_with_meower(&mut game, player, spell);
    game.state.players[player.index()].mana = [0, 0, 1, 0, 0, 3];
    act(&mut game, Action::AnnounceMayhem { card: spell });
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(enemy),
        },
    );
    choose_target(&mut game, Target::Card(owned));
    pay(&mut game);
    assert_eq!(
        game.state.stack.last().unwrap().cast.unwrap().method,
        CastMethod::Mayhem
    );
    settle(&mut game);
    assert_eq!(game.object_view(owned).zone, Zone::Battlefield);
    assert_eq!(game.counter_count(owned, CounterKind::PlusOne), 2);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
}

#[test]
fn pumpkin_bombardment_pays_each_hybrid_color_with_discard_or_two_mana() {
    let player = Player::P0;
    for color in [2, 3] {
        for generic in [0, 2] {
            let mut game = scenario(player);
            let spell = put(&mut game, player, "Pumpkin Bombardment", Zone::Hand);
            let discard = put(&mut game, player, "Forest", Zone::Hand);
            let target = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
            let mut colored = [0; 6];
            colored[color] = 1;
            game.state.players[player.index()].mana = colored;
            game.state.players[player.index()].mana[5] = generic;
            act(&mut game, Action::AnnounceCast { card: spell });
            let route = game
                .observe(player)
                .cost_choices
                .into_iter()
                .find(|view| view.cost.colored == colored && view.cost.generic == generic)
                .expect("matching hybrid and additional mana option");
            act(&mut game, Action::ChooseSpellCost { cost: route.index });
            choose_target(&mut game, Target::Card(target));
            if generic == 0 {
                let object = game.object_ref(discard);
                act(&mut game, Action::ChooseAdditionalCost { object });
            }
            pay(&mut game);
            settle(&mut game);
            assert_eq!(game.object_view(target).zone, Zone::Graveyard);
            assert_eq!(
                game.object_view(discard).zone,
                if generic == 0 {
                    Zone::Graveyard
                } else {
                    Zone::Hand
                }
            );
            assert_eq!(game.state.players[player.index()].mana, [0; 6]);
        }
    }
}

#[test]
fn supportive_parents_fund_a_paid_spell_by_tapping_two_untapped_creatures() {
    let player = Player::P0;
    let mut game = scenario(player);
    game.state.players[player.index()].mana = [0; 6];
    let parents = put(&mut game, player, "Supportive Parents", Zone::Battlefield);
    game.state.objects[parents as usize].summoning_sick = true;
    let witness = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let land = put(&mut game, player, "Forest", Zone::Battlefield);
    let spell = put(&mut game, player, "Healer's Hawk", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: spell });
    let white = [1, 0, 0, 0, 0, 0];
    choose(&mut game, |action| {
        matches!(action,
            Action::AnnounceMana { card, production, .. }
            if *card == parents && [white, [0; 6]].contains(production)
        )
    });
    let object = game.object_ref(land);
    reject(&mut game, Action::ChooseAdditionalCost { object });
    for card in [parents, witness] {
        let object = game.object_ref(card);
        act(&mut game, Action::ChooseAdditionalCost { object });
    }
    pay(&mut game);
    if matches!(game.state.pending, Pending::EffectMana { .. }) {
        act(&mut game, Action::ChooseMana { production: white });
    }
    assert!(game.object_view(parents).tapped && game.object_view(witness).tapped);
    assert!(game.state.stack.is_empty());
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(spell).zone, Zone::Battlefield);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
}

#[test]
fn swarm_flashes_in_from_the_graveyard_after_its_exact_discard() {
    let player = Player::P0;
    let mut game = scenario(player);
    let card = put(&mut game, player, "Swarm, Being of Bees", Zone::Hand);
    let unrelated = put(&mut game, player, "Swarm, Being of Bees", Zone::Graveyard);
    let shock = put(&mut game, player.other(), "Shock", Zone::Hand);
    act(&mut game, Action::Pass);
    cast(&mut game, shock, Some(Target::Player(player)));
    act(&mut game, Action::Pass);
    discard_with_meower(&mut game, player, card);
    assert_eq!(game.state.stack.len(), 1);
    game.state.players[player.index()].mana = [0, 0, 1, 0, 0, 0];
    reject(&mut game, Action::AnnounceMayhem { card: unrelated });
    act(&mut game, Action::AnnounceMayhem { card });
    pay(&mut game);
    pass_pair(&mut game);
    let view = game.object_view(card);
    assert_eq!(view.zone, Zone::Battlefield);
    assert!(
        view.keywords
            .contains(Keywords::FLASH.union(Keywords::FLYING))
    );
    assert_eq!(game.state.stack.len(), 1);
    settle(&mut game);
}

#[test]
fn ultimate_green_goblin_has_hybrid_mayhem_and_creates_treasure_after_discard() {
    let player = Player::P1;
    for color in [2, 3] {
        let mut game = scenario(player);
        let goblin = put(&mut game, player, "Ultimate Green Goblin", Zone::Hand);
        let drawn = *game.state.players[player.index()].library.last().unwrap();
        discard_with_meower(&mut game, player, goblin);
        let mut required = [0; 6];
        required[color] = 1;
        game.state.players[player.index()].mana = required;
        game.state.players[player.index()].mana[5] = 2;
        act(&mut game, Action::AnnounceMayhem { card: goblin });
        let route = game
            .observe(player)
            .cost_choices
            .into_iter()
            .find(|view| view.cost.colored == required && view.cost.generic == 2)
            .unwrap();
        act(&mut game, Action::ChooseSpellCost { cost: route.index });
        pay(&mut game);
        settle(&mut game);
        assert_eq!(stats(&game, goblin), (5, 4));
        assert_eq!(game.state.players[player.index()].mana, [0; 6]);
        act(&mut game, Action::PlayLand { card: drawn });
        assert_eq!(game.hand_count(player), 0);
        let extra = (color == 2).then(|| put(&mut game, player, "Forest", Zone::Hand));
        let turn = game.state.turn;
        advance_until(&mut game, |g| {
            g.state.active == player && g.state.turn > turn && g.state.step == Step::Upkeep
        });
        assert!(token_refs(&game, Card(5302), player).is_empty());
        if let Some(card) = extra {
            pass_pair(&mut game);
            act(&mut game, Action::Discard { card });
            assert_eq!(game.object_view(card).zone, Zone::Graveyard);
        }
        settle(&mut game);
        assert_eq!(token_refs(&game, Card(5302), player).len(), 1);
    }
}
