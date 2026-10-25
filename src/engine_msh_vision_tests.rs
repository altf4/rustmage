//! Paid per-turn modal choices, lifetime isolation, real turn/control/blink and owned replay.
use super::assurance::big_helpers::{checked, named, next_turn, settle, setup};
use super::*;
const VISION: &str = "The Vision";
#[track_caller]
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..220 {
        if stop(g) {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass
                        | Action::PutTrigger { .. }
                        | Action::PayCosts { .. }
                        | Action::FinishTargets
                        | Action::FinishAttackers
                        | Action::FinishBlockers
                )
            })
            .unwrap_or_else(|| panic!("unexpected {:?} at {:?}", g.state.pending, g.state.step));
        checked(g, a);
    }
    panic!("Vision scenario stalled {:?}", g.state.pending);
}
fn priority(g: &mut Game, p: Player) {
    if g.actor() != Some(p) {
        checked(g, Action::Pass);
    }
    assert_eq!(g.actor(), Some(p));
}
fn announce(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    priority(g, p);
    let id = named(g, p, name, Zone::Hand);
    let (colored, generic) = g.object_view(id).card.cost();
    g.state.players[p.index()].mana = colored;
    g.state.players[p.index()].mana[5] += generic;
    checked(g, Action::AnnounceCast { card: id });
    for &target in targets {
        checked(g, Action::ChooseTarget { target });
    }
    // Paying the spell can immediately announce Vision's mandatory trigger.
    // Only finish the selected spell packet, preserving the new mode decision.
    while g
        .state
        .announcing
        .as_ref()
        .is_some_and(|a| a.item.card == id && a.item.kind == StackKind::Spell)
    {
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
            .unwrap();
        checked(g, action);
    }
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    id
}
fn paid(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    let id = announce(g, p, name, targets);
    settle(g);
    id
}
fn body(p: Player) -> (Game, CardId) {
    let mut g = setup(p);
    for s in &mut g.state.players {
        s.mana = [0; 6];
    }
    let id = paid(&mut g, p, VISION, &[]);
    (g, id)
}
fn modes(g: &Game) -> Vec<u16> {
    g.legal_actions()
        .iter()
        .filter_map(|a| {
            if let Action::ChooseMode { mode } = a {
                Some(*mode)
            } else {
                None
            }
        })
        .collect()
}
fn wait_mode(g: &mut Game) {
    until(g, |g| matches!(g.state.pending, Pending::Modes { .. }));
}
fn choose(g: &mut Game, mode: u16) {
    wait_mode(g);
    checked(g, Action::ChooseMode { mode });
    settle(g);
}
fn shock(g: &mut Game, p: Player) {
    announce(g, p, "Shock", &[Target::Player(p.other())]);
}
fn reject(g: &mut Game, a: Action) {
    let before = g.snapshot();
    let revision = g.revision();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
}
fn orphan(g: Game, a: Action) -> Game {
    let before = g.snapshot();
    let actor = g.actor().unwrap();
    let revision = g.revision();
    let index = g.legal_actions().iter().position(|b| *b == a).unwrap();
    let (mut raw, mut indexed) = (g.clone(), g.clone());
    drop(g);
    raw.apply(actor, a.clone()).unwrap();
    indexed.apply_index(actor, revision, index).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    let after = raw.snapshot();
    raw.restore(&before).unwrap();
    raw.apply(actor, a).unwrap();
    assert_eq!(raw.snapshot(), after);
    indexed
}
fn main1(g: &mut Game) {
    until(g, |g| {
        g.state.step == Step::Main1
            && g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
    });
}
#[test]
fn msh_vision_paid_intrinsics_and_actual_flying_vigilant_double_strike_combat() {
    for p in [Player::P0, Player::P1] {
        let (mut g, id) = body(p);
        assert_eq!((g.stats(id).power, g.stats(id).toughness), (2, 5));
        assert!(g.object_view(id).card.is_artifact());
        assert!(
            g.keywords(id)
                .contains(Keywords::FLYING.union(Keywords::VIGILANCE))
        );
        next_turn(&mut g);
        next_turn(&mut g);
        main1(&mut g);
        let bear = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        shock(&mut g, p);
        choose(&mut g, 0);
        assert!(g.keywords(id).contains(Keywords::DOUBLE_STRIKE));
        until(&mut g, |g| g.state.pending == Pending::Attackers);
        checked(&mut g, Action::Attack { card: id });
        checked(&mut g, Action::FinishAttackers);
        assert!(!g.object_view(id).tapped);
        until(&mut g, |g| g.state.pending == Pending::Blockers);
        reject(
            &mut g,
            Action::Block {
                blocker: bear,
                attacker: id,
            },
        );
        until(&mut g, |g| {
            g.state.step == Step::Main2 && g.state.pending == Pending::Priority
        });
        assert_eq!(g.state.players[p.other().index()].life, 14);
    }
}
#[test]
fn msh_vision_three_distinct_modes_then_exhausted_trigger_drops_without_stalling() {
    for p in [Player::P0, Player::P1] {
        let (mut g, id) = body(p);
        for mode in [0, 1, 2] {
            shock(&mut g, p);
            wait_mode(&mut g);
            assert_eq!(modes(&g), (mode..3).collect::<Vec<_>>());
            checked(&mut g, Action::ChooseMode { mode });
            settle(&mut g);
        }
        assert!(
            g.keywords(id)
                .contains(Keywords::DOUBLE_STRIKE.union(Keywords::INDESTRUCTIBLE))
        );
        let row = &g.observe(p).modal_history[0];
        assert_eq!(row.modes, vec![0, 1, 2]);
        assert_eq!(row.turn, Some(g.state.turn));
        shock(&mut g, p);
        settle(&mut g);
        assert!(g.state.stack.is_empty());
        assert_eq!(g.state.players[p.other().index()].life, 12);
    }
}
#[test]
fn msh_vision_used_mode_and_out_of_range_failure_are_atomic() {
    for p in [Player::P0, Player::P1] {
        let (mut g, _) = body(p);
        shock(&mut g, p);
        choose(&mut g, 1);
        shock(&mut g, p);
        wait_mode(&mut g);
        reject(&mut g, Action::ChooseMode { mode: 1 });
        reject(&mut g, Action::ChooseMode { mode: 3 });
        assert_eq!(modes(&g), vec![0, 2]);
        choose(&mut g, 2);
    }
}
#[test]
fn msh_vision_resets_on_opponent_turn_and_keyword_grants_expire() {
    for p in [Player::P0, Player::P1] {
        let (mut g, id) = body(p);
        shock(&mut g, p);
        choose(&mut g, 0);
        next_turn(&mut g);
        main1(&mut g);
        assert_eq!(g.state.active, p.other());
        assert!(!g.keywords(id).contains(Keywords::DOUBLE_STRIKE));
        assert!(g.state.modal_history.is_none());
        shock(&mut g, p);
        wait_mode(&mut g);
        assert_eq!(modes(&g), vec![0, 1, 2]);
        choose(&mut g, 0);
        assert!(g.keywords(id).contains(Keywords::DOUBLE_STRIKE));
    }
}
#[test]
fn msh_vision_density_survives_actual_destroy_then_expires_next_turn() {
    for p in [Player::P0, Player::P1] {
        let (mut g, id) = body(p);
        shock(&mut g, p);
        choose(&mut g, 1);
        paid(&mut g, p.other(), "Murder", &[Target::Card(id)]);
        assert_eq!(g.object_view(id).zone, Zone::Battlefield);
        next_turn(&mut g);
        main1(&mut g);
        assert!(!g.keywords(id).contains(Keywords::INDESTRUCTIBLE));
        paid(&mut g, p.other(), "Murder", &[Target::Card(id)]);
        assert_eq!(g.object_view(id).zone, Zone::Graveyard);
    }
}
#[test]
fn msh_vision_draw_uses_captured_controller_after_source_dies() {
    for p in [Player::P0, Player::P1] {
        let (mut g, id) = body(p);
        shock(&mut g, p);
        wait_mode(&mut g);
        checked(&mut g, Action::ChooseMode { mode: 2 });
        let before = g.hand_count(p);
        paid(&mut g, p.other(), "Murder", &[Target::Card(id)]);
        assert_eq!(g.object_view(id).zone, Zone::Graveyard);
        assert_eq!(g.hand_count(p), before + 1);
        assert!(g.state.modal_history.is_none());
    }
}
#[test]
fn msh_vision_creatures_and_opponents_spells_are_quiet_but_artifact_spell_triggers() {
    for p in [Player::P0, Player::P1] {
        let (mut g, _) = body(p);
        paid(&mut g, p, "Llanowar Elves", &[]);
        assert!(g.state.modal_history.is_none());
        paid(&mut g, p.other(), "Shock", &[Target::Player(p)]);
        assert!(g.state.modal_history.is_none());
        announce(&mut g, p, "Hawkeye's Bow", &[]);
        wait_mode(&mut g);
        assert_eq!(modes(&g), vec![0, 1, 2]);
        choose(&mut g, 2);
    }
}
#[test]
fn msh_vision_mode_history_follows_control_and_new_controller_gets_remaining_choices() {
    for p in [Player::P0, Player::P1] {
        let (mut g, id) = body(p);
        next_turn(&mut g);
        main1(&mut g);
        shock(&mut g, p);
        choose(&mut g, 0);
        let old = g.object_ref(id);
        paid(
            &mut g,
            p.other(),
            "Involuntary Employment",
            &[Target::Card(id)],
        );
        assert_eq!(g.object_ref(id), old);
        assert_eq!(g.object_view(id).controller, p.other());
        shock(&mut g, p.other());
        wait_mode(&mut g);
        assert_eq!(modes(&g), vec![1, 2]);
        choose(&mut g, 2);
    }
}
#[test]
fn msh_vision_suppression_removes_cast_observer_without_erasing_prior_mode() {
    for p in [Player::P0, Player::P1] {
        let (mut g, id) = body(p);
        announce(&mut g, p, "Witness Protection", &[Target::Card(id)]);
        choose(&mut g, 0);
        assert!(
            !g.keywords(id)
                .contains(Keywords::FLYING.union(Keywords::DOUBLE_STRIKE))
        );
        shock(&mut g, p);
        settle(&mut g);
        assert_eq!(g.observe(p).modal_history[0].modes, vec![0]);
        let aura = named(&mut g, p, "Witness Protection", Zone::Graveyard);
        // Find the actually attached paid Aura; the graveyard namesake cannot substitute for it.
        let attached = (0..g.state.objects.len() as CardId)
            .find(|&c| {
                g.object_view(c).zone == Zone::Battlefield
                    && g.state.objects[c as usize].attached_to == Some(g.object_ref(id))
            })
            .unwrap();
        assert_ne!(attached, aura);
        paid(&mut g, p, "Disenchant", &[Target::Card(attached)]);
        shock(&mut g, p);
        wait_mode(&mut g);
        assert_eq!(modes(&g), vec![1, 2]);
        choose(&mut g, 1);
    }
}
#[test]
fn msh_vision_paid_end_step_blink_gives_new_incarnation_fresh_modes_same_turn() {
    for p in [Player::P0, Player::P1] {
        let (mut g, id) = body(p);
        shock(&mut g, p);
        choose(&mut g, 0);
        let old = g.object_ref(id);
        let turn = g.state.turn;
        let car = announce(&mut g, p, "S.H.I.E.L.D. Flying Car", &[]);
        wait_mode(&mut g);
        checked(&mut g, Action::ChooseMode { mode: 2 });
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::Targets { .. })
                && g.state
                    .announcing
                    .as_ref()
                    .is_some_and(|a| a.item.card == car)
        });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(id),
            },
        );
        settle(&mut g);
        assert_eq!(g.object_view(id).zone, Zone::Exile);
        until(&mut g, |g| {
            g.object_view(id).zone == Zone::Battlefield
                && g.state.pending == Pending::Priority
                && g.state.stack.is_empty()
        });
        assert_eq!(g.state.turn, turn);
        assert_ne!(g.object_ref(id), old);
        shock(&mut g, p);
        wait_mode(&mut g);
        assert_eq!(modes(&g), vec![0, 1, 2]);
        choose(&mut g, 0);
    }
}
#[test]
fn msh_vision_mode_choice_survives_orphan_raw_index_restore_and_hidden_hand_changes() {
    for p in [Player::P0, Player::P1] {
        let (mut g, _) = body(p);
        shock(&mut g, p);
        wait_mode(&mut g);
        let secret = named(&mut g, p, "Murder", Zone::Hand);
        let mut changed = g.clone();
        changed.state.objects[secret as usize].card = Card::from_name("Divination").unwrap();
        assert_eq!(
            serde_json::to_value(g.observe(p.other())).unwrap(),
            serde_json::to_value(changed.observe(p.other())).unwrap()
        );
        assert_eq!(
            g.observe(p).modal_history,
            g.observe(p.other()).modal_history
        );
        let mut g = orphan(g, Action::ChooseMode { mode: 2 });
        settle(&mut g);
        shock(&mut g, p);
        wait_mode(&mut g);
        assert_eq!(modes(&g), vec![0, 1]);
        choose(&mut g, 0);
    }
}
#[test]
fn msh_vision_modes_reset_after_paid_time_stop_skips_the_rest_of_turn() {
    for p in [Player::P0, Player::P1] {
        let (mut g, _) = body(p);
        shock(&mut g, p);
        choose(&mut g, 0);
        let turn = g.state.turn;
        paid(&mut g, p.other(), "Time Stop", &[]);
        assert!(g.state.turn > turn);
        assert!(g.state.modal_history.is_none());
        shock(&mut g, p);
        wait_mode(&mut g);
        assert_eq!(modes(&g), vec![0, 1, 2]);
        choose(&mut g, 0);
    }
}
#[test]
fn msh_vision_lifetime_demonic_pact_choices_do_not_reset_on_turn_change() {
    for p in [Player::P0, Player::P1] {
        let (mut g, _) = body(p);
        let pact = paid(&mut g, p, "Demonic Pact", &[]);
        next_turn(&mut g);
        next_turn(&mut g);
        wait_mode(&mut g);
        checked(&mut g, Action::ChooseMode { mode: 2 });
        settle(&mut g);
        assert!(
            g.observe(p)
                .modal_history
                .iter()
                .any(|r| r.object == g.object_ref(pact) && r.turn.is_none() && r.modes == vec![2])
        );
        next_turn(&mut g);
        next_turn(&mut g);
        wait_mode(&mut g);
        assert_eq!(modes(&g), vec![0, 1, 3]);
        checked(&mut g, Action::ChooseMode { mode: 1 });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        settle(&mut g);
    }
}
#[test]
fn msh_vision_selected_trigger_copy_repeats_effect_without_consuming_extra_mode() {
    for p in [Player::P0, Player::P1] {
        let (mut g, id) = body(p);
        shock(&mut g, p);
        wait_mode(&mut g);
        checked(&mut g, Action::ChooseMode { mode: 2 });
        let mut copy = g.state.stack.iter().find(|s| s.card == id).unwrap().clone();
        copy.id = g.next_stack_id();
        // Bounded packet-copy fixture: no new playable copy-ability consumer is admitted.
        g.state.stack.push(copy);
        let before = g.hand_count(p);
        settle(&mut g);
        assert_eq!(g.hand_count(p), before + 2);
        shock(&mut g, p);
        wait_mode(&mut g);
        assert_eq!(modes(&g), vec![0, 1]);
        choose(&mut g, 0);
    }
}
