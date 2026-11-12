use super::*;

const CLUE: u16 = 5296;

fn end_step(game: &mut Game) {
    game.begin_step(Step::End);
    game.prepare_triggers();
}

fn order(game: &mut Game) {
    while matches!(game.state.pending, Pending::OrderTriggers { .. }) {
        choose(game, |action| matches!(action, Action::PutTrigger { .. }));
    }
}

fn set_suspected(game: &mut Game, card: CardId, suspected: bool) -> bool {
    game.set_suspected(game.object_ref(card), suspected)
}

#[test]
fn airtight_alibi_flashes_in_to_untap_and_clear_suspicion() {
    let opponent = Player::P0;
    let player = opponent.other();
    let mut game = scenario(opponent);
    let bear = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let aura = put(&mut game, player, "Airtight Alibi", Zone::Hand);
    assert!(set_suspected(&mut game, bear, true));
    game.state.objects[bear as usize].tapped = true;
    act(&mut game, Action::Pass);
    cast(&mut game, aura, Some(Target::Card(bear)));
    settle(&mut game);
    assert_eq!(stats(&game, bear), (4, 4));
    assert!(!game.is_suspected(bear));
    assert!(!game.object_view(bear).tapped);
    assert!(game.keywords(bear).contains(Keywords::HEXPROOF));
    assert!(!set_suspected(&mut game, bear, true));

    game.finish_cleanup();
    assert!(!game.keywords(bear).contains(Keywords::HEXPROOF));
    assert!(!set_suspected(&mut game, bear, true));
    game.move_object(aura, Zone::Graveyard);
    assert!(set_suspected(&mut game, bear, true));
    assert_eq!(stats(&game, bear), (2, 2));
}

#[test]
fn locked_hothouse_gives_an_extra_land_then_rechecks_seven_before_unlocking_the_top() {
    let player = Player::P1;
    let mut game = scenario(player);
    let case = put(&mut game, player, "Case of the Locked Hothouse", Zone::Hand);
    let lands = [(); 5].map(|_| put(&mut game, player, "Forest", Zone::Battlefield));
    let sixth = put(&mut game, player, "Island", Zone::Hand);
    let seventh = put(&mut game, player, "Mountain", Zone::Hand);
    let top = put(&mut game, player, "Bear Cub", Zone::Library);
    cast(&mut game, case, None);
    settle(&mut game);
    reject(&mut game, Action::AnnounceCast { card: top });
    assert!(game.observe(player).known_library_top[player.index()].is_none());
    act(&mut game, Action::PlayLand { card: sixth });
    act(&mut game, Action::PlayLand { card: seventh });
    end_step(&mut game);
    assert_eq!(game.state.stack.len(), 1);

    let mut short = game.clone();
    short.move_object(seventh, Zone::Graveyard);
    settle(&mut short);
    assert!(short.observe(player).solved_cases.is_empty());
    assert!(short.observe(player).known_library_top[player.index()].is_none());

    settle(&mut game);
    assert_eq!(game.observe(player).solved_cases, [game.object_ref(case)]);
    assert_eq!(
        game.observe(player).known_library_top[player.index()]
            .as_ref()
            .unwrap()
            .object,
        game.object_ref(top),
    );
    assert!(game.observe(player.other()).known_library_top[player.index()].is_none());
    reject(&mut game, Action::AnnounceCast { card: top });

    put(&mut game, player, "Forest", Zone::Library);
    for _ in 0..36 {
        if game.state.active == player && game.state.step == Step::Main1 {
            break;
        }
        match game.state.pending {
            Pending::Attackers => act(&mut game, Action::FinishAttackers),
            _ => act(&mut game, Action::Pass),
        }
    }
    assert_eq!(game.state.step, Step::Main1);
    for card in &lands[..2] {
        act(&mut game, Action::TapMana { card: *card });
    }
    cast(&mut game, top, None);
    settle(&mut game);
    assert_eq!(game.object_view(top).zone, Zone::Battlefield);
    game.move_object(case, Zone::Exile);
    game.move_object(case, Zone::Battlefield);
    assert!(game.observe(player).solved_cases.is_empty());
    assert!(game.observe(player).known_library_top[player.index()].is_none());
}

