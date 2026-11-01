use super::*;

#[test]
fn mabel_friendly_simultaneous_entries_trigger_independently_after_source_leaves() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let mabel = put(&mut g, p, "Mabel, Valley Hero", Zone::Battlefield);
        put(&mut g, p, "Doubling Season", Zone::Battlefield);
        let a = put(&mut g, p, "Bear Cub", Zone::Hand);
        let b = put(&mut g, p, "Hill Giant", Zone::Hand);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Hand);
        let observers = g.death_trigger_sources();
        let entries = g.move_objects_raw(&[a, b, enemy], Zone::Battlefield);
        g.finish_zone_events(&entries, &observers);
        g.move_object(mabel, Zone::Graveyard);
        let mut chosen = 0;
        for _ in 0..64 {
            g.prepare_triggers();
            let actions = g.legal_actions();
            let target = if chosen == 0 { a } else { b };
            let choice = Action::ChooseTarget {
                target: Target::Card(target),
            };
            if chosen < 2 && actions.contains(&choice) {
                act(&mut g, choice);
                chosen += 1;
            } else if chosen == 2
                && g.state.pending == Pending::Priority
                && g.state.stack.is_empty()
                && g.state.pending_triggers.is_empty()
                && g.state.resolving.is_none()
            {
                break;
            } else if let Some(action) = actions
                .into_iter()
                .find(|a| matches!(a, Action::Pass | Action::PutTrigger { .. }))
            {
                act(&mut g, action);
            } else {
                panic!("unexpected Mabel entry choice {:?}", g.state.pending);
            }
        }
        assert_eq!(chosen, 2);
        assert_eq!(g.counter_count(a, CounterKind::PlusOne), 2);
        assert_eq!(g.counter_count(b, CounterKind::PlusOne), 2);
        assert_eq!(g.counter_count(enemy, CounterKind::PlusOne), 0);
    }
}

#[test]
fn repurposed_enforcer_uses_existing_jace_and_original_controller_after_control_change() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        g.create_tokens(p, Card::test_index(5435), 1, false);
        let jace = fra_jaces(&g, p)[0];
        g.add_counter(jace, CounterKind::Loyalty, 2);
        put(&mut g, p, "Doubling Season", Zone::Battlefield);
        let source = put(&mut g, p, "Repurposed Enforcer", Zone::Battlefield);
        put(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: source });
        act(&mut g, Action::FinishAttackers);
        let original = g.object_ref(source);
        g.change_controller(original, p.other());
        let saved = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        for replay in 0..2 {
            if replay == 1 {
                g.restore(&saved).unwrap();
            }
            settle(&mut g);
            assert_eq!(fra_jaces(&g, p), vec![jace]);
            assert_eq!(g.counter_count(jace, CounterKind::Loyalty), 4);
            assert!(fra_jaces(&g, p.other()).is_empty());
        }
    }
}

#[test]
fn dueling_arena_produces_colorless_and_rejects_unfunded_or_tapped_activation() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let arena = put(&mut g, p, "Hexhaven Dueling Arena", Zone::Battlefield);
        let target = put(&mut g, p, "Emeritus of Conflict", Zone::Battlefield);
        let action = Action::AnnounceActivate {
            card: arena,
            ability: 2,
            counter_payment: Counters::default(),
        };
        g.state.players[p.index()].mana = [0; 6];
        let mana = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::ActivateMana { card, ability: 0, .. } if *card == arena))
            .expect("colorless mana ability");
        act(&mut g, mana);
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 0, 0, 1]);
        assert!(g.object_view(arena).tapped);
        g.state.players[p.index()].mana[5] = 4;
        reject(&mut g, action.clone());
        g.untap_objects(&[arena], untap_rules::UntapCause::Effect);
        g.state.players[p.index()].mana[5] = 3;
        reject(&mut g, action.clone());
        assert!(!g.object_view(arena).tapped);
        g.state.players[p.index()].mana[5] = 4;
        act(&mut g, action);
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        pay(&mut g);
        let old = g.object_ref(target);
        g.move_object(target, Zone::Exile);
        g.move_object(target, Zone::Battlefield);
        assert_ne!(g.object_ref(target), old);
        super::super::contracts::branch_contract(g.clone());
        settle(&mut g);
        assert!(!g.is_prepared(g.object_ref(target)));
    }
}

