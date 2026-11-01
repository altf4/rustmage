//! Focused printed outcomes; shared engine invariants live in core tests.
use super::*;

// Exercise the gated authored definition without claiming public catalogue support.
fn charm(g: &mut Game, owner: Player) -> CardId {
    let id = put(g, owner, "Bear Cub", Zone::Hand);
    g.state.objects[id as usize].card = Card::from_name("Fatehold Charm").unwrap();
    id
}

fn cast_draw(g: &mut Game, owner: Player) {
    let spell = charm(g, owner);
    act(g, Action::AnnounceCast { card: spell });
    act(g, Action::ChooseMode { mode: 0 });
    pay(g);
}

fn jaces(g: &Game, player: Player) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, o)| {
            (o.zone == Zone::Battlefield
                && o.controller == player
                && o.is_token()
                && g.current_kinds(id as CardId) & 64 != 0
                && g.has_subtype(id as CardId, "Jace"))
            .then_some(id as CardId)
        })
        .collect()
}

#[test]
fn fatehold_empower_ignores_physical_and_opposing_jaces_then_reuses_own_token() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let physical = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.state.objects[physical as usize].card = Card::from_name("Jace Reawakened").unwrap();
        g.state.objects[physical as usize].counters.loyalty = 4;
        g.create_tokens(p.other(), Card::JacePlaneswalkerToken, 1, false);
        let enemy = jaces(&g, p.other())[0];
        g.state.objects[enemy as usize].counters.loyalty = 4;
        let before = g.state.players[p.index()].library.len();
        cast_draw(&mut g, p);

        settle(&mut g);
        assert_eq!(g.state.players[p.index()].library.len(), before - 1);
        let own = jaces(&g, p);
        assert_eq!(own.len(), 1);
        assert_eq!(g.counter_count(own[0], CounterKind::Loyalty), 2);
        assert_eq!(g.counter_count(physical, CounterKind::Loyalty), 4);
        assert_eq!(g.counter_count(enemy, CounterKind::Loyalty), 4);
        cast_draw(&mut g, p);
        settle(&mut g);
        assert_eq!(jaces(&g, p), own);
        assert_eq!(g.counter_count(own[0], CounterKind::Loyalty), 4);
    }
}
