//! Any-counter triggers, current counter-dependent blocking, and optional removal.
use super::ability_timeline_tests::{checked, named, next_turn, settle, setup};
use super::tmt_departure_tests::{activate, candidate, cast_candidate, food, mutagens, reject};
use super::*;
use crate::cards::AnimationDuration;
fn counters(g: &mut Game, id: CardId, values: Counters) {
    g.effect_counters(
        vec![(g.object_ref(id), values)],
        replacements::ResumeMode::Pending,
    );
    assert!(!g.advance_replacements(false));
    settle(g);
}
#[test]
fn tmt_mutagen_mikey_leo_paid_hybrid_and_self_counter_draw_once_each_turn() {
    for p in [Player::P0, Player::P1] {
        for pool in [[0, 0, 0, 0, 2, 0], [1, 0, 0, 0, 1, 0], [2, 0, 0, 0, 0, 0]] {
            let mut g = setup(p);
            let duo = candidate(&mut g, p, "Mikey & Leo, Chaos & Order", Zone::Hand);
            g.state.players[p.index()].mana = pool;
            if g.state.objects[duo as usize].card.is_supported() {
                checked(&mut g, Action::AnnounceCast { card: duo });
                if matches!(g.state.pending, Pending::SpellCost { .. }) {
                    let cost = g
                        .observe(p)
                        .cost_choices
                        .into_iter()
                        .find(|choice| choice.cost.colored == pool)
                        .unwrap()
                        .index;
                    checked(&mut g, Action::ChooseSpellCost { cost });
                }
                settle(&mut g);
            } else {
                // Cost alternatives are tested after admission through public actions.
                g.state.players[p.index()].mana = [30; 6];
                cast_candidate(&mut g, duo, &[]);
                settle(&mut g);
            }
            assert_eq!(g.state.objects[duo as usize].zone, Zone::Battlefield);
            let before = g.state.players[p.index()].library.len();
            counters(&mut g, duo, Counters::of(CounterKind::Stun, 1));
            assert_eq!(g.state.players[p.index()].library.len(), before - 1);
            counters(&mut g, duo, Counters::of(CounterKind::PlusOne, 3));
            assert_eq!(g.state.players[p.index()].library.len(), before - 1);
            next_turn(&mut g);
            let before = g.state.players[p.index()].library.len();
            counters(&mut g, duo, Counters::of(CounterKind::Lore, 1));
            assert_eq!(g.state.players[p.index()].library.len(), before - 1);
        }
    }
}
#[test]
fn tmt_mutagen_mikey_leo_every_counter_kind_including_named_and_multiple_recipients() {
    for p in [Player::P0, Player::P1] {
        for kind in CounterKind::ALL {
            let mut g = setup(p);
            candidate(&mut g, p, "Mikey & Leo, Chaos & Order", Zone::Battlefield);
            let a = named(&mut g, p, "Aegis Turtle", Zone::Battlefield);
            let b = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
            let before = g.state.players[p.index()].library.len();
            g.effect_counters(
                vec![
                    (g.object_ref(a), Counters::of(kind, 1)),
                    (g.object_ref(b), Counters::of(kind, 1)),
                ],
                replacements::ResumeMode::Pending,
            );
            assert!(!g.advance_replacements(false));
            settle(&mut g);
            assert_eq!(
                g.state.players[p.index()].library.len(),
                before - 1,
                "{kind:?}"
            );
        }
    }
}
#[test]
fn tmt_mutagen_mikey_leo_actual_mutagen_cost_targets_either_player_but_only_own_creature_draws() {
    for p in [Player::P0, Player::P1] {
        for relation in [p, p.other()] {
            let mut g = setup(p);
            candidate(&mut g, p, "Mikey & Leo, Chaos & Order", Zone::Battlefield);
            let bear = named(&mut g, relation, "Grizzly Bears", Zone::Battlefield);
            let token = named(
                &mut g,
                p,
                "@token/7506aa37538b239479a880e5",
                Zone::Battlefield,
            );
            let before = g.state.players[p.index()].library.len();
            activate(&mut g, token, &[Target::Card(bear)]);
            settle(&mut g);
            assert_eq!(g.counter_count(bear, CounterKind::PlusOne), 1);
            assert_eq!(
                g.state.players[p.index()].library.len(),
                before - usize::from(relation == p)
            );
        }
    }
}
#[test]
fn tmt_mutagen_mikey_leo_opponent_placed_and_noncreature_counters_do_not_consume_once_limit() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let duo = candidate(&mut g, p, "Mikey & Leo, Chaos & Order", Zone::Battlefield);
        let token = food(&mut g, p);
        let before = g.state.players[p.index()].library.len();
        counters(&mut g, token, Counters::of(CounterKind::Charge, 1));
        let r = g.object_ref(duo);
        g.set_counters(duo, Counters::of(CounterKind::Stun, 1));
        g.self_counters_added(r, CounterKind::Stun, 1, p.other());
        settle(&mut g);
        g.self_counters_added(r, CounterKind::Stun, 0, p);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].library.len(), before);
        counters(&mut g, duo, Counters::of(CounterKind::Lore, 1));
        assert_eq!(g.state.players[p.index()].library.len(), before - 1);
    }
}
#[test]
fn tmt_mutagen_mikey_leo_countering_trigger_still_consumes_limit_but_new_incarnation_resets() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let duo = candidate(&mut g, p, "Mikey & Leo, Chaos & Order", Zone::Battlefield);
        let before = g.state.players[p.index()].library.len();
        g.effect_counters(
            vec![(g.object_ref(duo), Counters::of(CounterKind::Stun, 1))],
            replacements::ResumeMode::Pending,
        );
        assert!(!g.advance_replacements(false));
        g.prepare_triggers();
        // A trigger with no targets is stacked without a target announcement.
        assert_eq!(g.state.stack.len(), 1);
        let trigger = g.state.stack[0].id;
        assert!(g.counter_stack_item(trigger, false));
        settle(&mut g);
        counters(&mut g, duo, Counters::of(CounterKind::Charge, 1));
        assert_eq!(g.state.players[p.index()].library.len(), before);
        g.move_object(duo, Zone::Hand);
        for player in &mut g.state.players {
            player.mana = [30; 6];
        }
        cast_candidate(&mut g, duo, &[]);
        settle(&mut g);
        counters(&mut g, duo, Counters::of(CounterKind::Stun, 1));
        assert_eq!(g.state.players[p.index()].library.len(), before - 1);
    }
}
#[test]
fn tmt_mutagen_mikey_leo_ability_loss_and_control_are_current() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let duo = candidate(&mut g, p, "Mikey & Leo, Chaos & Order", Zone::Battlefield);
        let before = g.state.players[p.index()].library.len();
        let r = g.object_ref(duo);
        g.remove_abilities_exact(&[r], AnimationDuration::Permanent);
        counters(&mut g, duo, Counters::of(CounterKind::Charge, 1));
        assert_eq!(g.state.players[p.index()].library.len(), before);
    }
}
#[test]
fn tmt_mutagen_bff_paid_entry_attack_and_only_current_controlled_creatures_with_counters() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bff = candidate(&mut g, p, "Michelangelo, Mutant BFF", Zone::Hand);
        cast_candidate(&mut g, bff, &[]);
        settle(&mut g);
        assert_eq!(mutagens(&g, p), 1);
        let bear = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        let artifact = food(&mut g, p);
        for id in [bff, bear, enemy, artifact] {
            g.set_counters(id, Counters::of(CounterKind::Charge, 1));
        }
        for (id, expected) in [(bff, true), (bear, true), (enemy, false), (artifact, false)] {
            assert_eq!(g.keywords(id).contains(Keywords::MAX_ONE_BLOCKER), expected);
        }
        g.set_counters(bear, Counters::default());
        assert!(!g.keywords(bear).contains(Keywords::MAX_ONE_BLOCKER));
        next_turn(&mut g);
        next_turn(&mut g);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        checked(&mut g, Action::Attack { card: bff });
        checked(&mut g, Action::FinishAttackers);
        settle(&mut g);
        assert_eq!(mutagens(&g, p), 2);
        let r = g.object_ref(bff);
        g.remove_abilities_exact(&[r], AnimationDuration::Permanent);
        assert!(!g.keywords(bff).contains(Keywords::MAX_ONE_BLOCKER));
    }
}
#[test]
fn tmt_mutagen_bff_any_counter_including_named_and_tokens() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        candidate(&mut g, p, "Michelangelo, Mutant BFF", Zone::Battlefield);
        let bear = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        for kind in CounterKind::ALL {
            g.set_counters(bear, Counters::of(kind, 1));
            assert!(
                g.keywords(bear).contains(Keywords::MAX_ONE_BLOCKER),
                "{kind:?}"
            );
            assert_eq!(
                ability_evaluation::evaluate_abilities(&g).result.objects[bear as usize].keywords,
                g.keywords(bear)
            );
        }
    }
}
#[test]
fn tmt_mutagen_bff_max_one_is_enforced_and_with_menace_cannot_be_blocked() {
    for p in [Player::P0, Player::P1] {
        for menace in [false, true] {
            let mut g = setup(p);
            candidate(&mut g, p, "Michelangelo, Mutant BFF", Zone::Battlefield);
            let attacker = if menace {
                candidate(&mut g, p, "Super Shredder", Zone::Battlefield)
            } else {
                named(&mut g, p, "Grizzly Bears", Zone::Battlefield)
            };
            g.set_counters(attacker, Counters::of(CounterKind::Lore, 1));
            let a = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            let b = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            g.state.step = Step::Attackers;
            g.state.pending = Pending::Attackers;
            checked(&mut g, Action::Attack { card: attacker });
            checked(&mut g, Action::FinishAttackers);
            settle(&mut g);
            // settle stops at priority before the blocker declaration.
            while g.state.pending != Pending::Blockers {
                checked(&mut g, Action::Pass);
            }
            if menace {
                reject(
                    &mut g,
                    Action::Block {
                        blocker: a,
                        attacker,
                    },
                );
            } else {
                checked(
                    &mut g,
                    Action::Block {
                        blocker: a,
                        attacker,
                    },
                );
                reject(
                    &mut g,
                    Action::Block {
                        blocker: b,
                        attacker,
                    },
                );
                checked(&mut g, Action::FinishBlockers);
            }
        }
    }
}
#[test]
fn tmt_mutagen_chain_optional_zero_and_any_players_artifacts_enchantments_or_flyers() {
    for p in [Player::P0, Player::P1] {
        for which in 0..5 {
            let mut g = setup(p);
            let target = match which {
                0 => None,
                1 => Some(food(&mut g, p.other())),
                2 => Some(named(
                    &mut g,
                    p.other(),
                    "Leader's Talent",
                    Zone::Battlefield,
                )),
                3 => Some(named(&mut g, p.other(), "Serra Angel", Zone::Battlefield)),
                _ => Some(food(&mut g, p)),
            };
            let spell = candidate(&mut g, p, "Mutant Chain Reaction", Zone::Hand);
            cast_candidate(
                &mut g,
                spell,
                &target.map(Target::Card).into_iter().collect::<Vec<_>>(),
            );
            settle(&mut g);
            assert_eq!(mutagens(&g, p), 1);
            if let Some(id) = target {
                assert_ne!(g.state.objects[id as usize].zone, Zone::Battlefield);
            }
        }
    }
}
#[test]
fn tmt_mutagen_chain_rejects_plain_creatures_and_all_targets_illegal_prevents_token() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        let flyer = named(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
        let spell = candidate(&mut g, p, "Mutant Chain Reaction", Zone::Hand);
        if g.state.objects[spell as usize].card.is_supported() {
            checked(&mut g, Action::AnnounceCast { card: spell });
        } else {
            let item = g.cast_item(p, spell, [0; 6]);
            g.open_payment_announcement(item, Counters::default(), None);
        }
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(flyer),
            },
        );
        let payment = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, payment);
        let r = g.object_ref(flyer);
        g.remove_abilities_exact(&[r], AnimationDuration::Permanent);
        settle(&mut g);
        assert_eq!(mutagens(&g, p), 0);
        assert_eq!(g.state.objects[flyer as usize].zone, Zone::Battlefield);
    }
}
#[test]
fn tmt_mutagen_chain_valid_indestructible_and_nonflying_artifact_creature_still_create() {
    for p in [Player::P0, Player::P1] {
        for indestructible in [false, true] {
            let mut g = setup(p);
            let target = named(&mut g, p.other(), "Darksteel Colossus", Zone::Battlefield);
            if !indestructible {
                let r = g.object_ref(target);
                g.remove_abilities_exact(&[r], AnimationDuration::Permanent);
            }
            let spell = candidate(&mut g, p, "Mutant Chain Reaction", Zone::Hand);
            cast_candidate(&mut g, spell, &[Target::Card(target)]);
            settle(&mut g);
            assert_eq!(mutagens(&g, p), 1);
            assert_eq!(
                g.state.objects[target as usize].zone == Zone::Battlefield,
                indestructible
            );
        }
    }
}