#[test]
fn trampled_garden_divides_entry_counters_and_only_solved_attacks_get_trample() {
    let player = Player::P0;
    let mut game = scenario(player);
    let case = put(&mut game, player, "Case of the Trampled Garden", Zone::Hand);
    let first = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let second = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let other = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    cast(&mut game, case, None);
    pass_pair(&mut game);
    reject(&mut game, target(other));
    act(&mut game, target(first));
    act(&mut game, target(second));
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    act(&mut game, Action::AllocateTargetAmount { amount: 1 });
    act(&mut game, Action::AllocateTargetAmount { amount: 1 });
    let mut partial = game.clone();
    partial.move_object(first, Zone::Exile);
    settle(&mut partial);
    assert_eq!(partial.counter_count(second, CounterKind::PlusOne), 1);
    attack(&mut partial, &[second]);
    assert!(partial.state.stack.is_empty());

    settle(&mut game);
    assert_eq!(stats(&game, first), (4, 4));
    assert_eq!(stats(&game, second), (4, 4));
    end_step(&mut game);
    assert_eq!(game.state.stack.len(), 1);
    let mut below = game.clone();
    below.move_object(second, Zone::Exile);
    settle(&mut below);
    assert!(below.observe(player).solved_cases.is_empty());
    settle(&mut game);
    assert_eq!(game.observe(player).solved_cases, [game.object_ref(case)]);
    attack(&mut game, &[first]);
    reject(&mut game, target(second));
    act(&mut game, target(first));
    settle(&mut game);
    assert_eq!(game.counter_count(first, CounterKind::PlusOne), 2);
    assert!(game.keywords(first).contains(Keywords::TRAMPLE));
    game.finish_cleanup();
    assert!(!game.keywords(first).contains(Keywords::TRAMPLE));
}

#[test]
fn hard_hitting_question_uses_the_friendly_creature_as_the_actual_damage_source() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Hard-Hitting Question", Zone::Hand);
    let shooter = put(&mut game, player, "Fynn, the Fangbearer", Zone::Battlefield);
    let victim = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
    let walker = put(
        &mut game,
        opponent,
        "Ajani, Caller of the Pride",
        Zone::Battlefield,
    );
    game.add_counter(walker, CounterKind::Loyalty, 4);
    let mut walker_branch = game.clone();
    announce(
        &mut walker_branch,
        spell,
        &[Target::Card(shooter), Target::Card(walker)],
    );
    settle(&mut walker_branch);
    assert_eq!(walker_branch.counter_count(walker, CounterKind::Loyalty), 3);

    act(&mut game, Action::AnnounceCast { card: spell });
    reject(&mut game, target(victim));
    act(&mut game, target(shooter));
    reject(&mut game, target(shooter));
    act(&mut game, target(victim));
    pay(&mut game);
    let mut lost_shooter = game.clone();
    lost_shooter.move_object(shooter, Zone::Exile);
    settle(&mut lost_shooter);
    assert_eq!(lost_shooter.object_view(victim).damage, 0);
    settle(&mut game);
    assert_eq!(game.object_view(victim).zone, Zone::Graveyard);
}

#[test]
fn pompous_gadabout_has_own_turn_hexproof_and_rejects_a_cloaked_blocker() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let source = put(&mut game, player, "Pompous Gadabout", Zone::Hand);
    let named = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let nameless = put(&mut game, opponent, "Hill Giant", Zone::Library);
    game.cloak_library_top(opponent, opponent);
    assert!(!game.advance_entries());
    game.state.objects[nameless as usize].summoning_sick = false;
    cast(&mut game, source, None);
    settle(&mut game);
    assert!(game.keywords(source).contains(Keywords::HEXPROOF));
    let mut other_turn = game.clone();
    other_turn.finish_cleanup();
    assert_eq!(other_turn.state.active, opponent);
    assert!(!other_turn.keywords(source).contains(Keywords::HEXPROOF));

    game.state.objects[source as usize].summoning_sick = false;
    attack(&mut game, &[source]);
    pass_pair(&mut game);
    reject(
        &mut game,
        Action::Block {
            blocker: nameless,
            attacker: source,
        },
    );
    act(
        &mut game,
        Action::Block {
            blocker: named,
            attacker: source,
        },
    );
    act(&mut game, Action::FinishBlockers);
}

