use super::tests::{act, cast, scenario, spawn};
use super::*;
fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let c = Card::from_name(name).unwrap();
    assert!(c.is_supported(), "{name}");
    spawn(g, p, c, zone)
}
fn replay(g: &mut Game, a: Action) {
    let saved = g.snapshot();
    let rev = g.revision();
    let actor = g.actor().unwrap();
    let mut fork = g.clone();
    act(&mut fork, a.clone());
    act(g, a.clone());
    let expected = g.snapshot();
    assert_eq!(fork.snapshot(), expected);
    g.restore(&saved).unwrap();
    assert_eq!(g.apply_index(actor, rev, 0), Err(Error::StaleRevision));
    act(g, a);
    assert_eq!(g.snapshot(), expected);
}
fn passes(g: &mut Game) {
    replay(g, Action::Pass);
    replay(g, Action::Pass);
}
fn losses(g: &Game, expected: [u128; 2]) {
    for p in [Player::P0, Player::P1] {
        let o = g.observe(p);
        assert_eq!(o.schema, crate::engine::OBSERVATION_SCHEMA);
        assert_eq!(
            [
                o.players[0].life_lost_this_turn,
                o.players[1].life_lost_this_turn
            ],
            expected
        );
    }
}
fn fund(g: &mut Game, name: &str) -> CardId {
    let id = named(g, Player::P0, name, Zone::Hand);
    g.state.players[0].mana = [10; 6];
    id
}
fn triggers(g: &mut Game, target: Option<Target>) {
    g.prepare_triggers();
    for _ in 0..20 {
        if g.state.pending == Pending::Priority {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| match a {
                Action::ChooseTarget { target: t } => Some(*t) == target,
                Action::PutTrigger { .. } | Action::FinishTargets => true,
                _ => false,
            })
            .expect("trigger action");
        replay(g, a);
    }
    panic!("trigger stalled");
}

#[test]
fn loss_history_tracks_positive_losses_instead_of_net_life_and_is_owned_across_turn_reset() {
    let mut g = scenario();
    g.lose_life(Player::P0, 0);
    g.lose_life(Player::P0, -3);
    losses(&g, [0, 0]);
    g.lose_life(Player::P0, 3);
    g.gain_life(Player::P0, 8);
    g.lose_life(Player::P0, 2);
    g.lose_life(Player::P1, 7);
    assert_eq!([g.state.players[0].life, g.state.players[1].life], [23, 13]);
    losses(&g, [5, 7]);
    let saved = g.snapshot();
    let fork = g.clone();
    g.begin_turn();
    losses(&g, [0, 0]);
    losses(&fork, [5, 7]);
    g.restore(&saved).unwrap();
    losses(&g, [5, 7]);
    g.state.players[0].life = -2;
    g.lose_life(Player::P0, 9);
    assert_eq!(g.state.players[0].life, -11);
    losses(&g, [14, 7]);
}

#[test]
fn simultaneous_damage_records_both_losses_even_when_lifelink_makes_the_source_controller_gain_net_life()
 {
    let mut g = scenario();
    let source = named(&mut g, Player::P0, "Bishop's Soldier", Zone::Battlefield);
    let damage = g.damage_source(source);
    let events = [
        DamageEvent {
            preventable: true,
            source: damage,
            target: Target::Player(Player::P0),
            amount: 2,
            combat: true,
        },
        DamageEvent {
            preventable: true,
            source: damage,
            target: Target::Player(Player::P1),
            amount: 2,
            combat: true,
        },
    ];
    g.apply_damage_batch(&events);
    losses(&g, [2, 2]);
    assert_eq!([g.state.players[0].life, g.state.players[1].life], [22, 18]);
    let target = named(&mut g, Player::P1, "Ordinary Bear", Zone::Battlefield);
    let walker = named(&mut g, Player::P1, "Professor Dellian Fel", Zone::Hand);
    g.move_object(walker, Zone::Battlefield);
    for target in [
        Target::Player(Player::P1),
        Target::Card(target),
        Target::Card(walker),
    ] {
        g.apply_damage_batch(&[DamageEvent {
            preventable: true,
            source: damage,
            target,
            amount: if matches!(target, Target::Player(_)) {
                0
            } else {
                1
            },
            combat: false,
        }]);
    }
    losses(&g, [2, 2]);
}

#[test]
fn actual_spell_damage_and_explicit_self_life_loss_use_the_same_checked_history() {
    let mut g = scenario();
    let shock = fund(&mut g, "Shock");
    cast(&mut g, shock, Some(Target::Player(Player::P1)));
    passes(&mut g);
    losses(&g, [0, 2]);
    let draw = fund(&mut g, "Visions of Villainy");
    cast(&mut g, draw, None);
    passes(&mut g);
    losses(&g, [2, 2]);
    assert_eq!(g.state.players[0].life, 18);
}

