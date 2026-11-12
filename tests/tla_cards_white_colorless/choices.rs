use super::*;

#[test]
fn aangs_journey_kicker_adds_a_shrine_to_the_same_search() {
    for kicked in [false, true] {
        let player = if kicked { Player::P1 } else { Player::P0 };
        let mut game = scenario(player);
        let journey = put(&mut game, player, "Aang's Journey", Zone::Hand);
        let land = put(&mut game, player, "Plains", Zone::Library);
        let shrine = spawn(
            &mut game,
            player,
            Card::from_name("Southern Air Temple").unwrap(),
            Zone::Library,
        );
        game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 4];
        act(&mut game, Action::AnnounceCast { card: journey });
        act(&mut game, Action::ChooseKicker { pay: kicked });
        pay(&mut game);
        assert_eq!(
            game.state.players[player.index()].mana[5],
            if kicked { 0 } else { 2 }
        );
        pass_pair(&mut game);
        assert!(game.observe(player.other()).actions.is_empty());
        if kicked {
            act(&mut game, Action::ChooseSearch { card: shrine });
        } else {
            reject(&mut game, Action::ChooseSearch { card: shrine });
        }
        act(&mut game, Action::ChooseSearch { card: land });
        if game.legal_actions().contains(&Action::FinishSearch) {
            act(&mut game, Action::FinishSearch);
        }
        assert_eq!(life(&game, player), 22);
        assert_eq!(game.object_view(land).zone, Zone::Hand);
        assert_eq!(
            game.object_view(shrine).zone,
            if kicked { Zone::Hand } else { Zone::Library }
        );
        let public = game.observe(player.other());
        assert!(public.objects.iter().any(|object| object.id == land));
        assert_eq!(
            public.objects.iter().any(|object| object.id == shrine),
            kicked
        );
    }
}

#[test]
fn destined_confrontation_finishes_both_choices_before_sacrificing() {
    let player = Player::P1;
    let other = player.other();
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Destined Confrontation", Zone::Hand);
    let keep = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let sacrifice = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let enemy = put(&mut game, other, "Bear Cub", Zone::Battlefield);
    let equipment = put(&mut game, other, "Short Bow", Zone::Battlefield);
    cast(&mut game, spell, None);
    pass_pair(&mut game);
    assert_eq!(game.actor(), Some(player));
    choose_effect(&mut game, keep);
    let excess = Action::ChooseEffectObject {
        object: game.object_ref(sacrifice),
    };
    reject(&mut game, excess);
    act(&mut game, Action::FinishEffectChoice);
    assert_eq!(game.actor(), Some(other));
    assert_eq!(game.object_view(sacrifice).zone, Zone::Battlefield);
    act(&mut game, Action::FinishEffectChoice);
    for card in [sacrifice, enemy] {
        assert_eq!(game.object_view(card).zone, Zone::Graveyard);
    }
    for card in [keep, equipment] {
        assert_eq!(game.object_view(card).zone, Zone::Battlefield);
    }
}

#[test]
fn planetarium_scries_and_offers_the_actual_new_top_only_until_one_cast() {
    let player = Player::P0;
    let mut game = scenario(player);
    let planetarium = put(&mut game, player, "Planetarium of Wan Shi Tong", Zone::Hand);
    let offered = put(&mut game, player, "Hill Giant", Zone::Library);
    let first = put(&mut game, player, "Forest", Zone::Library);
    cast(&mut game, planetarium, None);
    drain(&mut game);

    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 1];
    activate(&mut game, planetarium, 0);
    pay(&mut game);
    assert!(game.object_view(planetarium).tapped);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    pass_pair(&mut game);
    for (card, destination) in [
        (first, LibraryDestination::Bottom),
        (offered, LibraryDestination::Top),
    ] {
        act(&mut game, Action::ArrangeLibrary { card, destination });
    }
    pass_pair(&mut game);
    assert!(game.observe(player.other()).resolution_free_casts.is_none());
    let object = game.object_ref(offered);
    act(
        &mut game,
        Action::CastExiledDuringResolution {
            object,
            face: CastFace::Normal,
        },
    );
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(&mut game);
    }
    act(&mut game, Action::FinishResolutionCasts);
    drain(&mut game);
    assert_eq!(game.object_view(offered).zone, Zone::Battlefield);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);

    let surveil = put(&mut game, player, "Undercity Sewers", Zone::Hand);
    act(&mut game, Action::PlayLand { card: surveil });
    pass_pair(&mut game);
    let top = game.top_library_card(player).unwrap();
    act(
        &mut game,
        Action::ArrangeLibrary {
            card: top,
            destination: LibraryDestination::Top,
        },
    );
    drain(&mut game);
    assert!(game.observe(player).resolution_free_casts.is_none());
}

