use super::*;
use crate::cards::{AnimationCreatureTypes, AnimationDuration, AnimationSpec, ChoiceColor};

// Draft definitions remain gated in the public catalogue. Test their authored
// behavior directly without presenting them as supported deck entries.
fn put(g: &mut Game, owner: Player, name: &str, zone: Zone) -> CardId {
    let card = Card::from_name(name).unwrap_or_else(|| panic!("unknown fixture {name}"));
    if card.is_supported() {
        return super::put(g, owner, name, zone);
    }
    let id = g.state.objects.len() as CardId;
    g.state.objects.push(Object {
        copy_kind: CopyKind::None,
        card,
        owner,
        controller: owner,
        zone,
        generation: 0,
        tapped: false,
        summoning_sick: false,
        flashback: false,
        cast_for_warp: false,
        damage: 0,
        damaged_this_turn: false,
        dealt_damage_this_turn: false,
        deathtouch_since_sba: false,
        growth: Stats::default(),
        gained_keywords: Keywords::NONE,
        counters: InitialCounters::default(),
        used_abilities: 0,
        loyalty_activated_this_turn: false,
        class_level: 1,
        attached_to: None,
    });
    match zone {
        Zone::Library => g.state.players[owner.index()].library.push(id),
        Zone::Battlefield => g.register_battlefield_sources(id),
        Zone::Graveyard => g.register_graveyard_sources(id),
        _ => (),
    }
    id
}

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
            if g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::ChooseSpellCost { .. }))
            {
                choose(&mut g, |a| matches!(a, Action::ChooseSpellCost { .. }));
            }
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
                            && o.card == Card::test_index(5288)
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
                    && o.card == Card::test_index(5435))
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
                && o.card == Card::test_index(5435))
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
        let own = put(&mut g, p, "Liliana, Dreadhorde General", Zone::Battlefield);
        let opposing = put(
            &mut g,
            p.other(),
            "Liliana, Dreadhorde General",
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
        g.prepare_triggers();
        while matches!(g.state.pending, Pending::OrderTriggers { .. }) {
            choose(&mut g, |action| matches!(action, Action::PutTrigger { .. }));
        }
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
        for _ in 0..32 {
            if matches!(g.state.pending, Pending::EffectChoice { .. }) {
                break;
            }
            act(&mut g, Action::Pass);
        }
        choose(&mut g, |a| matches!(a, Action::ChooseJace { .. }));
        settle(&mut g);
        let jace = fra_jaces(&g, p)[0];
        assert_eq!(g.counter_count(jace, CounterKind::Loyalty), 10);
        g.gain_life(p, 1);
        g.prepare_triggers();
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
            let artifact = put(&mut g, caster, "Goldvein Pick", Zone::Hand);
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
                && o.card == Card::test_index(5266))
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
                    "Liliana, Dreadhorde General",
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
                    "Liliana, Dreadhorde General"
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
                        1 | 3 => "Liliana, Dreadhorde General",
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
            ("The Mind Stone", false, true, false),
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
            let old = put(&mut g, p, "Liliana, Dreadhorde General", Zone::Battlefield);
            g.add_counter(old, CounterKind::Loyalty, 5);
            let illegal = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let other = put(
                &mut g,
                p.other(),
                "Liliana, Dreadhorde General",
                Zone::Battlefield,
            );
            g.add_counter(other, CounterKind::Loyalty, 5);
            let found = put(&mut g, p, "Liliana, Dreadhorde General", Zone::Library);
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
        g.create_tokens(p, Card::test_index(5246), 2, false);
        g.create_tokens(p.other(), Card::test_index(5246), 2, false);
        let foods: Vec<_> = g
            .state
            .objects
            .iter()
            .enumerate()
            .filter_map(|(id, o)| {
                (o.zone == Zone::Battlefield && o.is_token() && o.card == Card::test_index(5246))
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

#[test]
fn curse_marred_demon_searches_privately_then_randomly_discards_with_replay() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let old_hand = put(&mut g, p, "Forest", Zone::Hand);
        let found = put(&mut g, p, "Shock", Zone::Library);
        let source = put(&mut g, p, "Curse-Marred Demon", Zone::Hand);
        cast(&mut g, source, None);
        for _ in 0..48 {
            g.prepare_triggers();
            if g.observe(p).search_choice.is_some() {
                break;
            }
            let action = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::Pass | Action::PutTrigger { .. }))
                .expect("search progression");
            act(&mut g, action);
        }
        assert!(g.observe(p).search_choice.is_some());
        assert!(g.observe(p.other()).search_choice.is_none());
        assert!(
            !g.observe(p.other())
                .objects
                .iter()
                .any(|o| o.id == old_hand)
        );
        reject(&mut g, Action::FinishSearch);
        act(&mut g, Action::ChooseSearch { card: found });
        super::contracts::branch_contract(g.clone());
        let mut replay = g.clone();
        act(&mut g, Action::FinishSearch);
        act(&mut replay, Action::FinishSearch);
        settle(&mut g);
        settle(&mut replay);
        assert_eq!(
            g.object_view(old_hand).zone,
            replay.object_view(old_hand).zone
        );
        assert_eq!(g.object_view(found).zone, replay.object_view(found).zone);
        assert_eq!(
            [old_hand, found]
                .into_iter()
                .filter(|&id| g.object_view(id).zone == Zone::Graveyard)
                .count(),
            1
        );
        assert_eq!(g.state.players[p.index()].cards_discarded_this_turn, 1);
        assert!(g.keywords(source).contains(Keywords::FLYING));
        assert!(g.keywords(source).contains(Keywords::TRAMPLE));
    }
}

#[test]
fn traxos_academy_guardian_discount_tracks_noncreature_casts_and_prowess() {
    for p in [Player::P0, Player::P1] {
        for history in 0..3 {
            let mut g = scenario(p);
            if history == 1 {
                let bear = put(&mut g, p, "Bear Cub", Zone::Hand);
                cast(&mut g, bear, None);
                settle(&mut g);
            } else if history == 2 {
                let shock = put(&mut g, p, "Shock", Zone::Hand);
                cast(&mut g, shock, Some(Target::Player(p.other())));
                settle(&mut g);
            }
            let traxos = put(&mut g, p, "Traxos, Academy Guardian", Zone::Hand);
            g.state.players[p.index()].mana = [0; 6];
            g.state.players[p.index()].mana[1] = 1;
            g.state.players[p.index()].mana[5] = 1;
            if history != 2 {
                reject(&mut g, Action::AnnounceCast { card: traxos });
                g.state.players[p.index()].mana[5] = 3;
            }
            act(&mut g, Action::AnnounceCast { card: traxos });
            pay(&mut g);
            super::contracts::branch_contract(g.clone());
            settle(&mut g);
            assert_eq!(stats(&g, traxos), (1, 5));
            assert!(g.keywords(traxos).contains(Keywords::FLYING));
            assert!(g.keywords(traxos).contains(Keywords::VIGILANCE));
            g.state.players[p.index()].mana = [8; 6];
            let bear = put(&mut g, p, "Bear Cub", Zone::Hand);
            cast(&mut g, bear, None);
            settle(&mut g);
            assert_eq!(stats(&g, traxos), (1, 5));
            let shock = put(&mut g, p, "Shock", Zone::Hand);
            cast(&mut g, shock, Some(Target::Player(p.other())));
            settle(&mut g);
            assert_eq!(stats(&g, traxos), (2, 6));
            g.finish_cleanup();
            assert_eq!(stats(&g, traxos), (1, 5));
        }
    }
}

#[test]
fn vraska_cutting_glare_rechecks_six_lands_and_creates_treasure_after_destruction() {
    for p in [Player::P0, Player::P1] {
        for (lands, lose_land, indestructible) in [
            (5, false, false),
            (6, true, false),
            (6, false, false),
            (6, false, true),
        ] {
            let mut g = scenario(p);
            let land_ids: Vec<_> = (0..lands)
                .map(|_| put(&mut g, p, "Forest", Zone::Battlefield))
                .collect();
            let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            if indestructible {
                g.state.objects[target as usize].gained_keywords = Keywords::INDESTRUCTIBLE;
            }
            let vraska = put(&mut g, p, "Vraska, the Cutting Glare", Zone::Hand);
            cast(&mut g, vraska, None);
            let mut selected = false;
            for _ in 0..64 {
                g.prepare_triggers();
                let actions = g.legal_actions();
                let wanted = Action::ChooseTarget {
                    target: Target::Card(target),
                };
                if !selected && actions.contains(&wanted) {
                    reject(
                        &mut g,
                        Action::ChooseTarget {
                            target: Target::Card(own),
                        },
                    );
                    act(&mut g, wanted);
                    selected = true;
                    if lose_land {
                        g.move_object(land_ids[0], Zone::Graveyard);
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
                    panic!("unexpected Vraska choice {:?}", g.state.pending);
                }
            }
            assert_eq!(selected, lands >= 6);
            assert!(g.keywords(vraska).contains(Keywords::DEATHTOUCH));
            let triggered = lands >= 6 && !lose_land;
            assert_eq!(
                g.object_view(target).zone,
                if triggered && !indestructible {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
            let treasures = g
                .state
                .objects
                .iter()
                .filter(|o| {
                    o.zone == Zone::Battlefield
                        && o.controller == p.other()
                        && o.is_token()
                        && o.card == Card::test_index(5302)
                })
                .count();
            assert_eq!(treasures, usize::from(triggered));
        }
    }
}

#[test]
fn mabel_bitter_recluse_selects_mixed_counters_and_removes_them_together() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let target = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        g.add_counter(target, CounterKind::PlusOne, 1);
        g.add_counter(target, CounterKind::Stun, 2);
        g.add_counter(target, CounterKind::Charge, 2);
        let mabel = put(&mut g, p, "Mabel, Bitter Recluse", Zone::Hand);
        cast(&mut g, mabel, None);
        let mut targeted = false;
        for _ in 0..64 {
            g.prepare_triggers();
            if matches!(g.state.pending, Pending::TargetCounterRemoval { .. }) {
                break;
            }
            let actions = g.legal_actions();
            let wanted = Action::ChooseTarget {
                target: Target::Card(target),
            };
            if !targeted && actions.contains(&wanted) {
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(mabel),
                    },
                );
                act(&mut g, wanted);
                targeted = true;
            } else if actions.contains(&Action::FinishTargets) {
                act(&mut g, Action::FinishTargets);
            } else if let Some(action) = actions
                .into_iter()
                .find(|a| matches!(a, Action::Pass | Action::PutTrigger { .. }))
            {
                act(&mut g, action);
            } else {
                panic!("unexpected Mabel counter choice {:?}", g.state.pending);
            }
        }
        assert!(targeted);
        assert_eq!(g.actor(), Some(p));
        assert!(g.observe(p.other()).target_counter_removal.is_none());
        assert_eq!(
            g.observe(p).target_counter_removal.unwrap().target,
            g.object_ref(target)
        );
        let own_wire = serde_json::to_value(g.observe(p)).unwrap();
        let other_wire = serde_json::to_value(g.observe(p.other())).unwrap();
        assert!(own_wire.get("target_counter_removal").is_some());
        assert!(other_wire.get("target_counter_removal").is_none());
        let choice = Action::SelectTargetCounter {
            counter: CounterKind::Charge,
        };
        assert_eq!(
            serde_json::from_value::<Action>(serde_json::to_value(&choice).unwrap()).unwrap(),
            choice
        );
        let before = g.snapshot();
        assert_eq!(
            g.apply(
                p.other(),
                Action::SelectTargetCounter {
                    counter: CounterKind::Stun
                }
            ),
            Err(Error::WrongActor)
        );
        assert_eq!(g.snapshot(), before);
        reject(
            &mut g,
            Action::SelectTargetCounter {
                counter: CounterKind::Loyalty,
            },
        );
        act(
            &mut g,
            Action::SelectTargetCounter {
                counter: CounterKind::PlusOne,
            },
        );
        act(
            &mut g,
            Action::SelectTargetCounter {
                counter: CounterKind::Stun,
            },
        );
        act(
            &mut g,
            Action::SelectTargetCounter {
                counter: CounterKind::Charge,
            },
        );
        reject(
            &mut g,
            Action::SelectTargetCounter {
                counter: CounterKind::Charge,
            },
        );
        assert_eq!(g.counter_count(target, CounterKind::PlusOne), 1);
        assert_eq!(g.counter_count(target, CounterKind::Stun), 2);
        assert_eq!(g.counter_count(target, CounterKind::Charge), 2);
        super::contracts::branch_contract(g.clone());
        let checkpoint = g.snapshot();
        act(&mut g, Action::FinishTargetCounterRemoval);
        settle(&mut g);
        assert_eq!(g.counter_count(target, CounterKind::PlusOne), 0);
        assert_eq!(g.counter_count(target, CounterKind::Stun), 1);
        assert_eq!(g.counter_count(target, CounterKind::Charge), 1);
        g.restore(&checkpoint).unwrap();
        assert_eq!(g.counter_count(target, CounterKind::Charge), 2);
        act(&mut g, Action::FinishTargetCounterRemoval);
        settle(&mut g);
        assert_eq!(g.counter_count(target, CounterKind::Charge), 1);
        assert!(g.keywords(mabel).contains(Keywords::DEATHTOUCH));
    }
}

#[test]
fn mabel_counter_choice_handles_zero_loyalty_and_departed_targets() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..4 {
            let mut g = scenario(p);
            let target = put(
                &mut g,
                p.other(),
                if mode == 2 {
                    "Bear Cub"
                } else {
                    "Liliana, Dreadhorde General"
                },
                Zone::Battlefield,
            );
            if mode != 2 {
                g.add_counter(target, CounterKind::Loyalty, 2);
            }
            let mabel = put(&mut g, p, "Mabel, Bitter Recluse", Zone::Hand);
            cast(&mut g, mabel, None);
            let mut targeted = false;
            let mut finished = false;
            for _ in 0..64 {
                g.prepare_triggers();
                let actions = g.legal_actions();
                let wanted = Action::ChooseTarget {
                    target: Target::Card(target),
                };
                if !targeted && actions.contains(&wanted) {
                    act(&mut g, wanted);
                    targeted = true;
                    if mode == 3 {
                        g.move_object(target, Zone::Exile);
                    }
                } else if matches!(g.state.pending, Pending::TargetCounterRemoval { .. }) {
                    assert!(targeted);
                    assert_ne!(mode, 2);
                    assert_ne!(mode, 3);
                    if mode == 1 {
                        act(
                            &mut g,
                            Action::SelectTargetCounter {
                                counter: CounterKind::Loyalty,
                            },
                        );
                        assert_eq!(g.object_view(target).zone, Zone::Battlefield);
                        act(
                            &mut g,
                            Action::SelectTargetCounter {
                                counter: CounterKind::Loyalty,
                            },
                        );
                        assert_eq!(g.object_view(target).zone, Zone::Battlefield);
                    }
                    act(&mut g, Action::FinishTargetCounterRemoval);
                    finished = true;
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
                    panic!("unexpected Mabel resolution {:?}", g.state.pending);
                }
            }
            assert!(targeted);
            assert_eq!(finished, mode < 2);
            assert_eq!(
                g.object_view(target).zone,
                if mode == 1 {
                    Zone::Graveyard
                } else if mode == 3 {
                    Zone::Exile
                } else {
                    Zone::Battlefield
                }
            );
            if mode == 0 {
                assert_eq!(g.counter_count(target, CounterKind::Loyalty), 2);
            }
        }
    }
}

#[test]
fn mabel_counter_target_rechecks_hexproof_and_survives_source_departure() {
    for p in [Player::P0, Player::P1] {
        for hexproof in [false, true] {
            let mut g = scenario(p);
            let target = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            g.add_counter(target, CounterKind::Stun, 1);
            let source = put(&mut g, p, "Mabel, Bitter Recluse", Zone::Hand);
            cast(&mut g, source, None);
            let mut targeted = false;
            let mut selected = false;
            for _ in 0..64 {
                g.prepare_triggers();
                let actions = g.legal_actions();
                let wanted = Action::ChooseTarget {
                    target: Target::Card(target),
                };
                if !targeted && actions.contains(&wanted) {
                    act(&mut g, wanted);
                    targeted = true;
                    if hexproof {
                        g.state.objects[target as usize].gained_keywords = Keywords::HEXPROOF;
                    } else {
                        g.move_object(source, Zone::Graveyard);
                    }
                } else if matches!(g.state.pending, Pending::TargetCounterRemoval { .. }) {
                    assert!(!hexproof);
                    act(
                        &mut g,
                        Action::SelectTargetCounter {
                            counter: CounterKind::Stun,
                        },
                    );
                    act(&mut g, Action::FinishTargetCounterRemoval);
                    selected = true;
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
                    panic!("unexpected Mabel target transition {:?}", g.state.pending);
                }
            }
            assert!(targeted);
            assert_eq!(selected, !hexproof);
            assert_eq!(
                g.counter_count(target, CounterKind::Stun),
                u64::from(hexproof)
            );
        }
        let mut g = scenario(p);
        let source = put(&mut g, p, "Mabel, Bitter Recluse", Zone::Hand);
        cast(&mut g, source, None);
        settle(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Battlefield);
        assert!(g.state.pending_triggers.is_empty());
    }
}

#[test]
fn mabel_can_remove_a_hexproof_counter_from_an_owned_creature() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let target = put(&mut g, p, "Hill Giant", Zone::Battlefield);
        g.add_counter(target, CounterKind::Hexproof, 1);
        assert!(g.keywords(target).contains(Keywords::HEXPROOF));
        let source = put(&mut g, p, "Mabel, Bitter Recluse", Zone::Hand);
        cast(&mut g, source, None);
        let mut targeted = false;
        for _ in 0..64 {
            g.prepare_triggers();
            if matches!(g.state.pending, Pending::TargetCounterRemoval { .. }) {
                break;
            }
            let actions = g.legal_actions();
            let wanted = Action::ChooseTarget {
                target: Target::Card(target),
            };
            if !targeted && actions.contains(&wanted) {
                act(&mut g, wanted);
                targeted = true;
            } else if actions.contains(&Action::FinishTargets) {
                act(&mut g, Action::FinishTargets);
            } else if let Some(action) = actions
                .into_iter()
                .find(|a| matches!(a, Action::Pass | Action::PutTrigger { .. }))
            {
                act(&mut g, action);
            } else {
                panic!("unexpected Mabel hexproof choice {:?}", g.state.pending);
            }
        }
        assert!(targeted);
        super::contracts::branch_contract(g.clone());
        let before = g.snapshot();
        act(
            &mut g,
            Action::SelectTargetCounter {
                counter: CounterKind::Hexproof,
            },
        );
        assert!(g.keywords(target).contains(Keywords::HEXPROOF));
        act(&mut g, Action::FinishTargetCounterRemoval);
        settle(&mut g);
        assert_eq!(g.counter_count(target, CounterKind::Hexproof), 0);
        assert!(!g.keywords(target).contains(Keywords::HEXPROOF));
        g.restore(&before).unwrap();
        assert_eq!(g.counter_count(target, CounterKind::Hexproof), 1);
        assert!(g.keywords(target).contains(Keywords::HEXPROOF));
    }
}

#[test]
fn mabel_can_select_different_pinned_named_counter_kinds() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let target = put(&mut g, p, "Hill Giant", Zone::Battlefield);
        for counter in [
            CounterKind::Blight,
            CounterKind::Conqueror,
            CounterKind::Net,
            CounterKind::Time,
        ] {
            g.add_counter(target, counter, 1);
        }
        let source = put(&mut g, p, "Mabel, Bitter Recluse", Zone::Hand);
        cast(&mut g, source, None);
        let mut targeted = false;
        for _ in 0..64 {
            g.prepare_triggers();
            if matches!(g.state.pending, Pending::TargetCounterRemoval { .. }) {
                break;
            }
            let actions = g.legal_actions();
            let wanted = Action::ChooseTarget {
                target: Target::Card(target),
            };
            if !targeted && actions.contains(&wanted) {
                act(&mut g, wanted);
                targeted = true;
            } else if actions.contains(&Action::FinishTargets) {
                act(&mut g, Action::FinishTargets);
            } else if let Some(action) = actions
                .into_iter()
                .find(|a| matches!(a, Action::Pass | Action::PutTrigger { .. }))
            {
                act(&mut g, action);
            } else {
                panic!("unexpected named counter choice {:?}", g.state.pending);
            }
        }
        assert!(targeted);
        for counter in [CounterKind::Blight, CounterKind::Net, CounterKind::Time] {
            act(&mut g, Action::SelectTargetCounter { counter });
        }
        let wire = serde_json::to_value(g.observe(p)).unwrap();
        assert!(wire.get("target_counter_removal").is_some());
        super::contracts::branch_contract(g.clone());
        act(&mut g, Action::FinishTargetCounterRemoval);
        settle(&mut g);
        for counter in [CounterKind::Blight, CounterKind::Net, CounterKind::Time] {
            assert_eq!(g.counter_count(target, counter), 0);
        }
        assert_eq!(g.counter_count(target, CounterKind::Conqueror), 1);
    }
}