#[test]
fn activated_life_payments_commit_history_before_the_ability_and_funding_queries_do_not_mutate_it()
{
    let mut g = scenario();
    let source = named(&mut g, Player::P0, "Desolation Prowler", Zone::Battlefield);
    let saved = g.snapshot();
    for _ in 0..3 {
        let _ = g.legal_actions();
        let _ = g.legal_action_space();
    }
    assert_eq!(g.snapshot(), saved);
    let action = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a,Action::Activate{card,..}if *card==source))
        .unwrap();
    replay(&mut g, action);
    losses(&g, [2, 0]);
    assert!(!g.state.stack.is_empty());
    let before = g.object_view(source).power;
    passes(&mut g);
    assert_eq!(g.object_view(source).power, before + 2);
    losses(&g, [2, 0]);
}

#[test]
fn entry_replacements_freeze_before_same_batch_life_payments_but_etb_triggers_see_them() {
    for prior_loss in [false, true] {
        for pay in [false, true] {
            for reverse in [false, true] {
                let mut g = scenario();
                if prior_loss {
                    g.lose_life(Player::P1, 1);
                }
                let land = named(&mut g, Player::P1, "Blood Crypt", Zone::Hand);
                let lizard = named(&mut g, Player::P0, "Frilled Sparkshooter", Zone::Graveyard);
                let collector = named(&mut g, Player::P0, "Bloodtithe Collector", Zone::Hand);
                let mut entries = vec![
                    g.entry_move(lizard, Zone::Battlefield, false),
                    g.entry_move(land, Zone::Battlefield, false),
                    g.entry_move(collector, Zone::Battlefield, false),
                ];
                if reverse {
                    entries.reverse();
                }
                g.stage_entries(entries, None);
                assert!(g.advance_entries());
                assert_eq!(g.actor(), Some(Player::P1));
                losses(&g, [0, u128::from(prior_loss)]);
                replay(&mut g, Action::ChooseEntryLife { pay });
                losses(&g, [0, u128::from(prior_loss) + if pay { 2 } else { 0 }]);
                // This replacement uses the pre-choice condition, even though
                // the completed payment may have enabled the ETB trigger.
                assert_eq!(
                    g.object_view(lizard).counters.plus_one,
                    u64::from(prior_loss)
                );
                assert_eq!(
                    g.state
                        .pending_triggers
                        .iter()
                        .chain(&g.state.ordering_triggers)
                        .chain(&g.state.stack)
                        .filter(|s| s.card == collector)
                        .count(),
                    usize::from(prior_loss || pay)
                );
                let later = named(&mut g, Player::P0, "Frilled Sparkshooter", Zone::Hand);
                g.move_object(later, Zone::Battlefield);
                assert_eq!(
                    g.object_view(later).counters.plus_one,
                    u64::from(prior_loss || pay)
                );
            }
        }
    }
}

// Explicit immutable future-IR fixture covers scaled drain and emblem-captured
// loss without admitting a card whose remaining printed body is unsupported.
fn program(g: &mut Game, code: &'static [Instruction]) {
    let source = named(g, Player::P0, "Shock", Zone::Hand);
    let mut item = g.cast_item(Player::P0, source, [0; 6]);
    item.life_event = Some(LifeTriggerView {
        player: Player::P0,
        amount: 7,
    });
    g.state.resolving = Some(Resolution {
        wheel: None,
        continuation: None,
        chosen_creature_type: None,
        chosen_color: None,
        explore: None,
        evidence: None,
        connive: None,
        item,
        mana_payment: None,
        forage: None,
        program: code,
        pc: 0,
        target_group: 0,
        moved_count: None,
        discards: [0; 2],
        selected_discards: [Vec::new(), Vec::new()],
        milled: Vec::new(),
        milled_selection: None,
        library: None,
        selection: None,
        target_library: None,
        search: None,
        optional_searches: None,
        choice: None,
    });
    g.run_program();
}
#[test]
fn scaled_drains_and_captured_emblem_losses_record_their_full_amounts() {
    let mut g = scenario();
    g.state.players[1].life = 100;
    program(
        &mut g,
        &[Instruction::DrainLifeScaled(AmountValue::Stat(
            StatValue::ControllerLife,
        ))],
    );
    losses(&g, [0, 20]);
    assert_eq!(g.state.players[0].life, 40);
    program(&mut g, &[Instruction::LoseGainedLife(PlayerSet::Opponent)]);
    losses(&g, [0, 27]);
}

#[test]
fn entry_counter_uses_opponents_loss_even_after_regain_and_never_the_controllers_own_loss() {
    for opponent in [false, true] {
        let mut g = scenario();
        g.lose_life(if opponent { Player::P1 } else { Player::P0 }, 2);
        g.gain_life(if opponent { Player::P1 } else { Player::P0 }, 4);
        let id = fund(&mut g, "Frilled Sparkshooter");
        cast(&mut g, id, None);
        passes(&mut g);
        let o = g.object_view(id);
        assert_eq!(o.counters.plus_one, u64::from(opponent));
        assert_eq!(
            (o.power, o.toughness),
            (3 + i128::from(opponent), 3 + i128::from(opponent))
        );
        assert!(o.keywords.contains(Keywords::MENACE));
        assert!(o.keywords.contains(Keywords::REACH));
    }
}

