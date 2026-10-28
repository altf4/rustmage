use super::*;

#[test]
fn awaken_landcycling_searches_or_fails_to_find_and_restores() {
    for player in [Player::P0, Player::P1] {
        for find in [false, true] {
            let mut game = scenario(player);
            let spell = put(&mut game, player, "Awaken the Inferno", Zone::Hand);
            let land = put(&mut game, player, "Forest", Zone::Library);
            let nonland = put(&mut game, player, "Bear Cub", Zone::Library);
            game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 1];
            let activation = Action::AnnounceActivate {
                card: spell,
                ability: 0,
                counter_payment: Counters::default(),
            };
            reject(&mut game, activation.clone());
            assert_eq!(game.object_view(spell).zone, Zone::Hand);
            game.state.players[player.index()].mana[5] = 2;
            act(&mut game, activation);
            pay(&mut game);
            assert_eq!(game.object_view(spell).zone, Zone::Graveyard);
            let saved = game.snapshot();
            for replay in 0..2 {
                if replay == 1 {
                    game.restore(&saved).unwrap();
                }
                let mut searched = false;
                for _ in 0..64 {
                    let actions = game.legal_actions();
                    if actions.contains(&Action::ChooseSearch { card: land }) && find && !searched {
                        reject(&mut game, Action::ChooseSearch { card: nonland });
                        super::super::contracts::branch_contract(game.clone());
                        act(&mut game, Action::ChooseSearch { card: land });
                        searched = true;
                    } else if actions.contains(&Action::FinishSearch) {
                        act(&mut game, Action::FinishSearch);
                        searched = true;
                    } else if searched
                        && game.state.pending == Pending::Priority
                        && game.state.stack.is_empty()
                        && game.state.pending_triggers.is_empty()
                        && game.state.resolving.is_none()
                    {
                        break;
                    } else if let Some(action) = actions
                        .into_iter()
                        .find(|a| matches!(a, Action::Pass | Action::PutTrigger { .. }))
                    {
                        act(&mut game, action);
                    } else {
                        panic!("unexpected landcycling choice {:?}", game.state.pending);
                    }
                }
                assert!(searched);
                assert_eq!(
                    game.object_view(land).zone,
                    if find { Zone::Hand } else { Zone::Library }
                );
                assert_eq!(game.object_view(nonland).zone, Zone::Library);
                assert_eq!(game.object_view(spell).zone, Zone::Graveyard);
            }
        }
    }
}