#[test]
fn marwyn_protects_controlled_lands_and_returns_own_graveyard_land() {
    for p in [Player::P0, Player::P1] {
        for target_leaves in [false, true] {
            let mut g = scenario(p);
            let source = put(&mut g, p, "Marwyn, the Preserver", Zone::Battlefield);
            let own_land = put(&mut g, p, "Forest", Zone::Battlefield);
            let opposing_land = put(&mut g, p.other(), "Forest", Zone::Battlefield);
            let creature = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let own_grave_land = put(&mut g, p, "Forest", Zone::Graveyard);
            let opposing_grave_land = put(&mut g, p.other(), "Forest", Zone::Graveyard);
            let nonland = put(&mut g, p, "Bear Cub", Zone::Graveyard);
            assert!(g.keywords(own_land).contains(Keywords::HEXPROOF));
            assert!(!g.keywords(opposing_land).contains(Keywords::HEXPROOF));
            assert!(!g.keywords(creature).contains(Keywords::HEXPROOF));
            g.state.players[p.index()].mana[5] = 2;
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
                    target: Target::Card(opposing_grave_land),
                },
            );
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(nonland),
                },
            );
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(own_grave_land),
                },
            );
            pay(&mut g);
            super::contracts::branch_contract(g.clone());
            g.move_object(source, Zone::Exile);
            assert!(!g.keywords(own_land).contains(Keywords::HEXPROOF));
            if target_leaves {
                g.move_object(own_grave_land, Zone::Exile);
            }
            settle(&mut g);
            assert_eq!(
                g.object_view(own_grave_land).zone,
                if target_leaves {
                    Zone::Exile
                } else {
                    Zone::Hand
                }
            );
        }
    }
}

#[test]
fn marwyn_land_hexproof_changes_actual_target_legality() {
    let land_rule = TargetRule::Object {
        zone: TargetZone::Battlefield,
        kinds: 2,
        relation: -1,
        other: false,
        nonland: false,
        tapped: false,
        min_mana: 0,
        max_mana: u32::MAX,
    };
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let marwyn = put(&mut g, p, "Marwyn, the Preserver", Zone::Battlefield);
        let land = put(&mut g, p, "Forest", Zone::Battlefield);
        let own_source = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let opposing_source = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        assert!(g.cast_target_matches(land_rule, p, g.object_ref(own_source), Target::Card(land)));
        assert!(!g.cast_target_matches(
            land_rule,
            p.other(),
            g.object_ref(opposing_source),
            Target::Card(land),
        ));
        let before = g.snapshot();
        super::contracts::branch_contract(g.clone());
        g.move_object(marwyn, Zone::Exile);
        assert!(g.cast_target_matches(
            land_rule,
            p.other(),
            g.object_ref(opposing_source),
            Target::Card(land),
        ));
        g.restore(&before).unwrap();
        assert!(!g.cast_target_matches(
            land_rule,
            p.other(),
            g.object_ref(opposing_source),
            Target::Card(land),
        ));
    }
}

#[test]
fn stingerquill_charm_modes_and_doubled_cadet_haste() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..3 {
            let mut g = scenario(p);
            let creature = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            g.create_tokens(p, Card::test_index(5298), 1, false);
            settle(&mut g);
            let old_cadet = g
                .state
                .objects
                .iter()
                .enumerate()
                .find_map(|(i, o)| {
                    (o.zone == Zone::Battlefield
                        && o.is_token()
                        && o.card == Card::test_index(5298))
                    .then_some(i as CardId)
                })
                .unwrap();
            if mode == 2 {
                put(&mut g, p, "Doubling Season", Zone::Battlefield);
            }
            let charm = put(&mut g, p, "Stingerquill Charm", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 1, 1, 0, 0];
            let life = g.state.players[p.other().index()].life;
            act(&mut g, Action::AnnounceCast { card: charm });
            act(&mut g, Action::ChooseMode { mode });
            if mode == 0 {
                act(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Player(p.other()),
                    },
                );
            } else if mode == 1 {
                act(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(creature),
                    },
                );
            }
            pay(&mut g);
            super::contracts::branch_contract(g.clone());
            settle(&mut g);
            match mode {
                0 => assert_eq!(g.state.players[p.other().index()].life, life - 3),
                1 => {
                    assert!(g.keywords(creature).contains(Keywords::FIRST_STRIKE));
                    assert!(g.keywords(creature).contains(Keywords::DEATHTOUCH));
                    g.finish_cleanup();
                    assert!(!g.keywords(creature).contains(Keywords::FIRST_STRIKE));
                    assert!(!g.keywords(creature).contains(Keywords::DEATHTOUCH));
                }
                _ => {
                    let cadets: Vec<_> = g
                        .state
                        .objects
                        .iter()
                        .enumerate()
                        .filter_map(|(i, o)| {
                            (o.zone == Zone::Battlefield
                                && o.controller == p
                                && o.is_token()
                                && o.card == Card::test_index(5298)
                                && i as CardId != old_cadet)
                                .then_some(i as CardId)
                        })
                        .collect();
                    assert_eq!(cadets.len(), 2);
                    assert!(!g.keywords(old_cadet).contains(Keywords::HASTE));
                    for &id in &cadets {
                        assert!(g.keywords(id).contains(Keywords::HASTE));
                        assert_eq!(stats(&g, id), (2, 2));
                    }
                    g.finish_cleanup();
                    for id in cadets {
                        assert!(!g.keywords(id).contains(Keywords::HASTE));
                    }
                }
            }
        }
    }
}

#[test]
fn way_of_the_necromancer_empowers_and_triggers_for_each_controlled_death() {
    for p in [Player::P0, Player::P1] {
        for source_leaves in [false, true] {
            let mut g = scenario(p);
            let way = put(&mut g, p, "Way of the Necromancer", Zone::Hand);
            cast(&mut g, way, None);
            settle(&mut g);
            let jaces = fra_jaces(&g, p);
            assert_eq!(jaces.len(), 1);
            let jace = jaces[0];
            assert_eq!(g.counter_count(jace, CounterKind::Loyalty), 2);
            let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            g.move_object(enemy, Zone::Graveyard);
            g.prepare_triggers();
            settle(&mut g);
            assert_eq!(g.counter_count(jace, CounterKind::Loyalty), 2);
            let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            g.move_object(own, Zone::Graveyard);
            g.prepare_triggers();
            assert!(!g.state.stack.is_empty() || !g.state.pending_triggers.is_empty());
            if source_leaves {
                g.move_object(way, Zone::Graveyard);
            }
            let before = g.snapshot();
            super::contracts::branch_contract(g.clone());
            settle(&mut g);
            assert_eq!(g.counter_count(jace, CounterKind::Loyalty), 3);
            g.restore(&before).unwrap();
            settle(&mut g);
            assert_eq!(g.counter_count(jace, CounterKind::Loyalty), 3);
        }
    }
}

#[test]
fn repurposed_enforcer_counts_controlled_creatures_when_attack_trigger_resolves() {
    for p in [Player::P0, Player::P1] {
        for attacker_leaves in [false, true] {
            let mut g = scenario(p);
            let enforcer = put(&mut g, p, "Repurposed Enforcer", Zone::Battlefield);
            put(&mut g, p, "Bear Cub", Zone::Battlefield);
            put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            g.state.objects[enforcer as usize].summoning_sick = false;
            g.state.step = Step::Attackers;
            g.state.pending = Pending::Attackers;
            act(&mut g, Action::Attack { card: enforcer });
            act(&mut g, Action::FinishAttackers);
            if attacker_leaves {
                g.move_object(enforcer, Zone::Graveyard);
            }
            put(&mut g, p, "Hill Giant", Zone::Battlefield);
            let before = g.snapshot();
            super::contracts::branch_contract(g.clone());
            settle(&mut g);
            let jaces = fra_jaces(&g, p);
            assert_eq!(jaces.len(), 1);
            assert_eq!(
                g.counter_count(jaces[0], CounterKind::Loyalty),
                if attacker_leaves { 2 } else { 3 }
            );
            g.restore(&before).unwrap();
            settle(&mut g);
            let jaces = fra_jaces(&g, p);
            assert_eq!(jaces.len(), 1);
            assert_eq!(
                g.counter_count(jaces[0], CounterKind::Loyalty),
                if attacker_leaves { 2 } else { 3 }
            );
        }
    }
}

#[test]
fn arni_uses_entering_creatures_current_or_last_known_power() {
    for p in [Player::P0, Player::P1] {
        for (leave, negative) in [(false, false), (true, false), (true, true)] {
            let mut g = scenario(p);
            let arni = put(&mut g, p, "Arni, Renowned Champion", Zone::Battlefield);
            let entrant = put(&mut g, p, "Bear Cub", Zone::Hand);
            g.move_object(entrant, Zone::Battlefield);
            g.prepare_triggers();
            assert!(!g.state.stack.is_empty() || !g.state.pending_triggers.is_empty());
            if negative {
                g.state.objects[entrant as usize].growth.power = -5;
            } else {
                g.state.objects[entrant as usize].growth.power = 2;
            }
            if leave {
                g.move_object(entrant, Zone::Graveyard);
            }
            let before = g.snapshot();
            super::contracts::branch_contract(g.clone());
            settle(&mut g);
            assert_eq!(stats(&g, arni), (if negative { 1 } else { 5 }, 5));
            g.restore(&before).unwrap();
            settle(&mut g);
            assert_eq!(stats(&g, arni), (if negative { 1 } else { 5 }, 5));
            g.finish_cleanup();
            assert_eq!(stats(&g, arni), (1, 5));
        }
    }
}

#[test]
fn massacre_girl_death_damage_triggers_her_counter_ability() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let girl = put(&mut g, p, "Massacre Girl, Most Wanted", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.move_object(enemy, Zone::Graveyard);
        g.prepare_triggers();
        settle(&mut g);
        assert_eq!(g.counter_count(girl, CounterKind::PlusOne), 0);
        let life = g.state.players[p.index()].life;
        let opponent_life = g.state.players[p.other().index()].life;
        let ally = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.move_object(ally, Zone::Graveyard);
        let mut targeted = false;
        for _ in 0..64 {
            g.prepare_triggers();
            let actions = g.legal_actions();
            let target = Action::ChooseTarget {
                target: Target::Player(p.other()),
            };
            if !targeted && actions.contains(&target) {
                act(&mut g, target);
                targeted = true;
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
                panic!("unexpected Massacre Girl choice {:?}", g.state.pending);
            }
        }
        assert!(targeted);
        assert_eq!(g.state.players[p.index()].life, life + 1);
        assert_eq!(g.state.players[p.other().index()].life, opponent_life - 1);
        assert_eq!(g.counter_count(girl, CounterKind::PlusOne), 1);
    }
}

#[test]
fn vraskas_final_mercy_modes_and_illegal_target() {
    for p in [Player::P0, Player::P1] {
        for case in 0..4 {
            let mut g = scenario(p);
            let victim = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            if case == 2 {
                g.state.objects[victim as usize].gained_keywords = Keywords::INDESTRUCTIBLE;
            }
            let spell = put(&mut g, p, "Vraska's Final Mercy", Zone::Hand);
            let life = g.state.players[p.index()].life;
            g.state.players[p.index()].mana = [0, 0, 2, 0, 0, 0];
            act(&mut g, Action::AnnounceCast { card: spell });
            act(
                &mut g,
                Action::ChooseMode {
                    mode: if case == 3 { 1 } else { 0 },
                },
            );
            if case != 3 {
                act(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(victim),
                    },
                );
            }
            pay(&mut g);
            if case == 1 {
                g.move_object(victim, Zone::Exile);
            }
            let before = g.snapshot();
            super::contracts::branch_contract(g.clone());
            settle(&mut g);
            assert_eq!(
                g.state.players[p.index()].life,
                life - if case == 1 { 0 } else { 2 }
            );
            match case {
                0 => assert_eq!(g.object_view(victim).zone, Zone::Graveyard),
                1 => assert_eq!(g.object_view(victim).zone, Zone::Exile),
                2 => assert_eq!(g.object_view(victim).zone, Zone::Battlefield),
                _ => {
                    let jaces = fra_jaces(&g, p);
                    assert_eq!(jaces.len(), 1);
                    assert_eq!(g.counter_count(jaces[0], CounterKind::Loyalty), 6);
                }
            }
            g.restore(&before).unwrap();
            settle(&mut g);
            assert_eq!(
                g.state.players[p.index()].life,
                life - if case == 1 { 0 } else { 2 }
            );
        }
    }
}

#[test]
fn way_of_the_paradox_observes_committed_loyalty_activations() {
    for p in [Player::P0, Player::P1] {
        for way_leaves in [false, true] {
            let mut g = scenario(p);
            let way = put(&mut g, p, "Way of the Paradox", Zone::Hand);
            cast(&mut g, way, None);
            settle(&mut g);
            let jace = fra_jaces(&g, p)[0];
            let life = g.state.players[p.index()].life;
            let allowance = g.state.players[p.index()].extra_land_plays_remaining;
            let activation = Action::AnnounceActivate {
                card: jace,
                ability: 1,
                counter_payment: Counters::default(),
            };
            g.state.priority = p.other();
            reject(&mut g, activation.clone());
            assert_eq!(g.state.players[p.index()].life, life);
            assert_eq!(
                g.state.players[p.index()].extra_land_plays_remaining,
                allowance
            );
            g.state.priority = p;
            act(&mut g, activation);
            pay(&mut g);
            if way_leaves {
                g.move_object(way, Zone::Graveyard);
            }
            let before = g.snapshot();
            super::contracts::branch_contract(g.clone());
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].life, life + 1);
            assert_eq!(
                g.state.players[p.index()].extra_land_plays_remaining,
                allowance + 1
            );
            g.restore(&before).unwrap();
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].life, life + 1);
            assert_eq!(
                g.state.players[p.index()].extra_land_plays_remaining,
                allowance + 1
            );
        }
    }
}

#[test]
fn inspired_tethermage_observes_loyalty_costs_and_its_own_empower() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let mage = put(&mut g, p, "Inspired Tethermage", Zone::Battlefield);
        let liliana = put(&mut g, p, "Liliana, Dreadhorde General", Zone::Battlefield);
        g.add_counter(liliana, CounterKind::Loyalty, 6);
        act(
            &mut g,
            Action::AnnounceActivate {
                card: liliana,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        super::contracts::branch_contract(g.clone());
        settle(&mut g);
        assert_eq!(g.counter_count(mage, CounterKind::PlusOne), 1);
        g.state.players[p.index()].mana[5] = 6;
        act(
            &mut g,
            Action::AnnounceActivate {
                card: mage,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        settle(&mut g);
        let jaces = fra_jaces(&g, p);
        assert_eq!(jaces.len(), 1);
        assert_eq!(g.counter_count(jaces[0], CounterKind::Loyalty), 2);
        assert_eq!(g.counter_count(mage, CounterKind::PlusOne), 2);
    }
}

#[test]
fn hexhaven_battalion_creates_cadets_and_empowers_jace() {
    for p in [Player::P0, Player::P1] {
        for doubled in [false, true] {
            let mut g = scenario(p);
            if doubled {
                put(&mut g, p, "Doubling Season", Zone::Battlefield);
            }
            let spell = put(&mut g, p, "Hexhaven Battalion", Zone::Hand);
            cast(&mut g, spell, None);
            super::contracts::branch_contract(g.clone());
            if doubled {
                for _ in 0..32 {
                    if matches!(g.state.pending, Pending::EffectChoice { .. }) {
                        break;
                    }
                    act(&mut g, Action::Pass);
                }
                let choice = g
                    .legal_actions()
                    .into_iter()
                    .find(|a| matches!(a, Action::ChooseJace { .. }))
                    .expect("doubled Jace choice");
                act(&mut g, choice);
            }
            settle(&mut g);
            let cadets = g
                .state
                .objects
                .iter()
                .filter(|o| {
                    o.zone == Zone::Battlefield
                        && o.controller == p
                        && o.is_token()
                        && o.card == Card::test_index(5298)
                })
                .count();
            assert_eq!(cadets, if doubled { 6 } else { 3 });
            let jaces = fra_jaces(&g, p);
            assert_eq!(jaces.len(), 1);
            assert_eq!(
                jaces
                    .iter()
                    .map(|&id| g.counter_count(id, CounterKind::Loyalty))
                    .sum::<u64>(),
                if doubled { 4 } else { 2 }
            );
        }
    }
}

#[test]
fn hexhaven_battalion_landcycling_discards_and_searches_a_basic_land() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let battalion = put(&mut g, p, "Hexhaven Battalion", Zone::Hand);
        let land = put(&mut g, p, "Forest", Zone::Library);
        let nonland = put(&mut g, p, "Bear Cub", Zone::Library);
        g.state.players[p.index()].mana[5] = 2;
        act(
            &mut g,
            Action::AnnounceActivate {
                card: battalion,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        assert_eq!(g.object_view(battalion).zone, Zone::Graveyard);
        let mut selected = false;
        for _ in 0..64 {
            let actions = g.legal_actions();
            let choice = Action::ChooseSearch { card: land };
            if actions.contains(&choice) {
                reject(&mut g, Action::ChooseSearch { card: nonland });
                super::contracts::branch_contract(g.clone());
                act(&mut g, choice);
                selected = true;
            } else if actions.contains(&Action::FinishSearch) {
                act(&mut g, Action::FinishSearch);
            } else if selected
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
        assert!(selected);
        assert_eq!(g.object_view(land).zone, Zone::Hand);
        assert_eq!(g.object_view(nonland).zone, Zone::Library);
    }
}

#[test]
fn awaken_the_inferno_resolves_each_target_group_independently() {
    for p in [Player::P0, Player::P1] {
        for invalid in 0..4 {
            let mut g = scenario(p);
            let victim = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let ally = put(&mut g, p, "Hill Giant", Zone::Battlefield);
            let spell = put(&mut g, p, "Awaken the Inferno", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 4];
            act(&mut g, Action::AnnounceCast { card: spell });
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(ally),
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
                    target: Target::Card(ally),
                },
            );
            if g.legal_actions().contains(&Action::FinishTargets) {
                act(&mut g, Action::FinishTargets);
            }
            pay(&mut g);
            if invalid == 1 || invalid == 3 {
                g.move_object(victim, Zone::Exile);
            }
            if invalid == 2 || invalid == 3 {
                g.move_object(ally, Zone::Exile);
            }
            let before = g.snapshot();
            super::contracts::branch_contract(g.clone());
            settle(&mut g);
            assert_eq!(
                g.object_view(victim).zone,
                if invalid == 1 || invalid == 3 {
                    Zone::Exile
                } else {
                    Zone::Graveyard
                }
            );
            if invalid != 2 && invalid != 3 {
                assert_eq!(g.counter_count(ally, CounterKind::PlusOne), 1);
            }
            g.restore(&before).unwrap();
            settle(&mut g);
            if invalid != 2 && invalid != 3 {
                assert_eq!(g.counter_count(ally, CounterKind::PlusOne), 1);
            }
        }
    }
}

#[test]
fn rewrite_regrets_reanimates_then_empowers_and_fizzles_for_stale_target() {
    for p in [Player::P0, Player::P1] {
        for stale in [false, true] {
            let mut g = scenario(p);
            let target = put(&mut g, p, "Bear Cub", Zone::Graveyard);
            let opposing = put(&mut g, p.other(), "Bear Cub", Zone::Graveyard);
            let expensive = put(&mut g, p, "Craterhoof Behemoth", Zone::Graveyard);
            let spell = put(&mut g, p, "Rewrite Regrets", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 3];
            act(&mut g, Action::AnnounceCast { card: spell });
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(opposing),
                },
            );
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(expensive),
                },
            );
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
            pay(&mut g);
            if stale {
                g.move_object(target, Zone::Exile);
            }
            let before = g.snapshot();
            super::contracts::branch_contract(g.clone());
            settle(&mut g);
            assert_eq!(
                g.object_view(target).zone,
                if stale {
                    Zone::Exile
                } else {
                    Zone::Battlefield
                }
            );
            let jaces = fra_jaces(&g, p);
            if stale {
                assert!(jaces.is_empty());
            } else {
                assert_eq!(jaces.len(), 1);
                assert_eq!(g.counter_count(jaces[0], CounterKind::Loyalty), 2);
            }
            g.restore(&before).unwrap();
            settle(&mut g);
            assert_eq!(
                g.object_view(target).zone,
                if stale {
                    Zone::Exile
                } else {
                    Zone::Battlefield
                }
            );
        }
    }
}

