//! Focused printed outcomes; shared engine invariants live in core tests.
//! Artifact sacrifice and controller-sensitive departure interactions.
use super::*;

#[test]
fn relic_leaves_without_dying_and_graveyard_changes_do_not_trigger() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let relic = put(&mut g, p, "Cryogen Relic", Zone::Battlefield);
        let before = g.state.players[p.index()].cards_drawn_this_turn;
        g.move_object(relic, Zone::Hand);
        g.prepare_triggers();
        big_helpers::settle(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, before + 1);
        g.move_object(relic, Zone::Graveyard);
        g.prepare_triggers();
        big_helpers::settle(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, before + 1);
    }
}

#[test]
fn tracker_ignores_opponent_sacrificing_nonartifact() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        put(&mut g, p.other(), "Vengeful Tracker", Zone::Battlefield);
        let land = put(&mut g, p, "Evolving Wilds", Zone::Battlefield);
        let life = g.state.players[p.index()].life;
        big_helpers::checked(
            &mut g,
            Action::AnnounceActivate {
                card: land,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        big_helpers::settle(&mut g);
        assert_eq!(g.object_view(land).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].life, life);
    }
}
