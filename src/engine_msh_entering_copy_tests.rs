//! Paid Ultron entry-copy rules. Bounded simultaneous/copy transitions are labeled.
use super::ability_world::CharacteristicWorld;
use super::*;
use crate::cards::{AnimationCreatureTypes, AnimationDuration, AnimationSpec, ChoiceColor};
use crate::engine::assurance::big_helpers::{cast, checked, named, next_turn, settle, setup};
const ULTRON: &str = "Ultron, Artificial Malevolence";
fn hand(g: &mut Game, p: Player, name: &str) -> CardId {
    named(g, p, name, Zone::Hand)
}

fn priority(g: &mut Game, p: Player) {
    if g.actor() != Some(p) {
        checked(g, Action::Pass);
    }
    assert_eq!(g.actor(), Some(p));
}
fn packet(g: &mut Game, p: Player, name: &str, mode: Option<u16>, targets: &[Target]) -> CardId {
    priority(g, p);
    let id = hand(g, p, name);
    let (colored, generic) = g.state.objects[id as usize].card.cost();
    g.state.players[p.index()].mana = colored;
    g.state.players[p.index()].mana[5] += generic;
    cast(g, id, mode, targets);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    id
}
fn paid(g: &mut Game, p: Player, name: &str) -> CardId {
    if g.state.active != p {
        next_turn(g);
        until(g, |g| g.state.active == p && g.state.step == Step::Main1);
    }
    let id = packet(g, p, name, None, &[]);
    settle(g);
    id
}
fn until(g: &mut Game, done: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..200 {
        if done(g) {
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
                        | Action::FinishTargets
                        | Action::PayCosts { .. }
                        | Action::FinishAttackers
                        | Action::FinishBlockers
                )
            })
            .unwrap_or_else(|| panic!("unhandled {:?}", g.state.pending));
        checked(g, a);
    }
    panic!("continuation did not complete");
}
fn reject(g: &mut Game, a: Action) {
    let snapshot = g.snapshot();
    let bytes = g.state_bytes();
    let views = serde_json::to_value((g.observe(Player::P0), g.observe(Player::P1))).unwrap();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), snapshot);
    assert_eq!(g.state_bytes(), bytes);
    assert_eq!(
        serde_json::to_value((g.observe(Player::P0), g.observe(Player::P1))).unwrap(),
        views
    );
}
fn views(g: &Game) -> [serde_json::Value; 2] {
    [Player::P0, Player::P1].map(|p| {
        let mut o = g.observe(p);
        o.revision = 0;
        serde_json::to_value(o).unwrap()
    })
}
fn orphan(g: Game, a: Action) -> Game {
    let actor = g.actor().unwrap();
    let rev = g.revision();
    let before = g.snapshot();
    let before_views = views(&g);
    let index = g.legal_actions().iter().position(|x| *x == a).unwrap();
    let (mut raw, mut indexed) = (g.clone(), g.clone());
    drop(g);
    raw.apply(actor, a.clone()).unwrap();
    indexed.apply_index(actor, rev, index).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(views(&raw), views(&indexed));
    let after = raw.snapshot();
    raw.restore(&before).unwrap();
    assert_eq!(views(&raw), before_views);
    assert_eq!(
        raw.apply_index(actor, rev, index),
        Err(Error::StaleRevision)
    );
    raw.apply(actor, a).unwrap();
    assert_eq!(raw.snapshot(), after);
    indexed
}