#[test]
fn solve_for_disappointment_reveals_hand_discards_permanent_then_empowers() {
    for p in [Player::P0, Player::P1] {
        for has_permanent in [false, true] {
            let mut g = scenario(p);
            let land = put(&mut g, p.other(), "Forest", Zone::Hand);
            let instant = put(&mut g, p.other(), "Shock", Zone::Hand);
            let permanent = has_permanent.then(|| put(&mut g, p.other(), "Bear Cub", Zone::Hand));
            let spell = put(&mut g, p, "Solve for Disappointment", Zone::Hand);
            assert!(!g.observe(p).objects.iter().any(|o| o.id == land));
            g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 1];
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
            assert!(g.observe(p).objects.iter().any(|o| o.id == land));
            if let Some(permanent) = permanent {
                let land_ref = g.object_ref(land);
                let instant_ref = g.object_ref(instant);
                reject(&mut g, Action::ChooseRevealedDiscard { object: land_ref });
                reject(
                    &mut g,
                    Action::ChooseRevealedDiscard {
                        object: instant_ref,
                    },
                );
                super::contracts::branch_contract(g.clone());
                let chosen = g.object_ref(permanent);
                act(&mut g, Action::ChooseRevealedDiscard { object: chosen });
                settle(&mut g);
                assert_eq!(g.object_view(permanent).zone, Zone::Graveyard);
            }
            assert_eq!(g.object_view(land).zone, Zone::Hand);
            assert_eq!(g.object_view(instant).zone, Zone::Hand);
            let jaces = fra_jaces(&g, p);
            assert_eq!(jaces.len(), 1);
            assert_eq!(g.counter_count(jaces[0], CounterKind::Loyalty), 1);
        }
    }
}

#[test]
fn overwrite_counts_exiled_creatures_including_tokens_before_empower() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let land = put(&mut g, p, "Forest", Zone::Battlefield);
        g.create_tokens(p, Card::test_index(5298), 1, false);
        settle(&mut g);
        let token = g
            .state
            .objects
            .iter()
            .enumerate()
            .find_map(|(i, o)| {
                (o.zone == Zone::Battlefield && o.is_token() && o.card == Card::test_index(5298))
                    .then_some(i as CardId)
            })
            .unwrap();
        let spell = put(&mut g, p, "Overwrite the Multiverse", Zone::Hand);
        cast(&mut g, spell, None);
        let before = g.snapshot();
        super::contracts::branch_contract(g.clone());
        settle(&mut g);
        assert_eq!(g.object_view(own).zone, Zone::Exile);
        assert_eq!(g.object_view(enemy).zone, Zone::Exile);
        assert_ne!(g.object_view(token).zone, Zone::Battlefield);
        assert_eq!(g.object_view(land).zone, Zone::Battlefield);
        let jaces = fra_jaces(&g, p);
        assert_eq!(jaces.len(), 1);
        assert_eq!(g.counter_count(jaces[0], CounterKind::Loyalty), 3);
        g.restore(&before).unwrap();
        settle(&mut g);
        let jaces = fra_jaces(&g, p);
        assert_eq!(jaces.len(), 1);
        assert_eq!(g.counter_count(jaces[0], CounterKind::Loyalty), 3);
    }
}

#[test]
fn vigorbloom_charm_protects_permanents_and_counters_before_fight() {
    for p in [Player::P0, Player::P1] {
        for case in 0..5 {
            let mut g = scenario(p);
            let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            if case == 4 {
                g.state.objects[enemy as usize].gained_keywords = Keywords::INDESTRUCTIBLE;
            }
            let land = put(&mut g, p, "Forest", Zone::Battlefield);
            let spell = put(&mut g, p, "Vigorbloom Charm", Zone::Hand);
            g.state.players[p.index()].mana = [1, 0, 0, 0, 1, 0];
            act(&mut g, Action::AnnounceCast { card: spell });
            act(
                &mut g,
                Action::ChooseMode {
                    mode: if case == 0 { 0 } else { 2 },
                },
            );
            if case == 0 {
                act(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(land),
                    },
                );
            } else {
                act(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(own),
                    },
                );
                act(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(enemy),
                    },
                );
            }
            pay(&mut g);
            if case == 2 {
                g.move_object(enemy, Zone::Exile);
            }
            if case == 3 {
                g.move_object(own, Zone::Exile);
            }
            let before = g.snapshot();
            super::contracts::branch_contract(g.clone());
            settle(&mut g);
            if case == 0 {
                assert!(g.keywords(land).contains(Keywords::HEXPROOF));
                assert!(g.keywords(land).contains(Keywords::INDESTRUCTIBLE));
                g.finish_cleanup();
                assert!(!g.keywords(land).contains(Keywords::HEXPROOF));
            } else if case != 3 {
                assert_eq!(g.counter_count(own, CounterKind::PlusOne), 1);
                assert_eq!(
                    g.object_view(enemy).zone,
                    if case == 2 {
                        Zone::Exile
                    } else if case == 4 {
                        Zone::Battlefield
                    } else {
                        Zone::Graveyard
                    }
                );
                assert_eq!(g.object_view(own).zone, Zone::Battlefield);
                if case == 4 {
                    assert_eq!(g.counter_count(enemy, CounterKind::PlusOne), 0);
                }
            } else {
                assert_eq!(g.object_view(enemy).zone, Zone::Battlefield);
                assert_eq!(g.counter_count(enemy, CounterKind::PlusOne), 0);
            }
            g.restore(&before).unwrap();
            settle(&mut g);
            if case == 1 || case == 2 {
                assert_eq!(g.counter_count(own, CounterKind::PlusOne), 1);
            }
        }
    }
}

#[test]
fn vigorbloom_charm_draw_mode_gains_life() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let card = put(&mut g, p, "Forest", Zone::Library);
        let spell = put(&mut g, p, "Vigorbloom Charm", Zone::Hand);
        let life = g.state.players[p.index()].life;
        g.state.players[p.index()].mana = [1, 0, 0, 0, 1, 0];
        act(&mut g, Action::AnnounceCast { card: spell });
        act(&mut g, Action::ChooseMode { mode: 1 });
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, life + 3);
        assert_eq!(g.object_view(card).zone, Zone::Hand);
    }
}

#[test]
fn edgar_moonlit_sovereign_rechecks_end_step_and_counters_only_eligible_creatures() {
    for p in [Player::P0, Player::P1] {
        for cast_after_trigger in [false, true] {
            let mut g = scenario(p);
            let edgar = put(&mut g, p, "Edgar, Moonlit Sovereign", Zone::Battlefield);
            let marked = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let unmarked = put(&mut g, p, "Hill Giant", Zone::Battlefield);
            let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            g.add_counter(marked, CounterKind::PlusOne, 1);
            g.add_counter(enemy, CounterKind::PlusOne, 1);
            g.begin_step(Step::End);
            g.prepare_triggers();
            if cast_after_trigger {
                g.state.players[p.index()].spells_cast_this_turn = 1;
            }
            let before = g.snapshot();
            super::contracts::branch_contract(g.clone());
            settle(&mut g);
            assert_eq!(
                g.counter_count(edgar, CounterKind::PlusOne),
                if cast_after_trigger { 0 } else { 2 }
            );
            g.restore(&before).unwrap();
            settle(&mut g);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 4];
            act(
                &mut g,
                Action::AnnounceActivate {
                    card: edgar,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            pay(&mut g);
            settle(&mut g);
            assert_eq!(g.counter_count(marked, CounterKind::PlusOne), 2);
            assert_eq!(g.counter_count(unmarked, CounterKind::PlusOne), 0);
            assert_eq!(g.counter_count(enemy, CounterKind::PlusOne), 1);
            assert_eq!(
                g.counter_count(edgar, CounterKind::PlusOne),
                if cast_after_trigger { 0 } else { 3 }
            );
        }
    }
}

#[test]
fn vraska_soul_of_stone_creates_sculpture_treasure_with_vigilance() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let vraska = put(&mut g, p, "Vraska, Soul of Stone", Zone::Battlefield);
        let own_bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let target = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let shock = put(&mut g, p, "Shock", Zone::Hand);
        g.state.players[p.index()].mana[3] = 1;
        cast(&mut g, shock, Some(Target::Card(target)));
        super::contracts::branch_contract(g.clone());
        settle(&mut g);
        let tokens: Vec<_> = g
            .state
            .objects
            .iter()
            .enumerate()
            .filter_map(|(i, o)| {
                (o.zone == Zone::Battlefield
                    && o.controller == p
                    && o.is_token()
                    && o.card == Card::test_index(5437))
                .then_some(i as CardId)
            })
            .collect();
        assert_eq!(tokens.len(), 1);
        let token = tokens[0];
        assert_eq!(stats(&g, token), (1, 1));
        assert!(g.has_subtype(token, "Sculpture"));
        assert!(g.has_subtype(token, "Treasure"));
        assert!(g.keywords(token).contains(Keywords::VIGILANCE));
        assert!(!g.keywords(own_bear).contains(Keywords::VIGILANCE));
        assert!(!g.legal_actions().iter().any(|a| matches!(a,
            Action::ActivateMana { card, .. } if *card == token)));
        g.state.objects[token as usize].summoning_sick = false;
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(a,
            Action::ActivateMana { card, production: [0, 0, 0, 0, 1, 0], .. } if *card == token)
            })
            .expect("Sculpture Treasure mana ability");
        let before = g.state.players[p.index()].mana[4];
        act(&mut g, action);
        assert_ne!(g.object_view(token).zone, Zone::Battlefield);
        assert_eq!(g.state.players[p.index()].mana[4], before + 1);
        g.move_object(vraska, Zone::Graveyard);
        assert!(!g.keywords(own_bear).contains(Keywords::VIGILANCE));
    }
}

#[test]
fn mind_meanderer_jace_vigilance_and_optional_fight_use_live_objects() {
    for p in [Player::P0, Player::P1] {
        for source_leaves in [false, true] {
            let mut g = scenario(p);
            let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let source = put(&mut g, p, "Mind Meanderer", Zone::Hand);
            cast(&mut g, source, None);
            let mut targeted = false;
            for _ in 0..48 {
                g.prepare_triggers();
                let actions = g.legal_actions();
                let choice = Action::ChooseTarget {
                    target: Target::Card(enemy),
                };
                if !targeted && actions.contains(&choice) {
                    act(&mut g, choice);
                    targeted = true;
                } else if actions.contains(&Action::FinishTargets) {
                    act(&mut g, Action::FinishTargets);
                    break;
                } else if targeted && g.state.pending == Pending::Priority {
                    break;
                } else if let Some(action) = actions
                    .into_iter()
                    .find(|a| matches!(a, Action::Pass | Action::PutTrigger { .. }))
                {
                    act(&mut g, action);
                } else {
                    panic!("unexpected Mind Meanderer choice {:?}", g.state.pending);
                }
            }
            assert!(targeted);
            if source_leaves {
                g.move_object(source, Zone::Exile);
            }
            super::contracts::branch_contract(g.clone());
            settle(&mut g);
            assert_eq!(
                g.object_view(enemy).zone,
                if source_leaves {
                    Zone::Battlefield
                } else {
                    Zone::Graveyard
                }
            );
            if !source_leaves {
                assert!(g.keywords(source).contains(Keywords::FLYING));
                assert!(!g.keywords(source).contains(Keywords::VIGILANCE));
                g.create_tokens(p, Card::test_index(5435), 1, false);
                let jace = fra_jaces(&g, p)[0];
                g.add_counter(jace, CounterKind::Loyalty, 1);
                assert!(g.keywords(source).contains(Keywords::VIGILANCE));
                g.move_object(jace, Zone::Exile);
                assert!(!g.keywords(source).contains(Keywords::VIGILANCE));
            }
        }
    }
}

#[test]
fn gallia_grants_haste_live_and_targets_recent_creatures_of_either_player() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let gallia = put(&mut g, p, "Gallia, the Merrymaker", Zone::Battlefield);
        let ally = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let old = put(&mut g, p, "Hill Giant", Zone::Battlefield);
        g.state.entered_this_turn = None;
        let incoming = put(&mut g, p.other(), "Bear Cub", Zone::Hand);
        g.move_object(incoming, Zone::Battlefield);
        assert!(g.keywords(gallia).contains(Keywords::HASTE));
        assert!(!g.keywords(ally).contains(Keywords::HASTE));
        g.add_counter(ally, CounterKind::PlusOne, 1);
        assert!(g.keywords(ally).contains(Keywords::HASTE));
        g.pay_counters(ally, &Counters::of(CounterKind::PlusOne, 1));
        assert!(!g.keywords(ally).contains(Keywords::HASTE));
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 1];
        act(
            &mut g,
            Action::AnnounceActivate {
                card: gallia,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(old),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(incoming),
            },
        );
        pay(&mut g);
        let before = g.snapshot();
        super::contracts::branch_contract(g.clone());
        settle(&mut g);
        assert_eq!(g.counter_count(incoming, CounterKind::PlusOne), 1);
        assert!(!g.keywords(incoming).contains(Keywords::HASTE));
        g.restore(&before).unwrap();
        settle(&mut g);
        assert_eq!(g.counter_count(incoming, CounterKind::PlusOne), 1);
    }
}

#[test]
fn hapatra_desert_fang_uses_greatest_graveyard_mana_value_at_resolution() {
    for p in [Player::P0, Player::P1] {
        for add_expensive in [false, true] {
            let mut g = scenario(p);
            let enemy = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            put(&mut g, p, "Bear Cub", Zone::Graveyard);
            let hapatra = put(&mut g, p, "Hapatra, the Desert Fang", Zone::Hand);
            cast(&mut g, hapatra, None);
            let mut targeted = false;
            for _ in 0..48 {
                g.prepare_triggers();
                let actions = g.legal_actions();
                let choice = Action::ChooseTarget {
                    target: Target::Card(enemy),
                };
                if !targeted && actions.contains(&choice) {
                    reject(
                        &mut g,
                        Action::ChooseTarget {
                            target: Target::Card(own),
                        },
                    );
                    act(&mut g, choice);
                    targeted = true;
                } else if actions.contains(&Action::FinishTargets) {
                    act(&mut g, Action::FinishTargets);
                    break;
                } else if targeted && g.state.pending == Pending::Priority {
                    break;
                } else if let Some(action) = actions
                    .into_iter()
                    .find(|a| matches!(a, Action::Pass | Action::PutTrigger { .. }))
                {
                    act(&mut g, action);
                } else {
                    panic!("unexpected Hapatra choice {:?}", g.state.pending);
                }
            }
            assert!(targeted);
            if add_expensive {
                put(&mut g, p, "Craterhoof Behemoth", Zone::Graveyard);
            }
            let before = g.snapshot();
            super::contracts::branch_contract(g.clone());
            settle(&mut g);
            if add_expensive {
                assert_eq!(g.object_view(enemy).zone, Zone::Graveyard);
            } else {
                assert_eq!(g.counter_count(enemy, CounterKind::MinusOne), 2);
                assert_eq!(g.object_view(enemy).zone, Zone::Battlefield);
            }
            g.restore(&before).unwrap();
            settle(&mut g);
            assert_eq!(
                g.object_view(enemy).zone,
                if add_expensive {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
        }
    }
}

#[test]
fn hapatra_desert_frost_stuns_even_tapped_creatures_and_untap_removes_stun() {
    for p in [Player::P0, Player::P1] {
        for already_tapped in [false, true] {
            let mut g = scenario(p);
            let target = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            g.state.objects[target as usize].tapped = already_tapped;
            let source = put(&mut g, p, "Hapatra, the Desert Frost", Zone::Hand);
            cast(&mut g, source, None);
            let mut targeted = false;
            for _ in 0..48 {
                g.prepare_triggers();
                let actions = g.legal_actions();
                let choice = Action::ChooseTarget {
                    target: Target::Card(target),
                };
                if !targeted && actions.contains(&choice) {
                    act(&mut g, choice);
                    targeted = true;
                } else if actions.contains(&Action::FinishTargets) {
                    act(&mut g, Action::FinishTargets);
                    break;
                } else if targeted && g.state.pending == Pending::Priority {
                    break;
                } else if let Some(action) = actions
                    .into_iter()
                    .find(|a| matches!(a, Action::Pass | Action::PutTrigger { .. }))
                {
                    act(&mut g, action);
                } else {
                    panic!("unexpected Hapatra Frost choice {:?}", g.state.pending);
                }
            }
            assert!(targeted);
            super::contracts::branch_contract(g.clone());
            settle(&mut g);
            assert!(g.state.objects[target as usize].tapped);
            assert_eq!(g.counter_count(target, CounterKind::Stun), 1);
            for expected_tapped in [true, false] {
                g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 2];
                act(
                    &mut g,
                    Action::AnnounceActivate {
                        card: source,
                        ability: 0,
                        counter_payment: Counters::default(),
                    },
                );
                act(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(target),
                    },
                );
                pay(&mut g);
                settle(&mut g);
                assert_eq!(g.state.objects[target as usize].tapped, expected_tapped);
                assert_eq!(g.counter_count(target, CounterKind::Stun), 0);
            }
        }
    }
}

#[test]
fn infinite_coursework_unprepares_blanks_and_restricts_only_untap_step() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let creature = put(&mut g, p, "Pigment Wrangler", Zone::Hand);
        cast(&mut g, creature, None);
        settle(&mut g);
        assert!(g.is_prepared(g.object_ref(creature)));
        assert!(g.keywords(creature).contains(Keywords::FLYING));
        let aura = put(&mut g, p, "Infinite Coursework", Zone::Hand);
        cast(&mut g, aura, Some(Target::Card(creature)));
        super::contracts::branch_contract(g.clone());
        settle(&mut g);
        assert!(!g.is_prepared(g.object_ref(creature)));
        assert!(!g.keywords(creature).contains(Keywords::FLYING));
        assert!(g.state.objects[creature as usize].tapped);
        g.untap_objects(&[creature], untap_rules::UntapCause::ControllerStep(p));
        assert!(g.state.objects[creature as usize].tapped);
        g.untap_objects(&[creature], untap_rules::UntapCause::Effect);
        assert!(!g.state.objects[creature as usize].tapped);
        g.move_object(aura, Zone::Graveyard);
        assert!(g.keywords(creature).contains(Keywords::FLYING));
        assert!(!g.is_prepared(g.object_ref(creature)));
    }
}

