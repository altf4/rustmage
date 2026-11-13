use super::*;

fn keep_top(game: &mut Game, count: usize) {
    let actor = game.actor().unwrap();
    for _ in 0..count {
        let view = game.observe(actor).library_choice.unwrap();
        act(
            game,
            Action::ArrangeLibrary {
                card: view.remaining[0].id,
                destination: LibraryDestination::Top,
            },
        );
    }
}

fn airbend_cast(card: CardId) -> Action {
    Action::AnnounceAirbend {
        card,
        face: CastFace::Normal,
        flash_surcharge: false,
    }
}

fn declare(game: &mut Game, card: CardId) {
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    act(game, Action::Attack { card });
    act(game, Action::FinishAttackers);
}

fn tap_mana(game: &mut Game, card: CardId, ability: u16, payment: Mana, production: Mana) {
    let direct = Action::ActivateMana {
        card,
        ability,
        payment,
        production,
        counter_payment: Counters::default(),
    };
    if game.legal_actions().contains(&direct) {
        act(game, direct);
    } else {
        act(
            game,
            Action::AnnounceMana {
                card,
                ability,
                counter_payment: Counters::default(),
                production,
            },
        );
        if game
            .legal_actions()
            .contains(&Action::ChooseMana { production })
        {
            act(game, Action::ChooseMana { production });
        }
        if game.legal_actions().contains(&Action::PayCosts { payment }) {
            act(game, Action::PayCosts { payment });
        }
    }
}

#[test]
fn iceberg_flashes_in_exiles_then_waterbends_to_free_the_card_and_scry() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(opponent);
    let iceberg = put(&mut game, player, "Aang's Iceberg", Zone::Hand);
    let captured = put(&mut game, opponent, "Short Bow", Zone::Battlefield);
    let creature = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    game.state.priority = player;
    cast(&mut game, iceberg, None);
    pass_pair(&mut game);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(iceberg),
        },
    );
    target(&mut game, Target::Card(captured));
    finish_targets(&mut game);
    drain(&mut game);
    assert_eq!(game.object_view(captured).zone, Zone::Exile);

    game.state.priority = player;
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 2];
    activate(&mut game, iceberg, 0);
    act(&mut game, Action::BeginWaterbend);
    let object = game.object_ref(creature);
    act(&mut game, Action::ChooseWaterbender { object });
    pay(&mut game);
    assert!(game.object_view(creature).tapped);
    assert_eq!(game.object_view(iceberg).zone, Zone::Battlefield);
    let mut departed = game.clone();
    departed.move_object(iceberg, Zone::Hand);
    drain(&mut departed);
    assert!(departed.observe(player).library_choice.is_none());

    pass_pair(&mut game);
    assert_eq!(game.object_view(iceberg).zone, Zone::Graveyard);
    assert_eq!(game.object_view(captured).zone, Zone::Battlefield);
    assert_eq!(game.object_view(captured).controller, opponent);
    keep_top(&mut game, 2);
}

#[test]
fn abandoned_air_temple_entry_uses_a_basic_and_its_paid_ability_counters_the_group() {
    let player = Player::P1;
    for basic in [false, true] {
        let mut game = scenario(player);
        let temple = put(&mut game, player, "Abandoned Air Temple", Zone::Hand);
        if basic {
            put(&mut game, player, "Plains", Zone::Battlefield);
        }
        let bear = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let opponent = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
        act(&mut game, Action::PlayLand { card: temple });
        assert_eq!(game.object_view(temple).tapped, !basic);
        if !basic {
            continue;
        }
        let mut mana = game.clone();
        mana.state.players[player.index()].mana = [0; 6];
        tap_mana(&mut mana, temple, 0, [0; 6], [1, 0, 0, 0, 0, 0]);
        assert_eq!(mana.state.players[player.index()].mana[0], 1);
        activate(&mut game, temple, 1);
        pay(&mut game);
        drain(&mut game);
        assert!(game.object_view(temple).tapped);
        assert_eq!(game.counter_count(bear, CounterKind::PlusOne), 1);
        assert_eq!(game.counter_count(opponent, CounterKind::PlusOne), 0);
    }
}

