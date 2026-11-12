use super::*;

#[test]
fn fire_lord_azula_copies_a_countered_spell_and_can_change_the_targets() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let azula = put(&mut game, player, "Fire Lord Azula", Zone::Battlefield);
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    let negate = put(&mut game, opponent, "Negate", Zone::Hand);

    let mut outside_combat = game.clone();
    cast(&mut outside_combat, shock, Some(Target::Player(opponent)));
    assert_eq!(outside_combat.state.stack.len(), 1);
    drain(&mut outside_combat);
    assert_eq!(life(&outside_combat, opponent), 18);

    attack(&mut game, &[azula]);
    drain(&mut game);
    assert_eq!(game.state.players[player.index()].combat_red_mana, 2);
    cast(&mut game, shock, Some(Target::Player(opponent)));
    assert_eq!(game.state.stack.len(), 2);
    act(&mut game, Action::Pass);
    cast(&mut game, negate, Some(Target::Spell(shock)));
    pass_pair(&mut game);
    assert_eq!(game.object_view(shock).zone, Zone::Graveyard);
    pass_pair(&mut game);
    act(
        &mut game,
        Action::SetStackTarget {
            index: 0,
            target: Target::Player(player),
        },
    );
    act(&mut game, Action::FinishStackTargets);
    assert!(game.state.stack.last().unwrap().is_copy);
    drain(&mut game);
    assert_eq!(life(&game, opponent), 20);
    assert_eq!(life(&game, player), 18);
}

#[test]
fn fire_lord_zuko_distinguishes_an_exile_cast_from_an_entry_and_bends_at_live_power() {
    let player = Player::P1;
    let mut game = scenario(player);
    let zuko = put(&mut game, player, "Fire Lord Zuko", Zone::Battlefield);
    let creature = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let swift = put(&mut game, player, "Aang, Swift Savior", Zone::Hand);
    let returning = put(&mut game, player, "Mountain", Zone::Exile);
    cast(&mut game, swift, None);
    pass_pair(&mut game);
    act(&mut game, target(creature));
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    drain(&mut game);
    assert_eq!(game.object_view(creature).zone, Zone::Exile);
    assert_eq!(game.counter_count(zuko, CounterKind::PlusOne), 0);
    act(
        &mut game,
        Action::AnnounceAirbend {
            card: creature,
            face: CastFace::Normal,
            flash_surcharge: false,
        },
    );
    pay(&mut game);
    drain(&mut game);
    assert_eq!(game.counter_count(zuko, CounterKind::PlusOne), 1);
    assert_eq!(game.counter_count(swift, CounterKind::PlusOne), 1);
    assert_eq!(game.counter_count(creature, CounterKind::PlusOne), 0);
    game.move_object(returning, Zone::Battlefield);
    drain(&mut game);
    assert_eq!(game.counter_count(zuko, CounterKind::PlusOne), 2);
    assert_eq!(game.counter_count(creature, CounterKind::PlusOne), 1);
    attack(&mut game, &[zuko]);
    game.add_counter(zuko, CounterKind::PlusOne, 1);
    drain(&mut game);
    assert_eq!(game.state.players[player.index()].combat_red_mana, 5);
}

#[test]
fn foggy_swamp_spirit_keeper_observes_the_second_draw_and_deals_lifelink_damage() {
    let player = Player::P0;
    let mut game = scenario(player);
    let keeper = put(
        &mut game,
        player,
        "Foggy Swamp Spirit Keeper",
        Zone::Battlefield,
    );
    game.draw_many(player, 3);
    drain(&mut game);
    let spirits = tokens(&game, player, 5705);
    assert_eq!(spirits.len(), 1);
    assert_eq!(game.object_view(spirits[0]).name, "Spirit Token");
    assert!(game.keywords(keeper).contains(Keywords::LIFELINK));
    attack(&mut game, &[keeper]);
    pass_pair(&mut game);
    act(&mut game, Action::FinishBlockers);
    pass_pair(&mut game);
    pass_pair(&mut game);
    drain(&mut game);
    assert_eq!(life(&game, player), 22);
    assert_eq!(life(&game, player.other()), 18);
}

#[test]
fn ozai_changes_expiring_mana_and_rechecks_both_keywords_at_six() {
    let player = Player::P0;
    let mut game = scenario(player);
    let ozai = put(
        &mut game,
        player,
        "Ozai, the Phoenix King",
        Zone::Battlefield,
    );
    let base = Keywords::HASTE.union(Keywords::TRAMPLE);
    let extra = Keywords::FLYING.union(Keywords::INDESTRUCTIBLE);
    game.state.players[player.index()].mana = [0, 2, 0, 0, 3, 0];
    assert_eq!(game.keywords(ozai), base);
    game.state.players[player.index()].mana[0] = 1;
    assert!(game.keywords(ozai).contains(base.union(extra)));
    pass_pair(&mut game);
    assert_eq!(game.state.players[player.index()].mana, [0, 0, 0, 6, 0, 0]);
    game.state.players[player.index()].mana = [0; 6];
    game.state.objects[ozai as usize].summoning_sick = true;
    assert_eq!(game.keywords(ozai), base);
    attack(&mut game, &[ozai]);
    drain(&mut game);
    assert_eq!(game.state.players[player.index()].combat_red_mana, 4);
}

#[test]
fn platypus_bear_mills_two_and_only_attacks_while_a_lesson_is_in_its_graveyard() {
    let player = Player::P1;
    let mut game = scenario(player);
    let platypus = put(&mut game, player, "Platypus-Bear", Zone::Hand);
    let lesson = put(&mut game, player, "Firebending Lesson", Zone::Library);
    let land = put(&mut game, player, "Forest", Zone::Library);
    cast(&mut game, platypus, None);
    drain(&mut game);
    assert_eq!(game.object_view(lesson).zone, Zone::Graveyard);
    assert_eq!(game.object_view(land).zone, Zone::Graveyard);
    assert!(game.keywords(platypus).contains(Keywords::DEFENDER));
    game.state.objects[platypus as usize].summoning_sick = false;
    let mut empty = game.clone();
    empty.move_object(lesson, Zone::Exile);
    empty.state.step = Step::Attackers;
    empty.state.pending = Pending::Attackers;
    reject(&mut empty, Action::Attack { card: platypus });
    attack(&mut game, &[platypus]);
    assert!(game.object_view(platypus).attacking);
}
