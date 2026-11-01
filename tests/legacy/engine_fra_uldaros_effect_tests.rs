//! Uldaros remains gated until its complete interaction suite is validated.
use super::*;

#[test]
fn exile_targets_creates_independent_budgeted_card_copies_and_restores() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let source = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let first = put(&mut game, player, "Bear Cub", Zone::Graveyard);
        let second = put(&mut game, player, "Gilded Lotus", Zone::Graveyard);
        let stale = put(&mut game, player, "Shock", Zone::Graveyard);
        let mut item = game.activation_item(player, source, 0);
        item.targets = [first, second, stale]
            .into_iter()
            .map(|id| BoundTarget {
                group: 0,
                target: Target::Card(id),
                object: Some(game.object_ref(id)),
            })
            .collect();
        game.move_object(stale, Zone::Exile);
        let snapshot = game.snapshot();
        for _ in 0..2 {
            assert!(game.exile_targets_copy_free_with_budget(&item, 6));
            assert_eq!(game.state.objects[first as usize].zone, Zone::Exile);
            assert_eq!(game.state.objects[second as usize].zone, Zone::Exile);
            let offer = game.state.resolution_free_casts.as_ref().unwrap();
            assert_eq!(offer.player, player);
            assert_eq!(offer.remaining_mana_value, Some(6));
            assert_eq!(offer.remaining.len(), 2);
            for reference in &offer.remaining {
                let copy = &game.state.objects[reference.card as usize];
                assert_eq!(copy.copy_kind, CopyKind::Card);
                assert_eq!(copy.owner, player);
                assert_eq!(copy.zone, Zone::Exile);
            }
            game.restore(&snapshot).unwrap();
        }
    }
}

#[test]
fn cast_uldaros_resolves_selected_types_and_excludes_departed_target() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let creature = put(&mut game, player, "Bear Cub", Zone::Graveyard);
        let artifact = put(&mut game, player, "Gilded Lotus", Zone::Graveyard);
        let source = put(&mut game, player, "Shock", Zone::Hand);
        game.state.objects[source as usize].card = Card::test_index(4739);
        // Verify the gated fixture index rather than accidentally testing a different card.
        assert_eq!(
            game.state.objects[source as usize].card.identity(),
            "Uldaros Theorix"
        );
        act(&mut game, Action::AnnounceCast { card: source });
        pay(&mut game);
        for _ in 0..40 {
            if matches!(game.state.pending, Pending::Targets { .. }) {
                break;
            }
            act(&mut game, Action::Pass);
        }
        act(
            &mut game,
            Action::ChooseTargetCardType {
                target: Target::Card(creature),
                type_mask: 1,
            },
        );
        act(
            &mut game,
            Action::ChooseTargetCardType {
                target: Target::Card(artifact),
                type_mask: 16,
            },
        );
        act(&mut game, Action::FinishTargets);
        game.move_object(artifact, Zone::Hand);
        let saved = game.snapshot();
        for _ in 0..2 {
            game.resolve_top();
            assert_eq!(game.state.objects[creature as usize].zone, Zone::Exile);
            assert_eq!(game.state.objects[artifact as usize].zone, Zone::Hand);
            let offer = game.state.resolution_free_casts.as_ref().unwrap();
            assert_eq!(offer.remaining_mana_value, Some(6));
            assert_eq!(offer.remaining.len(), 1);
            assert_eq!(
                game.state.objects[offer.remaining[0].card as usize]
                    .card
                    .identity(),
                "Bear Cub"
            );
            game.restore(&saved).unwrap();
        }
    }
}

// Start the printed entry trigger through normal cast and target actions.
fn uldaros_trigger(game: &mut Game, player: Player, targets: &[(CardId, u16)]) -> CardId {
    let source = put(game, player, "Shock", Zone::Hand);
    game.state.objects[source as usize].card = Card::test_index(4739);
    act(game, Action::AnnounceCast { card: source });
    pay(game);
    for _ in 0..40 {
        if matches!(game.state.pending, Pending::Targets { .. }) {
            break;
        }
        act(game, Action::Pass);
    }
    for &(id, type_mask) in targets {
        act(
            game,
            Action::ChooseTargetCardType {
                target: Target::Card(id),
                type_mask,
            },
        );
    }
    act(game, Action::FinishTargets);
    source
}