#[test]
fn airbender_ascension_airbends_on_entry_then_blinks_at_its_quest_threshold() {
    let player = Player::P0;
    let other = player.other();
    let mut game = scenario(player);
    let ascension = put(&mut game, player, "Airbender Ascension", Zone::Hand);
    let foe = put(&mut game, other, "Bear Cub", Zone::Battlefield);
    let ally = put(&mut game, player, "Bear Cub", Zone::Hand);
    cast(&mut game, ascension, None);
    pass_pair(&mut game);
    target(&mut game, Target::Card(foe));
    finish_targets(&mut game);
    drain(&mut game);
    assert_eq!(game.object_view(foe).zone, Zone::Exile);
    cast(&mut game, ally, None);
    drain(&mut game);
    assert_eq!(game.counter_count(ascension, CounterKind::Quest), 1);

    game.add_counter(ascension, CounterKind::Quest, 3);
    game.begin_step(Step::End);
    game.prepare_triggers();
    let before = game.object_ref(ally);
    target(&mut game, Target::Card(ally));
    finish_targets(&mut game);
    let mut lost_counter = game.clone();
    lost_counter.pay_counters(ascension, &Counters::of(CounterKind::Quest, 1));
    drain(&mut lost_counter);
    assert_eq!(lost_counter.object_ref(ally), before);
    drain(&mut game);
    assert_eq!(game.object_view(ally).zone, Zone::Battlefield);
    assert_ne!(game.object_ref(ally), before);
}

#[test]
fn steadfast_appa_flashes_in_airbends_several_permanents_and_rewards_each_recast() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(opponent);
    let appa = put(&mut game, player, "Appa, Steadfast Guardian", Zone::Hand);
    let artifacts = [
        put(&mut game, player, "Short Bow", Zone::Battlefield),
        put(&mut game, player, "Short Bow", Zone::Battlefield),
    ];
    let enemy = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    game.state.priority = player;
    cast(&mut game, appa, None);
    pass_pair(&mut game);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(enemy),
        },
    );
    for card in artifacts {
        target(&mut game, Target::Card(card));
    }
    finish_targets(&mut game);
    drain(&mut game);
    assert!(game.keywords(appa).contains(Keywords::FLYING));
    assert!(
        artifacts
            .iter()
            .all(|&card| game.object_view(card).zone == Zone::Exile)
    );

    game.state.active = player;
    game.state.priority = player;
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 4];
    for card in artifacts {
        act(&mut game, airbend_cast(card));
        pay(&mut game);
        drain(&mut game);
        assert_eq!(game.object_view(card).zone, Zone::Battlefield);
    }
    assert_eq!(tokens_of(&game, player, "Ally").len(), 2);
}

#[test]
fn avatars_wrath_spares_its_target_and_bans_opposing_nonhand_casts() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let wrath = put(&mut game, player, "Avatar's Wrath", Zone::Hand);
    let spared = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let own = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let enemy = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    cast(&mut game, wrath, Some(Target::Card(spared)));
    drain(&mut game);
    assert_eq!(game.object_view(wrath).zone, Zone::Exile);
    assert_eq!(game.object_view(spared).zone, Zone::Battlefield);
    assert_eq!(game.object_view(own).zone, Zone::Exile);
    assert_eq!(game.object_view(enemy).zone, Zone::Exile);
    let mut self_cast = game.clone();
    act(&mut self_cast, airbend_cast(own));
    pay(&mut self_cast);
    drain(&mut self_cast);
    assert_eq!(self_cast.object_view(own).zone, Zone::Battlefield);

    game.state.step = Step::End;
    game.give_priority();
    pass_pair(&mut game);
    pass_pair(&mut game);
    pass_pair(&mut game);
    assert_eq!(game.state.active, opponent);
    assert_eq!(game.state.step, Step::Main1);
    game.state.players[opponent.index()].mana = [0, 0, 0, 0, 0, 2];
    reject(&mut game, airbend_cast(enemy));
}

