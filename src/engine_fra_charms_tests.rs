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

fn fra_jaces(g: &Game, p: Player) -> Vec<CardId> {
    g.state
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
        .collect()
}

#[test]
fn empower_consumers_entry_and_draw() {
    for p in [Player::P0, Player::P1] {
        for (name, loyalty, pt) in [
            ("Arcane Amphisbaena", 2, (1, 1)),
            ("Keeper of the Quiet Hour", 2, (3, 2)),
        ] {
            let mut g = scenario(p);
            let creature = put(&mut g, p, name, Zone::Hand);
            cast(&mut g, creature, None);
            settle(&mut g);
            assert_eq!(stats(&g, creature), pt);
            if name == "Arcane Amphisbaena" {
                assert!(g.keywords(creature).contains(Keywords::DEATHTOUCH));
            }
            let jaces = fra_jaces(&g, p);
            assert_eq!(jaces.len(), 1);
            assert_eq!(g.counter_count(jaces[0], CounterKind::Loyalty), loyalty);
        }
        let mut g = scenario(p);
        let before = g.state.players[p.index()].library.len();
        let spell = put(&mut g, p, "Protege's Awakening", Zone::Hand);
        cast(&mut g, spell, None);
        settle(&mut g);
        let jaces = fra_jaces(&g, p);
        assert_eq!(jaces.len(), 1);
        assert_eq!(g.counter_count(jaces[0], CounterKind::Loyalty), 6);
        assert_eq!(g.state.players[p.index()].library.len(), before - 1);
    }
}

#[test]
fn academic_ascent_target_fizzle_and_temporary_effects() {
    for p in [Player::P0, Player::P1] {
        for fizzle in [false, true] {
            let mut g = scenario(p);
            let target = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let spell = put(&mut g, p, "Academic Ascent", Zone::Hand);
            cast(&mut g, spell, Some(Target::Card(target)));
            super::contracts::branch_contract(g.clone());
            if fizzle {
                g.move_object(target, Zone::Exile);
            }
            settle(&mut g);
            if fizzle {
                assert!(fra_jaces(&g, p).is_empty());
            } else {
                assert_eq!(stats(&g, target), (4, 4));
                assert!(g.keywords(target).contains(Keywords::FLYING));
                let jaces = fra_jaces(&g, p);
                assert_eq!(jaces.len(), 1);
                assert_eq!(g.counter_count(jaces[0], CounterKind::Loyalty), 2);
                g.finish_cleanup();
                assert_eq!(stats(&g, target), (2, 2));
                assert!(!g.keywords(target).contains(Keywords::FLYING));
            }
        }
    }
}

#[test]
fn tams_resistance_optional_target_hybrid_and_fizzle() {
    for p in [Player::P0, Player::P1] {
        for (targeted, fizzle, color) in [(false, false, 1), (true, false, 4), (true, true, 1)] {
            let mut g = scenario(p);
            let target = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let spell = put(&mut g, p, "Tam's Resistance", Zone::Hand);
            g.state.players[p.index()].mana = [0; 6];
            g.state.players[p.index()].mana[color] = 1;
            g.state.players[p.index()].mana[5] = 1;
            act(&mut g, Action::AnnounceCast { card: spell });
            let mut selected = false;
            for _ in 0..12 {
                if g.state.announcing.is_none() {
                    break;
                }
                let wanted = Action::ChooseTarget {
                    target: Target::Card(target),
                };
                if targeted && !selected && g.legal_actions().contains(&wanted) {
                    act(&mut g, wanted);
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
            assert!(g.state.announcing.is_none());
            assert_eq!(selected, targeted);
            super::contracts::branch_contract(g.clone());
            if fizzle {
                g.move_object(target, Zone::Exile);
            }
            settle(&mut g);
            if fizzle {
                assert!(fra_jaces(&g, p).is_empty());
            } else {
                let jaces = fra_jaces(&g, p);
                assert_eq!(jaces.len(), 1);
                assert_eq!(g.counter_count(jaces[0], CounterKind::Loyalty), 4);
                assert_eq!(
                    g.counter_count(target, CounterKind::PlusOne),
                    u64::from(targeted)
                );
                assert_eq!(g.keywords(target).contains(Keywords::VIGILANCE), targeted);
                g.finish_cleanup();
                assert!(!g.keywords(target).contains(Keywords::VIGILANCE));
                assert_eq!(
                    g.counter_count(target, CounterKind::PlusOne),
                    u64::from(targeted)
                );
            }
        }
    }
}

#[test]
fn tetsuko_uses_live_power_toughness_control_and_ability_origin() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let tetsuko = put(&mut g, p, "Tetsuko Umezawa, Fugitive", Zone::Battlefield);
        let low_power = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let low_toughness = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let ordinary = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let blocker = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.state.objects[low_power as usize].growth.power = -1;
        g.state.objects[low_toughness as usize].growth.toughness = -1;
        assert!(!g.can_block_pair(blocker, tetsuko));
        assert!(!g.can_block_pair(blocker, low_power));
        assert!(!g.can_block_pair(blocker, low_toughness));
        assert!(g.can_block_pair(blocker, ordinary));
        super::contracts::branch_contract(g.clone());
        g.state.objects[low_power as usize].growth.power = 0;
        assert!(g.can_block_pair(blocker, low_power));
        let aura = put(&mut g, p.other(), "Witness Protection", Zone::Battlefield);
        g.state.objects[aura as usize].attached_to = Some(g.object_ref(tetsuko));
        assert!(!g.printed_abilities_available(tetsuko));
        assert!(g.can_block_pair(blocker, low_toughness));
        g.move_object(aura, Zone::Graveyard);
        assert!(!g.can_block_pair(blocker, low_toughness));
        let reference = g.object_ref(tetsuko);
        g.change_controller(reference, p.other());
        assert!(g.can_block_pair(blocker, low_toughness));
    }
}

#[test]
fn way_of_the_mentor_empowers_and_rewards_each_controller_life_gain() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let own = put(&mut g, p, "Liliana the Faultless", Zone::Battlefield);
        let opposing = put(
            &mut g,
            p.other(),
            "Liliana the Faultless",
            Zone::Battlefield,
        );
        g.add_counter(own, CounterKind::Loyalty, 4);
        g.add_counter(opposing, CounterKind::Loyalty, 4);
        let way = put(&mut g, p, "Way of the Mentor", Zone::Hand);
        cast(&mut g, way, None);
        settle(&mut g);
        let jace = fra_jaces(&g, p)[0];
        assert_eq!(g.counter_count(jace, CounterKind::Loyalty), 5);
        g.gain_life(p.other(), 2);
        g.gain_life(p, 0);
        settle(&mut g);
        assert_eq!(g.counter_count(jace, CounterKind::Loyalty), 5);
        g.gain_life(p, 3);
        g.gain_life(p, 1);
        super::contracts::branch_contract(g.clone());
        g.move_object(way, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.counter_count(jace, CounterKind::Loyalty), 7);
        assert_eq!(g.counter_count(own, CounterKind::Loyalty), 6);
        assert_eq!(g.counter_count(opposing, CounterKind::Loyalty), 4);
    }
}