#[test]
fn yoshimaru_scrappy_stray_fights_and_targets_nonlegendary_creatures() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let own = put(&mut g, p, "Hill Giant", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let dog = put(&mut g, p, "Yoshimaru, Scrappy Stray", Zone::Hand);
        cast(&mut g, dog, None);
        let mut picked_own = false;
        let mut picked_enemy = false;
        for _ in 0..64 {
            g.prepare_triggers();
            let actions = g.legal_actions();
            let own_choice = Action::ChooseTarget {
                target: Target::Card(own),
            };
            let enemy_choice = Action::ChooseTarget {
                target: Target::Card(enemy),
            };
            if !picked_own && actions.contains(&own_choice) {
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(dog),
                    },
                );
                act(&mut g, own_choice);
                picked_own = true;
            } else if picked_own && !picked_enemy && actions.contains(&enemy_choice) {
                act(&mut g, enemy_choice);
                picked_enemy = true;
            } else if actions.contains(&Action::FinishTargets) {
                act(&mut g, Action::FinishTargets);
            } else if picked_enemy
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
                panic!("unexpected Yoshimaru choice {:?}", g.state.pending);
            }
        }
        assert!(picked_own && picked_enemy);
        assert_eq!(g.object_view(enemy).zone, Zone::Graveyard);
        assert_eq!(g.object_view(own).zone, Zone::Battlefield);
        g.state.players[p.index()].mana[5] = 6;
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
                target: Target::Card(own),
            },
        );
        pay(&mut g);
        let before = g.snapshot();
        super::contracts::branch_contract(g.clone());
        settle(&mut g);
        assert_eq!(g.counter_count(own, CounterKind::PlusOne), 1);
        g.restore(&before).unwrap();
        settle(&mut g);
        assert_eq!(g.counter_count(own, CounterKind::PlusOne), 1);
    }
}

#[test]
fn edgar_ancient_bloodlord_sacrifice_cost_triggers_life_and_grants_menace() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let edgar = put(&mut g, p, "Edgar, Ancient Bloodlord", Zone::Battlefield);
        let ally = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let life = g.state.players[p.index()].life;
        g.state.players[p.index()].mana[5] = 2;
        act(
            &mut g,
            Action::AnnounceActivate {
                card: edgar,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        let self_ref = g.object_ref(edgar);
        let enemy_ref = g.object_ref(enemy);
        let ally_ref = g.object_ref(ally);
        reject(&mut g, Action::ChooseAdditionalCost { object: self_ref });
        reject(&mut g, Action::ChooseAdditionalCost { object: enemy_ref });
        act(&mut g, Action::ChooseAdditionalCost { object: ally_ref });
        pay(&mut g);
        assert_eq!(g.object_view(ally).zone, Zone::Graveyard);
        let before = g.snapshot();
        super::contracts::branch_contract(g.clone());
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, life + 1);
        assert_eq!(g.counter_count(edgar, CounterKind::PlusOne), 1);
        assert!(g.keywords(edgar).contains(Keywords::MENACE));
        g.finish_cleanup();
        assert!(!g.keywords(edgar).contains(Keywords::MENACE));
        assert_eq!(g.counter_count(edgar, CounterKind::PlusOne), 1);
        g.restore(&before).unwrap();
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, life + 1);
        assert_eq!(g.counter_count(edgar, CounterKind::PlusOne), 1);
    }
}

#[test]
fn ferocity_returns_the_exact_dying_card_tapped_under_its_owner() {
    for p in [Player::P0, Player::P1] {
        for moved_away in [false, true] {
            let mut g = scenario(p);
            let creature = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let creature_ref = g.object_ref(creature);
            g.change_controller(creature_ref, p);
            let aura = put(&mut g, p, "Ferocity of the Hunt", Zone::Hand);
            cast(&mut g, aura, Some(Target::Card(creature)));
            settle(&mut g);
            assert_eq!(stats(&g, creature), (3, 2));
            assert!(g.keywords(creature).contains(Keywords::DEATHTOUCH));
            g.move_object(creature, Zone::Graveyard);
            g.prepare_triggers();
            if moved_away {
                g.move_object(creature, Zone::Exile);
                g.move_object(creature, Zone::Graveyard);
            }
            while g
                .legal_actions()
                .iter()
                .any(|a| matches!(a, Action::PutTrigger { .. }))
            {
                choose(&mut g, |a| matches!(a, Action::PutTrigger { .. }));
            }
            let before = g.snapshot();
            super::contracts::branch_contract(g.clone());
            settle(&mut g);
            if moved_away {
                assert_eq!(g.object_view(creature).zone, Zone::Graveyard);
            } else {
                assert_eq!(g.object_view(creature).zone, Zone::Battlefield);
                assert!(g.object_view(creature).tapped);
                assert_eq!(g.object_view(creature).controller, p.other());
            }
            g.restore(&before).unwrap();
            settle(&mut g);
            assert_eq!(
                g.object_view(creature).zone,
                if moved_away {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
        }
    }
}

#[test]
fn ferocity_flash_hybrid_payment_and_simultaneous_death() {
    for p in [Player::P0, Player::P1] {
        for color in [2, 4] {
            let mut g = scenario(p);
            g.state.active = p.other();
            g.state.step = Step::Attackers;
            let creature = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let aura = put(&mut g, p, "Ferocity of the Hunt", Zone::Hand);
            g.state.players[p.index()].mana = [0; 6];
            g.state.players[p.index()].mana[color] = 1;
            g.state.players[p.index()].mana[5] = 1;
            cast(&mut g, aura, Some(Target::Card(creature)));
            settle(&mut g);
            assert_eq!(stats(&g, creature), (3, 2));
            let observers = g.death_trigger_sources();
            let events = g.move_objects_raw(&[creature, aura], Zone::Graveyard);
            g.finish_zone_events(&events, &observers);
            g.prepare_triggers();
            super::contracts::branch_contract(g.clone());
            settle(&mut g);
            assert_eq!(g.object_view(aura).zone, Zone::Graveyard);
            assert_eq!(g.object_view(creature).zone, Zone::Battlefield);
            assert!(g.object_view(creature).tapped);
            assert_eq!(g.object_view(creature).controller, p.other());
        }
    }
}

#[test]
fn ferocity_does_not_return_a_dead_token() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        g.create_tokens(p, Card::test_index(5298), 1, false);
        settle(&mut g);
        let token = g
            .state
            .objects
            .iter()
            .enumerate()
            .find_map(|(i, object)| {
                (object.zone == Zone::Battlefield
                    && object.is_token()
                    && object.card == Card::test_index(5298))
                .then_some(i as CardId)
            })
            .unwrap();
        let aura = put(&mut g, p, "Ferocity of the Hunt", Zone::Hand);
        cast(&mut g, aura, Some(Target::Card(token)));
        settle(&mut g);
        g.move_object(token, Zone::Graveyard);
        g.prepare_triggers();
        super::contracts::branch_contract(g.clone());
        settle(&mut g);
        assert_ne!(g.object_view(token).zone, Zone::Battlefield);
    }
}

#[test]
fn loyalty_cost_uses_non_effect_counter_replacements() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let planeswalker = put(&mut g, p, "Liliana, Dreadhorde General", Zone::Battlefield);
        g.add_counter(planeswalker, CounterKind::Loyalty, 6);
        put(
            &mut g,
            p,
            "Doc Samson, Super Psychiatrist",
            Zone::Battlefield,
        );
        put(&mut g, p, "Doubling Season", Zone::Battlefield);
        act(
            &mut g,
            Action::AnnounceActivate {
                card: planeswalker,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        // Samson replaces the cost placement; that replacement makes it an effect,
        // so Doubling Season then doubles its two counters (CR 614.16).
        assert_eq!(
            g.counter_count(planeswalker, CounterKind::Loyalty),
            10,
            "static sources {:?}, pending {:?}",
            g.state
                .static_sources
                .iter()
                .map(|id| g.object_view(*id).name().to_string())
                .collect::<Vec<_>>(),
            g.state.pending
        );
        let before = g.snapshot();
        super::contracts::branch_contract(g.clone());
        settle(&mut g);
        assert_eq!(g.counter_count(planeswalker, CounterKind::Loyalty), 10);
        g.restore(&before).unwrap();
        settle(&mut g);
        assert_eq!(g.counter_count(planeswalker, CounterKind::Loyalty), 10);
    }
}

#[test]
fn activation_counter_cost_preserves_replacement_order_and_restore() {
    for p in [Player::P0, Player::P1] {
        for samson_first in [false, true] {
            let mut g = scenario(p);
            let jessica = put(&mut g, p, "Jessica Jones, Private Eye", Zone::Battlefield);
            let samson = put(
                &mut g,
                p,
                "Doc Samson, Super Psychiatrist",
                Zone::Battlefield,
            );
            let loading = put(&mut g, p, "Loading Zone", Zone::Battlefield);
            act(
                &mut g,
                Action::AnnounceActivate {
                    card: jessica,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            pay(&mut g);
            assert!(matches!(g.state.pending, Pending::Replacement { .. }));
            assert_eq!(g.actor(), Some(p));
            let chosen = if samson_first { samson } else { loading };
            let chosen_ref = g.object_ref(chosen);
            let before = g.snapshot();
            super::contracts::branch_contract(g.clone());
            let pick = Action::ChooseReplacement {
                source: chosen_ref,
                ability: 0,
            };
            act(&mut g, pick.clone());
            while g
                .legal_actions()
                .iter()
                .any(|a| matches!(a, Action::ChooseReplacement { .. }))
            {
                choose(&mut g, |a| matches!(a, Action::ChooseReplacement { .. }));
            }
            assert_eq!(
                g.counter_count(jessica, CounterKind::Stun),
                if samson_first { 4 } else { 3 }
            );
            assert!(g.object_view(jessica).tapped);
            let completed = g.snapshot();
            g.restore(&before).unwrap();
            act(&mut g, pick);
            while g
                .legal_actions()
                .iter()
                .any(|a| matches!(a, Action::ChooseReplacement { .. }))
            {
                choose(&mut g, |a| matches!(a, Action::ChooseReplacement { .. }));
            }
            assert_eq!(g.snapshot(), completed);
        }
    }
}

#[test]
fn generous_revival_targets_mana_value_and_flashback_entry_counters() {
    for p in [Player::P0, Player::P1] {
        for flashback in [false, true] {
            for fizzle in [false, true] {
                let mut g = scenario(p);
                let creature = put(&mut g, p, "Bear Cub", Zone::Graveyard);
                let too_large = put(&mut g, p, "Hill Giant", Zone::Graveyard);
                let opponent = put(&mut g, p.other(), "Bear Cub", Zone::Graveyard);
                let land = put(&mut g, p, "Forest", Zone::Graveyard);
                put(&mut g, p, "Doubling Season", Zone::Battlefield);
                let spell = put(
                    &mut g,
                    p,
                    "Generous Revival",
                    if flashback {
                        Zone::Graveyard
                    } else {
                        Zone::Hand
                    },
                );
                act(
                    &mut g,
                    if flashback {
                        Action::AnnounceFlashback { card: spell }
                    } else {
                        Action::AnnounceCast { card: spell }
                    },
                );
                for illegal in [too_large, opponent, land] {
                    reject(
                        &mut g,
                        Action::ChooseTarget {
                            target: Target::Card(illegal),
                        },
                    );
                }
                act(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(creature),
                    },
                );
                pay(&mut g);
                if fizzle {
                    g.move_object(creature, Zone::Exile);
                }
                let before = g.snapshot();
                super::contracts::branch_contract(g.clone());
                settle(&mut g);
                assert_eq!(
                    g.object_view(spell).zone,
                    if flashback {
                        Zone::Exile
                    } else {
                        Zone::Graveyard
                    }
                );
                assert_eq!(
                    g.object_view(creature).zone,
                    if fizzle {
                        Zone::Exile
                    } else {
                        Zone::Battlefield
                    }
                );
                if !fizzle {
                    assert_eq!(g.counter_count(creature, CounterKind::PlusOne), 2);
                }
                g.restore(&before).unwrap();
                settle(&mut g);
                assert_eq!(
                    g.object_view(creature).zone,
                    if fizzle {
                        Zone::Exile
                    } else {
                        Zone::Battlefield
                    }
                );
            }
        }
    }
}

#[test]
fn generous_revival_adds_to_intrinsic_entry_counters_before_replacements() {
    for p in [Player::P0, Player::P1] {
        for doubled in [false, true] {
            let mut g = scenario(p);
            let hydra = put(&mut g, p, "Mossborn Hydra", Zone::Graveyard);
            if doubled {
                put(&mut g, p, "Doubling Season", Zone::Battlefield);
            }
            let revival = put(&mut g, p, "Generous Revival", Zone::Hand);
            cast(&mut g, revival, Some(Target::Card(hydra)));
            let before = g.snapshot();
            super::contracts::branch_contract(g.clone());
            settle(&mut g);
            assert_eq!(g.object_view(hydra).zone, Zone::Battlefield);
            assert_eq!(
                g.counter_count(hydra, CounterKind::PlusOne),
                if doubled { 4 } else { 2 }
            );
            assert_eq!(stats(&g, hydra), if doubled { (4, 4) } else { (2, 2) });
            g.restore(&before).unwrap();
            settle(&mut g);
            assert_eq!(
                g.counter_count(hydra, CounterKind::PlusOne),
                if doubled { 4 } else { 2 }
            );
        }
    }
}

#[test]
fn recursive_recruitment_counts_own_graveyard_after_flashback_and_creation() {
    for p in [Player::P0, Player::P1] {
        for flashback in [false, true] {
            for doubled in [false, true] {
                for cards in [0, 2, 3, 6] {
                    let mut g = scenario(p);
                    for _ in 0..cards {
                        put(&mut g, p, "Forest", Zone::Graveyard);
                    }
                    for _ in 0..3 {
                        put(&mut g, p.other(), "Forest", Zone::Graveyard);
                    }
                    if doubled {
                        put(&mut g, p, "Doubling Season", Zone::Battlefield);
                    }
                    let spell = put(
                        &mut g,
                        p,
                        "Recursive Recruitment",
                        if flashback {
                            Zone::Graveyard
                        } else {
                            Zone::Hand
                        },
                    );
                    if flashback {
                        act(&mut g, Action::AnnounceFlashback { card: spell });
                        pay(&mut g);
                    } else {
                        cast(&mut g, spell, None);
                    }
                    let before = g.snapshot();
                    super::contracts::branch_contract(g.clone());
                    settle(&mut g);
                    let cadets: Vec<_> = g
                        .state
                        .objects
                        .iter()
                        .enumerate()
                        .filter_map(|(i, o)| {
                            (o.zone == Zone::Battlefield
                                && o.is_token()
                                && o.controller == p
                                && o.card == Card::test_index(5298))
                            .then_some(i as CardId)
                        })
                        .collect();
                    assert_eq!(cadets.len(), if doubled { 4 } else { 2 });
                    for token in cadets {
                        assert_eq!(
                            g.counter_count(token, CounterKind::PlusOne),
                            if flashback {
                                cards / 3 * if doubled { 2 } else { 1 }
                            } else {
                                0
                            }
                        );
                    }
                    assert_eq!(
                        g.object_view(spell).zone,
                        if flashback {
                            Zone::Exile
                        } else {
                            Zone::Graveyard
                        }
                    );
                    let completed = g.snapshot();
                    g.restore(&before).unwrap();
                    settle(&mut g);
                    assert_eq!(g.snapshot(), completed);
                }
            }
        }
    }
}

#[test]
fn recursive_recruitment_counters_all_replacement_created_tokens() {
    for p in [Player::P0, Player::P1] {
        for quina_first in [false, true] {
            let mut g = scenario(p);
            for _ in 0..3 {
                put(&mut g, p, "Forest", Zone::Graveyard);
            }
            let quina = put(&mut g, p, "Quina, Qu Gourmet", Zone::Battlefield);
            let doubling = put(&mut g, p, "Doubling Season", Zone::Battlefield);
            let spell = put(&mut g, p, "Recursive Recruitment", Zone::Graveyard);
            act(&mut g, Action::AnnounceFlashback { card: spell });
            pay(&mut g);
            for _ in 0..20 {
                if matches!(g.state.pending, Pending::Replacement { .. }) {
                    break;
                }
                act(&mut g, Action::Pass);
            }
            assert!(matches!(g.state.pending, Pending::Replacement { .. }));
            let first = g.object_ref(if quina_first { quina } else { doubling });
            let choice = Action::ChooseReplacement {
                source: first,
                ability: 0,
            };
            let before = g.snapshot();
            super::contracts::branch_contract(g.clone());
            act(&mut g, choice.clone());
            while g
                .legal_actions()
                .iter()
                .any(|a| matches!(a, Action::ChooseReplacement { .. }))
            {
                choose(&mut g, |a| matches!(a, Action::ChooseReplacement { .. }));
            }
            settle(&mut g);
            let tokens: Vec<_> = g
                .state
                .objects
                .iter()
                .enumerate()
                .filter_map(|(i, o)| {
                    (o.zone == Zone::Battlefield
                        && o.is_token()
                        && o.controller == p
                        && (o.card == Card::test_index(5298) || o.card == Card::test_index(5264)))
                    .then_some(i as CardId)
                })
                .collect();
            assert_eq!(tokens.len(), if quina_first { 6 } else { 5 });
            for token in tokens {
                assert_eq!(g.counter_count(token, CounterKind::PlusOne), 2);
            }
            let completed = g.snapshot();
            g.restore(&before).unwrap();
            act(&mut g, choice);
            while g
                .legal_actions()
                .iter()
                .any(|a| matches!(a, Action::ChooseReplacement { .. }))
            {
                choose(&mut g, |a| matches!(a, Action::ChooseReplacement { .. }));
            }
            settle(&mut g);
            assert_eq!(g.snapshot(), completed);
        }
    }
}

#[test]
fn silence_the_echo_sacrifice_or_mana_and_target_legality() {
    for p in [Player::P0, Player::P1] {
        for route in [0, 1] {
            for sacrifice_walker in [false, true] {
                for fizzle in [false, true] {
                    let mut g = scenario(p);
                    let creature = put(&mut g, p, "Bear Cub", Zone::Battlefield);
                    let walker = put(&mut g, p, "Liliana, Dreadhorde General", Zone::Battlefield);
                    g.add_counter(walker, CounterKind::Loyalty, 6);
                    let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
                    let land = put(&mut g, p, "Forest", Zone::Battlefield);
                    let spell = put(&mut g, p, "Silence the Echo", Zone::Hand);
                    g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 4];
                    act(&mut g, Action::AnnounceCast { card: spell });
                    act(&mut g, Action::ChooseSpellCost { cost: route });
                    reject(
                        &mut g,
                        Action::ChooseTarget {
                            target: Target::Card(land),
                        },
                    );
                    act(
                        &mut g,
                        Action::ChooseTarget {
                            target: Target::Card(enemy),
                        },
                    );
                    let chosen = if sacrifice_walker { walker } else { creature };
                    if route == 0 {
                        let land_ref = g.object_ref(land);
                        let enemy_ref = g.object_ref(enemy);
                        reject(&mut g, Action::ChooseAdditionalCost { object: land_ref });
                        reject(&mut g, Action::ChooseAdditionalCost { object: enemy_ref });
                        let chosen_ref = g.object_ref(chosen);
                        act(&mut g, Action::ChooseAdditionalCost { object: chosen_ref });
                    }
                    pay(&mut g);
                    assert_eq!(
                        g.object_view(chosen).zone,
                        if route == 0 {
                            Zone::Graveyard
                        } else {
                            Zone::Battlefield
                        }
                    );
                    if fizzle {
                        g.move_object(enemy, Zone::Exile);
                    }
                    let before = g.snapshot();
                    super::contracts::branch_contract(g.clone());
                    settle(&mut g);
                    assert_eq!(
                        g.object_view(enemy).zone,
                        if fizzle { Zone::Exile } else { Zone::Graveyard }
                    );
                    g.restore(&before).unwrap();
                    settle(&mut g);
                    assert_eq!(
                        g.object_view(enemy).zone,
                        if fizzle { Zone::Exile } else { Zone::Graveyard }
                    );
                }
            }
        }
    }
}