#[test]
fn barrels_filter_once_each_turn_and_sacrifice_to_deal_five() {
    let player = Player::P0;
    let mut game = scenario(player);
    let barrels = put(
        &mut game,
        player,
        "Barrels of Blasting Jelly",
        Zone::Battlefield,
    );
    let giant = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let generic = [0, 0, 0, 0, 0, 1];
    let red = [0, 0, 0, 1, 0, 0];
    let mountain = put(&mut game, player, "Mountain", Zone::Battlefield);
    game.state.players[player.index()].mana = [0; 6];
    act(&mut game, Action::TapMana { card: mountain });
    tap_mana(&mut game, barrels, 0, red, red);
    assert_eq!(game.state.players[player.index()].mana, red);
    reject(
        &mut game,
        Action::ActivateMana {
            card: barrels,
            ability: 0,
            payment: red,
            counter_payment: Counters::default(),
            production: red,
        },
    );

    game.state.step = Step::End;
    game.give_priority();
    pass_pair(&mut game);
    game.state.priority = player;
    game.state.players[player.index()].mana = generic;
    tap_mana(&mut game, barrels, 0, generic, red);
    assert_eq!(game.state.players[player.index()].mana, red);

    for _ in 0..4 {
        let land = put(&mut game, player, "Mountain", Zone::Battlefield);
        act(&mut game, Action::TapMana { card: land });
    }
    activate(&mut game, barrels, 1);
    target(&mut game, Target::Card(giant));
    pay(&mut game);
    assert_eq!(game.object_view(barrels).zone, Zone::Graveyard);
    drain(&mut game);
    assert_eq!(game.object_view(giant).zone, Zone::Graveyard);
}

#[test]
fn farm_animals_gain_life_even_when_their_sacrifice_ability_has_no_target() {
    let player = Player::P0;
    let mut game = scenario(player);
    let animals = put(&mut game, player, "Curious Farm Animals", Zone::Battlefield);
    let enchantment = put(
        &mut game,
        player.other(),
        "Impact Tremors",
        Zone::Battlefield,
    );
    activate(&mut game, animals, 0);
    let mut decline = game.clone();
    finish_targets(&mut decline);
    pay(&mut decline);
    drain(&mut decline);
    assert_eq!(life(&decline, player), 23);
    assert_eq!(decline.object_view(enchantment).zone, Zone::Battlefield);

    target(&mut game, Target::Card(enchantment));
    finish_targets(&mut game);
    pay(&mut game);
    drain(&mut game);
    assert_eq!(game.object_view(enchantment).zone, Zone::Graveyard);
    assert_eq!(life(&game, player), 23);
}

#[test]
fn fancy_footwork_untaps_legal_targets_and_grows_through_a_stun_counter() {
    let player = Player::P1;
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Fancy Footwork", Zone::Hand);
    let own = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let opponent = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    for creature in [own, opponent] {
        game.state.objects[creature as usize].tapped = true;
    }
    game.add_counter(opponent, CounterKind::Stun, 1);
    announce(
        &mut game,
        spell,
        &[Target::Card(own), Target::Card(opponent)],
    );
    let mut stale = game.clone();
    stale.move_object(own, Zone::Hand);
    drain(&mut stale);
    assert_eq!(stats(&stale, opponent), (4, 4));
    drain(&mut game);
    assert!(!game.object_view(own).tapped);
    assert!(game.object_view(opponent).tapped);
    assert_eq!(game.counter_count(opponent, CounterKind::Stun), 0);
    assert_eq!(stats(&game, own), (4, 4));
    assert_eq!(stats(&game, opponent), (4, 4));
}

#[test]
fn fire_nation_warship_crews_then_leaves_a_clue_even_after_animation_expires() {
    let player = Player::P0;
    let mut game = scenario(player);
    let ship = put(&mut game, player, "Fire Nation Warship", Zone::Battlefield);
    let pilot = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    activate(&mut game, ship, 0);
    let object = game.object_ref(pilot);
    act(&mut game, Action::ChooseCrewer { object });
    act(&mut game, Action::FinishCrew);
    drain(&mut game);
    assert_eq!(stats(&game, ship), (4, 4));
    assert!(game.keywords(ship).contains(Keywords::REACH));
    assert!(game.object_view(pilot).tapped);
    game.state.step = Step::End;
    game.give_priority();
    pass_pair(&mut game);
    assert!(!game.is_creature(ship));
    game.move_object(ship, Zone::Graveyard);
    drain(&mut game);
    assert_eq!(tokens_of(&game, player, "Clue").len(), 1);
}

