use super::*;

#[test]
fn jameson_suspects_a_creature_and_creates_treasure_for_each_menacing_attacker() {
    let player = Player::P0;
    let mut game = scenario(player);
    let suspect = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let menace = put(
        &mut game,
        player,
        "Prowler, Clawed Thief",
        Zone::Battlefield,
    );
    let plain = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let jameson = put(&mut game, player, "J. Jonah Jameson", Zone::Hand);
    cast(&mut game, jameson, None);
    pass_pair(&mut game);
    choose_target(&mut game, Target::Card(suspect));
    settle(&mut game);
    assert!(
        game.object_view(suspect)
            .keywords
            .contains(Keywords::MENACE.union(Keywords::CANT_BLOCK))
    );
    declare(&mut game, &[suspect, menace, plain]);
    order_triggers(&mut game);
    settle(&mut game);
    assert_eq!(token_refs(&game, Card(5302), player).len(), 2);
}

#[test]
fn jackal_copies_the_matching_spell_as_nonlegendary_then_gets_a_counter() {
    let player = Player::P0;
    let mut game = scenario(player);
    let jackal = put(
        &mut game,
        player,
        "Jackal, Genius Geneticist",
        Zone::Battlefield,
    );
    assert!(
        game.object_view(jackal)
            .keywords
            .contains(Keywords::TRAMPLE)
    );
    let giant = put(&mut game, player, "Hill Giant", Zone::Hand);
    cast(&mut game, giant, None);
    assert_eq!(game.state.stack.len(), 1);
    settle(&mut game);

    let aunt = put(&mut game, player, "Aunt May", Zone::Hand);
    cast(&mut game, aunt, None);
    assert_eq!(game.state.stack.len(), 2);
    let growth = put(&mut game, player, "Giant Growth", Zone::Hand);
    cast(&mut game, growth, Some(Target::Card(jackal)));
    pass_pair(&mut game);
    assert_eq!(stats(&game, jackal).0, 4);
    pass_pair(&mut game);
    assert!(game.state.stack.last().unwrap().is_copy);
    assert_eq!(game.counter_count(jackal, CounterKind::PlusOne), 1);
    settle(&mut game);
    assert!(game.object_view(aunt).is_legendary());
    let copies = token_refs(&game, game.object_view(aunt).card, player);
    assert_eq!(copies.len(), 1);
    assert!(!game.object_view(copies[0].card).is_legendary());
}

#[test]
fn kraven_draws_and_grows_once_for_each_tied_highest_power_opposing_death() {
    let player = Player::P0;
    for wrath in [false, true] {
        let mut game = scenario(player);
        let kraven = put(&mut game, player, "Kraven the Hunter", Zone::Battlefield);
        assert!(
            game.object_view(kraven)
                .keywords
                .contains(Keywords::TRAMPLE)
        );
        let greater = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
        let tied = put(
            &mut game,
            player.other(),
            "Grizzly Bears",
            Zone::Battlefield,
        );
        let lesser = put(
            &mut game,
            player.other(),
            "Grizzly Bears",
            Zone::Battlefield,
        );
        game.add_counter(tied, CounterKind::PlusOne, 1);
        if wrath {
            let card = put(&mut game, player, "Day of Judgment", Zone::Hand);
            cast(&mut game, card, None);
            pass_pair(&mut game);
            order_triggers(&mut game);
            settle(&mut game);
            assert_eq!(game.object_view(kraven).zone, Zone::Graveyard);
            assert_eq!(game.hand_count(player), 2);
            assert_eq!(game.counter_count(kraven, CounterKind::PlusOne), 0);
        } else {
            for (victim, count) in [(lesser, 0), (greater, 1)] {
                let card = put(&mut game, player, "Murder", Zone::Hand);
                cast(&mut game, card, Some(Target::Card(victim)));
                settle(&mut game);
                assert_eq!(game.hand_count(player), count);
                assert_eq!(
                    game.counter_count(kraven, CounterKind::PlusOne),
                    count as u64
                );
            }
        }
    }
}

#[test]
fn kraven_s_last_hunt_mills_then_damages_pumps_and_recovers() {
    let player = Player::P1;
    let mut game = scenario(player);
    let ally = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let enemy = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    game.add_counter(enemy, CounterKind::PlusOne, 2);
    let recovered = put(&mut game, player, "Kraven, Proud Predator", Zone::Graveyard);
    let their_card = put(&mut game, player.other(), "Grizzly Bears", Zone::Graveyard);
    let saga = put(&mut game, player, "Kraven's Last Hunt", Zone::Hand);
    let library_before = game.state.players[player.index()].library.len();
    cast(&mut game, saga, None);
    pass_pair(&mut game);
    pass_pair(&mut game);
    assert_eq!(
        game.state.players[player.index()].library.len(),
        library_before - 5
    );
    choose_target(&mut game, Target::Card(enemy));
    settle(&mut game);
    assert_eq!(game.object_view(enemy).damage, 4);

    let turn = game.state.turn;
    advance_until(&mut game, |g| {
        g.state.active == player && g.state.turn > turn && g.state.step == Step::Main1
    });
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(enemy),
        },
    );
    choose_target(&mut game, Target::Card(ally));
    settle(&mut game);
    assert_eq!(stats(&game, ally), (5, 5));
    let turn = game.state.turn;
    advance_until(&mut game, |g| {
        g.state.active == player && g.state.turn > turn && g.state.step == Step::Main1
    });
    assert_eq!(stats(&game, ally), (3, 3));
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(their_card),
        },
    );
    choose_target(&mut game, Target::Card(recovered));
    settle(&mut game);
    assert_eq!(game.object_view(recovered).zone, Zone::Hand);
    assert_eq!(game.object_view(saga).zone, Zone::Graveyard);
}

