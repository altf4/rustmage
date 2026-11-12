use super::*;

#[test]
fn azula_always_lies_has_independent_targets_when_both_modes_are_chosen() {
    let p = Player::P0;
    let mut game = scenario(p);
    let spell = put(&mut game, p, "Azula Always Lies", Zone::Hand);
    let victim = put(&mut game, p.other(), "Hill Giant", Zone::Battlefield);
    let own = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    for mode in 0..3 {
        let mut branch = game.clone();
        act(&mut branch, Action::AnnounceCast { card: spell });
        act(&mut branch, Action::ChooseMode { mode });
        if mode != 1 {
            act(&mut branch, target(victim));
        }
        if mode != 0 {
            act(&mut branch, target(own));
        }
        pay(&mut branch);
        if mode == 2 {
            let mut stale = branch.clone();
            stale.move_object(victim, Zone::Exile);
            settle(&mut stale);
            assert_eq!(stale.counter_count(own, CounterKind::PlusOne), 1);
        }
        settle(&mut branch);
        assert_eq!(
            stats(&branch, victim),
            if mode == 1 { (3, 3) } else { (2, 2) }
        );
        assert_eq!(
            branch.counter_count(own, CounterKind::PlusOne),
            u64::from(mode != 0)
        );
        branch.finish_cleanup();
        assert_eq!(stats(&branch, victim), (3, 3));
    }
}

#[test]
fn buzzard_wasp_may_sacrifice_and_inherits_every_counter_from_own_other_creatures() {
    let p = Player::P1;
    let mut game = scenario(p);
    let colony = put(&mut game, p, "Buzzard-Wasp Colony", Zone::Hand);
    let own = put(&mut game, p, "Hill Giant", Zone::Battlefield);
    let enemy = put(&mut game, p.other(), "Hill Giant", Zone::Battlefield);
    let artifact = put(&mut game, p, "Short Bow", Zone::Battlefield);
    for body in [own, enemy] {
        game.add_counter(body, CounterKind::PlusOne, 2);
        game.add_counter(body, CounterKind::Lore, 1);
    }
    cast(&mut game, colony, None);
    pass_pair(&mut game);
    pass_pair(&mut game);
    let mut decline = game.clone();
    act(&mut decline, Action::DeclineEffectPayment);
    assert_eq!(decline.state.players[p.index()].cards_drawn_this_turn, 0);
    let foreign = game.object_ref(enemy);
    reject(&mut game, Action::Sacrifice { object: foreign });
    sacrifice(&mut game, artifact);
    assert_eq!(game.object_view(artifact).zone, Zone::Graveyard);
    assert_eq!(game.state.players[p.index()].cards_drawn_this_turn, 1);
    assert!(game.keywords(colony).contains(Keywords::FLYING));

    game.move_object(enemy, Zone::Graveyard);
    game.prepare_triggers();
    settle(&mut game);
    assert_eq!(game.counter_count(colony, CounterKind::PlusOne), 0);
    game.move_object(own, Zone::Graveyard);
    game.prepare_triggers();
    settle(&mut game);
    assert_eq!(game.counter_count(colony, CounterKind::PlusOne), 2);
    assert_eq!(game.counter_count(colony, CounterKind::Lore), 1);
}

#[test]
fn dai_li_reveals_only_the_target_opponents_hand_or_earthbends_a_controlled_land() {
    let p = Player::P0;
    let mut game = scenario(p);
    let spell = put(&mut game, p, "Dai Li Indoctrination", Zone::Hand);
    let land_card = put(&mut game, p.other(), "Swamp", Zone::Hand);
    let instant = put(&mut game, p.other(), "Shock", Zone::Hand);
    let victim = put(&mut game, p.other(), "Short Bow", Zone::Hand);
    let land = put(&mut game, p, "Forest", Zone::Battlefield);
    let other_land = put(&mut game, p.other(), "Forest", Zone::Battlefield);
    let mut reveal = game.clone();
    assert!(
        !reveal
            .observe(p)
            .objects
            .iter()
            .any(|object| object.id == victim)
    );
    act(&mut reveal, Action::AnnounceCast { card: spell });
    act(&mut reveal, Action::ChooseMode { mode: 0 });
    reject(
        &mut reveal,
        Action::ChooseTarget {
            target: Target::Player(p),
        },
    );
    act(
        &mut reveal,
        Action::ChooseTarget {
            target: Target::Player(p.other()),
        },
    );
    pay(&mut reveal);
    pass_pair(&mut reveal);
    for card in [land_card, instant] {
        let object = reveal.object_ref(card);
        reject(&mut reveal, Action::ChooseRevealedDiscard { object });
    }
    let object = reveal.object_ref(victim);
    act(&mut reveal, Action::ChooseRevealedDiscard { object });
    assert_eq!(reveal.object_view(victim).zone, Zone::Graveyard);

    act(&mut game, Action::AnnounceCast { card: spell });
    act(&mut game, Action::ChooseMode { mode: 1 });
    reject(&mut game, target(other_land));
    act(&mut game, target(land));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.current_kinds(land), 3);
    assert_eq!(stats(&game, land), (2, 2));
    assert!(game.keywords(land).contains(Keywords::HASTE));
    game.move_object(land, Zone::Exile);
    game.prepare_triggers();
    settle(&mut game);
    assert_eq!(game.object_view(land).zone, Zone::Battlefield);
    assert!(game.object_view(land).tapped);
}

