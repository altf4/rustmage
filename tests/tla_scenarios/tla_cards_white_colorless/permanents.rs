use super::*;

fn attack_with(game: &mut Game, attackers: &[CardId]) {
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    for &card in attackers {
        act(game, Action::Attack { card });
    }
    act(game, Action::FinishAttackers);
}

fn mana(game: &mut Game, card: CardId, ability: u16, production: Mana) {
    let direct = Action::ActivateMana {
        card,
        ability,
        counter_payment: Counters::default(),
        payment: [0; 6],
        production,
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
        if game
            .legal_actions()
            .iter()
            .any(|action| matches!(action, Action::PayCosts { .. }))
        {
            pay(game);
        }
    }
}

fn keep_top(game: &mut Game, player: Player) {
    let card = game.top_library_card(player).unwrap();
    act(
        game,
        Action::ArrangeLibrary {
            card,
            destination: LibraryDestination::Top,
        },
    );
}

fn next_upkeep(game: &mut Game) {
    game.state.step = Step::End;
    game.give_priority();
    pass_pair(game);
    assert_eq!(game.state.step, Step::Upkeep);
}

#[test]
fn waterskin_untaps_in_the_other_untap_step_and_can_produce_any_color() {
    let player = Player::P0;
    let mut game = scenario(player);
    let skin = put(&mut game, player, "Bender's Waterskin", Zone::Hand);
    cast(&mut game, skin, None);
    drain(&mut game);
    game.state.players[player.index()].mana = [0; 6];
    mana(&mut game, skin, 0, [0, 1, 0, 0, 0, 0]);
    assert_eq!(game.state.players[player.index()].mana[1], 1);
    assert!(game.object_view(skin).tapped);

    let mut stunned = game.clone();
    stunned.add_counter(skin, CounterKind::Stun, 1);
    next_upkeep(&mut stunned);
    assert!(stunned.object_view(skin).tapped);
    assert_eq!(stunned.counter_count(skin, CounterKind::Stun), 0);

    next_upkeep(&mut game);
    assert_eq!(game.state.active, player.other());
    assert!(!game.object_view(skin).tapped);
}

#[test]
fn compassionate_healer_triggers_on_its_actual_tap() {
    let player = Player::P1;
    let mut game = scenario(player);
    let healer = put(&mut game, player, "Compassionate Healer", Zone::Battlefield);
    attack_with(&mut game, &[healer]);
    assert!(game.object_view(healer).tapped);
    pass_pair(&mut game);
    assert_eq!(life(&game, player), 21);
    assert_eq!(
        game.observe(player)
            .library_choice
            .as_ref()
            .map(|choice| choice.remaining.len()),
        Some(1)
    );
    keep_top(&mut game, player);
}

#[test]
fn hakoda_privately_casts_allies_from_the_top_and_protects_the_current_group() {
    let player = Player::P0;
    let other = player.other();
    let mut game = scenario(player);
    let hakoda = put(
        &mut game,
        player,
        "Hakoda, Selfless Commander",
        Zone::Battlefield,
    );
    let bear = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let foe = put(&mut game, other, "Bear Cub", Zone::Battlefield);
    let nonally = put(&mut game, player, "Grizzly Bears", Zone::Library);
    let ally = put(&mut game, player, "Water Tribe Captain", Zone::Library);
    assert!(game.keywords(hakoda).contains(Keywords::VIGILANCE));
    assert_eq!(
        game.observe(player).known_library_top[player.index()]
            .as_ref()
            .unwrap()
            .object
            .card,
        ally
    );
    assert!(game.observe(other).known_library_top[player.index()].is_none());
    act(&mut game, Action::AnnounceCast { card: ally });
    pay(&mut game);
    drain(&mut game);
    assert_eq!(game.object_view(ally).zone, Zone::Battlefield);
    assert_eq!(
        game.observe(player).known_library_top[player.index()]
            .as_ref()
            .unwrap()
            .object
            .card,
        nonally
    );
    reject(&mut game, Action::AnnounceCast { card: nonally });

    activate(&mut game, hakoda, 0);
    if game
        .legal_actions()
        .iter()
        .any(|action| matches!(action, Action::PayCosts { .. }))
    {
        pay(&mut game);
    }
    assert_eq!(game.object_view(hakoda).zone, Zone::Graveyard);
    drain(&mut game);
    assert_eq!(stats(&game, bear), (2, 7));
    assert!(game.keywords(bear).contains(Keywords::INDESTRUCTIBLE));
    assert_eq!(stats(&game, foe), (2, 2));
    let later = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    assert_eq!(stats(&game, later), (2, 2));
}

