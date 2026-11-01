use super::*;

#[test]
fn awaken_basic_landcycling_pays_discard_and_allows_fail_to_find() {
    for p in [Player::P0, Player::P1] {
        for find in [false, true] {
            let mut g = scenario(p);
            let spell = put(&mut g, p, "Awaken the Inferno", Zone::Hand);
            let land = put(&mut g, p, "Forest", Zone::Library);
            let invalid = put(&mut g, p, "Bear Cub", Zone::Library);
            let action = Action::AnnounceActivate {
                card: spell,
                ability: 0,
                counter_payment: Counters::default(),
            };
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
            reject(&mut g, action.clone());
            assert_eq!(g.object_view(spell).zone, Zone::Hand);
            g.state.players[p.index()].mana[5] = 2;
            act(&mut g, action);
            pay(&mut g);
            assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
            let mut searched = false;
            for _ in 0..64 {
                let actions = g.legal_actions();
                if !searched && actions.contains(&Action::ChooseSearch { card: land }) {
                    reject(&mut g, Action::ChooseSearch { card: invalid });
                    assert!(g.observe(p.other()).search_choice.is_none());
                    super::super::contracts::branch_contract(g.clone());
                    if find {
                        act(&mut g, Action::ChooseSearch { card: land });
                    }
                    act(&mut g, Action::FinishSearch);
                    searched = true;
                } else if searched
                    && g.state.pending == Pending::Priority
                    && g.state.stack.is_empty()
                    && g.state.resolving.is_none()
                    && g.state.pending_triggers.is_empty()
                {
                    break;
                } else if let Some(a) = actions
                    .into_iter()
                    .find(|a| matches!(a, Action::Pass | Action::PutTrigger { .. }))
                {
                    act(&mut g, a);
                } else {
                    panic!("unexpected landcycling choice {:?}", g.state.pending);
                }
            }
            assert!(searched);
            assert_eq!(
                g.object_view(land).zone,
                if find { Zone::Hand } else { Zone::Library }
            );
            assert_eq!(g.object_view(invalid).zone, Zone::Library);
        }
    }
}

#[test]
fn kiora_activation_at_opponents_turn_keeps_original_controller_after_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let kiora = put(&mut g, p, "Kiora of Fire and Ashes", Zone::Battlefield);
        g.state.active = p.other();
        g.state.priority = p;
        g.state.step = Step::BeginCombat;
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 8];
        act(
            &mut g,
            Action::AnnounceActivate {
                card: kiora,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        g.move_object(kiora, Zone::Exile);
        let saved = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        for replay in 0..2 {
            if replay == 1 {
                g.restore(&saved).unwrap();
            }
            settle(&mut g);
            let dragons: Vec<_> = g
                .state
                .objects
                .iter()
                .enumerate()
                .filter_map(|(i, o)| {
                    (o.zone == Zone::Battlefield
                        && o.is_token()
                        && o.card == Card::test_index(5288))
                    .then_some(i as CardId)
                })
                .collect();
            assert_eq!(dragons.len(), 1);
            assert_eq!(g.object_view(dragons[0]).owner, p);
            assert_eq!(g.object_view(dragons[0]).controller, p);
            assert_eq!(g.object_view(dragons[0]).colors, 8);
            assert_eq!(stats(&g, dragons[0]), (5, 5));
            assert!(g.keywords(dragons[0]).contains(Keywords::FLYING));
        }
    }
}

#[test]
fn massacre_girl_planeswalker_death_prevention_and_target_fizzle() {
    for p in [Player::P0, Player::P1] {
        for fizzle in [false, true] {
            let mut g = scenario(p);
            let girl = put(&mut g, p, "Massacre Girl, Most Wanted", Zone::Battlefield);
            let walker = put(&mut g, p, "Liliana, Dreadhorde General", Zone::Battlefield);
            g.add_counter(walker, CounterKind::Loyalty, 4);
            let prevention = put(&mut g, p.other(), "Forest", Zone::Battlefield);
            let source = g.object_ref(prevention);
            let recipient = g.object_ref(girl);
            let timestamp = g.next_role_timestamp();
            g.state
                .roles
                .get_or_insert_default()
                .source_linked_effects
                .push(SourceLinkedEffectView {
                    source,
                    recipient,
                    controller: p.other(),
                    kind: SourceLinkedEffectKind::PreventDamageBy,
                    timestamp,
                });
            let life = g.state.players[p.index()].life;
            let enemy_life = g.state.players[p.other().index()].life;
            g.move_object(walker, Zone::Graveyard);
            let mut selected = false;
            for _ in 0..48 {
                g.prepare_triggers();
                let choice = Action::ChooseTarget {
                    target: Target::Player(p.other()),
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
            if fizzle {
                g.state.players[p.other().index()].hexproof_until_end_turn = true;
            }
            let saved = g.snapshot();
            super::super::contracts::branch_contract(g.clone());
            for replay in 0..2 {
                if replay == 1 {
                    g.restore(&saved).unwrap();
                }
                settle(&mut g);
                assert_eq!(g.state.players[p.index()].life, life + i128::from(!fizzle));
                assert_eq!(g.state.players[p.other().index()].life, enemy_life);
                assert_eq!(g.counter_count(girl, CounterKind::PlusOne), 0);
            }
        }
    }
}

#[test]
fn overwrite_no_creatures_empowers_existing_jace_by_zero() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        g.create_tokens(p, Card::test_index(5435), 1, false);
        let jace = fra_jaces(&g, p)[0];
        g.add_counter(jace, CounterKind::Loyalty, 3);
        let spell = put(&mut g, p, "Overwrite the Multiverse", Zone::Hand);
        cast(&mut g, spell, None);
        let saved = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        for replay in 0..2 {
            if replay == 1 {
                g.restore(&saved).unwrap();
            }
            settle(&mut g);
            assert_eq!(fra_jaces(&g, p), vec![jace]);
            assert_eq!(g.counter_count(jace, CounterKind::Loyalty), 3);
        }
    }
}
