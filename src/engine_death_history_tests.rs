use super::tests::{act, cast, scenario, spawn};
use super::*;
fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let c = Card::from_name(name).unwrap();
    assert!(c.is_supported(), "{name}");
    spawn(g, p, c, zone)
}
fn replay(g: &mut Game, a: Action) {
    let saved = g.snapshot();
    let revision = g.revision();
    let actor = g.actor().unwrap();
    let mut fork = g.clone();
    act(&mut fork, a.clone());
    act(g, a.clone());
    let expected = g.snapshot();
    assert_eq!(fork.snapshot(), expected);
    g.restore(&saved).unwrap();
    assert_eq!(g.apply_index(actor, revision, 0), Err(Error::StaleRevision));
    act(g, a);
    assert_eq!(g.snapshot(), expected);
}
fn passes(g: &mut Game) {
    replay(g, Action::Pass);
    replay(g, Action::Pass);
}
fn die(g: &mut Game, p: Player) -> CardId {
    let id = named(g, p, "Ordinary Bear", Zone::Battlefield);
    g.move_object(id, Zone::Graveyard);
    id
}
fn history(g: &Game, n: u64) {
    assert_eq!(g.state.creatures_died_this_turn, n);
    for p in [Player::P0, Player::P1] {
        let obs = g.observe(p);
        assert_eq!(obs.schema, 51);
        assert_eq!(obs.creatures_died_this_turn, n);
    }
}
fn fund(g: &mut Game, name: &str) -> CardId {
    let id = named(g, Player::P0, name, Zone::Hand);
    g.state.players[0].mana = [10; 6];
    id
}
fn trigger(g: &mut Game, target: Option<Target>) {
    g.prepare_triggers();
    for _ in 0..30 {
        if g.state.pending == Pending::Priority {
            return;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| match a {
                Action::ChooseTarget { target: t } => Some(*t) == target,
                Action::PutTrigger { .. } | Action::FinishTargets => true,
                _ => false,
            })
            .expect("trigger action");
        replay(g, action);
    }
    panic!("trigger announcement stalled");
}

#[test]
fn only_actual_creature_battlefield_to_graveyard_transitions_count_for_either_owner() {
    let mut g = scenario();
    for p in [Player::P0, Player::P1] {
        for (name, from, to) in [
            ("Ordinary Bear", Zone::Hand, Zone::Graveyard),
            ("Ordinary Bear", Zone::Library, Zone::Graveyard),
            ("Ordinary Bear", Zone::Stack, Zone::Graveyard),
            ("Ordinary Bear", Zone::Battlefield, Zone::Exile),
            ("Ordinary Bear", Zone::Battlefield, Zone::Hand),
            ("Gold Pan", Zone::Battlefield, Zone::Graveyard),
        ] {
            let id = named(&mut g, p, name, from);
            g.move_object(id, to);
        }
    }
    history(&g, 0);
    let before = g.snapshot();
    let mut fork = g.clone();
    let a = die(&mut g, Player::P0);
    let b = die(&mut g, Player::P1);
    history(&g, 2);
    history(&fork, 0);
    g.move_object(a, Zone::Exile);
    g.move_object(b, Zone::Hand);
    history(&g, 2);
    let saved = g.snapshot();
    g.begin_turn();
    history(&g, 0);
    g.restore(&saved).unwrap();
    history(&g, 2);
    fork.restore(&before).unwrap();
    history(&fork, 0);
}

#[test]
fn token_deaths_and_reentered_incarnations_each_count_once_per_actual_death() {
    let mut g = scenario();
    let token = Card::catalogue()
        .find(|c| c.token_info().is_some_and(|i| i.name == "Sliver Token"))
        .unwrap();
    g.create_tokens(Player::P1, token, 1, false);
    let id = g
        .state
        .objects
        .iter()
        .position(|o| o.card == token && o.zone == Zone::Battlefield)
        .unwrap() as CardId;
    let old = g.object_ref(id);
    g.move_object(id, Zone::Graveyard);
    history(&g, 1);
    g.state_based_actions();
    history(&g, 1);
    g.create_tokens(Player::P0, token, 1, false);
    assert_ne!(g.object_ref(id), old);
    g.move_object(id, Zone::Graveyard);
    history(&g, 2);
    let creature = die(&mut g, Player::P0);
    g.move_object(creature, Zone::Battlefield);
    g.move_object(creature, Zone::Graveyard);
    history(&g, 4);
}