#[test]
fn way_of_the_mentor_counters_follow_replacement_effects() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Doubling Season", Zone::Battlefield);
        let way = put(&mut g, p, "Way of the Mentor", Zone::Hand);
        cast(&mut g, way, None);
        settle(&mut g);
        let jace = fra_jaces(&g, p)[0];
        assert_eq!(g.counter_count(jace, CounterKind::Loyalty), 10);
        g.gain_life(p, 1);
        settle(&mut g);
        assert_eq!(g.counter_count(jace, CounterKind::Loyalty), 12);
    }
}

#[test]
fn traxos_untap_restriction_and_cast_trigger() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let traxos = put(&mut g, p, "Traxos, Scourge Eternal", Zone::Battlefield);
        assert!(g.keywords(traxos).contains(Keywords::TRAMPLE));
        g.state.objects[traxos as usize].tapped = true;
        g.untap_active_player();
        assert!(g.state.objects[traxos as usize].tapped);
        let spell = put(&mut g, p, "Shock", Zone::Hand);
        cast(&mut g, spell, Some(Target::Player(p.other())));
        settle(&mut g);
        assert!(g.state.objects[traxos as usize].tapped);
        let creature = put(&mut g, p, "Bear Cub", Zone::Hand);
        cast(&mut g, creature, None);
        super::contracts::branch_contract(g.clone());
        settle(&mut g);
        assert!(!g.state.objects[traxos as usize].tapped);
        let aura = put(&mut g, p.other(), "Witness Protection", Zone::Battlefield);
        g.state.objects[aura as usize].attached_to = Some(g.object_ref(traxos));
        g.state.objects[traxos as usize].tapped = true;
        assert!(!g.printed_abilities_available(traxos));
        g.untap_active_player();
        assert!(!g.state.objects[traxos as usize].tapped);
    }
}

#[test]
fn traxos_observes_artifact_casts_only_by_its_controller() {
    for p in [Player::P0, Player::P1] {
        for caster in [p, p.other()] {
            let mut g = scenario(caster);
            let traxos = put(&mut g, p, "Traxos, Scourge Eternal", Zone::Battlefield);
            g.state.objects[traxos as usize].tapped = true;
            let artifact = put(&mut g, caster, "Sting, Bilbo's Sword", Zone::Hand);
            cast(&mut g, artifact, None);
            settle(&mut g);
            assert_eq!(g.state.objects[traxos as usize].tapped, caster != p);
        }
    }
}

#[test]
fn dark_matter_manipulator_mill_and_graveyard_groups_are_live() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let creature = put(&mut g, p, "Dark Matter Manipulator", Zone::Hand);
        let before = g.state.players[p.index()].library.len();
        cast(&mut g, creature, None);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].library.len(), before - 3);
        assert_eq!(stats(&g, creature), (1, 2));
        for _ in 0..3 {
            put(&mut g, p, "Forest", Zone::Graveyard);
        }
        for _ in 0..7 {
            put(&mut g, p.other(), "Forest", Zone::Graveyard);
        }
        assert_eq!(stats(&g, creature), (1, 2));
        let seventh = put(&mut g, p, "Forest", Zone::Graveyard);
        assert_eq!(stats(&g, creature), (3, 2));
        super::contracts::branch_contract(g.clone());
        for _ in 0..7 {
            put(&mut g, p, "Forest", Zone::Graveyard);
        }
        assert_eq!(stats(&g, creature), (5, 2));
        let reference = g.object_ref(creature);
        g.change_controller(reference, p.other());
        assert_eq!(stats(&g, creature), (3, 2));
        g.change_controller(reference, p);
        assert_eq!(stats(&g, creature), (5, 2));
        g.move_object(seventh, Zone::Exile);
        assert_eq!(stats(&g, creature), (3, 2));
        let aura = put(&mut g, p.other(), "Witness Protection", Zone::Battlefield);
        g.state.objects[aura as usize].attached_to = Some(g.object_ref(creature));
        assert!(!g.printed_abilities_available(creature));
        assert_eq!(stats(&g, creature), (1, 1));
    }
}

fn fra_thopters(g: &Game, p: Player) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(i, o)| {
            (o.zone == Zone::Battlefield
                && o.controller == p
                && o.is_token()
                && o.card == Card(5266))
            .then_some(i as CardId)
        })
        .collect()
}

#[test]
fn saheeli_cast_trigger_and_thopter_haste_follow_controller_and_source() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Saheeli, Jewel of Avishkar", Zone::Battlefield);
        let creature = put(&mut g, p, "Bear Cub", Zone::Hand);
        cast(&mut g, creature, None);
        settle(&mut g);
        assert!(fra_thopters(&g, p).is_empty());
        let shock = put(&mut g, p, "Shock", Zone::Hand);
        cast(&mut g, shock, Some(Target::Player(p.other())));
        super::contracts::branch_contract(g.clone());
        settle(&mut g);
        let thopters = fra_thopters(&g, p);
        assert_eq!(thopters.len(), 1);
        let thopter = thopters[0];
        assert_eq!(stats(&g, thopter), (1, 1));
        assert!(g.keywords(thopter).contains(Keywords::FLYING));
        assert!(g.keywords(thopter).contains(Keywords::HASTE));
        let reference = g.object_ref(thopter);
        g.change_controller(reference, p.other());
        assert!(!g.keywords(thopter).contains(Keywords::HASTE));
        g.change_controller(reference, p);
        assert!(g.keywords(thopter).contains(Keywords::HASTE));
        let aura = put(&mut g, p.other(), "Witness Protection", Zone::Battlefield);
        g.state.objects[aura as usize].attached_to = Some(g.object_ref(source));
        assert!(!g.printed_abilities_available(source));
        assert!(!g.keywords(thopter).contains(Keywords::HASTE));
        g.move_object(aura, Zone::Graveyard);
        assert!(g.keywords(thopter).contains(Keywords::HASTE));
        g.move_object(source, Zone::Graveyard);
        assert!(!g.keywords(thopter).contains(Keywords::HASTE));
    }
}

#[test]
fn saheeli_ignores_opponent_cast_and_trigger_survives_source_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p.other());
        put(&mut g, p, "Saheeli, Jewel of Avishkar", Zone::Battlefield);
        let shock = put(&mut g, p.other(), "Shock", Zone::Hand);
        cast(&mut g, shock, Some(Target::Player(p)));
        settle(&mut g);
        assert!(fra_thopters(&g, p).is_empty());

        let mut g = scenario(p);
        let source = put(&mut g, p, "Saheeli, Jewel of Avishkar", Zone::Battlefield);
        let shock = put(&mut g, p, "Shock", Zone::Hand);
        cast(&mut g, shock, Some(Target::Player(p.other())));
        g.move_object(source, Zone::Graveyard);
        settle(&mut g);
        let thopters = fra_thopters(&g, p);
        assert_eq!(thopters.len(), 1);
        assert!(!g.keywords(thopters[0]).contains(Keywords::HASTE));
    }
}

