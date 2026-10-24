//! Whole Rewrite History with grouped real costs, private loot and exact recovery.
use super::assurance::big_helpers::{cast, checked, named, settle, setup};
use super::*;

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
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..100 {
        if stop(g) {
            return;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass
                        | Action::PutTrigger { .. }
                        | Action::FinishTargets
                        | Action::PayCosts { .. }
                )
            })
            .unwrap_or_else(|| panic!("stalled {:?}", g.state.pending));
        checked(g, action);
    }
    panic!("stalled {:?}", g.state.pending);
}
fn plan(g: &mut Game, p: Player) -> CardId {
    let id = named(g, p, "Rewrite History", Zone::Hand);
    g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 2];
    cast(g, id, None, &[]);
    settle(g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    g.state.players[p.index()].mana = [30; 6];
    id
}
fn taps(g: &Game, id: CardId) -> usize {
    g.state
        .pending_triggers
        .iter()
        .filter(|s| s.card == id && s.kind == (StackKind::TriggeredAbility { ability: 0 }))
        .count()
}
fn loot(g: &mut Game, p: Player, id: CardId) {
    let before = g.counter_count(id, CounterKind::Plan);
    until(g, |g| {
        matches!(g.state.pending, Pending::EffectDiscard { .. })
    });
    assert_eq!(g.actor(), Some(p));
    assert_eq!(g.counter_count(id, CounterKind::Plan), before);
    let card = g
        .legal_actions()
        .into_iter()
        .find_map(|a| {
            if let Action::Discard { card } = a {
                Some(card)
            } else {
                None
            }
        })
        .unwrap();
    checked(g, Action::Discard { card });
    assert_eq!(g.counter_count(id, CounterKind::Plan), before + 1);
}
fn clause(g: &mut Game, kind: CastCostClause) {
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
        .position(|r| r.clause == kind)
        .unwrap() as u16;
    checked(g, Action::ChooseCastCost { clause });
}
fn pay(g: &mut Game) {
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    checked(g, a);
}
fn begin_vote(g: &mut Game, p: Player, casualty: bool) -> (CardId, CardId) {
    let target = named(g, p.other(), "Hill Giant", Zone::Battlefield);
    let spell = named(g, p, "Vote Out", Zone::Hand);
    checked(g, Action::AnnounceCast { card: spell });
    if g.legal_actions()
        .contains(&Action::ChooseCasualty { pay: casualty })
    {
        checked(g, Action::ChooseCasualty { pay: casualty });
    }
    checked(
        g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    checked(g, Action::BeginConvoke);
    (spell, target)
}
#[test]
fn msh_tap_batch_paid_plan_two_attackers_one_private_loot_before_counter() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = plan(&mut g, p);
        let a = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let b = named(&mut g, p, "Hill Giant", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Shock", Zone::Hand);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        for c in [a, b] {
            g.state.objects[c as usize].summoning_sick = false;
            checked(&mut g, Action::Attack { card: c });
        }
        assert!(!g.object_view(a).tapped && !g.object_view(b).tapped);
        checked(&mut g, Action::FinishAttackers);
        assert!(g.object_view(a).tapped && g.object_view(b).tapped);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectDiscard { .. })
        });
        assert_eq!(g.counter_count(id, CounterKind::Plan), 0);
        reject(&mut g, Action::Discard { card: enemy });
        let hidden = g
            .legal_actions()
            .into_iter()
            .find_map(|a| {
                if let Action::Discard { card } = a {
                    Some(card)
                } else {
                    None
                }
            })
            .unwrap();
        let mut alternate = g.clone();
        alternate.state.objects[hidden as usize].card = Card::from_name("Mountain").unwrap();
        assert_eq!(
            serde_json::to_value(g.observe(p.other())).unwrap(),
            serde_json::to_value(alternate.observe(p.other())).unwrap()
        );
        let discard = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::Discard { .. }))
            .unwrap();
        checked(&mut g, discard);
        assert_eq!(g.counter_count(id, CounterKind::Plan), 1);
        assert_eq!(taps(&g, id), 0);
    }
}
#[test]
fn msh_tap_batch_crew_whole_group_and_two_separate_events_same_turn() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = plan(&mut g, p);
        let a = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let b = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let v = named(&mut g, p, "Detention Chariot", Zone::Battlefield);
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: v,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        for c in [a, b] {
            let object = g.object_ref(c);
            checked(&mut g, Action::ChooseCrewer { object });
        }
        checked(&mut g, Action::FinishCrew);
        loot(&mut g, p, id);
        settle(&mut g);
        assert_eq!(g.counter_count(id, CounterKind::Plan), 1);
        for c in [a, b] {
            g.untap_objects(&[c], untap_rules::UntapCause::Effect);
            g.tap_objects(&[c]);
        }
        assert_eq!(taps(&g, id), 2);
        loot(&mut g, p, id);
        loot(&mut g, p, id);
        settle(&mut g);
        assert_eq!(g.counter_count(id, CounterKind::Plan), 3);
    }
}
#[test]
fn msh_tap_batch_teamwork_cost_group_triggers_before_spell() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = plan(&mut g, p);
        let a = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let b = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let spell = named(&mut g, p, "Team Tactics", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: spell });
        checked(&mut g, Action::ChooseTeamwork { pay: true });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(a),
            },
        );
        pay(&mut g);
        clause(&mut g, CastCostClause::Teamwork(1));
        for c in [a, b] {
            let object = g.object_ref(c);
            checked(&mut g, Action::ChooseTeammate { object });
        }
        assert_eq!(taps(&g, id), 0);
        checked(&mut g, Action::FinishTeamwork);
        loot(&mut g, p, id);
        settle(&mut g);
        assert_eq!(g.counter_count(id, CounterKind::Plan), 1);
    }
}
#[test]
fn msh_tap_batch_convoke_contributors_commit_together() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = plan(&mut g, p);
        let a = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let b = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 1];
        begin_vote(&mut g, p, false);
        for c in [a, b] {
            let object = g.object_ref(c);
            checked(
                &mut g,
                Action::ChooseConvoker {
                    object,
                    contribution: ConvokeContribution::Generic,
                },
            );
            assert!(!g.object_view(c).tapped);
            assert_eq!(taps(&g, id), 0);
        }
        pay(&mut g);
        assert!(g.object_view(a).tapped && g.object_view(b).tapped);
        loot(&mut g, p, id);
        settle(&mut g);
        assert_eq!(g.counter_count(id, CounterKind::Plan), 1);
    }
}
#[test]
fn msh_tap_batch_ordered_convoke_mana_clause_is_atomic_before_or_after_casualty() {
    for p in [Player::P0, Player::P1] {
        for mana_first in [false, true] {
            let mut g = setup(p);
            let id = plan(&mut g, p);
            named(&mut g, p, "Silverquill, the Disputant", Zone::Battlefield);
            let a = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let b = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let sacrifice = named(&mut g, p, "Hill Giant", Zone::Battlefield);
            g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 1];
            begin_vote(&mut g, p, true);
            if !mana_first {
                clause(&mut g, CastCostClause::Casualty(0));
                let object = g.object_ref(sacrifice);
                checked(&mut g, Action::PayCasualty { object });
            }
            clause(&mut g, CastCostClause::Mana);
            for c in [a, b] {
                let object = g.object_ref(c);
                checked(
                    &mut g,
                    Action::ChooseConvoker {
                        object,
                        contribution: ConvokeContribution::Generic,
                    },
                );
                assert!(!g.object_view(c).tapped);
                assert!(
                    !g.legal_actions()
                        .iter()
                        .any(|a| matches!(a, Action::ChooseCastCost { .. }))
                );
            }
            pay(&mut g);
            assert!(g.object_view(a).tapped && g.object_view(b).tapped);
            if mana_first {
                clause(&mut g, CastCostClause::Casualty(0));
                let object = g.object_ref(sacrifice);
                checked(&mut g, Action::PayCasualty { object });
            }
            // The casualty copy target selection can precede trigger resolution.
            until(&mut g, |g| {
                g.legal_actions().contains(&Action::FinishStackTargets)
                    || matches!(g.state.pending, Pending::EffectDiscard { .. })
            });
            if g.legal_actions().contains(&Action::FinishStackTargets) {
                checked(&mut g, Action::FinishStackTargets);
            }
            loot(&mut g, p, id);
            settle(&mut g);
            assert_eq!(g.counter_count(id, CounterKind::Plan), 1);
        }
    }
}
#[test]
fn msh_tap_batch_filters_noncreature_opponent_already_tapped_and_entry_tapped() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = plan(&mut g, p);
        let other = named(&mut g, p.other(), "Rewrite History", Zone::Battlefield);
        let a = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let b = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let land = named(&mut g, p, "Forest", Zone::Battlefield);
        g.tap_objects(&[land]);
        assert_eq!((taps(&g, id), taps(&g, other)), (0, 0));
        g.tap_objects(&[a, b, a]);
        assert_eq!((taps(&g, id), taps(&g, other)), (1, 1));
        g.tap_objects(&[a, b]);
        assert_eq!((taps(&g, id), taps(&g, other)), (1, 1));
        let entered = named(&mut g, p, "Hill Giant", Zone::Hand);
        let movement = g.entry_move(entered, Zone::Battlefield, true);
        g.stage_entries(vec![movement], None);
        g.advance_entries();
        assert_eq!((taps(&g, id), taps(&g, other)), (1, 1));
    }
}
#[test]
fn msh_tap_batch_fourth_counter_after_discard_sacrifices_then_selects_up_to_two_spells() {
    for p in [Player::P0, Player::P1] {
        for count in [0, 1, 2] {
            let mut g = setup(p);
            let id = plan(&mut g, p);
            g.set_counters(id, Counters::of(CounterKind::Plan, 3));
            let a = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let instant = named(&mut g, p, "Shock", Zone::Hand);
            let sorcery = named(&mut g, p, "Divination", Zone::Graveyard);
            let wrong = named(&mut g, p.other(), "Shock", Zone::Graveyard);
            g.tap_objects(&[a]);
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::EffectDiscard { .. })
            });
            checked(&mut g, Action::Discard { card: instant });
            until(&mut g, |g| {
                g.state
                    .announcing
                    .as_ref()
                    .is_some_and(|a| a.item.kind == (StackKind::TriggeredAbility { ability: 2 }))
            });
            assert_eq!(g.object_view(id).zone, Zone::Graveyard);
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(wrong),
                },
            );
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(a),
                },
            );
            for &card in [instant, sorcery].iter().take(count) {
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(card),
                    },
                );
            }
            if g.state.announcing.is_some() {
                checked(&mut g, Action::FinishTargets);
            }
            settle(&mut g);
            assert_eq!(
                g.object_view(instant).zone,
                if count > 0 {
                    Zone::Hand
                } else {
                    Zone::Graveyard
                }
            );
            assert_eq!(
                g.object_view(sorcery).zone,
                if count > 1 {
                    Zone::Hand
                } else {
                    Zone::Graveyard
                }
            );
        }
    }
}
#[test]
fn msh_tap_batch_loot_captured_controller_and_exact_source_after_departure_or_blink() {
    for p in [Player::P0, Player::P1] {
        for change in 0..3 {
            let mut g = setup(p);
            let id = plan(&mut g, p);
            let a = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            g.tap_objects(&[a]);
            g.prepare_triggers();
            if change == 0 {
                g.record_control(g.object_ref(id), p.other(), None);
            } else {
                g.move_object(id, Zone::Exile);
                if change == 2 {
                    g.move_object(id, Zone::Battlefield);
                }
            }
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::EffectDiscard { .. })
            });
            assert_eq!(g.actor(), Some(p));
            let discard = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::Discard { .. }))
                .unwrap();
            checked(&mut g, discard);
            settle(&mut g);
            assert_eq!(
                g.counter_count(id, CounterKind::Plan),
                if change == 0 { 1 } else { 0 }
            );
        }
    }
}
#[test]
fn msh_tap_batch_discard_and_counter_overflow_roll_back_the_whole_resolution_action() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = plan(&mut g, p);
        g.set_counters(id, Counters::of(CounterKind::Plan, u64::MAX));
        let a = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.tap_objects(&[a]);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectDiscard { .. })
        });
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::Discard { .. }))
            .unwrap();
        let before = g.snapshot();
        let revision = g.revision();
        assert!(matches!(
            g.apply(p, action),
            Err(Error::CapacityExceeded(_))
        ));
        assert_eq!(g.snapshot(), before);
        assert_eq!(g.revision(), revision);
    }
}