#[test]
fn yangchen_exiles_draws_transforms_and_airbends_on_the_second_spell() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let saga = put(
        &mut game,
        player,
        "The Legend of Yangchen // Avatar Yangchen",
        Zone::Hand,
    );
    let first = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
    let second = put(
        &mut game,
        opponent,
        "Water Tribe Captain",
        Zone::Battlefield,
    );
    let low = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let own = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    cast(&mut game, saga, None);
    pass_pair(&mut game);
    let front = game.object_ref(saga);
    assert_eq!(game.counter_count(saga, CounterKind::Lore), 1);
    pass_pair(&mut game);
    assert_eq!(game.actor(), Some(player));
    let illegal = Action::ChooseEffectObject {
        object: game.object_ref(low),
    };
    reject(&mut game, illegal);
    choose_effect(&mut game, first);
    assert_eq!(game.actor(), Some(opponent));
    choose_effect(&mut game, second);
    for card in [first, second] {
        assert_eq!(game.object_view(card).zone, Zone::Exile);
    }

    next_own_main(&mut game);
    let draws = [draw_count(&game, player), draw_count(&game, opponent)];
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(player),
        },
    );
    target(&mut game, Target::Player(opponent));
    pass_pair(&mut game);
    let mut decline = game.clone();
    optional(&mut decline, false);
    assert_eq!(draw_count(&decline, player), draws[0]);
    assert_eq!(draw_count(&decline, opponent), draws[1]);
    optional(&mut game, true);
    for (p, before) in [(player, draws[0]), (opponent, draws[1])] {
        assert_eq!(draw_count(&game, p), before + 3);
    }

    next_own_main(&mut game);
    drain(&mut game);
    let avatar = game.object_view(saga);
    assert_ne!(avatar.reference, front);
    assert_eq!(avatar.card, Card::from_catalogue_id(5703));
    assert_eq!((avatar.power, avatar.toughness), (4, 5));
    assert!(avatar.keywords.contains(Keywords::FLYING));

    game.state.players[player.index()].mana = [0, 0, 0, 2, 0, 0];
    let first_spell = put(&mut game, player, "Shock", Zone::Hand);
    let second_spell = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, first_spell, Some(Target::Player(opponent)));
    drain(&mut game);
    assert_eq!(game.object_view(own).zone, Zone::Battlefield);
    cast(&mut game, second_spell, Some(Target::Player(opponent)));
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(saga),
        },
    );
    target(&mut game, Target::Card(own));
    finish_targets(&mut game);
    drain(&mut game);
    assert_eq!(game.object_view(own).zone, Zone::Exile);
}

#[test]
fn iroh_makes_food_and_only_the_accepted_donation_creates_a_counted_ally() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let iroh = put(&mut game, player, "Iroh, Tea Master", Zone::Hand);
    let old = put(&mut game, player, "Short Bow", Zone::Battlefield);
    game.record_control(game.object_ref(old), opponent, None);
    cast(&mut game, iroh, None);
    drain(&mut game);
    let food = tokens_of(&game, player, "Food")[0].id;
    game.begin_step(Step::BeginCombat);
    game.prepare_triggers();
    target(&mut game, Target::Player(opponent));
    target(&mut game, Target::Card(food));
    pass_pair(&mut game);
    let mut decline = game.clone();
    optional(&mut decline, false);
    drain(&mut decline);
    assert_eq!(decline.object_view(food).controller, player);
    assert!(tokens_of(&decline, player, "Ally").is_empty());

    optional(&mut game, true);
    assert_eq!(game.object_view(food).controller, opponent);
    assert!(tokens_of(&game, player, "Ally").is_empty());
    drain(&mut game);
    let allies = tokens_of(&game, player, "Ally");
    assert_eq!(allies.len(), 1);
    assert_eq!((allies[0].power, allies[0].toughness), (3, 3));
}

#[test]
fn zuko_loses_life_for_each_mode_then_starts_fresh_under_opposing_control() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let zuko = put(&mut game, player, "Zuko, Conflicted", Zone::Hand);
    cast(&mut game, zuko, None);
    drain(&mut game);
    let exact = game.object_ref(zuko);
    for mode in 0..4 {
        game.state.turn += 1;
        game.begin_step(Step::Main1);
        game.prepare_triggers();
        if mode != 0 {
            reject(&mut game, Action::ChooseMode { mode: 0 });
        }
        act(&mut game, Action::ChooseMode { mode });
        let red = game.state.players[player.index()].mana[3];
        drain(&mut game);
        assert_eq!(life(&game, player), 20 - 2 * (i128::from(mode) + 1));
        match mode {
            0 => assert_eq!(draw_count(&game, player), 1),
            1 => assert_eq!(game.counter_count(zuko, CounterKind::PlusOne), 1),
            2 => assert_eq!(game.state.players[player.index()].mana[3], red + 1),
            3 => {
                assert_eq!(game.object_view(zuko).controller, opponent);
                assert_eq!(game.counter_count(zuko, CounterKind::PlusOne), 0);
                assert_ne!(game.object_ref(zuko), exact);
            }
            _ => unreachable!(),
        }
    }
    game.state.active = opponent;
    game.state.priority = opponent;
    game.begin_step(Step::Main1);
    game.prepare_triggers();
    act(&mut game, Action::ChooseMode { mode: 0 });
    drain(&mut game);
    assert_eq!(life(&game, opponent), 18);
}

fn next_own_main(game: &mut Game) {
    for _ in 0..2 {
        game.state.step = Step::End;
        game.give_priority();
        pass_pair(game);
        assert_eq!(game.state.step, Step::Upkeep);
    }
    pass_pair(game);
    pass_pair(game);
    assert_eq!(game.state.step, Step::Main1);
}