fn copies(g: &Game, p: Player, name: &str) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, o)| {
            (o.zone == Zone::Battlefield
                && o.controller == p
                && o.is_token()
                && g.object_view(id as CardId).name() == name)
                .then_some(id as CardId)
        })
        .collect()
}
fn await_pay(g: &mut Game, u: CardId) {
    until(g, |g| {
        g.state
            .resolving
            .as_ref()
            .is_some_and(|r| r.item.card == u && r.mana_payment.is_some())
    });
}
fn pay(g: &mut Game, p: Player, u: CardId) {
    await_pay(g, u);
    g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
    reject(g, Action::PayResolutionMana { payment: [0; 6] });
    checked(
        g,
        Action::PayResolutionMana {
            payment: [0, 0, 0, 0, 0, 2],
        },
    );
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
}
fn ready(p: Player) -> (Game, CardId) {
    let mut g = setup(p);
    let u = paid(&mut g, p, ULTRON);
    (g, u)
}
fn artifact_waiting(g: &mut Game, p: Player, u: CardId, name: &str) -> CardId {
    let id = packet(g, p, name, None, &[]);
    until(g, |g| {
        g.on_battlefield(id)
            && g.state.stack.iter().any(|i| i.card == u)
            && g.state.pending == Pending::Priority
    });
    id
}
fn animated(g: &Game, id: CardId) {
    assert_eq!(
        g.stats(id),
        Stats {
            power: 2,
            toughness: 2
        }
    );
    assert!(g.is_creature(id));
    assert!(g.current_kinds(id) & 16 != 0);
    assert!(g.has_subtype(id, "Robot"));
    assert!(g.has_subtype(id, "Villain"));
    let r = g.copiable_recipe(g.object_ref(id));
    assert_eq!(r.exceptions, CopyExceptions::NONE);
    assert!(r.overrides.is_none());
    assert_eq!(r.kinds() & 1, 0);
}
#[test]
fn msh_ultron_paid_body_cost_timing_and_self_entry_excluded() {
    for p in [Player::P0, Player::P1] {
        let (mut g, u) = ready(p);
        assert_eq!(
            g.stats(u),
            Stats {
                power: 2,
                toughness: 4
            }
        );
        assert!(g.current_kinds(u) & 16 != 0);
        assert!(g.object_view(u).is_legendary());
        assert!(g.has_subtype(u, "Robot"));
        assert!(g.has_subtype(u, "Villain"));
        assert!(g.state.pending_triggers.is_empty());
        let other = hand(&mut g, p, ULTRON);
        g.state.players[p.index()].mana = [0; 6];
        reject(&mut g, Action::AnnounceCast { card: other });
        next_turn(&mut g);
        until(&mut g, |g| {
            g.state.active == p.other() && g.state.step == Step::Main1
        });
        priority(&mut g, p);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
        reject(&mut g, Action::AnnounceCast { card: other });
    }
}
#[test]
fn msh_ultron_paid_optional_copy_enters_tapped_then_animates_permanently() {
    for p in [Player::P0, Player::P1] {
        let (mut g, u) = ready(p);
        packet(&mut g, p, "Arc Reactor", None, &[]);
        pay(&mut g, p, u);
        settle(&mut g);
        let v = copies(&g, p, "Arc Reactor");
        assert_eq!(v.len(), 1);
        animated(&g, v[0]);
        assert!(g.object_view(v[0]).tapped);
        next_turn(&mut g);
        assert_eq!(
            g.stats(v[0]),
            Stats {
                power: 2,
                toughness: 2
            }
        );
        assert_eq!(
            g.state
                .roles
                .as_ref()
                .unwrap()
                .animations
                .last()
                .unwrap()
                .expires_turn,
            None
        );
    }
}
#[test]
fn msh_ultron_decline_and_unfunded_payment_are_atomic() {
    for p in [Player::P0, Player::P1] {
        let (mut g, u) = ready(p);
        packet(&mut g, p, "Arc Reactor", None, &[]);
        await_pay(&mut g, u);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        reject(
            &mut g,
            Action::PayResolutionMana {
                payment: [0, 0, 0, 0, 0, 2],
            },
        );
        g = orphan(g, Action::DeclineResolutionMana);
        settle(&mut g);
        assert!(copies(&g, p, "Arc Reactor").is_empty());
    }
}
#[test]
fn msh_ultron_enemy_nonartifact_and_token_entries_do_not_trigger() {
    for p in [Player::P0, Player::P1] {
        let (mut g, u) = ready(p);
        let arc = packet(&mut g, p, "Arc Reactor", None, &[]);
        await_pay(&mut g, u);
        checked(&mut g, Action::DeclineResolutionMana);
        settle(&mut g);
        packet(&mut g, p, "Molten Duplication", None, &[Target::Card(arc)]);
        settle(&mut g);
        assert_eq!(copies(&g, p, "Arc Reactor").len(), 1);
        assert!(g.state.pending_triggers.is_empty());
        paid(&mut g, p, "Bear Cub");
        paid(&mut g, p.other(), "Arc Reactor");
        assert!(g.state.stack.is_empty());
        assert!(g.state.pending_triggers.is_empty());
        assert!(g.state.resolving.is_none());
    }
}
#[test]
fn msh_ultron_artifact_entry_observers_precede_noncreature_animation() {
    for p in [Player::P0, Player::P1] {
        let (mut g, u) = ready(p);
        let m = named(&mut g, p, "Machinesmith Automaton", Zone::Battlefield);
        named(&mut g, p, "Dazzling Angel", Zone::Battlefield);
        let life = g.life(p);
        let counters = g.counter_count(m, CounterKind::PlusOne);
        packet(&mut g, p, "Arc Reactor", None, &[]);
        pay(&mut g, p, u);
        settle(&mut g);
        assert_eq!(g.life(p), life);
        assert_eq!(g.counter_count(m, CounterKind::PlusOne), counters + 2);
        animated(&g, copies(&g, p, "Arc Reactor")[0]);
    }
}
#[test]
fn msh_ultron_artifact_creature_entry_and_token_keep_original_creature_values() {
    for p in [Player::P0, Player::P1] {
        let (mut g, u) = ready(p);
        named(&mut g, p, "Dazzling Angel", Zone::Battlefield);
        let life = g.life(p);
        packet(&mut g, p, "Hydraulic Helper", None, &[]);
        pay(&mut g, p, u);
        settle(&mut g);
        let v = copies(&g, p, "Hydraulic Helper");
        assert_eq!(v.len(), 1);
        assert_eq!(
            g.stats(v[0]),
            Stats {
                power: 2,
                toughness: 3
            }
        );
        assert!(g.keywords(v[0]).contains(Keywords::DEFENDER));
        assert!(!g.has_subtype(v[0], "Villain"));
        assert_eq!(g.life(p), life + 2);
        assert!(
            g.state
                .roles
                .as_ref()
                .is_none_or(|r| r.animations.is_empty())
        );
    }
}
#[test]
fn msh_ultron_actual_copy_of_animated_token_does_not_inherit_post_entry_animation() {
    for p in [Player::P0, Player::P1] {
        let (mut g, u) = ready(p);
        packet(&mut g, p, "Arc Reactor", None, &[]);
        pay(&mut g, p, u);
        settle(&mut g);
        let original = copies(&g, p, "Arc Reactor")[0];
        packet(
            &mut g,
            p,
            "Molten Duplication",
            None,
            &[Target::Card(original)],
        );
        settle(&mut g);
        let v = copies(&g, p, "Arc Reactor");
        assert_eq!(v.len(), 2);
        let new = *v.iter().find(|&&id| id != original).unwrap();
        assert!(!g.is_creature(new));
        assert!(!g.has_subtype(new, "Robot"));
        animated(&g, original);
    }
}
#[test]
fn msh_ultron_source_departure_does_not_cancel_independent_paid_trigger() {
    for p in [Player::P0, Player::P1] {
        let (mut g, u) = ready(p);
        artifact_waiting(&mut g, p, u, "Arc Reactor");
        packet(&mut g, p.other(), "Murder", None, &[Target::Card(u)]);
        await_pay(&mut g, u);
        assert!(!g.on_battlefield(u));
        pay(&mut g, p, u);
        settle(&mut g);
        assert_eq!(copies(&g, p, "Arc Reactor").len(), 1);
    }
}
#[test]
fn msh_ultron_latest_departure_copiable_recipe_survives_destroyed_original() {
    for p in [Player::P0, Player::P1] {
        let (mut g, u) = ready(p);
        let arc = artifact_waiting(&mut g, p, u, "Arc Reactor");
        packet(&mut g, p.other(), "Disenchant", None, &[Target::Card(arc)]);
        await_pay(&mut g, u);
        assert_eq!(g.object_view(arc).zone, Zone::Graveyard);
        pay(&mut g, p, u);
        settle(&mut g);
        animated(&g, copies(&g, p, "Arc Reactor")[0]);
    }
}
#[test]
fn msh_ultron_current_changed_copy_recipe_and_departure_lki_override_entry_recipe() {
    for p in [Player::P0, Player::P1] {
        for depart in [false, true] {
            let (mut g, u) = ready(p);
            let arc = artifact_waiting(&mut g, p, u, "Arc Reactor");
            let reference = g.object_ref(arc);
            g.apply_temporary_copy(
                reference,
                CopyRecipe::new(Card::from_name("Hydraulic Helper").unwrap()),
                CopyDeadline::EndOfTurn,
            );
            if depart {
                packet(&mut g, p.other(), "Disenchant", None, &[Target::Card(arc)]);
            }
            pay(&mut g, p, u);
            settle(&mut g);
            let v = copies(&g, p, "Hydraulic Helper");
            assert_eq!(v.len(), 1);
            assert_eq!(
                g.stats(v[0]),
                Stats {
                    power: 2,
                    toughness: 3
                }
            );
            assert!(!g.has_subtype(v[0], "Villain"));
        }
    }
}
#[test]
fn msh_ultron_noncopiable_animation_and_counters_are_excluded_from_blueprint() {
    for p in [Player::P0, Player::P1] {
        let (mut g, u) = ready(p);
        let arc = artifact_waiting(&mut g, p, u, "Arc Reactor");
        let reference = g.object_ref(arc);
        g.change_object_form(
            reference,
            AnimationSpec {
                artifact: false,
                creature_types: AnimationCreatureTypes::Named(&["Golem"]),
                base: Some((6, 6)),
                keywords: Keywords::HASTE,
                duration: AnimationDuration::Permanent,
            },
            characteristic_layers::AnimationTypeMode::AddCreature,
        );
        g.effect_counter(
            arc,
            CounterKind::PlusOne,
            2,
            replacements::ResumeMode::Pending,
        );
        assert_eq!(g.stats(arc).power, 8);
        pay(&mut g, p, u);
        settle(&mut g);
        let v = copies(&g, p, "Arc Reactor");
        animated(&g, v[0]);
        assert!(!g.keywords(v[0]).contains(Keywords::HASTE));
        assert!(!g.has_subtype(v[0], "Golem"));
        assert_eq!(g.counter_count(v[0], CounterKind::PlusOne), 0);
    }
}
#[test]
fn msh_ultron_source_suppression_then_restoration_controls_new_entries() {
    for p in [Player::P0, Player::P1] {
        let (mut g, u) = ready(p);
        let aura = packet(&mut g, p, "Witness Protection", None, &[Target::Card(u)]);
        settle(&mut g);
        paid(&mut g, p, "Arc Reactor");
        assert!(copies(&g, p, "Arc Reactor").is_empty());
        packet(&mut g, p, "Disenchant", None, &[Target::Card(aura)]);
        settle(&mut g);
        packet(&mut g, p, "Arc Reactor", None, &[]);
        pay(&mut g, p, u);
        settle(&mut g);
        assert_eq!(copies(&g, p, "Arc Reactor").len(), 1);
    }
}
#[test]
fn msh_ultron_replacement_added_map_is_also_actual_post_entry_recipient() {
    for p in [Player::P0, Player::P1] {
        let (mut g, u) = ready(p);
        named(&mut g, p, "Worldwalker Helm", Zone::Battlefield);
        packet(&mut g, p, "Arc Reactor", None, &[]);
        pay(&mut g, p, u);
        settle(&mut g);
        animated(&g, copies(&g, p, "Arc Reactor")[0]);
        let maps = g
            .state
            .objects
            .iter()
            .enumerate()
            .filter_map(|(id, o)| {
                (o.zone == Zone::Battlefield && o.is_token() && g.has_subtype(id as CardId, "Map"))
                    .then_some(id as CardId)
            })
            .collect::<Vec<_>>();
        assert_eq!(maps.len(), 1);
        assert!(g.is_creature(maps[0]));
        assert!(g.has_subtype(maps[0], "Robot"));
        assert_eq!(
            g.stats(maps[0]),
            Stats {
                power: 2,
                toughness: 2
            }
        );
    }
}
#[test]
fn msh_ultron_competing_token_replacement_orders_preserve_creature_entry_status() {
    for p in [Player::P0, Player::P1] {
        for helm_first in [false, true] {
            let (mut g, u) = ready(p);
            let helm = named(&mut g, p, "Worldwalker Helm", Zone::Battlefield);
            let dragon = named(&mut g, p, "Draconic Visitor", Zone::Battlefield);
            named(&mut g, p, "Dazzling Angel", Zone::Battlefield);
            let life = g.life(p);
            packet(&mut g, p, "Arc Reactor", None, &[]);
            pay(&mut g, p, u);
            let a = Action::ChooseReplacement {
                source: g.object_ref(if helm_first { helm } else { dragon }),
                ability: 0,
            };
            g = orphan(g, a);
            settle(&mut g);
            let tokens = g
                .state
                .objects
                .iter()
                .enumerate()
                .filter_map(|(id, o)| {
                    (o.zone == Zone::Battlefield && o.controller == p && o.is_token())
                        .then_some(id as CardId)
                })
                .collect::<Vec<_>>();
            assert_eq!(tokens.len(), if helm_first { 2 } else { 1 });
            for id in tokens {
                assert!(g.has_subtype(id, "Dragon"));
                assert!(!g.has_subtype(id, "Robot"));
                assert!(g.current_kinds(id) & 16 == 0);
            }
            assert_eq!(g.life(p), life + if helm_first { 2 } else { 1 });
        }
    }
}
#[test]
fn msh_ultron_actual_entry_color_choice_defers_animation_and_is_owned() {
    for p in [Player::P0, Player::P1] {
        let (mut g, u) = ready(p);
        packet(&mut g, p, "Heraldic Banner", None, &[]);
        until(&mut g, |g| {
            g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::ChooseEntryValue { .. }))
        });
        checked(
            &mut g,
            Action::ChooseEntryValue {
                value: EntryChoiceValue::Color(ChoiceColor::Blue),
            },
        );
        pay(&mut g, p, u);
        assert!(copies(&g, p, "Heraldic Banner").is_empty());
        assert!(g.state.entry_batch.is_some());
        g = orphan(
            g,
            Action::ChooseEntryValue {
                value: EntryChoiceValue::Color(ChoiceColor::Red),
            },
        );
        settle(&mut g);
        let v = copies(&g, p, "Heraldic Banner");
        animated(&g, v[0]);
        assert_eq!(
            g.entry_value(g.object_ref(v[0])),
            Some(EntryChoiceValue::Color(ChoiceColor::Red))
        );
    }
}
#[test]
fn msh_ultron_resolution_mana_abilities_pay_real_cost_and_reserve_taps() {
    for p in [Player::P0, Player::P1] {
        let (mut g, u) = ready(p);
        let a = named(&mut g, p, "Island", Zone::Battlefield);
        let b = named(&mut g, p, "Island", Zone::Battlefield);
        packet(&mut g, p, "Arc Reactor", None, &[]);
        await_pay(&mut g, u);
        checked(&mut g, Action::TapMana { card: a });
        checked(&mut g, Action::TapMana { card: b });
        reject(&mut g, Action::TapMana { card: a });
        checked(
            &mut g,
            Action::PayResolutionMana {
                payment: [0, 2, 0, 0, 0, 0],
            },
        );
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(copies(&g, p, "Arc Reactor").len(), 1);
    }
}
#[test]
fn msh_ultron_owned_optional_payment_and_full_nonactor_privacy() {
    for p in [Player::P0, Player::P1] {
        let (mut g, u) = ready(p);
        packet(&mut g, p, "Arc Reactor", None, &[]);
        await_pay(&mut g, u);
        let secret = named(&mut g, p, "Murder", Zone::Hand);
        let mut other = g.clone();
        other.state.objects[secret as usize].card = Card::from_name("Negate").unwrap();
        assert_eq!(
            serde_json::to_value(g.observe(p.other())).unwrap(),
            serde_json::to_value(other.observe(p.other())).unwrap()
        );
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        g = orphan(
            g,
            Action::PayResolutionMana {
                payment: [0, 0, 0, 0, 0, 2],
            },
        );
        settle(&mut g);
        animated(&g, copies(&g, p, "Arc Reactor")[0]);
    }
}

