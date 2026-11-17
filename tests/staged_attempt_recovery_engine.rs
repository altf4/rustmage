//! Concrete failed payment sequences have owned, reversible boundaries.
mod support;
use rustmage as _;
use rustmage_core::{cards::authored::CardRules, cards::*, engine::*};
use support::*;

const GREEN: Mana = [0, 0, 0, 0, 1, 0];
const FILTER: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            generic: 2,
            mana_component: true,
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::AddMana(ManaProduction::Fixed(GREEN))],
        sorcery: false,
        once_per_turn: false,
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
    }],
    ..CardRules::EMPTY
};

fn zero_pool(g: &mut Game, p: Player) {
    g.state.players[p.index()].mana = [0; 6];
}

fn close_and_submit(g: &mut Game) {
    act(g, Action::EndManaWindow);
    act(g, Action::SubmitAttempt);
}

#[test]
fn failed_concrete_cast_can_retry_and_old_indices_stay_stale() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        zero_pool(&mut g, p);
        let spell = put(&mut g, p, "Bear Cub", Zone::Hand);
        let first = put(&mut g, p, "Forest", Zone::Battlefield);
        let second = put(&mut g, p, "Forest", Zone::Battlefield);
        act(&mut g, Action::AnnounceCast { card: spell });
        let first_attempt = g.attempt_view(p).unwrap().attempt_id;
        assert!(g.attempt_view(p.other()).is_none());
        let revision = g.revision();
        let old_index = g
            .legal_actions()
            .iter()
            .position(|a| *a == Action::EndManaWindow)
            .unwrap();
        close_and_submit(&mut g);
        assert_eq!(g.object_view(spell).zone, Zone::Hand);
        assert!(g.state.announcing.is_none());
        assert_eq!(
            g.last_attempt_outcome_view(p).unwrap().outcome,
            AttemptOutcome::Failed
        );
        assert_eq!(
            g.apply_index(p, revision, old_index),
            Err(Error::StaleRevision)
        );
        assert!(
            g.observe(p.other())
                .objects
                .iter()
                .any(|o| o.reference.card == spell),
            "a revealed cast cannot erase the opponent's knowledge on failure"
        );
        act(&mut g, Action::AnnounceCast { card: spell });
        assert!(g.attempt_view(p).unwrap().attempt_id > first_attempt);
        act(&mut g, Action::TapMana { card: first });
        act(&mut g, Action::TapMana { card: second });
        act(&mut g, Action::EndManaWindow);
        assert!(!g.legal_actions().contains(&Action::TapMana { card: first }));
        pay(&mut g);
        assert_eq!(g.state.stack.len(), 1);
        assert!(g.state.cast_rollback.is_none());
        assert_eq!(
            g.last_attempt_outcome_view(p).unwrap().outcome,
            AttemptOutcome::Completed
        );
        reject(&mut g, Action::SubmitAttempt);
    }
}

#[test]
fn omitted_allocation_preserves_continuation_and_rejected_requests_do_not_mutate() {
    let p = Player::P0;
    let mut g = scenario(p);
    let spell = put(&mut g, p, "Bear Cub", Zone::Hand);
    act(&mut g, Action::AnnounceCast { card: spell });
    act(&mut g, Action::EndManaWindow);
    let saved = g.snapshot();
    let revision = g.revision();
    assert_eq!(
        g.apply(p.other(), Action::SubmitAttempt),
        Err(Error::WrongActor)
    );
    assert_eq!(g.snapshot(), saved);
    assert_eq!(g.revision(), revision);
    reject(
        &mut g,
        Action::PayCosts {
            payment: [u32::MAX; 6],
        },
    );
    act(&mut g, Action::SubmitAttempt);
    assert_eq!(
        g.last_attempt_outcome_view(p).unwrap().outcome,
        AttemptOutcome::Incomplete
    );
    assert!(g.state.announcing.is_some());
    assert_eq!(g.object_view(spell).zone, Zone::Stack);
    pay(&mut g);
    assert_eq!(g.state.stack.len(), 1);
}