#[test]
fn kindred_judgment_allows_an_absent_type_and_destroys_both_players_creatures() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Llanowar Elves", Zone::Battlefield);
        let spell = put(&mut g, p, "Kindred Judgment", Zone::Hand);
        cast(&mut g, spell, None);
        act(&mut g, Action::Pass);
        act(&mut g, Action::Pass);
        assert!(matches!(g.state.pending, Pending::CreatureTypeChoice { player } if player == p));
        let saved = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        for replay in 0..2 {
            if replay == 1 {
                g.restore(&saved).unwrap();
            }
            act(
                &mut g,
                Action::ChooseCreatureType {
                    creature_type: crate::cards::CreatureType::from_name("Dragon").unwrap(),
                },
            );
            settle(&mut g);
            assert_eq!(g.object_view(own).zone, Zone::Graveyard);
            assert_eq!(g.object_view(enemy).zone, Zone::Graveyard);
        }
    }
}

#[test]
fn mabel_token_entry_targets_friendly_token_or_fizzles_against_new_hexproof() {
    for p in [Player::P0, Player::P1] {
        for hexproof in [false, true] {
            let mut g = scenario(p);
            let mabel = put(&mut g, p, "Mabel, Valley Hero", Zone::Battlefield);
            let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Hand);
            g.move_object(enemy, Zone::Battlefield);
            g.create_tokens(p, Card::test_index(5298), 1, false);
            let token = g
                .state
                .objects
                .iter()
                .enumerate()
                .find_map(|(i, o)| {
                    (o.zone == Zone::Battlefield
                        && o.is_token()
                        && o.controller == p
                        && o.card == Card::test_index(5298))
                    .then_some(i as CardId)
                })
                .unwrap();
            let target = if hexproof { enemy } else { token };
            let mut chosen = false;
            for _ in 0..48 {
                g.prepare_triggers();
                let actions = g.legal_actions();
                let choice = Action::ChooseTarget {
                    target: Target::Card(target),
                };
                if actions.contains(&choice) {
                    act(&mut g, choice);
                    chosen = true;
                    break;
                }
                choose(&mut g, |a| {
                    matches!(a, Action::Pass | Action::PutTrigger { .. })
                });
            }
            assert!(chosen);
            if hexproof {
                g.state.objects[enemy as usize].gained_keywords = Keywords::HEXPROOF;
            }
            g.move_object(mabel, Zone::Graveyard);
            super::super::contracts::branch_contract(g.clone());
            settle(&mut g);
            assert_eq!(
                g.counter_count(target, CounterKind::PlusOne),
                u64::from(!hexproof)
            );
        }
    }
}

#[test]
fn mabel_target_reentry_invalidates_queued_counter() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Mabel, Valley Hero", Zone::Battlefield);
        let target = put(&mut g, p, "Hill Giant", Zone::Hand);
        g.move_object(target, Zone::Battlefield);
        let mut chosen = false;
        for _ in 0..48 {
            g.prepare_triggers();
            let choice = Action::ChooseTarget {
                target: Target::Card(target),
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
        let original = g.object_ref(target);
        g.move_object(target, Zone::Exile);
        g.move_object(target, Zone::Battlefield);
        assert_ne!(g.object_ref(target), original);
        // The reentry queues a second trigger before the source leaves.
        let source = g
            .state
            .objects
            .iter()
            .enumerate()
            .find_map(|(i, o)| {
                (o.zone == Zone::Battlefield
                    && o.card == Card::from_name("Mabel, Valley Hero").unwrap())
                .then_some(i as CardId)
            })
            .unwrap();
        g.move_object(source, Zone::Graveyard);
        // The reentry's own trigger is mandatory and can target the new object;
        // resolve it separately and ensure only one counter is added.
        let mut second = false;
        for _ in 0..64 {
            g.prepare_triggers();
            let actions = g.legal_actions();
            let choice = Action::ChooseTarget {
                target: Target::Card(target),
            };
            if !second && actions.contains(&choice) {
                act(&mut g, choice);
                second = true;
            } else if second
                && g.state.pending == Pending::Priority
                && g.state.stack.is_empty()
                && g.state.pending_triggers.is_empty()
                && g.state.resolving.is_none()
            {
                break;
            } else if let Some(a) = actions
                .into_iter()
                .find(|a| matches!(a, Action::Pass | Action::PutTrigger { .. }))
            {
                act(&mut g, a);
            } else {
                panic!("unexpected Mabel reentry {:?}", g.state.pending);
            }
        }
        assert!(second);
        assert_eq!(g.counter_count(target, CounterKind::PlusOne), 1);
    }
}

