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
            cast_rollback_mana::reset_rollback_planning_work();
            let saved = g.snapshot();
            let mut indexed = g.clone();
            indexed.restore(&saved).unwrap();
            let action = Action::ChooseManaRollback { keep };
            let index = indexed
                .legal_actions()
                .iter()
                .position(|a| *a == action)
                .unwrap();
            let rollback_work = cast_rollback_mana::rollback_planning_work();
            assert!(rollback_work.suffix_entries > 0);
            assert!(rollback_work.replay_branches > 0);
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

#[test]
fn drawn_cards_stay_opaque_but_can_pay_unrestricted_discard_until_attempt_finishes() {
    const UNRESTRICTED: SacrificeFilter = SacrificeFilter {
        keywords: Keywords::NONE,
        source_only: false,
        kinds: 0,
        nonland: false,
        other: true,
        tokens: None,
        artifact_enchantment_or_token: false,
        subtype: None,
    };
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let spell = put(&mut g, p, "Bear Cub", Zone::Hand);
        let drawn = put(&mut g, p, "Bear Cub", Zone::Library);
        act(&mut g, Action::AnnounceCast { card: spell });
        assert!(g.draw_one_unreplaced(p));
        let object = g.object_ref(drawn);
        let source = g
            .state
            .announcing
            .as_ref()
            .unwrap()
            .item
            .source
            .unwrap()
            .object;
        assert!(g.attempt_draw_is_face_down(object));
        assert_eq!(
            g.attempt_draws_view(p),
            vec![AttemptDrawView { object, player: p }]
        );
        assert!(g.attempt_draws_view(p.other()).is_empty());
        assert!(
            !g.observe(p)
                .objects
                .iter()
                .any(|row| row.reference == object)
        );
        assert!(g.additional_candidate(
            AdditionalCost {
                kind: EffectChoiceKind::Discard,
                filter: UNRESTRICTED
            },
            p,
            source,
            object,
        ));
        assert!(!g.additional_candidate(
            AdditionalCost {
                kind: EffectChoiceKind::Discard,
                filter: SacrificeFilter {
                    kinds: 1,
                    ..UNRESTRICTED
                }
            },
            p,
            source,
            object,
        ));
        pay(&mut g);
        assert!(!g.attempt_draw_is_face_down(object));
        assert!(g.attempt_draws_view(p).is_empty());
        assert!(
            g.observe(p)
                .objects
                .iter()
                .any(|row| row.reference == object)
        );
        assert!(
            !g.observe(p.other())
                .objects
                .iter()
                .any(|row| row.reference == object)
        );
    }
}

#[test]
fn reversal_retains_actual_draw_incarnation_ordinals_and_deferred_reveal_without_reroll() {
    let p = Player::P0;
    let mut g = scenario(p);
    zero_pool(&mut g, p);
    let spell = put(&mut g, p, "Bear Cub", Zone::Hand);
    let drawn = put(&mut g, p, "Shock", Zone::Library);
    act(&mut g, Action::AnnounceCast { card: spell });
    assert!(g.draw_one_unreplaced(p));
    let object = g.object_ref(drawn);
    let rng = g.state.rng;
    g.remember_public_hand(object);
    assert!(
        !g.observe(p.other())
            .objects
            .iter()
            .any(|row| row.reference == object)
    );
    close_and_submit(&mut g);
    assert_eq!(g.object_ref(drawn), object);
    assert_eq!(g.object_view(drawn).zone, Zone::Hand);
    assert!(!g.state.players[p.index()].library.contains(&drawn));
    assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
    assert_eq!(g.state.rng, rng);
    assert!(
        g.observe(p.other())
            .objects
            .iter()
            .any(|row| row.reference == object)
    );
    assert!(g.attempt_draws_view(p).is_empty());
}