#[test]
fn fatehold_charm_draw_empower_and_team_modes() {
    for p in [Player::P0, Player::P1] {
        for mode in [0, 2] {
            let mut g = scenario(p);
            let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let opposing = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let before = g.state.players[p.index()].library.len();
            let charm = put(&mut g, p, "Fatehold Charm", Zone::Hand);
            act(&mut g, Action::AnnounceCast { card: charm });
            act(&mut g, Action::ChooseMode { mode });
            pay(&mut g);
            super::contracts::branch_contract(g.clone());
            settle(&mut g);
            if mode == 0 {
                assert_eq!(g.state.players[p.index()].library.len(), before - 1);
                let jaces = fra_jaces(&g, p);
                assert_eq!(jaces.len(), 1);
                assert_eq!(g.counter_count(jaces[0], CounterKind::Loyalty), 2);
            } else {
                assert_eq!(stats(&g, own), (3, 4));
                assert_eq!(stats(&g, opposing), (2, 2));
                g.finish_cleanup();
                assert_eq!(stats(&g, own), (2, 2));
            }
        }
    }
}

#[test]
fn fatehold_charm_returns_creature_spell_and_spell_copy() {
    for p in [Player::P0, Player::P1] {
        for target_kind in 0..3 {
            let mut g = scenario(p.other());
            let target = put(
                &mut g,
                p.other(),
                "Bear Cub",
                if target_kind == 0 {
                    Zone::Battlefield
                } else {
                    Zone::Hand
                },
            );
            let charm = put(&mut g, p, "Fatehold Charm", Zone::Hand);
            let mut copy_id = None;
            if target_kind != 0 {
                cast(&mut g, target, None);
                if target_kind == 2 {
                    let template = g
                        .state
                        .stack
                        .iter()
                        .find(|s| s.card == target)
                        .unwrap()
                        .clone();
                    let copy = g.allocate_spell_copy(&template, p.other());
                    copy_id = Some(copy.id);
                    g.state.stack.push(copy);
                }
            }
            g.state.priority = p;
            act(&mut g, Action::AnnounceCast { card: charm });
            act(&mut g, Action::ChooseMode { mode: 1 });
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Player(p),
                },
            );
            let selected = match target_kind {
                0 => Target::Card(target),
                1 => Target::Spell(target),
                _ => Target::Stack(copy_id.unwrap()),
            };
            act(&mut g, Action::ChooseTarget { target: selected });
            pay(&mut g);
            super::contracts::branch_contract(g.clone());
            settle(&mut g);
            if let Some(copy_id) = copy_id {
                assert!(!g.state.stack.iter().any(|s| s.id == copy_id));
                assert_eq!(g.object_view(target).zone, Zone::Battlefield);
            } else {
                assert_eq!(g.object_view(target).zone, Zone::Hand);
            }
        }
    }
}

#[test]
fn fatehold_charm_fizzles_if_its_only_creature_target_leaves() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let creature = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let charm = put(&mut g, p, "Fatehold Charm", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: charm });
        act(&mut g, Action::ChooseMode { mode: 1 });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(creature),
            },
        );
        pay(&mut g);
        g.move_object(creature, Zone::Exile);
        settle(&mut g);
        assert_eq!(g.object_view(creature).zone, Zone::Exile);
        assert_eq!(g.object_view(charm).zone, Zone::Graveyard);
    }
}

#[test]
fn wrath_of_the_bloodmane_reduction_and_creature_or_planeswalker_damage() {
    for p in [Player::P0, Player::P1] {
        for (discounted, planeswalker) in [(false, false), (true, false), (true, true)] {
            let mut g = scenario(p);
            put(
                &mut g,
                p.other(),
                "Haliya, Guided by Light",
                Zone::Battlefield,
            );
            let target = if planeswalker {
                let id = put(
                    &mut g,
                    p.other(),
                    "Liliana the Faultless",
                    Zone::Battlefield,
                );
                g.add_counter(id, CounterKind::Loyalty, 5);
                id
            } else {
                put(&mut g, p.other(), "Bear Cub", Zone::Battlefield)
            };
            let spell = put(&mut g, p, "Wrath of the Bloodmane", Zone::Hand);
            g.state.players[p.index()].mana = [0; 6];
            g.state.players[p.index()].mana[3] = 1;
            g.state.players[p.index()].mana[5] = 1;
            if discounted {
                put(&mut g, p, "Haliya, Guided by Light", Zone::Battlefield);
            } else {
                reject(&mut g, Action::AnnounceCast { card: spell });
                g.state.players[p.index()].mana[5] = 2;
            }
            act(&mut g, Action::AnnounceCast { card: spell });
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Player(p.other()),
                },
            );
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
            pay(&mut g);
            super::contracts::branch_contract(g.clone());
            settle(&mut g);
            if planeswalker {
                assert_eq!(g.counter_count(target, CounterKind::Loyalty), 1);
            } else {
                assert_eq!(g.object_view(target).zone, Zone::Graveyard);
            }
        }
    }
}

#[test]
fn craterclaw_freezes_current_artifact_count_and_recipients() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let opposing = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let artifact = put(&mut g, p, "Mox Jasper", Zone::Battlefield);
        put(&mut g, p.other(), "Mox Jasper", Zone::Battlefield);
        let source = put(&mut g, p, "Craterclaw Colossus", Zone::Hand);
        cast(&mut g, source, None);
        super::contracts::branch_contract(g.clone());
        settle(&mut g);
        assert_eq!(stats(&g, source), (7, 5));
        assert!(g.keywords(source).contains(Keywords::HASTE));
        assert!(g.keywords(source).contains(Keywords::TRAMPLE));
        assert_eq!(stats(&g, own), (4, 2));
        assert!(g.keywords(own).contains(Keywords::TRAMPLE));
        assert_eq!(stats(&g, opposing), (2, 2));
        assert!(!g.keywords(opposing).contains(Keywords::TRAMPLE));
        g.move_object(artifact, Zone::Graveyard);
        let late = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        assert_eq!(stats(&g, own), (4, 2));
        assert_eq!(stats(&g, late), (2, 2));
        assert!(!g.keywords(late).contains(Keywords::TRAMPLE));
        let reference = g.object_ref(own);
        g.change_controller(reference, p.other());
        assert_eq!(stats(&g, own), (4, 2));
        assert!(g.keywords(own).contains(Keywords::TRAMPLE));
        g.finish_cleanup();
        assert_eq!(stats(&g, own), (2, 2));
        assert!(!g.keywords(own).contains(Keywords::TRAMPLE));
    }
}

#[test]
fn craterclaw_counts_artifacts_when_entry_trigger_resolves() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        put(&mut g, p, "Mox Jasper", Zone::Battlefield);
        let source = put(&mut g, p, "Craterclaw Colossus", Zone::Hand);
        cast(&mut g, source, None);
        for _ in 0..16 {
            if g.object_view(source).zone == Zone::Battlefield {
                break;
            }
            choose(&mut g, |a| {
                matches!(a, Action::Pass | Action::PutTrigger { .. })
            });
        }
        assert_eq!(g.object_view(source).zone, Zone::Battlefield);
        g.move_object(source, Zone::Graveyard);
        super::contracts::branch_contract(g.clone());
        settle(&mut g);
        assert_eq!(stats(&g, bear), (3, 2));
        assert!(g.keywords(bear).contains(Keywords::TRAMPLE));
    }
}

