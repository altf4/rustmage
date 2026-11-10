use super::*;

#[test]
fn spectacular_spider_man_casts_with_flash_and_pays_for_flying_and_team_protection() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(opponent);
    let ally = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let enemy = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
    let hero = put(&mut game, player, "Spectacular Spider-Man", Zone::Hand);
    act(&mut game, Action::Pass);
    cast(&mut game, hero, None);
    settle(&mut game);
    assert_eq!(game.object_view(hero).zone, Zone::Battlefield);
    let bounce = put(&mut game, opponent, "Unsummon", Zone::Hand);
    cast(&mut game, bounce, Some(Target::Card(ally)));
    act(&mut game, Action::Pass);

    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 2];
    activate(&mut game, hero, 0);
    pay(&mut game);
    pass_pair(&mut game);
    assert!(game.object_view(hero).keywords.contains(Keywords::FLYING));
    assert_eq!(game.state.players[player.index()].mana[5], 1);
    act(&mut game, Action::Pass);
    activate(&mut game, hero, 1);
    pay(&mut game);
    assert_eq!(game.object_view(hero).zone, Zone::Graveyard);
    settle(&mut game);
    assert_eq!(game.object_view(ally).zone, Zone::Battlefield);
    assert!(
        game.object_view(ally)
            .keywords
            .contains(Keywords::HEXPROOF.union(Keywords::INDESTRUCTIBLE))
    );
    assert!(
        !game
            .object_view(enemy)
            .keywords
            .contains(Keywords::HEXPROOF)
    );

    let wrath = put(&mut game, opponent, "Day of Judgment", Zone::Hand);
    cast(&mut game, wrath, None);
    settle(&mut game);
    assert_eq!(game.object_view(ally).zone, Zone::Battlefield);
    assert_eq!(game.object_view(enemy).zone, Zone::Graveyard);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    cleanup(&mut game);
    assert!(!game.object_view(ally).keywords.contains(Keywords::HEXPROOF));
    assert!(
        !game
            .object_view(ally)
            .keywords
            .contains(Keywords::INDESTRUCTIBLE)
    );
}

#[test]
fn spider_bot_can_decline_without_shuffling_or_put_a_revealed_basic_on_top() {
    let player = Player::P0;
    let mut game = scenario(player);
    let bot = put(&mut game, player, "Spider-Bot", Zone::Hand);
    let basic = put(&mut game, player, "Island", Zone::Library);
    let top = put(&mut game, player, "Opt", Zone::Library);
    cast(&mut game, bot, None);
    pass_pair(&mut game);
    pass_pair(&mut game);
    assert!(game.object_view(bot).keywords.contains(Keywords::REACH));

    let mut declined = game.clone();
    let before = declined.state.players[player.index()].library.clone();
    act(&mut declined, Action::ChooseOptional { accept: false });
    assert_eq!(declined.state.players[player.index()].library, before);
    assert_eq!(declined.top_library_card(player), Some(top));

    act(&mut game, Action::ChooseOptional { accept: true });
    assert!(game.observe(player).search_choice.is_some());
    assert!(game.observe(player.other()).search_choice.is_none());
    reject(&mut game, Action::ChooseSearch { card: top });
    act(&mut game, Action::ChooseSearch { card: basic });
    if game.legal_actions().contains(&Action::FinishSearch) {
        act(&mut game, Action::FinishSearch);
    }
    assert_eq!(game.top_library_card(player), Some(basic));
    assert_eq!(game.object_view(basic).zone, Zone::Library);
}

#[test]
fn spider_girl_flies_only_on_her_controllers_turn_and_creates_a_citizen_on_bounce() {
    let player = Player::P0;
    let mut game = scenario(player);
    let girl = put(&mut game, player, "Spider-Girl, Legacy Hero", Zone::Hand);
    cast(&mut game, girl, None);
    settle(&mut game);
    assert!(game.object_view(girl).keywords.contains(Keywords::FLYING));
    cleanup(&mut game);
    assert_eq!(game.state.active, player.other());
    assert!(!game.object_view(girl).keywords.contains(Keywords::FLYING));
    act(&mut game, Action::Pass);
    let bounce = put(&mut game, player, "Unsummon", Zone::Hand);
    game.state.players[player.index()].mana[1] = 1;
    cast(&mut game, bounce, Some(Target::Card(girl)));
    settle(&mut game);
    assert_eq!(game.object_view(girl).zone, Zone::Hand);
    let citizen = game
        .state
        .objects
        .iter()
        .enumerate()
        .find_map(|(id, object)| {
            (object.zone == Zone::Battlefield && object.is_token()).then_some(id as CardId)
        })
        .unwrap();
    let token = game.object_view(citizen);
    assert_eq!(token.controller, player);
    assert!(token.has_subtype("Human") && token.has_subtype("Citizen"));
    assert_eq!(token.colors, 1 | 16);
    assert_eq!((token.power, token.toughness), (1, 1));
}

