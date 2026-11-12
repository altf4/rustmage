use super::*;

#[test]
fn azula_attacks_with_two_independent_triggers_and_combat_only_red_mana() {
    let p = Player::P0;
    let mut game = scenario(p);
    let azula = put(&mut game, p, "Azula, On the Hunt", Zone::Battlefield);
    declare_attackers(&mut game, &[azula]);
    assert_eq!(game.state.pending, Pending::OrderTriggers { player: p });
    finish_ordered_triggers(&mut game);
    assert_eq!(game.state.players[p.index()].life, 19);
    assert_eq!(game.state.players[p.index()].combat_red_mana, 2);
    assert_eq!(token_count(&game, Card(5296)), 1);
    reach(&mut game, |g| g.state.step == Step::Main2);
    assert_eq!(game.state.players[p.index()].combat_red_mana, 0);
}

#[test]
fn merchants_only_sacrifice_another_owned_creature_or_artifact_on_attack() {
    let p = Player::P1;
    let mut game = scenario(p);
    let merchants = put(&mut game, p, "Beetle-Headed Merchants", Zone::Battlefield);
    let own = put(&mut game, p, "Short Bow", Zone::Battlefield);
    let foreign = put(&mut game, p.other(), "Short Bow", Zone::Battlefield);
    declare_attackers(&mut game, &[merchants]);
    pass_pair(&mut game);
    for forbidden in [merchants, foreign] {
        let object = game.object_ref(forbidden);
        reject(&mut game, Action::Sacrifice { object });
    }
    let mut decline = game.clone();
    act(&mut decline, Action::DeclineEffectPayment);
    assert_eq!(decline.counter_count(merchants, CounterKind::PlusOne), 0);
    assert_eq!(decline.state.players[p.index()].cards_drawn_this_turn, 0);

    sacrifice(&mut game, own);
    assert_eq!(game.counter_count(merchants, CounterKind::PlusOne), 1);
    assert_eq!(game.state.players[p.index()].cards_drawn_this_turn, 1);
}

#[test]
fn callous_inspector_deals_preventable_damage_and_investigates_on_death() {
    let p = Player::P0;
    let mut game = scenario(p);
    let inspector = put(&mut game, p, "Callous Inspector", Zone::Battlefield);
    assert!(game.keywords(inspector).contains(Keywords::MENACE));
    game.move_object(inspector, Zone::Graveyard);
    game.prepare_triggers();
    settle(&mut game);
    assert_eq!(game.state.players[p.index()].life, 19);
    assert_eq!(token_count(&game, Card(5296)), 1);
}

#[test]
fn canyon_crawler_makes_food_and_swampcycles_for_a_nonbasic_swamp() {
    let p = Player::P1;
    let mut game = scenario(p);
    let crawler = put(&mut game, p, "Canyon Crawler", Zone::Hand);
    let swamp = put(&mut game, p, "Undercity Sewers", Zone::Library);
    let forest = game.state.players[p.index()].library[0];
    let mut creature = game.clone();
    cast(&mut creature, crawler, None);
    settle(&mut creature);
    assert!(creature.keywords(crawler).contains(Keywords::DEATHTOUCH));
    assert_eq!(token_count(&creature, Card(5246)), 1);
    let food = creature
        .state
        .objects
        .iter()
        .position(|o| o.zone == Zone::Battlefield && o.card == Card(5246))
        .unwrap() as CardId;
    act(&mut creature, activate(food, 0));
    pay(&mut creature);
    settle(&mut creature);
    assert_eq!(creature.state.players[p.index()].life, 23);

    game.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
    act(&mut game, activate(crawler, 0));
    pay(&mut game);
    pass_pair(&mut game);
    reject(&mut game, Action::ChooseSearch { card: forest });
    act(&mut game, Action::ChooseSearch { card: swamp });
    if game.legal_actions().contains(&Action::FinishSearch) {
        act(&mut game, Action::FinishSearch);
    }
    assert_eq!(game.object_view(crawler).zone, Zone::Graveyard);
    assert_eq!(game.object_view(swamp).zone, Zone::Hand);
}