#[test]
fn tea_shop_and_hideout_spend_mana_only_on_their_printed_categories() {
    let player = Player::P0;
    let mut game = scenario(player);
    let shop = put(
        &mut game,
        player,
        "Jasmine Dragon Tea Shop",
        Zone::Battlefield,
    );
    let hideout = put(&mut game, player, "White Lotus Hideout", Zone::Battlefield);
    let captain = put(&mut game, player, "Water Tribe Captain", Zone::Battlefield);
    let bow = put(&mut game, player, "Short Bow", Zone::Battlefield);
    let lesson = put(&mut game, player, "Airbending Lesson", Zone::Hand);
    let guard = put(&mut game, player, "Water Tribe Captain", Zone::Hand);
    let bear = put(&mut game, player, "Bear Cub", Zone::Battlefield);

    let mut colorless = game.clone();
    colorless.state.players[player.index()].mana = [0; 6];
    mana(&mut colorless, shop, 0, [0, 0, 0, 0, 0, 1]);
    mana(&mut colorless, hideout, 0, [0, 0, 0, 0, 0, 1]);
    assert_eq!(colorless.state.players[player.index()].mana[5], 2);

    let mut ally = game.clone();
    ally.state.players[player.index()].mana = [0, 0, 0, 0, 0, 2];
    mana(&mut ally, shop, 1, [1, 0, 0, 0, 0, 0]);
    cast(&mut ally, guard, None);
    drain(&mut ally);
    assert_eq!(ally.object_view(guard).zone, Zone::Battlefield);
    assert_eq!(ally.state.players[player.index()].mana, [0; 6]);

    let mut ally_ability = game.clone();
    ally_ability.state.players[player.index()].mana = [0, 0, 0, 0, 0, 4];
    mana(&mut ally_ability, shop, 1, [1, 0, 0, 0, 0, 0]);
    activate(&mut ally_ability, captain, 0);
    pay(&mut ally_ability);
    drain(&mut ally_ability);
    assert_eq!(stats(&ally_ability, captain), (4, 4));
    assert_eq!(ally_ability.state.players[player.index()].mana, [0; 6]);

    let mut lesson_branch = game.clone();
    lesson_branch.state.players[player.index()].mana = [0, 0, 0, 0, 0, 2];
    mana(&mut lesson_branch, hideout, 1, [1, 0, 0, 0, 0, 0]);
    cast(&mut lesson_branch, lesson, Some(Target::Card(bear)));
    drain(&mut lesson_branch);
    assert_eq!(lesson_branch.object_view(bear).zone, Zone::Exile);

    let mut denied = game.clone();
    denied.state.players[player.index()].mana = [0; 6];
    mana(&mut denied, shop, 1, [1, 0, 0, 0, 0, 0]);
    mana(&mut denied, hideout, 1, [1, 0, 0, 0, 0, 0]);
    reject(
        &mut denied,
        Action::AnnounceActivate {
            card: bow,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    reject(
        &mut denied,
        Action::AnnounceActivate {
            card: captain,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );

    let mut tokens = game.clone();
    tokens.state.players[player.index()].mana = [0, 0, 0, 0, 0, 5];
    activate(&mut tokens, shop, 2);
    pay(&mut tokens);
    drain(&mut tokens);
    assert_eq!(tokens_of(&tokens, player, "Ally").len(), 1);
    assert!(tokens.object_view(shop).tapped);

    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 1];
    act(
        &mut game,
        Action::ActivateMana {
            card: hideout,
            ability: 2,
            payment: [0, 0, 0, 0, 0, 1],
            counter_payment: Counters::default(),
            production: [0, 0, 1, 0, 0, 0],
        },
    );
    assert_eq!(game.state.players[player.index()].mana, [0, 0, 1, 0, 0, 0]);
}

#[test]
fn kyoshi_battle_fan_attaches_to_its_entrant_and_can_equip_another_creature() {
    let player = Player::P1;
    let mut game = scenario(player);
    let fan = put(&mut game, player, "Kyoshi Battle Fan", Zone::Hand);
    let bear = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    cast(&mut game, fan, None);
    drain(&mut game);
    let ally = tokens_of(&game, player, "Ally")[0].reference;
    assert_eq!(game.object_view(fan).attached_to, Some(ally));
    assert_eq!(stats(&game, ally.card), (2, 1));

    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 2];
    activate(&mut game, fan, 0);
    target(&mut game, Target::Card(bear));
    pay(&mut game);
    drain(&mut game);
    assert_eq!(
        game.object_view(fan).attached_to,
        Some(game.object_ref(bear))
    );
    assert_eq!(stats(&game, ally.card), (1, 1));
    assert_eq!(stats(&game, bear), (3, 2));
}

#[test]
fn friendly_momo_discounts_the_first_other_flying_cast_and_grows_on_entry() {
    let player = Player::P0;
    let mut game = scenario(player);
    let momo = put(&mut game, player, "Momo, Friendly Flier", Zone::Hand);
    let glider = put(&mut game, player, "Glider Kids", Zone::Hand);
    let next = put(&mut game, player, "Glider Kids", Zone::Hand);
    cast(&mut game, momo, None);
    drain(&mut game);
    assert_eq!(stats(&game, momo), (1, 1));
    assert!(game.keywords(momo).contains(Keywords::FLYING));

    game.state.players[player.index()].mana = [1, 0, 0, 0, 0, 1];
    cast(&mut game, glider, None);
    pass_pair(&mut game);
    while matches!(game.state.pending, Pending::OrderTriggers { .. }) {
        choose(&mut game, |action| {
            matches!(action, Action::PutTrigger { .. })
        });
    }
    for _ in 0..8 {
        if game.state.stack.is_empty() && game.state.resolving.is_none() {
            break;
        }
        if game
            .legal_actions()
            .iter()
            .any(|action| matches!(action, Action::ArrangeLibrary { .. }))
        {
            keep_top(&mut game, player);
        } else {
            act(&mut game, Action::Pass);
        }
    }
    assert_eq!(stats(&game, momo), (2, 2));
    assert_eq!(game.object_view(glider).zone, Zone::Battlefield);
    game.state.players[player.index()].mana = [1, 0, 0, 0, 0, 1];
    reject(&mut game, Action::AnnounceCast { card: next });
}

#[test]
fn playful_momo_chooses_food_a_counter_or_scry_after_leaving() {
    let player = Player::P1;
    for (mode, zone) in [(0, Zone::Hand), (1, Zone::Graveyard), (2, Zone::Exile)] {
        let mut game = scenario(player);
        let momo = put(&mut game, player, "Momo, Playful Pet", Zone::Battlefield);
        let bear = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        assert!(
            game.keywords(momo)
                .contains(Keywords::FLYING.union(Keywords::VIGILANCE))
        );
        game.move_object(momo, zone);
        game.prepare_triggers();
        act(&mut game, Action::ChooseMode { mode });
        if mode == 1 {
            target(&mut game, Target::Card(bear));
        }
        pass_pair(&mut game);
        match mode {
            0 => assert_eq!(tokens_of(&game, player, "Food").len(), 1),
            1 => assert_eq!(game.counter_count(bear, CounterKind::PlusOne), 1),
            2 => {
                let top = game.state.players[player.index()]
                    .library
                    .iter()
                    .rev()
                    .copied()
                    .take(2)
                    .collect::<Vec<_>>();
                for card in top {
                    act(
                        &mut game,
                        Action::ArrangeLibrary {
                            card,
                            destination: LibraryDestination::Top,
                        },
                    );
                }
            }
            _ => unreachable!(),
        }
    }
}

#[test]
fn rabaroo_landfall_and_plainscycling_follow_the_actual_land_subtype() {
    let player = Player::P0;
    let mut game = scenario(player);
    let troop = put(&mut game, player, "Rabaroo Troop", Zone::Battlefield);
    let cycling = put(&mut game, player, "Rabaroo Troop", Zone::Hand);
    let nonbasic = put(&mut game, player, "Lush Portico", Zone::Library);
    let forest = put(&mut game, player, "Forest", Zone::Hand);
    act(&mut game, Action::PlayLand { card: forest });
    drain(&mut game);
    assert_eq!(life(&game, player), 21);
    assert!(game.keywords(troop).contains(Keywords::FLYING));

    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 2];
    activate(&mut game, cycling, 0);
    pay(&mut game);
    assert_eq!(game.object_view(cycling).zone, Zone::Graveyard);
    pass_pair(&mut game);
    act(&mut game, Action::ChooseSearch { card: nonbasic });
    if game.legal_actions().contains(&Action::FinishSearch) {
        act(&mut game, Action::FinishSearch);
    }
    assert_eq!(game.object_view(nonbasic).zone, Zone::Hand);
}

