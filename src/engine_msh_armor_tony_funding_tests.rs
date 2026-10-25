//! Borrowed-world matching for exact post-entry links and live granted stats.
use super::*;
use crate::engine::ability_world::CharacteristicWorld;
use crate::engine::assurance::big_helpers::{named, setup};
use crate::engine::msh_armor_tony_tests::{animate, armor, paid};
fn goal(p: Player) -> FundingGoal {
    FundingGoal {
        actor: p,
        required: [0; 6],
        generic: 0,
        reserve: None,
        source: None,
    }
}
#[test]
fn msh_armor_tony_funding_postentry_attachment_matches_complete_actual_views() {
    for p in [Player::P0, Player::P1] {
        for stale in [false, true] {
            let mut g = setup(p);
            let t = paid(&mut g, p, "Tony Stark", true, &[]);
            let n = named(&mut g, p, "Captain America's Shield", Zone::Hand);
            let r = g.object_ref(t);
            if stale {
                g.move_object(t, Zone::Exile);
                g.move_object(t, Zone::Battlefield);
            }
            let mut m = g.entry_move(n, Zone::Battlefield, false);
            m.attach_equipment_to = Some(r);
            let before = g.snapshot();
            let mut actual = g.clone();
            actual.stage_entries(vec![m.clone()], None);
            assert!(!actual.advance_entries());
            let mut visits = 0;
            let target = goal(p);
            let world = FundingState::from_game(&g, &target);
            world.visit_entry_moves(vec![m], true, None, &mut |after| {
                visits += 1;
                for id in [n, t] {
                    assert_eq!(
                        characteristic_query::CharacteristicQuery::in_world(after).object_view(id),
                        actual.object_view(id)
                    );
                }
                false
            });
            assert_eq!(visits, 1);
            assert_eq!(g.snapshot(), before);
        }
    }
}
#[test]
fn msh_armor_tony_funding_live_artifact_departure_matches_actual_complete_views() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let a = armor(&mut g, p, None);
        let r = paid(&mut g, p, "Arc Reactor", false, &[]);
        animate(&mut g, p, a);
        let goal = goal(p);
        let world = FundingState::from_game(&g, &goal);
        assert_eq!(world.power(a), 2);
        let mut actual = g.clone();
        actual.move_object(r, Zone::Graveyard);
        let mut visits = 0;
        world.cost_departures(&[g.object_ref(r)], Zone::Graveyard, &mut |after| {
            visits += 1;
            assert_eq!(after.power(a), 1);
            assert_eq!(
                characteristic_query::CharacteristicQuery::in_world(after).object_view(a),
                actual.object_view(a)
            );
            false
        });
        assert_eq!(visits, 1);
    }
}

#[test]
fn msh_armor_tony_funding_live_grant_toughness_guard_changes_with_artifact_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let a = armor(&mut g, p, None);
        let mut r = 0;
        for _ in 0..3 {
            r = paid(&mut g, p, "Arc Reactor", false, &[]);
        }
        animate(&mut g, p, a);
        let b = named(&mut g, p, "Baxter Building", Zone::Battlefield);
        let target = goal(p);
        let world = FundingState::from_game(&g, &target);
        let guard = ActivationGuard::ControlCreatureToughnessAtLeast(4);
        assert!(world.guard_available(guard, b, p));
        let mut visits = 0;
        world.cost_departures(&[g.object_ref(r)], Zone::Graveyard, &mut |after| {
            visits += 1;
            assert!(!after.guard_available(guard, b, p));
            assert_eq!(after.power(a), 3);
            false
        });
        assert_eq!(visits, 1);
    }
}