#[test]
fn warriors_blades_equip_cost_uses_chosen_creature_counters() {
    for p in [Player::P0, Player::P1] {
        for counters in 0..=4u64 {
            for fizzle in [false, true] {
                let mut g = scenario(p);
                let equipment = put(&mut g, p, "Warrior's Blades", Zone::Battlefield);
                let target = put(&mut g, p, "Bear Cub", Zone::Battlefield);
                let poor = put(&mut g, p, "Bear Cub", Zone::Battlefield);
                let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
                g.add_counter(target, CounterKind::PlusOne, counters);
                let cost = 3u32.saturating_sub(u32::try_from(counters).unwrap());
                g.state.players[p.index()].mana = [0, 0, 0, 0, 0, cost];
                act(
                    &mut g,
                    Action::AnnounceActivate {
                        card: equipment,
                        ability: 0,
                        counter_payment: Counters::default(),
                    },
                );
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(enemy),
                    },
                );
                if counters > 0 {
                    reject(
                        &mut g,
                        Action::ChooseTarget {
                            target: Target::Card(poor),
                        },
                    );
                }
                act(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(target),
                    },
                );
                pay(&mut g);
                assert_eq!(g.state.players[p.index()].mana[5], 0);
                if fizzle {
                    g.move_object(target, Zone::Exile);
                }
                let before = g.snapshot();
                super::contracts::branch_contract(g.clone());
                settle(&mut g);
                if fizzle {
                    assert_eq!(g.object_view(equipment).attached_to, None);
                } else {
                    assert_eq!(
                        g.object_view(equipment).attached_to,
                        Some(g.object_ref(target))
                    );
                    assert_eq!(
                        stats(&g, target),
                        (4 + i128::from(counters), 3 + i128::from(counters))
                    );
                }
                g.restore(&before).unwrap();
                settle(&mut g);
                assert_eq!(g.object_view(equipment).attached_to.is_some(), !fizzle);
            }
        }
    }
}

#[test]
fn warriors_blades_entry_damage_life_and_target_fizzle() {
    for p in [Player::P0, Player::P1] {
        for fizzle in [false, true] {
            let mut g = scenario(p);
            let target = put(&mut g, p.other(), "Shivan Dragon", Zone::Battlefield);
            let equipment = put(&mut g, p, "Warrior's Blades", Zone::Hand);
            let life = g.state.players[p.index()].life;
            cast(&mut g, equipment, None);
            let wanted = Action::ChooseTarget {
                target: Target::Card(target),
            };
            let mut selected = false;
            for _ in 0..48 {
                g.prepare_triggers();
                if g.legal_actions().contains(&wanted) {
                    act(&mut g, wanted.clone());
                    selected = true;
                    break;
                }
                choose(&mut g, |a| {
                    matches!(a, Action::Pass | Action::PutTrigger { .. })
                });
            }
            assert!(selected);
            g.move_object(equipment, Zone::Graveyard);
            if fizzle {
                g.move_object(target, Zone::Exile);
            }
            let before = g.snapshot();
            super::contracts::branch_contract(g.clone());
            settle(&mut g);
            assert_eq!(
                g.state.players[p.index()].life,
                life + if fizzle { 0 } else { 3 }
            );
            if !fizzle {
                assert_eq!(g.state.objects[target as usize].damage, 3);
            }
            let completed = g.snapshot();
            g.restore(&before).unwrap();
            settle(&mut g);
            assert_eq!(g.snapshot(), completed);
        }
    }
}

#[test]
fn warriors_blades_direct_equip_action_uses_target_counter_cost() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let equipment = put(&mut g, p, "Warrior's Blades", Zone::Battlefield);
        let cheap = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let expensive = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.add_counter(cheap, CounterKind::PlusOne, 2);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        let actions = g.legal_actions();
        assert!(!actions.iter().any(|a| matches!(a, Action::Activate { card, ability: 0, target: Some(Target::Card(id)), .. } if *card == equipment && *id == expensive)));
        let action = actions.into_iter().find(|a| matches!(a, Action::Activate { card, ability: 0, target: Some(Target::Card(id)), payment, .. } if *card == equipment && *id == cheap && payment[5] == 1)).expect("discounted direct equip");
        act(&mut g, action);
        super::contracts::branch_contract(g.clone());
        settle(&mut g);
        assert_eq!(
            g.object_view(equipment).attached_to,
            Some(g.object_ref(cheap))
        );
        assert_eq!(g.state.players[p.index()].mana[5], 0);
    }
}

#[test]
fn room_of_refuge_color_mana_sacrifice_and_counter_replacement() {
    for p in [Player::P0, Player::P1] {
        for color in ChoiceColor::ALL {
            for fizzle in [false, true] {
                let mut g = scenario(p);
                let land = put(&mut g, p, "Room of Refuge", Zone::Hand);
                let target = put(&mut g, p, "Bear Cub", Zone::Battlefield);
                put(&mut g, p, "Doubling Season", Zone::Battlefield);
                act(&mut g, Action::PlayLand { card: land });
                let before_entry = g.snapshot();
                super::contracts::branch_contract(g.clone());
                act(
                    &mut g,
                    Action::ChooseEntryValue {
                        value: EntryChoiceValue::Color(color),
                    },
                );
                assert!(g.object_view(land).tapped);
                g.untap_objects(&[land], untap_rules::UntapCause::Effect);
                g.state.players[p.index()].mana = [0; 6];
                let action = g.legal_actions().into_iter().find(|a| matches!(a, Action::ActivateMana { card, ability: 0, .. } if *card == land)).expect("chosen color mana ability");
                act(&mut g, action);
                assert_eq!(g.state.players[p.index()].mana[color.index()], 1);
                g.untap_objects(&[land], untap_rules::UntapCause::Effect);
                g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 5];
                act(
                    &mut g,
                    Action::AnnounceActivate {
                        card: land,
                        ability: 1,
                        counter_payment: Counters::default(),
                    },
                );
                act(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(target),
                    },
                );
                pay(&mut g);
                assert_eq!(g.object_view(land).zone, Zone::Graveyard);
                if fizzle {
                    g.move_object(target, Zone::Exile);
                }
                let before_resolution = g.snapshot();
                super::contracts::branch_contract(g.clone());
                settle(&mut g);
                assert_eq!(
                    g.counter_count(target, CounterKind::PlusOne),
                    if fizzle { 0 } else { 4 }
                );
                g.restore(&before_resolution).unwrap();
                settle(&mut g);
                assert_eq!(
                    g.counter_count(target, CounterKind::PlusOne),
                    if fizzle { 0 } else { 4 }
                );
                g.restore(&before_entry).unwrap();
                act(
                    &mut g,
                    Action::ChooseEntryValue {
                        value: EntryChoiceValue::Color(color),
                    },
                );
                assert!(g.object_view(land).tapped);
            }
        }
    }
}

#[test]
fn primal_witchstalker_mills_then_targets_only_after_all_four() {
    for p in [Player::P0, Player::P1] {
        for count in 0..=4 {
            for fizzle in [false, true] {
                let mut g = scenario(p);
                while g.state.players[p.index()].library.len() > count {
                    let card = *g.state.players[p.index()].library.last().unwrap();
                    g.move_object(card, Zone::Exile);
                }
                let target = g.state.players[p.index()].library.last().copied();
                let wolf = put(&mut g, p, "Primal Witchstalker", Zone::Hand);
                cast(&mut g, wolf, None);
                let mut selected = false;
                for _ in 0..48 {
                    g.prepare_triggers();
                    let actions = g.legal_actions();
                    if let Some(target) = target {
                        let choice = Action::ChooseTarget {
                            target: Target::Card(target),
                        };
                        if actions.contains(&choice) {
                            assert_eq!(count, 4);
                            act(&mut g, choice);
                            selected = true;
                            break;
                        }
                    }
                    if g.state.stack.is_empty()
                        && g.state.pending_triggers.is_empty()
                        && g.state.pending == Pending::Priority
                    {
                        break;
                    }
                    choose(&mut g, |a| {
                        matches!(a, Action::Pass | Action::PutTrigger { .. })
                    });
                }
                assert_eq!(g.state.players[p.index()].library.len(), 0);
                assert_eq!(selected, count == 4);
                if selected {
                    let target = target.unwrap();
                    g.move_object(wolf, Zone::Exile);
                    if fizzle {
                        g.move_object(target, Zone::Exile);
                    }
                    let before = g.snapshot();
                    super::contracts::branch_contract(g.clone());
                    settle(&mut g);
                    assert_eq!(
                        g.object_view(target).zone,
                        if fizzle {
                            Zone::Exile
                        } else {
                            Zone::Battlefield
                        }
                    );
                    if !fizzle {
                        assert!(g.object_view(target).tapped);
                    }
                    g.restore(&before).unwrap();
                    settle(&mut g);
                    assert_eq!(
                        g.object_view(target).zone,
                        if fizzle {
                            Zone::Exile
                        } else {
                            Zone::Battlefield
                        }
                    );
                }
            }
        }
    }
}

#[test]
fn sphinx_of_false_conclusions_death_copies_last_known_nontoken() {
    for p in [Player::P0, Player::P1] {
        for doubled in [false, true] {
            for reenter in [false, true] {
                let mut g = scenario(p);
                if doubled {
                    put(&mut g, p, "Doubling Season", Zone::Battlefield);
                }
                let sphinx = put(&mut g, p, "Sphinx of False Conclusions", Zone::Battlefield);
                let printed = g.object_view(sphinx).card;
                g.add_counter(sphinx, CounterKind::PlusOne, 2);
                g.move_object(sphinx, Zone::Graveyard);
                g.prepare_triggers();
                if reenter {
                    g.move_object(sphinx, Zone::Battlefield);
                }
                let before = g.snapshot();
                super::contracts::branch_contract(g.clone());
                settle(&mut g);
                let copies: Vec<_> = g
                    .state
                    .objects
                    .iter()
                    .enumerate()
                    .filter_map(|(i, object)| {
                        (object.zone == Zone::Battlefield
                            && object.is_token()
                            && object.card == printed
                            && object.controller == p)
                            .then_some(i as CardId)
                    })
                    .collect();
                assert_eq!(copies.len(), if doubled { 2 } else { 1 });
                for &copy in &copies {
                    assert_eq!(stats(&g, copy), (4, 2));
                    assert!(g.keywords(copy).contains(Keywords::FLYING));
                    assert_eq!(g.counter_count(copy, CounterKind::PlusOne), 0);
                }
                let after = g.snapshot();
                g.restore(&before).unwrap();
                settle(&mut g);
                assert_eq!(g.snapshot(), after);
                for copy in copies {
                    g.move_object(copy, Zone::Graveyard);
                }
                g.prepare_triggers();
                assert!(g.state.pending_triggers.is_empty());
                assert!(g.state.stack.is_empty());
            }
        }
    }
}

#[test]
fn sphinx_attack_draws_then_discards_with_private_choice() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let sphinx = put(&mut g, p, "Sphinx of False Conclusions", Zone::Battlefield);
        let discard = put(&mut g, p, "Bear Cub", Zone::Hand);
        let foreign = put(&mut g, p.other(), "Bear Cub", Zone::Hand);
        let library = g.state.players[p.index()].library.len();
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: sphinx });
        act(&mut g, Action::FinishAttackers);
        let mut discarded = false;
        for _ in 0..48 {
            let action = Action::Discard { card: discard };
            if g.legal_actions().contains(&action) {
                reject(&mut g, Action::Discard { card: foreign });
                super::contracts::branch_contract(g.clone());
                act(&mut g, action);
                discarded = true;
            } else if g.state.pending == Pending::Priority
                && g.state.stack.is_empty()
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
        assert_eq!(g.object_view(discard).zone, Zone::Graveyard);
    }
}

#[test]
fn hexhaven_dueling_arena_uses_exact_attacker_history() {
    for p in [Player::P0, Player::P1] {
        for reenter in [false, true] {
            let mut g = scenario(p);
            let arena = put(&mut g, p, "Hexhaven Dueling Arena", Zone::Battlefield);
            let fighter = put(&mut g, p, "Pigment Wrangler", Zone::Battlefield);
            let other = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
            let activation = Action::AnnounceActivate {
                card: arena,
                ability: 1,
                counter_payment: Counters::default(),
            };
            reject(&mut g, activation.clone());
            g.state.step = Step::Attackers;
            g.state.pending = Pending::Attackers;
            act(&mut g, Action::Attack { card: fighter });
            act(&mut g, Action::FinishAttackers);
            g.state.step = Step::Main2;
            g.state.priority = p;
            if reenter {
                g.move_object(fighter, Zone::Exile);
                g.move_object(fighter, Zone::Battlefield);
                reject(&mut g, activation);
            } else {
                act(&mut g, activation);
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(other),
                    },
                );
                act(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(fighter),
                    },
                );
                pay(&mut g);
                let before = g.snapshot();
                super::contracts::branch_contract(g.clone());
                settle(&mut g);
                assert!(g.is_prepared(g.object_ref(fighter)));
                g.restore(&before).unwrap();
                settle(&mut g);
                assert!(g.is_prepared(g.object_ref(fighter)));
            }
        }
    }
}

#[test]
fn hexhaven_dueling_arena_general_preparation_is_instant_speed() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let arena = put(&mut g, p, "Hexhaven Dueling Arena", Zone::Battlefield);
        let creature = put(&mut g, p.other(), "Pigment Wrangler", Zone::Battlefield);
        g.state.active = p.other();
        g.state.step = Step::Attackers;
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 4];
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: arena,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        act(
            &mut g,
            Action::AnnounceActivate {
                card: arena,
                ability: 2,
                counter_payment: Counters::default(),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(creature),
            },
        );
        pay(&mut g);
        super::contracts::branch_contract(g.clone());
        settle(&mut g);
        assert!(g.is_prepared(g.object_ref(creature)));
    }
}

#[test]
fn guiding_hydra_optional_counter_removal_and_simultaneous_placement() {
    for p in [Player::P0, Player::P1] {
        for accept in [false, true] {
            for doubled in [false, true] {
                let mut g = scenario(p);
                let hydra = put(&mut g, p, "Guiding Hydra", Zone::Battlefield);
                g.add_counter(hydra, CounterKind::PlusOne, 2);
                let first = put(&mut g, p, "Bear Cub", Zone::Battlefield);
                let second = put(&mut g, p, "Bear Cub", Zone::Battlefield);
                let opponent = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
                if doubled {
                    put(&mut g, p, "Doubling Season", Zone::Battlefield);
                }
                g.begin_step(Step::BeginCombat);
                g.prepare_triggers();
                for _ in 0..32 {
                    if matches!(g.state.pending, Pending::EffectOptional { .. }) {
                        break;
                    }
                    choose(&mut g, |a| {
                        matches!(a, Action::Pass | Action::PutTrigger { .. })
                    });
                }
                assert!(matches!(g.state.pending, Pending::EffectOptional { .. }));
                let before = g.snapshot();
                super::contracts::branch_contract(g.clone());
                act(&mut g, Action::ChooseOptional { accept });
                settle(&mut g);
                assert_eq!(
                    g.counter_count(hydra, CounterKind::PlusOne),
                    if accept { 1 } else { 2 }
                );
                for creature in [first, second] {
                    assert_eq!(
                        g.counter_count(creature, CounterKind::PlusOne),
                        if accept {
                            if doubled { 2 } else { 1 }
                        } else {
                            0
                        }
                    );
                }
                assert_eq!(g.counter_count(opponent, CounterKind::PlusOne), 0);
                let after = g.snapshot();
                g.restore(&before).unwrap();
                act(&mut g, Action::ChooseOptional { accept });
                settle(&mut g);
                assert_eq!(g.snapshot(), after);
            }
        }
    }
}

#[test]
fn guiding_hydra_enters_with_x_counters() {
    for p in [Player::P0, Player::P1] {
        for doubled in [false, true] {
            let mut g = scenario(p);
            if doubled {
                put(&mut g, p, "Doubling Season", Zone::Battlefield);
            }
            let hydra = put(&mut g, p, "Guiding Hydra", Zone::Hand);
            act(&mut g, Action::AnnounceCast { card: hydra });
            act(&mut g, Action::ChooseX { value: 2 });
            pay(&mut g);
            super::contracts::branch_contract(g.clone());
            settle(&mut g);
            assert_eq!(
                g.counter_count(hydra, CounterKind::PlusOne),
                if doubled { 4 } else { 2 }
            );
            assert_eq!(stats(&g, hydra), if doubled { (5, 4) } else { (3, 2) });
        }
    }
}

#[test]
fn guiding_hydra_cannot_remove_a_missing_counter_or_from_a_new_incarnation() {
    for p in [Player::P0, Player::P1] {
        for source_leaves in [false, true] {
            let mut g = scenario(p);
            let hydra = put(&mut g, p, "Guiding Hydra", Zone::Battlefield);
            let friend = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            if source_leaves {
                g.add_counter(hydra, CounterKind::PlusOne, 1);
            } else {
                g.state.objects[hydra as usize].growth.toughness = 1;
            }
            g.begin_step(Step::BeginCombat);
            g.prepare_triggers();
            if source_leaves {
                g.move_object(hydra, Zone::Exile);
                g.move_object(hydra, Zone::Battlefield);
                g.add_counter(hydra, CounterKind::PlusOne, 2);
            }
            let before = g.snapshot();
            super::contracts::branch_contract(g.clone());
            settle(&mut g);
            assert_eq!(g.counter_count(friend, CounterKind::PlusOne), 0);
            assert_eq!(
                g.counter_count(hydra, CounterKind::PlusOne),
                if source_leaves { 2 } else { 0 }
            );
            let after = g.snapshot();
            g.restore(&before).unwrap();
            settle(&mut g);
            assert_eq!(g.snapshot(), after);
        }
    }
}

#[test]
fn plan_for_all_outcomes_library_choice_belongs_to_target_owner() {
    for p in [Player::P0, Player::P1] {
        for destination in [LibraryDestination::Top, LibraryDestination::Bottom] {
            let mut g = scenario(p);
            let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let old_ref = g.object_ref(target);
            g.change_controller(old_ref, p);
            let land = put(&mut g, p, "Forest", Zone::Battlefield);
            let plan = put(&mut g, p, "Plan for All Outcomes", Zone::Hand);
            cast(&mut g, plan, None);
            let mut selected = false;
            for _ in 0..48 {
                g.prepare_triggers();
                let actions = g.legal_actions();
                let wanted = Action::ChooseTarget {
                    target: Target::Card(target),
                };
                if actions.contains(&wanted) {
                    reject(
                        &mut g,
                        Action::ChooseTarget {
                            target: Target::Card(land),
                        },
                    );
                    reject(
                        &mut g,
                        Action::ChooseTarget {
                            target: Target::Card(plan),
                        },
                    );
                    act(&mut g, wanted);
                    selected = true;
                    break;
                }
                choose(&mut g, |a| {
                    matches!(a, Action::Pass | Action::PutTrigger { .. })
                });
            }
            assert!(selected);
            for _ in 0..48 {
                if matches!(g.state.pending, Pending::TargetLibrary { .. }) {
                    break;
                }
                act(&mut g, Action::Pass);
            }
            assert_eq!(g.actor(), Some(p.other()));
            let choice = Action::ChooseLibraryDestination {
                object: g.object_ref(target),
                destination,
            };
            let before = g.snapshot();
            super::contracts::branch_contract(g.clone());
            act(&mut g, choice.clone());
            settle(&mut g);
            assert_eq!(g.object_view(target).zone, Zone::Library);
            let library = &g.state.players[p.other().index()].library;
            assert_eq!(
                if destination == LibraryDestination::Top {
                    library.last()
                } else {
                    library.first()
                },
                Some(&target)
            );
            let after = g.snapshot();
            g.restore(&before).unwrap();
            act(&mut g, choice);
            settle(&mut g);
            assert_eq!(g.snapshot(), after);
        }
    }
}

#[test]
fn plan_for_all_outcomes_counts_first_noncreature_cast_before_entry() {
    for p in [Player::P0, Player::P1] {
        for already_cast in [false, true] {
            let mut g = scenario(p);
            if already_cast {
                let earlier = put(&mut g, p, "Shock", Zone::Hand);
                cast(&mut g, earlier, Some(Target::Player(p.other())));
                settle(&mut g);
            }
            put(&mut g, p, "Plan for All Outcomes", Zone::Battlefield);
            let creature = put(&mut g, p, "Bear Cub", Zone::Hand);
            cast(&mut g, creature, None);
            settle(&mut g);
            let first = put(&mut g, p, "Shock", Zone::Hand);
            cast(&mut g, first, Some(Target::Player(p.other())));
            super::contracts::branch_contract(g.clone());
            settle(&mut g);
            let jaces = fra_jaces(&g, p);
            assert_eq!(jaces.len(), if already_cast { 0 } else { 1 });
            if !already_cast {
                assert_eq!(g.counter_count(jaces[0], CounterKind::Loyalty), 1);
            }
            let second = put(&mut g, p, "Shock", Zone::Hand);
            cast(&mut g, second, Some(Target::Player(p.other())));
            settle(&mut g);
            let jaces = fra_jaces(&g, p);
            assert_eq!(jaces.len(), if already_cast { 0 } else { 1 });
            if !already_cast {
                assert_eq!(g.counter_count(jaces[0], CounterKind::Loyalty), 1);
            }
        }
    }
}