#[test]
fn cat_gator_deals_lifelink_damage_based_on_live_swamps() {
    let p = Player::P0;
    let mut game = scenario(p);
    let gator = put(&mut game, p, "Cat-Gator", Zone::Hand);
    let first = put(&mut game, p, "Swamp", Zone::Battlefield);
    put(&mut game, p, "Undercity Sewers", Zone::Battlefield);
    put(&mut game, p.other(), "Swamp", Zone::Battlefield);
    cast(&mut game, gator, None);
    reach(&mut game, |g| {
        matches!(g.state.pending, Pending::Targets { .. })
    });
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(p.other()),
        },
    );
    let mut lost_swamp = game.clone();
    lost_swamp.move_object(first, Zone::Exile);
    settle(&mut lost_swamp);
    assert_eq!(lost_swamp.state.players[p.index()].life, 21);
    assert_eq!(lost_swamp.state.players[p.other().index()].life, 19);
    game.move_object(gator, Zone::Exile);
    settle(&mut game);
    assert_eq!(game.state.players[p.index()].life, 22);
    assert_eq!(game.state.players[p.other().index()].life, 18);
}

#[test]
fn corrupt_court_official_forces_only_its_target_opponent_to_discard() {
    let p = Player::P1;
    let mut game = scenario(p);
    let creature = put(&mut game, p, "Corrupt Court Official", Zone::Hand);
    let private = put(&mut game, p.other(), "Forest", Zone::Hand);
    cast(&mut game, creature, None);
    reach(&mut game, |g| {
        matches!(g.state.pending, Pending::Targets { .. })
    });
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(p),
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(p.other()),
        },
    );
    pass_pair(&mut game);
    assert_eq!(game.actor(), Some(p.other()));
    assert!(!game.observe(p).objects.iter().any(|o| o.id == private));
    act(&mut game, Action::Discard { card: private });
    assert_eq!(game.object_view(private).zone, Zone::Graveyard);
}

#[test]
fn day_of_black_sun_blanks_in_range_creatures_before_destroying_them() {
    let p = Player::P0;
    let mut game = scenario(p);
    let spell = put(&mut game, p, "Day of Black Sun", Zone::Hand);
    let inspector = put(&mut game, p.other(), "Callous Inspector", Zone::Battlefield);
    let own = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    let expensive = put(&mut game, p.other(), "Hill Giant", Zone::Battlefield);
    let source = game.object_ref(inspector);
    game.grant_keyword_effects(&[inspector, expensive], source, Keywords::INDESTRUCTIBLE);
    act(&mut game, Action::AnnounceCast { card: spell });
    act(&mut game, Action::ChooseX { value: 2 });
    pay(&mut game);
    settle(&mut game);
    for card in [inspector, own] {
        assert_eq!(game.object_view(card).zone, Zone::Graveyard);
    }
    assert_eq!(game.object_view(expensive).zone, Zone::Battlefield);
    assert!(game.keywords(expensive).contains(Keywords::INDESTRUCTIBLE));
    assert_eq!(game.state.players[p.other().index()].life, 20);
    assert_eq!(token_count(&game, Card(5296)), 0);
}

#[test]
fn epic_downfall_exiles_only_creatures_of_mana_value_three_or_more() {
    let p = Player::P1;
    let mut game = scenario(p);
    let spell = put(&mut game, p, "Epic Downfall", Zone::Hand);
    let cheap = put(&mut game, p.other(), "Bear Cub", Zone::Battlefield);
    let exact = put(&mut game, p.other(), "Nimble Brigand", Zone::Battlefield);
    act(&mut game, Action::AnnounceCast { card: spell });
    reject(&mut game, target(cheap));
    act(&mut game, target(exact));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(exact).zone, Zone::Exile);
}

