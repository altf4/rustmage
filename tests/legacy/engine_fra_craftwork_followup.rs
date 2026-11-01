//! Source and target continuations for Craftwork Crusher.
use super::*;

#[test]
fn crusher_targeted_modes_keep_source_lki_and_continue_after_source_leaves() {
    for p in [Player::P0, Player::P1] {
        for mode in [0, 1] {
            let mut g = scenario(p);
            let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let crusher = put(&mut g, p, "Craftwork Crusher", Zone::Hand);
            let library = g.state.players[p.index()].library.len();
            cast(&mut g, crusher, None);
            for _ in 0..48 {
                if matches!(g.state.pending, Pending::Modes { .. }) {
                    break;
                }
                choose(&mut g, |a| {
                    matches!(a, Action::Pass | Action::PutTrigger { .. })
                });
            }
            act(&mut g, Action::ChooseMode { mode });
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
            g.move_object(crusher, Zone::Exile);
            let saved = g.snapshot();
            super::super::contracts::branch_contract(g.clone());
            for replay in 0..2 {
                if replay == 1 {
                    g.restore(&saved).unwrap();
                }
                settle(&mut g);
                assert_eq!(g.object_view(target).zone, Zone::Graveyard);
                assert_eq!(g.object_view(crusher).zone, Zone::Exile);
                let cadets = g
                    .state
                    .objects
                    .iter()
                    .filter(|o| {
                        o.zone == Zone::Battlefield
                            && o.controller == p
                            && o.is_token()
                            && o.card == Card::test_index(5298)
                    })
                    .count();
                assert_eq!(cadets, usize::from(mode == 0));
                assert_eq!(
                    g.state.players[p.index()].library.len(),
                    library - usize::from(mode == 1)
                );
            }
        }
    }
}

#[test]
fn crusher_target_gaining_hexproof_counters_all_chosen_modes() {
    for p in [Player::P0, Player::P1] {
        for mode in [0, 1] {
            let mut g = scenario(p);
            let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let crusher = put(&mut g, p, "Craftwork Crusher", Zone::Hand);
            let library = g.state.players[p.index()].library.len();
            cast(&mut g, crusher, None);
            for _ in 0..48 {
                if matches!(g.state.pending, Pending::Modes { .. }) {
                    break;
                }
                choose(&mut g, |a| {
                    matches!(a, Action::Pass | Action::PutTrigger { .. })
                });
            }
            act(&mut g, Action::ChooseMode { mode });
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
            g.state.objects[target as usize].gained_keywords = Keywords::HEXPROOF;
            settle(&mut g);
            assert_eq!(g.object_view(target).zone, Zone::Battlefield);
            assert_eq!(g.state.players[p.index()].library.len(), library);
            assert!(!g.state.objects.iter().any(|o| o.zone == Zone::Battlefield
                && o.controller == p
                && o.is_token()
                && o.card == Card::test_index(5298)));
        }
    }
}
