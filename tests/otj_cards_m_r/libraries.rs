use super::*;

fn plot(game: &mut Game, card: CardId) {
    act(game, Action::AnnouncePlot { card });
    choose(game, |action| matches!(action, Action::PayPlot { .. }));
}

fn later_main(game: &mut Game, player: Player) {
    let turn = game.state.turn;
    reach(game, |g| {
        g.state.active == player && g.state.turn > turn && g.state.step == Step::Main1
    });
}

#[test]
fn make_your_own_luck_keeps_the_look_private_and_plots_one_exact_nonland() {
    let player = Player::P1;
    let mut game = scenario(player);
    let below = top(&mut game, player, "Island");
    let creature = top(&mut game, player, "Grizzly Bears");
    let land = top(&mut game, player, "Forest");
    let spell = hand(&mut game, player, "Make Your Own Luck");
    cast(&mut game, spell, None);
    reach(&mut game, |g| {
        matches!(g.state.pending, Pending::EffectLibrarySelection { .. })
    });
    assert!(game.observe(player.other()).library_selection.is_none());
    assert_eq!(
        game.observe(player).library_selection.unwrap().candidates,
        vec![game.object_ref(creature)]
    );
    reject(&mut game, Action::SelectLibrary { card: land });
    let checkpoint = game.snapshot();
    let mut decline = game.clone();
    act(&mut decline, Action::FinishLibrarySelection);
    assert_eq!(zone(&decline, creature), Zone::Hand);
    assert!(decline.observe(player).plotted_cards.is_empty());

    choose_library(&mut game, creature);
    for card in [land, below] {
        assert_eq!(zone(&game, card), Zone::Hand);
    }
    assert_eq!(zone(&game, creature), Zone::Exile);
    assert_eq!(
        game.observe(player.other()).plotted_cards[0].object,
        game.object_ref(creature)
    );
    let expected = game.snapshot();
    game.restore(&checkpoint).unwrap();
    choose_library(&mut game, creature);
    assert_eq!(game.snapshot(), expected);
    reject(&mut game, Action::AnnouncePlotted { card: creature });
    later_main(&mut game, player);
    act(&mut game, Action::AnnouncePlotted { card: creature });
    pay(&mut game);
    quiet(&mut game);
    assert_eq!(zone(&game, creature), Zone::Battlefield);
}

#[test]
fn pillage_the_bog_from_plot_looks_twice_the_live_land_count_and_replays_bottom_order() {
    let player = Player::P0;
    let mut game = scenario(player);
    board(&mut game, player.other(), "Forest");
    let spell = hand(&mut game, player, "Pillage the Bog");
    plot(&mut game, spell);
    reject(&mut game, Action::AnnouncePlotted { card: spell });
    later_main(&mut game, player);

    board(&mut game, player, "Forest");
    board(&mut game, player, "Swamp");
    let deep = top(&mut game, player, "Island");
    let first = top(&mut game, player, "Forest");
    let second = top(&mut game, player, "Swamp");
    let third = top(&mut game, player, "Mountain");
    let fourth = top(&mut game, player, "Plains");
    act(&mut game, Action::AnnouncePlotted { card: spell });
    pay(&mut game);
    reach(&mut game, |g| {
        matches!(g.state.pending, Pending::EffectLibrarySelection { .. })
    });
    let public = game.observe(player.other());
    assert!(public.library_selection.is_none());
    let candidates = game.observe(player).library_selection.unwrap().candidates;
    assert_eq!(candidates.len(), 4);
    assert!(!candidates.contains(&game.object_ref(deep)));
    reject(&mut game, Action::FinishLibrarySelection);
    let action = Action::SelectLibrary { card: second };
    let before = game.snapshot();
    let mut indexed = game.clone();
    let index = indexed
        .legal_actions()
        .iter()
        .position(|a| a == &action)
        .unwrap();
    indexed
        .apply_index(player, indexed.revision(), index)
        .unwrap();
    choose_library(&mut game, second);
    if has(&indexed, &Action::FinishLibrarySelection) {
        act(&mut indexed, Action::FinishLibrarySelection);
    }
    assert_eq!(game.snapshot(), indexed.snapshot());
    let final_state = game.snapshot();
    game.restore(&before).unwrap();
    choose_library(&mut game, second);
    assert_eq!(game.snapshot(), final_state);
    assert_eq!(zone(&game, second), Zone::Hand);
    let mut bottom = game.state.players[player.index()].library[..3].to_vec();
    bottom.sort_unstable();
    assert_eq!(bottom, vec![first, third, fourth]);
}

