use super::*;

#[test]
fn fixed_empower_entry_uses_existing_jace_and_survives_source_departure() {
    for p in [Player::P0, Player::P1] {
        for (name, amount) in [("Keeper of the Quiet Hour", 2u64), ("Mindseeker Oculus", 4)] {
            let mut g = scenario(p);
            g.create_tokens(p, Card::test_index(5435), 1, false);
            let jace = fra_jaces(&g, p)[0];
            g.add_counter(jace, CounterKind::Loyalty, 2);
            put(&mut g, p, "Doubling Season", Zone::Battlefield);
            let source = put(&mut g, p, name, Zone::Hand);
            cast(&mut g, source, None);
            for _ in 0..32 {
                if g.object_view(source).zone == Zone::Battlefield {
                    break;
                }
                choose(&mut g, |a| {
                    matches!(a, Action::Pass | Action::PutTrigger { .. })
                });
            }
            assert_eq!(g.object_view(source).zone, Zone::Battlefield);
            g.move_object(source, Zone::Graveyard);
            let saved = g.snapshot();
            super::super::contracts::branch_contract(g.clone());
            for replay in 0..2 {
                if replay == 1 {
                    g.restore(&saved).unwrap();
                }
                settle(&mut g);
                assert_eq!(fra_jaces(&g, p), vec![jace]);
                assert_eq!(g.counter_count(jace, CounterKind::Loyalty), 2 + 2 * amount);
                assert_eq!(g.object_view(source).zone, Zone::Graveyard);
            }
        }
    }
}

#[test]
fn campus_crier_opponent_turn_exile_cost_and_reentry_do_not_affect_stack_ability() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        g.create_tokens(p, Card::test_index(5435), 1, false);
        let jace = fra_jaces(&g, p)[0];
        g.add_counter(jace, CounterKind::Loyalty, 2);
        let crier = put(&mut g, p, "Campus Crier", Zone::Graveyard);
        g.state.active = p.other();
        g.state.priority = p;
        g.state.step = Step::BeginCombat;
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        let original = g.object_ref(crier);
        act(
            &mut g,
            Action::AnnounceActivate {
                card: crier,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        assert_eq!(g.object_view(crier).zone, Zone::Exile);
        g.move_object(crier, Zone::Graveyard);
        assert_ne!(g.object_ref(crier), original);
        let saved = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        for replay in 0..2 {
            if replay == 1 {
                g.restore(&saved).unwrap();
            }
            settle(&mut g);
            assert_eq!(g.object_view(crier).zone, Zone::Graveyard);
            assert_eq!(g.counter_count(jace, CounterKind::Loyalty), 4);
        }
    }
}

#[test]
fn protege_draw_occurs_after_choosing_between_replacement_created_jaces() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Doubling Season", Zone::Battlefield);
        let spell = put(&mut g, p, "Protege's Awakening", Zone::Hand);
        let library = g.state.players[p.index()].library.len();
        cast(&mut g, spell, None);
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
        assert_eq!(g.state.players[p.index()].library.len(), library);
        let saved = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        for replay in 0..2 {
            if replay == 1 {
                g.restore(&saved).unwrap();
            }
            act(&mut g, Action::ChooseJace { object: choices[1] });
            settle(&mut g);
            assert_eq!(g.counter_count(choices[1].card, CounterKind::Loyalty), 12);
            assert_eq!(g.object_view(choices[1].card).zone, Zone::Battlefield);
            assert_ne!(g.object_view(choices[0].card).zone, Zone::Battlefield);
            assert_eq!(g.state.players[p.index()].library.len(), library - 1);
        }
    }
}
