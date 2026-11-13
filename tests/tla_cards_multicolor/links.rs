use super::*;

fn exile(game: &Game, card: CardId) -> Action {
    Action::SelectResolutionExile {
        object: game.object_ref(card),
    }
}

fn finish_effect(game: &mut Game) {
    if game.legal_actions().contains(&Action::FinishEffectChoice) {
        act(game, Action::FinishEffectChoice);
    }
}

#[test]
fn azula_exiles_exact_opposing_choices_and_casts_with_flash_and_flexible_mana() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let azula = put(&mut game, player, "Azula, Cunning Usurper", Zone::Hand);
    let creature = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let own_creature = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let sorcery = put(&mut game, opponent, "Divination", Zone::Graveyard);
    let land = put(&mut game, opponent, "Forest", Zone::Graveyard);
    cast(&mut game, azula, None);
    pass_pair(&mut game);
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(opponent),
        },
    );
    pass_pair(&mut game);
    assert_eq!(game.actor(), Some(opponent));
    let own = exile(&game, own_creature);
    reject(&mut game, own);
    let first = exile(&game, creature);
    act(&mut game, first);
    finish_effect(&mut game);
    assert_eq!(game.object_view(creature).zone, Zone::Exile);
    let not_eligible = exile(&game, land);
    reject(&mut game, not_eligible);
    let second = exile(&game, sorcery);
    act(&mut game, second);
    finish_effect(&mut game);
    assert_eq!(game.object_view(sorcery).zone, Zone::Exile);
    let cast_it = Action::AnnounceLinkedPaid {
        object: game.object_ref(sorcery),
        source: game.object_ref(azula),
        face: CastFace::Normal,
        flash_surcharge: false,
    };
    let mut off_turn = game.clone();
    off_turn.state.active = opponent;
    reject(&mut off_turn, cast_it.clone());
    game.state.objects[azula as usize].summoning_sick = false;
    attack(&mut game, &[azula]);
    drain(&mut game);
    assert_eq!(game.state.players[player.index()].combat_red_mana, 2);
    game.state.players[player.index()].mana = [0, 0, 0, 2, 0, 1];
    act(&mut game, cast_it);
    pay(&mut game);
    assert_eq!(
        game.state.stack.last().unwrap().cast.unwrap().payment,
        [0, 0, 0, 2, 0, 1]
    );
    drain(&mut game);
    assert_eq!(game.object_view(sorcery).owner, opponent);
    assert_eq!(game.object_view(sorcery).zone, Zone::Graveyard);
    assert_eq!(draws(&game, player), 2);
}

#[test]
fn hama_mills_before_choosing_and_waterbends_only_with_continuous_control() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let hama = put(&mut game, player, "Hama, the Bloodbender", Zone::Hand);
    let graveyard = put(&mut game, opponent, "Bear Cub", Zone::Graveyard);
    let milled_creature = put(&mut game, opponent, "Grizzly Bears", Zone::Library);
    let milled_land = put(&mut game, opponent, "Forest", Zone::Library);
    let spell = put(&mut game, opponent, "Quick Study", Zone::Library);
    cast(&mut game, hama, None);
    pass_pair(&mut game);
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(opponent),
        },
    );
    pass_pair(&mut game);
    for illegal in [graveyard, milled_creature, milled_land] {
        assert_eq!(game.object_view(illegal).zone, Zone::Graveyard);
        let action = exile(&game, illegal);
        reject(&mut game, action);
    }
    let mut decline = game.clone();
    act(&mut decline, Action::DeclineEffectPayment);
    drain(&mut decline);
    assert_eq!(decline.object_view(spell).zone, Zone::Graveyard);
    let action = exile(&game, spell);
    act(&mut game, action);
    drain(&mut game);
    let source = game.object_ref(hama);
    let announcement = Action::AnnounceLinkedWaterbend {
        object: game.object_ref(spell),
        source,
        face: CastFace::Normal,
        flash_surcharge: false,
    };
    let mut lost = game.clone();
    lost.change_controller(source, opponent);
    lost.change_controller(source, player);
    reject(&mut lost, announcement.clone());
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 3];
    act(&mut game, announcement);
    pay(&mut game);
    let receipt = game.state.stack.last().unwrap().cast.unwrap();
    assert_eq!(receipt.method, CastMethod::LinkedExileWaterbend);
    assert_eq!(receipt.payment, [0, 0, 0, 0, 0, 3]);
    drain(&mut game);
    assert_eq!(game.object_view(spell).owner, opponent);
    assert_eq!(game.object_view(spell).zone, Zone::Graveyard);
    assert_eq!(draws(&game, player), 2);
}

#[test]
fn iroh_grants_two_printed_cost_routes_and_keeps_casts_exiling_after_departure() {
    let player = Player::P0;
    let mut game = scenario(player);
    let iroh = put(&mut game, player, "Iroh, Grand Lotus", Zone::Battlefield);
    let lesson = put(&mut game, player, "Firebending Lesson", Zone::Graveyard);
    let normal = put(&mut game, player, "Quick Study", Zone::Graveyard);
    let enemy = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let one = ManaCost {
        colored: [0; 6],
        generic: 1,
    };
    let fixed = Action::AnnounceGrantedFixedFlashback {
        card: lesson,
        cost: one,
        flash_surcharge: false,
    };
    let printed = Action::AnnounceGrantedFlashback {
        card: normal,
        flash_surcharge: false,
    };
    let mut off_turn = game.clone();
    off_turn.state.active = player.other();
    reject(&mut off_turn, fixed.clone());
    reject(&mut off_turn, printed.clone());

    attack(&mut game, &[iroh]);
    drain(&mut game);
    assert_eq!(game.state.players[player.index()].combat_red_mana, 2);
    game.state.step = Step::Main2;
    let mut loan = game.clone();
    loan.state.players[player.index()].mana = [0, 1, 0, 2, 0, 0];
    act(&mut loan, printed);
    pay(&mut loan);
    assert_eq!(
        loan.state.stack.last().unwrap().cast.unwrap().payment,
        [0, 1, 0, 2, 0, 0]
    );
    drain(&mut loan);
    assert_eq!(loan.object_view(normal).zone, Zone::Exile);

    act(&mut game, fixed);
    if game
        .legal_actions()
        .contains(&Action::ChooseKicker { pay: false })
    {
        act(&mut game, Action::ChooseKicker { pay: false });
    }
    act(&mut game, target(enemy));
    if game
        .legal_actions()
        .contains(&Action::ChooseKicker { pay: false })
    {
        act(&mut game, Action::ChooseKicker { pay: false });
    }
    pay(&mut game);
    assert_eq!(
        game.state.stack.last().unwrap().cast.unwrap().method,
        CastMethod::GrantedFixedFlashback(one)
    );
    game.move_object(iroh, Zone::Exile);
    drain(&mut game);
    assert_eq!(game.object_view(lesson).zone, Zone::Exile);
    assert_eq!(game.object_view(enemy).damage, 2);
}
