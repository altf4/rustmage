use super::*;

const CLUE: u16 = 5296;
const DETECTIVE: u16 = 5329;
const HUMAN: u16 = 5254;
const MERFOLK: u16 = 5398;
const GOBLIN: u16 = 5324;
const THOPTER: u16 = 5266;
const PLANT: u16 = 5682;
const OOZE: u16 = 5683;

fn create(game: &mut Game, player: Player, factory: u16, count: u128) {
    game.create_tokens(player, Card::from_catalogue_id(factory), count, false);
    assert!(!game.advance_entries());
}

fn activate_voltstrider_with_two_creatures(game: &mut Game, player: Player) {
    let vehicle = put(game, player, "Kylox's Voltstrider", Zone::Battlefield);
    let first = put(game, player, "Hill Giant", Zone::Graveyard);
    let second = put(game, player, "Hill Giant", Zone::Graveyard);
    act(game, activation(vehicle, 0));
    for card in [first, second] {
        let object = game.object_ref(card);
        act(game, Action::ChooseEvidence { object });
    }
    act(game, Action::FinishEvidence);
    pay(game);
    settle(game);
    for card in [first, second] {
        assert_eq!(game.object_view(card).zone, Zone::Exile);
    }
}

#[test]
fn aftermath_analyst_mills_three_then_sacrifices_to_return_all_own_lands_tapped() {
    let player = Player::P1;
    let mut game = scenario(player);
    let own_land = put(&mut game, player, "Forest", Zone::Graveyard);
    let hostile_land = put(&mut game, player.other(), "Forest", Zone::Graveyard);
    let third = put(&mut game, player, "Bear Cub", Zone::Library);
    let second = put(&mut game, player, "Island", Zone::Library);
    let first = put(&mut game, player, "Evolving Wilds", Zone::Library);
    let source = put(&mut game, player, "Aftermath Analyst", Zone::Hand);
    cast(&mut game, source, None);
    settle(&mut game);
    assert_eq!(stats(&game, source), (1, 3));
    for card in [first, second, third] {
        assert_eq!(game.object_view(card).zone, Zone::Graveyard);
    }
    assert_eq!(game.state.players[player.index()].library.len(), 8);

    act(&mut game, activation(source, 0));
    pay(&mut game);
    assert_eq!(game.object_view(source).zone, Zone::Graveyard);
    settle(&mut game);
    for card in [first, second, own_land] {
        let land = game.object_view(card);
        assert_eq!(land.zone, Zone::Battlefield);
        assert!(land.tapped);
    }
    for card in [third, hostile_land] {
        assert_eq!(game.object_view(card).zone, Zone::Graveyard);
    }
}

#[test]
fn audience_with_trostani_draws_for_distinct_names_after_its_plant_enters() {
    let player = Player::P0;
    let mut game = scenario(player);
    create(&mut game, player, HUMAN, 2);
    create(&mut game, player, CLUE, 1);
    create(&mut game, player.other(), DETECTIVE, 1);
    put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let spell = put(&mut game, player, "Audience with Trostani", Zone::Hand);
    cast(&mut game, spell, None);
    settle(&mut game);
    let plant = tokens(&game, player, PLANT);
    assert_eq!(plant.len(), 1);
    assert_eq!(stats(&game, plant[0]), (0, 1));
    assert_eq!(game.object_view(plant[0]).colors, 16);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 2);
}

#[test]
fn chalk_outline_makes_one_detective_and_clue_for_two_departing_creature_cards() {
    let player = Player::P0;
    let mut game = scenario(player);
    let chalk = put(&mut game, player, "Chalk Outline", Zone::Hand);
    cast(&mut game, chalk, None);
    settle(&mut game);
    let noncreature = put(&mut game, player, "Divination", Zone::Graveyard);
    let hostile = put(&mut game, player.other(), "Hill Giant", Zone::Graveyard);
    for card in [noncreature, hostile] {
        game.move_object(card, Zone::Exile);
    }
    game.prepare_triggers();
    assert!(game.state.stack.is_empty());
    activate_voltstrider_with_two_creatures(&mut game, player);
    let detective = tokens(&game, player, DETECTIVE);
    assert_eq!(detective.len(), 1);
    assert_eq!(stats(&game, detective[0]), (2, 2));
    assert_eq!(game.object_view(detective[0]).colors, 3);
    assert_eq!(tokens(&game, player, CLUE).len(), 1);
}

