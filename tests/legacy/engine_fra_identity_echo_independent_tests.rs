use super::*;

fn begin_echo(game: &mut Game, echo: CardId, target: CardId) {
    act(
        game,
        Action::AnnounceActivate {
            card: echo,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    act(
        game,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    pay(game);
    for _ in 0..48 {
        if matches!(game.state.pending, Pending::EffectLibrarySelection { .. }) {
            return;
        }
        choose(game, |a| {
            matches!(a, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("Identity Echo reveal did not start");
}

#[test]
fn identity_echo_preserves_selected_and_randomizes_only_the_rest_when_entry_is_prevented() {
    // No admitted effect prevents this entry. Inject the existing skipped-move
    // result into a real entry-choice batch, as in the shared friendship tests.
    for player in [Player::P0, Player::P1] {
        for prevented in [false, true] {
            let mut game = scenario(player);
            let echo = put(&mut game, player, "Identity Echo", Zone::Battlefield);
            let target = put(&mut game, player, "Bear Cub", Zone::Battlefield);
            let below = put(&mut game, player, "Shock", Zone::Library);
            let selected = put(&mut game, player, "Adaptive Automaton", Zone::Library);
            let above1 = put(&mut game, player, "Forest", Zone::Library);
            let above2 = put(&mut game, player, "Mountain", Zone::Library);
            begin_echo(&mut game, echo, target);
            for viewer in [player, player.other()] {
                let reveal = game.observe(viewer).library_selection.unwrap();
                assert!(reveal.public);
                assert_eq!(
                    reveal.remaining.iter().map(|v| v.id).collect::<Vec<_>>(),
                    vec![above2, above1, selected]
                );
            }
            let mut expected_rng = game.state.rng;
            let mut rest = vec![game.object_ref(above2), game.object_ref(above1)];
            expected_rng.shuffle(&mut rest);
            let mut expected_library = game.state.players[player.index()].library.to_vec();
            expected_library.retain(|id| !rest.iter().any(|object| object.card == *id));
            expected_library.splice(0..0, rest.iter().rev().map(|object| object.card));
            if !prevented {
                expected_library.retain(|&id| id != selected);
            }
            let selected_ref = game.object_ref(selected);
            act(&mut game, Action::SelectLibrary { card: selected });
            act(&mut game, Action::FinishLibrarySelection);
            assert!(matches!(game.state.pending, Pending::EntryValue { .. }));
            game.move_object(echo, Zone::Graveyard);
            if prevented {
                let batch = game.state.entry_batch.as_mut().unwrap();
                assert_eq!(batch.moves[0].before.reference, selected_ref);
                batch.moves[0].skipped = true;
            }
            let saved = game.snapshot();
            super::super::contracts::branch_contract(game.clone());
            for replay in 0..2 {
                if replay == 1 {
                    game.restore(&saved).unwrap();
                }
                choose(
                    &mut game,
                    |a| matches!(a, Action::ChooseEntryValue { value: EntryChoiceValue::CreatureType(t) } if t.name() == "Dwarf"),
                );
                settle(&mut game);
                assert_eq!(
                    game.object_view(selected).zone,
                    if prevented {
                        Zone::Library
                    } else {
                        Zone::Battlefield
                    }
                );
                assert_eq!(
                    game.state.players[player.index()].library.to_vec(),
                    expected_library
                );
                assert_eq!(game.state.rng, expected_rng);
                if prevented {
                    assert_eq!(
                        game.state.players[player.index()].library.last(),
                        Some(&selected)
                    );
                } else {
                    assert_eq!(
                        game.state.players[player.index()].library.last(),
                        Some(&below)
                    );
                }
                assert!(game.observe(player.other()).library_selection.is_none());
            }
        }
    }
}

#[test]
fn identity_echo_sorcery_timing_and_target_incarnation_are_enforced() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let echo = put(&mut game, player, "Identity Echo", Zone::Battlefield);
        let target = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let action = Action::AnnounceActivate {
            card: echo,
            ability: 0,
            counter_payment: Counters::default(),
        };
        game.state.step = Step::BeginCombat;
        reject(&mut game, action.clone());
        game.state.step = Step::Main1;
        act(&mut game, action);
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        pay(&mut game);
        let old = game.object_ref(target);
        game.move_object(target, Zone::Exile);
        game.move_object(target, Zone::Battlefield);
        assert_ne!(game.object_ref(target), old);
        let library = game.state.players[player.index()].library.clone();
        let saved = game.snapshot();
        settle(&mut game);
        assert_eq!(game.object_view(target).zone, Zone::Battlefield);
        assert_eq!(game.state.players[player.index()].library, library);
        game.restore(&saved).unwrap();
        settle(&mut game);
        assert_eq!(game.state.players[player.index()].library, library);
    }
}