#[test]
fn failing_nested_mana_restores_parent_and_retains_selected_completed_events() {
    let _scope = fixture_rules::activate(&[(Card::LlanowarElves, &FILTER)]);
    for p in [Player::P0, Player::P1] {
        for keep in [false, true] {
            let mut g = scenario(p);
            zero_pool(&mut g, p);
            let spell = put(&mut g, p, "Bear Cub", Zone::Hand);
            let filter = put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
            let first = put(&mut g, p, "Forest", Zone::Battlefield);
            let second = put(&mut g, p, "Forest", Zone::Battlefield);
            act(&mut g, Action::AnnounceCast { card: spell });
            let parent = g.attempt_view(p).unwrap().attempt_id;
            act(
                &mut g,
                Action::AnnounceMana {
                    card: filter,
                    ability: 0,
                    counter_payment: Counters::default(),
                    production: GREEN,
                },
            );
            let child = g.attempt_view(p).unwrap();
            assert_ne!(child.attempt_id, parent);
            assert_eq!(child.parent_attempt_id, Some(parent));
            act(&mut g, Action::TapMana { card: first });
            close_and_submit(&mut g);
            assert!(matches!(g.state.pending, Pending::ManaRollback { .. }));
            let saved = g.snapshot();
            let mut indexed = g.clone();
            indexed.restore(&saved).unwrap();
            let action = Action::ChooseManaRollback { keep };
            let index = indexed
                .legal_actions()
                .iter()
                .position(|a| *a == action)
                .unwrap();
            act(&mut g, action);
            let revision = indexed.revision();
            indexed.apply_index(p, revision, index).unwrap();
            assert_eq!(g.snapshot(), indexed.snapshot());
            assert_eq!(g.attempt_view(p).unwrap().attempt_id, parent);
            assert!(!g.attempt_view(p).unwrap().mana_window_closed);
            assert_eq!(
                g.last_attempt_outcome_view(p).unwrap().attempt_id,
                child.attempt_id
            );
            assert_eq!(
                g.last_attempt_outcome_view(p).unwrap().outcome,
                AttemptOutcome::Failed
            );
            assert_eq!(g.object_view(spell).zone, Zone::Stack);
            assert!(!g.object_view(filter).tapped);
            assert_eq!(g.object_view(first).tapped, keep);
            assert_eq!(g.state.players[p.index()].mana[4], u32::from(keep));
            if !keep {
                act(&mut g, Action::TapMana { card: first });
            }
            act(&mut g, Action::TapMana { card: second });
            act(&mut g, Action::EndManaWindow);
            pay(&mut g);
            assert_eq!(g.state.stack.len(), 1);
            assert!(g.state.cast_rollback.is_none());
        }
    }
}

#[test]
fn restored_active_attempt_has_independent_raw_and_indexed_failure_continuations() {
    let p = Player::P0;
    let mut original = scenario(p);
    zero_pool(&mut original, p);
    let spell = put(&mut original, p, "Bear Cub", Zone::Hand);
    act(&mut original, Action::AnnounceCast { card: spell });
    act(&mut original, Action::EndManaWindow);
    let saved = original.snapshot();
    let mut raw = original.clone();
    let mut indexed = original.clone();
    indexed.restore(&saved).unwrap();
    let index = indexed
        .legal_actions()
        .iter()
        .position(|a| *a == Action::SubmitAttempt)
        .unwrap();
    let revision = indexed.revision();
    act(&mut raw, Action::SubmitAttempt);
    indexed.apply_index(p, revision, index).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(original.snapshot(), saved);
    assert_eq!(original.object_view(spell).zone, Zone::Stack);
    assert_eq!(raw.object_view(spell).zone, Zone::Hand);
}
