use super::*;

fn finish_targets(game: &mut Game) {
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(game, Action::FinishTargets);
    }
}

fn mode(game: &mut Game, spell: CardId, index: u16) {
    act(game, Action::AnnounceCast { card: spell });
    act(game, Action::ChooseMode { mode: index });
}

#[test]
fn archdruids_charm_applies_each_mode_with_its_own_target_and_destination_rules() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let charm = put(&mut game, player, "Archdruid's Charm", Zone::Hand);
    let ally = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let enemy = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
    let artifact = put(&mut game, player, "Soul-Guide Lantern", Zone::Battlefield);
    let land = put(&mut game, player, "Island", Zone::Library);
    let creature = put(&mut game, player, "Grizzly Bears", Zone::Library);
    let wrong = put(&mut game, player, "Shock", Zone::Library);

    let mut search = game.clone();
    mode(&mut search, charm, 0);
    pay(&mut search);
    pass_pair(&mut search);
    reject(&mut search, Action::ChooseSearch { card: wrong });
    let mut creature_route = search.clone();
    act(&mut creature_route, Action::ChooseSearch { card: creature });
    act(&mut creature_route, Action::FinishSearch);
    assert_eq!(creature_route.object_view(creature).zone, Zone::Hand);
    assert!(
        creature_route
            .observe(opponent)
            .objects
            .iter()
            .any(|o| o.id == creature)
    );
    act(&mut search, Action::ChooseSearch { card: land });
    act(&mut search, Action::FinishSearch);
    assert_eq!(search.object_view(land).zone, Zone::Battlefield);
    assert!(search.object_view(land).tapped);

    let mut power = game.clone();
    mode(&mut power, charm, 1);
    reject(&mut power, target(enemy));
    act(&mut power, target(ally));
    reject(&mut power, target(ally));
    act(&mut power, target(enemy));
    pay(&mut power);
    let mut lost_enemy = power.clone();
    lost_enemy.move_object(enemy, Zone::Exile);
    settle(&mut lost_enemy);
    assert_eq!(lost_enemy.counter_count(ally, CounterKind::PlusOne), 1);
    let mut lost_ally = power.clone();
    lost_ally.move_object(ally, Zone::Exile);
    settle(&mut lost_ally);
    assert_eq!(lost_ally.object_view(enemy).damage, 0);
    settle(&mut power);
    assert_eq!(power.counter_count(ally, CounterKind::PlusOne), 1);
    assert_eq!(power.object_view(enemy).zone, Zone::Graveyard);

    mode(&mut game, charm, 2);
    reject(&mut game, target(ally));
    act(&mut game, target(artifact));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(artifact).zone, Zone::Exile);
}

#[test]
fn fanatical_strength_gives_trample_and_three_power_through_combat_then_expires() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Fanatical Strength", Zone::Hand);
    let attacker = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let blocker = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);

    let mut opposing = game.clone();
    cast(&mut opposing, spell, Some(Target::Card(blocker)));
    settle(&mut opposing);
    assert_eq!(stats(&opposing, blocker), (5, 5));
    assert!(opposing.keywords(blocker).contains(Keywords::TRAMPLE));

    attack(&mut game, &[attacker]);
    cast(&mut game, spell, Some(Target::Card(attacker)));
    settle(&mut game);
    assert_eq!(stats(&game, attacker), (5, 5));
    assert!(game.keywords(attacker).contains(Keywords::TRAMPLE));
    pass_pair(&mut game);
    act(&mut game, Action::Block { blocker, attacker });
    act(&mut game, Action::FinishBlockers);
    pass_pair(&mut game);
    if let Some(choice) = game.damage_assignment_range() {
        assert_eq!(choice.target, Target::Card(blocker));
        act(
            &mut game,
            Action::AssignDamage {
                target: choice.target,
                amount: 2,
            },
        );
    }
    assert_eq!(game.state.players[opponent.index()].life, 17);
    assert_eq!(game.object_view(blocker).zone, Zone::Graveyard);
    game.finish_cleanup();
    assert_eq!(stats(&game, attacker), (2, 2));
    assert!(!game.keywords(attacker).contains(Keywords::TRAMPLE));
}