#[test]
fn living_brain_animates_and_untaps_only_a_controlled_non_equipment_artifact() {
    let player = Player::P0;
    let mut game = scenario(player);
    put(
        &mut game,
        player,
        "Living Brain, Mechanical Marvel",
        Zone::Battlefield,
    );
    let archive = put(&mut game, player, "Hedron Archive", Zone::Battlefield);
    let equipment = put(&mut game, player, "Swiftfoot Boots", Zone::Battlefield);
    let opposing = put(
        &mut game,
        player.other(),
        "Hedron Archive",
        Zone::Battlefield,
    );
    game.state.objects[archive as usize].tapped = true;
    game.add_counter(archive, CounterKind::PlusOne, 1);
    game.begin_step(Step::BeginCombat);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(equipment),
        },
    );
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(opposing),
        },
    );
    choose_target(&mut game, Target::Card(archive));
    settle(&mut game);
    assert_eq!(game.object_view(archive).kinds, 1 | 16);
    assert_eq!(stats(&game, archive), (4, 4));
    assert!(!game.object_view(archive).tapped);
    advance_until(&mut game, |g| {
        g.state.active != player && g.state.step == Step::Main1
    });
    assert!(!game.is_creature(archive));
    assert_eq!(game.counter_count(archive, CounterKind::PlusOne), 1);
}

#[test]
fn lizard_replaces_the_target_s_form_until_it_changes_incarnation() {
    let player = Player::P0;
    let mut game = scenario(player);
    let target = put(
        &mut game,
        player,
        "Iron Spider, Stark Upgrade",
        Zone::Battlefield,
    );
    game.add_counter(target, CounterKind::PlusOne, 1);
    let lizard = put(&mut game, player, "Lizard, Connors's Curse", Zone::Hand);
    cast(&mut game, lizard, None);
    pass_pair(&mut game);
    assert!(
        game.object_view(lizard)
            .keywords
            .contains(Keywords::TRAMPLE)
    );
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(lizard),
        },
    );
    choose_target(&mut game, Target::Card(target));
    settle(&mut game);
    let old = game.object_ref(target);
    let form = game.object_view(target);
    assert_eq!(form.kinds, 1);
    assert_eq!(form.colors, 16);
    assert!(form.has_subtype("Lizard"));
    assert!(!form.has_subtype("Spider") && !form.has_subtype("Hero"));
    assert!(form.is_legendary());
    assert!(!form.printed_abilities_available());
    assert_eq!(stats(&game, target), (5, 5));

    let bounce = put(&mut game, player, "Unsummon", Zone::Hand);
    cast(&mut game, bounce, Some(Target::Card(lizard)));
    settle(&mut game);
    assert_eq!(game.object_view(target), form);
    let bounce = put(&mut game, player, "Unsummon", Zone::Hand);
    cast(&mut game, bounce, Some(Target::Card(target)));
    settle(&mut game);
    cast(&mut game, target, None);
    settle(&mut game);
    let new = game.object_view(target);
    assert_ne!(new.reference, old);
    assert_eq!(new.kinds, 1 | 16);
    assert!(new.keywords.contains(Keywords::VIGILANCE));
    assert!(new.has_subtype("Spider") && !new.has_subtype("Lizard"));
    assert_eq!(stats(&game, target), (2, 3));
}

#[test]
fn molten_man_counts_all_mountains_but_fetches_only_a_basic_and_costs_a_land() {
    let player = Player::P0;
    let mut game = scenario(player);
    put(&mut game, player, "Mountain", Zone::Battlefield);
    put(&mut game, player, "Stomping Ground", Zone::Battlefield);
    let sacrifice = put(&mut game, player, "Forest", Zone::Battlefield);
    let basic = put(&mut game, player, "Mountain", Zone::Library);
    let nonbasic = put(&mut game, player, "Stomping Ground", Zone::Library);
    let molten = put(
        &mut game,
        player,
        "Molten Man, Inferno Incarnate",
        Zone::Hand,
    );
    cast(&mut game, molten, None);
    pass_pair(&mut game);
    assert_eq!(stats(&game, molten), (2, 2));
    pass_pair(&mut game);
    reject(&mut game, Action::ChooseSearch { card: nonbasic });
    act(&mut game, Action::ChooseSearch { card: basic });
    if game.legal_actions().contains(&Action::FinishSearch) {
        act(&mut game, Action::FinishSearch);
    }
    assert!(game.object_view(basic).tapped);
    assert_eq!(game.object_view(basic).zone, Zone::Battlefield);
    assert_eq!(stats(&game, molten), (3, 3));

    let bounce = put(&mut game, player, "Unsummon", Zone::Hand);
    cast(&mut game, bounce, Some(Target::Card(molten)));
    pass_pair(&mut game);
    pass_pair(&mut game);
    let object = game.object_ref(sacrifice);
    act(&mut game, Action::Sacrifice { object });
    settle(&mut game);
    assert_eq!(game.object_view(sacrifice).zone, Zone::Graveyard);
    assert_eq!(game.object_view(molten).zone, Zone::Hand);
}