#[test]
fn pride_of_hull_clade_uses_live_toughness_for_cost_and_grants_player_damage_draw() {
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "The Pride of Hull Clade", Zone::Hand);
    let wall = put(&mut game, player, "Rune-Sealed Wall", Zone::Battlefield);
    let other = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    game.state.players[player.index()].mana = [0, 0, 0, 0, 1, 0];
    reject(&mut game, Action::AnnounceCast { card: source });
    game.add_counter(other, CounterKind::PlusOne, 2);
    cast(&mut game, source, None);
    settle(&mut game);
    assert_eq!(stats(&game, source), (2, 15));
    assert!(game.keywords(source).contains(Keywords::DEFENDER));
    assert!(game.keywords(wall).contains(Keywords::DEFENDER));
    let mut before = game.clone();
    before.state.step = Step::Attackers;
    before.state.pending = Pending::Attackers;
    reject(&mut before, Action::Attack { card: wall });

    game.state.players[player.index()].mana = [0, 2, 0, 0, 0, 2];
    act(&mut game, activation(source, 0));
    act(&mut game, target(wall));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(stats(&game, wall), (1, 6));
    game.add_counter(wall, CounterKind::PlusOne, 1);
    attack(&mut game, &[wall]);
    pass_pair(&mut game);
    act(&mut game, Action::FinishBlockers);
    pass_pair(&mut game);
    assert_eq!(game.state.stack.len(), 1);
    let mut gone = game.clone();
    gone.move_object(wall, Zone::Graveyard);
    settle(&mut gone);
    settle(&mut game);
    for branch in [&game, &gone] {
        assert_eq!(
            branch.state.players[player.index()].cards_drawn_this_turn,
            7
        );
        assert_eq!(branch.state.players[player.other().index()].life, 18);
    }
}

#[test]
fn rope_equips_reach_and_a_single_blocker_limit_then_can_be_sacrificed_as_a_clue() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let rope = put(&mut game, player, "Rope", Zone::Hand);
    let wearer = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let first = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let second = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    cast(&mut game, rope, None);
    settle(&mut game);
    let card = game.object_view(rope).card;
    assert!(card.has_subtype("Equipment") && card.has_subtype("Clue"));
    act(&mut game, activation(rope, 1));
    act(&mut game, target(wearer));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(stats(&game, wearer), (3, 4));
    assert!(game.keywords(wearer).contains(Keywords::REACH));

    let mut sacrificed = game.clone();
    act(&mut sacrificed, activation(rope, 0));
    pay(&mut sacrificed);
    settle(&mut sacrificed);
    assert_eq!(sacrificed.object_view(rope).zone, Zone::Graveyard);
    assert_eq!(
        sacrificed.state.players[player.index()].cards_drawn_this_turn,
        1
    );
    assert_eq!(stats(&sacrificed, wearer), (2, 2));
    assert!(!sacrificed.keywords(wearer).contains(Keywords::REACH));

    let mut menace = game.clone();
    menace.state.objects[wearer as usize].gained_keywords = Keywords::MENACE;
    attack(&mut menace, &[wearer]);
    pass_pair(&mut menace);
    reject(
        &mut menace,
        Action::Block {
            blocker: first,
            attacker: wearer,
        },
    );
    act(&mut menace, Action::FinishBlockers);

    let piranhas = put(&mut game, player, "Eaten by Piranhas", Zone::Hand);
    cast(&mut game, piranhas, Some(Target::Card(wearer)));
    settle(&mut game);
    assert!(!game.keywords(wearer).contains(Keywords::REACH));
    attack(&mut game, &[wearer]);
    pass_pair(&mut game);
    act(
        &mut game,
        Action::Block {
            blocker: first,
            attacker: wearer,
        },
    );
    reject(
        &mut game,
        Action::Block {
            blocker: second,
            attacker: wearer,
        },
    );
    act(&mut game, Action::FinishBlockers);
}

#[test]
fn agrus_kos_can_decline_then_suspects_and_later_exiles_a_suspected_creature() {
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(
        &mut game,
        player,
        "Agrus Kos, Spirit of Justice",
        Zone::Hand,
    );
    let victim = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    cast(&mut game, source, None);
    pass_pair(&mut game);
    let mut declined = game.clone();
    act(&mut declined, Action::FinishTargets);
    settle(&mut declined);
    assert!(!declined.is_suspected(victim));
    act(&mut game, target(victim));
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    settle(&mut game);
    assert!(game.is_suspected(victim));
    assert!(
        game.keywords(source)
            .contains(Keywords::DOUBLE_STRIKE.union(Keywords::VIGILANCE))
    );
    game.state.objects[source as usize].summoning_sick = false;
    attack(&mut game, &[source]);
    assert!(!game.object_view(source).tapped);
    act(&mut game, target(victim));
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    settle(&mut game);
    assert_eq!(game.object_view(victim).zone, Zone::Exile);
}