#[test]
fn get_a_leg_up_samples_live_controlled_creatures_once_and_grants_reach() {
    let player = Player::P1;
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Get a Leg Up", Zone::Hand);
    let ally = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let enemy = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let mut opposing = game.clone();
    cast(&mut opposing, spell, Some(Target::Card(enemy)));
    settle(&mut opposing);
    assert_eq!(stats(&opposing, enemy), (3, 3));
    assert!(opposing.keywords(enemy).contains(Keywords::REACH));

    cast(&mut game, spell, Some(Target::Card(ally)));
    let later = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    settle(&mut game);
    assert_eq!(stats(&game, ally), (4, 4));
    assert!(game.keywords(ally).contains(Keywords::REACH));
    game.move_object(later, Zone::Graveyard);
    assert_eq!(stats(&game, ally), (4, 4));
    game.finish_cleanup();
    assert_eq!(stats(&game, ally), (2, 2));
    assert!(!game.keywords(ally).contains(Keywords::REACH));
}

#[test]
fn glint_weaver_gains_life_after_distributing_surviving_counter_shares() {
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Glint Weaver", Zone::Hand);
    let ally = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let enemy = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    cast(&mut game, source, None);
    pass_pair(&mut game);
    assert!(game.keywords(source).contains(Keywords::REACH));
    act(&mut game, target(ally));
    act(&mut game, target(enemy));
    finish_targets(&mut game);
    act(&mut game, Action::AllocateTargetAmount { amount: 2 });
    act(&mut game, Action::AllocateTargetAmount { amount: 1 });
    let mut lost_ally = game.clone();
    lost_ally.move_object(ally, Zone::Exile);
    settle(&mut lost_ally);
    assert_eq!(lost_ally.counter_count(enemy, CounterKind::PlusOne), 1);
    assert_eq!(lost_ally.state.players[player.index()].life, 23);
    settle(&mut game);
    assert_eq!(stats(&game, ally), (4, 4));
    assert_eq!(stats(&game, enemy), (4, 4));
    assert_eq!(game.state.players[player.index()].life, 24);
}

#[test]
fn pick_your_poison_leaves_the_qualified_sacrifice_to_the_opponent() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Pick Your Poison", Zone::Hand);
    let own = put(&mut game, player, "Soul-Guide Lantern", Zone::Battlefield);
    let choices = [
        put(&mut game, opponent, "Soul-Guide Lantern", Zone::Battlefield),
        put(&mut game, opponent, "Doubling Season", Zone::Battlefield),
        put(&mut game, opponent, "Inspiring Overseer", Zone::Battlefield),
    ];
    for (index, victim) in choices.into_iter().enumerate() {
        let mut branch = game.clone();
        branch.state.objects[victim as usize].gained_keywords =
            Keywords::HEXPROOF.union(Keywords::INDESTRUCTIBLE);
        mode(&mut branch, spell, index as u16);
        pay(&mut branch);
        pass_pair(&mut branch);
        assert_eq!(branch.actor(), Some(opponent));
        let object = branch.object_ref(own);
        reject(&mut branch, Action::Sacrifice { object });
        let object = branch.object_ref(victim);
        act(&mut branch, Action::Sacrifice { object });
        settle(&mut branch);
        assert_eq!(branch.object_view(victim).zone, Zone::Graveyard);
        for other in choices.into_iter().filter(|&other| other != victim) {
            assert_eq!(branch.object_view(other).zone, Zone::Battlefield);
        }
    }
}

#[test]
fn rubblebelt_maverick_surveils_then_exiles_itself_as_a_sorcery_speed_cost() {
    let player = Player::P1;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Rubblebelt Maverick", Zone::Hand);
    let keep = put(&mut game, player, "Shock", Zone::Library);
    let mill = put(&mut game, player, "Island", Zone::Library);
    let creature = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    cast(&mut game, source, None);
    pass_pair(&mut game);
    pass_pair(&mut game);
    act(
        &mut game,
        Action::ArrangeLibrary {
            card: mill,
            destination: LibraryDestination::Graveyard,
        },
    );
    act(
        &mut game,
        Action::ArrangeLibrary {
            card: keep,
            destination: LibraryDestination::Top,
        },
    );
    assert_eq!(game.object_view(mill).zone, Zone::Graveyard);
    assert_eq!(
        game.state.players[player.index()].library.last(),
        Some(&keep)
    );

    game.move_object(source, Zone::Graveyard);
    let mut off_turn = game.clone();
    off_turn.state.active = player.other();
    reject(&mut off_turn, activation(source, 0));
    act(&mut game, activation(source, 0));
    act(&mut game, target(creature));
    pay(&mut game);
    assert_eq!(game.object_view(source).zone, Zone::Exile);
    settle(&mut game);
    assert_eq!(game.counter_count(creature, CounterKind::PlusOne), 1);
}

