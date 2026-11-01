//! Paid parity, triggered modes and exact-incarnation history.
use super::*;
fn until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..96 {
        if ready(g) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("Gollum choice budget {:?}", g.state.pending)
}
fn finish(g: &mut Game) {
    until(g, |g| {
        g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.ordering_triggers.is_empty()
            && g.state.resolving.is_none()
    });
}
fn enter(g: &mut Game, p: Player, parity: ManaValueParity) -> CardId {
    g.state.active = p;
    g.state.priority = p;
    g.state.step = Step::Main1;
    let id = put(g, p, "Gollum, Riddle Master", Zone::Hand);
    g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 1];
    cast(g, id, None);
    until(g, |g| matches!(g.state.pending, Pending::EntryValue { .. }));
    assert_eq!(g.actor(), Some(p));
    let value = EntryChoiceValue::ManaValueParity(parity);
    let before = g.snapshot();
    assert!(
        g.apply(p.other(), Action::ChooseEntryValue { value })
            .is_err()
    );
    assert_eq!(g.snapshot(), before);
    assert!(
        g.observe(p.other())
            .entry_value
            .as_ref()
            .is_none_or(|view| view.selected.is_empty())
    );
    contracts::branch_contract(g.clone());
    act(
        g,
        Action::ChooseEntryValue {
            value: EntryChoiceValue::ManaValueParity(parity),
        },
    );
    finish(g);
    assert_eq!(
        g.entry_value(g.object_ref(id)),
        Some(EntryChoiceValue::ManaValueParity(parity))
    );
    assert_eq!(stats(g, id), (3, 1));
    id
}
fn xcast(g: &mut Game, p: Player, x: u32) -> CardId {
    g.state.active = p;
    g.state.priority = p;
    g.state.step = Step::Main1;
    let id = put(g, p, "Mind Spring", Zone::Hand);
    g.state.players[p.index()].mana = [0, 2, 0, 0, 0, x];
    act(g, Action::AnnounceCast { card: id });
    act(g, Action::ChooseX { value: x });
    pay(g);
    id
}
fn modes(g: &mut Game) {
    until(g, |g| matches!(g.state.pending, Pending::Modes { .. }));
}
#[test]
fn hob_riddle_master_actual_parity_x_and_lifetime_modes() {
    for p in [Player::P0, Player::P1] {
        for parity in [ManaValueParity::Odd, ManaValueParity::Even] {
            let mut g = scenario(p);
            g.state.players[p.index()].life = 100;
            g.state.players[p.other().index()].life = 100;
            let id = enter(&mut g, p, parity);
            let x = if parity == ManaValueParity::Odd { 1 } else { 0 };
            for mode in 0..3 {
                xcast(&mut g, p.other(), x);
                modes(&mut g);
                assert_eq!(g.actor(), Some(p));
                for prev in 0..mode {
                    reject(&mut g, Action::ChooseMode { mode: prev });
                }
                contracts::branch_contract(g.clone());
                act(&mut g, Action::ChooseMode { mode });
                finish(&mut g);
            }
            assert_eq!(g.counter_count(id, CounterKind::PlusOne), 1);
            assert_eq!(g.state.players[p.index()].life, 102);
            assert_eq!(g.state.players[p.other().index()].life, 98);
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
            xcast(&mut g, p.other(), x);
            finish(&mut g);
            assert_eq!(g.counter_count(id, CounterKind::PlusOne), 1);
            let old = g.object_ref(id);
            g.move_object(id, Zone::Hand);
            g.state.active = p;
            g.state.priority = p;
            g.state.step = Step::Main1;
            g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 1];
            cast(&mut g, id, None);
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::EntryValue { .. })
            });
            act(
                &mut g,
                Action::ChooseEntryValue {
                    value: EntryChoiceValue::ManaValueParity(parity),
                },
            );
            finish(&mut g);
            assert_ne!(g.object_ref(id), old);
            xcast(&mut g, p.other(), x);
            modes(&mut g);
            assert!(g.legal_actions().contains(&Action::ChooseMode { mode: 0 }));
            act(&mut g, Action::ChooseMode { mode: 0 });
            finish(&mut g);
            assert_eq!(g.counter_count(id, CounterKind::PlusOne), 1);
        }
    }
}
#[test]
fn hob_riddle_master_zero_is_even_opponent_only_and_copy_is_not_cast() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let id = enter(&mut g, p, ManaValueParity::Even);
        g.state.active = p.other();
        g.state.priority = p.other();
        g.state.step = Step::Main1;
        let zero = put(&mut g, p.other(), "Mox Jasper", Zone::Hand);
        cast(&mut g, zero, None);
        modes(&mut g);
        act(&mut g, Action::ChooseMode { mode: 0 });
        finish(&mut g);
        assert_eq!(g.counter_count(id, CounterKind::PlusOne), 1);
        g.state.active = p;
        g.state.priority = p;
        g.state.step = Step::Main1;
        let own = put(&mut g, p, "Mind Spring", Zone::Hand);
        g.state.players[p.index()].mana = [0, 2, 0, 0, 0, 0];
        act(&mut g, Action::AnnounceCast { card: own });
        act(&mut g, Action::ChooseX { value: 0 });
        pay(&mut g);
        finish(&mut g);
        let opponent_spell = xcast(&mut g, p.other(), 0);
        modes(&mut g);
        act(&mut g, Action::ChooseMode { mode: 1 });
        let template = g
            .state
            .stack
            .iter()
            .find(|item| item.source.unwrap().id == opponent_spell && item.kind == StackKind::Spell)
            .unwrap()
            .clone();
        let before = g.state.pending_triggers.len();
        let mut copy_effect = template.clone();
        copy_effect.copy_event = Some(Box::new(CopyEvent {
            spell: template,
            amount: 1,
        }));
        let stack_before = g.state.stack.len();
        assert!(
            !g.copy_captured_spell(&copy_effect),
            "untargeted copy needs no retarget pause"
        );
        assert_eq!(g.state.stack.len(), stack_before + 1);
        assert!(g.state.stack.last().unwrap().is_copy);
        assert_eq!(g.state.pending_triggers.len(), before);
        finish(&mut g);
        assert_eq!(g.counter_count(id, CounterKind::PlusOne), 1);
    }
}

