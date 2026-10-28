use super::*;

#[test]
fn konstrari_charm_modes_targets_and_mana() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..3 {
            let mut g = scenario(p);
            let flying = put(&mut g, p.other(), "Ajani's Pridemate", Zone::Battlefield);
            let ground = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            g.state.objects[flying as usize].gained_keywords = Keywords::FLYING;
            let charm = put(&mut g, p, "Konstrari Charm", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 0, 1, 1, 0];
            act(&mut g, Action::AnnounceCast { card: charm });
            act(&mut g, Action::ChooseMode { mode });
            if mode == 0 {
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(ground),
                    },
                );
                act(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(flying),
                    },
                );
            } else if mode == 1 {
                act(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(ground),
                    },
                );
            }
            super::contracts::branch_contract(g.clone());
            pay(&mut g);
            settle(&mut g);
            match mode {
                0 => assert_eq!(g.object_view(flying).zone, Zone::Graveyard),
                1 => {
                    assert_eq!(g.counter_count(ground, CounterKind::PlusOne), 2);
                    assert!(g.keywords(ground).contains(Keywords::TRAMPLE));
                    g.finish_cleanup();
                    assert!(!g.keywords(ground).contains(Keywords::TRAMPLE));
                    assert_eq!(g.counter_count(ground, CounterKind::PlusOne), 2);
                }
                _ => assert_eq!(g.state.players[p.index()].mana[5], 3),
            }
        }
    }
}

#[test]
fn charge_the_sanctum_hybrid_modes_and_cleanup() {
    for p in [Player::P0, Player::P1] {
        for (mode, color) in [(0, 0), (1, 3)] {
            let mut g = scenario(p);
            let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let other = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let charm = put(&mut g, p, "Charge the Sanctum", Zone::Hand);
            g.state.players[p.index()].mana = [0; 6];
            g.state.players[p.index()].mana[color] = 1;
            g.state.players[p.index()].mana[5] = 2;
            act(&mut g, Action::AnnounceCast { card: charm });
            act(&mut g, Action::ChooseMode { mode });
            if mode == 1 {
                act(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(other),
                    },
                );
            }
            pay(&mut g);
            settle(&mut g);
            if mode == 0 {
                assert_eq!(stats(&g, own), (4, 2));
                assert_eq!(stats(&g, other), (2, 2));
            } else {
                assert_eq!(stats(&g, other), (5, 3));
                assert!(g.keywords(other).contains(Keywords::FIRST_STRIKE));
            }
            g.finish_cleanup();
            assert_eq!(stats(&g, own), (2, 2));
            assert!(!g.keywords(other).contains(Keywords::FIRST_STRIKE));
            assert_eq!(stats(&g, other), if mode == 0 { (2, 2) } else { (3, 3) });
        }
    }
}

#[test]
fn konstrari_flying_target_loses_ability_before_resolution() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let bird = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.state.objects[bird as usize].gained_keywords = Keywords::FLYING;
        let charm = put(&mut g, p, "Konstrari Charm", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: charm });
        act(&mut g, Action::ChooseMode { mode: 0 });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bird),
            },
        );
        pay(&mut g);
        g.state.objects[bird as usize].gained_keywords = Keywords::NONE;
        settle(&mut g);
        assert_eq!(g.object_view(bird).zone, Zone::Battlefield);
        assert_eq!(g.state.objects[bird as usize].damage, 0);
    }
}