fn choose_type(name: &str) -> Action {
    Action::ChooseCreatureType {
        creature_type: CreatureType::from_name(name).unwrap(),
    }
}

#[test]
fn a_killer_among_us_only_reveals_on_payment_and_checks_the_attacking_token_type() {
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "A Killer Among Us", Zone::Hand);
    cast(&mut game, source, None);
    pass_pair(&mut game);
    pass_pair(&mut game);
    let human = tokens(&game, player, HUMAN)[0];
    let merfolk = tokens(&game, player, MERFOLK)[0];
    let goblin = tokens(&game, player, GOBLIN)[0];
    for (card, color) in [(human, 1), (merfolk, 2), (goblin, 8)] {
        assert_eq!(stats(&game, card), (1, 1));
        assert_eq!(game.object_view(card).colors, color);
    }
    let mut wrong = game.clone();
    act(&mut game, choose_type("Human"));
    act(&mut wrong, choose_type("Goblin"));
    assert_eq!(
        serde_json::to_value(game.observe(player.other())).unwrap(),
        serde_json::to_value(wrong.observe(player.other())).unwrap(),
    );
    for branch in [&mut game, &mut wrong] {
        reject(branch, activation(source, 0));
        branch.state.objects[human as usize].summoning_sick = false;
        attack(branch, &[human]);
        act(branch, activation(source, 0));
        reject(branch, target(goblin));
        act(branch, target(human));
        assert!(branch.observe(player.other()).stack.is_empty());
        pay(branch);
        assert_eq!(branch.object_view(source).zone, Zone::Graveyard);
        assert!(matches!(
            branch.observe(player.other()).stack[0]
                .event_object
                .as_ref()
                .unwrap()
                .facts,
            EventObjectFacts::RevealedCreatureType(_)
        ));
        settle(branch);
    }
    assert_eq!(game.counter_count(human, CounterKind::PlusOne), 3);
    assert!(game.keywords(human).contains(Keywords::DEATHTOUCH));
    assert_eq!(wrong.counter_count(human, CounterKind::PlusOne), 0);
    assert!(!wrong.keywords(human).contains(Keywords::DEATHTOUCH));
    game.finish_cleanup();
    assert_eq!(game.counter_count(human, CounterKind::PlusOne), 3);
    assert!(!game.keywords(human).contains(Keywords::DEATHTOUCH));
}

#[test]
fn loxodon_eavesdropper_investigates_and_only_its_controllers_second_draw_buffs_it() {
    let player = Player::P1;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Loxodon Eavesdropper", Zone::Hand);
    cast(&mut game, source, None);
    settle(&mut game);
    assert_eq!(tokens(&game, player, CLUE).len(), 1);
    game.draw_many(player.other(), 2);
    game.draw_many(player, 1);
    game.prepare_triggers();
    assert!(game.state.stack.is_empty());
    assert_eq!(stats(&game, source), (3, 3));
    game.draw_many(player, 2);
    game.prepare_triggers();
    assert_eq!(game.state.stack.len(), 1);
    settle(&mut game);
    assert_eq!(stats(&game, source), (4, 4));
    assert!(game.keywords(source).contains(Keywords::VIGILANCE));
    game.finish_cleanup();
    assert_eq!(stats(&game, source), (3, 3));
    assert!(!game.keywords(source).contains(Keywords::VIGILANCE));
}

#[test]
fn sharp_eyed_rookie_checks_greater_stats_at_entry_and_at_resolution() {
    let player = Player::P0;
    let mut game = scenario(player);
    let rookie = put(&mut game, player, "Sharp-Eyed Rookie", Zone::Hand);
    cast(&mut game, rookie, None);
    settle(&mut game);
    assert!(game.keywords(rookie).contains(Keywords::VIGILANCE));
    let larger = put(&mut game, player, "Hill Giant", Zone::Hand);
    cast(&mut game, larger, None);
    pass_pair(&mut game);
    assert_eq!(game.state.stack.len(), 1);
    let mut no_longer_greater = game.clone();
    let growth = put(&mut no_longer_greater, player, "Giant Growth", Zone::Hand);
    cast(&mut no_longer_greater, growth, Some(Target::Card(rookie)));
    settle(&mut no_longer_greater);
    assert!(tokens(&no_longer_greater, player, CLUE).is_empty());
    assert_eq!(
        no_longer_greater.counter_count(rookie, CounterKind::PlusOne),
        0
    );

    settle(&mut game);
    assert_eq!(stats(&game, rookie), (3, 3));
    let tied = put(&mut game, player, "Hill Giant", Zone::Hand);
    cast(&mut game, tied, None);
    settle(&mut game);
    assert_eq!(stats(&game, rookie), (3, 3));
    assert_eq!(tokens(&game, player, CLUE).len(), 1);
}

