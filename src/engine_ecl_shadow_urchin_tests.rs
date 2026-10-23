//! Exact pre-death counter totals and normal paid play until the next actual end step.
use super::ability_timeline_tests::{checked, named, settle, setup};
use super::*;
fn paid(g: &mut Game, p: Player, black: bool) -> CardId {
    let id = named(g, p, "Shadow Urchin", Zone::Hand);
    let payment = [0, 0, u32::from(black), u32::from(!black), 0, 2];
    g.state.players[p.index()].mana = payment;
    checked(g, Action::AnnounceCast { card: id });
    checked(
        g,
        Action::ChooseSpellCost {
            cost: u16::from(black),
        },
    );
    checked(g, Action::PayCosts { payment });
    settle(g);
    id
}
fn library(g: &mut Game, p: Player, n: usize) -> Vec<CardId> {
    g.state.players[p.index()].library.clear();
    (0..n)
        .map(|_| named(g, p, "Forest", Zone::Library))
        .collect()
}
fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let r = g.revision();
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), r);
}
#[test]
fn ecl_shadow_paid_hybrids_attack_blight_and_self_death_capture_all_counters() {
    for p in [Player::P0, Player::P1] {
        for black in [false, true] {
            let mut g = setup(p);
            let id = paid(&mut g, p, black);
            let cards = library(&mut g, p, 5);
            assert_eq!(
                g.stats(id),
                Stats {
                    power: 3,
                    toughness: 4
                }
            );
            g.state.objects[id as usize].summoning_sick = false;
            g.state.step = Step::Attackers;
            g.state.pending = Pending::Attackers;
            checked(&mut g, Action::Attack { card: id });
            checked(&mut g, Action::FinishAttackers);
            settle(&mut g);
            assert_eq!(g.object_view(id).counters.get(CounterKind::MinusOne), 1);
            // Three existing counters plus the printed attack's blight produce lethal toughness.
            g.set_counters(id, Counters::of(CounterKind::MinusOne, 4));
            g.state_based_actions();
            let packet = g
                .state
                .pending_triggers
                .iter()
                .find(|s| s.death_event.is_some())
                .unwrap();
            assert_eq!(packet.death_event.as_ref().unwrap().counter_total, 4);
            settle(&mut g);
            assert_eq!(g.state.objects[id as usize].zone, Zone::Graveyard);
            assert_eq!(g.state.exile_permissions.len(), 4);
            assert_eq!(g.state.objects[cards[0] as usize].zone, Zone::Library);
            assert!(
                g.state
                    .exile_permissions
                    .iter()
                    .all(|row| row.next_end_step == Some(p) && row.player == p)
            );
            assert_eq!(
                g.observe(p).exile_permissions,
                g.observe(p.other()).exile_permissions
            );
        }
    }
}
#[test]
fn ecl_shadow_counter_total_is_wide_and_frozen_before_source_and_victim_depart() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = paid(&mut g, p, false);
        let cards = library(&mut g, p, 7);
        let victim = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let mut counters = Counters::of(CounterKind::Flying, u64::MAX);
        counters.set(CounterKind::Charge, u64::MAX);
        g.set_counters(victim, counters);
        let events = g.move_objects_raw(&[source, victim], Zone::Graveyard);
        let sources = events.iter().map(|e| e.before.clone()).collect::<Vec<_>>();
        g.finish_zone_events(&events, &sources);
        let packet = g
            .state
            .pending_triggers
            .iter()
            .find(|s| s.death_event.is_some())
            .unwrap();
        assert_eq!(
            packet.death_event.as_ref().unwrap().counter_total,
            u128::from(u64::MAX) * 2
        );
        settle(&mut g);
        assert!(
            cards
                .iter()
                .all(|&id| g.state.objects[id as usize].zone == Zone::Exile)
        );
        assert_eq!(g.state.exile_permissions.len(), 7);
    }
}
#[test]
fn ecl_shadow_only_countered_controlled_creature_dies_and_actual_graveyard_destinations() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, true);
        library(&mut g, p, 8);
        let plain = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let artifact = named(&mut g, p, "Firdoch Core", Zone::Battlefield);
        let exiled = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        for id in [enemy, artifact, exiled] {
            g.set_counters(id, Counters::of(CounterKind::Charge, 1));
        }
        for id in [plain, enemy, artifact] {
            g.move_object(id, Zone::Graveyard);
        }
        g.move_object(exiled, Zone::Exile);
        settle(&mut g);
        assert!(g.state.exile_permissions.is_empty());
        let borrowed = named(&mut g, p.other(), "Bear Cub", Zone::Hand);
        g.state.objects[borrowed as usize].controller = p;
        g.move_object(borrowed, Zone::Battlefield);
        g.set_counters(borrowed, Counters::of(CounterKind::Stun, 2));
        g.move_object(borrowed, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.state.exile_permissions.len(), 2);
    }
}
#[test]
fn ecl_shadow_paid_spell_and_land_use_normal_timing_quota_and_exact_incarnation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, true);
        g.state.players[p.index()].library.clear();
        let land = named(&mut g, p, "Forest", Zone::Library);
        let second = named(&mut g, p, "Swamp", Zone::Library);
        let spell = named(&mut g, p, "Shock", Zone::Library);
        let victim = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.set_counters(victim, Counters::of(CounterKind::Charge, 3));
        g.move_object(victim, Zone::Graveyard);
        settle(&mut g);
        g.state.players[p.index()].mana = [0; 6];
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a,Action::Cast{card,..}if *card==spell))
        );
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 0];
        checked(&mut g, Action::AnnounceExile { card: spell });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        checked(
            &mut g,
            Action::PayCosts {
                payment: [0, 0, 0, 1, 0, 0],
            },
        );
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 18);
        checked(&mut g, Action::PlayLand { card: land });
        reject(&mut g, Action::PlayLand { card: second });
        let old = g.object_ref(second);
        g.move_object(second, Zone::Hand);
        g.move_object(second, Zone::Exile);
        assert_ne!(g.object_ref(second), old);
        assert!(!g.can_play_exiled(p, second));
    }
}
#[test]
fn ecl_shadow_deadline_occurs_before_own_end_triggers_and_survives_other_end_and_cleanup() {
    for p in [Player::P0, Player::P1] {
        for during_end in [false, true] {
            let mut g = setup(p);
            paid(&mut g, p, false);
            library(&mut g, p, 2);
            if during_end {
                g.begin_step(Step::End);
            }
            let victim = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            g.set_counters(victim, Counters::of(CounterKind::Charge, 1));
            g.move_object(victim, Zone::Graveyard);
            settle(&mut g);
            let card = g.state.exile_permissions[0].object.card;
            if during_end {
                g.finish_cleanup();
                assert!(g.can_play_exiled(p, card));
                g.state.active = p.other();
                g.state.turn += 1;
                g.begin_turn();
                g.begin_step(Step::End);
                assert!(g.can_play_exiled(p, card));
                g.finish_cleanup();
                assert!(g.can_play_exiled(p, card));
                g.state.active = p;
                g.state.turn += 1;
                g.begin_turn();
                assert!(g.can_play_exiled(p, card));
            }
            g.begin_step(Step::End);
            assert!(!g.can_play_exiled(p, card));
            assert!(g.state.exile_permissions.is_empty());
        }
    }
}
#[test]
fn ecl_shadow_forced_blight_can_choose_a_counter_prohibited_controlled_creature() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = paid(&mut g, p, false);
        let victim = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let aura = named(&mut g, p, "Blossombind", Zone::Hand);
        g.state.players[p.index()].mana = [30; 6];
        super::ability_timeline_tests::cast(&mut g, aura, None, &[Target::Card(victim)]);
        settle(&mut g);
        g.state.objects[source as usize].summoning_sick = false;
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        checked(&mut g, Action::Attack { card: source });
        checked(&mut g, Action::FinishAttackers);
        for _ in 0..20 {
            if matches!(g.state.pending, Pending::EffectChoice { .. }) {
                break;
            }
            checked(&mut g, Action::Pass);
        }
        let object = g.object_ref(victim);
        checked(&mut g, Action::Blight { object });
        settle(&mut g);
        assert_eq!(g.counter_count(victim, CounterKind::MinusOne), 0);
        assert_eq!(g.counter_count(source, CounterKind::MinusOne), 0);
    }
}
#[test]
fn ecl_shadow_simultaneous_counter_cancellation_and_death_use_pre_event_total() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, false);
        library(&mut g, p, 3);
        let victim = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.set_counters(
            victim,
            Counters::of(CounterKind::PlusOne, 1).with(CounterKind::MinusOne, 1),
        );
        g.state.objects[victim as usize].damage = 2;
        g.state_based_actions();
        assert_eq!(
            g.state
                .pending_triggers
                .iter()
                .find_map(|s| s.death_event.as_ref())
                .unwrap()
                .counter_total,
            2
        );
        settle(&mut g);
        assert_eq!(g.state.exile_permissions.len(), 2);
    }
}
