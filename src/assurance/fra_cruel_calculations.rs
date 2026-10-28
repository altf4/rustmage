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