#[test]
fn bloodtithe_trigger_demands_only_opponent_private_discard_for_either_controller() {
    for controller in [Player::P0, Player::P1] {
        for lost in [false, true] {
            let mut g = scenario();
            let opponent = controller.other();
            let discard = named(&mut g, opponent, "Ordinary Bear", Zone::Hand);
            if lost {
                g.lose_life(opponent, 1);
                g.gain_life(opponent, 2);
            } else {
                g.lose_life(controller, 1);
            }
            let id = named(&mut g, controller, "Bloodtithe Collector", Zone::Hand);
            g.move_object(id, Zone::Battlefield);
            assert_eq!(g.state.pending_triggers.len(), usize::from(lost));
            assert!(g.keywords(id).contains(Keywords::FLYING));
            if !lost {
                continue;
            }
            triggers(&mut g, None);
            passes(&mut g);
            assert_eq!(g.actor(), Some(opponent));
            assert_eq!(g.observe(opponent).decision, "effect_discard");
            assert!(g.observe(controller).effect_discards.is_empty());
            assert!(g.observe(controller).actions.is_empty());
            replay(&mut g, Action::Discard { card: discard });
            assert_eq!(g.object_view(discard).zone, Zone::Graveyard);
        }
    }
}

#[test]
fn stromkirk_targets_controlled_vampire_permanents_including_changelings_and_checks_beginning_guard()
 {
    for target_name in ["Bloodtithe Collector", "Three Tree Mascot"] {
        for lost in [false, true] {
            let mut g = scenario();
            named(
                &mut g,
                Player::P0,
                "Stromkirk Bloodthief",
                Zone::Battlefield,
            );
            let target = named(&mut g, Player::P0, target_name, Zone::Battlefield);
            let wrong = named(&mut g, Player::P0, "Ordinary Bear", Zone::Battlefield);
            if lost {
                g.lose_life(Player::P1, 1);
            } else {
                g.lose_life(Player::P0, 1);
            }
            g.begin_step(Step::End);
            assert_eq!(g.state.pending_triggers.len(), usize::from(lost));
            if !lost {
                continue;
            }
            g.prepare_triggers();
            let saved = g.snapshot();
            let rev = g.revision();
            assert_eq!(
                g.apply(
                    Player::P0,
                    Action::ChooseTarget {
                        target: Target::Card(wrong)
                    }
                ),
                Err(Error::IllegalAction)
            );
            assert_eq!(g.snapshot(), saved);
            assert_eq!(g.revision(), rev);
            triggers(&mut g, Some(Target::Card(target)));
            passes(&mut g);
            assert_eq!(g.object_view(target).counters.plus_one, 1);
        }
    }
}

#[test]
fn wide_loss_history_and_overflow_preserve_exact_json_and_restore_the_action() {
    let mut g = scenario();
    g.state.players[1].life = 1i128 << 100;
    g.lose_life(Player::P1, 1i128 << 80);
    assert_eq!(
        serde_json::to_value(g.observe(Player::P0)).unwrap()["players"][1]["life_lost_this_turn"]
            .to_string(),
        (1u128 << 80).to_string()
    );
    let shock = fund(&mut g, "Shock");
    cast(&mut g, shock, Some(Target::Player(Player::P1)));
    replay(&mut g, Action::Pass);
    g.state.players[1].life_lost_this_turn = u128::MAX - 1;
    let saved = g.snapshot();
    let rev = g.revision();
    for _ in 0..2 {
        assert_eq!(
            g.apply(g.actor().unwrap(), Action::Pass),
            Err(Error::CapacityExceeded(crate::CapacityExceeded {
                resource: "life lost this turn representation exhausted"
            }))
        );
        assert_eq!(g.snapshot(), saved);
        assert_eq!(g.revision(), rev);
    }
}

#[test]
fn failed_entry_life_payment_restores_owned_choice_flags_life_and_history_before_decline() {
    let mut g = scenario();
    let land = named(&mut g, Player::P0, "Blood Crypt", Zone::Hand);
    replay(&mut g, Action::PlayLand { card: land });
    g.state.players[0].life_lost_this_turn = u128::MAX - 1;
    let saved = g.snapshot();
    let revision = g.revision();
    for _ in 0..2 {
        assert_eq!(
            g.apply(Player::P0, Action::ChooseEntryLife { pay: true }),
            Err(Error::CapacityExceeded(crate::CapacityExceeded {
                resource: "life lost this turn representation exhausted"
            }))
        );
        assert_eq!(g.snapshot(), saved);
        assert_eq!(g.revision(), revision);
        assert_eq!(g.observe(Player::P0).decision, "entry_life");
        assert_eq!(g.state.players[0].life, 20);
    }
    replay(&mut g, Action::ChooseEntryLife { pay: false });
    assert!(g.object_view(land).tapped);
    assert_eq!(g.state.players[0].life_lost_this_turn, u128::MAX - 1);
}
