use super::tests::{act, scenario, spawn};
use super::*;
const NAMES: [&str; 8] = [
    "Avenger of the Fallen",
    "Dalkovan Packbeasts",
    "Dragonback Lancer",
    "Nightblade Brigade",
    "Reigning Victor",
    "Shock Brigade",
    "Stadium Headliner",
    "Zurgo's Vanguard",
];
fn named(g: &mut Game, p: Player, n: &str, z: Zone) -> CardId {
    let c = Card::from_name(n).unwrap();
    assert!(c.is_supported(), "{n}");
    spawn(g, p, c, z)
}
fn setup(p: Player) -> Game {
    let mut g = scenario();
    g.state.active = p;
    g.state.priority = p;
    g.state.players[p.index()].mana = [40; 6];
    g
}
fn replay(g: &mut Game, a: Action) {
    assert!(
        g.legal_actions().contains(&a),
        "{a:?} {:?}",
        g.state.pending
    );
    let old = g.snapshot();
    let rev = g.revision();
    let p = g.actor().unwrap();
    let mut f = g.clone();
    act(&mut f, a.clone());
    act(g, a.clone());
    let after = g.snapshot();
    assert_eq!(f.snapshot(), after);
    g.restore(&old).unwrap();
    assert_eq!(g.apply_index(p, rev, 0), Err(Error::StaleRevision));
    act(g, a);
    assert_eq!(g.snapshot(), after);
}
fn reject(g: &mut Game, a: Action) {
    let before = g.snapshot();
    let rev = g.revision();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), rev);
}
fn pick(g: &mut Game) {
    let a = g
        .legal_actions()
        .into_iter()
        .min_by_key(|a| match a {
            Action::ChooseTarget {
                target: Target::Card(_),
            } => 0,
            Action::ArrangeLibrary {
                destination: LibraryDestination::Graveyard,
                ..
            } => 0,
            Action::ChooseSpellCost { .. } => 0,
            Action::PayCosts { .. } => 1,
            Action::PutTrigger { .. } => 1,
            Action::ChooseMobilizeDefender {
                defender: CombatDefender::Player(_),
            } => 0,
            Action::FinishTargets => 2,
            Action::Pass => 3,
            Action::Concede => 10000,
            _ => 100,
        })
        .unwrap();
    replay(g, a);
}
fn idle(g: &Game) -> bool {
    g.state.stack.is_empty()
        && g.state.announcing.is_none()
        && g.state.resolving.is_none()
        && g.state.pending_triggers.is_empty()
        && g.state.ordering_triggers.is_empty()
        && g.state.pending == Pending::Priority
}
fn settle(g: &mut Game) {
    g.prepare_triggers();
    for _ in 0..100 {
        if idle(g) {
            return;
        }
        pick(g);
    }
    panic!("unsettled {:?}", g.state.pending)
}
fn paid(g: &mut Game, p: Player, n: &str) -> CardId {
    let id = named(g, p, n, Zone::Hand);
    replay(g, Action::AnnounceCast { card: id });
    while g.state.announcing.is_some() {
        pick(g);
    }
    let s = g
        .state
        .stack
        .iter()
        .find(|s| s.card == id && s.kind == StackKind::Spell)
        .unwrap();
    assert_eq!(s.controller, p);
    if n == "Reigning Victor" {
        assert_eq!(s.mana_value(), 6);
    }
    settle(g);
    assert_eq!(g.object_view(id).zone, Zone::Battlefield);
    id
}
fn attack(g: &mut Game, source: CardId) {
    g.state.objects[source as usize].summoning_sick = false;
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    replay(g, Action::Attack { card: source });
    replay(g, Action::FinishAttackers);
}
fn to_end(g: &mut Game) {
    g.state.step = Step::Main2;
    g.state.pending = Pending::Priority;
    g.state.priority = g.state.active;
    g.state.passes = 0;
    replay(g, Action::Pass);
    replay(g, Action::Pass);
    assert_eq!(g.state.step, Step::End);
}
#[test]
fn all_eight_paid_full_bodies_create_real_attackers_and_real_delayed_sacrifices_both_seats() {
    for p in [Player::P0, Player::P1] {
        for (i, n) in NAMES.iter().enumerate() {
            let mut g = setup(p);
            if i == 0 {
                named(&mut g, p, "Bear Cub", Zone::Graveyard);
                named(&mut g, p, "Bear Cub", Zone::Graveyard);
            }
            let source = paid(&mut g, p, n);
            let entries = g.state.players[p.index()].nonland_permanents_entered_this_turn;
            attack(&mut g, source);
            let declared = g.state.attacked_this_turn.len();
            settle(&mut g);
            let count = if i == 0 {
                2
            } else if i == 1 {
                3
            } else {
                1
            };
            let d = g.observe(p).delayed_sacrifices;
            assert_eq!(d.len(), 1, "{n}");
            assert_eq!(d[0].tokens.len(), count);
            assert_eq!(g.observe(p.other()).delayed_sacrifices, d);
            assert_eq!(g.state.attacked_this_turn.len(), declared);
            assert_eq!(
                g.state.players[p.index()].nonland_permanents_entered_this_turn,
                entries + count as u64
            );
            for r in &d[0].tokens {
                let v = g.object_view(r.card);
                assert_eq!(v.reference, *r);
                assert!(v.tapped && v.attacking && v.token.is_some());
                assert_eq!((v.power, v.toughness), (1, 1));
                assert_eq!(v.card.colors(), 8);
                assert!(v.card.has_subtype("Warrior"));
                assert_eq!(v.keywords, Keywords::NONE);
                assert!(
                    g.state.attackers.iter().any(|a| a.card == r.card
                        && a.defender == Some(CombatDefender::Player(p.other())))
                );
            }
            if i == 1 {
                assert!(!g.object_view(source).tapped);
            }
            if i == 7 {
                assert_eq!(g.object_view(source).power, 1 + count as i128);
            }
            let died = g.state.creatures_died_this_turn;
            to_end(&mut g);
            assert!(g.observe(p).delayed_sacrifices.is_empty());
            assert!(
                g.state
                    .stack
                    .iter()
                    .any(|s| s.kind == StackKind::DelayedSacrifice)
            );
            for r in &d[0].tokens {
                assert!(g.on_battlefield(r.card));
            }
            settle(&mut g);
            assert_eq!(g.state.creatures_died_this_turn, died + count as u64);
            for r in &d[0].tokens {
                assert_eq!(g.object_view(r.card).zone, Zone::Outside);
            }
            assert!(g.observe(p).delayed_sacrifices.is_empty());
        }
    }
}
#[test]
fn every_selected_two_brid_payment_retains_full_victor_entry_and_printed_mana_value() {
    for p in [Player::P0, Player::P1] {
        let c = Card::from_name("Reigning Victor").unwrap();
        let costs: Vec<_> = c.cost_options().collect();
        assert_eq!(costs.len(), 8);
        for (index, cost) in costs.into_iter().enumerate() {
            let mut g = setup(p);
            let s = named(&mut g, p, "Reigning Victor", Zone::Hand);
            replay(&mut g, Action::AnnounceCast { card: s });
            replay(&mut g, Action::ChooseSpellCost { cost: index as u16 });
            assert_eq!(g.state.announcing.as_ref().unwrap().total_cost, Some(cost));
            while g.state.announcing.is_some() {
                pick(&mut g);
            }
            let item = g.state.stack.last().unwrap();
            assert_eq!(item.mana_value(), 6);
            assert_eq!(item.source_card.colors(), 13);
            settle(&mut g);
            let v = g.object_view(s);
            assert_eq!(v.power, 4);
            assert!(v.keywords.contains(Keywords::INDESTRUCTIBLE));
        }
    }
}
#[test]
fn three_tokens_choose_separate_player_and_exact_planeswalkers_without_target_or_attack_events() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let src = paid(&mut g, p, "Dalkovan Packbeasts");
        let a = named(
            &mut g,
            p.other(),
            "Ajani, Caller of the Pride",
            Zone::Battlefield,
        );
        let b = named(
            &mut g,
            p.other(),
            "Professor Dellian Fel",
            Zone::Battlefield,
        );
        for id in [a, b] {
            g.state.objects[id as usize].counters.loyalty = 10;
        }
        attack(&mut g, src);
        for _ in 0..20 {
            if matches!(g.state.pending, Pending::Mobilize { .. }) {
                break;
            }
            pick(&mut g);
        }
        assert!(matches!(g.state.pending, Pending::Mobilize { .. }));
        let before = g.state.players[p.index()].nonland_permanents_entered_this_turn;
        assert!(g.observe(p.other()).mobilize_choice.is_none());
        let ar = g.object_ref(a);
        let br = g.object_ref(b);
        reject(
            &mut g,
            Action::ChooseMobilizeDefender {
                defender: CombatDefender::Planeswalker(ObjectRef {
                    card: a,
                    generation: ar.generation + 1,
                }),
            },
        );
        reject(
            &mut g,
            Action::ChooseMobilizeDefender {
                defender: CombatDefender::Player(p),
            },
        );
        for d in [
            CombatDefender::Player(p.other()),
            CombatDefender::Planeswalker(ar),
            CombatDefender::Planeswalker(br),
        ] {
            let rev = g.revision();
            let snap = g.snapshot();
            assert_eq!(
                g.apply(p.other(), Action::ChooseMobilizeDefender { defender: d }),
                Err(Error::WrongActor)
            );
            assert_eq!(g.snapshot(), snap);
            assert_eq!(g.revision(), rev);
            replay(&mut g, Action::ChooseMobilizeDefender { defender: d });
        }
        assert_eq!(
            g.state.players[p.index()].nonland_permanents_entered_this_turn,
            before + 3
        );
        assert_eq!(g.state.attacked_this_turn.len(), 1);
        let view = g.observe(p);
        let tokens = &view.delayed_sacrifices[0].tokens;
        for (r, d) in tokens.iter().zip([
            CombatDefender::Player(p.other()),
            CombatDefender::Planeswalker(ar),
            CombatDefender::Planeswalker(br),
        ]) {
            assert!(
                g.state
                    .attackers
                    .iter()
                    .any(|a| a.card == r.card && a.defender == Some(d))
            );
        }
        assert!(g.state.resolving.is_none());
    }
}
#[test]
fn zero_dynamic_count_allocates_nothing_and_dynamic_value_is_live_after_source_departure() {
    for p in [Player::P0, Player::P1] {
        for count in [0, 2] {
            let mut g = setup(p);
            let s = paid(&mut g, p, "Avenger of the Fallen");
            attack(&mut g, s);
            let old = g.object_ref(s);
            g.move_object(s, Zone::Exile);
            for _ in 0..count {
                named(&mut g, p, "Bear Cub", Zone::Graveyard);
            }
            settle(&mut g);
            let d = g.observe(p).delayed_sacrifices;
            assert_eq!(d.len(), usize::from(count > 0));
            if count > 0 {
                assert_eq!(d[0].tokens.len(), count);
                assert_eq!(d[0].source, old);
                to_end(&mut g);
                settle(&mut g);
                assert!(
                    d[0].tokens
                        .iter()
                        .all(|r| g.object_view(r.card).zone == Zone::Outside)
                );
            } else {
                assert!(g.state.delayed_sacrifices.is_none());
            }
        }
    }
}
#[test]
fn countered_mobilize_or_delayed_packet_never_spawns_or_requeues_costs() {
    for p in [Player::P0, Player::P1] {
        for delayed in [false, true] {
            let mut g = setup(p);
            let s = paid(&mut g, p, "Dragonback Lancer");
            attack(&mut g, s);
            if delayed {
                settle(&mut g);
                to_end(&mut g);
            }
            let packet = g
                .state
                .stack
                .iter()
                .find(|s| {
                    if delayed {
                        s.kind == StackKind::DelayedSacrifice
                    } else {
                        matches!(s.kind, StackKind::TriggeredAbility { .. })
                    }
                })
                .unwrap()
                .id;
            assert!(g.counter_stack_item(packet, false));
            settle(&mut g);
            if delayed {
                assert_eq!(
                    g.state
                        .objects
                        .iter()
                        .filter(|o| o.zone == Zone::Battlefield && o.card.is_token())
                        .count(),
                    1
                );
                g.begin_step(Step::End);
                g.prepare_triggers();
                assert!(g.state.stack.is_empty());
            } else {
                assert!(g.observe(p).delayed_sacrifices.is_empty());
                assert!(
                    !g.state
                        .objects
                        .iter()
                        .any(|o| o.zone == Zone::Battlefield && o.card.is_token())
                );
            }
        }
    }
}
#[test]
fn late_delayed_registration_failure_restores_complete_token_attack_entry_batch() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = paid(&mut g, p, "Dalkovan Packbeasts");
        attack(&mut g, s);
        replay(&mut g, Action::Pass);
        g.state.next_stack_id = u64::MAX;
        let before = g.snapshot();
        let rev = g.revision();
        assert!(matches!(
            g.apply(p.other(), Action::Pass),
            Err(Error::CapacityExceeded(_))
        ));
        assert_eq!(g.snapshot(), before);
        assert_eq!(g.revision(), rev);
        assert!(g.observe(p).delayed_sacrifices.is_empty());
    }
}