#[test]
fn june_and_foggy_swamp_hunters_get_their_abilities_after_the_second_actual_draw() {
    let p = Player::P0;
    let mut game = scenario(p);
    let june = put(&mut game, p, "June, Bounty Hunter", Zone::Battlefield);
    let hunters = put(&mut game, p, "Foggy Swamp Hunters", Zone::Battlefield);
    let victim = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    let foreign = put(&mut game, p.other(), "Bear Cub", Zone::Battlefield);
    let hunter_keywords = Keywords::MENACE.union(Keywords::LIFELINK);
    game.draw_many(p, 1);
    assert!(!game.keywords(june).contains(Keywords::CANT_BE_BLOCKED));
    assert!(!game.keywords(hunters).intersects(hunter_keywords));
    game.draw_many(p, 1);
    assert!(game.keywords(june).contains(Keywords::CANT_BE_BLOCKED));
    assert!(game.keywords(hunters).contains(hunter_keywords));

    let mut offturn = game.clone();
    offturn.state.active = p.other();
    reject(&mut offturn, activate(june, 0));
    game.state.step = Step::BeginCombat;
    act(&mut game, activate(june, 0));
    for card in [june, foreign] {
        let object = game.object_ref(card);
        reject(&mut game, Action::ChooseAdditionalCost { object });
    }
    let object = game.object_ref(victim);
    act(&mut game, Action::ChooseAdditionalCost { object });
    pay(&mut game);
    settle(&mut game);
    assert_eq!(token_count(&game, Card(5296)), 1);
    game.finish_cleanup();
    assert!(!game.keywords(june).contains(Keywords::CANT_BE_BLOCKED));
    assert!(!game.keywords(hunters).intersects(hunter_keywords));
}

#[test]
fn merchants_graveyard_activation_returns_only_the_current_source_incarnation() {
    let p = Player::P1;
    let mut game = scenario(p);
    let card = put(&mut game, p, "Merchant of Many Hats", Zone::Graveyard);
    let other = put(
        &mut game,
        p.other(),
        "Merchant of Many Hats",
        Zone::Graveyard,
    );
    reject(&mut game, activate(other, 0));
    game.state.players[p.index()].mana = [0, 0, 1, 0, 0, 2];
    act(&mut game, activate(card, 0));
    pay(&mut game);
    let mut stale = game.clone();
    stale.move_object(card, Zone::Exile);
    stale.move_object(card, Zone::Graveyard);
    settle(&mut stale);
    assert_eq!(stale.object_view(card).zone, Zone::Graveyard);
    settle(&mut game);
    assert_eq!(game.object_view(card).zone, Zone::Hand);
}

#[test]
fn obsessive_pursuit_and_pirate_peddlers_track_actual_permanent_sacrifices() {
    let p = Player::P0;
    let mut game = scenario(p);
    let spell = put(&mut game, p, "Obsessive Pursuit", Zone::Hand);
    let peddlers = put(&mut game, p, "Pirate Peddlers", Zone::Battlefield);
    let other = put(&mut game, p.other(), "Pirate Peddlers", Zone::Battlefield);
    let wilds = put(&mut game, p, "Evolving Wilds", Zone::Battlefield);
    cast(&mut game, spell, None);
    settle(&mut game);
    assert_eq!(game.state.players[p.index()].life, 19);
    assert_eq!(token_count(&game, Card(5296)), 1);
    game.begin_step(Step::Upkeep);
    game.give_priority();
    game.prepare_triggers();
    settle(&mut game);
    assert_eq!(game.state.players[p.index()].life, 18);
    assert_eq!(token_count(&game, Card(5296)), 2);
    let tokens: Vec<_> = game.controlled_token_ids(p).collect();
    for clue in tokens {
        act(&mut game, activate(clue, 0));
        pay(&mut game);
        settle(&mut game);
    }
    act(&mut game, activate(wilds, 0));
    pay(&mut game);
    reach(&mut game, |g| {
        g.legal_actions().contains(&Action::FinishSearch)
    });
    act(&mut game, Action::FinishSearch);
    settle(&mut game);
    assert_eq!(game.counter_count(peddlers, CounterKind::PlusOne), 3);
    assert_eq!(game.counter_count(other, CounterKind::PlusOne), 0);
    assert!(game.keywords(peddlers).contains(Keywords::DEATHTOUCH));

    declare_attackers(&mut game, &[peddlers]);
    reject(&mut game, target(other));
    act(&mut game, target(peddlers));
    settle(&mut game);
    assert_eq!(game.counter_count(peddlers, CounterKind::PlusOne), 6);
    assert!(game.keywords(peddlers).contains(Keywords::LIFELINK));
}

