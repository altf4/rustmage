use super::*;

fn spell(g: &mut Game, p: Player, target: Player) -> CardId {
    let id = put(g, p, "Bear Cub", Zone::Hand);
    g.state.objects[id as usize].card = Card::from_name("Cruel Calculations").unwrap();
    cast(g, id, Some(Target::Player(target)));
    id
}

#[test]
fn cruel_calculations_counts_actual_target_player_moves_at_resolution() {
    for p in [Player::P0, Player::P1] {
        for target in [p, p.other()] {
            let mut g = scenario(p);
            let a = put(&mut g, target, "Forest", Zone::Library);
            let b = put(&mut g, target, "Forest", Zone::Library);
            let other = put(&mut g, target.other(), "Forest", Zone::Library);
            let hand = put(&mut g, target, "Forest", Zone::Hand);
            g.move_object(a, Zone::Graveyard);
            g.move_object(a, Zone::Library);
            g.move_object(a, Zone::Graveyard);
            g.move_object(other, Zone::Graveyard);
            g.move_object(hand, Zone::Graveyard);
            let before = g.state.players[p.index()].library.len();
            spell(&mut g, p, target);
            // The value is evaluated as the spell resolves, not on announcement.
            g.move_object(b, Zone::Graveyard);
            let snapshot = g.snapshot();
            super::contracts::branch_contract(g.clone());
            settle(&mut g);
            let expected = before - 3 - usize::from(target == p);
            assert_eq!(g.state.players[p.index()].library.len(), expected);
            g.restore(&snapshot).unwrap();
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].library.len(), expected);
        }
    }
}

#[test]
fn cruel_calculations_ignores_replaced_moves_and_resets_each_turn() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let card = put(&mut g, p.other(), "Forest", Zone::Library);
        let rip = put(&mut g, p, "Rest in Peace", Zone::Battlefield);
        g.move_object(card, Zone::Graveyard);
        assert_eq!(g.object_view(card).zone, Zone::Exile);
        assert_eq!(
            g.state.players[p.other().index()].cards_library_to_graveyard_this_turn,
            0
        );
        g.move_object(rip, Zone::Exile);
        let other = put(&mut g, p.other(), "Forest", Zone::Library);
        g.move_object(other, Zone::Graveyard);
        assert_eq!(
            g.state.players[p.other().index()].cards_library_to_graveyard_this_turn,
            1
        );
        g.begin_turn();
        g.state.step = Step::Main1;
        g.state.active = p;
        g.state.priority = p;
        assert_eq!(
            g.state.players[p.other().index()].cards_library_to_graveyard_this_turn,
            0
        );
        let before = g.state.players[p.index()].library.len();
        spell(&mut g, p, p.other());
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].library.len(), before);
    }
}

#[test]
fn cruel_calculations_public_history_counts_mixed_owner_batches_and_restores() {
    for controller in [Player::P0, Player::P1] {
        let mut g = scenario(controller);
        let a = put(&mut g, controller, "Forest", Zone::Library);
        let b = put(&mut g, controller.other(), "Forest", Zone::Library);
        let before = g.snapshot();
        let changes = g.move_objects_raw(&[a, b], Zone::Graveyard);
        assert!(
            changes
                .iter()
                .all(|change| change.moved && change.destination == Zone::Graveyard)
        );
        for viewer in [Player::P0, Player::P1] {
            let observation = g.observe(viewer);
            assert_eq!(
                observation.players[controller.index()].cards_library_to_graveyard_this_turn,
                1
            );
            assert_eq!(
                observation.players[controller.other().index()]
                    .cards_library_to_graveyard_this_turn,
                1
            );
        }
        g.move_object(a, Zone::Library);
        g.move_object(a, Zone::Graveyard);
        for viewer in [Player::P0, Player::P1] {
            let observation = g.observe(viewer);
            assert_eq!(
                observation.players[controller.index()].cards_library_to_graveyard_this_turn,
                2
            );
        }
        let after = g.snapshot();
        g.begin_turn();
        for viewer in [Player::P0, Player::P1] {
            assert!(
                g.observe(viewer)
                    .players
                    .iter()
                    .all(|p| p.cards_library_to_graveyard_this_turn == 0)
            );
        }
        g.restore(&after).unwrap();
        assert_eq!(
            g.observe(controller.other()).players[controller.index()]
                .cards_library_to_graveyard_this_turn,
            2
        );
        g.restore(&before).unwrap();
        assert!(
            g.observe(controller)
                .players
                .iter()
                .all(|p| p.cards_library_to_graveyard_this_turn == 0)
        );

        put(&mut g, controller, "Rest in Peace", Zone::Battlefield);
        let changes = g.move_objects_raw(&[a, b], Zone::Graveyard);
        assert!(
            changes
                .iter()
                .all(|change| change.moved && change.destination == Zone::Exile)
        );
        for viewer in [Player::P0, Player::P1] {
            assert!(
                g.observe(viewer)
                    .players
                    .iter()
                    .all(|p| p.cards_library_to_graveyard_this_turn == 0)
            );
        }
    }
}

#[test]
fn cruel_calculations_illegal_target_at_resolution_draws_nothing() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let milled = put(&mut g, p.other(), "Forest", Zone::Library);
        g.move_object(milled, Zone::Graveyard);
        let before = g.state.players[p.index()].library.len();
        let source = spell(&mut g, p, p.other());
        g.state.players[p.other().index()].hexproof_until_end_turn = true;
        settle(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].library.len(), before);
    }
}

#[test]
fn cruel_calculations_does_not_count_a_transient_card_copy_as_a_card() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let copy = put(&mut g, p, "Forest", Zone::Library);
        g.state.objects[copy as usize].copy_kind = CopyKind::Card;
        let card = put(&mut g, p, "Forest", Zone::Library);
        let events = g.move_objects_raw(&[copy, card], Zone::Graveyard);
        assert!(events.iter().all(|event| event.moved));
        assert_eq!(g.object_view(copy).zone, Zone::Graveyard);
        for viewer in [Player::P0, Player::P1] {
            assert_eq!(
                g.observe(viewer).players[p.index()].cards_library_to_graveyard_this_turn,
                1
            );
        }
    }
}