#[test]
fn spider_ham_buffs_the_union_once_and_the_food_pays_its_printed_cost() {
    let player = Player::P0;
    let mut game = scenario(player);
    let ham = put(&mut game, player, "Spider-Ham, Peter Porker", Zone::Hand);
    let bear = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let changeling = put(&mut game, player, "Feisty Spikeling", Zone::Battlefield);
    let giant = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let enemy = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let prior = stats(&game, changeling);
    cast(&mut game, ham, None);
    settle(&mut game);
    assert_eq!(stats(&game, ham), (2, 2));
    assert_eq!(stats(&game, bear), (3, 3));
    assert_eq!(stats(&game, changeling), (prior.0 + 1, prior.1 + 1));
    assert_eq!(stats(&game, giant), (3, 3));
    assert_eq!(stats(&game, enemy), (2, 2));

    let food = game
        .state
        .objects
        .iter()
        .enumerate()
        .find_map(|(id, object)| {
            (object.zone == Zone::Battlefield && object.is_token()).then_some(id as CardId)
        })
        .unwrap();
    assert!(game.object_view(food).has_subtype("Food"));
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 2];
    activate(&mut game, food, 0);
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, 23);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    assert_ne!(game.object_view(food).zone, Zone::Battlefield);
}

#[test]
fn spider_man_no_more_keeps_card_types_and_counters_and_removes_only_older_abilities() {
    let player = Player::P0;
    let mut game = scenario(player);
    let creature = put(&mut game, player, "Oasis Gardener", Zone::Battlefield);
    let boots = put(&mut game, player, "Swiftfoot Boots", Zone::Battlefield);
    game.add_counter(creature, CounterKind::PlusOne, 1);
    activate(&mut game, boots, 0);
    target(&mut game, Target::Card(creature));
    pay(&mut game);
    settle(&mut game);
    let original = game.object_view(creature);
    assert!(original.keywords.contains(Keywords::HASTE));
    assert_eq!((original.power, original.toughness), (3, 3));

    let aura = put(&mut game, player, "Spider-Man No More", Zone::Hand);
    cast(&mut game, aura, Some(Target::Card(creature)));
    settle(&mut game);
    let citizen = game.object_view(creature);
    assert_eq!(
        (citizen.kinds, citizen.colors),
        (original.kinds, original.colors)
    );
    assert_eq!((citizen.power, citizen.toughness), (2, 2));
    assert!(citizen.has_subtype("Citizen"));
    assert!(!citizen.has_subtype("Scarecrow"));
    assert!(citizen.keywords.contains(Keywords::DEFENDER));
    assert!(!citizen.keywords.contains(Keywords::HASTE));
    assert!(!game.legal_actions().iter().any(|action| matches!(
        action, Action::ActivateMana { card, .. } if *card == creature
    )));

    let beetle = put(
        &mut game,
        player,
        "Beetle, Legacy Criminal",
        Zone::Graveyard,
    );
    activate(&mut game, beetle, 0);
    target(&mut game, Target::Card(creature));
    pay(&mut game);
    settle(&mut game);
    assert!(
        game.object_view(creature)
            .keywords
            .contains(Keywords::DEFENDER.union(Keywords::FLYING))
    );
    let removal = put(&mut game, player, "Make Your Move", Zone::Hand);
    cast(&mut game, removal, Some(Target::Card(aura)));
    settle(&mut game);
    let restored = game.object_view(creature);
    assert_eq!((restored.power, restored.toughness), (4, 4));
    assert!(restored.has_subtype("Scarecrow"));
    assert!(!restored.has_subtype("Citizen"));
    assert!(restored.keywords.contains(Keywords::HASTE));
    assert!(!restored.keywords.contains(Keywords::DEFENDER));
}

#[test]
fn spider_mobile_crews_for_two_and_counts_current_controlled_spiders_when_attacking_or_blocking() {
    let player = Player::P0;
    for attacking in [true, false] {
        let active = if attacking { player } else { player.other() };
        let mut game = scenario(active);
        let mobile = put(&mut game, player, "Spider-Mobile", Zone::Battlefield);
        let pilot = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        put(&mut game, player, "Spider Manifestation", Zone::Battlefield);
        let enemy = put(
            &mut game,
            player.other(),
            "Spider Manifestation",
            Zone::Battlefield,
        );
        assert!(!game.is_creature(mobile));
        if !attacking {
            act(&mut game, Action::Pass);
        }
        activate(&mut game, mobile, 0);
        if matches!(game.state.pending, Pending::PayCosts { .. }) {
            pay(&mut game);
        }
        let object = game.object_ref(pilot);
        act(&mut game, Action::ChooseCrewer { object });
        act(&mut game, Action::FinishCrew);
        settle(&mut game);
        assert!(game.is_creature(mobile));
        assert!(game.object_view(pilot).tapped);
        assert!(
            game.object_view(mobile)
                .keywords
                .contains(Keywords::TRAMPLE)
        );
        assert_eq!(stats(&game, mobile), (3, 3));
        if attacking {
            declare(&mut game, &[mobile]);
        } else {
            declare(&mut game, &[enemy]);
            pass_pair(&mut game);
            act(
                &mut game,
                Action::Block {
                    blocker: mobile,
                    attacker: enemy,
                },
            );
            act(&mut game, Action::FinishBlockers);
        }
        settle(&mut game);
        assert_eq!(stats(&game, mobile), (4, 4));
        cleanup(&mut game);
        assert!(!game.is_creature(mobile));
    }
}