#[test]
fn slime_against_humanity_counts_visible_owned_cards_and_counters_every_actual_ooze() {
    let player = Player::P0;
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Slime Against Humanity", Zone::Hand);
    put(&mut game, player, "Slime Against Humanity", Zone::Graveyard);
    put(&mut game, player, "Scavenging Ooze", Zone::Exile);
    put(
        &mut game,
        player.other(),
        "Slime Against Humanity",
        Zone::Graveyard,
    );
    let hidden = put(&mut game, player, "Slime Against Humanity", Zone::Exile);
    game.remember_face_down_exile(game.object_ref(hidden), 0b11, None);

    let mut doubled = game.clone();
    put(&mut doubled, player, "Doubling Season", Zone::Battlefield);
    for (branch, count, counters) in [(&mut game, 1, 4), (&mut doubled, 2, 8)] {
        cast(branch, spell, None);
        settle(branch);
        let oozes = tokens(branch, player, OOZE);
        assert_eq!(oozes.len(), count);
        for ooze in oozes {
            assert_eq!(branch.counter_count(ooze, CounterKind::PlusOne), counters);
            assert!(branch.keywords(ooze).contains(Keywords::TRAMPLE));
        }
    }
}

#[test]
fn they_went_this_way_puts_the_basic_tapped_and_investigates_on_a_miss_too() {
    let player = Player::P1;
    let mut game = scenario(player);
    let spell = put(&mut game, player, "They Went This Way", Zone::Hand);
    let land = put(&mut game, player, "Island", Zone::Library);
    let nonbasic = put(&mut game, player, "Cavern of Souls", Zone::Library);
    cast(&mut game, spell, None);
    pass_pair(&mut game);
    reject(&mut game, Action::ChooseSearch { card: nonbasic });
    let mut miss = game.clone();
    act(&mut miss, Action::FinishSearch);
    assert_eq!(miss.object_view(land).zone, Zone::Library);
    act(&mut game, Action::ChooseSearch { card: land });
    act(&mut game, Action::FinishSearch);
    assert_eq!(game.object_view(land).zone, Zone::Battlefield);
    assert!(game.object_view(land).tapped);
    for branch in [&game, &miss] {
        assert_eq!(tokens(branch, player, CLUE).len(), 1);
    }
}

#[test]
fn undergrowth_recon_targets_only_a_land_in_its_controllers_graveyard_at_own_upkeep() {
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Undergrowth Recon", Zone::Hand);
    let land = put(&mut game, player, "Island", Zone::Graveyard);
    let other_land = put(&mut game, player.other(), "Forest", Zone::Graveyard);
    let own_creature = put(&mut game, player, "Bear Cub", Zone::Graveyard);
    cast(&mut game, source, None);
    settle(&mut game);
    game.finish_cleanup();
    game.prepare_triggers();
    assert_eq!(game.state.active, player.other());
    assert!(game.state.stack.is_empty());
    game.finish_cleanup();
    game.prepare_triggers();
    assert_eq!(game.state.active, player);
    assert_eq!(game.state.step, Step::Upkeep);
    reject(&mut game, target(other_land));
    reject(&mut game, target(own_creature));
    act(&mut game, target(land));
    settle(&mut game);
    assert_eq!(game.object_view(land).zone, Zone::Battlefield);
    assert!(game.object_view(land).tapped);
}

