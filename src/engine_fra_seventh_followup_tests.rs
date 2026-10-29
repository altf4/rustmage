use super::*;

#[test]
fn tam_resistance_empowers_existing_jace_and_both_counter_effects_are_doubled() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        g.create_tokens(p, Card::test_index(5435), 1, false);
        let jace = fra_jaces(&g, p)[0];
        g.add_counter(jace, CounterKind::Loyalty, 3);
        put(&mut g, p, "Doubling Season", Zone::Battlefield);
        let target = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, p, "Tam's Resistance", Zone::Hand);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 1];
        act(&mut g, Action::AnnounceCast { card: spell });
        let mut selected = false;
        for _ in 0..16 {
            if g.state.announcing.is_none() {
                break;
            }
            let choice = Action::ChooseTarget {
                target: Target::Card(target),
            };
            if !selected && g.legal_actions().contains(&choice) {
                act(&mut g, choice);
                selected = true;
            } else {
                choose(&mut g, |a| {
                    matches!(
                        a,
                        Action::ChooseSpellCost { .. }
                            | Action::FinishTargets
                            | Action::PayCosts { .. }
                    )
                });
            }
        }
        assert!(selected);
        let saved = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        for replay in 0..2 {
            if replay == 1 {
                g.restore(&saved).unwrap();
            }
            settle(&mut g);
            assert_eq!(fra_jaces(&g, p), vec![jace]);
            assert_eq!(g.counter_count(jace, CounterKind::Loyalty), 11);
            assert_eq!(g.counter_count(target, CounterKind::PlusOne), 2);
            assert!(g.keywords(target).contains(Keywords::VIGILANCE));
        }
    }
}

#[test]
fn diamondblade_grants_working_deathtouch_after_teyo_leaves() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let bearer = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let victim = put(&mut g, p.other(), "Gigantosaurus", Zone::Battlefield);
        let teyo = put(&mut g, p, "Teyo, Diamondblade Mage", Zone::Hand);
        cast(&mut g, teyo, None);
        let mut chosen = false;
        for _ in 0..48 {
            g.prepare_triggers();
            let actions = g.legal_actions();
            let choice = Action::ChooseTarget {
                target: Target::Card(bearer),
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
        g.move_object(teyo, Zone::Graveyard);
        let saved = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        for replay in 0..2 {
            if replay == 1 {
                g.restore(&saved).unwrap();
            }
            settle(&mut g);
            assert_eq!(g.counter_count(bearer, CounterKind::PlusOne), 1);
            assert!(g.keywords(bearer).contains(Keywords::DEATHTOUCH));
            g.apply_damage_batch(&[DamageEvent {
                source: g.damage_source(bearer),
                target: Target::Card(victim),
                amount: 1,
                combat: false,
                preventable: true,
            }]);
            g.state_based_actions();
            assert_eq!(g.object_view(victim).zone, Zone::Graveyard);
        }
    }
}

#[test]
fn graft_surgeon_death_replacement_and_target_control_are_rechecked() {
    for p in [Player::P0, Player::P1] {
        for replacement in [false, true] {
            let mut g = scenario(p);
            let target = put(&mut g, p, "Hill Giant", Zone::Battlefield);
            let surgeon = put(&mut g, p, "Graft Surgeon", Zone::Hand);
            cast(&mut g, surgeon, None);
            settle(&mut g);
            assert_eq!(g.counter_count(surgeon, CounterKind::PlusOne), 1);
            if replacement {
                put(&mut g, p, "Rest in Peace", Zone::Battlefield);
            }
            g.move_object(surgeon, Zone::Graveyard);
            if replacement {
                g.prepare_triggers();
                settle(&mut g);
                assert_eq!(g.object_view(surgeon).zone, Zone::Exile);
                assert_eq!(g.counter_count(target, CounterKind::PlusOne), 0);
            } else {
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
                let object = g.object_ref(target);
                g.change_controller(object, p.other());
                let saved = g.snapshot();
                super::super::contracts::branch_contract(g.clone());
                for replay in 0..2 {
                    if replay == 1 {
                        g.restore(&saved).unwrap();
                    }
                    settle(&mut g);
                    assert_eq!(g.counter_count(target, CounterKind::PlusOne), 0);
                }
            }
        }
    }
}