#[test]
fn secret_tunnel_targets_two_friendly_creatures_with_a_live_shared_type() {
    let player = Player::P1;
    let mut game = scenario(player);
    let tunnel = put(&mut game, player, "Secret Tunnel", Zone::Battlefield);
    let first = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let second = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let giant = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let foreign = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    assert!(game.has_subtype(tunnel, "Cave"));
    assert!(game.keywords(tunnel).contains(Keywords::CANT_BE_BLOCKED));
    let mut colorless = game.clone();
    colorless.state.players[player.index()].mana = [0; 6];
    mana(&mut colorless, tunnel, 0, [0, 0, 0, 0, 0, 1]);
    assert_eq!(colorless.state.players[player.index()].mana[5], 1);

    activate(&mut game, tunnel, 1);
    target(&mut game, Target::Card(first));
    for card in [giant, foreign] {
        reject(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(card),
            },
        );
    }
    target(&mut game, Target::Card(second));
    pay(&mut game);
    drain(&mut game);
    for card in [first, second] {
        assert!(game.keywords(card).contains(Keywords::CANT_BE_BLOCKED));
    }
}

#[test]
fn voyager_draws_on_its_second_resolution_and_temple_scales_with_shrines() {
    let player = Player::P0;
    let mut game = scenario(player);
    let voyager = put(&mut game, player, "South Pole Voyager", Zone::Hand);
    let ally = put(&mut game, player, "Water Tribe Captain", Zone::Hand);
    let another = put(&mut game, player, "Water Tribe Captain", Zone::Hand);
    cast(&mut game, voyager, None);
    drain(&mut game);
    assert_eq!(life(&game, player), 21);
    assert_eq!(draw_count(&game, player), 0);
    cast(&mut game, ally, None);
    drain(&mut game);
    assert_eq!(life(&game, player), 22);
    assert_eq!(draw_count(&game, player), 1);
    cast(&mut game, another, None);
    drain(&mut game);
    assert_eq!(life(&game, player), 23);
    assert_eq!(draw_count(&game, player), 1);

    let shrine_card = Card::from_name("Crescent Island Temple").unwrap();
    let existing = spawn(&mut game, player, shrine_card, Zone::Battlefield);
    let temple = put(&mut game, player, "Southern Air Temple", Zone::Hand);
    let shrine = spawn(
        &mut game,
        player,
        Card::from_name("Northern Air Temple").unwrap(),
        Zone::Exile,
    );
    game.state.players[player.index()].mana = [8; 6];
    cast(&mut game, temple, None);
    drain(&mut game);
    assert!(game.has_subtype(existing, "Shrine"));
    for card in [voyager, ally, another] {
        assert_eq!(game.counter_count(card, CounterKind::PlusOne), 2);
    }
    game.move_object(shrine, Zone::Battlefield);
    drain(&mut game);
    for card in [voyager, ally, another] {
        assert_eq!(game.counter_count(card, CounterKind::PlusOne), 3);
    }
}