#[test]
fn anzrag_gets_one_additional_combat_and_untaps_its_whole_creature_team() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let source = put(&mut game, player, "Anzrag, the Quake-Mole", Zone::Hand);
    let reserve = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let first = put(&mut game, opponent, "Llanowar Elves", Zone::Battlefield);
    let second = put(&mut game, opponent, "Llanowar Elves", Zone::Battlefield);
    cast(&mut game, source, None);
    settle(&mut game);
    assert_eq!(stats(&game, source), (8, 4));
    game.state.objects[reserve as usize].tapped = true;
    game.state.objects[source as usize].summoning_sick = false;
    act(&mut game, activation(source, 0));
    pay(&mut game);
    settle(&mut game);
    attack(&mut game, &[source]);
    pass_pair(&mut game);
    reject(&mut game, Action::FinishBlockers);
    for blocker in [first, second] {
        act(
            &mut game,
            Action::Block {
                blocker,
                attacker: source,
            },
        );
    }
    act(&mut game, Action::FinishBlockers);
    assert_eq!(game.state.stack.len(), 1);
    settle(&mut game);
    assert!(!game.object_view(source).tapped);
    assert!(!game.object_view(reserve).tapped);
    assert_eq!(
        game.state.combat_rules.as_ref().unwrap().after_combat[0].remaining,
        1
    );
}

#[test]
fn aurelia_uses_either_players_frozen_three_and_five_creature_declaration() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(opponent);
    let source = put(
        &mut game,
        player,
        "Aurelia, the Law Above",
        Zone::Battlefield,
    );
    let attackers = [(); 5].map(|_| put(&mut game, opponent, "Bear Cub", Zone::Battlefield));
    assert_eq!(stats(&game, source), (4, 4));
    assert!(
        game.keywords(source).contains(
            Keywords::FLYING
                .union(Keywords::VIGILANCE)
                .union(Keywords::HASTE)
        )
    );

    let mut three = game.clone();
    attack(&mut three, &attackers[..3]);
    settle(&mut three);
    assert_eq!(three.state.players[player.index()].cards_drawn_this_turn, 1);
    assert_eq!(three.state.players[player.index()].life, 20);

    attack(&mut game, &attackers);
    order(&mut game);
    assert_eq!(game.state.stack.len(), 2);
    game.move_object(attackers[0], Zone::Exile);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
    assert_eq!(game.state.players[player.index()].life, 23);
    assert_eq!(game.state.players[opponent.index()].life, 17);
}

fn choose_deadly_mode(game: &mut Game, spell: CardId, mode: u16, targets: &[CardId]) {
    act(game, Action::AnnounceCast { card: spell });
    act(game, Action::ChooseMode { mode });
    for &card in targets {
        act(game, target(card));
    }
    pay(game);
}

#[test]
fn deadly_complication_supports_each_mode_and_independent_targets_for_both() {
    let player = Player::P0;
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Deadly Complication", Zone::Hand);
    let own = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let enemy = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    assert!(set_suspected(&mut game, own, true));
    let mut destroy = game.clone();
    choose_deadly_mode(&mut destroy, spell, 0, &[enemy]);
    settle(&mut destroy);
    assert_eq!(destroy.object_view(enemy).zone, Zone::Graveyard);
    assert_eq!(destroy.counter_count(own, CounterKind::PlusOne), 0);

    let mut counter = game.clone();
    choose_deadly_mode(&mut counter, spell, 1, &[own]);
    pass_pair(&mut counter);
    act(&mut counter, Action::ChooseOptional { accept: false });
    assert!(counter.is_suspected(own));
    assert_eq!(counter.counter_count(own, CounterKind::PlusOne), 1);

    let mut shared = game.clone();
    choose_deadly_mode(&mut shared, spell, 2, &[own, own]);
    settle(&mut shared);
    assert_eq!(shared.object_view(own).zone, Zone::Graveyard);
    choose_deadly_mode(&mut game, spell, 2, &[enemy, own]);
    pass_pair(&mut game);
    act(&mut game, Action::ChooseOptional { accept: true });
    assert!(!game.is_suspected(own));
    assert_eq!(game.counter_count(own, CounterKind::PlusOne), 1);
    assert_eq!(game.object_view(enemy).zone, Zone::Graveyard);
}

#[test]
fn meddling_youths_can_attack_on_entry_and_investigate_only_for_three_declared_attackers() {
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Meddling Youths", Zone::Hand);
    let first = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let second = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    cast(&mut game, source, None);
    settle(&mut game);
    assert!(game.object_view(source).summoning_sick);
    let mut two = game.clone();
    attack(&mut two, &[source, first]);
    settle(&mut two);
    assert!(tokens(&two, player, CLUE).is_empty());
    attack(&mut game, &[source, first, second]);
    settle(&mut game);
    assert_eq!(tokens(&game, player, CLUE).len(), 1);
}