#[test]
fn topiary_panther_can_be_cast_or_basic_landcycled_during_an_opposing_turn() {
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Topiary Panther", Zone::Hand);
    let land = put(&mut game, player, "Island", Zone::Library);
    let nonbasic = put(&mut game, player, "Cavern of Souls", Zone::Library);
    let mut creature = game.clone();
    cast(&mut creature, source, None);
    settle(&mut creature);
    assert_eq!(stats(&creature, source), (6, 5));
    assert!(creature.keywords(source).contains(Keywords::TRAMPLE));

    game.state.active = player.other();
    game.state.step = Step::BeginCombat;
    game.state.players[player.index()].mana = [0, 0, 0, 0, 1, 1];
    act(&mut game, activation(source, 0));
    pay(&mut game);
    assert_eq!(game.object_view(source).zone, Zone::Graveyard);
    pass_pair(&mut game);
    reject(&mut game, Action::ChooseSearch { card: nonbasic });
    act(&mut game, Action::ChooseSearch { card: land });
    act(&mut game, Action::FinishSearch);
    assert_eq!(game.object_view(land).zone, Zone::Hand);
    assert!(
        game.observe(player.other())
            .objects
            .iter()
            .any(|o| o.id == land)
    );
}

#[test]
fn assassins_trophy_lets_the_targets_controller_find_an_untapped_basic_even_if_it_survives() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Assassin's Trophy", Zone::Hand);
    let own = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let victim = put(&mut game, opponent, "Soul-Guide Lantern", Zone::Battlefield);
    let basic = put(&mut game, opponent, "Island", Zone::Library);
    let nonbasic = put(&mut game, opponent, "Cavern of Souls", Zone::Library);
    act(&mut game, Action::AnnounceCast { card: spell });
    reject(&mut game, target(own));
    act(&mut game, target(victim));
    pay(&mut game);
    let mut stale = game.clone();
    stale.move_object(victim, Zone::Exile);
    settle(&mut stale);
    assert_eq!(stale.object_view(basic).zone, Zone::Library);
    let mut declined = game.clone();
    pass_pair(&mut declined);
    assert_eq!(declined.object_view(victim).zone, Zone::Graveyard);
    assert_eq!(declined.actor(), Some(opponent));
    let before = declined.state.rng;
    act(&mut declined, Action::ChooseOptional { accept: false });
    assert_eq!(declined.state.rng, before);

    game.state.objects[victim as usize].gained_keywords = Keywords::INDESTRUCTIBLE;
    pass_pair(&mut game);
    assert_eq!(game.object_view(victim).zone, Zone::Battlefield);
    act(&mut game, Action::ChooseOptional { accept: true });
    reject(&mut game, Action::ChooseSearch { card: nonbasic });
    act(&mut game, Action::ChooseSearch { card: basic });
    act(&mut game, Action::FinishSearch);
    assert_eq!(game.object_view(basic).zone, Zone::Battlefield);
    assert_eq!(game.object_view(basic).controller, opponent);
    assert!(!game.object_view(basic).tapped);
}

fn select_library(game: &mut Game, card: CardId) {
    act(game, Action::SelectLibrary { card });
    act(game, Action::FinishLibrarySelection);
}