#[test]
fn msh_tap_batch_existing_self_observer_and_global_observer_both_see_whole_crew() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = plan(&mut g, p);
        let wylie = named(&mut g, p, "Wylie Duke, Atiin Hero", Zone::Battlefield);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let v = named(&mut g, p, "Detention Chariot", Zone::Battlefield);
        let hand = g.hand_count(p);
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: v,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        for c in [wylie, bear] {
            let object = g.object_ref(c);
            checked(&mut g, Action::ChooseCrewer { object });
        }
        checked(&mut g, Action::FinishCrew);
        loot(&mut g, p, id);
        settle(&mut g);
        assert_eq!(g.counter_count(id, CounterKind::Plan), 1);
        assert_eq!(g.state.players[p.index()].life, 21);
        assert_eq!(g.hand_count(p), hand + 1);
    }
}
#[test]
fn msh_tap_batch_two_mana_activations_are_separate_events_and_late_observer_gets_no_history() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let a = named(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        let b = named(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        for c in [a, b] {
            g.state.objects[c as usize].summoning_sick = false;
        }
        checked(&mut g, Action::TapMana { card: a });
        let id = plan(&mut g, p);
        assert_eq!(taps(&g, id), 0);
        g.untap_objects(&[a], untap_rules::UntapCause::Effect);
        for c in [a, b] {
            checked(&mut g, Action::TapMana { card: c });
        }
        loot(&mut g, p, id);
        loot(&mut g, p, id);
        settle(&mut g);
        assert_eq!(g.counter_count(id, CounterKind::Plan), 2);
    }
}

