//! Plan thresholds are historical placement events, followed by an actual sacrifice.
use super::assurance::big_helpers::{cast, checked, named, settle, setup};
use super::*;

fn plan(g: &mut Game, p: Player, zone: Zone) -> CardId {
    named(g, p, "Death to Our Enemies", zone)
}
fn place(g: &mut Game, id: CardId, amount: u64) {
    g.effect_counter(
        id,
        CounterKind::Plan,
        amount,
        replacements::ResumeMode::Pending,
    );
}
fn thresholds(g: &Game, id: CardId) -> usize {
    g.state
        .pending_triggers
        .iter()
        .filter(|item| item.card == id && item.kind == (StackKind::TriggeredAbility { ability: 1 }))
        .count()
}
fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let revision = g.revision();
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
}
fn next(g: &mut Game) {
    let action = if g.state.pending == Pending::Priority {
        Action::Pass
    } else {
        g.legal_actions()
            .into_iter()
            .find(|a| !matches!(a, Action::Concede))
            .unwrap()
    };
    checked(g, action);
}
fn await_payoff(g: &mut Game) {
    g.prepare_triggers();
    for _ in 0..30 {
        if g.state
            .announcing
            .as_ref()
            .is_some_and(|a| a.item.kind == (StackKind::TriggeredAbility { ability: 2 }))
        {
            return;
        }
        next(g);
    }
    panic!("no sacrifice payoff at {:?}", g.state.pending);
}
fn split(g: &mut Game, first: Target, second: Target, amount: u16) {
    reject(g, Action::FinishTargets);
    checked(g, Action::ChooseTarget { target: first });
    checked(g, Action::ChooseTarget { target: second });
    if g.legal_actions().contains(&Action::FinishTargets) {
        checked(g, Action::FinishTargets);
    }
    reject(g, Action::AllocateTargetAmount { amount: 0 });
    reject(g, Action::AllocateTargetAmount { amount: 7 });
    checked(g, Action::AllocateTargetAmount { amount });
    reject(
        g,
        Action::AllocateTargetAmount {
            amount: 7 - amount - 1,
        },
    );
    checked(g, Action::AllocateTargetAmount { amount: 7 - amount });
}

#[test]
fn plan_whole_paid_card_tapped_treasure_historical_threshold_and_split_damage() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = plan(&mut g, p, Zone::Hand);
        cast(&mut g, id, None, &[]);
        settle(&mut g);
        assert_eq!(g.counter_count(id, CounterKind::Plan), 0);
        let creature = named(&mut g, p, "Aegis Turtle", Zone::Hand);
        cast(&mut g, creature, None, &[]);
        settle(&mut g);
        assert_eq!(g.counter_count(id, CounterKind::Plan), 0);
        let opponent_spell = named(&mut g, p.other(), "Giant Growth", Zone::Hand);
        checked(&mut g, Action::Pass);
        cast(&mut g, opponent_spell, None, &[Target::Card(creature)]);
        settle(&mut g);
        assert_eq!(g.counter_count(id, CounterKind::Plan), 0);
        for count in 1..=4 {
            let spell = named(&mut g, p, "Giant Growth", Zone::Hand);
            cast(&mut g, spell, None, &[Target::Card(creature)]);
            if count < 4 {
                settle(&mut g);
            } else {
                await_payoff(&mut g);
            }
            let treasures: Vec<_> = g
                .state
                .objects
                .iter()
                .filter(|o| {
                    o.zone == Zone::Battlefield
                        && o.controller == p
                        && o.card == Card::from_name("@token/b60f6cd3b44b22bd289fe4e2").unwrap()
                })
                .collect();
            assert_eq!(treasures.len(), count);
            assert!(treasures.iter().all(|o| o.tapped));
            if count < 4 {
                assert_eq!(g.counter_count(id, CounterKind::Plan), count as u64);
            }
        }
        assert_eq!(g.object_view(id).zone, Zone::Graveyard);
        assert!(g.state.announcing.as_ref().unwrap().item.targets.is_empty());
        split(&mut g, Target::Player(p.other()), Target::Card(creature), 5);
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 15);
        assert_eq!(g.object_view(creature).damage, 2);
        assert_eq!(g.counter_count(id, CounterKind::Plan), 0);
    }
}

#[test]
fn plan_crossing_skips_exact_value_recrosses_and_never_rechecks_live_count() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = plan(&mut g, p, Zone::Battlefield);
        place(&mut g, id, 3);
        assert_eq!(thresholds(&g, id), 0);
        place(&mut g, id, 2);
        assert_eq!(g.counter_count(id, CounterKind::Plan), 5);
        assert_eq!(thresholds(&g, id), 1);
        place(&mut g, id, 1);
        assert_eq!(thresholds(&g, id), 1);
        g.set_counters(id, Counters::of(CounterKind::Plan, 2));
        place(&mut g, id, 2);
        assert_eq!(thresholds(&g, id), 2);
        g.set_counters(id, Counters::default());
        await_payoff(&mut g);
        assert_eq!(g.object_view(id).zone, Zone::Graveyard);
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        checked(&mut g, Action::FinishTargets);
        settle(&mut g);
        // One packet sacrifices successfully. The other cannot sacrifice it again.
        assert_eq!(g.state.players[p.other().index()].life, 13);
    }
}