#[test]
fn break_out_reveals_at_most_one_creature_and_only_cheap_creatures_can_enter_with_haste() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Break Out", Zone::Hand);
    let [outside, cheap, costly, first, second, third, fourth] = [
        "Bear Cub",
        "Llanowar Elves",
        "Hill Giant",
        "Forest",
        "Shock",
        "Mountain",
        "Island",
    ]
    .map(|name| put(&mut game, player, name, Zone::Library));
    cast(&mut game, spell, None);
    pass_pair(&mut game);
    assert_eq!(
        game.observe(player)
            .library_selection
            .unwrap()
            .remaining
            .len(),
        6
    );
    assert!(game.observe(opponent).library_selection.is_none());
    reject(&mut game, Action::SelectLibrary { card: outside });
    reject(&mut game, Action::SelectLibrary { card: first });

    let mut none = game.clone();
    act(&mut none, Action::FinishLibrarySelection);
    let library = &none.state.players[player.index()].library;
    assert_eq!(library.last(), Some(&outside));
    assert_eq!(
        library[..6]
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>(),
        [cheap, costly, first, second, third, fourth]
            .into_iter()
            .collect()
    );
    assert!(none.observe(opponent).zone_reveals.is_empty());

    let mut expensive = game.clone();
    select_library(&mut expensive, costly);
    assert_eq!(expensive.object_view(costly).zone, Zone::Hand);
    assert!(
        expensive
            .observe(opponent)
            .objects
            .iter()
            .any(|o| o.id == costly)
    );

    select_library(&mut game, cheap);
    let mut hand = game.clone();
    act(&mut hand, Action::ChooseOptional { accept: false });
    assert_eq!(hand.object_view(cheap).zone, Zone::Hand);
    act(&mut game, Action::ChooseOptional { accept: true });
    settle(&mut game);
    assert_eq!(game.object_view(cheap).zone, Zone::Battlefield);
    assert!(game.object_view(cheap).summoning_sick);
    assert!(game.keywords(cheap).contains(Keywords::HASTE));
    game.finish_cleanup();
    assert!(!game.keywords(cheap).contains(Keywords::HASTE));
}

fn copied_tokens(game: &Game, player: Player, card: Card) -> Vec<CardId> {
    game.controlled_token_ids(player)
        .filter(|&id| game.object_view(id).card == card)
        .collect()
}

#[test]
fn doppelgang_charges_three_per_x_and_copies_only_surviving_targets() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Doppelgang", Zone::Hand);
    let creature = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let land = put(&mut game, opponent, "Forest", Zone::Battlefield);
    put(&mut game, player, "Doubling Season", Zone::Battlefield);
    game.add_counter(creature, CounterKind::PlusOne, 2);
    game.state.players[player.index()].mana = [0, 1, 0, 0, 1, 6];
    let creature_definition = game.object_view(creature).card;
    let land_definition = game.object_view(land).card;
    act(&mut game, Action::AnnounceCast { card: spell });
    reject(&mut game, Action::ChooseX { value: 3 });
    let mut zero = game.clone();
    act(&mut zero, Action::ChooseX { value: 0 });
    act(
        &mut zero,
        Action::PayCosts {
            payment: [0, 1, 0, 0, 1, 0],
        },
    );
    settle(&mut zero);
    assert!(copied_tokens(&zero, player, creature_definition).is_empty());
    assert!(copied_tokens(&zero, player, land_definition).is_empty());

    act(&mut game, Action::ChooseX { value: 2 });
    act(&mut game, target(creature));
    reject(&mut game, target(creature));
    reject(&mut game, Action::FinishTargets);
    act(&mut game, target(land));
    finish_targets(&mut game);
    assert_eq!(game.mana_payment_domain().unwrap().generic, 6);
    act(
        &mut game,
        Action::PayCosts {
            payment: [0, 1, 0, 0, 1, 6],
        },
    );
    assert_eq!(game.state.stack[0].mana_value(), 8);
    let mut all = game.clone();
    settle(&mut all);
    assert_eq!(copied_tokens(&all, player, land_definition).len(), 4);
    game.move_object(land, Zone::Hand);
    game.move_object(land, Zone::Battlefield);
    settle(&mut game);
    assert!(copied_tokens(&game, player, land_definition).is_empty());
    for branch in [&all, &game] {
        let copies = copied_tokens(branch, player, creature_definition);
        assert_eq!(copies.len(), 4);
        for copy in copies {
            assert_eq!(stats(branch, copy), (2, 2));
            assert_eq!(branch.counter_count(copy, CounterKind::PlusOne), 0);
        }
    }
}

