use super::*;

fn sacrifice(game: &mut Game, card: CardId) {
    let object = game.object_ref(card);
    act(game, Action::Sacrifice { object });
    if game.legal_actions().contains(&Action::FinishEffectChoice) {
        act(game, Action::FinishEffectChoice);
    }
}

fn kickless_lesson(game: &mut Game, lesson: CardId, victim: CardId) {
    act(game, Action::AnnounceCast { card: lesson });
    if game
        .legal_actions()
        .contains(&Action::ChooseKicker { pay: false })
    {
        act(game, Action::ChooseKicker { pay: false });
    }
    act(game, target(victim));
    if game
        .legal_actions()
        .contains(&Action::ChooseKicker { pay: false })
    {
        act(game, Action::ChooseKicker { pay: false });
    }
    pay(game);
}

#[test]
fn dragonfly_swarm_has_live_graveyard_power_ward_and_an_intervening_lesson() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let swarm = put(&mut game, player, "Dragonfly Swarm", Zone::Hand);
    let lesson = put(&mut game, player, "Firebending Lesson", Zone::Graveyard);
    put(&mut game, player, "Opt", Zone::Graveyard);
    put(&mut game, player, "Forest", Zone::Graveyard);
    put(&mut game, player, "Bear Cub", Zone::Graveyard);
    put(&mut game, opponent, "Divination", Zone::Graveyard);
    assert_eq!(stats(&game, swarm), (2, 3));
    cast(&mut game, swarm, None);
    drain(&mut game);
    assert!(game.keywords(swarm).contains(Keywords::FLYING));
    let removal = put(&mut game, opponent, "Murder", Zone::Hand);
    act(&mut game, Action::Pass);
    cast(&mut game, removal, Some(Target::Card(swarm)));
    pass_pair(&mut game);
    act(&mut game, Action::DeclineResolutionMana);
    drain(&mut game);
    assert_eq!(game.object_view(swarm).zone, Zone::Battlefield);
    assert_eq!(game.object_view(removal).zone, Zone::Graveyard);
    game.move_object(swarm, Zone::Graveyard);
    game.prepare_triggers();
    let mut without_lesson = game.clone();
    without_lesson.move_object(lesson, Zone::Exile);
    drain(&mut without_lesson);
    assert_eq!(draws(&without_lesson, player), 0);
    drain(&mut game);
    assert_eq!(draws(&game, player), 1);
}

#[test]
fn guru_pathik_reveals_qualified_cards_and_counters_only_another_own_creature() {
    let player = Player::P0;
    let mut game = scenario(player);
    let guru = put(&mut game, player, "Guru Pathik", Zone::Hand);
    let friend = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let opposing = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let lesson = put(&mut game, player, "Accumulate Wisdom", Zone::Library);
    let saga = put(&mut game, player, "The Apprentice's Folly", Zone::Library);
    let opt = put(&mut game, player, "Opt", Zone::Library);
    cast(&mut game, guru, None);
    pass_pair(&mut game);
    pass_pair(&mut game);
    assert!(game.observe(player).library_selection.is_some());
    assert!(game.observe(player.other()).library_selection.is_none());
    reject(&mut game, Action::SelectLibrary { card: opt });
    let mut saga_selection = game.clone();
    act(&mut saga_selection, Action::SelectLibrary { card: saga });
    if saga_selection
        .legal_actions()
        .contains(&Action::FinishLibrarySelection)
    {
        act(&mut saga_selection, Action::FinishLibrarySelection);
    }
    assert_eq!(saga_selection.object_view(saga).zone, Zone::Hand);
    act(&mut game, Action::SelectLibrary { card: lesson });
    if game
        .legal_actions()
        .contains(&Action::FinishLibrarySelection)
    {
        act(&mut game, Action::FinishLibrarySelection);
    }
    assert!(
        game.observe(player.other())
            .known_hand
            .contains(&game.object_ref(lesson))
    );
    cast(&mut game, lesson, None);
    reject(&mut game, target(guru));
    reject(&mut game, target(opposing));
    act(&mut game, target(friend));
    pass_pair(&mut game);
    assert_eq!(game.counter_count(friend, CounterKind::PlusOne), 1);
}

#[test]
fn hei_bai_pays_only_for_other_objects_and_transfers_all_counters_on_departure() {
    let player = Player::P0;
    let mut game = scenario(player);
    let hei = put(&mut game, player, "Hei Bai, Spirit of Balance", Zone::Hand);
    let artifact = put(&mut game, player, "Short Bow", Zone::Battlefield);
    let recipient = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let offered_creature = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    cast(&mut game, hei, None);
    pass_pair(&mut game);
    pass_pair(&mut game);
    let invalid = Action::Sacrifice {
        object: game.object_ref(hei),
    };
    reject(&mut game, invalid);
    let mut decline = game.clone();
    act(&mut decline, Action::DeclineEffectPayment);
    drain(&mut decline);
    assert_eq!(stats(&decline, hei), (3, 3));
    sacrifice(&mut game, artifact);
    drain(&mut game);
    assert_eq!(stats(&game, hei), (5, 5));
    assert_eq!(game.object_view(artifact).zone, Zone::Graveyard);
    game.state.objects[hei as usize].summoning_sick = false;
    attack(&mut game, &[hei]);
    pass_pair(&mut game);
    sacrifice(&mut game, offered_creature);
    drain(&mut game);
    assert_eq!(game.counter_count(hei, CounterKind::PlusOne), 4);
    game.add_counter(hei, CounterKind::Stun, 2);
    game.move_object(hei, Zone::Hand);
    game.prepare_triggers();
    act(&mut game, target(recipient));
    drain(&mut game);
    assert_eq!(game.counter_count(recipient, CounterKind::PlusOne), 4);
    assert_eq!(game.counter_count(recipient, CounterKind::Stun), 2);
}

