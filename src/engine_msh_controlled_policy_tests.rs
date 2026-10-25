//! Paid resolution mana policy keeps the payer distinct from the external decider.
use super::assurance::big_helpers::{cast, checked, named, settle, setup};
use super::msh_thirteenth_combined_tests::owned_checked_step;
use super::*;

#[test]
fn msh_controlled_policy_baxter_extort_scores_and_spends_actual_payer_pool() {
    for payer in [Player::P0, Player::P1] {
        for controlled in [false, true] {
            for external_pool in [[0; 6], [7, 2, 3, 4, 5, 6]] {
                let mut g = setup(payer);
                for player in &mut g.state.players {
                    player.mana = [0; 6];
                }
                let king = named(&mut g, payer, "The Kingpin of Crime", Zone::Hand);
                g.state.players[payer.index()].mana = [0, 0, 1, 0, 0, 3];
                cast(&mut g, king, None, &[]);
                settle(&mut g);
                assert_eq!(g.object_view(king).zone, Zone::Battlefield);
                let baxter = named(&mut g, payer, "Baxter Building", Zone::Hand);
                if g.actor() != Some(payer) {
                    checked(&mut g, Action::Pass);
                }
                checked(&mut g, Action::PlayLand { card: baxter });
                // Bounded control-entitlement seam; paid Cube scheduling is tested
                // separately. The Kingpin, land play, spell and mana below are real.
                if controlled {
                    g.schedule_player_control(payer, payer.other());
                    g.begin_player_control_turn();
                }
                g.state.players[payer.index()].mana = [0, 0, 0, 0, 0, 4];
                g.state.players[payer.other().index()].mana = external_pool;
                let spell = named(&mut g, payer, "Mind Stone", Zone::Hand);
                // Preserve exactly four C for Baxter after the actual two-mana cast.
                g.state.players[payer.index()].mana[5] += 2;
                cast(&mut g, spell, None, &[]);
                g.prepare_triggers();
                for _ in 0..30 {
                    if g.observe(g.actor().unwrap()).resolution_payment.is_some() {
                        break;
                    }
                    let action = g
                        .legal_actions()
                        .into_iter()
                        .find(|a| {
                            matches!(
                                a,
                                Action::Pass | Action::PutTrigger { .. } | Action::FinishTargets
                            )
                        })
                        .expect("advance real Extort trigger");
                    checked(&mut g, action);
                }
                let decider = if controlled { payer.other() } else { payer };
                assert_eq!(g.actor(), Some(decider));
                assert_eq!(g.semantic_actor(), Some(payer));
                let observation = g.observe(decider);
                assert_eq!(observation.viewer, decider);
                assert_eq!(observation.resolution_payment.unwrap().payer, payer);
                assert_eq!(g.state.players[payer.index()].mana, [0, 0, 0, 0, 0, 4]);
                let actions = g.legal_actions();
                assert!(actions.iter().any(|a| matches!(a, Action::ActivateMana {
                    card, ability: 1, payment: [0, 0, 0, 0, 0, 4], ..
                } if *card == baxter)));
                let before = g.snapshot();
                let index = crate::policy::smoke_index(&g, &actions);
                assert_eq!(g.snapshot(), before, "policy must be read-only");
                let action = actions[index].clone();
                let production = match &action {
                    Action::ActivateMana {
                        card,
                        ability: 1,
                        payment,
                        production,
                        ..
                    } if *card == baxter && *payment == [0, 0, 0, 0, 0, 4] => *production,
                    _ => panic!("policy must fund the printed W/B Extort: {action:?}"),
                };
                assert!(production[0] + production[2] > 0);
                g = owned_checked_step(g, decider, index, action);
                assert_eq!(g.state.players[payer.index()].mana, production);
                assert_eq!(g.state.players[payer.other().index()].mana, external_pool);
                assert!(g.object_view(baxter).tapped);
                let actions = g.legal_actions();
                let index = crate::policy::smoke_index(&g, &actions);
                let action = actions[index].clone();
                assert!(matches!(action, Action::PayResolutionMana { .. }));
                g = owned_checked_step(g, decider, index, action);
                settle(&mut g);
                assert_eq!(g.state.players[payer.index()].life, 21);
                assert_eq!(g.state.players[payer.other().index()].life, 19);
                assert_eq!(g.state.players[payer.other().index()].mana, external_pool);
            }
        }
    }
}