#[test]
fn gather_the_white_lotus_counts_a_nonbasic_plains_and_then_scries_two() {
    let player = Player::P0;
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Gather the White Lotus", Zone::Hand);
    put(&mut game, player, "Plains", Zone::Battlefield);
    put(&mut game, player, "Lush Portico", Zone::Battlefield);
    put(&mut game, player.other(), "Plains", Zone::Battlefield);
    cast(&mut game, spell, None);
    pass_pair(&mut game);
    assert_eq!(tokens_of(&game, player, "Ally").len(), 2);
    keep_top(&mut game, 2);
}

#[test]
fn ally_entries_reward_enthusiasts_and_reinforcements_have_flash() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let source = put(&mut game, player, "Avatar Enthusiasts", Zone::Hand);
    let warriors = put(&mut game, player, "Kyoshi Warriors", Zone::Hand);
    let reinforcements = put(&mut game, player, "Invasion Reinforcements", Zone::Hand);
    cast(&mut game, source, None);
    drain(&mut game);
    assert_eq!(game.counter_count(source, CounterKind::PlusOne), 0);
    cast(&mut game, warriors, None);
    drain(&mut game);
    assert_eq!(game.counter_count(source, CounterKind::PlusOne), 2);
    assert_eq!(tokens_of(&game, player, "Ally").len(), 1);

    game.state.active = opponent;
    game.state.priority = player;
    game.state.step = Step::BeginCombat;
    cast(&mut game, reinforcements, None);
    drain(&mut game);
    assert_eq!(game.counter_count(source, CounterKind::PlusOne), 4);
    assert_eq!(tokens_of(&game, player, "Ally").len(), 2);
}

#[test]
fn deserters_counter_the_target_and_piandao_privately_looks_for_three_types() {
    let player = Player::P1;
    let mut game = scenario(player);
    let deserters = put(&mut game, player, "Jeong Jeong's Deserters", Zone::Hand);
    let foreign = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    cast(&mut game, deserters, None);
    pass_pair(&mut game);
    target(&mut game, Target::Card(foreign));
    drain(&mut game);
    assert_eq!(game.counter_count(foreign, CounterKind::PlusOne), 1);

    let piandao = put(&mut game, player, "Master Piandao", Zone::Battlefield);
    assert!(game.keywords(piandao).contains(Keywords::FIRST_STRIKE));
    let eligible = [
        put(&mut game, player, "Kyoshi Warriors", Zone::Library),
        put(&mut game, player, "Short Bow", Zone::Library),
        put(&mut game, player, "Airbending Lesson", Zone::Library),
    ];
    let ineligible = put(&mut game, player, "Hill Giant", Zone::Library);
    declare(&mut game, piandao);
    pass_pair(&mut game);
    assert!(game.observe(player.other()).library_selection.is_none());
    reject(&mut game, Action::SelectLibrary { card: ineligible });
    for card in eligible {
        let mut branch = game.clone();
        act(&mut branch, Action::SelectLibrary { card });
        act(&mut branch, Action::FinishLibrarySelection);
        assert_eq!(branch.object_view(card).zone, Zone::Hand);
        assert!(
            branch
                .observe(player.other())
                .objects
                .iter()
                .any(|object| object.id == card)
        );
    }
    act(&mut game, Action::FinishLibrarySelection);
    assert!(
        eligible
            .iter()
            .all(|&card| game.object_view(card).zone == Zone::Library)
    );
}

#[test]
fn meteor_sword_destroys_any_permanent_and_equips_for_three() {
    let player = Player::P0;
    let mut game = scenario(player);
    let sword = put(&mut game, player, "Meteor Sword", Zone::Hand);
    let bearer = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let land = put(&mut game, player.other(), "Forest", Zone::Battlefield);
    cast(&mut game, sword, None);
    pass_pair(&mut game);
    target(&mut game, Target::Card(land));
    drain(&mut game);
    assert_eq!(game.object_view(land).zone, Zone::Graveyard);
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 3];
    activate(&mut game, sword, 0);
    target(&mut game, Target::Card(bearer));
    pay(&mut game);
    drain(&mut game);
    assert_eq!(stats(&game, bearer), (5, 5));
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
}