#[test]
fn ill_timed_explosion_draws_before_an_optional_discard_and_freezes_the_maximum_value() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Ill-Timed Explosion", Zone::Hand);
    let small = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let large = put(
        &mut game,
        opponent,
        "Earthshaker Dreadmaw",
        Zone::Battlefield,
    );
    let high = put(&mut game, player, "Hill Giant", Zone::Library);
    let x_spell = put(&mut game, player, "Exsanguinate", Zone::Library);
    cast(&mut game, spell, None);
    pass_pair(&mut game);
    for card in [high, x_spell] {
        assert_eq!(game.object_view(card).zone, Zone::Hand);
    }
    let mut decline = game.clone();
    act(&mut decline, Action::ChooseOptional { accept: false });
    assert!(decline.state.stack.is_empty());
    assert_eq!(decline.object_view(small).zone, Zone::Battlefield);
    assert_eq!(decline.object_view(large).damage, 0);

    act(&mut game, Action::ChooseOptional { accept: true });
    act(&mut game, Action::Discard { card: x_spell });
    assert_eq!(game.object_view(x_spell).zone, Zone::Hand);
    reject(&mut game, Action::Discard { card: x_spell });
    act(&mut game, Action::Discard { card: high });
    assert_eq!(game.state.stack.len(), 1);
    assert_eq!(
        game.state.stack[0].event_object.as_ref().unwrap().facts,
        EventObjectFacts::DiscardedManaValue(4)
    );
    assert_eq!(game.object_view(small).zone, Zone::Battlefield);
    assert_eq!(game.object_view(large).damage, 0);
    assert_eq!(game.object_view(spell).zone, Zone::Graveyard);

    game.move_object(high, Zone::Exile);
    let late = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
    settle(&mut game);
    assert_eq!(game.object_view(small).zone, Zone::Graveyard);
    assert_eq!(game.object_view(late).zone, Zone::Graveyard);
    assert_eq!(game.object_view(large).damage, 4);
    assert_eq!(game.state.players[opponent.index()].life, 20);
}

#[test]
fn kraul_whipcracker_reaches_and_can_destroy_an_opposing_noncreature_token() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let source = put(&mut game, player, "Kraul Whipcracker", Zone::Hand);
    let enemy = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let clue = Card::from_catalogue_id(5296);
    for controller in [player, opponent] {
        game.create_tokens(controller, clue, 1, false);
        assert!(!game.advance_entries());
    }
    let own = tokens(&game, player, 5296)[0];
    let hostile = tokens(&game, opponent, 5296)[0];
    cast(&mut game, source, None);
    pass_pair(&mut game);
    assert_eq!(stats(&game, source), (3, 2));
    assert!(game.keywords(source).contains(Keywords::REACH));
    reject(&mut game, target(own));
    reject(&mut game, target(enemy));
    act(&mut game, target(hostile));
    settle(&mut game);
    assert_eq!(tokens(&game, player, 5296), [own]);
    assert!(tokens(&game, opponent, 5296).is_empty());
}

fn sacrifice(game: &Game, card: CardId) -> Action {
    Action::Sacrifice {
        object: game.object_ref(card),
    }
}

fn cast_during_kylox(
    game: &mut Game,
    spell: CardId,
    spell_target: Option<Target>,
    cost: Option<ObjectRef>,
) {
    let object = game.object_ref(spell);
    act(
        game,
        Action::CastExiledDuringResolution {
            object,
            face: CastFace::Normal,
        },
    );
    for _ in 0..12 {
        match game.state.pending {
            Pending::ResolutionCasts { .. } => return,
            Pending::CastCosts { .. } => choose(game, |action| {
                matches!(action, Action::ChooseCastCost { .. })
            }),
            Pending::AdditionalCost { .. } => act(
                game,
                Action::ChooseAdditionalCost {
                    object: cost.unwrap(),
                },
            ),
            Pending::PayCosts { .. } => pay(game),
            Pending::Targets { .. } => act(
                game,
                Action::ChooseTarget {
                    target: spell_target.unwrap(),
                },
            ),
            ref pending => panic!("unexpected Kylox cast state {pending:?}"),
        }
    }
    panic!("Kylox free cast did not finish");
}

