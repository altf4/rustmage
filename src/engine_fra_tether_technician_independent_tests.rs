use super::*;

#[test]
fn tether_discard_replacement_and_target_reentry_fizzle_reflexive_damage() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        put(&mut g, player, "Rest in Peace", Zone::Battlefield);
        let discarded = put(&mut g, player, "Forest", Zone::Hand);
        let target = put(&mut g, player.other(), "Hill Giant", Zone::Battlefield);
        let technician = put(&mut g, player, "Tether Technician", Zone::Hand);
        cast(&mut g, technician, None);
        let mut paid = false;
        let mut targeted = false;
        for _ in 0..64 {
            g.prepare_triggers();
            let actions = g.legal_actions();
            if !paid && actions.contains(&Action::Discard { card: discarded }) {
                act(&mut g, Action::Discard { card: discarded });
                paid = true;
                assert_eq!(g.object_view(discarded).zone, Zone::Exile);
            } else if paid
                && !targeted
                && actions.contains(&Action::ChooseTarget {
                    target: Target::Card(target),
                })
            {
                act(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(target),
                    },
                );
                targeted = true;
                if g.legal_actions().contains(&Action::FinishTargets) {
                    act(&mut g, Action::FinishTargets);
                }
                break;
            } else if let Some(action) = actions
                .into_iter()
                .find(|a| matches!(a, Action::Pass | Action::PutTrigger { .. }))
            {
                act(&mut g, action);
            } else {
                panic!("unexpected reflexive state {:?}", g.state.pending);
            }
        }
        assert!(paid && targeted);
        let old = g.object_ref(target);
        g.move_object(target, Zone::Exile);
        g.move_object(target, Zone::Battlefield);
        assert_ne!(g.object_ref(target), old);
        let saved = g.snapshot();
        for replay in 0..2 {
            if replay == 1 {
                g.restore(&saved).unwrap();
            }
            settle(&mut g);
            assert_eq!(g.object_view(target).damage, 0);
        }
    }
}
