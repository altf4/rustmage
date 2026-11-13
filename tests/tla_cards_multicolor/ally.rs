use super::*;

#[test]
fn katara_fearless_doubles_ally_triggers_from_live_sources() {
    let player = Player::P0;
    let mut game = scenario(player);
    let katara = put(&mut game, player, "Katara, the Fearless", Zone::Battlefield);
    let lieutenant = put(
        &mut game,
        player,
        "Earth King's Lieutenant",
        Zone::Battlefield,
    );
    let ally = put(&mut game, player, "Earthen Ally", Zone::Hand);
    let enemy_ally = put(&mut game, player.other(), "Earthen Ally", Zone::Hand);
    cast(&mut game, ally, None);
    drain(&mut game);
    assert_eq!(game.counter_count(lieutenant, CounterKind::PlusOne), 2);
    assert_eq!(game.counter_count(katara, CounterKind::PlusOne), 0);
    game.state.active = player.other();
    game.state.priority = player.other();
    cast(&mut game, enemy_ally, None);
    drain(&mut game);
    assert_eq!(game.counter_count(lieutenant, CounterKind::PlusOne), 2);
}

#[test]
fn bold_sokka_draws_actual_discards_and_checks_artifact_or_lesson_casts() {
    let player = Player::P0;
    let mut game = scenario(player);
    let sokka = put(&mut game, player, "Sokka, Bold Boomeranger", Zone::Hand);
    let first = put(&mut game, player, "Bear Cub", Zone::Hand);
    let second = put(&mut game, player, "Grizzly Bears", Zone::Hand);
    let artifact = put(&mut game, player, "Short Bow", Zone::Hand);
    let spell = put(&mut game, player, "Divination", Zone::Hand);
    let lesson = put(&mut game, player, "Accumulate Wisdom", Zone::Hand);
    cast(&mut game, sokka, None);
    pass_pair(&mut game);
    pass_pair(&mut game);
    let mut decline = game.clone();
    act(&mut decline, Action::DeclineEffectPayment);
    drain(&mut decline);
    assert_eq!(draws(&decline, player), 0);
    for card in [first, second] {
        act(&mut game, Action::Discard { card });
    }
    drain(&mut game);
    assert_eq!(draws(&game, player), 2);
    assert_eq!(game.object_view(first).zone, Zone::Graveyard);
    assert_eq!(game.object_view(second).zone, Zone::Graveyard);
    cast(&mut game, artifact, None);
    drain(&mut game);
    assert_eq!(game.counter_count(sokka, CounterKind::PlusOne), 1);
    cast(&mut game, spell, None);
    drain(&mut game);
    assert_eq!(game.counter_count(sokka, CounterKind::PlusOne), 1);
    cast(&mut game, lesson, None);
    pass_pair(&mut game);
    assert_eq!(game.counter_count(sokka, CounterKind::PlusOne), 2);
}

#[test]
fn lateral_sokka_draws_for_a_companion_and_has_vigilance() {
    let player = Player::P1;
    let mut game = scenario(player);
    let sokka = put(
        &mut game,
        player,
        "Sokka, Lateral Strategist",
        Zone::Battlefield,
    );
    let cub = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let mut alone = game.clone();
    attack(&mut alone, &[sokka]);
    drain(&mut alone);
    assert_eq!(draws(&alone, player), 0);
    assert!(!alone.object_view(sokka).tapped);
    attack(&mut game, &[sokka, cub]);
    drain(&mut game);
    assert_eq!(draws(&game, player), 1);
    assert!(!game.object_view(sokka).tapped);
    assert!(game.object_view(cub).tapped);
}