#[test]
fn suki_boosts_other_creatures_and_triggers_once_for_an_own_turn_departure() {
    let player = Player::P0;
    let other = player.other();
    let mut game = scenario(player);
    let suki = put(
        &mut game,
        player,
        "Suki, Courageous Rescuer",
        Zone::Battlefield,
    );
    let bear = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let foe = put(&mut game, other, "Bear Cub", Zone::Battlefield);
    let artifact = put(&mut game, player, "Short Bow", Zone::Battlefield);
    let later = put(&mut game, player, "Short Bow", Zone::Battlefield);
    assert_eq!(stats(&game, suki), (2, 4));
    assert_eq!(stats(&game, bear), (3, 2));
    assert_eq!(stats(&game, foe), (2, 2));

    let mut off_turn = game.clone();
    off_turn.state.active = other;
    off_turn.state.priority = other;
    off_turn.move_object(artifact, Zone::Exile);
    drain(&mut off_turn);
    assert!(tokens_of(&off_turn, player, "Ally").is_empty());

    game.move_object(artifact, Zone::Hand);
    game.move_object(later, Zone::Exile);
    drain(&mut game);
    let tokens = tokens_of(&game, player, "Ally");
    assert_eq!(tokens.len(), 1);
    assert_eq!((tokens[0].power, tokens[0].toughness), (2, 1));
}