#[test]
fn hob_riddle_master_adventure_uses_cast_face_not_front_face() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let id = enter(&mut g, p, ManaValueParity::Odd);
        let caster = p.other();
        g.state.active = caster;
        g.state.priority = caster;
        g.state.step = Step::Main1;
        let card = put(
            &mut g,
            caster,
            "An Unexpected Party // At the Door",
            Zone::Hand,
        );
        g.state.players[caster.index()].mana = [1, 0, 0, 0, 0, 2];
        act(&mut g, Action::AnnounceAdventure { card });
        act(&mut g, Action::ChooseX { value: 0 });
        pay(&mut g);
        assert_eq!(g.state.stack.last().unwrap().mana_value(), 3);
        modes(&mut g);
        act(&mut g, Action::ChooseMode { mode: 0 });
        finish(&mut g);
        assert_eq!(g.counter_count(id, CounterKind::PlusOne), 1);
    }
}

#[test]
fn hob_riddle_master_trigger_freezes_controller_and_counter_replacement() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let id = enter(&mut g, p, ManaValueParity::Even);
        put(&mut g, p, "Doubling Season", Zone::Battlefield);
        xcast(&mut g, p.other(), 1);
        finish(&mut g);
        assert!(
            g.state
                .modal_history
                .as_ref()
                .is_none_or(|rows| rows.is_empty())
        );
        xcast(&mut g, p.other(), 0);
        modes(&mut g);
        act(&mut g, Action::ChooseMode { mode: 0 });
        finish(&mut g);
        assert_eq!(g.counter_count(id, CounterKind::PlusOne), 2);
        xcast(&mut g, p.other(), 0);
        modes(&mut g);
        g.change_controller(g.object_ref(id), p.other());
        act(&mut g, Action::ChooseMode { mode: 1 });
        finish(&mut g);
        assert_eq!(g.state.players[p.index()].life, 22);
        assert_eq!(g.state.players[p.other().index()].life, 18);
    }
}
#[test]
fn hob_riddle_master_queued_counter_never_affects_returned_incarnation() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let id = enter(&mut g, p, ManaValueParity::Odd);
        xcast(&mut g, p.other(), 1);
        modes(&mut g);
        act(&mut g, Action::ChooseMode { mode: 0 });
        g.move_object(id, Zone::Exile);
        g.move_object(id, Zone::Battlefield);
        finish(&mut g);
        assert_eq!(g.counter_count(id, CounterKind::PlusOne), 0);
    }
}

#[test]
fn hob_riddle_master_queued_modes_are_reserved_before_resolution() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let id = enter(&mut g, p, ManaValueParity::Odd);
        let caster = p.other();
        g.state.active = caster;
        g.state.priority = caster;
        g.state.step = Step::Main1;
        let first = put(&mut g, caster, "Shock", Zone::Hand);
        g.state.players[caster.index()].mana = [0, 0, 0, 2, 0, 0];
        cast(&mut g, first, Some(Target::Player(p)));
        modes(&mut g);
        act(&mut g, Action::ChooseMode { mode: 0 });
        g.state.priority = caster;
        let second = put(&mut g, caster, "Shock", Zone::Hand);
        cast(&mut g, second, Some(Target::Player(p)));
        modes(&mut g);
        reject(&mut g, Action::ChooseMode { mode: 0 });
        act(&mut g, Action::ChooseMode { mode: 1 });
        finish(&mut g);
        assert_eq!(g.counter_count(id, CounterKind::PlusOne), 1);
        assert_eq!(g.state.players[p.index()].life, 18);
        assert_eq!(g.state.players[caster.index()].life, 18);
    }
}