#[test]
fn pia_creates_thopter_and_uses_resolution_time_artifact_count() {
    for p in [Player::P0, Player::P1] {
        for fizzle in [false, true] {
            let mut g = scenario(p);
            let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let pia = put(&mut g, p, "Pia, Determined Rebuilder", Zone::Hand);
            cast(&mut g, pia, None);
            settle(&mut g);
            let thopters = fra_thopters(&g, p);
            assert_eq!(thopters.len(), 1);
            assert_eq!(stats(&g, thopters[0]), (1, 1));
            assert!(g.keywords(thopters[0]).contains(Keywords::FLYING));
            let extra = put(&mut g, p, "Mox Jasper", Zone::Battlefield);
            put(&mut g, p.other(), "Mox Jasper", Zone::Battlefield);
            act(
                &mut g,
                Action::AnnounceActivate {
                    card: pia,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Player(p),
                },
            );
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
            pay(&mut g);
            super::contracts::branch_contract(g.clone());
            g.move_object(pia, Zone::Graveyard);
            g.move_object(extra, Zone::Graveyard);
            if fizzle {
                g.move_object(target, Zone::Exile);
            }
            settle(&mut g);
            if fizzle {
                assert_eq!(g.object_view(target).zone, Zone::Exile);
            } else {
                assert_eq!(stats(&g, target), (3, 2));
                g.finish_cleanup();
                assert_eq!(stats(&g, target), (2, 2));
            }
        }
    }
}

#[test]
fn flickering_hound_optional_target_and_owner_control() {
    for p in [Player::P0, Player::P1] {
        for targeted in [false, true] {
            let mut g = scenario(p);
            let hound = put(&mut g, p, "Flickering Hound", Zone::Battlefield);
            let creature = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let stolen = g.object_ref(creature);
            g.change_controller(stolen, p);
            let opponent = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let spell = put(&mut g, p, "Bear Cub", Zone::Hand);
            cast(&mut g, spell, None);
            let mut selected = false;
            let mut rejected = false;
            for _ in 0..64 {
                g.prepare_triggers();
                let actions = g.legal_actions();
                let target = Action::ChooseTarget {
                    target: Target::Card(creature),
                };
                if actions.contains(&target) && !selected {
                    if !rejected {
                        reject(
                            &mut g,
                            Action::ChooseTarget {
                                target: Target::Card(hound),
                            },
                        );
                        reject(
                            &mut g,
                            Action::ChooseTarget {
                                target: Target::Card(opponent),
                            },
                        );
                        rejected = true;
                    }
                    if targeted {
                        act(&mut g, target);
                        selected = true;
                    } else {
                        act(&mut g, Action::FinishTargets);
                        selected = true;
                    }
                    super::contracts::branch_contract(g.clone());
                } else if actions.contains(&Action::FinishTargets) {
                    act(&mut g, Action::FinishTargets);
                } else if actions.contains(&Action::Pass) {
                    if g.state.stack.is_empty()
                        && g.state.pending_triggers.is_empty()
                        && g.state.resolving.is_none()
                    {
                        break;
                    }
                    act(&mut g, Action::Pass);
                } else if let Some(action) = actions
                    .into_iter()
                    .find(|a| matches!(a, Action::PutTrigger { .. }))
                {
                    act(&mut g, action);
                } else {
                    panic!("unexpected Flickering Hound choice {:?}", g.state.pending);
                }
            }
            assert!(selected);
            assert_eq!(g.object_view(creature).zone, Zone::Battlefield);
            assert_eq!(g.object_ref(creature) != stolen, targeted);
            assert_eq!(
                g.object_view(creature).controller,
                if targeted { p.other() } else { p }
            );
            assert_eq!(g.object_view(spell).zone, Zone::Battlefield);
        }
    }
}

#[test]
fn echoverse_fulcrum_entry_draw_discard_and_private_choice() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let spare = put(&mut g, p, "Bear Cub", Zone::Hand);
        let foreign = put(&mut g, p.other(), "Bear Cub", Zone::Hand);
        let source = put(&mut g, p, "The Echoverse Fulcrum", Zone::Hand);
        let library = g.state.players[p.index()].library.len();
        cast(&mut g, source, None);
        let mut discarded = false;
        for _ in 0..48 {
            let action = Action::Discard { card: spare };
            if g.legal_actions().contains(&action) {
                assert_eq!(g.actor(), Some(p));
                reject(&mut g, Action::Discard { card: foreign });
                super::contracts::branch_contract(g.clone());
                act(&mut g, action);
                discarded = true;
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
        assert!(discarded);
        assert_eq!(g.state.players[p.index()].library.len(), library - 1);
        assert_eq!(g.object_view(spare).zone, Zone::Graveyard);
        assert_eq!(g.object_view(foreign).zone, Zone::Hand);
    }
}

#[test]
fn echoverse_fulcrum_sorcery_activation_exiles_cost_and_destroys_simultaneously() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "The Echoverse Fulcrum", Zone::Battlefield);
        let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let opposing = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let indestructible = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.state.objects[indestructible as usize].gained_keywords = Keywords::INDESTRUCTIBLE;
        let activate = Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        };
        g.state.step = Step::BeginCombat;
        reject(&mut g, activate.clone());
        g.state.step = Step::Main1;
        g.state.players[p.index()].mana = [0; 6];
        g.state.players[p.index()].mana[5] = 4;
        reject(&mut g, activate.clone());
        g.state.players[p.index()].mana[5] = 5;
        act(&mut g, activate);
        pay(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Exile);
        super::contracts::branch_contract(g.clone());
        settle(&mut g);
        assert_eq!(g.object_view(own).zone, Zone::Graveyard);
        assert_eq!(g.object_view(opposing).zone, Zone::Graveyard);
        assert_eq!(g.object_view(indestructible).zone, Zone::Battlefield);
    }
}

#[test]
fn darklight_phoenix_counts_both_players_deaths_on_its_owners_turn() {
    for p in [Player::P0, Player::P1] {
        for (deaths, active_own) in [(0, true), (1, true), (2, true), (2, false)] {
            let mut g = scenario(if active_own { p } else { p.other() });
            let phoenix = put(&mut g, p, "Darklight Phoenix", Zone::Graveyard);
            for index in 0..deaths {
                let owner = if index == 0 { p } else { p.other() };
                let creature = put(&mut g, owner, "Bear Cub", Zone::Battlefield);
                g.move_object(creature, Zone::Graveyard);
            }
            g.begin_step(Step::BeginCombat);
            g.prepare_triggers();
            super::contracts::branch_contract(g.clone());
            settle(&mut g);
            if deaths == 2 && active_own {
                assert_eq!(g.object_view(phoenix).zone, Zone::Battlefield);
                assert!(g.keywords(phoenix).contains(Keywords::FLYING));
                assert!(g.keywords(phoenix).contains(Keywords::HASTE));
            } else {
                assert_eq!(g.object_view(phoenix).zone, Zone::Graveyard);
            }
        }
    }
}