#[test]
fn spell_damage_simultaneous_destruction_and_zero_toughness_sba_record_committed_deaths() {
    let mut g = scenario();
    let victim = named(&mut g, Player::P1, "Savannah Lions", Zone::Battlefield);
    let shock = fund(&mut g, "Shock");
    cast(&mut g, shock, Some(Target::Card(victim)));
    history(&g, 0);
    passes(&mut g);
    history(&g, 1);
    named(&mut g, Player::P0, "Ordinary Bear", Zone::Battlefield);
    named(&mut g, Player::P1, "Ordinary Bear", Zone::Battlefield);
    let wipe = fund(&mut g, "Day of Judgment");
    cast(&mut g, wipe, None);
    passes(&mut g);
    history(&g, 3);
    let weak = named(&mut g, Player::P0, "Savannah Lions", Zone::Battlefield);
    g.state.objects[weak as usize].counters.minus_one = 1;
    g.state_based_actions();
    history(&g, 4);
}

#[test]
fn all_simultaneous_deaths_precede_linked_return_entry_replacements_in_either_order() {
    for reverse in [false, true] {
        let mut g = scenario();
        let slasher = named(&mut g, Player::P1, "Cackling Slasher", Zone::Battlefield);
        let light = named(&mut g, Player::P0, "Banishing Light", Zone::Hand);
        g.move_object(light, Zone::Battlefield);
        trigger(&mut g, Some(Target::Card(slasher)));
        passes(&mut g);
        assert_eq!(g.object_view(slasher).zone, Zone::Exile);
        history(&g, 0);
        let victim = named(&mut g, Player::P0, "Ordinary Bear", Zone::Battlefield);
        let mut ids = [light, victim];
        if reverse {
            ids.reverse();
        }
        let sources = g.death_trigger_sources();
        let events = g.move_objects_raw(&ids, Zone::Graveyard);
        g.finish_zone_events(&events, &sources);
        history(&g, 1);
        assert_eq!(g.object_view(slasher).zone, Zone::Battlefield);
        assert_eq!(g.object_view(slasher).counters.plus_one, 1);
    }
}

#[test]
fn slasher_conditional_counter_is_present_on_cast_resolution_and_reentry() {
    for died in [false, true] {
        let mut g = scenario();
        if died {
            die(&mut g, Player::P1);
        }
        let id = fund(&mut g, "Cackling Slasher");
        cast(&mut g, id, None);
        passes(&mut g);
        let o = g.object_view(id);
        assert_eq!(o.counters.plus_one, u64::from(died));
        assert_eq!(
            (o.power, o.toughness),
            (3 + i128::from(died), 3 + i128::from(died))
        );
        assert!(o.keywords.contains(Keywords::DEATHTOUCH));
        g.move_object(id, Zone::Hand);
        g.begin_turn();
        g.move_object(id, Zone::Battlefield);
        assert_eq!(g.object_view(id).counters.plus_one, 0);
    }
}

#[test]
fn bat_discount_uses_global_history_and_validates_exact_staged_colored_payment() {
    for died in [false, true] {
        let mut g = scenario();
        if died {
            die(&mut g, Player::P1);
        }
        let bat = fund(&mut g, "Dreaded Bat-Cloud");
        replay(&mut g, Action::AnnounceCast { card: bat });
        let request = g.observe(Player::P0).payment_request.unwrap();
        assert_eq!(request.generic, if died { 1 } else { 4 });
        assert_eq!(request.required, [0, 0, 1, 0, 0, 0]);
        let saved = g.snapshot();
        let rev = g.revision();
        assert_eq!(
            g.apply(Player::P0, Action::PayCosts { payment: [0; 6] }),
            Err(Error::IllegalAction)
        );
        assert_eq!(g.snapshot(), saved);
        assert_eq!(g.revision(), rev);
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        if let Action::PayCosts { payment } = action {
            assert_eq!(payment.into_iter().sum::<u32>(), if died { 2 } else { 5 });
        }
        replay(&mut g, action);
        passes(&mut g);
        assert_eq!(
            (g.object_view(bat).power, g.object_view(bat).toughness),
            (4, 2)
        );
        assert!(g.keywords(bat).contains(Keywords::FLYING));
        assert!(g.keywords(bat).contains(Keywords::DEATHTOUCH));
    }
}

