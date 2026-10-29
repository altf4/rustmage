use super::*;

#[test]
fn roiling_canopy_target_losing_control_or_reentering_is_illegal() {
    for player in [Player::P0, Player::P1] {
        for reenter in [false, true] {
            let mut g = scenario(player);
            let canopy = put(&mut g, player, "Roiling Canopy", Zone::Battlefield);
            for _ in 0..5 {
                put(&mut g, player, "Forest", Zone::Battlefield);
            }
            let target = put(&mut g, player, "Bear Cub", Zone::Battlefield);
            let incoming = put(&mut g, player, "Forest", Zone::Hand);
            g.move_object(incoming, Zone::Battlefield);
            g.prepare_triggers();
            for _ in 0..48 {
                if g.legal_actions().contains(&Action::ChooseTarget {
                    target: Target::Card(target),
                }) {
                    break;
                }
                choose(&mut g, |a| {
                    matches!(a, Action::Pass | Action::PutTrigger { .. })
                });
            }
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
            if g.legal_actions().contains(&Action::FinishTargets) {
                act(&mut g, Action::FinishTargets);
            }
            g.move_object(canopy, Zone::Graveyard);
            if reenter {
                let old = g.object_ref(target);
                g.move_object(target, Zone::Exile);
                g.move_object(target, Zone::Battlefield);
                assert_ne!(g.object_ref(target), old);
            } else {
                let reference = g.object_ref(target);
                g.change_controller(reference, player.other());
            }
            let saved = g.snapshot();
            for replay in 0..2 {
                if replay == 1 {
                    g.restore(&saved).unwrap();
                }
                settle(&mut g);
                assert_eq!(stats(&g, target), (2, 2));
            }
        }
    }
}
