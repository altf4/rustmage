//! Player-only solo attack events and exact attacking-object follow-up.
use super::*;

#[test]
fn jiang_requires_one_attacker_and_a_player_defender() {
    for player in [Player::P0, Player::P1] {
        for case in 0..3 {
            let mut game = scenario(player);
            let jiang = put(&mut game, player, "Jiang Yanggu, Alone", Zone::Battlefield);
            let attacker = put(&mut game, player, "Bear Cub", Zone::Battlefield);
            let other = put(&mut game, player, "Bear Cub", Zone::Battlefield);
            let walker = put(
                &mut game,
                player.other(),
                "Ajani, Caller of the Pride",
                Zone::Battlefield,
            );
            game.state.objects[walker as usize].counters.loyalty = 4;
            game.state.step = Step::Attackers;
            game.state.pending = Pending::Attackers;
            if case == 1 {
                let defender = game.object_ref(walker);
                act(
                    &mut game,
                    Action::AttackPlaneswalker {
                        card: attacker,
                        defender,
                    },
                );
            } else {
                act(&mut game, Action::Attack { card: attacker });
            }
            if case == 2 {
                act(&mut game, Action::Attack { card: other });
            }
            act(&mut game, Action::FinishAttackers);
            let queued = game
                .state
                .stack
                .iter()
                .chain(game.state.pending_triggers.iter())
                .filter(|item| {
                    item.card == jiang && matches!(item.kind, StackKind::TriggeredAbility { .. })
                })
                .count();
            assert_eq!(queued, usize::from(case == 0));
        }
    }
}

#[test]
fn jiang_counts_actual_discards_and_does_not_follow_a_new_incarnation() {
    for player in [Player::P0, Player::P1] {
        for blink in [false, true] {
            let mut game = scenario(player);
            put(&mut game, player, "Jiang Yanggu, Alone", Zone::Battlefield);
            let attacker = put(&mut game, player, "Bear Cub", Zone::Battlefield);
            let prior = put(&mut game, player, "Forest", Zone::Hand);
            let discard = put(&mut game, player, "Island", Zone::Hand);
            game.discard_batch(&[prior], None);
            assert_eq!(
                game.state.players[player.index()].cards_discarded_this_turn,
                1
            );
            game.state.step = Step::Attackers;
            game.state.pending = Pending::Attackers;
            act(&mut game, Action::Attack { card: attacker });
            act(&mut game, Action::FinishAttackers);
            if blink {
                game.move_object(attacker, Zone::Hand);
                game.move_object(attacker, Zone::Battlefield);
            }
            for _ in 0..32 {
                if game
                    .legal_actions()
                    .contains(&Action::Discard { card: discard })
                {
                    break;
                }
                choose(&mut game, |action| {
                    matches!(action, Action::Pass | Action::PutTrigger { .. })
                });
            }
            let before = game.state.players[player.index()].library.len();
            act(&mut game, Action::Discard { card: discard });
            settle(&mut game);
            assert_eq!(
                game.state.players[player.index()].cards_discarded_this_turn,
                2
            );
            assert_eq!(game.state.players[player.index()].library.len(), before - 1);
            assert_eq!(
                game.counter_count(attacker, CounterKind::PlusOne),
                if blink { 0 } else { 2 }
            );
            super::super::contracts::branch_contract(game);
        }
    }
}
