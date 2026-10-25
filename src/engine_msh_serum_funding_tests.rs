//! Shared layer-four facts in actual and prospective cost-departure worlds.
use super::*;
use crate::engine::ability_world::CharacteristicWorld;
use crate::engine::assurance::big_helpers::{named, setup};
use crate::engine::msh_serum_tests::paid;
const SERUM: &str = "Super-Soldier Serum";
#[test]
fn msh_serum_funding_aura_departure_matches_full_actual_characteristics() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let serum = paid(&mut g, p, SERUM, &[bear]);
        let goal = FundingGoal {
            actor: p,
            required: [0; 6],
            generic: 0,
            reserve: None,
            source: None,
        };
        let before = g.snapshot();
        let mut world = FundingState::from_game(&g, &goal);
        world.prepare_entry_projection();
        assert!(world.legendary(bear));
        assert_eq!(
            characteristic_query::CharacteristicQuery::in_world(&world).object_view(bear),
            g.object_view(bear)
        );
        let mut actual = {
            let parent = g.clone();
            parent.clone()
        };
        actual.move_object(serum, Zone::Graveyard);
        let expected = actual.object_view(bear);
        let mut visits = 0;
        world.cost_departures(&[g.object_ref(serum)], Zone::Graveyard, &mut |after| {
            visits += 1;
            assert!(!after.legendary(bear));
            assert_eq!(
                characteristic_query::CharacteristicQuery::in_world(after).object_view(bear),
                expected
            );
            false
        });
        assert_eq!(visits, 1);
        assert_eq!(g.snapshot(), before);
    }
}