#[test]
fn deadly_precision_uses_either_four_generic_or_an_exact_sacrifice() {
    let p = Player::P1;
    let mut game = scenario(p);
    let spell = put(&mut game, p, "Deadly Precision", Zone::Hand);
    let victim = put(&mut game, p.other(), "Hill Giant", Zone::Battlefield);
    let artifact = put(&mut game, p, "Short Bow", Zone::Battlefield);
    let land = put(&mut game, p, "Swamp", Zone::Battlefield);
    act(&mut game, Action::AnnounceCast { card: spell });
    for route in 0..2 {
        let mut branch = game.clone();
        act(&mut branch, Action::ChooseSpellCost { cost: route });
        act(&mut branch, target(victim));
        if branch
            .legal_actions()
            .iter()
            .any(|a| matches!(a, Action::ChooseAdditionalCost { .. }))
        {
            let object = branch.object_ref(land);
            reject(&mut branch, Action::ChooseAdditionalCost { object });
            let object = branch.object_ref(artifact);
            act(&mut branch, Action::ChooseAdditionalCost { object });
        }
        pay(&mut branch);
        settle(&mut branch);
        assert_eq!(branch.object_view(victim).zone, Zone::Graveyard);
        assert_eq!(
            branch.object_view(artifact).zone == Zone::Graveyard,
            route == 0
        );
        let spent: u32 = branch.state.players[p.index()].mana.iter().sum();
        assert_eq!(spent, if route == 0 { 47 } else { 43 });
    }
}

#[test]
fn fatal_fissure_observes_the_exact_targets_death_and_then_targets_a_land() {
    let p = Player::P0;
    let mut game = scenario(p);
    let spell = put(&mut game, p, "Fatal Fissure", Zone::Hand);
    let victim = put(&mut game, p.other(), "Hill Giant", Zone::Battlefield);
    let unrelated = put(&mut game, p.other(), "Bear Cub", Zone::Battlefield);
    let land = put(&mut game, p, "Forest", Zone::Battlefield);
    let enemy_land = put(&mut game, p.other(), "Forest", Zone::Battlefield);
    cast(&mut game, spell, Some(Target::Card(victim)));
    settle(&mut game);
    game.move_object(unrelated, Zone::Graveyard);
    game.prepare_triggers();
    settle(&mut game);
    assert_eq!(game.current_kinds(land), 2);
    game.move_object(victim, Zone::Graveyard);
    game.prepare_triggers();
    reject(&mut game, target(enemy_land));
    act(&mut game, target(land));
    settle(&mut game);
    assert_eq!(stats(&game, land), (4, 4));
    assert!(game.keywords(land).contains(Keywords::HASTE));
}

#[test]
fn the_engineers_raid_targets_another_creature_or_even_an_uncrewed_vehicle() {
    let p = Player::P1;
    let mut game = scenario(p);
    let engineer = put(&mut game, p, "Fire Nation Engineer", Zone::Battlefield);
    let attacker = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    let vehicle = put(&mut game, p, "Phoenix Fleet Airship", Zone::Battlefield);
    let enemy = put(&mut game, p.other(), "Hill Giant", Zone::Battlefield);
    let mut no_raid = game.clone();
    start_end_step(&mut no_raid, p);
    assert_eq!(no_raid.state.pending, Pending::Priority);
    declare_attackers(&mut game, &[attacker]);
    start_end_step(&mut game, p);
    reject(&mut game, target(engineer));
    reject(&mut game, target(enemy));
    act(&mut game, target(vehicle));
    settle(&mut game);
    assert_eq!(game.counter_count(vehicle, CounterKind::PlusOne), 1);
}

