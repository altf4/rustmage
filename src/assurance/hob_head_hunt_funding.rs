//! Sparse actual and projected departure agree; deferred Wolves cannot fund costs.
use super::*;
#[test]
fn hob_hunt_funding_projects_opposing_replacement_without_creating_reflexive_tokens() {
    for p in [Player::P0, Player::P1] {
        let (mut g, bolg, fodder) = fixture(p);
        let h = named(&mut g, p.other(), "Head of the Hunt", Zone::Battlefield);
        let before = g.snapshot();
        let w = FundingState::from_game(&g, &goal(p));
        let n = child(&w, &step(&w, p, bolg, fodder));
        assert_eq!(n.object(fodder).zone, Zone::Exile);
        assert_eq!(n.creatures_died, w.creatures_died);
        assert!(g.can_fund(goal(p)));
        assert_eq!(g.snapshot(), before);
        drop(n);
        drop(w);
        mana(&mut g, bolg, fodder);
        assert_eq!(g.object_view(fodder).zone, Zone::Exile);
        assert_eq!(
            g.state
                .pending_triggers
                .iter()
                .chain(&g.state.ordering_triggers)
                .chain(&g.state.stack)
                .filter(|item| item.object == g.object_ref(h))
                .count(),
            1
        );
        assert!(
            !g.state
                .objects
                .iter()
                .any(|o| o.zone == Zone::Battlefield && o.is_token() && o.card.index() == 5412)
        );
    }
}