#[test]
fn darklight_phoenix_uses_exact_graveyard_incarnation_and_intervening_guard() {
    for p in [Player::P0, Player::P1] {
        for reenter in [false, true] {
            let mut g = scenario(p);
            let phoenix = put(&mut g, p, "Darklight Phoenix", Zone::Graveyard);
            for owner in [p, p.other()] {
                let creature = put(&mut g, owner, "Bear Cub", Zone::Battlefield);
                g.move_object(creature, Zone::Graveyard);
            }
            g.begin_step(Step::BeginCombat);
            g.prepare_triggers();
            assert!(!g.state.stack.is_empty());
            if reenter {
                g.move_object(phoenix, Zone::Exile);
                g.move_object(phoenix, Zone::Graveyard);
            } else {
                g.state.creatures_died_this_turn = 1;
            }
            super::contracts::branch_contract(g.clone());
            settle(&mut g);
            assert_eq!(g.object_view(phoenix).zone, Zone::Graveyard);
        }
    }
}

#[test]
fn darklight_phoenix_does_not_trigger_from_the_battlefield() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let phoenix = put(&mut g, p, "Darklight Phoenix", Zone::Battlefield);
        for owner in [p, p.other()] {
            let creature = put(&mut g, owner, "Bear Cub", Zone::Battlefield);
            g.move_object(creature, Zone::Graveyard);
        }
        g.begin_step(Step::BeginCombat);
        assert!(g.state.pending_triggers.is_empty());
        assert_eq!(g.object_view(phoenix).zone, Zone::Battlefield);
    }
}

#[test]
fn living_library_pays_sacrifice_and_shuffles_into_actual_owners_library() {
    for p in [Player::P0, Player::P1] {
        for (stolen, fizzle, planeswalker) in [
            (false, false, false),
            (true, false, false),
            (false, true, false),
            (false, false, true),
        ] {
            let mut g = scenario(p);
            let source = put(&mut g, p, "Living Library", Zone::Battlefield);
            let owner = if stolen { p } else { p.other() };
            let target = put(
                &mut g,
                owner,
                if planeswalker {
                    "Liliana the Faultless"
                } else {
                    "Bear Cub"
                },
                Zone::Battlefield,
            );
            if planeswalker {
                g.add_counter(target, CounterKind::Loyalty, 5);
            }
            if stolen {
                let reference = g.object_ref(target);
                g.change_controller(reference, p.other());
            }
            let illegal = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let before = g.state.players[owner.index()].library.len();
            g.state.players[p.index()].mana = [0; 6];
            g.state.players[p.index()].mana[5] = 6;
            let activation = Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            };
            act(&mut g, activation);
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(illegal),
                },
            );
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
            pay(&mut g);
            assert_eq!(g.object_view(source).zone, Zone::Graveyard);
            if fizzle {
                let reference = g.object_ref(target);
                g.change_controller(reference, p);
            }
            super::contracts::branch_contract(g.clone());
            settle(&mut g);
            if fizzle {
                assert_eq!(g.object_view(target).zone, Zone::Battlefield);
                assert_eq!(g.state.players[owner.index()].library.len(), before);
            } else {
                assert_eq!(g.object_view(target).zone, Zone::Library);
                assert_eq!(g.state.players[owner.index()].library.len(), before + 1);
                assert!(g.state.players[owner.index()].library.contains(&target));
            }
        }
    }
}

#[test]
fn gardenize_counts_own_deaths_and_uses_latest_departure_counter_quantity() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Gardenize", Zone::Battlefield);
        for owner in [p, p.other(), p] {
            let dead = put(&mut g, owner, "Bear Cub", Zone::Battlefield);
            g.move_object(dead, Zone::Graveyard);
            g.prepare_triggers();
            settle(&mut g);
        }
        assert_eq!(g.counter_count(source, CounterKind::Charge), 2);
        g.state.players[p.index()].mana = [0; 6];
        g.begin_step(Step::Main1);
        g.prepare_triggers();
        assert!(!g.state.stack.is_empty());
        g.add_counter(source, CounterKind::Charge, 1);
        let before = g.object_ref(source);
        g.move_object(source, Zone::Graveyard);
        g.move_object(source, Zone::Battlefield);
        assert_ne!(g.object_ref(source), before);
        g.add_counter(source, CounterKind::Charge, 7);
        super::contracts::branch_contract(g.clone());
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana[4], 3);
        assert_eq!(g.counter_count(source, CounterKind::Charge), 7);
    }
}

#[test]
fn gardenize_does_not_trigger_on_opponents_first_main_phase() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p.other());
        let source = put(&mut g, p, "Gardenize", Zone::Battlefield);
        g.add_counter(source, CounterKind::Charge, 3);
        g.state.players[p.index()].mana = [0; 6];
        g.begin_step(Step::Main1);
        assert!(g.state.pending_triggers.is_empty());
        assert_eq!(g.state.players[p.index()].mana[4], 0);
    }
}

#[test]
fn teyo_flash_entry_grants_permanent_and_kind_appropriate_counters() {
    for (name, keyword) in [
        ("Teyo, Diamondblade Mage", Keywords::DEATHTOUCH),
        ("Teyo, Lightshield Expert", Keywords::HEXPROOF),
    ] {
        for p in [Player::P0, Player::P1] {
            for (kind, doubled, fizzle) in [
                (0, false, false),
                (1, true, false),
                (2, false, false),
                (3, false, false),
                (0, false, true),
            ] {
                let mut g = scenario(p.other());
                let target = put(
                    &mut g,
                    p,
                    match kind {
                        0 => "Bear Cub",
                        1 | 3 => "Liliana the Faultless",
                        _ => "Forest",
                    },
                    Zone::Battlefield,
                );
                if kind == 1 || kind == 3 {
                    g.add_counter(target, CounterKind::Loyalty, 5);
                }
                if kind == 3 {
                    let reference = g.object_ref(target);
                    g.change_object_form(
                        reference,
                        AnimationSpec {
                            artifact: false,
                            creature_types: AnimationCreatureTypes::Unchanged,
                            base: Some((3, 3)),
                            keywords: Keywords::NONE,
                            duration: AnimationDuration::EndOfTurn,
                        },
                        characteristic_layers::AnimationTypeMode::AddCreature,
                    );
                    assert_eq!(g.current_kinds(target) & 65, 65);
                }
                if doubled {
                    put(&mut g, p, "Doubling Season", Zone::Battlefield);
                }
                let illegal = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
                let teyo = put(&mut g, p, name, Zone::Hand);
                g.state.priority = p;
                cast(&mut g, teyo, None);
                let mut chosen = false;
                for _ in 0..48 {
                    g.prepare_triggers();
                    let actions = g.legal_actions();
                    let wanted = Action::ChooseTarget {
                        target: Target::Card(target),
                    };
                    if !chosen && actions.contains(&wanted) {
                        reject(
                            &mut g,
                            Action::ChooseTarget {
                                target: Target::Card(illegal),
                            },
                        );
                        act(&mut g, wanted);
                        chosen = true;
                        if fizzle {
                            let reference = g.object_ref(target);
                            g.change_controller(reference, p.other());
                        }
                        super::contracts::branch_contract(g.clone());
                    } else if actions.contains(&Action::FinishTargets) {
                        act(&mut g, Action::FinishTargets);
                    } else if g.state.pending == Pending::Priority
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
                        panic!("unexpected Teyo choice {:?}", g.state.pending);
                    }
                }
                assert!(chosen);
                assert_eq!(g.keywords(target).contains(keyword), !fizzle);
                assert_eq!(
                    g.counter_count(target, CounterKind::PlusOne),
                    u64::from((kind == 0 || kind == 3) && !fizzle)
                );
                assert_eq!(
                    g.counter_count(target, CounterKind::Loyalty),
                    if kind == 1 {
                        7
                    } else if kind == 3 {
                        6
                    } else {
                        0
                    }
                );
                if keyword == Keywords::HEXPROOF && kind == 0 && !fizzle {
                    g.state.priority = p.other();
                    let shock = put(&mut g, p.other(), "Shock", Zone::Hand);
                    act(&mut g, Action::AnnounceCast { card: shock });
                    reject(
                        &mut g,
                        Action::ChooseTarget {
                            target: Target::Card(target),
                        },
                    );
                    act(
                        &mut g,
                        Action::ChooseTarget {
                            target: Target::Player(p),
                        },
                    );
                    pay(&mut g);
                    settle(&mut g);
                }
                g.finish_cleanup();
                assert!(!g.keywords(target).contains(keyword));
            }
        }
    }
}

