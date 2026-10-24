//! The same exact relation follows projected control after cost departures.
use super::*;
use crate::engine::assurance::big_helpers::{cast, named, settle, setup};
#[test]
fn msh_super_funding_guard_tracks_exact_virtual_control_source_departure() {
    for p in [Player::P0, Player::P1] {
        let q = p.other();
        let mut g = setup(p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let aura = named(&mut g, p, "Super Intelligence", Zone::Hand);
        cast(&mut g, aura, None, &[Target::Card(bear)]);
        settle(&mut g);
        g.state.active = q;
        g.state.priority = q;
        let control = named(&mut g, q, "Confiscate", Zone::Hand);
        cast(&mut g, control, None, &[Target::Card(bear)]);
        settle(&mut g);
        assert_eq!(g.object_view(bear).controller, q);
        let source = g.object_ref(aura);
        assert!(steps::attached_controller_turn(&g, source, q));
        let before = g.snapshot();
        let mut actual = {
            let parent = g.clone();
            parent.clone()
        };
        actual.move_object(control, Zone::Graveyard);
        assert_eq!(actual.object_view(bear).controller, p);
        assert!(!steps::attached_controller_turn(&actual, source, q));
        let goal = FundingGoal {
            actor: q,
            required: [0; 6],
            generic: 0,
            reserve: None,
            source: None,
        };
        let mut world = FundingState::from_game(&g, &goal);
        world.prepare_entry_projection();
        let movement = entries::EntryMove::from_view(g.object_view(aura), Zone::Battlefield, false);
        assert!(world.entry_guard(StepGuard::AttachedControllerTurn, &movement));
        let mut visits = 0;
        world.cost_departures(&[g.object_ref(control)], Zone::Graveyard, &mut |after| {
            visits += 1;
            assert_eq!(after.object(bear).controller, p);
            assert!(!after.entry_guard(StepGuard::AttachedControllerTurn, &movement));
            false
        });
        assert_eq!(visits, 1);
        assert_eq!(g.snapshot(), before);
    }
}