#[test]
fn real_combat_tokens_damage_each_chosen_defender_and_stop_attacking_before_sacrifice() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = paid(&mut g, p, "Dalkovan Packbeasts");
        let a = named(
            &mut g,
            p.other(),
            "Ajani, Caller of the Pride",
            Zone::Battlefield,
        );
        let b = named(
            &mut g,
            p.other(),
            "Professor Dellian Fel",
            Zone::Battlefield,
        );
        for id in [a, b] {
            g.state.objects[id as usize].counters.loyalty = 10;
        }
        attack(&mut g, s);
        while !matches!(g.state.pending, Pending::Mobilize { .. }) {
            pick(&mut g);
        }
        for d in [
            CombatDefender::Player(p.other()),
            CombatDefender::Planeswalker(g.object_ref(a)),
            CombatDefender::Planeswalker(g.object_ref(b)),
        ] {
            replay(&mut g, Action::ChooseMobilizeDefender { defender: d });
        }
        let tokens = g.observe(p).delayed_sacrifices[0].tokens.clone();
        replay(&mut g, Action::Pass);
        replay(&mut g, Action::Pass);
        assert!(matches!(g.state.pending, Pending::Blockers));
        replay(&mut g, Action::FinishBlockers);
        replay(&mut g, Action::Pass);
        replay(&mut g, Action::Pass);
        while !matches!(g.state.pending, Pending::Priority) {
            pick(&mut g);
        }
        assert_eq!(g.state.players[p.other().index()].life, 19);
        assert_eq!(g.object_view(a).counters.loyalty, 9);
        assert_eq!(g.object_view(b).counters.loyalty, 9);
        for _ in 0..8 {
            if g.state.step == Step::Main2 {
                break;
            }
            replay(&mut g, Action::Pass);
        }
        assert_eq!(g.state.step, Step::Main2);
        assert!(tokens.iter().all(|r| !g.object_view(r.card).attacking));
        assert_eq!(g.observe(p).delayed_sacrifices.len(), 1);
        replay(&mut g, Action::Pass);
        replay(&mut g, Action::Pass);
        assert_eq!(g.state.step, Step::End);
        assert!(tokens.iter().all(|r| g.on_battlefield(r.card)));
        settle(&mut g);
        assert!(
            tokens
                .iter()
                .all(|r| g.object_view(r.card).zone == Zone::Outside)
        );
    }
}
#[test]
fn departed_token_slots_never_rebind_and_each_creation_batch_owns_its_own_sacrifice() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = paid(&mut g, p, "Dalkovan Packbeasts");
        attack(&mut g, s);
        settle(&mut g);
        let old = g.observe(p).delayed_sacrifices[0].tokens.clone();
        let factory = g.state.objects[old[0].card as usize].card;
        g.move_object(old[0].card, Zone::Graveyard);
        g.state_based_actions();
        assert_eq!(g.object_view(old[0].card).zone, Zone::Outside);
        g.create_tokens(p, factory, 1, false);
        let new = g.object_ref(old[0].card);
        assert_ne!(new, old[0]);
        assert!(!g.object_view(new.card).attacking);
        to_end(&mut g);
        settle(&mut g);
        assert_eq!(g.object_ref(new.card), new);
        assert!(g.on_battlefield(new.card));
        assert!(
            old[1..]
                .iter()
                .all(|r| g.object_view(r.card).zone == Zone::Outside)
        );
    }
}
#[test]
fn multiple_batches_queue_once_in_controller_order_and_countering_one_keeps_only_its_tokens() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let a = paid(&mut g, p, "Dalkovan Packbeasts");
        let b = paid(&mut g, p, "Dragonback Lancer");
        for id in [a, b] {
            g.state.objects[id as usize].summoning_sick = false;
        }
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        replay(&mut g, Action::Attack { card: a });
        replay(&mut g, Action::Attack { card: b });
        replay(&mut g, Action::FinishAttackers);
        settle(&mut g);
        let batches = g.observe(p).delayed_sacrifices;
        assert_eq!(batches.len(), 2);
        assert_ne!(batches[0].id, batches[1].id);
        to_end(&mut g);
        assert!(g.observe(p).delayed_sacrifices.is_empty());
        while matches!(g.state.pending, Pending::OrderTriggers { .. }) {
            pick(&mut g);
        }
        assert_eq!(g.state.stack.len(), 2);
        let canceled = g.state.stack.last().unwrap().clone();
        let kept = canceled.delayed_sacrifice.as_ref().unwrap().tokens.clone();
        assert!(g.counter_stack_item(canceled.id, false));
        settle(&mut g);
        assert!(
            kept.iter()
                .all(|r| g.object_ref(r.card) == *r && g.on_battlefield(r.card))
        );
        assert!(
            batches
                .iter()
                .flat_map(|b| &b.tokens)
                .filter(|r| !kept.contains(r))
                .all(|r| g.object_view(r.card).zone == Zone::Outside)
        );
        g.begin_step(Step::End);
        g.prepare_triggers();
        assert!(g.state.stack.is_empty());
    }
}
#[test]
fn later_end_step_queue_and_second_sacrifice_failure_roll_back_only_current_action() {
    for p in [Player::P0, Player::P1] {
        for queue in [true, false] {
            let mut g = setup(p);
            let a = paid(&mut g, p, "Dalkovan Packbeasts");
            attack(&mut g, a);
            settle(&mut g);
            if queue {
                // A second separately owned batch, not a merged obligation.
                let source = g.object_view(a);
                let i = source
                    .card
                    .triggers()
                    .iter()
                    .position(|t| matches!(t.program, [Instruction::Mobilize { .. }]))
                    .unwrap();
                g.queue_trigger(&source, i);
                settle(&mut g);
                assert_eq!(g.observe(p).delayed_sacrifices.len(), 2);
                g.state.step = Step::Main2;
                g.state.pending = Pending::Priority;
                g.state.priority = p;
                g.state.passes = 0;
                replay(&mut g, Action::Pass);
                g.state.next_stack_id = u64::MAX - 1;
            } else {
                let r = g.state.delayed_sacrifices.as_ref().unwrap()[0].view.tokens[1];
                g.state.objects[r.card as usize].generation = u64::MAX;
                g.state.delayed_sacrifices.as_mut().unwrap()[0].view.tokens[1].generation =
                    u64::MAX;
                to_end(&mut g);
                replay(&mut g, Action::Pass);
            }
            let before = g.snapshot();
            let rev = g.revision();
            let actor = g.actor().unwrap();
            assert!(matches!(
                g.apply(actor, Action::Pass),
                Err(Error::CapacityExceeded(_))
            ));
            assert_eq!(g.snapshot(), before);
            assert_eq!(g.revision(), rev);
            assert_eq!(
                g.state
                    .objects
                    .iter()
                    .filter(|o| o.zone == Zone::Battlefield && o.card.is_token())
                    .count(),
                if queue { 6 } else { 3 }
            );
            assert_eq!(
                g.observe(p).delayed_sacrifices.len(),
                if queue { 2 } else { 0 }
            );
        }
    }
}
#[test]
fn partial_creation_nonland_history_overflow_restores_arena_attack_and_delayed_state() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = paid(&mut g, p, "Dalkovan Packbeasts");
        attack(&mut g, s);
        replay(&mut g, Action::Pass);
        g.state.players[p.index()].nonland_permanents_entered_this_turn = u64::MAX - 1;
        let before = g.snapshot();
        let rev = g.revision();
        assert!(matches!(
            g.apply(p.other(), Action::Pass),
            Err(Error::CapacityExceeded(_))
        ));
        assert_eq!(g.snapshot(), before);
        assert_eq!(g.revision(), rev);
        assert_eq!(g.state.attackers.len(), 1);
        assert!(g.state.delayed_sacrifices.is_none());
    }
}
#[test]
fn stadium_pays_real_sacrifice_and_damage_counts_live_creatures_after_payment_and_responses() {
    for p in [Player::P0, Player::P1] {
        for remove_warrior in [false, true] {
            let mut g = setup(p);
            let source = paid(&mut g, p, "Stadium Headliner");
            named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let target = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            g.state.objects[target as usize].growth.toughness = 20;
            attack(&mut g, source);
            settle(&mut g);
            let r = g.observe(p).delayed_sacrifices[0].tokens[0];
            replay(
                &mut g,
                Action::AnnounceActivate {
                    card: source,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            replay(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
            while g.state.announcing.is_some() {
                pick(&mut g);
            }
            assert_eq!(g.object_view(source).zone, Zone::Graveyard);
            if remove_warrior {
                g.move_object(r.card, Zone::Graveyard);
                g.state_based_actions();
            }
            settle(&mut g);
            assert_eq!(
                g.object_view(target).damage,
                if remove_warrior { 1 } else { 2 }
            );
            to_end(&mut g);
            settle(&mut g);
            assert!(g.observe(p).delayed_sacrifices.is_empty());
        }
    }
}
#[test]
fn nightblade_actual_surveil_and_vanguard_characteristic_apply_outside_battlefield() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let before = g.state.players[p.index()].library.len();
        paid(&mut g, p, "Nightblade Brigade");
        assert_eq!(g.state.players[p.index()].library.len(), before - 1);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
        assert!(
            g.state
                .objects
                .iter()
                .any(|o| o.owner == p && o.zone == Zone::Graveyard && o.card == Card::Forest)
        );
        let source = named(&mut g, p, "Zurgo's Vanguard", Zone::Hand);
        assert_eq!(g.object_view(source).power, 1);
        g.move_object(source, Zone::Battlefield);
        assert_eq!(g.object_view(source).power, 2);
        attack(&mut g, source);
        settle(&mut g);
        assert_eq!(g.object_view(source).power, 3);
        g.move_object(source, Zone::Graveyard);
        assert_eq!(g.object_view(source).power, 2);
        to_end(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(source).power, 1);
    }
}
#[test]
fn shock_brigade_requires_real_two_blockers_while_new_warrior_remains_unblocked() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = paid(&mut g, p, "Shock Brigade");
        let a = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let b = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        attack(&mut g, s);
        settle(&mut g);
        replay(&mut g, Action::Pass);
        replay(&mut g, Action::Pass);
        replay(
            &mut g,
            Action::Block {
                blocker: a,
                attacker: s,
            },
        );
        reject(&mut g, Action::FinishBlockers);
        replay(
            &mut g,
            Action::Block {
                blocker: b,
                attacker: s,
            },
        );
        replay(&mut g, Action::FinishBlockers);
        for _ in 0..20 {
            if g.state.step == Step::Main2 {
                break;
            }
            pick(&mut g);
        }
        assert_eq!(g.state.players[p.other().index()].life, 19);
        assert_eq!(g.object_view(s).zone, Zone::Graveyard);
        to_end(&mut g);
        settle(&mut g);
    }
}