#[test]
fn graft_surgeon_optional_death_target_uses_all_frozen_counter_kinds() {
    for p in [Player::P0, Player::P1] {
        for targeted in [false, true] {
            let mut g = scenario(p);
            let target = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let opponent = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let surgeon = put(&mut g, p, "Graft Surgeon", Zone::Hand);
            cast(&mut g, surgeon, None);
            settle(&mut g);
            assert_eq!(g.counter_count(surgeon, CounterKind::PlusOne), 1);
            g.add_counter(surgeon, CounterKind::Stun, 2);
            let old = g.object_ref(surgeon);
            g.move_object(surgeon, Zone::Graveyard);
            g.move_object(surgeon, Zone::Battlefield);
            assert_ne!(g.object_ref(surgeon), old);
            g.add_counter(surgeon, CounterKind::PlusOne, 7);
            let mut selected = false;
            for _ in 0..48 {
                g.prepare_triggers();
                let actions = g.legal_actions();
                let wanted = Action::ChooseTarget {
                    target: Target::Card(target),
                };
                if !selected && actions.contains(&wanted) {
                    reject(
                        &mut g,
                        Action::ChooseTarget {
                            target: Target::Card(opponent),
                        },
                    );
                    if targeted {
                        act(&mut g, wanted);
                    } else {
                        act(&mut g, Action::FinishTargets);
                    }
                    selected = true;
                    super::contracts::branch_contract(g.clone());
                } else if actions.contains(&Action::FinishTargets) {
                    act(&mut g, Action::FinishTargets);
                } else if g.state.pending == Pending::Priority
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
                    panic!("unexpected Graft Surgeon choice {:?}", g.state.pending);
                }
            }
            assert!(selected);
            assert_eq!(
                g.counter_count(target, CounterKind::PlusOne),
                u64::from(targeted)
            );
            assert_eq!(
                g.counter_count(target, CounterKind::Stun),
                if targeted { 2 } else { 0 }
            );
        }
    }
}

#[test]
fn graft_surgeon_death_sees_counter_pairs_before_state_based_removal() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let target = put(&mut g, p, "Hill Giant", Zone::Battlefield);
        let surgeon = put(&mut g, p, "Graft Surgeon", Zone::Hand);
        cast(&mut g, surgeon, None);
        settle(&mut g);
        g.add_counter(surgeon, CounterKind::MinusOne, 3);
        g.state_based_actions();
        assert_eq!(g.object_view(surgeon).zone, Zone::Graveyard);
        let mut selected = false;
        for _ in 0..48 {
            g.prepare_triggers();
            let actions = g.legal_actions();
            let wanted = Action::ChooseTarget {
                target: Target::Card(target),
            };
            if !selected && actions.contains(&wanted) {
                act(&mut g, wanted);
                selected = true;
                super::contracts::branch_contract(g.clone());
            } else if actions.contains(&Action::FinishTargets) {
                act(&mut g, Action::FinishTargets);
            } else if g.state.pending == Pending::Priority
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
                panic!("unexpected Graft Surgeon choice {:?}", g.state.pending);
            }
        }
        assert!(selected);
        assert_eq!(g.object_view(target).zone, Zone::Battlefield);
        assert_eq!(g.counter_count(target, CounterKind::PlusOne), 0);
        assert_eq!(g.counter_count(target, CounterKind::MinusOne), 2);
    }
}

#[test]
fn prophesied_end_uses_attacking_status_and_controller_at_resolution() {
    for p in [Player::P0, Player::P1] {
        for (attacking, indestructible, stolen, fizzle) in [
            (false, false, false, false),
            (true, false, false, false),
            (false, true, false, false),
            (false, false, true, false),
            (false, false, false, true),
        ] {
            let mut g = scenario(p);
            let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            if indestructible {
                g.state.objects[target as usize].gained_keywords = Keywords::INDESTRUCTIBLE;
            }
            if attacking {
                g.state.attackers.push(Attacker {
                    card: target,
                    defender: Some(CombatDefender::Player(p)),
                    blocked: false,
                });
            }
            let spell = put(&mut g, p, "Prophesied End", Zone::Hand);
            cast(&mut g, spell, Some(Target::Card(target)));
            if stolen {
                let reference = g.object_ref(target);
                g.change_controller(reference, p);
            }
            if fizzle {
                g.move_object(target, Zone::Exile);
            }
            let controller = if stolen { p } else { p.other() };
            let before = g.state.players[controller.index()].library.len();
            super::contracts::branch_contract(g.clone());
            settle(&mut g);
            assert_eq!(
                g.state.players[controller.index()].library.len(),
                before - usize::from(!attacking && !fizzle)
            );
            assert_eq!(
                g.object_view(target).zone,
                if fizzle {
                    Zone::Exile
                } else if indestructible {
                    Zone::Battlefield
                } else {
                    Zone::Graveyard
                }
            );
        }
    }
}

#[test]
fn kindred_judgment_chooses_at_resolution_and_destroys_one_batch() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let elf = put(&mut g, p.other(), "Llanowar Elves", Zone::Battlefield);
        let protected = put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        g.state.objects[protected as usize].gained_keywords = Keywords::INDESTRUCTIBLE;
        let shielded = put(&mut g, p.other(), "Llanowar Elves", Zone::Battlefield);
        g.add_counter(shielded, CounterKind::Shield, 1);
        let spell = put(&mut g, p, "Kindred Judgment", Zone::Hand);
        cast(&mut g, spell, None);
        act(&mut g, Action::Pass);
        act(&mut g, Action::Pass);
        assert!(matches!(g.state.pending, Pending::CreatureTypeChoice { player } if player == p));
        assert_eq!(g.object_view(elf).zone, Zone::Battlefield);
        super::contracts::branch_contract(g.clone());
        act(
            &mut g,
            Action::ChooseCreatureType {
                creature_type: crate::cards::CreatureType::from_name("Bear").unwrap(),
            },
        );
        settle(&mut g);
        assert_eq!(g.object_view(bear).zone, Zone::Battlefield);
        assert_eq!(g.object_view(elf).zone, Zone::Graveyard);
        assert_eq!(g.object_view(protected).zone, Zone::Battlefield);
        assert_eq!(g.object_view(shielded).zone, Zone::Battlefield);
        assert_eq!(g.counter_count(shielded, CounterKind::Shield), 0);
    }
}