#[test]
fn kylox_attacks_immediately_and_uses_exact_sacrifices_for_multiple_free_spells() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let source = put(&mut game, player, "Kylox, Visionary Inventor", Zone::Hand);
    let first = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let second = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let reserve = put(&mut game, player, "Llanowar Elves", Zone::Battlefield);
    let [outside, land, x_spell, shock, mandatory] = [
        "Divination",
        "Forest",
        "Exsanguinate",
        "Shock",
        "Corrupted Conviction",
    ]
    .map(|name| put(&mut game, player, name, Zone::Library));
    cast(&mut game, source, None);
    settle(&mut game);
    assert_eq!(stats(&game, source), (4, 4));
    assert!(
        game.keywords(source)
            .contains(Keywords::MENACE.union(Keywords::HASTE))
    );
    assert!(game.object_view(source).summoning_sick);

    let mut ward = game.clone();
    let hostile = put(&mut ward, opponent, "Shock", Zone::Hand);
    act(&mut ward, Action::Pass);
    cast(&mut ward, hostile, Some(Target::Card(source)));
    assert_eq!(ward.state.stack.len(), 2);
    pass_pair(&mut ward);
    assert_eq!(ward.actor(), Some(opponent));
    act(&mut ward, Action::DeclineResolutionMana);
    assert_eq!(ward.object_view(source).damage, 0);
    assert_eq!(ward.object_view(hostile).zone, Zone::Graveyard);

    game.state.players[player.index()].mana = [0; 6];
    attack(&mut game, &[source]);
    pass_pair(&mut game);
    let bad = sacrifice(&game, source);
    reject(&mut game, bad);
    let mut none = game.clone();
    act(&mut none, Action::FinishEffectChoice);
    assert!(none.state.stack.is_empty());
    assert_eq!(none.object_view(mandatory).zone, Zone::Library);
    for card in [first, second] {
        let action = sacrifice(&game, card);
        act(&mut game, action);
    }
    act(&mut game, Action::FinishEffectChoice);
    for card in [first, second] {
        assert_eq!(game.object_view(card).zone, Zone::Graveyard);
    }
    for card in [land, x_spell, shock, mandatory] {
        assert_eq!(game.object_view(card).zone, Zone::Exile);
    }
    assert_eq!(game.object_view(outside).zone, Zone::Library);
    let object = game.object_ref(land);
    reject(
        &mut game,
        Action::CastExiledDuringResolution {
            object,
            face: CastFace::Normal,
        },
    );

    cast_during_kylox(&mut game, x_spell, None, None);
    assert_eq!(game.state.stack.last().unwrap().x, Some(0));
    cast_during_kylox(&mut game, shock, Some(Target::Player(opponent)), None);
    let cost = game.object_ref(reserve);
    cast_during_kylox(&mut game, mandatory, None, Some(cost));
    assert_eq!(game.object_view(reserve).zone, Zone::Graveyard);
    assert_eq!(game.state.stack.len(), 3);
    assert!(game.state.stack.iter().all(|item| {
        item.cast
            .is_some_and(|cast| cast.method == CastMethod::Free && cast.payment == [0; 6])
    }));
    act(&mut game, Action::FinishResolutionCasts);
    settle(&mut game);
    assert_eq!(game.state.players[opponent.index()].life, 18);
}

#[test]
fn lightning_helix_gains_life_with_prevented_damage_but_needs_a_legal_target() {
    let player = Player::P1;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Lightning Helix", Zone::Hand);
    let victim = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    cast(&mut game, source, Some(Target::Card(victim)));
    let mut stale = game.clone();
    stale.move_object(victim, Zone::Exile);
    settle(&mut stale);
    assert_eq!(stale.state.players[player.index()].life, 20);

    let mut prevented = game.clone();
    prevented.add_counter(victim, CounterKind::Shield, 1);
    settle(&mut prevented);
    assert_eq!(prevented.object_view(victim).zone, Zone::Battlefield);
    assert_eq!(prevented.object_view(victim).damage, 0);
    assert_eq!(prevented.counter_count(victim, CounterKind::Shield), 0);
    assert_eq!(prevented.state.players[player.index()].life, 23);
    settle(&mut game);
    assert_eq!(game.object_view(victim).zone, Zone::Graveyard);
    assert_eq!(game.state.players[player.index()].life, 23);
}
