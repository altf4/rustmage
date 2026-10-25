//! Hand-only entry ordering must not advance the characteristic timestamp clock.
use super::*;
use crate::engine::assurance::big_helpers::{cast, checked, named, settle, setup};
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
fn msh_ten_rings_funding_both_entry_orders_equal_actual_without_role_clock_drift() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.next_role_timestamp();
        let role_before = g.state.roles.as_deref().cloned();
        let ids = [
            named(&mut g, p, "The Ten Rings", Zone::Graveyard),
            named(&mut g, p, "Niv-Mizzet, Visionary", Zone::Graveyard),
        ];
        let moves = ids
            .map(|id| g.entry_move(id, Zone::Battlefield, false))
            .to_vec();
        let before = g.snapshot();
        let world = FundingState::from_game(&g, &goal(p));
        let mut outcomes = Vec::new();
        world.visit_entry_moves(moves.clone(), true, None, &mut |after| {
            let projection = after.entry_projection.as_ref().unwrap();
            assert_eq!(Some(&projection.roles), role_before.as_ref());
            assert_eq!(projection.hand_size.sources.len(), 2);
            let times = ids.map(|id| {
                projection
                    .hand_size
                    .sources
                    .iter()
                    .find(|v| v.source == after.reference(id))
                    .unwrap()
                    .timestamp
            });
            assert!(times[0] > 0 && times[1] > 0 && times[0] != times[1]);
            outcomes.push((times[0] < times[1], projection.hand_size.clone()));
            false
        });
        assert_eq!(outcomes.len(), 2);
        assert_ne!(outcomes[0].0, outcomes[1].0);
        assert_eq!(g.snapshot(), before);
        for (rings_first, expected) in outcomes {
            let mut actual = {
                let parent = g.clone();
                parent.clone()
            };
            actual.stage_entries(moves.clone(), None);
            actual.advance_entries();
            let object = actual.object_ref(ids[usize::from(!rings_first)]);
            checked(&mut actual, Action::ChooseEntryTimestamp { object });
            settle(&mut actual);
            assert_eq!(actual.state.hand_size.as_deref(), Some(&expected));
            assert_eq!(actual.state.roles.as_deref(), role_before.as_ref());
            assert_eq!(
                actual.maximum_hand_size(p),
                if rings_first { None } else { Some(10) }
            );
        }
    }
}
#[test]
fn msh_ten_rings_funding_departure_keeps_lasting_grant_and_parent_owned() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let wisdom = named(&mut g, p, "Wisdom of Ages", Zone::Hand);
        cast(&mut g, wisdom, None, &[]);
        settle(&mut g);
        let rings = named(&mut g, p, "The Ten Rings", Zone::Battlefield);
        let reference = g.object_ref(rings);
        let before = g.snapshot();
        let expected_lasting = g.state.hand_size.as_ref().unwrap().lasting;
        let mut world = FundingState::from_game(&g, &goal(p));
        world.prepare_entry_projection();
        let mut visits = 0;
        world.cost_departures(&[reference], Zone::Graveyard, &mut |after| {
            visits += 1;
            let projection = after.entry_projection.as_ref().unwrap();
            assert_eq!(projection.hand_size.lasting, expected_lasting);
            assert!(
                projection
                    .hand_size
                    .sources
                    .iter()
                    .all(|row| row.source != reference)
            );
            assert_eq!(after.object(rings).zone, Zone::Graveyard);
            false
        });
        assert_eq!(visits, 1);
        assert_eq!(g.snapshot(), before);
        assert_eq!(g.maximum_hand_size(p), Some(10));
    }
}