#[test]
fn tenacious_sokka_grants_menace_and_independent_prowess_and_creates_an_ally() {
    let player = Player::P0;
    let mut game = scenario(player);
    let sokka = put(
        &mut game,
        player,
        "Sokka, Tenacious Tactician",
        Zone::Battlefield,
    );
    let ally = put(
        &mut game,
        player,
        "Earth King's Lieutenant",
        Zone::Battlefield,
    );
    let opposing = put(
        &mut game,
        player.other(),
        "Earth King's Lieutenant",
        Zone::Battlefield,
    );
    let ordinary = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let spell = put(&mut game, player, "Divination", Zone::Hand);
    let creature = put(&mut game, player, "Hill Giant", Zone::Hand);
    assert!(game.keywords(sokka).contains(Keywords::MENACE));
    assert!(game.keywords(ally).contains(Keywords::MENACE));
    assert!(!game.keywords(opposing).contains(Keywords::MENACE));
    assert!(!game.keywords(ordinary).contains(Keywords::MENACE));

    cast(&mut game, creature, None);
    drain(&mut game);
    assert!(tokens(&game, player, 5272).is_empty());
    assert_eq!(stats(&game, sokka), (3, 3));
    cast(&mut game, spell, None);
    drain(&mut game);
    let tokens = tokens(&game, player, 5272);
    assert_eq!(tokens.len(), 1);
    assert_eq!(stats(&game, sokka), (4, 4));
    assert_eq!(stats(&game, ally), (3, 3));
    assert_eq!(stats(&game, tokens[0]), (1, 1));
    assert!(game.keywords(tokens[0]).contains(Keywords::MENACE));
    game.finish_cleanup();
    assert_eq!(stats(&game, ally), (2, 2));
}

#[test]
fn kyoshi_warrior_counts_creatures_in_every_zone_and_makes_an_attacking_ally() {
    let player = Player::P1;
    let mut game = scenario(player);
    let suki = put(&mut game, player, "Suki, Kyoshi Warrior", Zone::Hand);
    let friend = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    assert_eq!(stats(&game, suki), (1, 4));
    cast(&mut game, suki, None);
    drain(&mut game);
    assert_eq!(stats(&game, suki), (2, 4));
    game.state.objects[suki as usize].summoning_sick = false;
    attack(&mut game, &[suki]);
    drain(&mut game);
    let tokens = tokens(&game, player, 5272);
    assert_eq!(tokens.len(), 1);
    let ally = game.object_view(tokens[0]);
    assert!(ally.attacking && ally.tapped);
    assert!(game.has_subtype(ally.id, "Ally"));
    assert_eq!(stats(&game, suki), (3, 4));
    game.move_object(friend, Zone::Exile);
    assert_eq!(stats(&game, suki), (2, 4));
}

#[test]
fn water_tribe_katara_creates_an_ally_and_freezes_the_paid_x_for_current_creatures() {
    let player = Player::P0;
    let mut game = scenario(player);
    let katara = put(&mut game, player, "Katara, Water Tribe's Hope", Zone::Hand);
    let cub = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let opponent = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    game.add_counter(cub, CounterKind::PlusOne, 1);
    cast(&mut game, katara, None);
    drain(&mut game);
    let tokens = tokens(&game, player, 5272);
    assert_eq!(tokens.len(), 1);
    assert!(game.keywords(katara).contains(Keywords::VIGILANCE));
    let announce = activation(katara, 0);
    let mut off_turn = game.clone();
    off_turn.state.active = player.other();
    reject(&mut off_turn, announce.clone());
    act(&mut game, announce);
    reject(&mut game, Action::ChooseX { value: 0 });
    act(&mut game, Action::ChooseX { value: 4 });
    pay(&mut game);
    drain(&mut game);
    assert_eq!(stats(&game, katara), (4, 4));
    assert_eq!(stats(&game, cub), (5, 5));
    assert_eq!(stats(&game, tokens[0]), (4, 4));
    assert_eq!(stats(&game, opponent), (3, 3));
    game.finish_cleanup();
    assert_eq!(stats(&game, cub), (3, 3));
    assert_eq!(stats(&game, tokens[0]), (1, 1));
}
