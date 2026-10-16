use super::tests::{act, cast, pass_pair, scenario, spawn};
use super::*;

fn named(game: &mut Game, player: Player, name: &str, zone: Zone) -> CardId {
    spawn(game, player, Card::from_name(name).unwrap(), zone)
}

fn enter_choice(game: &mut Game, player: Player, name: &str) -> CardId {
    let source = named(game, player, name, Zone::Hand);
    game.move_object(source, Zone::Battlefield);
    game.prepare_triggers();
    pass_pair(game);
    assert_eq!(game.state.pending, Pending::EffectLibrary { player });
    source
}

fn arrange(game: &mut Game, card: CardId, destination: LibraryDestination) {
    act(game, Action::ArrangeLibrary { card, destination });
}

#[test]
fn scry_partition_orders_both_groups_and_keeps_middle() {
    let mut game = scenario();
    // The private interpreter fixture exercises counts larger than current cards.
    static PROGRAM: &[Instruction] = &[
        Instruction::Scry(4),
        Instruction::Draw(PlayerSet::Controller, 1),
    ];
    let source = named(&mut game, Player::P0, "Helpful Hunter", Zone::Hand);
    game.move_object(source, Zone::Battlefield);
    game.prepare_triggers();
    let item = game.state.stack.pop().unwrap();
    game.state.resolving = Some(Resolution {
        explore: None,
        connive: None,
        item,
        program: PROGRAM,
        pc: 0,
        target_group: 0,
        moved_count: None,
        discards: [0; 2],
        mana_payment: None,
        forage: None,
        selected_discards: [Vec::new(), Vec::new()],
        milled: Vec::new(),
        library: None,
        selection: None,
        target_library: None,
        search: None,
        choice: None,
    });
    let original = game.state.players[0].library.clone();
    let n = original.len();
    let [a, b, c, d] = [
        original[n - 1],
        original[n - 2],
        original[n - 3],
        original[n - 4],
    ];
    game.run_program();
    arrange(&mut game, c, LibraryDestination::Bottom);
    arrange(&mut game, b, LibraryDestination::Top);
    arrange(&mut game, a, LibraryDestination::Bottom);
    assert_eq!(game.state.players[0].library, original);
    let known = game.observe(Player::P0).library_choice.unwrap();
    assert_eq!(known.top[0].id, b);
    assert_eq!(
        known.bottom.iter().map(|o| o.id).collect::<Vec<_>>(),
        vec![c, a]
    );
    assert!(
        known
            .top
            .iter()
            .chain(&known.bottom)
            .all(|o| o.zone == Zone::Library)
    );
    arrange(&mut game, d, LibraryDestination::Top);
    assert_eq!(game.state.objects[b as usize].zone, Zone::Hand);
    let mut expected = vec![a, c]; // chosen c is above a in bottom group.
    expected.extend_from_slice(&original[..n - 4]);
    expected.push(d);
    assert_eq!(
        game.state.players[0].library.as_slice(),
        expected.as_slice()
    );
    assert!(game.state.resolving.is_none());
}

#[test]
fn surveil_choices_are_private_atomic_and_replayable_at_every_stage() {
    let mut game = scenario();
    enter_choice(&mut game, Player::P0, "Sterling Hound");
    let original = game.state.players[0].library.clone();
    let refs: Vec<_> = original.iter().map(|&id| game.object_ref(id)).collect();
    let cards: Vec<_> = game
        .observe(Player::P0)
        .library_choice
        .unwrap()
        .remaining
        .iter()
        .map(|o| o.id)
        .collect();
    assert_eq!(cards.len(), 2);
    assert!(game.observe(Player::P1).library_choice.is_none());
    assert!(game.observe(Player::P1).actions.is_empty());
    for invalid in [
        Action::Pass,
        Action::ArrangeLibrary {
            card: cards[0],
            destination: LibraryDestination::Bottom,
        },
        Action::ArrangeLibrary {
            card: u32::MAX,
            destination: LibraryDestination::Top,
        },
    ] {
        let state = game.state.clone();
        let revision = game.revision();
        assert_eq!(game.apply(Player::P0, invalid), Err(Error::IllegalAction));
        assert_eq!(game.state, state);
        assert_eq!(game.revision(), revision);
    }
    let initial = game.snapshot();
    let opponent = serde_json::to_value(game.observe(Player::P1)).unwrap();
    arrange(&mut game, cards[0], LibraryDestination::Graveyard);
    assert_eq!(game.state.players[0].library, original);
    assert_eq!(game.object_ref(cards[0]), *refs.last().unwrap());
    let mut later = serde_json::to_value(game.observe(Player::P1)).unwrap();
    later["revision"] = opponent["revision"].clone();
    assert_eq!(opponent, later);
    let known = game.observe(Player::P0).library_choice.unwrap();
    assert_eq!(known.graveyard[0].id, cards[0]);
    assert_eq!(
        known.graveyard[0].card,
        game.state.objects[cards[0] as usize].card
    );
    assert_eq!(known.graveyard[0].zone, Zone::Library);
    let partial = game.snapshot();
    let final_action = Action::ArrangeLibrary {
        card: cards[1],
        destination: LibraryDestination::Top,
    };
    act(&mut game, final_action.clone());
    let result = game.state.clone();
    assert_eq!(game.state.objects[cards[0] as usize].zone, Zone::Graveyard);
    assert_ne!(game.object_ref(cards[0]), *refs.last().unwrap());
    assert_eq!(game.state.players[0].library.last(), Some(&cards[1]));
    game.restore(&partial).unwrap();
    act(&mut game, final_action.clone());
    assert_eq!(game.state, result);
    game.restore(&initial).unwrap();
    arrange(&mut game, cards[0], LibraryDestination::Graveyard);
    act(&mut game, final_action);
    assert_eq!(game.state, result);
}