#[test]
fn stingcaster_mage_grants_exact_temporary_flashback_permission() {
    for p in [Player::P0, Player::P1] {
        for stale in [false, true] {
            let mut g = scenario(p);
            let spell = put(&mut g, p, "Shock", Zone::Graveyard);
            let foreign = put(&mut g, p.other(), "Shock", Zone::Graveyard);
            let creature = put(&mut g, p, "Bear Cub", Zone::Graveyard);
            let mage = put(&mut g, p, "Stingcaster Mage", Zone::Hand);
            cast(&mut g, mage, None);
            let wanted = Action::ChooseTarget {
                target: Target::Card(spell),
            };
            let mut selected = false;
            for _ in 0..48 {
                g.prepare_triggers();
                if g.legal_actions().contains(&wanted) {
                    reject(
                        &mut g,
                        Action::ChooseTarget {
                            target: Target::Card(foreign),
                        },
                    );
                    reject(
                        &mut g,
                        Action::ChooseTarget {
                            target: Target::Card(creature),
                        },
                    );
                    act(&mut g, wanted.clone());
                    selected = true;
                    break;
                }
                choose(&mut g, |a| {
                    matches!(a, Action::Pass | Action::PutTrigger { .. })
                });
            }
            assert!(selected);
            g.move_object(mage, Zone::Graveyard);
            if stale {
                g.move_object(spell, Zone::Exile);
                g.move_object(spell, Zone::Graveyard);
            }
            let before = g.snapshot();
            super::contracts::branch_contract(g.clone());
            settle(&mut g);
            assert_eq!(
                g.has_graveyard_cast(p, spell, GraveyardCastKind::Flashback),
                !stale
            );
            if !stale {
                let granted = g.snapshot();
                let mana = g.state.players[p.index()].mana.iter().sum::<u32>();
                act(
                    &mut g,
                    Action::AnnounceGrantedFlashback {
                        card: spell,
                        flash_surcharge: false,
                    },
                );
                act(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Player(p.other()),
                    },
                );
                pay(&mut g);
                assert_eq!(
                    mana - g.state.players[p.index()].mana.iter().sum::<u32>(),
                    1
                );
                settle(&mut g);
                assert_eq!(g.object_view(spell).zone, Zone::Exile);
                g.restore(&granted).unwrap();
                g.finish_cleanup();
                assert!(!g.has_graveyard_cast(p, spell, GraveyardCastKind::Flashback));
            }
            g.restore(&before).unwrap();
            settle(&mut g);
            assert_eq!(
                g.has_graveyard_cast(p, spell, GraveyardCastKind::Flashback),
                !stale
            );
        }
    }
}

#[test]
fn aerid_heartwood_mana_growth_and_death() {
    for p in [Player::P0, Player::P1] {
        for doubled in [false, true] {
            let mut g = scenario(p);
            if doubled {
                put(&mut g, p, "Doubling Season", Zone::Battlefield);
            }
            let source = put(&mut g, p, "Aerid Konstrari", Zone::Hand);
            cast(&mut g, source, None);
            settle(&mut g);
            let tokens = |g: &Game| -> Vec<CardId> {
                g.state
                    .objects
                    .iter()
                    .enumerate()
                    .filter_map(|(i, o)| {
                        (o.zone == Zone::Battlefield
                            && o.controller == p
                            && o.is_token()
                            && o.card == Card::test_index(5438))
                        .then_some(i as CardId)
                    })
                    .collect()
            };
            let first = tokens(&g);
            assert_eq!(first.len(), if doubled { 2 } else { 1 });
            let heartwood = first[0];
            assert_eq!(g.object_view(heartwood).colors, 8 | 16);
            assert!(g.has_subtype(heartwood, "Heartwood"));
            for color in [3, 4] {
                g.state.players[p.index()].mana = [0; 6];
                let action = g.legal_actions().into_iter().find(|a| matches!(a, Action::ActivateMana { card, production, .. } if *card == heartwood && production[color] == 1)).expect("Heartwood mana");
                act(&mut g, action);
                assert_eq!(g.state.players[p.index()].mana[color], 1);
                g.untap_objects(&[heartwood], untap_rules::UntapCause::Effect);
            }
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 6];
            act(
                &mut g,
                Action::AnnounceActivate {
                    card: source,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            pay(&mut g);
            let before = g.snapshot();
            super::contracts::branch_contract(g.clone());
            settle(&mut g);
            assert_eq!(tokens(&g).len(), if doubled { 4 } else { 2 });
            assert_eq!(stats(&g, source), if doubled { (9, 4) } else { (7, 4) });
            g.restore(&before).unwrap();
            settle(&mut g);
            g.finish_cleanup();
            assert_eq!(stats(&g, source), (5, 4));
            g.move_object(source, Zone::Graveyard);
            g.prepare_triggers();
            settle(&mut g);
            assert_eq!(tokens(&g).len(), if doubled { 6 } else { 3 });
        }
    }
}

#[test]
fn tenured_tethermage_can_tap_itself_if_it_is_an_artifact() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let mage = put(&mut g, p, "Tenured Tethermage", Zone::Battlefield);
        let artifact = put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Goldvein Pick", Zone::Battlefield);
        let stone = put(&mut g, p, "Stone by Sunlight", Zone::Hand);
        g.state.players[p.index()].mana = [10; 6];
        act(&mut g, Action::AnnounceCast { card: stone });
        act(&mut g, Action::ChooseMode { mode: 1 });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(mage),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        pay(&mut g);
        settle(&mut g);
        assert_ne!(g.object_view(mage).kinds & 16, 0);
        act(
            &mut g,
            Action::AnnounceActivate {
                card: mage,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        let mage_ref = g.object_ref(mage);
        let artifact_ref = g.object_ref(artifact);
        let enemy_ref = g.object_ref(enemy);
        reject(&mut g, Action::ChooseAdditionalCost { object: enemy_ref });
        act(&mut g, Action::ChooseAdditionalCost { object: mage_ref });
        reject(&mut g, Action::ChooseAdditionalCost { object: mage_ref });
        super::contracts::branch_contract(g.clone());
        act(
            &mut g,
            Action::ChooseAdditionalCost {
                object: artifact_ref,
            },
        );
        pay(&mut g);
        assert!(g.object_view(mage).tapped);
        assert!(g.object_view(artifact).tapped);
        let before = g.snapshot();
        settle(&mut g);
        assert_eq!(g.counter_count(mage, CounterKind::PlusOne), 2);
        g.restore(&before).unwrap();
        settle(&mut g);
        assert_eq!(g.counter_count(mage, CounterKind::PlusOne), 2);
    }
}

#[test]
fn tenured_tethermage_land_choice_creates_tapped_heartwood() {
    for p in [Player::P0, Player::P1] {
        for (sacrifice, doubled, stolen) in [
            (false, false, false),
            (true, false, false),
            (true, true, false),
            (true, false, true),
        ] {
            let mut g = scenario(p);
            if doubled {
                put(&mut g, p, "Doubling Season", Zone::Battlefield);
            }
            let land = put(
                &mut g,
                if stolen { p.other() } else { p },
                "Forest",
                Zone::Battlefield,
            );
            if stolen {
                let land_ref = g.object_ref(land);
                g.change_controller(land_ref, p);
            }
            let enemy = put(&mut g, p.other(), "Forest", Zone::Battlefield);
            let nonland = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let mage = put(&mut g, p, "Tenured Tethermage", Zone::Hand);
            cast(&mut g, mage, None);
            let mut decided = false;
            for _ in 0..64 {
                g.prepare_triggers();
                let actions = g.legal_actions();
                if !decided && actions.contains(&Action::DeclineEffectPayment) {
                    let enemy_ref = g.object_ref(enemy);
                    let nonland_ref = g.object_ref(nonland);
                    reject(&mut g, Action::Sacrifice { object: enemy_ref });
                    reject(
                        &mut g,
                        Action::Sacrifice {
                            object: nonland_ref,
                        },
                    );
                    super::contracts::branch_contract(g.clone());
                    if sacrifice {
                        let chosen = g.object_ref(land);
                        act(&mut g, Action::Sacrifice { object: chosen });
                    } else {
                        act(&mut g, Action::DeclineEffectPayment);
                    }
                    decided = true;
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
                    panic!("unexpected Tethermage choice {:?}", g.state.pending);
                }
            }
            assert!(decided);
            assert_eq!(
                g.object_view(land).zone,
                if sacrifice {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
            if sacrifice {
                let owner = if stolen { p.other() } else { p };
                assert_eq!(g.object_view(land).owner, owner);
            }
            let heartwoods: Vec<_> = g
                .state
                .objects
                .iter()
                .filter(|o| {
                    o.zone == Zone::Battlefield
                        && o.controller == p
                        && o.is_token()
                        && o.card == Card::test_index(5438)
                })
                .collect();
            assert_eq!(
                heartwoods.len(),
                if !sacrifice {
                    0
                } else if doubled {
                    4
                } else {
                    2
                }
            );
            assert!(heartwoods.iter().all(|o| o.tapped));
        }
    }
}

#[test]
fn tenured_tethermage_requires_two_artifacts_but_its_source_may_already_be_tapped() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let mage = put(&mut g, p, "Tenured Tethermage", Zone::Battlefield);
        let first = put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
        let activation = Action::AnnounceActivate {
            card: mage,
            ability: 0,
            counter_payment: Counters::default(),
        };
        reject(&mut g, activation.clone());
        let second = put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
        g.state.objects[mage as usize].tapped = true;
        g.state.objects[mage as usize].summoning_sick = true;
        act(&mut g, activation);
        let first_ref = g.object_ref(first);
        let second_ref = g.object_ref(second);
        act(&mut g, Action::ChooseAdditionalCost { object: first_ref });
        act(&mut g, Action::ChooseAdditionalCost { object: second_ref });
        pay(&mut g);
        assert!(g.object_view(first).tapped);
        assert!(g.object_view(second).tapped);
        settle(&mut g);
        assert_eq!(g.counter_count(mage, CounterKind::PlusOne), 2);
    }
}

#[test]
fn hungering_puppetbeast_chooses_keyword_on_resolution_and_cleans_up() {
    for p in [Player::P0, Player::P1] {
        for (index, keyword) in [
            (0, Keywords::TRAMPLE),
            (1, Keywords::HEXPROOF),
            (2, Keywords::HASTE),
        ] {
            let mut g = scenario(p);
            let beast = put(&mut g, p, "Hungering Puppetbeast", Zone::Battlefield);
            let artifact = put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
            let enemy = put(&mut g, p.other(), "Goldvein Pick", Zone::Battlefield);
            act(
                &mut g,
                Action::AnnounceActivate {
                    card: beast,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            let beast_ref = g.object_ref(beast);
            let enemy_ref = g.object_ref(enemy);
            reject(&mut g, Action::ChooseAdditionalCost { object: beast_ref });
            reject(&mut g, Action::ChooseAdditionalCost { object: enemy_ref });
            let artifact_ref = g.object_ref(artifact);
            act(
                &mut g,
                Action::ChooseAdditionalCost {
                    object: artifact_ref,
                },
            );
            pay(&mut g);
            assert_eq!(g.object_view(artifact).zone, Zone::Graveyard);
            assert!(!g.legal_actions().contains(&Action::ChooseKeyword { index }));
            for _ in 0..32 {
                if g.legal_actions().contains(&Action::ChooseKeyword { index }) {
                    break;
                }
                act(&mut g, Action::Pass);
            }
            assert_eq!(g.counter_count(beast, CounterKind::PlusOne), 1);
            for viewer in [p, p.other()] {
                let view = g
                    .observe(viewer)
                    .keyword_choice
                    .expect("public keyword options");
                assert_eq!(view.player, p);
                assert_eq!(view.recipients, vec![g.object_ref(beast)]);
                assert_eq!(
                    view.choices,
                    vec![Keywords::TRAMPLE, Keywords::HEXPROOF, Keywords::HASTE]
                );
            }
            reject(&mut g, Action::ChooseKeyword { index: 3 });
            let before_wrong_actor = g.snapshot();
            let before_revision = g.revision();
            assert_eq!(
                g.apply(p.other(), Action::ChooseKeyword { index }),
                Err(Error::WrongActor)
            );
            assert_eq!(g.snapshot(), before_wrong_actor);
            assert_eq!(g.revision(), before_revision);
            let saved = g.snapshot();
            super::contracts::branch_contract(g.clone());
            act(&mut g, Action::ChooseKeyword { index });
            settle(&mut g);
            assert!(g.keywords(beast).contains(keyword));
            g.restore(&saved).unwrap();
            act(&mut g, Action::ChooseKeyword { index });
            settle(&mut g);
            assert!(g.keywords(beast).contains(keyword));
            g.finish_cleanup();
            assert!(!g.keywords(beast).contains(keyword));
            assert_eq!(g.counter_count(beast, CounterKind::PlusOne), 1);
        }
    }
}

#[test]
fn hungering_puppetbeast_does_not_choose_for_a_new_incarnation() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let beast = put(&mut g, p, "Hungering Puppetbeast", Zone::Battlefield);
        let artifact = put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
        act(
            &mut g,
            Action::AnnounceActivate {
                card: beast,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        let artifact_ref = g.object_ref(artifact);
        act(
            &mut g,
            Action::ChooseAdditionalCost {
                object: artifact_ref,
            },
        );
        pay(&mut g);
        let old = g.object_ref(beast);
        g.move_object(beast, Zone::Graveyard);
        g.move_object(beast, Zone::Battlefield);
        assert_ne!(g.object_ref(beast), old);
        for _ in 0..32 {
            g.prepare_triggers();
            let actions = g.legal_actions();
            assert!(
                !actions
                    .iter()
                    .any(|a| matches!(a, Action::ChooseKeyword { .. }))
            );
            if g.state.pending == Pending::Priority
                && g.state.stack.is_empty()
                && g.state.pending_triggers.is_empty()
                && g.state.resolving.is_none()
            {
                break;
            }
            let action = actions
                .into_iter()
                .find(|a| matches!(a, Action::Pass | Action::PutTrigger { .. }))
                .expect("source-departure resolution");
            act(&mut g, action);
        }
        assert_eq!(g.counter_count(beast, CounterKind::PlusOne), 0);
        for keyword in [Keywords::TRAMPLE, Keywords::HEXPROOF, Keywords::HASTE] {
            assert!(!g.keywords(beast).contains(keyword));
        }
    }
}

#[test]
fn hungering_puppetbeast_entry_heartwood_is_a_usable_sacrifice() {
    for p in [Player::P0, Player::P1] {
        for doubled in [false, true] {
            let mut g = scenario(p);
            if doubled {
                put(&mut g, p, "Doubling Season", Zone::Battlefield);
            }
            let beast = put(&mut g, p, "Hungering Puppetbeast", Zone::Hand);
            cast(&mut g, beast, None);
            settle(&mut g);
            let tokens: Vec<_> = g
                .state
                .objects
                .iter()
                .enumerate()
                .filter_map(|(i, o)| {
                    (o.zone == Zone::Battlefield
                        && o.controller == p
                        && o.is_token()
                        && o.card == Card::test_index(5438))
                    .then_some(i as CardId)
                })
                .collect();
            assert_eq!(tokens.len(), if doubled { 2 } else { 1 });
            assert!(tokens.iter().all(|id| !g.object_view(*id).tapped));
            g.state.players[p.index()].mana = [0; 6];
            let mana = g.legal_actions().into_iter().find(|a| matches!(a,
                Action::ActivateMana { card, production, .. } if *card == tokens[0] && production[3] == 1
            )).expect("Heartwood produces red");
            act(&mut g, mana);
            act(
                &mut g,
                Action::AnnounceActivate {
                    card: beast,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            let sacrificed = g.object_ref(tokens[0]);
            act(&mut g, Action::ChooseAdditionalCost { object: sacrificed });
            pay(&mut g);
            for _ in 0..32 {
                if g.legal_actions()
                    .contains(&Action::ChooseKeyword { index: 0 })
                {
                    break;
                }
                act(&mut g, Action::Pass);
            }
            assert_eq!(
                g.counter_count(beast, CounterKind::PlusOne),
                if doubled { 2 } else { 1 }
            );
            act(&mut g, Action::ChooseKeyword { index: 0 });
            settle(&mut g);
            assert!(g.keywords(beast).contains(Keywords::TRAMPLE));
            assert_ne!(g.object_view(tokens[0]).zone, Zone::Battlefield);
        }
    }
}

#[test]
fn hungering_puppetbeast_counter_replacements_resolve_before_keyword_choice() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let beast = put(&mut g, p, "Hungering Puppetbeast", Zone::Battlefield);
        let artifact = put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
        put(&mut g, p, "Doubling Season", Zone::Battlefield);
        put(&mut g, p, "Doubling Season", Zone::Battlefield);
        act(
            &mut g,
            Action::AnnounceActivate {
                card: beast,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        let artifact_ref = g.object_ref(artifact);
        act(
            &mut g,
            Action::ChooseAdditionalCost {
                object: artifact_ref,
            },
        );
        pay(&mut g);
        for _ in 0..32 {
            if g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::ChooseReplacement { .. }))
            {
                break;
            }
            act(&mut g, Action::Pass);
        }
        assert!(matches!(g.state.pending, Pending::Replacement { .. }));
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::ChooseKeyword { .. }))
        );
        let saved = g.snapshot();
        super::contracts::branch_contract(g.clone());
        while g
            .legal_actions()
            .iter()
            .any(|a| matches!(a, Action::ChooseReplacement { .. }))
        {
            choose(&mut g, |a| matches!(a, Action::ChooseReplacement { .. }));
        }
        assert_eq!(g.counter_count(beast, CounterKind::PlusOne), 4);
        assert!(
            g.legal_actions()
                .contains(&Action::ChooseKeyword { index: 1 })
        );
        act(&mut g, Action::ChooseKeyword { index: 1 });
        settle(&mut g);
        assert!(g.keywords(beast).contains(Keywords::HEXPROOF));
        g.restore(&saved).unwrap();
        while g
            .legal_actions()
            .iter()
            .any(|a| matches!(a, Action::ChooseReplacement { .. }))
        {
            choose(&mut g, |a| matches!(a, Action::ChooseReplacement { .. }));
        }
        assert_eq!(g.counter_count(beast, CounterKind::PlusOne), 4);
        act(&mut g, Action::ChooseKeyword { index: 2 });
        settle(&mut g);
        assert!(g.keywords(beast).contains(Keywords::HASTE));
        assert!(!g.keywords(beast).contains(Keywords::HEXPROOF));
    }
}

#[test]
fn saheeli_grants_haste_to_a_noncreature_thopter() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Saheeli, Jewel of Avishkar", Zone::Battlefield);
        g.create_tokens(p, Card::test_index(5266), 1, false);
        let thopter = fra_thopters(&g, p)[0];
        let reference = g.object_ref(thopter);
        let mut recipe = CopyRecipe::new(Card::test_index(5266));
        recipe.overrides = Some(CopyOverrides {
            kinds: Some(16 | 128),
            ..CopyOverrides::NONE
        });
        g.apply_temporary_copy(reference, recipe, CopyDeadline::EndOfTurn);
        assert!(!g.is_creature(thopter));
        assert!(g.has_subtype(thopter, "Thopter"));
        assert!(g.keywords(thopter).contains(Keywords::HASTE));
        super::contracts::branch_contract(g.clone());
        g.change_controller(reference, p.other());
        assert!(!g.keywords(thopter).contains(Keywords::HASTE));
        g.change_controller(reference, p);
        assert!(g.keywords(thopter).contains(Keywords::HASTE));
        g.move_object(source, Zone::Graveyard);
        assert!(!g.keywords(thopter).contains(Keywords::HASTE));
    }
}