#[test]
fn hermitic_herbalist_can_split_lesson_only_mana_and_make_unrestricted_mana() {
    let player = Player::P0;
    let mut game = scenario(player);
    let herbalist = put(&mut game, player, "Hermitic Herbalist", Zone::Battlefield);
    let lesson = put(&mut game, player, "Firebending Lesson", Zone::Hand);
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    let victim = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    game.state.players[player.index()].mana = [0; 6];
    let produce = |ability, production| Action::ActivateMana {
        card: herbalist,
        ability,
        payment: [0; 6],
        counter_payment: Counters::default(),
        production,
    };
    let mut unrestricted = game.clone();
    act(&mut unrestricted, produce(0, [0, 0, 0, 1, 0, 0]));
    cast(&mut unrestricted, shock, Some(Target::Card(victim)));
    drain(&mut unrestricted);
    assert_eq!(unrestricted.object_view(victim).damage, 2);

    assert!(
        game.legal_actions()
            .contains(&produce(1, [0, 0, 0, 2, 0, 0]))
    );
    act(&mut game, produce(1, [0, 1, 0, 1, 0, 0]));
    assert_eq!(game.state.players[player.index()].mana, [0, 1, 0, 1, 0, 0]);
    reject(&mut game, Action::AnnounceCast { card: shock });
    kickless_lesson(&mut game, lesson, victim);
    drain(&mut game);
    assert_eq!(game.object_view(victim).damage, 2);
    assert_eq!(game.state.players[player.index()].mana, [0, 1, 0, 0, 0, 0]);
}

#[test]
fn sandbenders_exile_their_exact_death_card_before_targeting_at_last_known_power() {
    let player = Player::P0;
    let mut game = scenario(player);
    let sandbenders = put(
        &mut game,
        player,
        "Sandbender Scavengers",
        Zone::Battlefield,
    );
    let first = put(&mut game, player, "Forest", Zone::Battlefield);
    let second = put(&mut game, player, "Short Bow", Zone::Battlefield);
    let cheap = put(&mut game, player, "Bear Cub", Zone::Graveyard);
    let costly = put(&mut game, player, "Hill Giant", Zone::Graveyard);
    let opponent = put(&mut game, player.other(), "Bear Cub", Zone::Graveyard);
    game.sacrifice_batch(&[(player, first), (player, second)]);
    drain(&mut game);
    assert_eq!(stats(&game, sandbenders), (3, 3));
    game.move_object(sandbenders, Zone::Graveyard);
    game.prepare_triggers();
    pass_pair(&mut game);
    let mut declined = game.clone();
    act(&mut declined, Action::ChooseOptional { accept: false });
    drain(&mut declined);
    assert_eq!(declined.object_view(sandbenders).zone, Zone::Graveyard);
    act(&mut game, Action::ChooseOptional { accept: true });
    assert_eq!(game.object_view(sandbenders).zone, Zone::Exile);
    reject(&mut game, target(costly));
    reject(&mut game, target(opponent));
    act(&mut game, target(cheap));
    drain(&mut game);
    assert_eq!(game.object_view(cheap).zone, Zone::Battlefield);
}

#[test]
fn hardheaded_toph_discards_for_recovery_and_uses_the_committed_lesson_subtype() {
    let player = Player::P0;
    let mut game = scenario(player);
    let toph = put(&mut game, player, "Toph, Hardheaded Teacher", Zone::Hand);
    let land = put(&mut game, player, "Forest", Zone::Battlefield);
    let opt = put(&mut game, player, "Opt", Zone::Graveyard);
    let discard = put(&mut game, player, "Bear Cub", Zone::Hand);
    let lesson = put(&mut game, player, "Firebending Lesson", Zone::Hand);
    let victim = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    cast(&mut game, toph, None);
    pass_pair(&mut game);
    act(&mut game, target(opt));
    pass_pair(&mut game);
    let mut declined = game.clone();
    act(&mut declined, Action::DeclineEffectPayment);
    drain(&mut declined);
    assert_eq!(declined.object_view(opt).zone, Zone::Graveyard);
    act(&mut game, Action::Discard { card: discard });
    drain(&mut game);
    assert_eq!(game.object_view(discard).zone, Zone::Graveyard);
    assert_eq!(game.object_view(opt).zone, Zone::Hand);

    cast(&mut game, opt, None);
    act(&mut game, target(land));
    pass_pair(&mut game);
    assert_eq!(game.counter_count(land, CounterKind::PlusOne), 1);
    pass_pair(&mut game);
    choose(&mut game, |action| {
        matches!(
            action,
            Action::ArrangeLibrary {
                destination: LibraryDestination::Top,
                ..
            }
        )
    });
    drain(&mut game);
    kickless_lesson(&mut game, lesson, victim);
    act(&mut game, target(land));
    drain(&mut game);
    assert_eq!(game.counter_count(land, CounterKind::PlusOne), 3);
    assert_eq!(game.earthbend_views().len(), 2);
}