#[test]
fn ajanis_anguish_keeps_paid_x_for_entry_damage_and_grants_trample() {
    for p in [Player::P0, Player::P1] {
        for x in [0, 2] {
            let mut g = scenario(p);
            let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let opponent = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let spell = put(&mut g, p, "Ajani's Anguish", Zone::Hand);
            act(&mut g, Action::AnnounceCast { card: spell });
            act(&mut g, Action::ChooseX { value: x });
            pay(&mut g);
            let mut targeted = false;
            let before = g.state.players[p.other().index()].life;
            for _ in 0..48 {
                g.prepare_triggers();
                let actions = g.legal_actions();
                let wanted = Action::ChooseTarget {
                    target: Target::Player(p.other()),
                };
                if !targeted && actions.contains(&wanted) {
                    act(&mut g, wanted);
                    targeted = true;
                    assert!(g.keywords(own).contains(Keywords::TRAMPLE));
                    assert!(!g.keywords(opponent).contains(Keywords::TRAMPLE));
                    super::contracts::branch_contract(g.clone());
                } else if actions.contains(&Action::FinishTargets) {
                    act(&mut g, Action::FinishTargets);
                } else if targeted
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
                    panic!("unexpected Ajani's Anguish choice {:?}", g.state.pending);
                }
            }
            assert!(targeted);
            assert_eq!(
                g.state.players[p.other().index()].life,
                before - i128::from(x)
            );
            g.move_object(spell, Zone::Graveyard);
            assert!(!g.keywords(own).contains(Keywords::TRAMPLE));
        }
    }
}

#[test]
fn karn_gilded_guardian_counts_distinct_colors_of_other_controlled_artifacts() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Ancient Cornucopia", Zone::Battlefield);
        put(&mut g, p, "Wildfire Wickerfolk", Zone::Battlefield);
        put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
        put(&mut g, p.other(), "Futurist Forge", Zone::Battlefield);
        let before = g.state.players[p.index()].library.len();
        let karn = put(&mut g, p, "Karn, Gilded Guardian", Zone::Hand);
        cast(&mut g, karn, None);
        super::contracts::branch_contract(g.clone());
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].library.len(), before - 2);
        assert_eq!(stats(&g, karn), (5, 5));
        assert!(g.keywords(karn).contains(Keywords::VIGILANCE));
        assert!(g.keywords(karn).contains(Keywords::TRAMPLE));
    }
}

#[test]
fn budding_insurgent_draws_for_legendary_enchantments_even_if_indestructible() {
    for p in [Player::P0, Player::P1] {
        for (name, legendary_enchantment, indestructible, fizzle) in [
            ("Way of the Mentor", true, false, false),
            ("Way of the Mentor", true, true, false),
            ("Way of the Mentor", true, false, true),
            ("Pacifism", false, false, false),
            ("The Mind Stone", false, false, false),
        ] {
            let mut g = scenario(p);
            let source = put(&mut g, p, "Budding Insurgent", Zone::Battlefield);
            let target = put(&mut g, p.other(), name, Zone::Battlefield);
            let illegal = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            if indestructible {
                g.state.objects[target as usize].gained_keywords = Keywords::INDESTRUCTIBLE;
            }
            let before = g.state.players[p.index()].library.len();
            assert!(g.keywords(source).contains(Keywords::VIGILANCE));
            act(
                &mut g,
                Action::AnnounceActivate {
                    card: source,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(illegal),
                },
            );
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
            if g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::PayCosts { .. }))
            {
                pay(&mut g);
            }
            assert_eq!(g.object_view(source).zone, Zone::Graveyard);
            if fizzle {
                g.move_object(target, Zone::Exile);
            }
            super::contracts::branch_contract(g.clone());
            settle(&mut g);
            assert_eq!(
                g.state.players[p.index()].library.len(),
                before - usize::from(legendary_enchantment && !fizzle)
            );
            assert_eq!(
                g.object_view(target).zone,
                if fizzle {
                    Zone::Exile
                } else if indestructible {
                    Zone::Battlefield
                } else {
                    Zone::Graveyard
                }
            );
        }
    }
}

#[test]
fn mabel_valley_hero_targets_only_current_turns_creature_incarnations() {
    for p in [Player::P0, Player::P1] {
        for stale_at_resolution in [false, true] {
            let mut g = scenario(p);
            let old = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            g.state.entered_this_turn = None;
            let mabel = put(&mut g, p, "Mabel, Valley Hero", Zone::Hand);
            cast(&mut g, mabel, None);
            let mut chosen = false;
            for _ in 0..48 {
                g.prepare_triggers();
                let actions = g.legal_actions();
                let target = Action::ChooseTarget {
                    target: Target::Card(mabel),
                };
                if !chosen && actions.contains(&target) {
                    reject(
                        &mut g,
                        Action::ChooseTarget {
                            target: Target::Card(old),
                        },
                    );
                    act(&mut g, target);
                    chosen = true;
                    if stale_at_resolution {
                        g.state.entered_this_turn = None;
                    }
                    super::contracts::branch_contract(g.clone());
                } else if actions.contains(&Action::FinishTargets) {
                    act(&mut g, Action::FinishTargets);
                } else if chosen
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
                    panic!("unexpected Mabel choice {:?}", g.state.pending);
                }
            }
            assert!(chosen);
            assert_eq!(
                g.counter_count(mabel, CounterKind::PlusOne),
                u64::from(!stale_at_resolution)
            );
        }
    }
}

#[test]
fn stinging_vitriol_deals_damage_then_reveals_and_discards_a_nonland() {
    for p in [Player::P0, Player::P1] {
        for has_nonland in [false, true] {
            let mut g = scenario(p);
            let land = put(&mut g, p.other(), "Forest", Zone::Hand);
            let nonland = has_nonland.then(|| put(&mut g, p.other(), "Shock", Zone::Hand));
            let spell = put(&mut g, p, "Stinging Vitriol", Zone::Hand);
            let life = g.state.players[p.other().index()].life;
            assert!(!g.observe(p).objects.iter().any(|o| o.id == land));
            act(&mut g, Action::AnnounceCast { card: spell });
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Player(p),
                },
            );
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Player(p.other()),
                },
            );
            pay(&mut g);
            for _ in 0..48 {
                if matches!(g.state.pending, Pending::RevealedDiscard { .. })
                    || (g.state.pending == Pending::Priority
                        && g.state.stack.is_empty()
                        && g.state.resolving.is_none())
                {
                    break;
                }
                act(&mut g, Action::Pass);
            }
            assert_eq!(g.state.players[p.other().index()].life, life - 2);
            assert!(g.observe(p).objects.iter().any(|o| o.id == land));
            if let Some(nonland) = nonland {
                assert_eq!(g.actor(), Some(p));
                assert!(g.observe(p).objects.iter().any(|o| o.id == nonland));
                let land_ref = g.object_ref(land);
                reject(&mut g, Action::ChooseRevealedDiscard { object: land_ref });
                super::contracts::branch_contract(g.clone());
                let chosen = g.object_ref(nonland);
                act(&mut g, Action::ChooseRevealedDiscard { object: chosen });
                settle(&mut g);
                assert_eq!(g.object_view(nonland).zone, Zone::Graveyard);
            }
            assert_eq!(g.object_view(land).zone, Zone::Hand);
        }
    }
}

