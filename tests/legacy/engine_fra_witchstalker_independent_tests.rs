use super::*;

#[test]
fn witchstalker_can_return_preexisting_graveyard_land_after_full_mill() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let old_land = put(&mut g, player, "Forest", Zone::Graveyard);
        let wolf = put(&mut g, player, "Primal Witchstalker", Zone::Hand);
        cast(&mut g, wolf, None);
        let mut selected = false;
        for _ in 0..48 {
            g.prepare_triggers();
            let actions = g.legal_actions();
            if actions.contains(&Action::ChooseTarget {
                target: Target::Card(old_land),
            }) {
                act(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(old_land),
                    },
                );
                if g.legal_actions().contains(&Action::FinishTargets) {
                    act(&mut g, Action::FinishTargets);
                }
                selected = true;
                break;
            }
            choose(&mut g, |a| {
                matches!(a, Action::Pass | Action::PutTrigger { .. })
            });
        }
        assert!(selected);
        let saved = g.snapshot();
        for replay in 0..2 {
            if replay == 1 {
                g.restore(&saved).unwrap();
            }
            settle(&mut g);
            assert_eq!(g.object_view(old_land).zone, Zone::Battlefield);
            assert!(g.object_view(old_land).tapped);
        }
    }
}

#[test]
fn witchstalker_counts_self_shuffle_as_part_of_mill_without_faking_a_zone_change() {
    for player in [Player::P0, Player::P1] {
        for total in [3, 4] {
            let mut game = scenario(player);
            let old_land = put(&mut game, player, "Forest", Zone::Graveyard);
            game.state.players[player.index()].library.clear();
            for _ in 1..total {
                put(&mut game, player, "Mountain", Zone::Library);
            }
            let progenitus = put(&mut game, player, "Progenitus", Zone::Library);
            let original = game.object_ref(progenitus);
            let wolf = put(&mut game, player, "Primal Witchstalker", Zone::Hand);
            cast(&mut game, wolf, None);
            let saved = game.snapshot();
            for replay in 0..2 {
                if replay == 1 {
                    game.restore(&saved).unwrap();
                }
                let mut chosen = false;
                for _ in 0..64 {
                    game.prepare_triggers();
                    let action = Action::ChooseTarget {
                        target: Target::Card(old_land),
                    };
                    if game.legal_actions().contains(&action) {
                        act(&mut game, action);
                        chosen = true;
                        break;
                    }
                    let Some(action) = game
                        .legal_actions()
                        .into_iter()
                        .find(|action| matches!(action, Action::Pass | Action::PutTrigger { .. }))
                    else {
                        break;
                    };
                    act(&mut game, action);
                    if game.state.stack.is_empty()
                        && game.state.pending_triggers.is_empty()
                        && game.state.ordering_triggers.is_empty()
                        && game.state.resolving.is_none()
                        && game.state.announcing.is_none()
                        && game.state.pending == Pending::Priority
                        && game.on_battlefield(wolf)
                    {
                        break;
                    }
                }
                assert_eq!(chosen, total == 4);
                if chosen {
                    settle(&mut game);
                }
                assert_eq!(game.object_ref(progenitus), original);
                assert_eq!(game.object_view(progenitus).zone, Zone::Library);
                assert_eq!(
                    game.object_view(old_land).zone,
                    if total == 4 {
                        Zone::Battlefield
                    } else {
                        Zone::Graveyard
                    }
                );
                if total == 4 {
                    assert!(game.object_view(old_land).tapped);
                }
                assert_eq!(
                    game.state.players[player.index()].cards_library_to_graveyard_this_turn,
                    total - 1
                );
            }
        }
    }
}
