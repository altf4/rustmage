//! Borrowed source permissions and owned parent privacy through virtual entries/departures.
use super::*;
use crate::engine::assurance::big_helpers::{named, settle, setup};
use crate::engine::msh_land_origins_tests::{goal, moved, observation, paid};
const KAZAR: &str = "Ka-Zar of the Savage Land";
const MOLE: &str = "Mole Man, Moloid Master";
#[test]
fn msh_land_origins_borrowed_provider_departure_matches_actual_without_funding_land_drops() {
    for p in [Player::P0, Player::P1] {
        for origin in [Zone::Library, Zone::Graveyard] {
            let mut g = setup(p);
            let s = named(
                &mut g,
                p,
                if origin == Zone::Library { KAZAR } else { MOLE },
                Zone::Battlefield,
            );
            let x = named(&mut g, p, "Forest", origin);
            g.state.players[p.index()].mana = [0; 6];
            let before = g.snapshot();
            let obs = [observation(&g, p), observation(&g, p.other())];
            let need = goal(p);
            let mut w = FundingState::from_game(&g, &need);
            w.prepare_entry_projection();
            assert!(cast_origins::land_origin_available(&w, p, origin));
            let mut calls = 0;
            w.cost_departures(&[g.object_ref(s)], Zone::Graveyard, &mut |after| {
                calls += 1;
                assert!(!cast_origins::land_origin_available(after, p, origin));
                false
            });
            assert_eq!(calls, 1);
            assert!(!g.can_fund(FundingGoal {
                generic: 1,
                ..goal(p)
            }));
            assert_eq!(g.snapshot(), before);
            assert_eq!([observation(&g, p), observation(&g, p.other())], obs);
            assert!(g.can_play_static_land(p, x));
        }
    }
}
#[test]
fn msh_land_origins_borrowed_new_entry_and_control_aura_departure_use_live_owner() {
    for p in [Player::P0, Player::P1] {
        for origin in [Zone::Library, Zone::Graveyard] {
            let mut g = setup(p);
            let s = named(
                &mut g,
                p,
                if origin == Zone::Library { KAZAR } else { MOLE },
                Zone::Hand,
            );
            let before = g.snapshot();
            let need = goal(p);
            let w = FundingState::from_game(&g, &need);
            assert!(!cast_origins::land_origin_available(&w, p, origin));
            let movement = g.entry_move(s, Zone::Battlefield, false);
            let mut calls = 0;
            w.commit_projected_entries(vec![movement], &[0], &mut |after| {
                calls += 1;
                assert!(cast_origins::land_origin_available(after, p, origin));
                false
            });
            assert_eq!(calls, 1);
            assert_eq!(g.snapshot(), before);
            moved(&mut g, &[s], Zone::Battlefield);
            settle(&mut g);
            g.change_controller(g.object_ref(s), p.other());
            let aura = paid(&mut g, p, "Confiscate", &[Target::Card(s)]);
            let need = goal(p);
            let mut w = FundingState::from_game(&g, &need);
            w.prepare_entry_projection();
            let mut calls = 0;
            w.cost_departures(&[g.object_ref(aura)], Zone::Graveyard, &mut |after| {
                calls += 1;
                assert!(cast_origins::land_origin_available(
                    after,
                    p.other(),
                    origin
                ));
                assert!(!cast_origins::land_origin_available(after, p, origin));
                false
            });
            assert_eq!(calls, 1);
        }
    }
}
