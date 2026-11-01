//! The follow-on must retain actual discard facts through a suspended result.
use super::*;
use crate::engine::assurance::big_helpers::{named, setup};
#[test]
fn silvan_discard_followon_rejects_departed_new_incarnation_and_replacement_destination() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..4 {
            let mut g = setup(p);
            let land = named(&mut g, p, "Forest", Zone::Hand);
            if mode == 3 {
                named(&mut g, p.other(), "Rest in Peace", Zone::Battlefield);
            }
            // Freeze an actual Hand->Graveyard attempt. The result continuation may
            // resume later, so its old exact event must never recover a different card.
            let events = g.move_objects_raw(&[land], Zone::Graveyard);
            let old = events[0].after;
            if mode == 1 {
                g.move_object(land, Zone::Exile);
            }
            if mode == 2 {
                g.move_object(land, Zone::Hand);
                g.move_object(land, Zone::Graveyard);
            }
            let mut counts = [0; 2];
            counts[p.index()] = 1;
            g.finish_discard_after(
                Some(DiscardAfter::ReturnLandFromGraveyard {
                    player: p,
                    tapped: true,
                }),
                counts,
                &events,
            );
            assert_eq!(
                g.object_view(land).zone,
                match mode {
                    0 => Zone::Battlefield,
                    1 | 3 => Zone::Exile,
                    _ => Zone::Graveyard,
                }
            );
            if mode == 0 {
                assert!(g.object_view(land).tapped);
            } else if mode == 2 {
                assert_ne!(g.object_ref(land), old);
            }
        }
    }
}
