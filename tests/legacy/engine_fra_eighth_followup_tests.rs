use super::*;

#[test]
fn stingcaster_target_reentry_fizzles_but_haste_allows_attack() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let spell = put(&mut g, p, "Shock", Zone::Graveyard);
        let mage = put(&mut g, p, "Stingcaster Mage", Zone::Hand);
        cast(&mut g, mage, None);
        let mut selected = false;
        for _ in 0..48 {
            g.prepare_triggers();
            let choice = Action::ChooseTarget {
                target: Target::Card(spell),
            };
            if g.legal_actions().contains(&choice) {
                act(&mut g, choice);
                selected = true;
                break;
            }
            choose(&mut g, |a| {
                matches!(a, Action::Pass | Action::PutTrigger { .. })
            });
        }
        assert!(selected);
        let old = g.object_ref(spell);
        g.move_object(spell, Zone::Exile);
        g.move_object(spell, Zone::Graveyard);
        assert_ne!(old, g.object_ref(spell));
        super::super::contracts::branch_contract(g.clone());
        settle(&mut g);
        assert!(!g.has_graveyard_cast(p, spell, GraveyardCastKind::Flashback));
        assert!(g.keywords(mage).contains(Keywords::HASTE));
        assert!(g.object_view(mage).summoning_sick);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: mage });
        act(&mut g, Action::FinishAttackers);
        assert!(g.state.attackers.iter().any(|a| a.card == mage));
    }
}

#[test]
fn aerid_rejects_unfunded_and_queued_activation_survives_exile_replacement() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Rest in Peace", Zone::Battlefield);
        let source = put(&mut g, p, "Aerid Konstrari", Zone::Hand);
        cast(&mut g, source, None);
        settle(&mut g);
        assert!(g.keywords(source).contains(Keywords::FLYING));
        let action = Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        };
        // The entry trigger created a Heartwood token that can fund the cost.
        for object in &mut g.state.objects {
            if object.zone == Zone::Battlefield
                && object.controller == p
                && object.card == Card::test_index(5438)
            {
                object.tapped = true;
            }
        }
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 5];
        reject(&mut g, action.clone());
        assert_eq!(g.object_view(source).zone, Zone::Battlefield);
        g.state.players[p.index()].mana[5] = 6;
        act(&mut g, action);
        pay(&mut g);
        g.move_object(source, Zone::Graveyard);
        assert_eq!(g.object_view(source).zone, Zone::Exile);
        let saved = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        for replay in 0..2 {
            if replay == 1 {
                g.restore(&saved).unwrap();
            }
            settle(&mut g);
            let tokens: Vec<_> = g
                .state
                .objects
                .iter()
                .filter(|o| {
                    o.zone == Zone::Battlefield
                        && o.is_token()
                        && o.controller == p
                        && o.card == Card::test_index(5438)
                })
                .collect();
            assert_eq!(tokens.len(), 2);
            assert!(tokens.iter().all(|o| o.owner == p));
        }
    }
}

#[test]
fn mind_meanderer_can_decline_fight_and_vigilance_tracks_jace_control() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let source = put(&mut g, p, "Mind Meanderer", Zone::Hand);
        cast(&mut g, source, None);
        let mut declined = false;
        for _ in 0..48 {
            g.prepare_triggers();
            let actions = g.legal_actions();
            if actions.contains(&Action::FinishTargets) {
                act(&mut g, Action::FinishTargets);
                declined = true;
                break;
            }
            choose(&mut g, |a| {
                matches!(a, Action::Pass | Action::PutTrigger { .. })
            });
        }
        assert!(declined);
        settle(&mut g);
        assert_eq!(g.object_view(enemy).zone, Zone::Battlefield);
        assert!(!g.keywords(source).contains(Keywords::VIGILANCE));
        g.create_tokens(p.other(), Card::test_index(5435), 1, false);
        let jace = fra_jaces(&g, p.other())[0];
        g.add_counter(jace, CounterKind::Loyalty, 2);
        assert!(!g.keywords(source).contains(Keywords::VIGILANCE));
        let reference = g.object_ref(jace);
        g.change_controller(reference, p);
        assert!(g.keywords(source).contains(Keywords::VIGILANCE));
        let aura = put(&mut g, p.other(), "Witness Protection", Zone::Battlefield);
        g.state.objects[aura as usize].attached_to = Some(g.object_ref(source));
        assert!(!g.keywords(source).contains(Keywords::VIGILANCE));
        g.move_object(aura, Zone::Graveyard);
        assert!(g.keywords(source).contains(Keywords::VIGILANCE));
    }
}