#[test]
fn msh_tap_batch_paid_multi_target_effect_commits_all_taps_before_one_loot() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = plan(&mut g, p);
        let a = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let b = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let spell = named(&mut g, p, "Out Cold", Zone::Hand);
        cast(&mut g, spell, None, &[Target::Card(a), Target::Card(b)]);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectDiscard { .. })
        });
        assert!(g.object_view(a).tapped && g.object_view(b).tapped);
        assert_eq!(g.counter_count(a, CounterKind::Stun), 1);
        assert_eq!(g.counter_count(b, CounterKind::Stun), 1);
        loot(&mut g, p, id);
        settle(&mut g);
        assert_eq!(g.counter_count(id, CounterKind::Plan), 1);
    }
}
#[test]
fn msh_tap_batch_recovery_keeps_legal_targets_and_rejects_a_fresh_graveyard_incarnation() {
    for p in [Player::P0, Player::P1] {
        for all_invalid in [false, true] {
            let mut g = setup(p);
            let id = plan(&mut g, p);
            let instant = named(&mut g, p, "Shock", Zone::Graveyard);
            let sorcery = named(&mut g, p, "Divination", Zone::Graveyard);
            g.effect_counter(id, CounterKind::Plan, 4, replacements::ResumeMode::Pending);
            until(&mut g, |g| {
                g.state
                    .announcing
                    .as_ref()
                    .is_some_and(|a| a.item.kind == (StackKind::TriggeredAbility { ability: 2 }))
            });
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(instant),
                },
            );
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(sorcery),
                },
            );
            if g.state.announcing.is_some() {
                checked(&mut g, Action::FinishTargets);
            }
            g.move_object(instant, Zone::Exile);
            g.move_object(instant, Zone::Graveyard);
            if all_invalid {
                g.move_object(sorcery, Zone::Exile);
            }
            settle(&mut g);
            assert_eq!(g.object_view(instant).zone, Zone::Graveyard);
            assert_eq!(
                g.object_view(sorcery).zone,
                if all_invalid { Zone::Exile } else { Zone::Hand }
            );
        }
    }
}
#[test]
fn msh_tap_batch_failed_source_sacrifice_never_selects_recovery_targets() {
    for p in [Player::P0, Player::P1] {
        for change in 0..3 {
            let mut g = setup(p);
            let id = plan(&mut g, p);
            let spell = named(&mut g, p, "Shock", Zone::Graveyard);
            g.effect_counter(id, CounterKind::Plan, 4, replacements::ResumeMode::Pending);
            g.prepare_triggers();
            if change == 0 {
                g.record_control(g.object_ref(id), p.other(), None);
            } else {
                g.move_object(id, Zone::Exile);
                if change == 2 {
                    g.move_object(id, Zone::Battlefield);
                }
            }
            settle(&mut g);
            assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
            assert_eq!(
                g.counter_count(id, CounterKind::Plan),
                if change == 0 { 4 } else { 0 }
            );
        }
    }
}