#[test]
fn plains_reprint_and_rumble_arena_produce_mana_and_arena_has_vigilance() {
    let player = Player::P1;
    let mut game = scenario(player);
    let arena = put(&mut game, player, "Rumble Arena", Zone::Hand);
    let plains = put(&mut game, player, "Plains", Zone::Hand);
    let storm = put(&mut game, player, "Sandbenders' Storm", Zone::Hand);
    let mut basic = game.clone();
    act(&mut basic, Action::PlayLand { card: plains });
    basic.state.players[player.index()].mana = [0; 6];
    act(&mut basic, Action::TapMana { card: plains });
    assert_eq!(basic.state.players[player.index()].mana[0], 1);

    act(&mut game, Action::PlayLand { card: arena });
    pass_pair(&mut game);
    keep_top(&mut game, 1);
    let mut filter = game.clone();
    filter.state.players[player.index()].mana = [0, 0, 0, 0, 0, 1];
    tap_mana(
        &mut filter,
        arena,
        1,
        [0, 0, 0, 0, 0, 1],
        [0, 1, 0, 0, 0, 0],
    );
    assert_eq!(filter.state.players[player.index()].mana[1], 1);

    act(&mut game, Action::AnnounceCast { card: storm });
    act(&mut game, Action::ChooseMode { mode: 1 });
    target(&mut game, Target::Card(arena));
    pay(&mut game);
    drain(&mut game);
    declare(&mut game, arena);
    assert!(!game.object_view(arena).tapped);
    game.state.players[player.index()].mana = [0; 6];
    tap_mana(&mut game, arena, 0, [0; 6], [0, 0, 0, 0, 0, 1]);
    assert_eq!(game.state.players[player.index()].mana[5], 1);
}

#[test]
fn united_front_includes_created_allies_and_yip_yip_only_grants_flying_to_an_ally() {
    let player = Player::P0;
    let mut game = scenario(player);
    let front = put(&mut game, player, "United Front", Zone::Hand);
    let yip = put(&mut game, player, "Yip Yip!", Zone::Hand);
    let bear = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    act(&mut game, Action::AnnounceCast { card: front });
    act(&mut game, Action::ChooseX { value: 2 });
    pay(&mut game);
    drain(&mut game);
    assert_eq!(stats(&game, bear), (3, 3));
    let allies = tokens_of(&game, player, "Ally");
    assert_eq!(allies.len(), 2);
    assert!(
        allies
            .iter()
            .all(|ally| (ally.power, ally.toughness) == (2, 2))
    );

    let mut nonally = game.clone();
    cast(&mut nonally, yip, Some(Target::Card(bear)));
    drain(&mut nonally);
    assert_eq!(stats(&nonally, bear), (5, 5));
    assert!(!nonally.keywords(bear).contains(Keywords::FLYING));

    let ally = allies[0].id;
    cast(&mut game, yip, Some(Target::Card(ally)));
    drain(&mut game);
    assert_eq!(stats(&game, ally), (4, 4));
    assert!(game.keywords(ally).contains(Keywords::FLYING));
}

#[test]
fn glider_kids_scries_on_entry_and_water_tribe_captain_buffs_current_creatures() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let glider = put(&mut game, player, "Glider Kids", Zone::Hand);
    let captain = put(&mut game, player, "Water Tribe Captain", Zone::Battlefield);
    let foe = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    cast(&mut game, glider, None);
    pass_pair(&mut game);
    pass_pair(&mut game);
    assert!(game.keywords(glider).contains(Keywords::FLYING));
    keep_top(&mut game, 1);

    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 5];
    game.state.objects[captain as usize].tapped = true;
    activate(&mut game, captain, 0);
    pay(&mut game);
    drain(&mut game);
    assert_eq!(stats(&game, captain), (4, 4));
    assert_eq!(stats(&game, glider), (3, 4));
    assert_eq!(stats(&game, foe), (2, 2));
    let late = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    assert_eq!(stats(&game, late), (2, 2));
}
