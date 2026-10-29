use super::*;

#[test]
fn your_fate_ends_here_checks_mana_value_and_surveils_only_if_it_resolves() {
    for player in [Player::P0, Player::P1] {
        for case in 0..3 {
            let mut game = scenario(player);
            let small = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
            let target = put(
                &mut game,
                player.other(),
                "Budding Insurgent",
                Zone::Battlefield,
            );
            if case == 1 {
                game.state.objects[target as usize].gained_keywords = Keywords::INDESTRUCTIBLE;
            }
            let spell = put(&mut game, player, "Your Fate Ends Here", Zone::Hand);
            act(&mut game, Action::AnnounceCast { card: spell });
            reject(
                &mut game,
                Action::ChooseTarget {
                    target: Target::Card(small),
                },
            );
            act(
                &mut game,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
            pay(&mut game);
            if case == 2 {
                // Opponent responds with Unsummon, making the only target illegal.
                act(&mut game, Action::Pass);
                let bounce = put(&mut game, player.other(), "Unsummon", Zone::Hand);
                cast(&mut game, bounce, Some(Target::Card(target)));
            }
            for _ in 0..48 {
                if game.state.stack.is_empty()
                    && game.state.resolving.is_none()
                    && game.state.pending == Pending::Priority
                {
                    break;
                }
                choose(&mut game, |a| {
                    matches!(
                        a,
                        Action::Pass
                            | Action::PutTrigger { .. }
                            | Action::ArrangeLibrary {
                                destination: LibraryDestination::Top,
                                ..
                            }
                    )
                });
            }
            assert!(game.state.stack.is_empty() && game.state.resolving.is_none());
            assert_eq!(
                game.object_view(target).zone,
                match case {
                    0 => Zone::Graveyard,
                    1 => Zone::Battlefield,
                    _ => Zone::Hand,
                }
            );
            assert_eq!(
                game.state.players[player.index()].scried_or_surveilled_this_turn,
                case != 2
            );
        }
    }
}
