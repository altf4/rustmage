//! Independent source-less radiation stack ordering and observation tests.
use super::*;

#[test]
fn radiation_orders_with_a_sourced_main_phase_trigger_and_replays() {
    for player in [Player::P0, Player::P1] {
        for rad_first in [false, true] {
            let mut game = scenario(player);
            let source = put(
                &mut game,
                player,
                "Scathing Shadelock, Venomous Words",
                Zone::Battlefield,
            );
            let top = put(&mut game, player, "Shock", Zone::Library);
            game.state.players[player.index()]
                .add_player_counter(crate::cards::PlayerCounterKind::Rad, 1);
            game.begin_step(Step::Main1);
            game.prepare_triggers();
            assert!(
                matches!(game.state.pending, Pending::OrderTriggers { player: p } if p == player)
            );
            let rad = game.state.ordering_triggers.iter().find(|item| matches!(item.kind,
                StackKind::InherentRule { rule: SourceLessRule::Radiation, affected_player } if affected_player == player)).unwrap().id;
            let other = game
                .state
                .ordering_triggers
                .iter()
                .find(|item| item.card == source)
                .unwrap()
                .id;
            let snapshot = game.snapshot();
            for _ in 0..2 {
                let pick = Action::PutTrigger {
                    id: if rad_first { rad } else { other },
                };
                assert!(game.legal_actions().contains(&pick));
                act(&mut game, pick);
                assert_eq!(game.state.stack.len(), 2);
                assert_eq!(game.state.stack[0].id, if rad_first { rad } else { other });
                assert_eq!(game.state.stack[1].id, if rad_first { other } else { rad });
                for viewer in [player, player.other()] {
                    let view = serde_json::to_value(game.observe(viewer)).unwrap();
                    let packet = view["stack"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|item| item["id"] == rad)
                        .unwrap();
                    assert!(packet["card"].is_null());
                    assert!(packet["object"].is_null());
                    assert!(packet["source_card"].is_null());
                    assert!(packet["source_facts"].is_null());
                }
                assert!(game.counter_stack_item(other, false));
                game.resolve_top();
                assert_eq!(game.object_view(top).zone, Zone::Graveyard);
                assert_eq!(game.state.players[player.index()].life, 19);
                assert_eq!(
                    game.state.players[player.index()]
                        .player_counter(crate::cards::PlayerCounterKind::Rad),
                    0
                );
                game.restore(&snapshot).unwrap();
            }
        }
    }
}