#[test]
fn blood_spatter_analysis_mills_once_per_death_batch_and_returns_a_creature_after_sacrifice() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let source = put(&mut game, player, "Blood Spatter Analysis", Zone::Hand);
    let enemy = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
    let first = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let second = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let returned = put(&mut game, player, "Grizzly Bears", Zone::Library);
    let first_mill = put(&mut game, player, "Island", Zone::Library);
    cast(&mut game, source, None);
    pass_pair(&mut game);
    reject(&mut game, target(first));
    act(&mut game, target(enemy));
    settle(&mut game);
    assert_eq!(game.object_view(enemy).zone, Zone::Graveyard);
    assert_eq!(game.object_view(first_mill).zone, Zone::Graveyard);
    assert_eq!(game.counter_count(source, CounterKind::Bloodstain), 1);
    game.add_counter(source, CounterKind::Bloodstain, 3);
    game.sacrifice_batch(&[(player, first), (opponent, second)]);
    game.prepare_triggers();
    assert_eq!(game.state.stack.len(), 1);
    pass_pair(&mut game);
    assert_eq!(game.object_view(returned).zone, Zone::Graveyard);
    assert_eq!(game.object_view(source).zone, Zone::Graveyard);
    reject(&mut game, target(second));
    act(&mut game, target(returned));
    settle(&mut game);
    assert_eq!(game.object_view(returned).zone, Zone::Hand);
}

#[test]
fn curious_cadaver_returns_for_its_owners_clue_sacrifice() {
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Curious Cadaver", Zone::Graveyard);
    create(&mut game, player, CLUE, 1);
    create(&mut game, player.other(), CLUE, 1);
    let own = tokens(&game, player, CLUE)[0];
    let other = tokens(&game, player.other(), CLUE)[0];
    let nonclue = put(&mut game, player, "Soul-Guide Lantern", Zone::Battlefield);
    game.sacrifice_batch(&[(player.other(), other), (player, nonclue)]);
    game.prepare_triggers();
    assert!(game.state.stack.is_empty());
    act(&mut game, activation(own, 0));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(source).zone, Zone::Hand);
    cast(&mut game, source, None);
    settle(&mut game);
    assert_eq!(stats(&game, source), (3, 1));
    assert!(game.keywords(source).contains(Keywords::FLYING));
}

#[test]
fn detectives_satchel_investigates_twice_and_uses_prior_artifact_sacrifice_history() {
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Detective's Satchel", Zone::Hand);
    let artifact = put(&mut game, player, "Candy Trail", Zone::Battlefield);
    let mut only_destroyed = game.clone();
    only_destroyed.move_object(artifact, Zone::Graveyard);
    cast(&mut only_destroyed, source, None);
    settle(&mut only_destroyed);
    assert_eq!(tokens(&only_destroyed, player, CLUE).len(), 2);
    reject(&mut only_destroyed, activation(source, 0));

    game.sacrifice_batch(&[(player, artifact)]);
    cast(&mut game, source, None);
    settle(&mut game);
    assert_eq!(tokens(&game, player, CLUE).len(), 2);
    act(&mut game, activation(source, 0));
    pay(&mut game);
    settle(&mut game);
    assert!(game.object_view(source).tapped);
    let thopter = tokens(&game, player, THOPTER)[0];
    assert_eq!(stats(&game, thopter), (1, 1));
    assert_eq!(game.object_view(thopter).colors, 0);
    assert_eq!(game.object_view(thopter).kinds, 1 | 16);
    assert!(game.keywords(thopter).contains(Keywords::FLYING));
    game.finish_cleanup();
    game.finish_cleanup();
    assert!(!game.object_view(source).tapped);
    reject(&mut game, activation(source, 0));
}

#[test]
fn drag_the_canal_checks_any_actual_creature_death_and_surveils_before_investigating() {
    let player = Player::P1;
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Drag the Canal", Zone::Hand);
    let enemy = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let kept = put(&mut game, player, "Shock", Zone::Library);
    let milled = put(&mut game, player, "Island", Zone::Library);
    let mut survived = game.clone();
    cast(&mut survived, spell, None);
    settle(&mut survived);
    assert_eq!(tokens(&survived, player, DETECTIVE).len(), 1);
    assert!(tokens(&survived, player, CLUE).is_empty());
    assert_eq!(survived.state.players[player.index()].life, 20);

    game.sacrifice_batch(&[(player.other(), enemy)]);
    cast(&mut game, spell, None);
    pass_pair(&mut game);
    assert_eq!(game.state.players[player.index()].life, 22);
    assert_eq!(tokens(&game, player, DETECTIVE).len(), 1);
    assert!(tokens(&game, player, CLUE).is_empty());
    act(
        &mut game,
        Action::ArrangeLibrary {
            card: milled,
            destination: LibraryDestination::Graveyard,
        },
    );
    act(
        &mut game,
        Action::ArrangeLibrary {
            card: kept,
            destination: LibraryDestination::Top,
        },
    );
    settle(&mut game);
    assert_eq!(game.object_view(milled).zone, Zone::Graveyard);
    assert_eq!(
        game.state.players[player.index()].library.last(),
        Some(&kept)
    );
    assert_eq!(tokens(&game, player, CLUE).len(), 1);
}

