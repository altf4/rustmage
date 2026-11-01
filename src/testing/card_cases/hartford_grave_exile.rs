//! Focused printed outcomes; shared engine invariants live in core tests.
use super::big_helpers::{checked, settle};
use super::*;

#[test]
fn betrayal_opponent_chooses_creature_and_exiles_their_graveyard() {
    {
        let p = Player::P0;
        for creature in [false, true] {
            let mut g = scenario(p);
            let grave = put(&mut g, p.other(), "Bear Cub", Zone::Graveyard);
            let own_grave = put(&mut g, p, "Bear Cub", Zone::Graveyard);
            let selected = creature.then(|| put(&mut g, p.other(), "Bear Cub", Zone::Battlefield));
            let spell = put(&mut g, p, "Strategic Betrayal", Zone::Hand);
            checked(&mut g, Action::AnnounceCast { card: spell });
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Player(p.other()),
                },
            );
            pay(&mut g);
            if let Some(selected) = selected {
                for _ in 0..10 {
                    if g.legal_actions().contains(&Action::SelectResolutionExile {
                        object: g.object_ref(selected),
                    }) {
                        break;
                    }
                    let action = g
                        .legal_actions()
                        .into_iter()
                        .find(|a| matches!(a, Action::Pass | Action::PutTrigger { .. }))
                        .unwrap();
                    checked(&mut g, action);
                }
                assert_eq!(g.actor(), Some(p.other()));
                let object = g.object_ref(selected);
                checked(&mut g, Action::SelectResolutionExile { object });
            }
            settle(&mut g);
            assert_eq!(g.object_view(grave).zone, Zone::Exile);
            assert_eq!(g.object_view(own_grave).zone, Zone::Graveyard);
            if let Some(selected) = selected {
                assert_eq!(g.object_view(selected).zone, Zone::Exile);
            }
        }
    }
}