#[test]
fn plan_the_heist_surveils_only_for_an_empty_hand_at_resolution() {
    let player = Player::P1;
    for empty in [false, true] {
        let mut game = scenario(player);
        let first = top(&mut game, player, "Forest");
        let second = top(&mut game, player, "Island");
        let third = top(&mut game, player, "Swamp");
        if !empty {
            hand(&mut game, player, "Forest");
        }
        let spell = hand(&mut game, player, "Plan the Heist");
        cast(&mut game, spell, None);
        if empty {
            reach(&mut game, |g| {
                matches!(g.state.pending, Pending::EffectLibrary { .. })
            });
            arrange(&mut game, third, LibraryDestination::Graveyard);
            arrange(&mut game, second, LibraryDestination::Top);
            arrange(&mut game, first, LibraryDestination::Top);
        }
        quiet(&mut game);
        assert_eq!(zone(&game, first), Zone::Hand);
        assert_eq!(zone(&game, second), Zone::Hand);
        assert_eq!(
            zone(&game, third),
            if empty { Zone::Graveyard } else { Zone::Hand }
        );
        assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 3);
    }
}

#[test]
fn patient_naturalist_recovers_a_milled_land_and_creates_treasure_only_on_failure() {
    let player = Player::P0;
    for milled_land in [false, true] {
        let mut game = scenario(player);
        let old = graveyard(&mut game, player, "Forest");
        let first = top(&mut game, player, "Grizzly Bears");
        let second = top(&mut game, player, "Shock");
        let third = top(
            &mut game,
            player,
            if milled_land { "Island" } else { "Bear Cub" },
        );
        let naturalist = hand(&mut game, player, "Patient Naturalist");
        cast(&mut game, naturalist, None);
        if milled_land {
            reach(&mut game, |g| {
                matches!(g.state.pending, Pending::MilledSelection { .. })
            });
            let select = Action::SelectMilled {
                object: game.object_ref(third),
            };
            let prior = game.object_ref(old);
            reject(&mut game, Action::SelectMilled { object: prior });
            act(&mut game, select);
        }
        quiet(&mut game);
        assert_eq!(zone(&game, first), Zone::Graveyard);
        assert_eq!(zone(&game, second), Zone::Graveyard);
        assert_eq!(
            zone(&game, third),
            if milled_land {
                Zone::Hand
            } else {
                Zone::Graveyard
            }
        );
        assert_eq!(
            tokens(&game, player, TREASURE).len(),
            usize::from(!milled_land)
        );
    }
}

#[test]
fn pitiless_carnage_counts_actual_sacrifices_including_lands_and_tokens() {
    let player = Player::P1;
    let mut game = scenario(player);
    let land = board(&mut game, player, "Forest");
    let token = spawn(
        &mut game,
        player,
        Card::from_catalogue_id(TREASURE),
        Zone::Battlefield,
    );
    let foreign = board(&mut game, player.other(), "Grizzly Bears");
    let spell = hand(&mut game, player, "Pitiless Carnage");
    cast(&mut game, spell, None);
    until_action(&mut game, &Action::FinishEffectChoice);
    let mut zero = game.clone();
    act(&mut zero, Action::FinishEffectChoice);
    quiet(&mut zero);
    assert_eq!(zero.state.players[player.index()].cards_drawn_this_turn, 0);

    let enemy = game.object_ref(foreign);
    reject(&mut game, Action::Sacrifice { object: enemy });
    for card in [land, token] {
        let object = game.object_ref(card);
        act(&mut game, Action::Sacrifice { object });
    }
    if has(&game, &Action::FinishEffectChoice) {
        act(&mut game, Action::FinishEffectChoice);
    }
    quiet(&mut game);
    assert_eq!(zone(&game, land), Zone::Graveyard);
    assert!(tokens(&game, player, TREASURE).is_empty());
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 2);
}

#[test]
fn each_intrinsic_plot_route_charges_the_printed_color_and_generic_cost() {
    let player = Player::P0;
    for (name, payment) in [
        ("Outcaster Trailblazer", [0, 0, 0, 0, 1, 2]),
        ("Outlaw Stitcher", [0, 1, 0, 0, 0, 4]),
        ("Pillage the Bog", [0, 0, 1, 0, 1, 1]),
        ("Pitiless Carnage", [0, 0, 2, 0, 0, 1]),
        ("Plan the Heist", [0, 1, 0, 0, 0, 3]),
        ("Railway Brawler", [0, 0, 0, 0, 1, 3]),
        ("Rictus Robber", [0, 0, 1, 0, 0, 2]),
        ("Rise of the Varmints", [0, 0, 0, 0, 1, 2]),
    ] {
        let mut game = scenario(player);
        let card = hand(&mut game, player, name);
        game.state.players[player.index()].mana = payment;
        let begin = Action::AnnouncePlot { card };
        let mut underfunded = game.clone();
        underfunded.state.players[player.index()].mana[5] -= 1;
        reject(&mut underfunded, begin.clone());
        act(&mut game, begin);
        let pay = Action::PayPlot { payment };
        assert!(has(&game, &pay), "{name}");
        assert!(game.observe(player.other()).plot_payment.is_none());
        act(&mut game, pay);
        assert_eq!(game.state.players[player.index()].mana, [0; 6], "{name}");
        assert_eq!(
            game.observe(player.other()).plotted_cards[0].object,
            game.object_ref(card)
        );
        assert_eq!(game.state.players[player.index()].spells_cast_this_turn, 0);
        reject(&mut game, Action::AnnouncePlotted { card });
    }
}
