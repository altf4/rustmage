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
        g.move_object(seventh, Zone::Exile);
        assert_eq!(stats(&g, creature), (3, 2));
        let aura = put(&mut g, p.other(), "Witness Protection", Zone::Battlefield);
        g.state.objects[aura as usize].attached_to = Some(g.object_ref(creature));
        assert!(!g.printed_abilities_available(creature));
        assert_eq!(stats(&g, creature), (1, 1));
    }
}