#[test]
fn optional_draw_reveal_waits_for_recovery_and_resumes_original_priority() {
    let p = Player::P0;
    let mut g = scenario(p);
    zero_pool(&mut g, p);
    put(&mut g, p, "Lorehold, the Historian", Zone::Battlefield);
    let spell = put(&mut g, p, "Bear Cub", Zone::Hand);
    let drawn = put(&mut g, p, "Shock", Zone::Library);
    act(&mut g, Action::AnnounceCast { card: spell });
    assert!(g.draw_one_unreplaced(p));
    assert!(g.observe(p).miracle.is_none());
    assert!(!matches!(g.state.pending, Pending::MiracleReveal { .. }));
    close_and_submit(&mut g);
    assert_eq!(g.observe(p).miracle.unwrap().object, g.object_ref(drawn));
    assert!(g.state.announcing.is_none());
    act(&mut g, Action::DeclineMiracle);
    assert_eq!(g.state.pending, Pending::Priority);
    assert_eq!(g.state.priority, p);
}

const OVERLAPPING_COUNTERS: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[
        ActivatedAbility {
            cost: ActivationCost {
                generic: 1,
                mana_component: true,
                remove: Some((Some(CounterKind::PlusOne), 1)),
                additional: Some(AdditionalCost {
                    kind: EffectChoiceKind::RemoveCreatureCounter(CounterKind::PlusOne),
                    filter: SacrificeFilter {
                        kinds: 1,
                        source_only: false,
                        other: false,
                        nonland: false,
                        tokens: None,
                        subtype: None,
                        keywords: Keywords::NONE,
                        artifact_enchantment_or_token: false,
                    },
                }),
                ..ActivationCost::ZERO
            },
            program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
            ..FILTER.activated[0]
        },
        ActivatedAbility {
            cost: ActivationCost {
                remove: Some((Some(CounterKind::PlusOne), 1)),
                ..ActivationCost::ZERO
            },
            program: &[Instruction::AddMana(ManaProduction::Fixed([
                0, 0, 0, 0, 0, 1,
            ]))],
            ..FILTER.activated[0]
        },
    ],
    ..CardRules::EMPTY
};

#[test]
fn overlapping_current_costs_can_edit_an_unpaid_selection_or_fail_with_owned_branches() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &OVERLAPPING_COUNTERS)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let source = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let helper = put(&mut g, p, "Goblin Piker", Zone::Battlefield);
    g.state.objects[source as usize].counters.plus_one = 2;
    g.state.objects[helper as usize].counters.plus_one = 1;
    g.state.players[0].mana = [0, 0, 0, 0, 0, 1];
    act(
        &mut g,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::of(CounterKind::PlusOne, 1),
        },
    );
    let object = g.object_ref(source);
    act(
        &mut g,
        Action::ChooseCreatureCounterCost {
            object,
            counter: CounterKind::PlusOne,
        },
    );
    act(
        &mut g,
        Action::AnnounceMana {
            card: source,
            ability: 1,
            counter_payment: Counters::of(CounterKind::PlusOne, 1),
            production: [0, 0, 0, 0, 0, 1],
        },
    );
    act(&mut g, Action::PayCosts { payment: [0; 6] });
    assert!(g.mana_payment_domain().is_none());
    act(&mut g, Action::EndManaWindow);
    let saved = g.snapshot();
    let mut raw = g.clone();
    let mut indexed = g.clone();
    indexed.restore(&saved).unwrap();
    let index = indexed
        .legal_actions()
        .iter()
        .position(|a| *a == Action::ResetCostSelection)
        .unwrap();
    let revision = indexed.revision();
    act(&mut raw, Action::ResetCostSelection);
    indexed.apply_index(p, revision, index).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(g.snapshot(), saved);
    assert_eq!(
        raw.state.objects[source as usize].counters.plus_one, 1,
        "editing a provisional selection retains the completed nested mana ability"
    );
    assert_eq!(raw.state.players[0].mana[5], 2);
    assert!(raw.attempt_view(p).unwrap().mana_window_closed);
    let helper_object = raw.object_ref(helper);
    act(
        &mut raw,
        Action::ChooseCreatureCounterCost {
            object: helper_object,
            counter: CounterKind::PlusOne,
        },
    );
    act(
        &mut raw,
        Action::PayCosts {
            payment: [0, 0, 0, 0, 0, 1],
        },
    );
    assert_eq!(raw.state.objects[source as usize].counters.plus_one, 0);
    assert_eq!(raw.state.objects[helper as usize].counters.plus_one, 0);
    assert_eq!(raw.state.stack.len(), 1);

    let mut failed_raw = g.clone();
    let mut failed_indexed = g.clone();
    failed_indexed.restore(&saved).unwrap();
    let index = failed_indexed
        .legal_actions()
        .iter()
        .position(|a| *a == Action::SubmitAttempt)
        .unwrap();
    let revision = failed_indexed.revision();
    act(&mut failed_raw, Action::SubmitAttempt);
    failed_indexed.apply_index(p, revision, index).unwrap();
    assert_eq!(failed_raw.snapshot(), failed_indexed.snapshot());
    assert_eq!(
        failed_raw.last_attempt_outcome_view(p).unwrap().reason,
        Some(AttemptFailureReason::UnsatisfiedMaterial)
    );
    assert!(matches!(
        failed_raw.state.pending,
        Pending::ManaRollback { .. }
    ));
    act(&mut failed_raw, Action::ChooseManaRollback { keep: false });
    assert!(failed_raw.state.announcing.is_none());
    assert_eq!(
        failed_raw.state.objects[source as usize].counters.plus_one,
        2
    );
    assert_eq!(
        failed_raw.state.objects[helper as usize].counters.plus_one,
        1
    );
    assert_eq!(failed_raw.state.players[0].mana[5], 1);
}

