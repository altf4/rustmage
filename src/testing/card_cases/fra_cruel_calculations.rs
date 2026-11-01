//! Focused printed outcomes; shared engine invariants live in core tests.
use super::*;

fn spell(g: &mut Game, p: Player, target: Player) -> CardId {
    let id = put(g, p, "Bear Cub", Zone::Hand);
    g.state.objects[id as usize].card = Card::from_name("Cruel Calculations").unwrap();
    cast(g, id, Some(Target::Player(target)));
    id
}

#[test]
fn cruel_calculations_ignores_replaced_moves_and_resets_each_turn() {
    {
        let p = Player::P0;
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
fn cruel_calculations_illegal_target_at_resolution_draws_nothing() {
    {
        let p = Player::P0;
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