#[test]
fn ezrim_investigates_twice_and_can_pay_an_artifact_for_each_keyword_choice() {
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Ezrim, Agency Chief", Zone::Hand);
    let bear = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    cast(&mut game, source, None);
    settle(&mut game);
    assert_eq!(stats(&game, source), (5, 5));
    assert!(game.keywords(source).contains(Keywords::FLYING));
    let clues = tokens(&game, player, CLUE);
    assert_eq!(clues.len(), 2);
    act(&mut game, activation(source, 0));
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(&mut game);
    }
    let object = game.object_ref(bear);
    reject(&mut game, Action::ChooseAdditionalCost { object });
    let object = game.object_ref(clues[0]);
    act(&mut game, Action::ChooseAdditionalCost { object });
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(&mut game);
    }
    assert_eq!(tokens(&game, player, CLUE), [clues[1]]);
    pass_pair(&mut game);
    for (index, keyword) in [Keywords::VIGILANCE, Keywords::LIFELINK, Keywords::HEXPROOF]
        .into_iter()
        .enumerate()
    {
        let mut branch = game.clone();
        act(
            &mut branch,
            Action::ChooseKeyword {
                index: index as u16,
            },
        );
        settle(&mut branch);
        assert!(branch.keywords(source).contains(keyword));
        branch.finish_cleanup();
        assert!(!branch.keywords(source).contains(keyword));
    }
}

#[test]
fn gleaming_geardrake_investigates_and_counts_each_own_artifact_sacrifice() {
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Gleaming Geardrake", Zone::Hand);
    let artifact = put(&mut game, player, "Candy Trail", Zone::Battlefield);
    let hostile = put(&mut game, player.other(), "Candy Trail", Zone::Battlefield);
    let creature = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    cast(&mut game, source, None);
    settle(&mut game);
    assert!(game.keywords(source).contains(Keywords::FLYING));
    let clue = tokens(&game, player, CLUE)[0];
    game.sacrifice_batch(&[(player, creature), (player.other(), hostile)]);
    game.prepare_triggers();
    assert!(game.state.stack.is_empty());
    game.sacrifice_batch(&[(player, artifact), (player, clue)]);
    game.prepare_triggers();
    while matches!(game.state.pending, Pending::OrderTriggers { .. }) {
        choose(&mut game, |action| {
            matches!(action, Action::PutTrigger { .. })
        });
    }
    settle(&mut game);
    assert_eq!(stats(&game, source), (3, 3));
}

#[test]
fn insidious_roots_counters_each_plant_once_and_only_creature_tokens_gain_mana() {
    let player = Player::P0;
    let mut game = scenario(player);
    let roots = put(&mut game, player, "Insidious Roots", Zone::Hand);
    let creature = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    create(&mut game, player, PLANT, 1);
    create(&mut game, player, CLUE, 1);
    let old = tokens(&game, player, PLANT)[0];
    game.state.objects[old as usize].summoning_sick = false;
    let clue = tokens(&game, player, CLUE)[0];
    cast(&mut game, roots, None);
    settle(&mut game);
    activate_voltstrider_with_two_creatures(&mut game, player);
    let plants = tokens(&game, player, PLANT);
    assert_eq!(plants.len(), 2);
    let new = *plants.iter().find(|&&card| card != old).unwrap();
    for card in plants {
        assert_eq!(stats(&game, card), (1, 2));
        assert_eq!(game.counter_count(card, CounterKind::PlusOne), 1);
    }
    let grantor = game.object_ref(roots);
    for card in [new, creature, clue] {
        reject(
            &mut game,
            Action::TapGrantedMana {
                card,
                grantor,
                production: [1, 0, 0, 0, 0, 0],
            },
        );
    }
    let white_before = game.state.players[player.index()].mana[0];
    act(
        &mut game,
        Action::TapGrantedMana {
            card: old,
            grantor,
            production: [1, 0, 0, 0, 0, 0],
        },
    );
    assert!(game.object_view(old).tapped);
    assert_eq!(game.state.players[player.index()].mana[0], white_before + 1);
}