#[test]
fn uldaros_copies_cast_for_free_consume_budget_and_replay_observations() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let creature = put(&mut game, player, "Bear Cub", Zone::Graveyard);
        let artifact = put(&mut game, player, "Gilded Lotus", Zone::Graveyard);
        let foreign = put(&mut game, player.other(), "Shock", Zone::Graveyard);
        let source = put(&mut game, player, "Shock", Zone::Hand);
        game.state.objects[source as usize].card = Card::test_index(4739);
        act(&mut game, Action::AnnounceCast { card: source });
        pay(&mut game);
        for _ in 0..40 {
            if matches!(game.state.pending, Pending::Targets { .. }) {
                break;
            }
            act(&mut game, Action::Pass);
        }
        assert!(
            !game
                .legal_actions()
                .contains(&Action::ChooseTargetCardType {
                    target: Target::Card(foreign),
                    type_mask: 8,
                })
        );
        for (id, type_mask) in [(creature, 1), (artifact, 16)] {
            act(
                &mut game,
                Action::ChooseTargetCardType {
                    target: Target::Card(id),
                    type_mask,
                },
            );
        }
        act(&mut game, Action::FinishTargets);
        game.resolve_top();
        let copies = game
            .state
            .resolution_free_casts
            .as_ref()
            .unwrap()
            .remaining
            .clone();
        let bear = copies
            .iter()
            .copied()
            .find(|reference| {
                game.state.objects[reference.card as usize].card.identity() == "Bear Cub"
            })
            .unwrap();
        let lotus = copies
            .iter()
            .copied()
            .find(|reference| {
                game.state.objects[reference.card as usize].card.identity() == "Gilded Lotus"
            })
            .unwrap();
        let views = [player, player.other()].map(|viewer| {
            let mut view = game.observe(viewer);
            view.revision = 0;
            serde_json::to_string(&view).unwrap()
        });
        let revision = game.revision();
        let checkpoint = game.snapshot();
        for _ in 0..2 {
            assert_eq!(
                [player, player.other()].map(|viewer| {
                    let mut view = game.observe(viewer);
                    view.revision = 0;
                    serde_json::to_string(&view).unwrap()
                }),
                views
            );
            let cast = Action::CastExiledDuringResolution {
                object: bear,
                face: CastFace::Normal,
            };
            assert!(game.legal_actions().contains(&cast));
            act(&mut game, cast);
            pay(&mut game);
            assert_eq!(
                game.state
                    .resolution_free_casts
                    .as_ref()
                    .unwrap()
                    .remaining_mana_value,
                Some(4)
            );
            assert!(
                !game
                    .legal_actions()
                    .contains(&Action::CastExiledDuringResolution {
                        object: lotus,
                        face: CastFace::Normal,
                    })
            );
            act(&mut game, Action::FinishResolutionCasts);
            game.resolve_top();
            assert_eq!(
                game.state.objects[bear.card as usize].zone,
                Zone::Battlefield
            );
            assert!(game.state.objects[bear.card as usize].is_token());
            game.restore(&checkpoint).unwrap();
            assert!(game.revision() > revision);
            assert_eq!(
                game.apply_index(player, revision, 0),
                Err(Error::StaleRevision)
            );
        }
    }
}

#[test]
fn uldaros_all_targets_illegal_fizzles_even_when_source_left() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let target = put(&mut game, player, "Bear Cub", Zone::Graveyard);
        let source = uldaros_trigger(&mut game, player, &[(target, 1)]);
        game.move_object(source, Zone::Graveyard);
        game.move_object(target, Zone::Hand);
        let saved = game.snapshot();
        for _ in 0..2 {
            game.resolve_top();
            assert_eq!(game.state.objects[target as usize].zone, Zone::Hand);
            assert!(game.state.resolution_free_casts.is_none());
            game.restore(&saved).unwrap();
        }
    }
}

#[test]
fn copied_uldaros_trigger_survives_source_departure() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let echo = put(
            &mut game,
            player,
            "Echo, Perceptive Prodigy",
            Zone::Battlefield,
        );
        let target = put(&mut game, player, "Bear Cub", Zone::Graveyard);
        let source = uldaros_trigger(&mut game, player, &[(target, 1)]);
        let original = game.state.stack.last().unwrap().id;
        act(
            &mut game,
            Action::AnnounceActivate {
                card: echo,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Stack(original),
            },
        );
        if game.legal_actions().contains(&Action::FinishTargets) {
            act(&mut game, Action::FinishTargets);
        }
        pay(&mut game);
        act(&mut game, Action::Pass);
        act(&mut game, Action::Pass);
        act(&mut game, Action::FinishStackTargets);
        game.move_object(source, Zone::Graveyard);
        let saved = game.snapshot();
        for _ in 0..2 {
            game.resolve_top();
            assert_eq!(game.state.objects[target as usize].zone, Zone::Exile);
            assert_eq!(
                game.state.resolution_free_casts.as_ref().unwrap().player,
                player
            );
            act(&mut game, Action::FinishResolutionCasts);
            game.resolve_top();
            assert!(game.state.resolution_free_casts.is_none());
            game.restore(&saved).unwrap();
        }
    }
}

#[test]
fn uldaros_retains_declared_type_slot_when_multitype_card_changes() {
    for player in [Player::P0, Player::P1] {
        for slot in [1, 16] {
            let mut game = scenario(player);
            let target = put(&mut game, player, "Solemn Simulacrum", Zone::Graveyard);
            uldaros_trigger(&mut game, player, &[(target, slot)]);
            // Synthetic characteristic change: no supported effect changes a
            // graveyard card's type in this checkout. Keep the same incarnation.
            game.state.objects[target as usize].card = Card::from_name("Bear Cub").unwrap();
            let saved = game.snapshot();
            for _ in 0..2 {
                game.resolve_top();
                if slot == 1 {
                    assert_eq!(game.state.objects[target as usize].zone, Zone::Exile);
                    assert_eq!(
                        game.state
                            .resolution_free_casts
                            .as_ref()
                            .unwrap()
                            .remaining
                            .len(),
                        1
                    );
                } else {
                    assert_eq!(game.state.objects[target as usize].zone, Zone::Graveyard);
                    assert!(game.state.resolution_free_casts.is_none());
                }
                game.restore(&saved).unwrap();
            }
        }
    }
}
