use super::*;

#[test]
fn vraska_treasure_goes_to_target_controller_and_target_control_is_rechecked() {
    for p in [Player::P0, Player::P1] {
        for invalid in [false, true] {
            let mut g = scenario(p);
            for _ in 0..6 {
                put(&mut g, p, "Forest", Zone::Battlefield);
            }
            put(&mut g, p.other(), "Doubling Season", Zone::Battlefield);
            let target = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let reference = g.object_ref(target);
            g.change_controller(reference, p.other());
            let source = put(&mut g, p, "Vraska, the Cutting Glare", Zone::Hand);
            cast(&mut g, source, None);
            let mut selected = false;
            for _ in 0..64 {
                g.prepare_triggers();
                let actions = g.legal_actions();
                let choice = Action::ChooseTarget {
                    target: Target::Card(target),
                };
                if actions.contains(&choice) {
                    act(&mut g, choice);
                    selected = true;
                    break;
                } else if let Some(action) = actions
                    .into_iter()
                    .find(|a| matches!(a, Action::Pass | Action::PutTrigger { .. }))
                {
                    act(&mut g, action);
                } else {
                    panic!("unexpected Vraska target choice {:?}", g.state.pending);
                }
            }
            assert!(selected);
            if invalid {
                g.change_controller(reference, p);
            }
            let saved = g.snapshot();
            super::super::contracts::branch_contract(g.clone());
            settle(&mut g);
            assert_eq!(
                g.object_view(target).zone,
                if invalid {
                    Zone::Battlefield
                } else {
                    Zone::Graveyard
                }
            );
            let treasures: Vec<_> = g
                .state
                .objects
                .iter()
                .filter(|o| {
                    o.zone == Zone::Battlefield && o.is_token() && o.card == Card::test_index(5302)
                })
                .collect();
            assert_eq!(treasures.len(), if invalid { 0 } else { 2 });
            assert!(treasures
                .iter()
                .all(|o| o.controller == p.other() && o.owner == p.other()));
            g.restore(&saved).unwrap();
            settle(&mut g);
            assert_eq!(
                g.object_view(target).zone,
                if invalid {
                    Zone::Battlefield
                } else {
                    Zone::Graveyard
                }
            );
        }
    }
}

#[test]
fn arni_ignores_own_entry_and_opponent_entry_and_does_not_boost_reentered_source() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let arni = put(&mut g, p, "Arni, Renowned Champion", Zone::Hand);
        cast(&mut g, arni, None);
        settle(&mut g);
        assert_eq!(stats(&g, arni), (1, 5));
        let enemy = put(&mut g, p.other(), "Hill Giant", Zone::Hand);
        g.move_object(enemy, Zone::Battlefield);
        g.prepare_triggers();
        assert!(g.state.pending_triggers.is_empty() && g.state.stack.is_empty());
        let own = put(&mut g, p, "Hill Giant", Zone::Hand);
        g.move_object(own, Zone::Battlefield);
        g.prepare_triggers();
        assert!(!g.state.stack.is_empty() || !g.state.pending_triggers.is_empty());
        let before = g.object_ref(arni);
        g.move_object(arni, Zone::Exile);
        g.move_object(arni, Zone::Battlefield);
        assert_ne!(g.object_ref(arni), before);
        super::super::contracts::branch_contract(g.clone());
        settle(&mut g);
        assert_eq!(stats(&g, arni), (1, 5));
        assert!(g.keywords(arni).contains(Keywords::TRAMPLE));
    }
}

#[test]
fn battalion_cadets_have_printed_characteristics_and_owner() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let spell = put(&mut g, p, "Hexhaven Battalion", Zone::Hand);
        cast(&mut g, spell, None);
        settle(&mut g);
        let cadets: Vec<_> = g
            .state
            .objects
            .iter()
            .enumerate()
            .filter_map(|(i, o)| {
                (o.zone == Zone::Battlefield && o.is_token() && o.card == Card::test_index(5298))
                    .then_some(i as CardId)
            })
            .collect();
        assert_eq!(cadets.len(), 3);
        for id in cadets {
            let view = g.object_view(id);
            assert_eq!(view.owner, p);
            assert_eq!(view.controller, p);
            assert_eq!(view.colors, 0);
            assert_eq!(g.object_name(id), "Cadet");
            assert!(view.card.is_creature());
            assert!(g.has_subtype(id, "Wizard"));
            assert!(g.has_subtype(id, "Soldier"));
            assert_eq!(stats(&g, id), (2, 2));
        }
    }
}

#[test]
fn battalion_cycling_rejects_insufficient_mana_and_search_may_fail_to_find() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Rest in Peace", Zone::Battlefield);
        let spell = put(&mut g, p, "Hexhaven Battalion", Zone::Hand);
        let activation = Action::AnnounceActivate {
            card: spell,
            ability: 0,
            counter_payment: Counters::default(),
        };
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        reject(&mut g, activation.clone());
        assert_eq!(g.object_view(spell).zone, Zone::Hand);
        g.state.players[p.index()].mana[5] = 2;
        act(&mut g, activation);
        pay(&mut g);
        assert_eq!(g.object_view(spell).zone, Zone::Exile);
        let mut searched = false;
        for _ in 0..64 {
            let actions = g.legal_actions();
            if actions.contains(&Action::FinishSearch) {
                assert!(g.observe(p.other()).search_choice.is_none());
                act(&mut g, Action::FinishSearch);
                searched = true;
            } else if searched
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
                panic!("unexpected landcycling choice {:?}", g.state.pending);
            }
        }
        assert!(searched);
    }
}

#[test]
fn warriors_blades_equip_is_sorcery_speed_and_entry_may_target_player() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let equipment = put(&mut g, p, "Warrior's Blades", Zone::Battlefield);
        put(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
        g.state.step = Step::BeginCombat;
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: equipment,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        let mut g = scenario(p);
        let equipment = put(&mut g, p, "Warrior's Blades", Zone::Hand);
        let own_life = g.state.players[p.index()].life;
        let enemy_life = g.state.players[p.other().index()].life;
        cast(&mut g, equipment, None);
        let mut targeted = false;
        for _ in 0..48 {
            g.prepare_triggers();
            let actions = g.legal_actions();
            let choice = Action::ChooseTarget {
                target: Target::Player(p.other()),
            };
            if actions.contains(&choice) {
                act(&mut g, choice);
                targeted = true;
                break;
            } else if let Some(action) = actions
                .into_iter()
                .find(|a| matches!(a, Action::Pass | Action::PutTrigger { .. }))
            {
                act(&mut g, action);
            } else {
                panic!("unexpected Blades target choice {:?}", g.state.pending);
            }
        }
        assert!(targeted);
        super::super::contracts::branch_contract(g.clone());
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, own_life + 3);
        assert_eq!(g.state.players[p.other().index()].life, enemy_life - 3);
    }
}