#[test]
fn failure_reverses_discard_after_a_frozen_library_departure() {
    for draw in [false, true] {
        let p = Player::P0;
        let mut g = scenario(p);
        zero_pool(&mut g, p);
        let spell = put(&mut g, p, "Thrill of Possibility", Zone::Hand);
        let card = put(&mut g, p, "Shock", Zone::Library);
        act(&mut g, Action::AnnounceCast { card: spell });
        if draw {
            assert!(g.draw_one_unreplaced(p));
        } else {
            g.move_object(card, Zone::Hand);
        }
        let frozen = g.object_ref(card);
        act(&mut g, Action::ChooseAdditionalCost { object: frozen });
        act(&mut g, Action::BeginOrderedCosts);
        let clause = g
            .state
            .announcing
            .as_ref()
            .unwrap()
            .cast_costs
            .as_ref()
            .unwrap()
            .clauses
            .iter()
            .position(|c| c.clause == CastCostClause::Additional)
            .unwrap();
        act(
            &mut g,
            Action::ChooseCastCost {
                clause: clause as u16,
            },
        );
        assert_eq!(g.object_view(card).zone, Zone::Graveyard);
        let saved = g.snapshot();
        let mut indexed = g.clone();
        let revision = indexed.revision();
        let index = indexed
            .legal_actions()
            .iter()
            .position(|action| *action == Action::SubmitAttempt)
            .unwrap();
        act(&mut g, Action::SubmitAttempt);
        indexed.apply_index(p, revision, index).unwrap();
        assert_eq!(g.snapshot(), indexed.snapshot());
        indexed.restore(&saved).unwrap();
        act(&mut indexed, Action::SubmitAttempt);
        assert_eq!(g.snapshot(), indexed.snapshot());
        assert_eq!(g.object_view(card).zone, Zone::Hand);
        assert_eq!(g.object_ref(card), frozen);
        assert!(
            g.observe(p.other())
                .objects
                .iter()
                .any(|row| row.reference == frozen),
            "recovery erased public knowledge of discarded card"
        );
        assert!(!g.state.players[p.index()].library.contains(&card));
        assert_eq!(g.object_view(spell).zone, Zone::Hand);
    }
}