#[test]
fn yoshimaru_fight_needs_both_targets_to_remain_legal() {
    for p in [Player::P0, Player::P1] {
        for friend_invalid in [false, true] {
            let mut g = scenario(p);
            let friend = put(&mut g, p, "Hill Giant", Zone::Battlefield);
            let enemy = put(&mut g, p.other(), "Gigantosaurus", Zone::Battlefield);
            let dog = put(&mut g, p, "Yoshimaru, Scrappy Stray", Zone::Hand);
            cast(&mut g, dog, None);
            let mut chosen = 0;
            for _ in 0..64 {
                g.prepare_triggers();
                let actions = g.legal_actions();
                let target = if chosen == 0 { friend } else { enemy };
                let choice = Action::ChooseTarget {
                    target: Target::Card(target),
                };
                if chosen < 2 && actions.contains(&choice) {
                    if chosen == 0 {
                        reject(
                            &mut g,
                            Action::ChooseTarget {
                                target: Target::Card(dog),
                            },
                        );
                    }
                    act(&mut g, choice);
                    chosen += 1;
                } else if chosen == 2 && g.state.pending == Pending::Priority {
                    break;
                } else if actions.contains(&Action::FinishTargets) {
                    act(&mut g, Action::FinishTargets);
                } else if let Some(a) = actions
                    .into_iter()
                    .find(|a| matches!(a, Action::Pass | Action::PutTrigger { .. }))
                {
                    act(&mut g, a);
                } else {
                    panic!("unexpected Yoshimaru targets {:?}", g.state.pending);
                }
            }
            assert_eq!(chosen, 2);
            let invalid = if friend_invalid { friend } else { enemy };
            let reference = g.object_ref(invalid);
            g.change_controller(reference, if friend_invalid { p.other() } else { p });
            let saved = g.snapshot();
            super::super::contracts::branch_contract(g.clone());
            for replay in 0..2 {
                if replay == 1 {
                    g.restore(&saved).unwrap();
                }
                settle(&mut g);
                assert_eq!(g.object_view(friend).zone, Zone::Battlefield);
                assert_eq!(g.object_view(enemy).zone, Zone::Battlefield);
                assert_eq!(g.state.objects[friend as usize].damage, 0);
                assert_eq!(g.state.objects[enemy as usize].damage, 0);
            }
        }
    }
}

fn stingcaster_grant(g: &mut Game, p: Player, spell: CardId) {
    let mage = put(g, p, "Stingcaster Mage", Zone::Hand);
    cast(g, mage, None);
    let mut selected = false;
    for _ in 0..48 {
        g.prepare_triggers();
        let choice = Action::ChooseTarget {
            target: Target::Card(spell),
        };
        if g.legal_actions().contains(&choice) {
            act(g, choice);
            selected = true;
            break;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    assert!(selected);
    settle(g);
    assert!(g.has_graveyard_cast(p, spell, GraveyardCastKind::Flashback));
}

#[test]
fn stingcaster_grants_hybrid_mana_cost_without_intrinsic_flashback_discard() {
    for p in [Player::P0, Player::P1] {
        for blue in [false, true] {
            let mut g = scenario(p);
            let spell = put(&mut g, p, "Twinned Vision", Zone::Graveyard);
            stingcaster_grant(&mut g, p, spell);
            let library = g.state.players[p.index()].library.len();
            g.state.players[p.index()].mana = if blue {
                [0, 1, 0, 0, 0, 1]
            } else {
                [0, 0, 0, 1, 0, 1]
            };
            act(
                &mut g,
                Action::AnnounceGrantedFlashback {
                    card: spell,
                    flash_surcharge: false,
                },
            );
            for _ in 0..16 {
                if g.state.announcing.is_none() {
                    break;
                }
                assert!(
                    !g.legal_actions()
                        .iter()
                        .any(|a| matches!(a, Action::ChooseAdditionalCost { .. }))
                );
                choose(&mut g, |a| {
                    matches!(a, Action::ChooseSpellCost { .. } | Action::PayCosts { .. })
                });
            }
            assert!(g.state.announcing.is_none());
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            let saved = g.snapshot();
            super::super::contracts::branch_contract(g.clone());
            for replay in 0..2 {
                if replay == 1 {
                    g.restore(&saved).unwrap();
                }
                settle(&mut g);
                assert_eq!(g.state.players[p.index()].library.len(), library - 2);
                assert_eq!(g.object_view(spell).zone, Zone::Exile);
            }
        }
    }
}

#[test]
fn stingcaster_granted_flashback_uses_chosen_x_and_full_printed_mana_cost() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let spell = put(&mut g, p, "Exsanguinate", Zone::Graveyard);
        stingcaster_grant(&mut g, p, spell);
        let life = g.state.players[p.other().index()].life;
        g.state.players[p.index()].mana = [0, 0, 2, 0, 0, 2];
        act(
            &mut g,
            Action::AnnounceGrantedFlashback {
                card: spell,
                flash_surcharge: false,
            },
        );
        act(&mut g, Action::ChooseX { value: 2 });
        pay(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        super::super::contracts::branch_contract(g.clone());
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, life - 2);
        assert_eq!(g.object_view(spell).zone, Zone::Exile);
    }
}