#[test]
fn saheeli_grants_haste_based_on_copied_subtype() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Saheeli, Jewel of Avishkar", Zone::Battlefield);
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        assert!(!g.keywords(bear).contains(Keywords::HASTE));
        let reference = g.object_ref(bear);
        g.apply_temporary_copy(
            reference,
            CopyRecipe::new(Card::test_index(5266)),
            CopyDeadline::EndOfTurn,
        );
        assert!(g.has_subtype(bear, "Thopter"));
        assert!(g.keywords(bear).contains(Keywords::HASTE));
        super::contracts::branch_contract(g.clone());
        g.move_object(source, Zone::Graveyard);
        assert!(!g.keywords(bear).contains(Keywords::HASTE));
    }
}

#[test]
fn saheeli_grant_tracks_thopter_subtype_changes() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Saheeli, Jewel of Avishkar", Zone::Battlefield);
        g.create_tokens(p, Card::test_index(5266), 1, false);
        let thopter = fra_thopters(&g, p)[0];
        let thopter_ref = g.object_ref(thopter);
        assert!(g.keywords(thopter).contains(Keywords::HASTE));
        let aura = put(&mut g, p.other(), "Witness Protection", Zone::Battlefield);
        g.state.objects[aura as usize].attached_to = Some(thopter_ref);
        assert!(!g.has_subtype(thopter, "Thopter"));
        assert!(g.has_subtype(thopter, "Citizen"));
        assert!(!g.keywords(thopter).contains(Keywords::HASTE));
        super::contracts::branch_contract(g.clone());
        g.move_object(aura, Zone::Graveyard);
        assert!(g.has_subtype(thopter, "Thopter"));
        assert!(g.keywords(thopter).contains(Keywords::HASTE));
    }
}

#[test]
fn koth_landfall_uses_current_or_last_known_land_subtype() {
    for p in [Player::P0, Player::P1] {
        for (name, leaves) in [("Forest", false), ("Mountain", false), ("Mountain", true)] {
            let mut g = scenario(p);
            let koth = put(&mut g, p, "Koth, the Geomancer", Zone::Battlefield);
            let land = put(&mut g, p, name, Zone::Hand);
            let life = g.state.players[p.other().index()].life;
            g.state.players[p.index()].mana = [0; 6];
            g.move_object(land, Zone::Battlefield);
            g.prepare_triggers();
            if leaves {
                g.move_object(land, Zone::Graveyard);
            }
            super::contracts::branch_contract(g.clone());
            settle(&mut g);
            assert_eq!(g.state.players[p.other().index()].life, life - 1);
            assert_eq!(
                g.state.players[p.index()].mana[3],
                u32::from(name == "Mountain")
            );
            assert!(g.keywords(koth).contains(Keywords::REACH));
        }
    }
}

#[test]
fn koth_landfall_tracks_subtype_changes_before_resolution_or_departure() {
    for p in [Player::P0, Player::P1] {
        for (initial, replacement, leaves, red) in [
            ("Forest", "Mountain", false, 1),
            ("Mountain", "Forest", false, 0),
            ("Forest", "Mountain", true, 1),
            ("Mountain", "Forest", true, 0),
        ] {
            let mut g = scenario(p);
            put(&mut g, p, "Koth, the Geomancer", Zone::Battlefield);
            let land = put(&mut g, p, initial, Zone::Hand);
            let life = g.state.players[p.other().index()].life;
            g.state.players[p.index()].mana = [0; 6];
            g.move_object(land, Zone::Battlefield);
            g.prepare_triggers();
            let reference = g.object_ref(land);
            g.apply_temporary_copy(
                reference,
                CopyRecipe::new(Card::from_name(replacement).unwrap()),
                CopyDeadline::EndOfTurn,
            );
            assert_eq!(g.has_subtype(land, "Mountain"), red == 1);
            if leaves {
                g.move_object(land, Zone::Graveyard);
            }
            let saved = g.snapshot();
            super::contracts::branch_contract(g.clone());
            settle(&mut g);
            assert_eq!(g.state.players[p.other().index()].life, life - 1);
            assert_eq!(g.state.players[p.index()].mana[3], red);
            g.restore(&saved).unwrap();
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].mana[3], red);
        }
    }
}

#[test]
fn koth_event_subtype_facts_coexist_with_other_event_facts() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Koth, the Geomancer", Zone::Battlefield);
        let land = put(&mut g, p, "Mountain", Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        g.move_object(land, Zone::Battlefield);
        let packet = g
            .state
            .pending_triggers
            .last_mut()
            .expect("landfall packet");
        let event = packet.event_object.as_mut().expect("bound event");
        assert!(event.subtype_lki.is_some());
        // Simulate another event-object consumer retaining controller facts.
        event.facts = EventObjectFacts::Controller(p);
        g.prepare_triggers();
        g.move_object(land, Zone::Graveyard);
        let packet = g.state.stack.last().expect("landfall ability");
        let event = packet.event_object.as_ref().unwrap();
        assert!(matches!(event.facts, EventObjectFacts::Controller(controller) if controller == p));
        assert!(event.subtype_lki.unwrap().has_subtype("Mountain"));
        super::contracts::branch_contract(g.clone());
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana[3], 1);
    }
}

#[test]
fn koth_landfall_keeps_multiple_land_events_independent() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Koth, the Geomancer", Zone::Battlefield);
        let mountain = put(&mut g, p, "Mountain", Zone::Hand);
        let forest = put(&mut g, p, "Forest", Zone::Hand);
        let life = g.state.players[p.other().index()].life;
        g.state.players[p.index()].mana = [0; 6];
        g.move_object(mountain, Zone::Battlefield);
        g.move_object(forest, Zone::Battlefield);
        g.move_object(mountain, Zone::Graveyard);
        g.prepare_triggers();
        let saved = g.snapshot();
        super::contracts::branch_contract(g.clone());
        for _ in 0..64 {
            if g.state.pending == Pending::Priority
                && g.state.stack.is_empty()
                && g.state.pending_triggers.is_empty()
                && g.state.resolving.is_none()
            {
                break;
            }
            let action = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::Pass | Action::PutTrigger { .. }))
                .expect("landfall ordering");
            act(&mut g, action);
        }
        assert_eq!(g.state.players[p.other().index()].life, life - 2);
        assert_eq!(g.state.players[p.index()].mana[3], 1);
        g.restore(&saved).unwrap();
        for _ in 0..64 {
            if g.state.pending == Pending::Priority
                && g.state.stack.is_empty()
                && g.state.pending_triggers.is_empty()
                && g.state.resolving.is_none()
            {
                break;
            }
            let action = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::Pass | Action::PutTrigger { .. }))
                .expect("restored landfall ordering");
            act(&mut g, action);
        }
        assert_eq!(g.state.players[p.other().index()].life, life - 2);
        assert_eq!(g.state.players[p.index()].mana[3], 1);
    }
}

#[test]
fn break_under_pressure_opponent_chooses_tied_greatest_mana_value() {
    for p in [Player::P0, Player::P1] {
        for choose_second in [false, true] {
            let mut g = scenario(p);
            let small = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let first = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            let second = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            let irrelevant = put(&mut g, p.other(), "Doubling Season", Zone::Battlefield);
            let friendly = put(&mut g, p, "Hill Giant", Zone::Battlefield);
            let spell = put(&mut g, p, "Break Under Pressure", Zone::Hand);
            let life = g.state.players[p.index()].life;
            cast(&mut g, spell, Some(Target::Player(p.other())));
            for _ in 0..32 {
                if matches!(g.state.pending, Pending::EffectChoice { .. }) {
                    break;
                }
                act(&mut g, Action::Pass);
            }
            assert_eq!(g.actor(), Some(p.other()));
            for id in [small, irrelevant, friendly] {
                let object = g.object_ref(id);
                reject(&mut g, Action::Sacrifice { object });
            }
            let selected = if choose_second { second } else { first };
            let object = g.object_ref(selected);
            let saved = g.snapshot();
            super::contracts::branch_contract(g.clone());
            act(&mut g, Action::Sacrifice { object });
            settle(&mut g);
            assert_eq!(g.object_view(selected).zone, Zone::Graveyard);
            assert_eq!(
                g.object_view(if choose_second { first } else { second })
                    .zone,
                Zone::Battlefield
            );
            assert_eq!(g.state.players[p.index()].life, life + 2);
            g.restore(&saved).unwrap();
            act(&mut g, Action::Sacrifice { object });
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].life, life + 2);
        }
    }
}

#[test]
fn break_under_pressure_gains_life_when_opponent_has_no_eligible_permanent() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let land = put(&mut g, p.other(), "Forest", Zone::Battlefield);
        let enchantment = put(&mut g, p.other(), "Doubling Season", Zone::Battlefield);
        let spell = put(&mut g, p, "Break Under Pressure", Zone::Hand);
        let life = g.state.players[p.index()].life;
        cast(&mut g, spell, Some(Target::Player(p.other())));
        let saved = g.snapshot();
        super::contracts::branch_contract(g.clone());
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, life + 2);
        assert_eq!(g.object_view(land).zone, Zone::Battlefield);
        assert_eq!(g.object_view(enchantment).zone, Zone::Battlefield);
        g.restore(&saved).unwrap();
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, life + 2);
    }
}

#[test]
fn break_under_pressure_counts_controlled_planeswalkers_owned_by_the_caster() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let small = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let walker = put(&mut g, p, "Vivien Reid", Zone::Battlefield);
        g.add_counter(walker, CounterKind::Loyalty, 5);
        let walker_ref = g.object_ref(walker);
        g.change_controller(walker_ref, p.other());
        let spell = put(&mut g, p, "Break Under Pressure", Zone::Hand);
        let life = g.state.players[p.index()].life;
        cast(&mut g, spell, Some(Target::Player(p.other())));
        for _ in 0..32 {
            if matches!(g.state.pending, Pending::EffectChoice { .. }) {
                break;
            }
            act(&mut g, Action::Pass);
        }
        assert_eq!(g.actor(), Some(p.other()));
        let small_ref = g.object_ref(small);
        reject(&mut g, Action::Sacrifice { object: small_ref });
        let selected = g.object_ref(walker);
        act(&mut g, Action::Sacrifice { object: selected });
        settle(&mut g);
        assert_eq!(g.object_view(walker).zone, Zone::Graveyard);
        assert_eq!(g.object_view(walker).owner, p);
        assert_eq!(g.object_view(small).zone, Zone::Battlefield);
        assert_eq!(g.state.players[p.index()].life, life + 2);
    }
}

#[test]
fn break_under_pressure_sacrifice_is_untargeted_and_obeys_zone_replacement() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Rest in Peace", Zone::Battlefield);
        let creature = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        g.state.objects[creature as usize].gained_keywords =
            Keywords::HEXPROOF.union(Keywords::INDESTRUCTIBLE);
        let spell = put(&mut g, p, "Break Under Pressure", Zone::Hand);
        let life = g.state.players[p.index()].life;
        cast(&mut g, spell, Some(Target::Player(p.other())));
        for _ in 0..32 {
            if matches!(g.state.pending, Pending::EffectChoice { .. }) {
                break;
            }
            act(&mut g, Action::Pass);
        }
        assert_eq!(g.actor(), Some(p.other()));
        let object = g.object_ref(creature);
        assert!(g.legal_actions().contains(&Action::Sacrifice { object }));
        super::contracts::branch_contract(g.clone());
        act(&mut g, Action::Sacrifice { object });
        settle(&mut g);
        assert_eq!(g.object_view(creature).zone, Zone::Exile);
        assert_eq!(g.state.players[p.index()].life, life + 2);
    }
}

#[test]
fn break_under_pressure_does_nothing_if_its_target_becomes_illegal() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let creature = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let spell = put(&mut g, p, "Break Under Pressure", Zone::Hand);
        let life = g.state.players[p.index()].life;
        cast(&mut g, spell, Some(Target::Player(p.other())));
        g.state.players[p.other().index()].hexproof_until_end_turn = true;
        super::contracts::branch_contract(g.clone());
        settle(&mut g);
        assert_eq!(g.object_view(creature).zone, Zone::Battlefield);
        assert_eq!(g.state.players[p.index()].life, life);
    }
}

#[test]
fn break_under_pressure_uses_current_copied_mana_values_at_resolution() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let changed = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let highest = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let spell = put(&mut g, p, "Break Under Pressure", Zone::Hand);
        cast(&mut g, spell, Some(Target::Player(p.other())));
        let changed_ref = g.object_ref(changed);
        g.apply_temporary_copy(
            changed_ref,
            CopyRecipe::new(Card::from_name("Bear Cub").unwrap()),
            CopyDeadline::EndOfTurn,
        );
        assert!(g.object_mana_value(changed) < g.object_mana_value(highest));
        for _ in 0..32 {
            if matches!(g.state.pending, Pending::EffectChoice { .. }) {
                break;
            }
            act(&mut g, Action::Pass);
        }
        reject(
            &mut g,
            Action::Sacrifice {
                object: changed_ref,
            },
        );
        let highest_ref = g.object_ref(highest);
        act(
            &mut g,
            Action::Sacrifice {
                object: highest_ref,
            },
        );
        settle(&mut g);
        assert_eq!(g.object_view(highest).zone, Zone::Graveyard);
        assert_eq!(g.object_view(changed).zone, Zone::Battlefield);
    }
}

#[test]
fn koth_ignores_opponents_land_and_its_queued_trigger_survives_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let koth = put(&mut g, p, "Koth, the Geomancer", Zone::Battlefield);
        let enemy_land = put(&mut g, p.other(), "Mountain", Zone::Hand);
        let own_land = put(&mut g, p, "Mountain", Zone::Hand);
        let life = g.state.players[p.other().index()].life;
        g.state.players[p.index()].mana = [0; 6];
        g.move_object(enemy_land, Zone::Battlefield);
        assert!(g.state.pending_triggers.is_empty());
        g.move_object(own_land, Zone::Battlefield);
        g.prepare_triggers();
        g.move_object(koth, Zone::Graveyard);
        let saved = g.snapshot();
        super::contracts::branch_contract(g.clone());
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, life - 1);
        assert_eq!(g.state.players[p.index()].mana[3], 1);
        g.restore(&saved).unwrap();
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, life - 1);
        assert_eq!(g.state.players[p.index()].mana[3], 1);
    }
}

#[test]
fn hunters_axe_grants_attack_choice_to_equipped_creature() {
    for p in [Player::P0, Player::P1] {
        for (index, keyword, remove_axe) in [
            (0, Keywords::TRAMPLE, false),
            (1, Keywords::DEATHTOUCH, false),
            (1, Keywords::DEATHTOUCH, true),
        ] {
            let mut g = scenario(p);
            let bearer = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let axe = put(&mut g, p, "Hunter's Axe", Zone::Battlefield);
            let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            act(
                &mut g,
                Action::AnnounceActivate {
                    card: axe,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(enemy),
                },
            );
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(bearer),
                },
            );
            if g.legal_actions().contains(&Action::FinishTargets) {
                act(&mut g, Action::FinishTargets);
            }
            pay(&mut g);
            settle(&mut g);
            assert_eq!(g.object_view(axe).attached_to, Some(g.object_ref(bearer)));
            assert_eq!(stats(&g, bearer), (4, 2));
            g.state.step = Step::Attackers;
            g.state.pending = Pending::Attackers;
            act(&mut g, Action::Attack { card: bearer });
            act(&mut g, Action::FinishAttackers);
            if remove_axe {
                g.move_object(axe, Zone::Graveyard);
            }
            for _ in 0..48 {
                if g.legal_actions().contains(&Action::ChooseKeyword { index }) {
                    break;
                }
                choose(&mut g, |a| {
                    matches!(a, Action::Pass | Action::PutTrigger { .. })
                });
            }
            assert_eq!(g.actor(), Some(p));
            let view = g.observe(p).keyword_choice.expect("attack keyword choice");
            assert_eq!(view.recipients, vec![g.object_ref(bearer)]);
            assert_eq!(view.choices, vec![Keywords::TRAMPLE, Keywords::DEATHTOUCH]);
            let saved = g.snapshot();
            super::contracts::branch_contract(g.clone());
            act(&mut g, Action::ChooseKeyword { index });
            assert!(g.keywords(bearer).contains(keyword));
            g.restore(&saved).unwrap();
            act(&mut g, Action::ChooseKeyword { index });
            assert!(g.keywords(bearer).contains(keyword));
            g.finish_cleanup();
            assert!(!g.keywords(bearer).contains(keyword));
        }
    }
}

#[test]
fn hunters_axes_create_independent_attack_choices() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let bearer = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let first = put(&mut g, p, "Hunter's Axe", Zone::Battlefield);
        let second = put(&mut g, p, "Hunter's Axe", Zone::Battlefield);
        let bearer_ref = g.object_ref(bearer);
        g.state.objects[first as usize].attached_to = Some(bearer_ref);
        g.state.objects[second as usize].attached_to = Some(bearer_ref);
        assert_eq!(stats(&g, bearer), (6, 2));
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: bearer });
        act(&mut g, Action::FinishAttackers);
        let mut chosen = 0;
        for _ in 0..64 {
            let actions = g.legal_actions();
            if chosen < 2 && actions.contains(&Action::ChooseKeyword { index: chosen }) {
                if chosen == 0 {
                    super::contracts::branch_contract(g.clone());
                }
                act(&mut g, Action::ChooseKeyword { index: chosen });
                chosen += 1;
            } else if chosen == 2 {
                break;
            } else if let Some(action) = actions
                .into_iter()
                .find(|a| matches!(a, Action::Pass | Action::PutTrigger { .. }))
            {
                act(&mut g, action);
            } else {
                panic!("unexpected multiple Axe state {:?}", g.state.pending);
            }
        }
        assert_eq!(chosen, 2);
        assert!(g.keywords(bearer).contains(Keywords::TRAMPLE));
        assert!(g.keywords(bearer).contains(Keywords::DEATHTOUCH));
        g.finish_cleanup();
        assert!(!g.keywords(bearer).contains(Keywords::TRAMPLE));
        assert!(!g.keywords(bearer).contains(Keywords::DEATHTOUCH));
    }
}

#[test]
fn hunters_axe_attack_choice_does_not_affect_returned_creature() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let bearer = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let axe = put(&mut g, p, "Hunter's Axe", Zone::Battlefield);
        let before = g.object_ref(bearer);
        g.state.objects[axe as usize].attached_to = Some(before);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: bearer });
        act(&mut g, Action::FinishAttackers);
        g.move_object(bearer, Zone::Graveyard);
        g.move_object(bearer, Zone::Battlefield);
        assert_ne!(g.object_ref(bearer), before);
        for _ in 0..48 {
            let actions = g.legal_actions();
            assert!(
                !actions
                    .iter()
                    .any(|a| matches!(a, Action::ChooseKeyword { .. }))
            );
            if g.state.pending == Pending::Priority
                && g.state.stack.is_empty()
                && g.state.pending_triggers.is_empty()
                && g.state.resolving.is_none()
            {
                break;
            }
            let action = actions
                .into_iter()
                .find(|a| matches!(a, Action::Pass | Action::PutTrigger { .. }))
                .expect("granted attack resolution");
            act(&mut g, action);
        }
        assert!(!g.keywords(bearer).contains(Keywords::TRAMPLE));
        assert!(!g.keywords(bearer).contains(Keywords::DEATHTOUCH));
    }
}

#[test]
fn hunters_axe_equip_is_sorcery_speed_and_uses_exact_target_incarnation() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let axe = put(&mut g, p, "Hunter's Axe", Zone::Battlefield);
        let bearer = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let activation = Action::AnnounceActivate {
            card: axe,
            ability: 0,
            counter_payment: Counters::default(),
        };
        g.state.step = Step::BeginCombat;
        reject(&mut g, activation.clone());
        g.state.step = Step::Main1;
        act(&mut g, activation);
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bearer),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        pay(&mut g);
        let before = g.object_ref(bearer);
        g.move_object(bearer, Zone::Graveyard);
        g.move_object(bearer, Zone::Battlefield);
        assert_ne!(g.object_ref(bearer), before);
        super::contracts::branch_contract(g.clone());
        settle(&mut g);
        assert_eq!(g.object_view(axe).attached_to, None);
        assert_eq!(stats(&g, bearer), (2, 2));
    }
}