#[test]
fn repurposed_enforcer_doubled_jaces_allow_choice_before_legend_rule() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Doubling Season", Zone::Battlefield);
        let source = put(&mut g, p, "Repurposed Enforcer", Zone::Battlefield);
        put(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: source });
        act(&mut g, Action::FinishAttackers);
        for _ in 0..48 {
            if g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::ChooseJace { .. }))
            {
                break;
            }
            choose(&mut g, |a| {
                matches!(a, Action::Pass | Action::PutTrigger { .. })
            });
        }
        let choices: Vec<_> = g
            .legal_actions()
            .into_iter()
            .filter_map(|a| match a {
                Action::ChooseJace { object } => Some(object),
                _ => None,
            })
            .collect();
        assert_eq!(choices.len(), 2);
        let saved = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        for replay in 0..2 {
            if replay == 1 {
                g.restore(&saved).unwrap();
            }
            act(&mut g, Action::ChooseJace { object: choices[1] });
            settle(&mut g);
            assert_eq!(g.counter_count(choices[1].card, CounterKind::Loyalty), 4);
            assert_eq!(g.object_view(choices[1].card).zone, Zone::Battlefield);
            assert_ne!(g.object_view(choices[0].card).zone, Zone::Battlefield);
        }
    }
}

#[test]
fn dueling_arena_attacker_activation_fizzles_for_a_new_incarnation() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let arena = put(&mut g, p, "Hexhaven Dueling Arena", Zone::Battlefield);
        let fighter = put(&mut g, p, "Emeritus of Conflict", Zone::Battlefield);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: fighter });
        act(&mut g, Action::FinishAttackers);
        g.state.step = Step::Main2;
        g.state.priority = p;
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        act(
            &mut g,
            Action::AnnounceActivate {
                card: arena,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(fighter),
            },
        );
        pay(&mut g);
        let old = g.object_ref(fighter);
        g.move_object(fighter, Zone::Exile);
        g.move_object(fighter, Zone::Battlefield);
        assert_ne!(old, g.object_ref(fighter));
        super::super::contracts::branch_contract(g.clone());
        settle(&mut g);
        assert!(!g.is_prepared(g.object_ref(fighter)));
    }
}

#[test]
fn kindred_judgment_preserves_changeling_and_counts_simultaneous_graveyard_batch_once() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let observer = put(&mut g, p, "Robot Domination", Zone::Battlefield);
        let a = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let b = put(&mut g, p, "Hill Giant", Zone::Battlefield);
        let changeling = put(&mut g, p.other(), "Graveshifter", Zone::Battlefield);
        assert!(g.has_subtype(changeling, "Dragon"));
        let spell = put(&mut g, p, "Kindred Judgment", Zone::Hand);
        cast(&mut g, spell, None);
        act(&mut g, Action::Pass);
        act(&mut g, Action::Pass);
        assert!(matches!(g.state.pending, Pending::CreatureTypeChoice { player } if player == p));
        super::super::contracts::branch_contract(g.clone());
        act(
            &mut g,
            Action::ChooseCreatureType {
                creature_type: crate::cards::CreatureType::from_name("Dragon").unwrap(),
            },
        );
        settle(&mut g);
        assert_eq!(g.object_view(a).zone, Zone::Graveyard);
        assert_eq!(g.object_view(b).zone, Zone::Graveyard);
        assert_eq!(g.object_view(changeling).zone, Zone::Battlefield);
        assert_eq!(g.counter_count(observer, CounterKind::Plan), 1);
    }
}
