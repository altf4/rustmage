//! Aura attachment last-known information is refreshed when the source leaves.
use super::*;

#[test]
fn meltstrider_uses_latest_attachment_if_aura_moves_then_leaves() {
    for p in [Player::P0, Player::P1] {
        for leave_before_target in [false, true] {
            let mut g = scenario(p);
            let first = put(&mut g, p, "Hill Giant", Zone::Battlefield);
            let second = put(&mut g, p, "Hill Giant", Zone::Battlefield);
            let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let aura = put(&mut g, p, "Meltstrider's Resolve", Zone::Hand);
            big_helpers::cast(&mut g, aura, None, &[Target::Card(first)]);
            for _ in 0..40 {
                g.prepare_triggers();
                if matches!(g.state.pending, Pending::Targets { .. }) {
                    break;
                }
                big_helpers::checked(&mut g, Action::Pass);
            }
            assert!(matches!(g.state.pending, Pending::Targets { .. }));
            if !leave_before_target {
                big_helpers::checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(enemy),
                    },
                );
            }
            // A legal move-Aura effect can change the enchanted creature while
            // its trigger is queued. This sets up that established state.
            g.state.objects[aura as usize].attached_to = Some(g.object_ref(second));
            g.move_object(aura, Zone::Graveyard);
            if leave_before_target {
                big_helpers::checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(enemy),
                    },
                );
            }
            let before = g.snapshot();
            let mut branch = g.clone();
            big_helpers::settle(&mut g);
            assert_eq!(g.object_view(first).damage, 0);
            assert_eq!(g.object_view(second).damage, 2);
            assert_eq!(g.object_view(enemy).zone, Zone::Graveyard);
            branch.restore(&before).unwrap();
            big_helpers::settle(&mut branch);
            assert_eq!(branch.snapshot(), g.snapshot());
        }
    }
}