#[test]
fn swampsnare_trap_locks_the_flying_discount_and_applies_minus_five_minus_three_once() {
    let p = Player::P1;
    let mut game = scenario(p);
    let spell = put(&mut game, p, "Swampsnare Trap", Zone::Hand);
    let flyer = put(&mut game, p.other(), "Canyon Crawler", Zone::Battlefield);
    let grounded = put(&mut game, p.other(), "Canyon Crawler", Zone::Battlefield);
    game.state.objects[flyer as usize].gained_keywords = Keywords::FLYING;
    game.state.players[p.index()].mana = [0, 0, 1, 0, 0, 1];
    act(&mut game, Action::AnnounceCast { card: spell });
    reject(&mut game, target(grounded));
    act(&mut game, target(flyer));
    pay(&mut game);
    game.state.objects[flyer as usize].gained_keywords = Keywords::NONE;
    settle(&mut game);
    assert_eq!(stats(&game, flyer), (1, 3));
    assert_eq!(game.state.players[p.index()].mana, [0; 6]);
}

#[test]
fn tundra_tank_protects_a_controlled_creature_and_can_crew_then_firebend() {
    let p = Player::P0;
    let mut game = scenario(p);
    let tank = put(&mut game, p, "Tundra Tank", Zone::Hand);
    let own = put(&mut game, p, "Goblin Piker", Zone::Battlefield);
    let foreign = put(&mut game, p.other(), "Bear Cub", Zone::Battlefield);
    cast(&mut game, tank, None);
    reach(&mut game, |g| {
        matches!(g.state.pending, Pending::Targets { .. })
    });
    reject(&mut game, target(foreign));
    act(&mut game, target(own));
    settle(&mut game);
    assert!(game.keywords(own).contains(Keywords::INDESTRUCTIBLE));
    crew(&mut game, tank, 0, own);
    assert_eq!(stats(&game, tank), (4, 4));
    game.state.objects[tank as usize].summoning_sick = false;
    declare_attackers(&mut game, &[tank]);
    settle(&mut game);
    assert_eq!(game.state.players[p.index()].combat_red_mana, 1);
    game.finish_cleanup();
    assert!(!game.keywords(own).contains(Keywords::INDESTRUCTIBLE));
    assert_eq!(game.current_kinds(tank), 16);
}

#[test]
fn zukos_conviction_returns_the_target_to_hand_or_the_battlefield_tapped() {
    let p = Player::P1;
    let mut game = scenario(p);
    let spell = put(&mut game, p, "Zuko's Conviction", Zone::Hand);
    let own = put(&mut game, p, "Bear Cub", Zone::Graveyard);
    let enemy = put(&mut game, p.other(), "Bear Cub", Zone::Graveyard);
    for kicked in [false, true] {
        let mut branch = game.clone();
        act(&mut branch, Action::AnnounceCast { card: spell });
        act(&mut branch, Action::ChooseKicker { pay: kicked });
        reject(&mut branch, target(enemy));
        act(&mut branch, target(own));
        pay(&mut branch);
        settle(&mut branch);
        let returned = branch.object_view(own);
        assert_eq!(
            returned.zone,
            if kicked {
                Zone::Battlefield
            } else {
                Zone::Hand
            }
        );
        assert_eq!(returned.tapped, kicked);
    }
}