#[test]
fn trebuchet_has_defender_reach_and_makes_one_attacking_boulder_per_attack() {
    let p = Player::P0;
    let mut game = scenario(p);
    let trebuchet = put(&mut game, p, "Fire Navy Trebuchet", Zone::Battlefield);
    let first = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    let second = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    assert!(
        game.keywords(trebuchet)
            .contains(Keywords::DEFENDER.union(Keywords::REACH))
    );
    game.state.pending = Pending::Attackers;
    game.state.step = Step::Attackers;
    reject(&mut game, Action::Attack { card: trebuchet });
    declare_attackers(&mut game, &[first, second]);
    settle(&mut game);
    let boulder = game
        .state
        .objects
        .iter()
        .position(|o| o.zone == Zone::Battlefield && o.card == Card(5710))
        .unwrap() as CardId;
    assert_eq!(token_count(&game, Card(5710)), 1);
    assert!(game.object_view(boulder).tapped);
    assert_eq!(game.observe(p).attackers.len(), 3);
    assert_eq!(stats(&game, boulder), (2, 1));
    start_end_step(&mut game, p);
    settle(&mut game);
    assert_eq!(token_count(&game, Card(5710)), 0);
}

#[test]
fn heartless_act_rechecks_for_any_counter_or_removes_up_to_three_of_different_kinds() {
    let p = Player::P1;
    let mut game = scenario(p);
    let spell = put(&mut game, p, "Heartless Act", Zone::Hand);
    let clean = put(&mut game, p.other(), "Hill Giant", Zone::Battlefield);
    let marked = put(&mut game, p.other(), "Hill Giant", Zone::Battlefield);
    for kind in [
        CounterKind::PlusOne,
        CounterKind::Lore,
        CounterKind::Stun,
        CounterKind::Quest,
    ] {
        game.add_counter(marked, kind, 1);
    }
    act(&mut game, Action::AnnounceCast { card: spell });
    let mut destroy = game.clone();
    act(&mut destroy, Action::ChooseMode { mode: 0 });
    reject(&mut destroy, target(marked));
    act(&mut destroy, target(clean));
    pay(&mut destroy);
    let mut gained = destroy.clone();
    gained.add_counter(clean, CounterKind::Stun, 1);
    settle(&mut gained);
    assert_eq!(gained.object_view(clean).zone, Zone::Battlefield);
    settle(&mut destroy);
    assert_eq!(destroy.object_view(clean).zone, Zone::Graveyard);

    act(&mut game, Action::ChooseMode { mode: 1 });
    act(&mut game, target(marked));
    pay(&mut game);
    pass_pair(&mut game);
    let mut none = game.clone();
    act(&mut none, Action::FinishTargetCounterRemoval);
    assert_eq!(none.counter_count(marked, CounterKind::Stun), 1);
    for counter in [CounterKind::PlusOne, CounterKind::Lore, CounterKind::Stun] {
        act(&mut game, Action::SelectTargetCounter { counter });
    }
    if game
        .legal_actions()
        .contains(&Action::FinishTargetCounterRemoval)
    {
        act(&mut game, Action::FinishTargetCounterRemoval);
    }
    assert_eq!(game.counter_count(marked, CounterKind::Quest), 1);
    assert_eq!(game.counter_count(marked, CounterKind::Stun), 0);
}

#[test]
fn hog_monkey_exhausts_for_two_counters_and_grants_menace_only_to_countered_creatures() {
    let p = Player::P0;
    let mut game = scenario(p);
    let hog = put(&mut game, p, "Hog-Monkey", Zone::Battlefield);
    let plain = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    let enemy = put(&mut game, p.other(), "Hill Giant", Zone::Battlefield);
    game.add_counter(enemy, CounterKind::PlusOne, 1);
    act(&mut game, activate(hog, 0));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.counter_count(hog, CounterKind::PlusOne), 2);
    reject(&mut game, activate(hog, 0));
    game.begin_step(Step::BeginCombat);
    game.give_priority();
    game.prepare_triggers();
    reject(&mut game, target(plain));
    reject(&mut game, target(enemy));
    act(&mut game, target(hog));
    settle(&mut game);
    assert!(game.keywords(hog).contains(Keywords::MENACE));
    game.finish_cleanup();
    game.state.priority = p;
    game.state.players[p.index()].mana = [8; 6];
    reject(&mut game, activate(hog, 0));
    assert!(!game.keywords(hog).contains(Keywords::MENACE));
}

#[test]
fn mai_charges_each_noncreature_caster_and_has_first_strike() {
    let p = Player::P1;
    let mut game = scenario(p);
    let mai = put(&mut game, p, "Mai, Scornful Striker", Zone::Battlefield);
    let creature = put(&mut game, p, "Bear Cub", Zone::Hand);
    let spell = put(&mut game, p, "Quick Study", Zone::Hand);
    let response = put(&mut game, p.other(), "Quick Study", Zone::Hand);
    assert!(game.keywords(mai).contains(Keywords::FIRST_STRIKE));
    cast(&mut game, creature, None);
    settle(&mut game);
    assert_eq!(game.state.players[p.index()].life, 20);
    cast(&mut game, spell, None);
    act(&mut game, Action::Pass);
    cast(&mut game, response, None);
    settle(&mut game);
    assert_eq!(game.state.players[p.index()].life, 18);
    assert_eq!(game.state.players[p.other().index()].life, 18);
}