#[test]
fn twinned_vision_flashback_hybrid_discard_and_draw() {
    for p in [Player::P0, Player::P1] {
        for colored in [[0, 2, 0, 0, 0, 1], [0, 1, 0, 1, 0, 1], [0, 0, 0, 2, 0, 1]] {
            let mut g = scenario(p);
            let spell = put(&mut g, p, "Twinned Vision", Zone::Graveyard);
            let discard = put(&mut g, p, "Bear Cub", Zone::Hand);
            let foreign = put(&mut g, p.other(), "Bear Cub", Zone::Hand);
            g.state.players[p.index()].mana = colored;
            let initial_library = g.state.players[p.index()].library.len();
            act(&mut g, Action::AnnounceFlashback { card: spell });
            let mut chose_discard = false;
            for _ in 0..24 {
                let actions = g.legal_actions();
                if let Some(action) = actions
                    .iter()
                    .find(|a| matches!(a, Action::ChooseSpellCost { .. }))
                    .cloned()
                {
                    act(&mut g, action);
                } else if actions
                    .iter()
                    .any(|a| matches!(a, Action::ChooseAdditionalCost { .. }))
                {
                    let forbidden = g.object_ref(foreign);
                    reject(&mut g, Action::ChooseAdditionalCost { object: forbidden });
                    assert!(g.observe(p.other()).additional_cost.is_none());
                    let selected = g.object_ref(discard);
                    super::contracts::branch_contract(g.clone());
                    act(&mut g, Action::ChooseAdditionalCost { object: selected });
                    chose_discard = true;
                } else if let Some(action) = actions
                    .into_iter()
                    .find(|a| matches!(a, Action::PayCosts { .. }))
                {
                    act(&mut g, action);
                } else if g.state.pending == Pending::Priority && !g.state.stack.is_empty() {
                    break;
                } else {
                    panic!(
                        "unexpected flashback state {:?}, actions {:?}, mana {:?}, announcement {:?}",
                        g.state.pending,
                        g.legal_actions(),
                        g.state.players[p.index()].mana,
                        g.state.announcing
                    );
                }
            }
            assert!(chose_discard);
            assert_eq!(g.object_view(discard).zone, Zone::Graveyard);
            let saved = g.snapshot();
            settle(&mut g);
            assert_eq!(
                g.state.players[p.index()].library.len(),
                initial_library - 2
            );
            assert_eq!(g.object_view(spell).zone, Zone::Exile);
            g.restore(&saved).unwrap();
            settle(&mut g);
            assert_eq!(
                g.state.players[p.index()].library.len(),
                initial_library - 2
            );
        }
    }
}

#[test]
fn twinned_vision_hand_cast_draws_one_and_flashback_requires_discard() {
    for p in [Player::P0, Player::P1] {
        for mana in [[0, 1, 0, 0, 0, 1], [0, 0, 0, 1, 0, 1]] {
            let mut g = scenario(p);
            let spell = put(&mut g, p, "Twinned Vision", Zone::Hand);
            g.state.players[p.index()].mana = mana;
            let initial_library = g.state.players[p.index()].library.len();
            cast(&mut g, spell, None);
            settle(&mut g);
            assert_eq!(
                g.state.players[p.index()].library.len(),
                initial_library - 1
            );
            assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
            let mut g = scenario(p);
            let spell = put(&mut g, p, "Twinned Vision", Zone::Graveyard);
            g.state.players[p.index()].mana = [0, 2, 0, 0, 0, 1];
            assert!(
                !g.legal_actions()
                    .contains(&Action::AnnounceFlashback { card: spell })
            );
            reject(&mut g, Action::AnnounceFlashback { card: spell });
        }
    }
}

#[test]
fn loyalty_cost_without_a_replacement_is_not_doubled() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let planeswalker = put(&mut g, p, "Liliana, Dreadhorde General", Zone::Battlefield);
        g.add_counter(planeswalker, CounterKind::Loyalty, 6);
        put(&mut g, p, "Doubling Season", Zone::Battlefield);
        act(
            &mut g,
            Action::AnnounceActivate {
                card: planeswalker,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        assert_eq!(g.counter_count(planeswalker, CounterKind::Loyalty), 7);
        super::contracts::branch_contract(g.clone());
        settle(&mut g);
        assert_eq!(g.counter_count(planeswalker, CounterKind::Loyalty), 7);
    }
}

#[test]
fn konstrari_charm_exact_damage_and_counter_replacement() {
    for p in [Player::P0, Player::P1] {
        for mode in [0, 1] {
            let mut g = scenario(p);
            let creature = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            g.state.objects[creature as usize].growth.toughness = 10;
            g.state.objects[creature as usize].gained_keywords = Keywords::FLYING;
            if mode == 1 {
                put(&mut g, p, "Doubling Season", Zone::Battlefield);
            }
            let spell = put(&mut g, p, "Konstrari Charm", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 0, 1, 1, 0];
            act(&mut g, Action::AnnounceCast { card: spell });
            act(&mut g, Action::ChooseMode { mode });
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(creature),
                },
            );
            pay(&mut g);
            let before = g.snapshot();
            super::contracts::branch_contract(g.clone());
            settle(&mut g);
            if mode == 0 {
                assert_eq!(g.state.objects[creature as usize].damage, 6);
                assert_eq!(g.counter_count(creature, CounterKind::PlusOne), 0);
            } else {
                assert_eq!(g.counter_count(creature, CounterKind::PlusOne), 4);
                assert!(g.keywords(creature).contains(Keywords::TRAMPLE));
            }
            g.restore(&before).unwrap();
            settle(&mut g);
            if mode == 0 {
                assert_eq!(g.state.objects[creature as usize].damage, 6);
            } else {
                assert_eq!(g.counter_count(creature, CounterKind::PlusOne), 4);
            }
        }
    }
}

#[test]
fn charge_the_sanctum_fizzles_when_its_creature_leaves() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let creature = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let land = put(&mut g, p, "Forest", Zone::Battlefield);
        let spell = put(&mut g, p, "Charge the Sanctum", Zone::Hand);
        g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 2];
        act(&mut g, Action::AnnounceCast { card: spell });
        act(&mut g, Action::ChooseMode { mode: 1 });
        if g.legal_actions()
            .iter()
            .any(|a| matches!(a, Action::ChooseSpellCost { .. }))
        {
            choose(&mut g, |a| matches!(a, Action::ChooseSpellCost { .. }));
        }
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(land),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(creature),
            },
        );
        pay(&mut g);
        g.move_object(creature, Zone::Exile);
        let before = g.snapshot();
        super::contracts::branch_contract(g.clone());
        settle(&mut g);
        assert_eq!(g.object_view(creature).zone, Zone::Exile);
        assert_eq!(g.counter_count(creature, CounterKind::PlusOne), 0);
        assert!(!g.keywords(creature).contains(Keywords::FIRST_STRIKE));
        g.restore(&before).unwrap();
        settle(&mut g);
        assert_eq!(g.object_view(creature).zone, Zone::Exile);
        assert_eq!(g.counter_count(creature, CounterKind::PlusOne), 0);
    }
}

#[test]
fn fatehold_charm_returns_noncreature_spell_and_stolen_creature_to_owners_hand() {
    for p in [Player::P0, Player::P1] {
        for stack_spell in [false, true] {
            let mut g = scenario(if stack_spell { p.other() } else { p });
            let target = put(
                &mut g,
                p.other(),
                if stack_spell { "Shock" } else { "Bear Cub" },
                if stack_spell {
                    Zone::Hand
                } else {
                    Zone::Battlefield
                },
            );
            if stack_spell {
                cast(&mut g, target, Some(Target::Player(p)));
                g.state.priority = p;
            } else {
                let reference = g.object_ref(target);
                g.change_controller(reference, p);
            }
            let spell = put(&mut g, p, "Fatehold Charm", Zone::Hand);
            let life = g.state.players[p.index()].life;
            act(&mut g, Action::AnnounceCast { card: spell });
            act(&mut g, Action::ChooseMode { mode: 1 });
            act(
                &mut g,
                Action::ChooseTarget {
                    target: if stack_spell {
                        Target::Spell(target)
                    } else {
                        Target::Card(target)
                    },
                },
            );
            pay(&mut g);
            super::contracts::branch_contract(g.clone());
            settle(&mut g);
            assert_eq!(g.object_view(target).zone, Zone::Hand);
            assert_eq!(g.object_view(target).owner, p.other());
            assert_eq!(g.state.players[p.index()].life, life);
        }
    }
}

#[test]
fn stingerquill_charm_damage_hits_creatures_or_planeswalkers_and_rechecks_targets() {
    for p in [Player::P0, Player::P1] {
        for planeswalker in [false, true] {
            for invalid in [false, true] {
                let mut g = scenario(p);
                let target = put(
                    &mut g,
                    p.other(),
                    if planeswalker {
                        "Liliana, Dreadhorde General"
                    } else {
                        "Bear Cub"
                    },
                    Zone::Battlefield,
                );
                if planeswalker {
                    g.add_counter(target, CounterKind::Loyalty, 5);
                } else {
                    g.state.objects[target as usize].growth.toughness = 10;
                }
                let spell = put(&mut g, p, "Stingerquill Charm", Zone::Hand);
                g.state.players[p.index()].mana = [0, 0, 1, 1, 0, 0];
                act(&mut g, Action::AnnounceCast { card: spell });
                act(&mut g, Action::ChooseMode { mode: 0 });
                act(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(target),
                    },
                );
                pay(&mut g);
                if invalid {
                    g.move_object(target, Zone::Exile);
                }
                super::contracts::branch_contract(g.clone());
                settle(&mut g);
                if invalid {
                    assert_eq!(g.object_view(target).zone, Zone::Exile);
                } else if planeswalker {
                    assert_eq!(g.counter_count(target, CounterKind::Loyalty), 2);
                } else {
                    assert_eq!(g.state.objects[target as usize].damage, 3);
                }
            }
        }
    }
}

#[test]
fn koth_landfall_keeps_trigger_controller_after_source_changes_controller() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Koth, the Geomancer", Zone::Battlefield);
        let land = put(&mut g, p, "Mountain", Zone::Hand);
        let before_life = g.state.players[p.other().index()].life;
        g.state.players[p.index()].mana = [0; 6];
        g.state.players[p.other().index()].mana = [0; 6];
        g.move_object(land, Zone::Battlefield);
        g.prepare_triggers();
        let source_ref = g.object_ref(source);
        g.change_controller(source_ref, p.other());
        let checkpoint = g.snapshot();
        super::contracts::branch_contract(g.clone());
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, before_life - 1);
        assert_eq!(g.state.players[p.index()].mana[3], 1);
        assert_eq!(g.state.players[p.other().index()].mana[3], 0);
        g.restore(&checkpoint).unwrap();
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana[3], 1);
    }
}

#[test]
fn pia_thopter_creation_is_doubled_and_target_hexproof_fizzles() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Doubling Season", Zone::Battlefield);
        let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let pia = put(&mut g, p, "Pia, Determined Rebuilder", Zone::Hand);
        cast(&mut g, pia, None);
        settle(&mut g);
        assert_eq!(fra_thopters(&g, p).len(), 2);
        act(
            &mut g,
            Action::AnnounceActivate {
                card: pia,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        pay(&mut g);
        g.state.objects[target as usize].gained_keywords = Keywords::HEXPROOF;
        let checkpoint = g.snapshot();
        super::contracts::branch_contract(g.clone());
        settle(&mut g);
        assert_eq!(stats(&g, target), (2, 2));
        g.restore(&checkpoint).unwrap();
        settle(&mut g);
        assert_eq!(stats(&g, target), (2, 2));
    }
}

#[test]
fn vigorbloom_fight_mode_rechecks_control_and_hexproof_independently() {
    for p in [Player::P0, Player::P1] {
        for own_changes_controller in [false, true] {
            let mut g = scenario(p);
            let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let spell = put(&mut g, p, "Vigorbloom Charm", Zone::Hand);
            g.state.players[p.index()].mana = [1, 0, 0, 0, 1, 0];
            act(&mut g, Action::AnnounceCast { card: spell });
            act(&mut g, Action::ChooseMode { mode: 2 });
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(own),
                },
            );
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(enemy),
                },
            );
            pay(&mut g);
            if own_changes_controller {
                let reference = g.object_ref(own);
                g.change_controller(reference, p.other());
            } else {
                g.state.objects[enemy as usize].gained_keywords = Keywords::HEXPROOF;
            }
            let checkpoint = g.snapshot();
            super::contracts::branch_contract(g.clone());
            settle(&mut g);
            assert_eq!(
                g.counter_count(own, CounterKind::PlusOne),
                u64::from(!own_changes_controller)
            );
            assert_eq!(g.counter_count(enemy, CounterKind::PlusOne), 0);
            assert_eq!(g.state.objects[own as usize].damage, 0);
            assert_eq!(g.state.objects[enemy as usize].damage, 0);
            g.restore(&checkpoint).unwrap();
            settle(&mut g);
            assert_eq!(g.counter_count(enemy, CounterKind::PlusOne), 0);
        }
    }
}

#[test]
fn loyal_tutor_search_is_private_revealed_and_replayable() {
    for p in [Player::P0, Player::P1] {
        for find in [false, true] {
            let mut g = scenario(p);
            let walker = put(&mut g, p, "Ajani, Caller of the Pride", Zone::Library);
            let other = put(&mut g, p, "Forest", Zone::Library);
            let spell = put(&mut g, p, "Loyal Tutor", Zone::Hand);
            cast(&mut g, spell, None);
            for _ in 0..48 {
                if g.observe(p).search_choice.is_some() {
                    break;
                }
                g.prepare_triggers();
                let action = g
                    .legal_actions()
                    .into_iter()
                    .find(|a| matches!(a, Action::Pass | Action::PutTrigger { .. }))
                    .expect("search progression");
                act(&mut g, action);
            }
            assert!(g.observe(p).search_choice.is_some());
            assert!(g.observe(p.other()).search_choice.is_none());
            reject(&mut g, Action::ChooseSearch { card: other });
            if find {
                act(&mut g, Action::ChooseSearch { card: walker });
            }
            super::contracts::branch_contract(g.clone());
            let mut replay = g.clone();
            act(&mut g, Action::FinishSearch);
            act(&mut replay, Action::FinishSearch);
            settle(&mut g);
            settle(&mut replay);
            assert_eq!(
                g.state.players[p.index()].library,
                replay.state.players[p.index()].library
            );
            assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
            if find {
                assert_eq!(g.state.players[p.index()].library.last(), Some(&walker));
                assert_eq!(
                    g.observe(p.other()).known_library_top[p.index()]
                        .unwrap()
                        .object,
                    g.object_ref(walker)
                );
            }
        }
    }
}

#[path = "engine_fra_awaken_tests.rs"]
mod fra_awaken_tests;

#[path = "engine_fra_loyal_tutor_tests.rs"]
mod loyal_tutor_assurance;

#[path = "engine_fra_discard_tests.rs"]
mod discard_assurance;

#[path = "engine_fra_gallia_tragic_tests.rs"]
mod gallia_tragic_assurance;

#[path = "engine_fra_gallia_entry_tests.rs"]
mod gallia_entry_assurance;

#[path = "engine_fra_hapatra_extra_tests.rs"]
mod hapatra_extra_assurance;

#[path = "engine_fra_hapatra_final_tests.rs"]
mod hapatra_final_assurance;

#[path = "engine_fra_ferocity_extra_tests.rs"]
mod ferocity_extra_assurance;

#[path = "engine_fra_candidate_followup_tests.rs"]
mod candidate_followup;

#[path = "engine_fra_second_followup_tests.rs"]
mod second_followup;

#[path = "engine_fra_scry_events_tests.rs"]
mod scry_events;
#[path = "engine_fra_tax_tests.rs"]
mod fra_tax_tests;

#[path = "engine_fra_third_followup_tests.rs"]
mod third_followup;

#[path = "engine_fra_fourth_followup_tests.rs"]
mod fourth_followup;

#[path = "engine_fra_peer_review_tests.rs"]
mod peer_review_tests;

#[path = "engine_fra_noncombat_history_tests.rs"]
mod noncombat_history_tests;

#[path = "engine_fra_noncombat_followup.rs"]
mod noncombat_followup;

#[path = "engine_fra_prepared_extra_tests.rs"]
mod engine_fra_prepared_extra_tests;

#[path = "engine_fra_teyo_followup_tests.rs"]
mod engine_fra_teyo_followup_tests;

#[path = "engine_fra_scry_followup_tests.rs"]
mod engine_fra_scry_followup_tests;

#[path = "engine_fra_kiora_teyo_tests.rs"]
mod engine_fra_kiora_teyo_tests;

#[path = "engine_fra_fifth_followup_tests.rs"]
mod engine_fra_fifth_followup_tests;

#[path = "engine_fra_sixth_followup_tests.rs"]
mod engine_fra_sixth_followup_tests;

#[path = "engine_fra_seventh_followup_tests.rs"]
mod engine_fra_seventh_followup_tests;

#[path = "engine_fra_eighth_followup_tests.rs"]
mod engine_fra_eighth_followup_tests;

#[path = "engine_fra_ninth_followup_tests.rs"]
mod engine_fra_ninth_followup_tests;

#[path = "engine_fra_tenth_followup_tests.rs"]
mod engine_fra_tenth_followup_tests;

#[path = "engine_fra_eleventh_followup_tests.rs"]
mod engine_fra_eleventh_followup_tests;

#[path = "engine_fra_artifact_review_tests.rs"]
mod engine_fra_artifact_review_tests;

#[path = "engine_fra_cohort_review.rs"]
mod engine_fra_cohort_review;

#[path = "engine_fra_peer_lifecycle_review_tests.rs"]
mod engine_fra_peer_lifecycle_review_tests;

#[path = "engine_fra_noncombat_boost_tests.rs"]
mod engine_fra_noncombat_boost_tests;

#[path = "engine_fra_combat_assignment_tests.rs"]
mod engine_fra_combat_assignment_tests;

#[path = "engine_fra_tomik_followup.rs"]
mod engine_fra_tomik_followup;

#[path = "engine_fra_ghalta_followup.rs"]
mod engine_fra_ghalta_followup;

#[path = "engine_fra_beast_ways_tests.rs"]
mod engine_fra_beast_ways_tests;

#[path = "engine_fra_conditional_artifacts_tests.rs"]
mod engine_fra_conditional_artifacts_tests;

#[path = "engine_fra_loyalty_grants_extra_tests.rs"]
mod engine_fra_loyalty_grants_extra_tests;

#[path = "engine_fra_cryomancer_tests.rs"]
mod engine_fra_cryomancer_tests;

#[path = "engine_fra_winter_tests.rs"]
mod engine_fra_winter_tests;

#[path = "engine_fra_craftwork_tests.rs"]
mod engine_fra_craftwork_tests;

#[path = "engine_fra_beast_followup.rs"]
mod engine_fra_beast_followup;

#[path = "engine_fra_conditional_review.rs"]
mod engine_fra_conditional_review;

#[path = "engine_fra_warlord_followup.rs"]
mod engine_fra_warlord_followup;

#[path = "engine_fra_winter_followup.rs"]
mod engine_fra_winter_followup;

#[path = "engine_fra_craftwork_followup.rs"]
mod engine_fra_craftwork_followup;

#[path = "engine_fra_edgar_followup.rs"]
mod engine_fra_edgar_followup;

#[path = "engine_fra_violent_echoes_independent_tests.rs"]
mod engine_fra_violent_echoes_independent_tests;

#[path = "engine_fra_solitary_cell_independent_tests.rs"]
mod engine_fra_solitary_cell_independent_tests;

#[path = "engine_fra_ingris_independent_tests.rs"]
mod engine_fra_ingris_independent_tests;

#[path = "engine_fra_tether_technician_independent_tests.rs"]
mod engine_fra_tether_technician_independent_tests;

#[path = "engine_fra_violent_echoes_tests.rs"]
mod engine_fra_violent_echoes_tests;

#[path = "engine_fra_solitary_cell_tests.rs"]
mod engine_fra_solitary_cell_tests;

#[path = "engine_fra_ingris_tests.rs"]
mod engine_fra_ingris_tests;