#[test]
fn plan_entry_batch_with_counters_triggers_once_per_source_after_simultaneous_commit() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let a = plan(&mut g, p, Zone::Hand);
        let b = plan(&mut g, p, Zone::Graveyard);
        let mut first = g.entry_move(a, Zone::Battlefield, false);
        first.effect_counter = Some((CounterKind::Plan, 4));
        let mut second = g.entry_move(b, Zone::Battlefield, false);
        second.effect_counter = Some((CounterKind::Plan, 5));
        g.stage_entries(vec![first, second], None);
        g.advance_entries();
        assert!(g.on_battlefield(a) && g.on_battlefield(b));
        assert_eq!(g.counter_count(a, CounterKind::Plan), 4);
        assert_eq!(g.counter_count(b, CounterKind::Plan), 5);
        assert_eq!((thresholds(&g, a), thresholds(&g, b)), (1, 1));
        assert_eq!(g.observe(p).objects, g.observe(p.other()).objects);
        let before = g.snapshot();
        let branch = {
            let parent = g.clone();
            parent.clone()
        };
        assert_eq!(branch.snapshot(), before);
        g.restore(&before).unwrap();
        assert_eq!((thresholds(&g, a), thresholds(&g, b)), (1, 1));
    }
}

#[test]
fn plan_counter_replacement_uses_actual_added_amount_and_final_entry_count() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        named(&mut g, p, "Doubling Season", Zone::Battlefield);
        let id = plan(&mut g, p, Zone::Battlefield);
        place(&mut g, id, 2);
        g.advance_replacements(false);
        assert_eq!(g.counter_count(id, CounterKind::Plan), 4);
        assert_eq!(thresholds(&g, id), 1);
        let entered = plan(&mut g, p, Zone::Hand);
        let mut movement = g.entry_move(entered, Zone::Battlefield, false);
        movement.effect_counter = Some((CounterKind::Plan, 3));
        g.stage_entries(vec![movement], None);
        g.advance_entries();
        g.advance_replacements(false);
        g.advance_entries();
        assert_eq!(g.counter_count(entered, CounterKind::Plan), 6);
        assert_eq!(thresholds(&g, entered), 1);
    }
}

#[test]
fn plan_failed_exact_sacrifice_has_no_reflexive_after_departure_blink_or_control_change() {
    for p in [Player::P0, Player::P1] {
        for change in 0..3 {
            let mut g = setup(p);
            let id = plan(&mut g, p, Zone::Battlefield);
            place(&mut g, id, 4);
            g.prepare_triggers();
            match change {
                0 => {
                    g.move_object(id, Zone::Graveyard);
                }
                1 => {
                    g.move_object(id, Zone::Exile);
                    g.move_object(id, Zone::Battlefield);
                }
                _ => {
                    g.state.objects[id as usize].controller = p.other();
                }
            }
            settle(&mut g);
            assert_eq!(g.state.players[p.other().index()].life, 20);
            assert!(g.state.stack.is_empty());
            assert!(g.state.announcing.is_none());
            if change != 0 {
                assert!(g.on_battlefield(id));
            }
        }
    }
}

#[test]
fn plan_sacrifice_replaced_by_exile_still_creates_one_target_payoff() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        named(&mut g, p, "Rest in Peace", Zone::Battlefield);
        let id = plan(&mut g, p, Zone::Battlefield);
        place(&mut g, id, 4);
        await_payoff(&mut g);
        assert_eq!(g.object_view(id).zone, Zone::Exile);
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        checked(&mut g, Action::FinishTargets);
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 13);
    }
}

#[test]
fn plan_divided_damage_keeps_shares_when_a_target_is_invalid_and_fizzles_if_all_are() {
    for p in [Player::P0, Player::P1] {
        for all_invalid in [false, true] {
            let mut g = setup(p);
            let id = plan(&mut g, p, Zone::Battlefield);
            let a = named(&mut g, p.other(), "Aegis Turtle", Zone::Battlefield);
            let b = named(&mut g, p.other(), "Aegis Turtle", Zone::Battlefield);
            place(&mut g, id, 4);
            await_payoff(&mut g);
            split(&mut g, Target::Card(a), Target::Card(b), 5);
            g.move_object(a, Zone::Hand);
            if all_invalid {
                g.move_object(b, Zone::Hand);
            }
            settle(&mut g);
            assert_eq!(g.object_view(b).damage, if all_invalid { 0 } else { 2 });
            assert_eq!(g.state.players[p.other().index()].life, 20);
        }
    }
}