#[test]
fn kiora_dragons_entry_activation_and_replacement() {
    for p in [Player::P0, Player::P1] {
        for doubled in [false, true] {
            let mut g = scenario(p);
            if doubled {
                put(&mut g, p, "Doubling Season", Zone::Battlefield);
            }
            let kiora = put(&mut g, p, "Kiora of Fire and Ashes", Zone::Hand);
            cast(&mut g, kiora, None);
            settle(&mut g);
            let dragons = |g: &Game| -> Vec<CardId> {
                g.state
                    .objects
                    .iter()
                    .enumerate()
                    .filter(|(_, o)| {
                        o.zone == Zone::Battlefield
                            && o.controller == p
                            && o.is_token()
                            && o.card == Card(5288)
                    })
                    .map(|(id, _)| id as CardId)
                    .collect()
            };
            let amount = if doubled { 2 } else { 1 };
            assert_eq!(dragons(&g).len(), amount);
            for dragon in dragons(&g) {
                assert_eq!(stats(&g, dragon), (5, 5));
                assert!(g.has_subtype(dragon, "Dragon"));
                assert!(g.keywords(dragon).contains(Keywords::FLYING));
                assert_eq!(g.object_view(dragon).card.colors(), 8);
            }
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 7];
            reject(
                &mut g,
                Action::AnnounceActivate {
                    card: kiora,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
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
            super::contracts::branch_contract(g.clone());
            g.move_object(kiora, Zone::Exile);
            settle(&mut g);
            assert_eq!(dragons(&g).len(), amount * 2);
        }
    }
}

#[test]
fn twisted_fates_independent_targets_partial_legality_and_counter_replacement() {
    for p in [Player::P0, Player::P1] {
        for invalid in 0..3 {
            let mut g = scenario(p);
            let victim = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let survivor = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            put(&mut g, p.other(), "Doubling Season", Zone::Battlefield);
            let land = put(&mut g, p.other(), "Forest", Zone::Battlefield);
            let spell = put(&mut g, p, "Twisted Fates", Zone::Hand);
            act(&mut g, Action::AnnounceCast { card: spell });
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(land),
                },
            );
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(victim),
                },
            );
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Player(p.other()),
                },
            );
            super::contracts::branch_contract(g.clone());
            pay(&mut g);
            if invalid == 1 {
                g.move_object(victim, Zone::Exile);
            }
            if invalid == 2 {
                g.state.players[p.other().index()].hexproof_until_end_turn = true;
            }
            settle(&mut g);
            assert_eq!(
                g.object_view(victim).zone,
                if invalid == 1 {
                    Zone::Exile
                } else {
                    Zone::Graveyard
                }
            );
            assert_eq!(
                g.counter_count(survivor, CounterKind::PlusOne),
                if invalid == 2 { 0 } else { 2 }
            );
        }
    }
}

#[test]
fn eardrum_rattler_target_limits_tap_and_power_recheck() {
    for p in [Player::P0, Player::P1] {
        for fizzle in [false, true] {
            let mut g = scenario(p);
            let rattler = put(&mut g, p, "Eardrum Rattler", Zone::Battlefield);
            let friend = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let big = put(&mut g, p, "Fateshaper Aspirant", Zone::Battlefield);
            act(
                &mut g,
                Action::AnnounceActivate {
                    card: rattler,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(rattler),
                },
            );
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(enemy),
                },
            );
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(big),
                },
            );
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(friend),
                },
            );
            super::contracts::branch_contract(g.clone());
            pay(&mut g);
            assert!(g.object_view(rattler).tapped);
            if fizzle {
                g.state.objects[friend as usize].growth.power += 1;
            }
            settle(&mut g);
            assert_eq!(
                g.keywords(friend).contains(Keywords::CANT_BE_BLOCKED),
                !fizzle
            );
            if !fizzle {
                g.state.objects[friend as usize].growth.power += 1;
                assert!(g.keywords(friend).contains(Keywords::CANT_BE_BLOCKED));
            }
            g.finish_cleanup();
            assert!(!g.keywords(friend).contains(Keywords::CANT_BE_BLOCKED));
        }
    }
}

#[test]
fn master_of_barbs_observes_any_noncombat_damage_once_per_batch() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let master = put(&mut g, p, "Master of Barbs", Zone::Battlefield);
        let friend = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let opponent_source = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let event = |g: &Game, target: Player, combat: bool| DamageEvent {
            preventable: true,
            source: g.damage_source(opponent_source),
            target: Target::Player(target),
            amount: 1,
            combat,
        };
        g.apply_damage_batch(&[event(&g, p, false), event(&g, p.other(), true)]);
        assert!(g.state.pending_triggers.is_empty());
        g.apply_damage_batch(&[event(&g, p.other(), false), event(&g, p.other(), false)]);
        assert_eq!(g.state.pending_triggers.len(), 1);
        super::contracts::branch_contract(g.clone());
        g.move_object(master, Zone::Exile);
        for _ in 0..48 {
            if g.state.pending == Pending::Priority
                && g.state.stack.is_empty()
                && g.state.pending_triggers.is_empty()
                && g.state.resolving.is_none()
            {
                break;
            }
            choose(&mut g, |a| {
                matches!(a, Action::Pass | Action::PutTrigger { .. })
            });
        }
        assert_eq!(stats(&g, friend), (3, 2));
        assert_eq!(stats(&g, opponent_source), (2, 2));
        g.finish_cleanup();
        assert_eq!(stats(&g, friend), (2, 2));
    }
}

