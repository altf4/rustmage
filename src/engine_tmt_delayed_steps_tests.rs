//! Independent step delays, actual paid entry, deterministic random discard.
use super::ability_timeline_tests::{checked, named, next_turn, settle, setup};
use super::tmt_departure_tests::{candidate, cast_candidate};
use super::*;
fn entry(g: &mut Game, p: Player) -> CardId {
    let id = candidate(g, p, "Casey Jones, Vigilante", Zone::Hand);
    cast_candidate(g, id, &[]);
    settle(g);
    id
}
fn upkeep(g: &mut Game, p: Player) {
    for _ in 0..200 {
        if g.state.active == p && g.state.step == Step::Upkeep {
            return;
        }
        checked(g, Action::Pass);
    }
    panic!("no upkeep");
}
#[test]
fn tmt_delayed_casey_paid_entry_draws_three_and_next_own_upkeep_discards_once() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let n = g.hand_count(p);
        let c = entry(&mut g, p);
        assert_eq!(g.hand_count(p), n + 3);
        assert_eq!((g.stats(c).power, g.stats(c).toughness), (4, 3));
        assert_eq!(g.observe(p).delayed_steps.len(), 1);
        next_turn(&mut g);
        upkeep(&mut g, p.other());
        settle(&mut g);
        assert_eq!(g.observe(p).delayed_steps.len(), 1);
        next_turn(&mut g);
        upkeep(&mut g, p);
        let before = g.hand_count(p);
        settle(&mut g);
        assert_eq!(g.hand_count(p), before - 3);
        assert!(g.observe(p).delayed_steps.is_empty());
    }
}
#[test]
fn tmt_delayed_casey_source_departure_and_reentry_keep_independent_pending_delays() {
    let p = Player::P0;
    let mut g = setup(p);
    let c = entry(&mut g, p);
    g.move_object(c, Zone::Hand);
    cast_candidate(&mut g, c, &[]);
    settle(&mut g);
    assert_eq!(g.observe(p).delayed_steps.len(), 2);
    g.move_object(c, Zone::Graveyard);
    next_turn(&mut g);
    next_turn(&mut g);
    upkeep(&mut g, p);
    let before = g.hand_count(p);
    settle(&mut g);
    assert_eq!(g.hand_count(p), before - 6);
    assert!(g.observe(p).delayed_steps.is_empty());
}
#[test]
fn tmt_delayed_casey_random_discard_handles_short_hands_and_is_owned_replayable() {
    for count in [0, 1, 2] {
        let p = Player::P0;
        let mut g = setup(p);
        entry(&mut g, p);
        let hand: Vec<_> = g
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(_, o)| o.zone == Zone::Hand && o.owner == p)
            .map(|(i, _)| i as CardId)
            .collect();
        for id in hand {
            g.move_object(id, Zone::Graveyard);
        }
        for _ in 0..count {
            named(&mut g, p, "Mountain", Zone::Hand);
        }
        next_turn(&mut g);
        next_turn(&mut g);
        upkeep(&mut g, p);
        settle(&mut g);
        assert_eq!(g.hand_count(p), 0);
        assert!(g.observe(p).delayed_steps.is_empty());
    }
}
#[test]
fn tmt_delayed_casey_countered_entry_trigger_neither_draws_nor_schedules() {
    let p = Player::P0;
    let mut g = setup(p);
    let c = candidate(&mut g, p, "Casey Jones, Vigilante", Zone::Hand);
    cast_candidate(&mut g, c, &[]);
    for _ in 0..20 {
        if g.state
            .stack
            .last()
            .is_some_and(|s| matches!(s.kind, StackKind::TriggeredAbility { .. }))
        {
            break;
        }
        checked(&mut g, Action::Pass);
    }
    let trigger = g.state.stack.last().unwrap().id;
    let mut sibling = g.clone();
    assert!(sibling.counter_stack_item(trigger, false));
    assert!(g.counter_stack_item(trigger, false));
    assert_eq!(g.snapshot(), sibling.snapshot());
    settle(&mut g);
    assert!(g.observe(p).delayed_steps.is_empty());
}

#[test]
fn tmt_delayed_step_packet_keeps_exact_link_after_source_departure_and_fires_once() {
    let p = Player::P0;
    let mut g = setup(p);
    let c = candidate(&mut g, p, "Casey Jones, Vigilante", Zone::Graveyard);
    let exiled = named(&mut g, p.other(), "Grizzly Bears", Zone::Exile);
    let mut item = g.activation_item(p, c, 0);
    item.return_link = Some(Box::new(ReturnLinkView {
        object: g.object_ref(exiled),
        zone: Zone::Exile,
        controller: p.other(),
        counter: None,
        tapped: false,
        additional: vec![],
    }));
    g.schedule_delayed_step(&item, TriggerStep::End, CastRelation::Any, 1);
    let row = g
        .state
        .roles
        .as_ref()
        .unwrap()
        .delayed_steps
        .last()
        .unwrap();
    assert_eq!(row.item.return_link, item.return_link);
    let expected = row.item.return_link.clone();
    g.begin_step(Step::End);
    assert_eq!(
        g.state.pending_triggers.last().unwrap().return_link,
        expected
    );
    assert!(g.visible_delayed_steps().is_empty());
    let count = g.state.pending_triggers.len();
    g.begin_step(Step::End);
    assert_eq!(g.state.pending_triggers.len(), count);
}

#[test]
fn tmt_delayed_casey_schedule_capacity_failure_rolls_back_draws_and_packet_registration() {
    let p = Player::P0;
    let mut g = setup(p);
    let c = candidate(&mut g, p, "Casey Jones, Vigilante", Zone::Hand);
    cast_candidate(&mut g, c, &[]);
    for _ in 0..20 {
        if g.state
            .stack
            .last()
            .is_some_and(|s| matches!(s.kind, StackKind::TriggeredAbility { .. }))
        {
            break;
        }
        checked(&mut g, Action::Pass);
    }
    checked(&mut g, Action::Pass);
    g.state.next_stack_id = u64::MAX;
    let before = g.snapshot();
    let hand = g.hand_count(p);
    super::tmt_departure_tests::reject(&mut g, Action::Pass);
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.hand_count(p), hand);
    assert!(g.visible_delayed_steps().is_empty());
}
