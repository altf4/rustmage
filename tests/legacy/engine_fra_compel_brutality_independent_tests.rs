use super::*;

#[test]
fn compel_reentered_operand_is_not_the_original_target() {
    for player in [Player::P0, Player::P1] {
        for mode in 0..=1 {
            for source_leaves in [false, true] {
                let mut g = scenario(player);
                let source = put(
                    &mut g,
                    player,
                    if mode == 0 {
                        "Bear Cub"
                    } else {
                        "Ajani, Caller of the Pride"
                    },
                    Zone::Battlefield,
                );
                if mode == 1 {
                    g.state.objects[source as usize].counters.loyalty = 4;
                }
                let target = put(&mut g, player.other(), "Hill Giant", Zone::Battlefield);
                let spell = put(&mut g, player, "Compel Brutality", Zone::Hand);
                act(&mut g, Action::AnnounceCast { card: spell });
                act(&mut g, Action::ChooseMode { mode });
                act(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(source),
                    },
                );
                act(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(target),
                    },
                );
                if g.legal_actions().contains(&Action::FinishTargets) {
                    act(&mut g, Action::FinishTargets);
                }
                pay(&mut g);
                let changed = if source_leaves { source } else { target };
                let old = g.object_ref(changed);
                g.move_object(changed, Zone::Exile);
                g.move_object(changed, Zone::Battlefield);
                assert_ne!(g.object_ref(changed), old);
                let saved = g.snapshot();
                for replay in 0..2 {
                    if replay == 1 {
                        g.restore(&saved).unwrap();
                    }
                    settle(&mut g);
                    assert_eq!(g.object_view(target).damage, 0);
                }
            }
        }
    }
}

#[test]
fn compel_target_losing_required_type_prevents_damage() {
    for player in [Player::P0, Player::P1] {
        for mode in 0..=1 {
            for source_changes in [false, true] {
                let mut g = scenario(player);
                let source = put(
                    &mut g,
                    player,
                    if mode == 0 {
                        "Bear Cub"
                    } else {
                        "Ajani, Caller of the Pride"
                    },
                    Zone::Battlefield,
                );
                if mode == 1 {
                    g.state.objects[source as usize].counters.loyalty = 4;
                }
                let target = put(&mut g, player.other(), "Hill Giant", Zone::Battlefield);
                let spell = put(&mut g, player, "Compel Brutality", Zone::Hand);
                act(&mut g, Action::AnnounceCast { card: spell });
                act(&mut g, Action::ChooseMode { mode });
                act(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(source),
                    },
                );
                act(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(target),
                    },
                );
                if g.legal_actions().contains(&Action::FinishTargets) {
                    act(&mut g, Action::FinishTargets);
                }
                pay(&mut g);
                let changed = if source_changes { source } else { target };
                let timestamp = g.next_role_timestamp();
                let reference = g.object_ref(changed);
                g.state
                    .roles
                    .get_or_insert_with(Default::default)
                    .animations
                    .push(characteristic_layers::food_form(reference, timestamp));
                assert_eq!(g.current_kinds(changed) & (1 | 64), 0);
                let saved = g.snapshot();
                for replay in 0..2 {
                    if replay == 1 {
                        g.restore(&saved).unwrap();
                    }
                    settle(&mut g);
                    assert_eq!(g.object_view(target).damage, 0);
                }
            }
        }
    }
}

#[test]
fn compel_copied_modes_can_retarget_both_groups() {
    for player in [Player::P0, Player::P1] {
        for mode in 0..=1 {
            let mut g = scenario(player);
            let way = put(&mut g, player, "Way of the Cryomancer", Zone::Hand);
            cast(&mut g, way, None);
            settle(&mut g);
            let jace = g
                .state
                .objects
                .iter()
                .enumerate()
                .find_map(|(id, o)| {
                    (o.zone == Zone::Battlefield
                        && o.controller == player
                        && o.is_token()
                        && g.has_subtype(id as CardId, "Jace"))
                    .then_some(id as CardId)
                })
                .unwrap();
            let walker_ref = g.object_ref(jace);
            let grantor_ref = g.object_ref(way);
            act(
                &mut g,
                Action::AnnouncePlaneswalkerGrant {
                    source: walker_ref,
                    grantor: grantor_ref,
                    ability: 0,
                },
            );
            pay(&mut g);
            settle(&mut g);
            let source = put(
                &mut g,
                player,
                if mode == 0 {
                    "Bear Cub"
                } else {
                    "Ajani, Caller of the Pride"
                },
                Zone::Battlefield,
            );
            let other_source = if mode == 0 {
                put(&mut g, player, "Hill Giant", Zone::Battlefield)
            } else {
                jace
            };
            if mode == 1 {
                g.state.objects[source as usize].counters.loyalty = 3;
            }
            let target = put(&mut g, player.other(), "Hill Giant", Zone::Battlefield);
            let other_target = put(&mut g, player.other(), "Hill Giant", Zone::Battlefield);
            g.state.objects[target as usize].growth.toughness += 10;
            g.state.objects[other_target as usize].growth.toughness += 10;
            let spell = put(&mut g, player, "Compel Brutality", Zone::Hand);
            act(&mut g, Action::AnnounceCast { card: spell });
            act(&mut g, Action::ChooseMode { mode });
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(source),
                },
            );
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
            if g.legal_actions().contains(&Action::FinishTargets) {
                act(&mut g, Action::FinishTargets);
            }
            pay(&mut g);
            for _ in 0..32 {
                if matches!(g.state.pending, Pending::Retarget { .. }) {
                    break;
                }
                choose(&mut g, |a| {
                    matches!(a, Action::Pass | Action::PutTrigger { .. })
                });
            }
            assert!(matches!(g.state.pending, Pending::Retarget { .. }));
            act(
                &mut g,
                Action::SetStackTarget {
                    index: 0,
                    target: Target::Card(other_source),
                },
            );
            act(
                &mut g,
                Action::SetStackTarget {
                    index: 1,
                    target: Target::Card(other_target),
                },
            );
            act(&mut g, Action::FinishStackTargets);
            let saved = g.snapshot();
            for replay in 0..2 {
                if replay == 1 {
                    g.restore(&saved).unwrap();
                }
                settle(&mut g);
                assert_eq!(g.object_view(target).damage, if mode == 0 { 2 } else { 3 });
                assert_eq!(
                    g.object_view(other_target).damage,
                    if mode == 0 { 3 } else { 2 }
                );
            }
        }
    }
}