#[test]
fn spider_punk_grants_riot_and_makes_countering_and_prevention_fail() {
    let player = Player::P0;
    let mut game = scenario(player);
    let punk = put(&mut game, player, "Spider-Punk", Zone::Hand);
    cast(&mut game, punk, None);
    pass_pair(&mut game);
    act(&mut game, Action::ChooseEntryRiot { counter: true });
    settle(&mut game);
    assert_eq!(game.counter_count(punk, CounterKind::PlusOne), 1);

    let spider = put(&mut game, player, "Spider Manifestation", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: spider });
    choose(&mut game, |action| {
        matches!(action, Action::ChooseSpellCost { .. })
    });
    pay(&mut game);
    pass_pair(&mut game);
    act(&mut game, Action::ChooseEntryRiot { counter: false });
    settle(&mut game);
    assert!(game.object_view(spider).keywords.contains(Keywords::HASTE));
    let bear = put(&mut game, player, "Grizzly Bears", Zone::Hand);
    cast(&mut game, bear, None);
    settle(&mut game);
    assert_eq!(game.counter_count(bear, CounterKind::PlusOne), 0);
    assert!(!game.object_view(bear).keywords.contains(Keywords::HASTE));

    let victim = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    game.add_counter(victim, CounterKind::Shield, 1);
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, shock, Some(Target::Card(victim)));
    let cancel = put(&mut game, player, "Cancel", Zone::Hand);
    cast(&mut game, cancel, Some(Target::Spell(shock)));
    settle(&mut game);
    assert_eq!(game.object_view(victim).damage, 2);
    assert_eq!(game.counter_count(victim, CounterKind::Shield), 0);

    let bounce = put(&mut game, player, "Unsummon", Zone::Hand);
    cast(&mut game, bounce, Some(Target::Card(punk)));
    settle(&mut game);
    assert!(game.object_view(spider).keywords.contains(Keywords::HASTE));
    let second = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, second, Some(Target::Player(player.other())));
    let cancel = put(&mut game, player, "Cancel", Zone::Hand);
    cast(&mut game, cancel, Some(Target::Spell(second)));
    settle(&mut game);
    assert_eq!(game.state.players[player.other().index()].life, 20);
}

#[test]
fn spider_slayer_destroys_a_damaged_spider_without_targeting_and_pays_to_make_tapped_robots() {
    let player = Player::P0;
    for spider in [false, true] {
        let mut game = scenario(player);
        let slayer = put(&mut game, player, "Spider-Slayer, Hatred Honed", Zone::Hand);
        cast(&mut game, slayer, None);
        settle(&mut game);
        assert_eq!(game.object_view(slayer).kinds, 17);
        game.state.objects[slayer as usize].summoning_sick = false;
        let blocker = put(
            &mut game,
            player.other(),
            if spider {
                "Spider-Rex, Daring Dino"
            } else {
                "Hill Giant"
            },
            Zone::Battlefield,
        );
        declare(&mut game, &[slayer]);
        pass_pair(&mut game);
        act(
            &mut game,
            Action::Block {
                blocker,
                attacker: slayer,
            },
        );
        act(&mut game, Action::FinishBlockers);
        pass_pair(&mut game);
        settle(&mut game);
        assert_eq!(game.object_view(slayer).zone, Zone::Graveyard);
        assert_eq!(
            game.object_view(blocker).zone,
            if spider {
                Zone::Graveyard
            } else {
                Zone::Battlefield
            }
        );
        game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 6];
        activate(&mut game, slayer, 0);
        pay(&mut game);
        assert_eq!(game.object_view(slayer).zone, Zone::Exile);
        settle(&mut game);
        let robots: Vec<_> = game
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(_, object)| object.zone == Zone::Battlefield && object.is_token())
            .map(|(id, _)| game.object_view(id as CardId))
            .collect();
        assert_eq!(robots.len(), 2);
        for robot in robots {
            assert_eq!((robot.kinds, robot.colors), (17, 0));
            assert!(robot.has_subtype("Robot"));
            assert_eq!((robot.power, robot.toughness), (1, 1));
            assert!(robot.keywords.contains(Keywords::FLYING) && robot.tapped);
        }
        assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    }
}