#[test]
fn entrust_the_spark_optional_planeswalker_sacrifice_and_private_search() {
    for p in [Player::P0, Player::P1] {
        for (pay_sacrifice, find_card) in [(false, true), (true, false), (true, true)] {
            let mut g = scenario(p);
            let old = put(&mut g, p, "Liliana the Faultless", Zone::Battlefield);
            g.add_counter(old, CounterKind::Loyalty, 5);
            let illegal = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let other = put(
                &mut g,
                p.other(),
                "Liliana the Faultless",
                Zone::Battlefield,
            );
            g.add_counter(other, CounterKind::Loyalty, 5);
            let found = put(&mut g, p, "Liliana the Faultless", Zone::Library);
            let spell = put(&mut g, p, "Entrust the Spark", Zone::Hand);
            cast(&mut g, spell, None);
            let mut decided = false;
            let mut searched = false;
            for _ in 0..64 {
                g.prepare_triggers();
                let actions = g.legal_actions();
                if !decided && actions.contains(&Action::DeclineEffectPayment) {
                    let wrong = g.object_ref(illegal);
                    let opponents = g.object_ref(other);
                    reject(&mut g, Action::Sacrifice { object: wrong });
                    reject(&mut g, Action::Sacrifice { object: opponents });
                    super::contracts::branch_contract(g.clone());
                    if pay_sacrifice {
                        let choice = g.object_ref(old);
                        act(&mut g, Action::Sacrifice { object: choice });
                    } else {
                        act(&mut g, Action::DeclineEffectPayment);
                    }
                    decided = true;
                } else if actions.contains(&Action::ChooseSearch { card: found }) {
                    assert!(g.observe(p.other()).search_choice.is_none());
                    if find_card {
                        act(&mut g, Action::ChooseSearch { card: found });
                    }
                    act(&mut g, Action::FinishSearch);
                    searched = true;
                } else if decided
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
                    panic!("unexpected Entrust choice {:?}", g.state.pending);
                }
            }
            assert!(decided);
            assert_eq!(searched, pay_sacrifice);
            assert_eq!(
                g.object_view(old).zone,
                if pay_sacrifice {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
            assert_eq!(
                g.object_view(found).zone,
                if pay_sacrifice && find_card {
                    Zone::Battlefield
                } else {
                    Zone::Library
                }
            );
        }
    }
}

#[test]
fn jiang_yanggu_creates_legendary_mowu_and_untaps_all_owned_tokens() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let jiang = put(&mut g, p, "Jiang Yanggu, Never Alone", Zone::Hand);
        cast(&mut g, jiang, None);
        settle(&mut g);
        let mowu = g
            .state
            .objects
            .iter()
            .enumerate()
            .find_map(|(id, o)| {
                (o.zone == Zone::Battlefield
                    && o.controller == p
                    && o.is_token()
                    && g.object_name(id as CardId) == "Mowu")
                    .then_some(id as CardId)
            })
            .expect("Mowu token");
        let view = g.object_view(mowu);
        assert!(view.is_legendary());
        assert!(g.has_subtype(mowu, "Dog"));
        assert_eq!(view.colors, 16);
        assert_eq!(stats(&g, mowu), (3, 3));
        g.create_tokens(p, Card(5246), 2, false);
        g.create_tokens(p.other(), Card(5246), 2, false);
        let foods: Vec<_> = g
            .state
            .objects
            .iter()
            .enumerate()
            .filter_map(|(id, o)| {
                (o.zone == Zone::Battlefield && o.is_token() && o.card == Card(5246))
                    .then_some((id as CardId, o.controller))
            })
            .collect();
        assert_eq!(foods.len(), 4);
        let stolen = foods
            .iter()
            .find(|(_, owner)| *owner == p.other())
            .unwrap()
            .0;
        let stolen_ref = g.object_ref(stolen);
        g.change_controller(stolen_ref, p);
        for &(id, _) in &foods {
            g.state.objects[id as usize].tapped = true;
        }
        g.state.objects[mowu as usize].tapped = true;
        let stunned = foods.iter().find(|(_, owner)| *owner == p).unwrap().0;
        g.add_counter(stunned, CounterKind::Stun, 1);
        g.begin_step(Step::End);
        g.prepare_triggers();
        super::contracts::branch_contract(g.clone());
        settle(&mut g);
        assert!(!g.object_view(mowu).tapped);
        for (id, owner) in foods {
            assert_eq!(
                g.object_view(id).tapped,
                (owner != p && id != stolen) || id == stunned
            );
        }
        assert_eq!(g.object_view(stolen).owner, p.other());
        assert_eq!(g.counter_count(stunned, CounterKind::Stun), 0);
    }
}

#[test]
fn tether_technician_discards_before_choosing_reflexive_target() {
    for p in [Player::P0, Player::P1] {
        for pay in [false, true] {
            let mut g = scenario(p);
            let hand = put(&mut g, p, "Forest", Zone::Hand);
            let opposing_hand = put(&mut g, p.other(), "Forest", Zone::Hand);
            let creature = put(&mut g, p, "Tether Technician", Zone::Hand);
            let life = g.state.players[p.other().index()].life;
            cast(&mut g, creature, None);
            let mut decided = false;
            let mut targeted = false;
            for _ in 0..64 {
                g.prepare_triggers();
                let actions = g.legal_actions();
                if !decided && actions.contains(&Action::DeclineEffectPayment) {
                    assert!(!actions.contains(&Action::ChooseTarget {
                        target: Target::Player(p.other())
                    }));
                    assert!(g.observe(p.other()).effect_choice.is_none());
                    reject(
                        &mut g,
                        Action::Discard {
                            card: opposing_hand,
                        },
                    );
                    super::contracts::branch_contract(g.clone());
                    if pay {
                        act(&mut g, Action::Discard { card: hand });
                        g.move_object(creature, Zone::Graveyard);
                    } else {
                        act(&mut g, Action::DeclineEffectPayment);
                    }
                    decided = true;
                } else if pay
                    && !targeted
                    && actions.contains(&Action::ChooseTarget {
                        target: Target::Player(p.other()),
                    })
                {
                    assert!(decided);
                    act(
                        &mut g,
                        Action::ChooseTarget {
                            target: Target::Player(p.other()),
                        },
                    );
                    targeted = true;
                } else if actions.contains(&Action::FinishTargets) {
                    act(&mut g, Action::FinishTargets);
                } else if decided
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
                    panic!("unexpected Tether choice {:?}", g.state.pending);
                }
            }
            assert!(decided);
            assert_eq!(targeted, pay);
            assert_eq!(
                g.state.players[p.other().index()].life,
                life - if pay { 2 } else { 0 }
            );
            assert_eq!(
                g.object_view(hand).zone,
                if pay { Zone::Graveyard } else { Zone::Hand }
            );
        }
    }
}