#[test]
fn msh_ultron_copied_trigger_owns_independent_resolution_payment_without_extra_entry() {
    for p in [Player::P0, Player::P1] {
        let (mut g, u) = ready(p);
        let echo = paid(&mut g, p, "Echo, Perceptive Prodigy");
        next_turn(&mut g);
        next_turn(&mut g);
        until(&mut g, |g| {
            g.state.active == p && g.state.step == Step::Main1
        });
        artifact_waiting(&mut g, p, u, "Arc Reactor");
        let original = g.state.stack.iter().find(|i| i.card == u).unwrap().clone();
        priority(&mut g, p);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: echo,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Stack(original.id),
            },
        );
        while g.state.announcing.is_some() {
            let a = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
                .unwrap();
            checked(&mut g, a);
        }
        pay(&mut g, p, u);
        assert_eq!(copies(&g, p, "Arc Reactor").len(), 1);
        pay(&mut g, p, u);
        settle(&mut g);
        assert_eq!(copies(&g, p, "Arc Reactor").len(), 2);
        assert!(g.state.stack.is_empty());
    }
}
#[test]
fn msh_ultron_bounded_new_observer_and_artifacts_enter_one_simultaneous_batch() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let u = hand(&mut g, p, ULTRON);
        let a = hand(&mut g, p, "Arc Reactor");
        let b = hand(&mut g, p, "Hydraulic Helper");
        let moves = [u, a, b]
            .into_iter()
            .map(|id| g.entry_move(id, Zone::Battlefield, false))
            .collect();
        g.stage_entries(moves, None);
        g.advance_entries();
        let packets = g
            .state
            .pending_triggers
            .iter()
            .chain(&g.state.ordering_triggers)
            .chain(&g.state.stack)
            .filter(|i| i.card == u)
            .collect::<Vec<_>>();
        assert_eq!(packets.len(), 2);
        assert!(
            packets
                .iter()
                .all(|i| i.entering_object != Some(g.object_ref(u)))
        );
        assert_ne!(
            packets[0].event_object.as_ref().unwrap().object,
            packets[1].event_object.as_ref().unwrap().object
        );
        pay(&mut g, p, u);
        pay(&mut g, p, u);
        settle(&mut g);
        assert_eq!(copies(&g, p, "Arc Reactor").len(), 1);
        assert_eq!(copies(&g, p, "Hydraulic Helper").len(), 1);
    }
}
#[test]
fn msh_ultron_bounded_departure_reentry_does_not_rebind_old_entrant_recipe() {
    for p in [Player::P0, Player::P1] {
        let (mut g, u) = ready(p);
        let a = artifact_waiting(&mut g, p, u, "Arc Reactor");
        let old = g.object_ref(a);
        g.apply_temporary_copy(
            old,
            CopyRecipe::new(Card::from_name("Hydraulic Helper").unwrap()),
            CopyDeadline::EndOfTurn,
        );
        g.move_object(a, Zone::Graveyard);
        g.move_object(a, Zone::Battlefield);
        assert_ne!(g.object_ref(a), old);
        pay(&mut g, p, u);
        pay(&mut g, p, u);
        settle(&mut g);
        assert_eq!(copies(&g, p, "Arc Reactor").len(), 1);
        assert_eq!(copies(&g, p, "Hydraulic Helper").len(), 1);
    }
}
#[test]
fn msh_ultron_copied_single_face_structure_is_preserved_in_current_and_departure_recipe() {
    for p in [Player::P0, Player::P1] {
        for depart in [false, true] {
            let (mut g, u) = ready(p);
            let a = artifact_waiting(&mut g, p, u, "Arc Reactor");
            g.apply_temporary_copy(
                g.object_ref(a),
                CopyRecipe::new(Card::from_name("Monica Rambeau").unwrap()),
                CopyDeadline::EndOfTurn,
            );
            if depart {
                packet(&mut g, p.other(), "Murder", None, &[Target::Card(a)]);
            }
            pay(&mut g, p, u);
            // The two actual legendary copies require the real public legend choice.
            if let Some(a) = g.legal_actions().into_iter().find(|a| {
                matches!(a,
                Action::KeepLegend { object } if g.object_view(object.card).is_token())
            }) {
                checked(&mut g, a);
            }
            settle(&mut g);
            let v = copies(&g, p, "Monica Rambeau");
            assert_eq!(v.len(), 1);
            assert!(g.object_view(v[0]).face.is_none());
            assert!(g.object_view(v[0]).is_token());
        }
    }
}