#[test]
fn plan_and_invasion_sparse_counters_roundtrip_clear_and_branch_without_hot_mutation() {
    let mut g = setup(Player::P0);
    let id = plan(&mut g, Player::P0, Zone::Battlefield);
    let cold = Counters::of(CounterKind::Plan, 3).with(CounterKind::Invasion, 7);
    let serialized = serde_json::to_string(&cold).unwrap();
    assert_eq!(serde_json::from_str::<Counters>(&serialized).unwrap(), cold);
    let parent = g.clone();
    g.set_counters(id, cold.clone());
    assert_eq!(g.counter_count(id, CounterKind::Plan), 3);
    assert_eq!(g.counter_count(id, CounterKind::Invasion), 7);
    assert_eq!(parent.counter_count(id, CounterKind::Plan), 0);
    assert!(g.state.objects.ptr_eq(&parent.state.objects));
    assert_eq!(g.object_view(id).counters, cold);
    let snapshot = g.snapshot();
    let bytes = g.state_bytes();
    let mut branch = {
        let cloned = g.clone();
        cloned.clone()
    };
    branch.move_object(id, Zone::Exile);
    assert_eq!(branch.counter_count(id, CounterKind::Plan), 0);
    assert_eq!(branch.counter_count(id, CounterKind::Invasion), 0);
    assert_eq!(g.state_bytes(), bytes);
    branch.restore(&snapshot).unwrap();
    assert_eq!(branch.snapshot(), snapshot);
    g.set_counters(id, Counters::default());
    assert!(g.state.named_counters.is_none());
}

#[test]
fn plan_overflow_rolls_back_treasure_creation_and_pending_placement_transaction() {
    let p = Player::P0;
    let mut g = setup(p);
    let id = plan(&mut g, p, Zone::Battlefield);
    g.set_counters(id, Counters::of(CounterKind::Plan, u64::MAX));
    let creature = named(&mut g, p, "Aegis Turtle", Zone::Battlefield);
    let spell = named(&mut g, p, "Giant Growth", Zone::Hand);
    cast(&mut g, spell, None, &[Target::Card(creature)]);
    for _ in 0..8 {
        let before = g.snapshot();
        let revision = g.revision();
        match g.apply(g.actor().unwrap(), Action::Pass) {
            Ok(()) => (),
            Err(Error::CapacityExceeded(_)) => {
                assert_eq!(g.snapshot(), before);
                assert_eq!(g.revision(), revision);
                assert_eq!(g.counter_count(id, CounterKind::Plan), u64::MAX);
                assert!(!g.state.objects.iter().any(|o| o.zone == Zone::Battlefield
                    && o.card == Card::from_name("@token/b60f6cd3b44b22bd289fe4e2").unwrap()));
                return;
            }
            other => panic!("unexpected {other:?}"),
        }
    }
    panic!("counter overflow did not fail transactionally");
}

#[test]
fn plan_reflexive_damage_uses_departure_lki_gained_after_threshold_fired() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = plan(&mut g, p, Zone::Battlefield);
        place(&mut g, id, 4);
        g.prepare_triggers();
        // A characteristic fixture isolates the timing contract: the keyword
        // appears after the threshold packet was captured, before sacrifice.
        g.state.objects[id as usize].gained_keywords = Keywords::LIFELINK;
        await_payoff(&mut g);
        assert_eq!(g.object_view(id).zone, Zone::Graveyard);
        assert!(
            g.state
                .announcing
                .as_ref()
                .unwrap()
                .item
                .source_keywords
                .contains(Keywords::LIFELINK)
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        checked(&mut g, Action::FinishTargets);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 27);
        assert_eq!(g.state.players[p.other().index()].life, 13);
    }
}

#[test]
fn plan_queued_cast_trigger_creates_treasure_after_source_departure_without_rebinding_counters() {
    for p in [Player::P0, Player::P1] {
        for blink in [false, true] {
            let mut g = setup(p);
            let id = plan(&mut g, p, Zone::Battlefield);
            let creature = named(&mut g, p, "Aegis Turtle", Zone::Battlefield);
            let spell = named(&mut g, p, "Giant Growth", Zone::Hand);
            cast(&mut g, spell, None, &[Target::Card(creature)]);
            let old = g.object_ref(id);
            g.move_object(id, Zone::Graveyard);
            if blink {
                g.move_object(id, Zone::Battlefield);
                g.state.objects[id as usize].controller = p.other();
            }
            settle(&mut g);
            assert_ne!(g.object_ref(id), old);
            assert_eq!(g.counter_count(id, CounterKind::Plan), 0);
            let treasures: Vec<_> = g
                .state
                .objects
                .iter()
                .filter(|o| {
                    o.zone == Zone::Battlefield
                        && o.card == Card::from_name("@token/b60f6cd3b44b22bd289fe4e2").unwrap()
                })
                .collect();
            assert_eq!(treasures.len(), 1);
            assert_eq!(treasures[0].controller, p);
            assert!(treasures[0].tapped);
        }
    }
}