#[test]
fn smoke_policy_keeps_each_players_failure_history_through_alternating_priority() {
    let mut g = scenario(Player::P0);
    zero_pool(&mut g, Player::P0);
    zero_pool(&mut g, Player::P1);
    put(&mut g, Player::P0, "Shock", Zone::Hand);
    put(&mut g, Player::P1, "Shock", Zone::Hand);
    let turn = g.state.turn;
    let mut failed = [false; 2];
    for _ in 0..80 {
        let actions = g.legal_actions();
        let index = rustmage_core::policy::smoke_index(&g, &actions);
        let action = actions[index].clone();
        act(&mut g, action);
        for player in [Player::P0, Player::P1] {
            failed[player.index()] |= g
                .last_attempt_outcome_view(player)
                .is_some_and(|o| o.outcome == AttemptOutcome::Failed);
        }
        if g.state.turn != turn {
            break;
        }
    }
    assert!(failed.into_iter().all(|v| v));
    assert_ne!(
        g.state.turn, turn,
        "smoke policy repeatedly retried a failed spell"
    );
}

#[test]
fn deferred_draw_reveal_joins_cast_triggers_before_ordering() {
    let p = Player::P0;
    let mut g = scenario(p);
    const OBSERVER: CardRules = CardRules {
        effect: Effect::Permanent,
        triggers: &[TriggeredAbility {
            condition: TriggerCondition::SpellCast {
                relation: CastRelation::You,
                filter: CastFilter::Noncreature,
                ordinal: None,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
        }],
        ..CardRules::EMPTY
    };
    let _rules = fixture_rules::activate(&[(Card::from_name("Short Bow").unwrap(), &OBSERVER)]);
    let iconoclast = put(&mut g, p, "Short Bow", Zone::Battlefield);
    put(&mut g, p, "Lorehold, the Historian", Zone::Battlefield);
    let spell = put(&mut g, p, "Divination", Zone::Hand);
    put(&mut g, p, "Shock", Zone::Library);
    act(&mut g, Action::AnnounceCast { card: spell });
    assert!(g.draw_one_unreplaced(p));
    pay(&mut g);
    assert!(matches!(g.state.pending, Pending::MiracleReveal { .. }));
    assert!(g.state.ordering_triggers.is_empty());
    assert_eq!(
        g.state.stack.len(),
        1,
        "cast trigger stacked before deferred reveal"
    );
    assert!(
        g.state
            .pending_triggers
            .iter()
            .any(|item| item.source.is_some_and(|source| source.id == iconoclast))
    );
    act(&mut g, Action::RevealMiracle);
    assert!(matches!(g.state.pending, Pending::OrderTriggers { .. }));
    assert!(g.state.ordering_triggers.len() >= 2);
}

#[test]
fn failed_attempt_preserves_shuffle_rng_even_for_an_unchanged_permutation() {
    let p = Player::P0;
    let mut g = scenario(p);
    zero_pool(&mut g, p);
    let spell = put(&mut g, p, "Bear Cub", Zone::Hand);
    // Two cards let an actual shuffle advance RNG without changing order.
    let library = g.state.players[p.index()].library.clone();
    for &card in library.iter().skip(1) {
        g.move_object(card, Zone::Graveyard);
    }
    put(&mut g, p, "Shock", Zone::Library);
    act(&mut g, Action::AnnounceCast { card: spell });
    let original_rng = g.state.rng;
    let before_order = g.state.players[p.index()].library.clone();
    for _ in 0..100 {
        g.shuffle_library_cards(p);
        if g.state.players[p.index()].library == before_order {
            break;
        }
    }
    assert_eq!(g.state.players[p.index()].library, before_order);
    let frozen_rng = g.state.rng;
    assert_ne!(frozen_rng, original_rng);
    close_and_submit(&mut g);
    assert_eq!(g.state.rng, frozen_rng);
    assert_eq!(g.state.players[p.index()].library, before_order);
}
