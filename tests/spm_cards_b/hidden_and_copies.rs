use super::*;

#[test]
fn lady_octopus_counts_two_actual_draws_and_privately_casts_a_small_artifact() {
    let player = Player::P0;
    let mut game = scenario(player);
    let lady = put(
        &mut game,
        player,
        "Lady Octopus, Inspired Inventor",
        Zone::Battlefield,
    );
    for _ in 0..2 {
        let draw = put(&mut game, player, "Divination", Zone::Hand);
        cast(&mut game, draw, None);
        pass_pair(&mut game);
        order_triggers(&mut game);
        settle(&mut game);
        assert_eq!(game.counter_count(lady, CounterKind::Ingenuity), 2);
    }
    let boots = put(&mut game, player, "Swiftfoot Boots", Zone::Hand);
    let archive = put(&mut game, player, "Hedron Archive", Zone::Hand);
    let bears = put(&mut game, player, "Grizzly Bears", Zone::Hand);
    game.state.players[player.index()].mana = [0; 6];
    announce_ability(&mut game, lady, 0);
    pay(&mut game);
    pass_pair(&mut game);
    assert!(game.object_view(lady).tapped);
    assert!(game.observe(player.other()).resolution_free_casts.is_none());
    assert_eq!(
        game.observe(player)
            .resolution_free_casts
            .unwrap()
            .remaining_mana_value,
        Some(2)
    );
    let free = |game: &Game, card| Action::CastFreeDuringResolution {
        object: game.object_ref(card),
        face: CastFace::Normal,
    };
    for card in [archive, bears] {
        let action = free(&game, card);
        reject(&mut game, action);
    }
    let mut declined = game.clone();
    act(&mut declined, Action::FinishResolutionCasts);
    assert_eq!(declined.object_view(boots).zone, Zone::Hand);
    let action = free(&game, boots);
    act(&mut game, action);
    pay(&mut game);
    assert_eq!(
        game.state.stack.last().unwrap().cast.unwrap().method,
        CastMethod::Free
    );
    if game
        .legal_actions()
        .contains(&Action::FinishResolutionCasts)
    {
        act(&mut game, Action::FinishResolutionCasts);
    }
    settle(&mut game);
    assert_eq!(game.object_view(boots).zone, Zone::Battlefield);
}

#[test]
fn madame_web_privately_looks_and_mills_to_enable_noncreature_and_spider_casts() {
    let player = Player::P1;
    let mut game = scenario(player);
    put(
        &mut game,
        player,
        "Madame Web, Clairvoyant",
        Zone::Battlefield,
    );
    let attacker = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let spider = put(&mut game, player, "Radioactive Spider", Zone::Library);
    let burn = put(&mut game, player, "Shock", Zone::Library);
    let covered = put(&mut game, player, "Grizzly Bears", Zone::Library);
    let top = game.object_ref(covered);
    assert_eq!(
        game.observe(player).known_library_top[player.index()]
            .unwrap()
            .object,
        top
    );
    assert!(game.observe(player.other()).known_library_top[player.index()].is_none());
    reject(&mut game, Action::AnnounceCast { card: covered });
    declare(&mut game, &[attacker]);
    pass_pair(&mut game);
    let mut declined = game.clone();
    act(&mut declined, Action::ChooseOptional { accept: false });
    assert_eq!(declined.object_view(covered).zone, Zone::Library);
    act(&mut game, Action::ChooseOptional { accept: true });
    assert_eq!(game.object_view(covered).zone, Zone::Graveyard);
    advance_until(&mut game, |g| g.state.step == Step::Main2);
    game.state.players[player.index()].mana = [0, 0, 0, 1, 1, 0];
    cast(&mut game, burn, Some(Target::Player(player.other())));
    assert_eq!(
        game.state.stack.last().unwrap().cast.unwrap().origin,
        Zone::Library
    );
    settle(&mut game);
    cast(&mut game, spider, None);
    settle(&mut game);
    assert_eq!(game.object_view(spider).zone, Zone::Battlefield);
    let land = *game.state.players[player.index()].library.last().unwrap();
    reject(&mut game, Action::PlayLand { card: land });
}

