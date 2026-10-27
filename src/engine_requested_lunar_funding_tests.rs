//! Projected life payment preserves prior gain for compound guard evaluation.
use super::*;
use crate::engine::assurance::big_helpers::{named, setup};

#[test]
fn projected_mana_life_payment_satisfies_gained_and_lost_guard() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, "Lunar Convocation", Zone::Hand);
        let hall = named(&mut g, p, "Great Hall of the Biblioplex", Zone::Battlefield);
        g.gain_life(p, 1);
        let saved = g.snapshot();
        let movement =
            entries::EntryMove::from_view(g.object_view(source), Zone::Battlefield, false);
        let goal = FundingGoal {
            actor: p,
            required: [0; 6],
            generic: 1,
            reserve: None,
            source: None,
        };
        let world = FundingState::from_game(&g, &goal);
        assert!(!world.entry_guard(StepGuard::GainedAndLostLife, &movement));
        let step = world
            .moves(&goal)
            .into_iter()
            .find(|m| world.resources[m.resource].id == hall && m.ability == Some(1))
            .unwrap();
        let mut after = None;
        world.visit_applied(&step, &mut |next| {
            after = Some(next.clone());
            true
        });
        let after = after.unwrap();
        assert!(after.entry_guard(StepGuard::GainedAndLostLife, &movement));
        assert!(!world.entry_guard(StepGuard::GainedAndLostLife, &movement));
        assert_eq!(g.snapshot(), saved);
    }
}