#[test]
fn all_two_card_partitions_and_orderings_are_reachable() {
    for surveil in [false, true] {
        let mut root = scenario();
        enter_choice(
            &mut root,
            Player::P0,
            if surveil {
                "Sterling Hound"
            } else {
                "Wakandan Drone Flock"
            },
        );
        let cards = root
            .state
            .resolving
            .as_ref()
            .unwrap()
            .library
            .as_ref()
            .unwrap()
            .remaining
            .clone();
        let mut outcomes = Vec::new();
        for first in root
            .legal_actions()
            .into_iter()
            .filter(|a| matches!(a, Action::ArrangeLibrary { .. }))
        {
            let mut branch = root.clone();
            act(&mut branch, first);
            for last in branch
                .legal_actions()
                .into_iter()
                .filter(|a| matches!(a, Action::ArrangeLibrary { .. }))
            {
                let mut leaf = branch.clone();
                act(&mut leaf, last);
                let outcome = (
                    leaf.state.players[0].library.clone(),
                    cards
                        .iter()
                        .map(|&id| leaf.state.objects[id as usize].zone)
                        .collect::<Vec<_>>(),
                );
                if !outcomes.contains(&outcome) {
                    outcomes.push(outcome);
                }
            }
        }
        // Both kept: two orders; one kept: two choices; neither kept:
        // scry has two bottom orders, surveil one unordered graveyard subset.
        assert_eq!(outcomes.len(), if surveil { 5 } else { 6 });
    }
}

#[test]
fn short_empty_and_zero_looks_do_not_attempt_empty_draws() {
    for available in [0, 1] {
        let mut game = scenario();
        while game.state.players[0].library.len() > available {
            let id = *game.state.players[0].library.last().unwrap();
            game.move_object(id, Zone::Hand);
        }
        let id = named(&mut game, Player::P0, "Sterling Hound", Zone::Hand);
        game.move_object(id, Zone::Battlefield);
        game.prepare_triggers();
        pass_pair(&mut game);
        if available == 1 {
            let card = game.observe(Player::P0).library_choice.unwrap().remaining[0].id;
            arrange(&mut game, card, LibraryDestination::Graveyard);
        }
        assert!(!game.state.players[0].attempted_empty_draw);
        assert!(game.state.resolving.is_none());
        assert!(game.outcome().is_none());
    }
    static PROGRAM: &[Instruction] = &[
        Instruction::Scry(0),
        Instruction::Surveil(0),
        Instruction::GainLife(PlayerSet::Controller, 1),
    ];
    let mut game = scenario();
    let id = named(&mut game, Player::P0, "Helpful Hunter", Zone::Hand);
    game.move_object(id, Zone::Battlefield);
    game.prepare_triggers();
    let item = game.state.stack.pop().unwrap();
    game.state.resolving = Some(Resolution {
        explore: None,
        connive: None,
        item,
        program: PROGRAM,
        pc: 0,
        target_group: 0,
        moved_count: None,
        discards: [0; 2],
        mana_payment: None,
        forage: None,
        selected_discards: [Vec::new(), Vec::new()],
        milled: Vec::new(),
        library: None,
        selection: None,
        target_library: None,
        search: None,
        choice: None,
    });
    game.run_program();
    assert_eq!(game.state.players[0].life, 21);
    assert!(game.state.resolving.is_none());
}

#[test]
fn activated_surveil_uses_controller_library_on_opponents_turn() {
    let mut game = scenario();
    let wall = named(&mut game, Player::P1, "Rune-Sealed Wall", Zone::Battlefield);
    act(&mut game, Action::Pass);
    let action = game
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a,Action::Activate{card,..} if *card==wall))
        .unwrap();
    act(&mut game, action);
    pass_pair(&mut game);
    assert_eq!(game.actor(), Some(Player::P1));
    let viewed = game.observe(Player::P1).library_choice.unwrap();
    assert!(viewed.surveil);
    assert_eq!(viewed.remaining[0].owner, Player::P1);
    assert!(game.observe(Player::P0).library_choice.is_none());
    arrange(
        &mut game,
        viewed.remaining[0].id,
        LibraryDestination::Graveyard,
    );
    assert_eq!(game.state.priority, Player::P0);
}

