//! Focused printed outcomes; shared engine invariants live in core tests.
use super::*;

fn finish_fblthp(g: &mut Game) {
    for _ in 0..48 {
        if g.outcome().is_some()
            || (g.state.pending == Pending::Priority
                && g.state.stack.is_empty()
                && g.state.pending_triggers.is_empty()
                && g.state.resolving.is_none())
        {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("trigger did not finish");
}

#[test]
fn fblthp_observes_one_combat_batch_only_on_its_controllers_turn() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        for id in g.state.players[p.index()].library.clone() {
            g.move_object(id, Zone::Exile);
        }
        let fblthp = put(&mut g, p, "Fblthp, Impossibly Lost", Zone::Battlefield);
        let dealer = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        for _ in 0..3 {
            put(&mut g, p, "Forest", Zone::Library);
        }
        let event = |g: &Game, target, combat| DamageEvent {
            preventable: true,
            source: g.damage_source(dealer),
            target: Target::Player(target),
            amount: 1,
            combat,
        };
        g.apply_damage_batch(&[event(&g, p, true), event(&g, p.other(), false)]);
        assert!(g.state.pending_triggers.is_empty());
        g.state.active = p.other();
        g.apply_damage_batch(&[event(&g, p.other(), true)]);
        assert!(g.state.pending_triggers.is_empty());
        g.state.active = p;
        g.apply_damage_batch(&[event(&g, p.other(), true), event(&g, p.other(), true)]);
        assert_eq!(g.state.pending_triggers.len(), 1);

        finish_fblthp(&mut g);
        assert_eq!(g.object_view(fblthp).zone, Zone::Library);
        assert_eq!(g.state.players[p.index()].library.len(), 2);
        assert!(g.outcome().is_none());
    }
}

#[test]
fn fblthp_wins_after_the_draw_before_shuffling_itself() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        for id in g.state.players[p.index()].library.clone() {
            g.move_object(id, Zone::Exile);
        }
        let fblthp = put(&mut g, p, "Fblthp, Impossibly Lost", Zone::Battlefield);
        let dealer = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        for _ in 0..2 {
            put(&mut g, p, "Forest", Zone::Library);
        }
        let event = DamageEvent {
            preventable: true,
            source: g.damage_source(dealer),
            target: Target::Player(p.other()),
            amount: 1,
            combat: true,
        };
        g.apply_damage_batch(&[event]);
        finish_fblthp(&mut g);
        assert_eq!(g.outcome(), Some(Outcome::Winner(p)));
        assert_eq!(g.object_view(fblthp).zone, Zone::Battlefield);
    }
}
