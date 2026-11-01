//! Projected Case thresholds retain cumulative gain through actual mana payments.
use super::*;
use crate::engine::assurance::big_helpers::{named, setup};

#[test]
fn case_threshold_survives_projected_life_payment() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let case = named(&mut g, p, "Case of the Uneaten Feast", Zone::Hand);
        let hall = named(&mut g, p, "Great Hall of the Biblioplex", Zone::Battlefield);
        g.gain_life(p, 4);
        let goal = FundingGoal {
            actor: p,
            required: [0; 6],
            generic: 1,
            reserve: None,
            source: None,
        };
        let movement = entries::EntryMove::from_view(g.object_view(case), Zone::Battlefield, false);
        let before = FundingState::from_game(&g, &goal);
        assert!(!before.entry_guard(StepGuard::UnsolvedLifeGainedAtLeast(5), &movement));
        g.gain_life(p, 1);
        let snapshot = g.snapshot();
        let world = FundingState::from_game(&g, &goal);
        assert!(world.entry_guard(StepGuard::UnsolvedLifeGainedAtLeast(5), &movement));
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
        assert!(after.entry_guard(StepGuard::UnsolvedLifeGainedAtLeast(5), &movement));
        assert!(after.lost_life(p));
        assert_eq!(g.snapshot(), snapshot);
    }
}