#[test]
fn northern_air_temple_drains_for_own_shrines_and_other_own_shrine_entries() {
    let p = Player::P0;
    let mut game = scenario(p);
    let temple = put(&mut game, p, "Northern Air Temple", Zone::Hand);
    let first = put(&mut game, p, "The Spirit Oasis", Zone::Battlefield);
    put(&mut game, p.other(), "The Spirit Oasis", Zone::Battlefield);
    cast(&mut game, temple, None);
    finish_ordered_triggers(&mut game);
    assert_eq!(game.state.players[p.index()].life, 22);
    assert_eq!(game.state.players[p.other().index()].life, 18);
    game.move_object(first, Zone::Exile);
    let shrine = put(&mut game, p, "The Spirit Oasis", Zone::Hand);
    cast(&mut game, shrine, None);
    finish_ordered_triggers(&mut game);
    assert_eq!(game.state.players[p.index()].life, 23);
    assert_eq!(game.state.players[p.other().index()].life, 17);
}

#[test]
fn ozais_cruelty_damages_then_forces_the_same_player_to_discard() {
    let p = Player::P1;
    let mut game = scenario(p);
    let spell = put(&mut game, p, "Ozai's Cruelty", Zone::Hand);
    let own = put(&mut game, p, "Bear Cub", Zone::Hand);
    let first = put(&mut game, p.other(), "Swamp", Zone::Hand);
    let second = put(&mut game, p.other(), "Hill Giant", Zone::Hand);
    cast(&mut game, spell, Some(Target::Player(p.other())));
    pass_pair(&mut game);
    assert_eq!(game.state.players[p.other().index()].life, 18);
    assert_eq!(game.actor(), Some(p.other()));
    reject(&mut game, Action::Discard { card: own });
    for card in [first, second] {
        act(&mut game, Action::Discard { card });
    }
    for card in [first, second] {
        assert_eq!(game.object_view(card).zone, Zone::Graveyard);
    }
}

#[test]
fn raven_eagle_has_separate_entry_attack_and_own_second_draw_obligations() {
    let p = Player::P0;
    let mut game = scenario(p);
    let eagle = put(&mut game, p, "Raven Eagle", Zone::Hand);
    let body = put(&mut game, p, "Bear Cub", Zone::Graveyard);
    let spell = put(&mut game, p.other(), "Shock", Zone::Graveyard);
    let draw = put(&mut game, p, "Quick Study", Zone::Hand);
    cast(&mut game, eagle, None);
    reach(&mut game, |g| {
        matches!(g.state.pending, Pending::Targets { .. })
    });
    act(&mut game, target(spell));
    finish_targets(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(spell).zone, Zone::Exile);
    assert_eq!(token_count(&game, Card(5296)), 0);
    cast(&mut game, draw, None);
    settle(&mut game);
    assert_eq!(game.state.players[p.index()].life, 21);
    assert_eq!(game.state.players[p.other().index()].life, 19);
    game.state.objects[eagle as usize].summoning_sick = false;
    declare_attackers(&mut game, &[eagle]);
    reach(&mut game, |g| {
        matches!(g.state.pending, Pending::Targets { .. })
    });
    act(&mut game, target(body));
    finish_targets(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(body).zone, Zone::Exile);
    assert_eq!(token_count(&game, Card(5296)), 1);
}

#[test]
fn realm_of_koh_uses_basic_land_entry_black_mana_and_a_spirit_token() {
    let p = Player::P1;
    let mut game = scenario(p);
    let land = put(&mut game, p, "Realm of Koh", Zone::Hand);
    let mut no_basic = game.clone();
    act(&mut no_basic, Action::PlayLand { card: land });
    assert!(no_basic.object_view(land).tapped);
    put(&mut game, p, "Forest", Zone::Battlefield);
    act(&mut game, Action::PlayLand { card: land });
    assert!(!game.object_view(land).tapped);
    let mut mana = game.clone();
    let black = mana.state.players[p.index()].mana[2];
    act(
        &mut mana,
        Action::ActivateMana {
            card: land,
            ability: 0,
            payment: [0; 6],
            counter_payment: Counters::default(),
            production: [0, 0, 1, 0, 0, 0],
        },
    );
    assert_eq!(mana.state.players[p.index()].mana[2], black + 1);
    act(&mut game, activate(land, 1));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(token_count(&game, Card(5705)), 1);
    assert!(game.object_view(land).tapped);
}