#[test]
fn morbid_end_step_guards_respect_beginning_condition_controller_and_target_ownership() {
    for name in ["Cackling Prowler", "Needletooth Pack"] {
        for died in [false, true] {
            let mut g = scenario();
            let source = named(&mut g, Player::P0, name, Zone::Battlefield);
            let target = named(&mut g, Player::P0, "Ordinary Bear", Zone::Battlefield);
            let opponent = named(&mut g, Player::P1, "Ordinary Bear", Zone::Battlefield);
            if died {
                die(&mut g, Player::P1);
            }
            g.state.active = Player::P1;
            g.begin_step(Step::End);
            assert!(g.state.pending_triggers.is_empty());
            g.state.active = Player::P0;
            g.begin_step(Step::End);
            assert_eq!(g.state.pending_triggers.len(), usize::from(died));
            if !died {
                die(&mut g, Player::P0);
                assert!(g.state.pending_triggers.is_empty());
                continue;
            }
            if name == "Needletooth Pack" {
                g.prepare_triggers();
                let saved = g.snapshot();
                let rev = g.revision();
                assert_eq!(
                    g.apply(
                        Player::P0,
                        Action::ChooseTarget {
                            target: Target::Card(opponent)
                        }
                    ),
                    Err(Error::IllegalAction)
                );
                assert_eq!(g.snapshot(), saved);
                assert_eq!(g.revision(), rev);
            }
            trigger(&mut g, Some(Target::Card(target)));
            passes(&mut g);
            assert_eq!(
                g.object_view(if name == "Cackling Prowler" {
                    source
                } else {
                    target
                })
                .counters
                .plus_one,
                if name == "Cackling Prowler" { 1 } else { 2 }
            );
            if name == "Cackling Prowler" {
                assert!(
                    g.object_view(source)
                        .card
                        .triggers()
                        .iter()
                        .any(|a| a.condition == TriggerCondition::Ward
                            && matches!(
                                a.program[0],
                                Instruction::CounterUnlessPay {
                                    cost: ManaCost { generic: 2, .. },
                                    ..
                                }
                            ))
                );
            }
        }
    }
}

#[test]
fn scorpion_morbid_target_connive_suspends_privately_and_replays_without_another_draw() {
    let mut g = scenario();
    named(
        &mut g,
        Player::P0,
        "Scorpion, Seething Striker",
        Zone::Battlefield,
    );
    let target = named(&mut g, Player::P0, "Ordinary Bear", Zone::Battlefield);
    let discard = named(&mut g, Player::P0, "Ordinary Bear", Zone::Hand);
    die(&mut g, Player::P1);
    g.begin_step(Step::End);
    trigger(&mut g, Some(Target::Card(target)));
    let library = g.state.players[0].library.len();
    passes(&mut g);
    assert_eq!(g.observe(Player::P0).decision, "effect_discard");
    assert_eq!(g.state.players[0].library.len(), library - 1);
    assert!(g.observe(Player::P1).actions.is_empty());
    replay(&mut g, Action::Discard { card: discard });
    assert_eq!(g.state.players[0].library.len(), library - 1);
    assert_eq!(g.object_view(target).counters.plus_one, 1);
    history(&g, 1);
}

#[test]
fn death_history_capacity_failure_restores_the_entire_simultaneous_action() {
    let mut g = scenario();
    named(&mut g, Player::P0, "Ordinary Bear", Zone::Battlefield);
    named(&mut g, Player::P1, "Ordinary Bear", Zone::Battlefield);
    let wipe = fund(&mut g, "Day of Judgment");
    cast(&mut g, wipe, None);
    replay(&mut g, Action::Pass);
    g.state.creatures_died_this_turn = u64::MAX - 1;
    let saved = g.snapshot();
    let rev = g.revision();
    for _ in 0..2 {
        assert_eq!(
            g.apply(g.actor().unwrap(), Action::Pass),
            Err(Error::CapacityExceeded(crate::CapacityExceeded {
                resource: "creature death history exhausted"
            }))
        );
        assert_eq!(g.snapshot(), saved);
        assert_eq!(g.revision(), rev);
    }
}

#[test]
fn real_combat_deaths_and_additional_cost_sacrifice_advance_history_before_spell_completion() {
    let mut g = scenario();
    let attacker = named(&mut g, Player::P0, "Savannah Lions", Zone::Battlefield);
    let blocker = named(&mut g, Player::P1, "Savannah Lions", Zone::Battlefield);
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    replay(&mut g, Action::Attack { card: attacker });
    replay(&mut g, Action::FinishAttackers);
    passes(&mut g);
    replay(&mut g, Action::Block { blocker, attacker });
    replay(&mut g, Action::FinishBlockers);
    passes(&mut g);
    // A single blocker has an automatic damage assignment.
    history(&g, 2);
    g.state.step = Step::Main2;
    g.give_priority();
    let victim = named(&mut g, Player::P0, "Ordinary Bear", Zone::Battlefield);
    let spell = fund(&mut g, "Corrupted Conviction");
    replay(&mut g, Action::AnnounceCast { card: spell });
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::ChooseAdditionalCost { .. }))
        .unwrap();
    replay(&mut g, a);
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    replay(&mut g, a);
    assert_eq!(g.object_view(victim).zone, Zone::Graveyard);
    history(&g, 3);
    assert!(g.state.stack.iter().any(|s| s.card == spell));
}