#[test]
fn parker_luck_targets_both_players_and_swaps_life_losses_without_drawing() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let enchantment = put(&mut game, player, "Parker Luck", Zone::Hand);
    let own = put(&mut game, player, "Hill Giant", Zone::Library);
    let enemy = put(&mut game, opponent, "Shock", Zone::Library);
    cast(&mut game, enchantment, None);
    settle(&mut game);
    game.begin_step(Step::End);
    choose_target(&mut game, Target::Player(opponent));
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(opponent),
        },
    );
    choose_target(&mut game, Target::Player(player));
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, 19);
    assert_eq!(game.state.players[opponent.index()].life, 16);
    assert_eq!(game.object_view(own).zone, Zone::Hand);
    assert_eq!(game.object_view(enemy).zone, Zone::Hand);
    assert!(
        game.state
            .players
            .iter()
            .all(|seat| seat.cards_drawn_this_turn == 0)
    );
}

#[test]
fn peter_s_camera_spends_film_and_copies_activated_or_triggered_abilities() {
    let player = Player::P0;
    for triggered in [false, true] {
        let mut game = scenario(player);
        let camera = put(&mut game, player, "Peter Parker's Camera", Zone::Hand);
        cast(&mut game, camera, None);
        settle(&mut game);
        assert_eq!(game.counter_count(camera, CounterKind::Film), 3);
        if triggered {
            let wrestler = put(&mut game, player, "Professional Wrestler", Zone::Hand);
            cast(&mut game, wrestler, None);
            pass_pair(&mut game);
        } else {
            let meower = put(&mut game, player, "Masked Meower", Zone::Battlefield);
            let discard = put(&mut game, player, "Island", Zone::Hand);
            announce_ability(&mut game, meower, 0);
            let object = game.object_ref(discard);
            act(&mut game, Action::ChooseAdditionalCost { object });
            pay(&mut game);
        }
        let original = game.state.stack.last().unwrap().id;
        let shock = put(&mut game, player, "Shock", Zone::Hand);
        cast(&mut game, shock, Some(Target::Player(player.other())));
        announce_ability(&mut game, camera, 0);
        reject(
            &mut game,
            Action::ChooseTarget {
                target: Target::Spell(shock),
            },
        );
        choose_target(&mut game, Target::Stack(original));
        pay(&mut game);
        assert!(game.object_view(camera).tapped);
        assert_eq!(game.counter_count(camera, CounterKind::Film), 2);
        pass_pair(&mut game);
        assert!(game.state.stack.last().unwrap().is_copy);
        settle(&mut game);
        if triggered {
            assert_eq!(token_refs(&game, Card(5302), player).len(), 2);
        } else {
            assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 2);
        }
    }
}

#[test]
fn pictures_reveal_zero_one_or_two_creatures_and_sacrifice_for_treasure() {
    let player = Player::P0;
    let mut game = scenario(player);
    let mut creatures = Vec::new();
    for name in ["Grizzly Bears", "Hill Giant", "Radioactive Spider"] {
        creatures.push(put(&mut game, player, name, Zone::Library));
    }
    let wrong = put(&mut game, player, "Forest", Zone::Library);
    let card = put(&mut game, player, "Pictures of Spider-Man", Zone::Hand);
    cast(&mut game, card, None);
    pass_pair(&mut game);
    pass_pair(&mut game);
    assert!(game.observe(player.other()).library_selection.is_none());
    reject(&mut game, Action::SelectLibrary { card: wrong });
    for count in 0..=2 {
        let mut branch = game.clone();
        for &candidate in &creatures[..count] {
            act(&mut branch, Action::SelectLibrary { card: candidate });
        }
        if branch
            .legal_actions()
            .contains(&Action::FinishLibrarySelection)
        {
            act(&mut branch, Action::FinishLibrarySelection);
        }
        settle(&mut branch);
        assert_eq!(branch.hand_count(player), count);
        for (i, &candidate) in creatures.iter().enumerate() {
            assert_eq!(
                branch.object_view(candidate).zone,
                if i < count { Zone::Hand } else { Zone::Library }
            );
        }
        announce_ability(&mut branch, card, 0);
        pay(&mut branch);
        assert_eq!(branch.object_view(card).zone, Zone::Graveyard);
        assert!(token_refs(&branch, Card(5302), player).is_empty());
        settle(&mut branch);
        assert_eq!(token_refs(&branch, Card(5302), player).len(), 1);
    }
}