#[test]
fn tmt_mutagen_bff_external_block_rule_survives_recipient_ability_removal_and_stops_on_source_loss()
{
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bff = candidate(&mut g, p, "Michelangelo, Mutant BFF", Zone::Battlefield);
        let bear = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        g.set_counters(bear, Counters::of(CounterKind::Charge, 1));
        let r = g.object_ref(bear);
        g.remove_abilities_exact(&[r], AnimationDuration::Permanent);
        assert!(g.keywords(bear).contains(Keywords::MAX_ONE_BLOCKER));
        assert_eq!(
            ability_evaluation::evaluate_abilities(&g).result.objects[bear as usize].keywords,
            g.keywords(bear)
        );
        let r = g.object_ref(bff);
        g.remove_abilities_exact(&[r], AnimationDuration::Permanent);
        assert!(!g.keywords(bear).contains(Keywords::MAX_ONE_BLOCKER));
    }
}
#[test]
fn tmt_mutagen_mikey_leo_actual_entry_counters_and_replaced_mixed_batch_trigger_once() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        candidate(&mut g, p, "Mikey & Leo, Chaos & Order", Zone::Battlefield);
        let artifact = food(&mut g, p);
        g.move_object(artifact, Zone::Graveyard);
        let pals = named(&mut g, p, "Putrid Pals", Zone::Hand);
        let before = g.state.players[p.index()].library.len();
        super::ability_timeline_tests::cast(&mut g, pals, None, &[]);
        settle(&mut g);
        assert_eq!(g.counter_count(pals, CounterKind::PlusOne), 2);
        assert_eq!(g.state.players[p.index()].library.len(), before - 1);
        next_turn(&mut g);
        named(&mut g, p, "Doubling Season", Zone::Battlefield);
        let before = g.state.players[p.index()].library.len();
        let mut mixed = Counters::of(CounterKind::Stun, 1);
        mixed.add(CounterKind::Charge, 3);
        counters(&mut g, pals, mixed);
        assert_eq!(g.counter_count(pals, CounterKind::Stun), 2);
        assert_eq!(g.counter_count(pals, CounterKind::Charge), 6);
        assert_eq!(g.state.players[p.index()].library.len(), before - 1);
    }
}
#[test]
fn tmt_mutagen_mikey_leo_prevented_counter_event_leaves_trigger_available() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let duo = candidate(&mut g, p, "Mikey & Leo, Chaos & Order", Zone::Battlefield);
        let bear = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let aura = named(&mut g, p, "Blossombind", Zone::Hand);
        super::ability_timeline_tests::cast(&mut g, aura, None, &[Target::Card(bear)]);
        settle(&mut g);
        let before = g.state.players[p.index()].library.len();
        counters(&mut g, bear, Counters::of(CounterKind::Charge, 1));
        assert_eq!(g.counter_count(bear, CounterKind::Charge), 0);
        assert_eq!(g.state.players[p.index()].library.len(), before);
        counters(&mut g, duo, Counters::of(CounterKind::Charge, 1));
        assert_eq!(g.state.players[p.index()].library.len(), before - 1);
    }
}
#[test]
fn tmt_mutagen_mikey_leo_control_change_draws_for_new_controller() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let duo = candidate(&mut g, p, "Mikey & Leo, Chaos & Order", Zone::Battlefield);
        next_turn(&mut g);
        while g.state.step != Step::Main1 {
            checked(&mut g, Action::Pass);
        }
        g.state.players[p.other().index()].mana = [30; 6];
        let aura = named(&mut g, p.other(), "Confiscate", Zone::Hand);
        super::ability_timeline_tests::cast(&mut g, aura, None, &[Target::Card(duo)]);
        settle(&mut g);
        assert_eq!(g.state.objects[duo as usize].controller, p.other());
        let before = g.state.players[p.other().index()].library.len();
        let original = g.state.players[p.index()].library.len();
        counters(&mut g, duo, Counters::of(CounterKind::Charge, 1));
        assert_eq!(g.state.players[p.other().index()].library.len(), before - 1);
        assert_eq!(g.state.players[p.index()].library.len(), original);
    }
}
