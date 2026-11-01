//! Focused printed outcomes; shared engine invariants live in core tests.
use super::*;
use crate::cards::{AnimationCreatureTypes, AnimationDuration, AnimationSpec};

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
        dealt_combat_damage_this_incarnation: false,
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
    {
        let p = Player::P0;
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
    {
        let p = Player::P0;
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
fn kiora_dragons_entry_activation_and_replacement() {
    {
        let p = Player::P0;
        {
            let mut g = scenario(p);

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
                            && o.card == Card::from_name("@token/8a8f70e310dc7f6a790f1a8f").unwrap()
                    })
                    .map(|(id, _)| id as CardId)
                    .collect()
            };
            let amount = { 1 };
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

            g.move_object(kiora, Zone::Exile);
            settle(&mut g);
            assert_eq!(dragons(&g).len(), amount * 2);
        }
    }
}

#[test]
fn eardrum_rattler_target_limits_tap_and_power_recheck() {
    {
        let p = Player::P0;
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
    {
        let p = Player::P0;
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
fn return_to_light_realms_simultaneous_entries_and_aura_choice() {
    {
        let p = Player::P0;
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
    {
        let p = Player::P0;
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
                    && o.card == Card::from_name("@token/jace-planeswalker").unwrap())
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
                && o.card == Card::from_name("@token/jace-planeswalker").unwrap())
            .then_some(i as CardId)
        })
        .collect()
}

#[test]
fn empower_consumers_entry_and_draw() {
    {
        let p = Player::P0;
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
fn tetsuko_uses_live_power_toughness_control_and_ability_origin() {
    {
        let p = Player::P0;
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
    {
        let p = Player::P0;
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
fn traxos_untap_restriction_and_cast_trigger() {
    {
        let p = Player::P0;
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
fn dark_matter_manipulator_mill_and_graveyard_groups_are_live() {
    {
        let p = Player::P0;
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
                && o.card == Card::from_name("@token/5941d6d4c2898b1bf55710b4").unwrap())
            .then_some(i as CardId)
        })
        .collect()
}

#[test]
fn saheeli_cast_trigger_and_thopter_haste_follow_controller_and_source() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let source = put(&mut g, p, "Saheeli, Jewel of Avishkar", Zone::Battlefield);
        let creature = put(&mut g, p, "Bear Cub", Zone::Hand);
        cast(&mut g, creature, None);
        settle(&mut g);
        assert!(fra_thopters(&g, p).is_empty());
        let shock = put(&mut g, p, "Shock", Zone::Hand);
        cast(&mut g, shock, Some(Target::Player(p.other())));

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
fn fatehold_charm_draw_empower_and_team_modes() {
    {
        let p = Player::P0;
        for mode in [0, 2] {
            let mut g = scenario(p);
            let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let opposing = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let before = g.state.players[p.index()].library.len();
            let charm = put(&mut g, p, "Fatehold Charm", Zone::Hand);
            act(&mut g, Action::AnnounceCast { card: charm });
            act(&mut g, Action::ChooseMode { mode });
            pay(&mut g);

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
fn wrath_of_the_bloodmane_reduction_and_creature_or_planeswalker_damage() {
    {
        let p = Player::P0;
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
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let opposing = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let artifact = put(&mut g, p, "Mox Jasper", Zone::Battlefield);
        put(&mut g, p.other(), "Mox Jasper", Zone::Battlefield);
        let source = put(&mut g, p, "Craterclaw Colossus", Zone::Hand);
        cast(&mut g, source, None);

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
fn pia_creates_thopter_and_uses_resolution_time_artifact_count() {
    {
        let p = Player::P0;
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
    {
        let p = Player::P0;
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
fn echoverse_fulcrum_sorcery_activation_exiles_cost_and_destroys_simultaneously() {
    {
        let p = Player::P0;
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

        settle(&mut g);
        assert_eq!(g.object_view(own).zone, Zone::Graveyard);
        assert_eq!(g.object_view(opposing).zone, Zone::Graveyard);
        assert_eq!(g.object_view(indestructible).zone, Zone::Battlefield);
    }
}

#[test]
fn darklight_phoenix_counts_both_players_deaths_on_its_owners_turn() {
    {
        let p = Player::P0;
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
fn living_library_pays_sacrifice_and_shuffles_into_actual_owners_library() {
    {
        let p = Player::P0;
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
fn gardenize_does_not_trigger_on_opponents_first_main_phase() {
    {
        let p = Player::P0;
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
        {
            let p = Player::P0;
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
    {
        let p = Player::P0;
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
    {
        let p = Player::P0;
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
    {
        let p = Player::P0;
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
    {
        let p = Player::P0;
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
    {
        let p = Player::P0;
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
    {
        let p = Player::P0;
        let mut g = scenario(p);
        put(&mut g, p, "Wildfire Wickerfolk", Zone::Battlefield);
        put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
        put(&mut g, p.other(), "Futurist Forge", Zone::Battlefield);
        let before = g.state.players[p.index()].library.len();
        let karn = put(&mut g, p, "Karn, Gilded Guardian", Zone::Hand);
        cast(&mut g, karn, None);

        settle(&mut g);
        assert_eq!(g.state.players[p.index()].library.len(), before - 2);
        assert_eq!(stats(&g, karn), (5, 5));
        assert!(g.keywords(karn).contains(Keywords::VIGILANCE));
        assert!(g.keywords(karn).contains(Keywords::TRAMPLE));
    }
}

#[test]
fn budding_insurgent_draws_for_legendary_enchantments_even_if_indestructible() {
    {
        let p = Player::P0;
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
    {
        let p = Player::P0;
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
    {
        let p = Player::P0;
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
fn jiang_yanggu_creates_legendary_mowu_and_untaps_all_owned_tokens() {
    {
        let p = Player::P0;
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
        g.create_tokens(
            p,
            Card::from_name("@token/283fd83a9aec83a51b736926").unwrap(),
            2,
            false,
        );
        g.create_tokens(
            p.other(),
            Card::from_name("@token/283fd83a9aec83a51b736926").unwrap(),
            2,
            false,
        );
        let foods: Vec<_> = g
            .state
            .objects
            .iter()
            .enumerate()
            .filter_map(|(id, o)| {
                (o.zone == Zone::Battlefield
                    && o.is_token()
                    && o.card == Card::from_name("@token/283fd83a9aec83a51b736926").unwrap())
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
    {
        let p = Player::P0;
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
fn traxos_academy_guardian_discount_tracks_noncreature_casts_and_prowess() {
    {
        let p = Player::P0;
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
    {
        let p = Player::P0;
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
                        && o.card == Card::from_name("@token/b60f6cd3b44b22bd289fe4e2").unwrap()
                })
                .count();
            assert_eq!(treasures, usize::from(triggered));
        }
    }
}

#[test]
fn mabel_can_select_different_pinned_named_counter_kinds() {
    {
        let p = Player::P0;
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
    {
        let p = Player::P0;
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
fn stingerquill_charm_modes_and_doubled_cadet_haste() {
    {
        let p = Player::P0;
        for mode in 0..3 {
            let mut g = scenario(p);
            let creature = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            g.create_tokens(
                p,
                Card::from_name("@token/adcb48b581ad80b8654a11b9").unwrap(),
                1,
                false,
            );
            settle(&mut g);
            let old_cadet = g
                .state
                .objects
                .iter()
                .enumerate()
                .find_map(|(i, o)| {
                    (o.zone == Zone::Battlefield
                        && o.is_token()
                        && o.card == Card::from_name("@token/adcb48b581ad80b8654a11b9").unwrap())
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
                                && o.card
                                    == Card::from_name("@token/adcb48b581ad80b8654a11b9").unwrap()
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
fn massacre_girl_death_damage_triggers_her_counter_ability() {
    {
        let p = Player::P0;
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
fn inspired_tethermage_observes_loyalty_costs_and_its_own_empower() {
    {
        let p = Player::P0;
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
    {
        let p = Player::P0;
        {
            let mut g = scenario(p);

            let spell = put(&mut g, p, "Hexhaven Battalion", Zone::Hand);
            cast(&mut g, spell, None);

            settle(&mut g);
            let cadets = g
                .state
                .objects
                .iter()
                .filter(|o| {
                    o.zone == Zone::Battlefield
                        && o.controller == p
                        && o.is_token()
                        && o.card == Card::from_name("@token/adcb48b581ad80b8654a11b9").unwrap()
                })
                .count();
            assert_eq!(cadets, { 3 });
            let jaces = fra_jaces(&g, p);
            assert_eq!(jaces.len(), 1);
            assert_eq!(
                jaces
                    .iter()
                    .map(|&id| g.counter_count(id, CounterKind::Loyalty))
                    .sum::<u64>(),
                { 2 }
            );
        }
    }
}

#[test]
fn solve_for_disappointment_reveals_hand_discards_permanent_then_empowers() {
    {
        let p = Player::P0;
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
fn vigorbloom_charm_draw_mode_gains_life() {
    {
        let p = Player::P0;
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
fn vraska_soul_of_stone_creates_sculpture_treasure_with_vigilance() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let vraska = put(&mut g, p, "Vraska, Soul of Stone", Zone::Battlefield);
        let own_bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let target = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let shock = put(&mut g, p, "Shock", Zone::Hand);
        g.state.players[p.index()].mana[3] = 1;
        cast(&mut g, shock, Some(Target::Card(target)));

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
                    && o.card == Card::from_name("@token/sculpture-treasure").unwrap())
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
    {
        let p = Player::P0;
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
                g.create_tokens(
                    p,
                    Card::from_name("@token/jace-planeswalker").unwrap(),
                    1,
                    false,
                );
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
fn hapatra_desert_frost_stuns_even_tapped_creatures_and_untap_removes_stun() {
    {
        let p = Player::P0;
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
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let creature = put(&mut g, p, "Pigment Wrangler", Zone::Hand);
        cast(&mut g, creature, None);
        settle(&mut g);
        assert!(g.is_prepared(g.object_ref(creature)));
        assert!(g.keywords(creature).contains(Keywords::FLYING));
        let aura = put(&mut g, p, "Infinite Coursework", Zone::Hand);
        cast(&mut g, aura, Some(Target::Card(creature)));

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
fn ferocity_flash_hybrid_payment_and_simultaneous_death() {
    {
        let p = Player::P0;
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

            settle(&mut g);
            assert_eq!(g.object_view(aura).zone, Zone::Graveyard);
            assert_eq!(g.object_view(creature).zone, Zone::Battlefield);
            assert!(g.object_view(creature).tapped);
            assert_eq!(g.object_view(creature).controller, p.other());
        }
    }
}

#[test]
fn warriors_blades_direct_equip_action_uses_target_counter_cost() {
    {
        let p = Player::P0;
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

        settle(&mut g);
        assert_eq!(
            g.object_view(equipment).attached_to,
            Some(g.object_ref(cheap))
        );
        assert_eq!(g.state.players[p.index()].mana[5], 0);
    }
}

#[test]
fn hexhaven_dueling_arena_general_preparation_is_instant_speed() {
    {
        let p = Player::P0;
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

        settle(&mut g);
        assert!(g.is_prepared(g.object_ref(creature)));
    }
}

#[test]
fn guiding_hydra_enters_with_x_counters() {
    {
        let p = Player::P0;
        {
            let mut g = scenario(p);

            let hydra = put(&mut g, p, "Guiding Hydra", Zone::Hand);
            act(&mut g, Action::AnnounceCast { card: hydra });
            act(&mut g, Action::ChooseX { value: 2 });
            pay(&mut g);

            settle(&mut g);
            assert_eq!(g.counter_count(hydra, CounterKind::PlusOne), { 2 });
            assert_eq!(stats(&g, hydra), { (3, 2) });
        }
    }
}

#[test]
fn plan_for_all_outcomes_counts_first_noncreature_cast_before_entry() {
    {
        let p = Player::P0;
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
fn tenured_tethermage_land_choice_creates_tapped_heartwood() {
    {
        let p = Player::P0;
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
                        && o.card == Card::from_name("@token/heartwood").unwrap()
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
fn hungering_puppetbeast_does_not_choose_for_a_new_incarnation() {
    {
        let p = Player::P0;
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
fn koth_landfall_uses_current_or_last_known_land_subtype() {
    {
        let p = Player::P0;
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
fn break_under_pressure_counts_controlled_planeswalkers_owned_by_the_caster() {
    {
        let p = Player::P0;
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
fn hunters_axe_attack_choice_does_not_affect_returned_creature() {
    {
        let p = Player::P0;
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
fn twinned_vision_hand_cast_draws_one_and_flashback_requires_discard() {
    {
        let p = Player::P0;
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

#[path = "engine_fra_loyal_tutor_tests.rs"]
mod loyal_tutor_assurance;

#[path = "engine_fra_gallia_tragic_tests.rs"]
mod gallia_tragic_assurance;

#[path = "engine_fra_tax_tests.rs"]
mod fra_tax_tests;
#[path = "engine_fra_scry_events_tests.rs"]
mod scry_events;

#[path = "engine_fra_noncombat_history_tests.rs"]
mod noncombat_history_tests;

#[path = "engine_fra_kiora_teyo_tests.rs"]
mod engine_fra_kiora_teyo_tests;

#[path = "engine_fra_noncombat_boost_tests.rs"]
mod engine_fra_noncombat_boost_tests;

#[path = "engine_fra_combat_assignment_tests.rs"]
mod engine_fra_combat_assignment_tests;

#[path = "engine_fra_beast_ways_tests.rs"]
mod engine_fra_beast_ways_tests;

#[path = "engine_fra_conditional_artifacts_tests.rs"]
mod engine_fra_conditional_artifacts_tests;

#[path = "engine_fra_cryomancer_tests.rs"]
mod engine_fra_cryomancer_tests;

#[path = "engine_fra_winter_tests.rs"]
mod engine_fra_winter_tests;

#[path = "engine_fra_craftwork_tests.rs"]
mod engine_fra_craftwork_tests;

#[path = "engine_fra_violent_echoes_tests.rs"]
mod engine_fra_violent_echoes_tests;

#[path = "engine_fra_ingris_tests.rs"]
mod engine_fra_ingris_tests;

#[path = "engine_fra_compel_brutality_tests.rs"]
mod engine_fra_compel_brutality_tests;

#[path = "engine_fra_ajani_unrelenting_tests.rs"]
mod engine_fra_ajani_unrelenting_tests;

#[path = "engine_fra_gideon_memorial_tests.rs"]
mod engine_fra_gideon_memorial_tests;

#[path = "engine_fra_behold_tests.rs"]
mod engine_fra_behold_tests;

#[path = "engine_fra_rise_draw_tests.rs"]
mod engine_fra_rise_draw_tests;

#[path = "engine_fra_mana_token_tests.rs"]
mod engine_fra_mana_token_tests;

#[path = "engine_fra_vanguard_tests.rs"]
mod engine_fra_vanguard_tests;

#[path = "engine_fra_roiling_canopy_tests.rs"]
mod engine_fra_roiling_canopy_tests;

#[path = "engine_fra_kiora_loyalty_history_tests.rs"]
mod engine_fra_kiora_loyalty_history_tests;

#[path = "engine_fra_kwia_kraken_tests.rs"]
mod engine_fra_kwia_kraken_tests;

#[path = "engine_fra_theorist_proxy_tests.rs"]
mod engine_fra_theorist_proxy_tests;

#[path = "engine_fra_token_name_tests.rs"]
mod engine_fra_token_name_tests;

#[path = "engine_fra_fblthp_tests.rs"]
mod engine_fra_fblthp_tests;

#[path = "engine_fra_hexhaven_invigorator_tests.rs"]
mod engine_fra_hexhaven_invigorator_tests;

#[path = "engine_fra_fblthp_domain_tests.rs"]
mod engine_fra_fblthp_domain_tests;

#[path = "engine_fra_proft_threshold_tests.rs"]
mod engine_fra_proft_threshold_tests;

#[path = "engine_fra_identity_echo_tests.rs"]
mod engine_fra_identity_echo_tests;

#[path = "engine_fra_flourishing_grapple_tests.rs"]
mod engine_fra_flourishing_grapple_tests;

#[path = "engine_fra_lyra_tests.rs"]
mod engine_fra_lyra_tests;

#[path = "engine_fra_liliana_repentant_tests.rs"]
mod engine_fra_liliana_repentant_tests;

#[path = "engine_fra_diviner_tests.rs"]
mod engine_fra_diviner_tests;

#[path = "engine_fra_bloodline_tests.rs"]
mod engine_fra_bloodline_tests;

#[path = "engine_fra_clash_tests.rs"]
mod engine_fra_clash_tests;

#[path = "engine_fra_combat_restrictions_tests.rs"]
mod engine_fra_combat_restrictions_tests;

#[path = "engine_fra_split_second_tests.rs"]
mod engine_fra_split_second_tests;

#[path = "engine_fra_player_alone_tests.rs"]
mod engine_fra_player_alone_tests;

#[path = "engine_fra_tetsuko_pursuer_tests.rs"]
mod engine_fra_tetsuko_pursuer_tests;

#[path = "engine_fra_loot_nexus_tests.rs"]
mod engine_fra_loot_nexus_tests;

#[path = "engine_fra_omnipresence_tests.rs"]
mod engine_fra_omnipresence_tests;

#[path = "engine_fra_codie_tests.rs"]
mod engine_fra_codie_tests;

#[path = "engine_fra_face_yourself_tests.rs"]
mod engine_fra_face_yourself_tests;

#[path = "engine_fra_variable_chaser_tests.rs"]
mod engine_fra_variable_chaser_tests;

#[path = "engine_fra_puppet_crafting_tests.rs"]
mod engine_fra_puppet_crafting_tests;

#[path = "engine_fra_vindictive_triumph_tests.rs"]
mod engine_fra_vindictive_triumph_tests;

#[path = "engine_fra_loyalty_observer_tests.rs"]
mod engine_fra_loyalty_observer_tests;

#[path = "engine_fra_tomik_lawmage_tests.rs"]
mod engine_fra_tomik_lawmage_tests;

#[path = "engine_fra_way_mind_sculptor_tests.rs"]
mod engine_fra_way_mind_sculptor_tests;

#[path = "engine_fra_garruk_curse_breaker_tests.rs"]
mod engine_fra_garruk_curse_breaker_tests;

#[path = "engine_fra_garruk_curse_timing_tests.rs"]
mod engine_fra_garruk_curse_timing_tests;

#[path = "engine_fra_chandra_torch_tests.rs"]
mod engine_fra_chandra_torch_tests;

#[path = "engine_fra_pyre_rhymer_tests.rs"]
mod engine_fra_pyre_rhymer_tests;

#[path = "engine_fra_null_summoner_tests.rs"]
mod engine_fra_null_summoner_tests;

#[path = "engine_fra_outside_pool_tests.rs"]
mod engine_fra_outside_pool_tests;

#[path = "engine_fra_planeswalker_type_count_tests.rs"]
mod engine_fra_planeswalker_type_count_tests;

#[path = "engine_fra_tam_proliferate_tests.rs"]
mod engine_fra_tam_proliferate_tests;

#[path = "engine_fra_essence_burn_tests.rs"]
mod engine_fra_essence_burn_tests;