#[test]
fn team_avatar_pumps_a_lone_attacker_and_uses_live_count_for_its_hand_ability() {
    let player = Player::P0;
    let mut game = scenario(player);
    let team = put(&mut game, player, "Team Avatar", Zone::Battlefield);
    let hand = put(&mut game, player, "Team Avatar", Zone::Hand);
    let bear = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let extra = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let enemy = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    attack_with(&mut game, &[bear]);
    drain(&mut game);
    assert_eq!(stats(&game, bear), (4, 4));
    assert_eq!(stats(&game, extra), (2, 2));
    reject(
        &mut game,
        Action::AnnounceActivate {
            card: team,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );

    activate(&mut game, hand, 0);
    target(&mut game, Target::Card(enemy));
    pay(&mut game);
    assert_eq!(game.object_view(hand).zone, Zone::Graveyard);
    put(&mut game, player, "Bear Cub", Zone::Battlefield);
    drain(&mut game);
    assert_eq!(game.object_view(enemy).zone, Zone::Graveyard);
}

#[test]
fn the_walls_protect_other_friendly_permanents_and_have_defender() {
    let player = Player::P0;
    let mut game = scenario(player);
    let wall = put(
        &mut game,
        player,
        "The Walls of Ba Sing Se",
        Zone::Battlefield,
    );
    let land = put(&mut game, player, "Forest", Zone::Battlefield);
    let enemy = put(&mut game, player.other(), "Forest", Zone::Battlefield);
    assert_eq!(stats(&game, wall), (0, 30));
    assert!(game.keywords(wall).contains(Keywords::DEFENDER));
    assert!(!game.keywords(wall).contains(Keywords::INDESTRUCTIBLE));
    assert!(game.keywords(land).contains(Keywords::INDESTRUCTIBLE));
    assert!(!game.keywords(enemy).contains(Keywords::INDESTRUCTIBLE));
    game.move_object(wall, Zone::Exile);
    assert!(!game.keywords(land).contains(Keywords::INDESTRUCTIBLE));
}

#[test]
fn trusty_boomerang_equips_and_grants_a_tap_ability_to_its_bearer() {
    let player = Player::P1;
    let mut game = scenario(player);
    let equipment = put(&mut game, player, "Trusty Boomerang", Zone::Battlefield);
    let bearer = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let foe = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    activate(&mut game, equipment, 1);
    target(&mut game, Target::Card(bearer));
    pay(&mut game);
    drain(&mut game);
    let grantor = game.object_ref(equipment);
    assert_eq!(
        game.object_view(equipment).attached_to,
        Some(game.object_ref(bearer))
    );
    act(
        &mut game,
        Action::AnnounceAttachedAbility {
            grantor,
            ability: 0,
        },
    );
    target(&mut game, Target::Card(foe));
    pay(&mut game);
    assert!(game.object_view(bearer).tapped);
    assert!(!game.object_view(equipment).tapped);
    drain(&mut game);
    assert!(game.object_view(foe).tapped);
    assert_eq!(game.object_view(equipment).zone, Zone::Hand);
}

#[test]
fn water_tribe_rallier_waterbends_and_privately_selects_a_small_creature() {
    let player = Player::P0;
    let mut game = scenario(player);
    let rallier = put(&mut game, player, "Water Tribe Rallier", Zone::Battlefield);
    let bow = put(&mut game, player, "Short Bow", Zone::Battlefield);
    let small = put(&mut game, player, "Hill Giant", Zone::Library);
    let big = put(&mut game, player, "Gigantosaurus", Zone::Library);
    let noncreature = put(&mut game, player, "Short Bow", Zone::Library);
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 3];
    activate(&mut game, rallier, 0);
    act(&mut game, Action::BeginWaterbend);
    for id in [rallier, bow] {
        let object = game.object_ref(id);
        act(&mut game, Action::ChooseWaterbender { object });
    }
    pay(&mut game);
    assert!(game.object_view(rallier).tapped && game.object_view(bow).tapped);
    pass_pair(&mut game);
    assert!(game.observe(player.other()).actions.is_empty());
    for card in [big, noncreature] {
        reject(&mut game, Action::SelectLibrary { card });
    }
    act(&mut game, Action::SelectLibrary { card: small });
    act(&mut game, Action::FinishLibrarySelection);
    assert_eq!(game.object_view(small).zone, Zone::Hand);
    assert!(
        game.observe(player.other())
            .objects
            .iter()
            .any(|object| object.id == small)
    );
}

#[test]
fn white_lotus_tile_enters_tapped_and_produces_the_largest_one_color_group() {
    let player = Player::P1;
    let mut game = scenario(player);
    let tile = put(&mut game, player, "White Lotus Tile", Zone::Hand);
    put(&mut game, player, "Bear Cub", Zone::Battlefield);
    put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    put(&mut game, player, "Three Tree Mascot", Zone::Battlefield);
    put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    cast(&mut game, tile, None);
    drain(&mut game);
    assert!(game.object_view(tile).tapped);

    game.state.objects[tile as usize].tapped = false;
    game.state.players[player.index()].mana = [0; 6];
    reject(
        &mut game,
        Action::ActivateMana {
            card: tile,
            ability: 0,
            payment: [0; 6],
            counter_payment: Counters::default(),
            production: [2, 1, 0, 0, 0, 0],
        },
    );
    mana(&mut game, tile, 0, [0, 0, 0, 0, 3, 0]);
    assert_eq!(game.state.players[player.index()].mana, [0, 0, 0, 0, 3, 0]);
}