#[test]
fn master_of_barbs_paid_damage_of_either_player() {
    for p in [Player::P0, Player::P1] {
        for target in [p, p.other()] {
            let mut g = scenario(p);
            let master = put(&mut g, p, "Master of Barbs", Zone::Hand);
            cast(&mut g, master, None);
            settle(&mut g);
            let creature = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let shock = put(&mut g, p.other(), "Shock", Zone::Hand);
            g.state.priority = p.other();
            cast(&mut g, shock, Some(Target::Player(target)));
            for _ in 0..48 {
                if g.state.pending == Pending::Priority
                    && g.state.stack.is_empty()
                    && g.state.pending_triggers.is_empty()
                    && g.state.resolving.is_none()
                {
                    break;
                }
                choose(&mut g, |a| {
                    matches!(a, Action::Pass | Action::PutTrigger { .. })
                });
            }
            assert_eq!(
                stats(&g, creature),
                if target == p.other() { (3, 2) } else { (2, 2) }
            );
        }
    }
}

#[test]
fn return_to_light_realms_simultaneous_entries_and_aura_choice() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let host = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let angel = put(&mut g, p, "Dazzling Angel", Zone::Graveyard);
        let bear = put(&mut g, p, "Bear Cub", Zone::Graveyard);
        let artifact = put(&mut g, p, "Mox Jasper", Zone::Graveyard);
        let aura = put(&mut g, p, "Pacifism", Zone::Graveyard);
        let land = put(&mut g, p, "Forest", Zone::Graveyard);
        let instant = put(&mut g, p, "Lightning Strike", Zone::Graveyard);
        let foreign = put(&mut g, p.other(), "Bear Cub", Zone::Graveyard);
        let spell = put(&mut g, p, "Return to the Light Realms", Zone::Hand);
        let old = g.object_ref(bear);
        g.state.players[p.index()].mana = [2, 0, 0, 0, 0, 7];
        cast(&mut g, spell, None);
        for _ in 0..64 {
            if matches!(g.state.pending, Pending::AuraEntry { .. }) {
                super::contracts::branch_contract(g.clone());
                let object = g.object_ref(host);
                act(&mut g, Action::ChooseAuraAttachment { object });
            } else if g.state.pending == Pending::Priority
                && g.state.stack.is_empty()
                && g.state.pending_triggers.is_empty()
                && g.state.resolving.is_none()
            {
                break;
            } else {
                choose(&mut g, |a| {
                    matches!(a, Action::Pass | Action::PutTrigger { .. })
                });
            }
        }
        for id in [angel, bear, artifact, aura] {
            assert_eq!(g.object_view(id).zone, Zone::Battlefield);
        }
        for id in [land, instant, foreign] {
            assert_eq!(g.object_view(id).zone, Zone::Graveyard);
        }
        assert_ne!(g.object_ref(bear), old);
        assert_eq!(g.object_view(aura).attached_to, Some(g.object_ref(host)));
        assert_eq!(g.state.players[p.index()].life, 21);
    }
}

#[test]
fn fixed_empower_entry_and_graveyard_activation_reuse_jace() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let eye = put(&mut g, p, "Mindseeker Oculus", Zone::Hand);
        cast(&mut g, eye, None);
        settle(&mut g);
        let jaces: Vec<_> = g
            .state
            .objects
            .iter()
            .enumerate()
            .filter_map(|(i, o)| {
                (o.zone == Zone::Battlefield
                    && o.controller == p
                    && o.is_token()
                    && o.card == Card(5435))
                .then_some(i as CardId)
            })
            .collect();
        assert_eq!(jaces.len(), 1);
        let jace = jaces[0];
        assert_eq!(g.counter_count(jace, CounterKind::Loyalty), 4);
        let crier = put(&mut g, p, "Campus Crier", Zone::Graveyard);
        g.state.active = p.other();
        g.state.priority = p.other();
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: crier,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        g.state.priority = p;
        g.state.players[p.index()].mana = [0; 6];
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: crier,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        assert_eq!(g.object_view(crier).zone, Zone::Graveyard);
        g.state.players[p.index()].mana[5] = 1;
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
        super::contracts::branch_contract(g.clone());
        settle(&mut g);
        assert_eq!(g.counter_count(jace, CounterKind::Loyalty), 6);
    }
}