#[test]
fn targeted_spell_completes_removal_before_surveil_and_fizzles_without_it() {
    for remove_target in [false, true] {
        let mut game = scenario();
        let target = named(&mut game, Player::P1, "Grizzly Bears", Zone::Battlefield);
        let spell = named(&mut game, Player::P0, "Unauthorized Exit", Zone::Hand);
        game.state.players[0].mana = [0, 2, 0, 0, 0, 0];
        cast(&mut game, spell, Some(Target::Card(target)));
        if remove_target {
            game.move_object(target, Zone::Graveyard);
        }
        pass_pair(&mut game);
        if remove_target {
            assert!(game.state.resolving.is_none());
            assert!(game.observe(Player::P0).library_choice.is_none());
        } else {
            assert_eq!(game.state.objects[target as usize].zone, Zone::Hand);
            assert_eq!(game.state.objects[spell as usize].zone, Zone::Stack);
            let id = game.observe(Player::P0).library_choice.unwrap().remaining[0].id;
            arrange(&mut game, id, LibraryDestination::Top);
            assert_eq!(game.state.objects[spell as usize].zone, Zone::Graveyard);
        }
    }
}

#[test]
fn source_departure_does_not_cancel_private_etb_choices() {
    let mut game = scenario();
    let source = named(&mut game, Player::P0, "Wakandan Drone Flock", Zone::Hand);
    game.move_object(source, Zone::Battlefield);
    game.prepare_triggers();
    game.move_object(source, Zone::Hand);
    pass_pair(&mut game);
    let choice = game.observe(Player::P0).library_choice.unwrap();
    assert_eq!(choice.remaining.len(), 2);
    for object in choice.remaining {
        arrange(&mut game, object.id, LibraryDestination::Bottom);
    }
    assert!(game.state.resolving.is_none());
    assert_eq!(game.state.objects[source as usize].zone, Zone::Hand);
}

#[test]
fn sba_waits_through_library_choices_and_following_instructions() {
    static PROGRAM: &[Instruction] = &[
        Instruction::LoseLife(PlayerSet::Controller, 20),
        Instruction::Scry(2),
        Instruction::GainLife(PlayerSet::Controller, 1),
    ];
    let mut game = scenario();
    let source = named(&mut game, Player::P0, "Helpful Hunter", Zone::Hand);
    game.move_object(source, Zone::Battlefield);
    game.prepare_triggers();
    let item = game.state.stack.pop().unwrap();
    game.state.resolving = Some(Resolution {
        explore: None,
        connive: None,
        item,
        program: PROGRAM,
        pc: 0,
        target_group: 0,
        moved_count: None,
        discards: [0; 2],
        mana_payment: None,
        forage: None,
        selected_discards: [Vec::new(), Vec::new()],
        milled: Vec::new(),
        library: None,
        selection: None,
        target_library: None,
        search: None,
        choice: None,
    });
    game.run_program();
    let cards = game.observe(Player::P0).library_choice.unwrap().remaining;
    assert_eq!(game.state.players[0].life, 0);
    assert!(game.outcome().is_none());
    arrange(&mut game, cards[0].id, LibraryDestination::Top);
    assert!(game.outcome().is_none());
    arrange(&mut game, cards[1].id, LibraryDestination::Top);
    assert_eq!(game.state.players[0].life, 1);
    assert!(game.outcome().is_none());
}

#[test]
fn opt_and_diresight_draw_after_the_arranged_library_commits() {
    for name in ["Opt", "Diresight"] {
        let mut game = scenario();
        let spell = named(&mut game, Player::P0, name, Zone::Hand);
        game.state.players[0].mana = [0, 3, 3, 0, 0, 0];
        let original = game.state.players[0].library.clone();
        let hand = game.hand_count(Player::P0);
        cast(&mut game, spell, None);
        pass_pair(&mut game);
        let view = game.observe(Player::P0).library_choice.unwrap();
        let card = view.remaining[0].id;
        if name == "Opt" {
            arrange(&mut game, card, LibraryDestination::Bottom);
            assert_eq!(
                game.state.objects[original[original.len() - 2] as usize].zone,
                Zone::Hand
            );
            assert_eq!(game.state.players[0].library.first(), Some(&card));
            assert_eq!(game.hand_count(Player::P0), hand);
        } else {
            arrange(&mut game, card, LibraryDestination::Graveyard);
            let kept = view.remaining[1].id;
            arrange(&mut game, kept, LibraryDestination::Top);
            assert_eq!(game.state.objects[kept as usize].zone, Zone::Hand);
            assert_eq!(game.state.objects[card as usize].zone, Zone::Graveyard);
            assert_eq!(game.hand_count(Player::P0), hand + 1);
            assert_eq!(game.state.players[0].life, 18);
        }
        assert_eq!(game.state.objects[spell as usize].zone, Zone::Graveyard);
        assert!(game.observe(Player::P0).library_choice.is_none());
    }
}
