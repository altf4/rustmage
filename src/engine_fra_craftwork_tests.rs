//! Exact pairs of Craftwork Crusher's triggered modes.
use super::*;

#[test]
fn craftwork_three_distinct_mode_pairs_and_target_fizzle() {
    for player in [Player::P0, Player::P1] {
        for mode in 0..3 {
            for fizzle in [false, true] {
                if fizzle && mode == 2 {
                    continue;
                }
                let mut game = scenario(player);
                let target = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
                let crusher = put(&mut game, player, "Craftwork Crusher", Zone::Hand);
                let library_before = game.state.players[player.index()].library.len();
                cast(&mut game, crusher, None);
                for _ in 0..48 {
                    if matches!(game.state.pending, Pending::Modes { .. }) {
                        break;
                    }
                    choose(&mut game, |a| {
                        matches!(a, Action::Pass | Action::PutTrigger { .. })
                    });
                }
                act(&mut game, Action::ChooseMode { mode });
                if mode != 2 {
                    act(
                        &mut game,
                        Action::ChooseTarget {
                            target: Target::Card(target),
                        },
                    );
                    if fizzle {
                        game.move_object(target, Zone::Hand);
                    }
                }
                super::super::contracts::branch_contract(game.clone());
                settle(&mut game);
                assert_eq!(stats(&game, crusher), (7, 5));
                assert!(game.keywords(crusher).contains(Keywords::TRAMPLE));
                assert_eq!(
                    game.object_view(target).zone,
                    if fizzle {
                        Zone::Hand
                    } else if mode != 2 {
                        Zone::Graveyard
                    } else {
                        Zone::Battlefield
                    }
                );
                let cadets = game
                    .state
                    .objects
                    .iter()
                    .filter(|object| {
                        object.zone == Zone::Battlefield
                            && object.controller == player
                            && object.card == Card::test_index(5298)
                    })
                    .count();
                assert_eq!(cadets, usize::from(!fizzle && mode != 1));
                assert_eq!(
                    game.state.players[player.index()].library.len(),
                    library_before - usize::from(!fizzle && mode != 0)
                );
            }
        }
    }
}

#[test]
fn craftwork_nontargeted_pair_remains_legal_without_a_target() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let crusher = put(&mut game, player, "Craftwork Crusher", Zone::Hand);
        cast(&mut game, crusher, None);
        for _ in 0..48 {
            if matches!(game.state.pending, Pending::Modes { .. }) {
                break;
            }
            choose(&mut game, |a| {
                matches!(a, Action::Pass | Action::PutTrigger { .. })
            });
        }
        game.move_object(crusher, Zone::Exile);
        assert!(
            !game
                .legal_actions()
                .contains(&Action::ChooseMode { mode: 0 })
        );
        assert!(
            !game
                .legal_actions()
                .contains(&Action::ChooseMode { mode: 1 })
        );
        assert!(
            game.legal_actions()
                .contains(&Action::ChooseMode { mode: 2 })
        );
        act(&mut game, Action::ChooseMode { mode: 2 });
        settle(&mut game);
    }
}