#[test]
fn mind_meanderer_fight_target_reentry_is_not_the_original_creature() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let source = put(&mut g, p, "Mind Meanderer", Zone::Hand);
        cast(&mut g, source, None);
        let mut chosen = false;
        for _ in 0..48 {
            g.prepare_triggers();
            let choice = Action::ChooseTarget {
                target: Target::Card(enemy),
            };
            if g.legal_actions().contains(&choice) {
                act(&mut g, choice);
                chosen = true;
                break;
            }
            choose(&mut g, |a| {
                matches!(a, Action::Pass | Action::PutTrigger { .. })
            });
        }
        assert!(chosen);
        let old = g.object_ref(enemy);
        g.move_object(enemy, Zone::Exile);
        g.move_object(enemy, Zone::Battlefield);
        assert_ne!(old, g.object_ref(enemy));
        let saved = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        for replay in 0..2 {
            if replay == 1 {
                g.restore(&saved).unwrap();
            }
            settle(&mut g);
            assert_eq!(g.object_view(enemy).zone, Zone::Battlefield);
            assert_eq!(g.state.objects[enemy as usize].damage, 0);
            assert_eq!(g.state.objects[source as usize].damage, 0);
        }
    }
}

#[test]
fn yoshimaru_may_omit_fight_opponent_and_counter_activation_fizzles_on_reentry() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let own = put(&mut g, p, "Hill Giant", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let dog = put(&mut g, p, "Yoshimaru, Scrappy Stray", Zone::Hand);
        cast(&mut g, dog, None);
        let mut own_selected = false;
        let mut omitted = false;
        for _ in 0..48 {
            g.prepare_triggers();
            let actions = g.legal_actions();
            let choice = Action::ChooseTarget {
                target: Target::Card(own),
            };
            if !own_selected && actions.contains(&choice) {
                act(&mut g, choice);
                own_selected = true;
            } else if own_selected && actions.contains(&Action::FinishTargets) {
                act(&mut g, Action::FinishTargets);
                omitted = true;
                break;
            } else {
                choose(&mut g, |a| {
                    matches!(a, Action::Pass | Action::PutTrigger { .. })
                });
            }
        }
        assert!(own_selected && omitted);
        settle(&mut g);
        assert_eq!(g.state.objects[own as usize].damage, 0);
        assert_eq!(g.state.objects[enemy as usize].damage, 0);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 6];
        act(
            &mut g,
            Action::AnnounceActivate {
                card: dog,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(dog),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        pay(&mut g);
        let old = g.object_ref(enemy);
        g.move_object(enemy, Zone::Exile);
        g.move_object(enemy, Zone::Battlefield);
        assert_ne!(old, g.object_ref(enemy));
        let saved = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        for replay in 0..2 {
            if replay == 1 {
                g.restore(&saved).unwrap();
            }
            settle(&mut g);
            assert_eq!(g.counter_count(enemy, CounterKind::PlusOne), 0);
        }
    }
}
