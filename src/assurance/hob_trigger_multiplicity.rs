//! Additional trigger instances are source-qualified events, not stack copies.
use super::*;
use crate::cards::{AnimationCreatureTypes, AnimationDuration, AnimationSpec};
use crate::engine::subtypes::CreatureTypesEffectView;

fn game(p: Player) -> Game {
    big_helpers::setup(p)
}
fn staff(g: &mut Game, p: Player, recipient: CardId) -> CardId {
    let id = put(g, p, "Wizard's Staff", Zone::Battlefield);
    assert!(g.attach_equipment(g.object_ref(id), g.object_ref(recipient)));
    id
}
fn story(g: &mut Game, p: Player) -> CardId {
    let bifur = put(g, p, "Bifur, Melodic Rider", Zone::Battlefield);
    put(g, p, "Goldvein Pick", Zone::Battlefield);
    put(g, p, "Goldvein Pick", Zone::Battlefield);
    g.refresh_enduring_stories();
    assert!(g.state.players[p.index()].enduring_story);
    bifur
}
fn dwarf(g: &mut Game, id: CardId) {
    let reference = g.object_ref(id);
    let timestamp = g.next_role_timestamp();
    g.state
        .roles
        .get_or_insert_with(Default::default)
        .creature_types
        .push(CreatureTypesEffectView {
            recipient: reference,
            source: reference,
            types: &["Dwarf"],
            additional: false,
            timestamp,
        });
}
fn animate(g: &mut Game, id: CardId) {
    g.change_object_form(
        g.object_ref(id),
        AnimationSpec {
            base: Some((2, 2)),
            artifact: true,
            creature_types: AnimationCreatureTypes::Named(&[]),
            keywords: Keywords::NONE,
            duration: AnimationDuration::Permanent,
        },
        characteristic_layers::AnimationTypeMode::AddCreature,
    );
}
fn items(g: &Game, id: CardId) -> Vec<&SpellView> {
    g.state
        .pending_triggers
        .iter()
        .chain(&g.state.ordering_triggers)
        .chain(&g.state.stack)
        .chain(g.state.announcing.iter().map(|a| &a.item))
        .chain(g.state.resolving.iter().map(|r| &r.item))
        .filter(|s| s.card == id && matches!(s.kind, StackKind::TriggeredAbility { .. }))
        .collect()
}
fn drive(g: &mut Game, targets: &[CardId]) {
    let mut target = 0usize;
    for _ in 0..96 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.resolving.is_none()
        {
            return;
        }
        let actions = g.legal_actions();
        let action = if actions.iter().any(|a| matches!(a, Action::ChooseTarget { .. })) {
            let wanted = targets.get(target).copied();
            target += 1;
            actions.iter().find(|a| matches!(a, Action::ChooseTarget { target: Target::Card(id) } if Some(*id) == wanted)).or_else(|| actions.iter().find(|a| matches!(a, Action::ChooseTarget { .. })))
        } else {
            actions.iter().find(|a| matches!(a, Action::PutTrigger { .. } | Action::FinishTargets | Action::PayCosts { .. } | Action::Pass))
        }.unwrap_or_else(|| panic!("explicit choice required: {:?}", g.state.pending)).clone();
        act(g, action);
    }
    panic!("trigger driver budget");
}
fn until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..64 {
        if ready(g) {
            return;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::PutTrigger { .. }
                        | Action::FinishTargets
                        | Action::PayCosts { .. }
                        | Action::Pass
                )
            })
            .unwrap_or_else(|| panic!("unexpected {:?}", g.state.pending));
        act(g, action);
    }
    panic!("choice budget");
}
fn start(g: &mut Game, id: CardId, target: Option<CardId>) {
    act(g, Action::AnnounceCast { card: id });
    if matches!(g.state.pending, Pending::SpellCost { .. }) {
        choose(g, |a| matches!(a, Action::ChooseSpellCost { .. }));
    }
    if let Some(id) = target {
        act(
            g,
            Action::ChooseTarget {
                target: Target::Card(id),
            },
        );
    }
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
    pay(g);
}
fn normal_equip(g: &mut Game, equipment: CardId, ability: u16, target: CardId) {
    act(
        g,
        Action::AnnounceActivate {
            card: equipment,
            ability,
            counter_payment: Counters::default(),
        },
    );
    act(
        g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
    pay(g);
    drive(g, &[]);
}

#[test]
fn hob_extra_triggers_bifur_paid_hybrid_threshold_entry_and_attack_independent_targets() {
    for p in [Player::P0, Player::P1] {
        for colors in [[2, 0, 0, 0, 0, 4], [1, 0, 0, 1, 0, 4], [0, 0, 0, 2, 0, 4]] {
            let mut g = game(p);
            put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
            put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
            let enemy = put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
            let bifur = put(&mut g, p, "Bifur, Melodic Rider", Zone::Hand);
            g.state.players[p.index()].mana = colors;
            start(&mut g, bifur, None);
            until(&mut g, |g| g.on_battlefield(bifur));
            assert!(g.state.players[p.index()].enduring_story);
            assert_eq!(items(&g, bifur).len(), 2);
            assert!(
                items(&g, bifur)
                    .iter()
                    .all(|s| !s.is_copy && s.additional_triggers == 0)
            );
            drive(&mut g, &[bifur, enemy]);
            assert_eq!(g.counter_count(bifur, CounterKind::PlusOne), 1);
            assert_eq!(g.counter_count(enemy, CounterKind::PlusOne), 1);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            g.state.objects[bifur as usize].summoning_sick = false;
            g.state.step = Step::Attackers;
            g.state.pending = Pending::Attackers;
            act(&mut g, Action::Attack { card: bifur });
            act(&mut g, Action::FinishAttackers);
            assert_eq!(items(&g, bifur).len(), 2);
            drive(&mut g, &[enemy, enemy]);
            assert_eq!(g.counter_count(enemy, CounterKind::PlusOne), 3);
        }
    }
}

#[test]
fn hob_extra_triggers_current_qualifications_and_copied_provider_registration() {
    for p in [Player::P0, Player::P1] {
        let mut g = game(p);
        let bifur = put(&mut g, p, "Bifur, Melodic Rider", Zone::Battlefield);
        let own = put(&mut g, p, "Axgard Cavalry", Zone::Battlefield);
        let other = put(&mut g, p.other(), "Axgard Cavalry", Zone::Battlefield);
        let bear = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        assert_eq!(g.object_view(own).additional_triggers, 0, "no designation");
        put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
        put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
        g.refresh_enduring_stories();
        assert_eq!(g.object_view(own).additional_triggers, 1);
        assert_eq!(g.object_view(other).additional_triggers, 0);
        assert_eq!(g.object_view(bear).additional_triggers, 0);
        dwarf(&mut g, bear);
        assert_eq!(g.object_view(bear).additional_triggers, 1);
        g.record_control(g.object_ref(bear), p.other(), None);
        assert_eq!(g.object_view(bear).additional_triggers, 0);
        let recipe = CopyRecipe::new(Card::from_name("Bifur, Melodic Rider").unwrap());
        g.move_object(bifur, Zone::Exile);
        let copied = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        g.apply_temporary_copy(g.object_ref(copied), recipe, CopyDeadline::EndOfTurn);
        assert_eq!(
            g.object_view(copied).additional_triggers,
            1,
            "copied provider counts itself"
        );
        assert_eq!(g.object_view(own).additional_triggers, 1);
        g.apply_temporary_copy(
            g.object_ref(copied),
            CopyRecipe::new(Card::from_name("Grizzly Bears").unwrap()),
            CopyDeadline::EndOfTurn,
        );
        assert_eq!(
            g.object_view(own).additional_triggers,
            0,
            "obsolete copied provider removed"
        );
    }
}

#[test]
fn hob_extra_triggers_staff_paid_equip_prowess_and_equipment_controller() {
    for p in [Player::P0, Player::P1] {
        for cheap in [false, true] {
            let mut g = game(p);
            let creature = put(
                &mut g,
                p,
                if cheap {
                    "Ghitu Lavarunner"
                } else {
                    "Grizzly Bears"
                },
                Zone::Battlefield,
            );
            let staff = put(&mut g, p, "Wizard's Staff", Zone::Hand);
            start(&mut g, staff, None);
            drive(&mut g, &[]);
            let before = g.state.players[p.index()].mana;
            normal_equip(&mut g, staff, if cheap { 0 } else { 1 }, creature);
            assert_eq!(
                before.iter().sum::<u32>() - g.state.players[p.index()].mana.iter().sum::<u32>(),
                if cheap { 1 } else { 3 }
            );
            assert_eq!(g.object_view(creature).additional_triggers, 1);
            // Control of the Staff is immaterial to its equipped creature's triggers.
            g.record_control(g.object_ref(staff), p.other(), None);
            let spell = put(&mut g, p, "Goldvein Pick", Zone::Hand);
            start(&mut g, spell, None);
            assert_eq!(items(&g, creature).len(), 2);
            assert!(items(&g, creature).iter().all(|s| {
                s.controller == p
                    && s.attachment_trigger
                        .as_ref()
                        .is_some_and(|e| e.recipient.card == creature)
            }));
            drive(&mut g, &[]);
            let base = if cheap { (1, 2) } else { (2, 2) };
            assert_eq!(stats(&g, creature), (base.0 + 2, base.1 + 2));
            animate(&mut g, staff);
            assert_eq!(
                g.object_view(creature).additional_triggers,
                0,
                "creature Equipment cannot supply attached effects before detachment SBA"
            );
        }
    }
    let p = Player::P0;
    let mut g = game(p);
    let bear = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let staff = put(&mut g, p, "Wizard's Staff", Zone::Battlefield);
    let before = g.snapshot();
    reject(
        &mut g,
        Action::AnnounceActivate {
            card: staff,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    assert_eq!(g.snapshot(), before);
    let _ = bear;
}

#[test]
fn hob_extra_triggers_two_providers_are_additive_and_frozen_on_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = game(p);
        let bifur = story(&mut g, p);
        let staff = staff(&mut g, p, bifur);
        let spell = put(&mut g, p, "Goldvein Pick", Zone::Hand);
        start(&mut g, spell, None);
        assert_eq!(
            items(&g, bifur).len(),
            3,
            "one prowess ability plus Bifur and Staff"
        );
        assert!(items(&g, bifur).iter().all(|s| s.additional_triggers == 0));
        g.move_object(staff, Zone::Exile);
        drive(&mut g, &[]);
        assert_eq!(
            stats(&g, bifur),
            (7, 8),
            "queued granted ability survives grantor departure"
        );
    }
}

#[test]
fn hob_extra_triggers_suppression_and_newer_grant_availability() {
    let p = Player::P0;
    let mut g = game(p);
    let bifur = story(&mut g, p);
    let wasp = put(&mut g, p, "The Wondrous Wasp", Zone::Hand);
    start(&mut g, wasp, None);
    until(&mut g, |g| {
        g.legal_actions().contains(&Action::ChooseTarget {
            target: Target::Card(bifur),
        })
    });
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(bifur),
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(&mut g, Action::FinishTargets);
    }
    drive(&mut g, &[]);
    assert!(!g.printed_abilities_available(bifur));
    assert_eq!(g.object_view(bifur).additional_triggers, 0);
    let staff = staff(&mut g, p, bifur);
    assert_eq!(
        g.object_view(bifur).additional_triggers,
        1,
        "Staff remains independent of removed Bifur static ability"
    );
    let spell = put(&mut g, p, "Goldvein Pick", Zone::Hand);
    start(&mut g, spell, None);
    assert_eq!(
        items(&g, bifur).len(),
        2,
        "a newer granted prowess survives the older removal effect"
    );
    drive(&mut g, &[]);
    g.move_object(wasp, Zone::Exile);
    assert_eq!(g.object_view(bifur).additional_triggers, 2);
    g.move_object(staff, Zone::Exile);
    assert_eq!(g.object_view(bifur).additional_triggers, 1);
}

#[test]
fn hob_extra_triggers_simultaneous_lki_recruit_hidden_choices_and_owned_pending() {
    for p in [Player::P0, Player::P1] {
        let mut g = game(p);
        let bifur = story(&mut g, p);
        let lookout = put(&mut g, p, "Lake-town Lookout", Zone::Battlefield);
        dwarf(&mut g, lookout);
        let staff = staff(&mut g, p, lookout);
        let discard = put(&mut g, p, "Grizzly Bears", Zone::Hand);
        g.sacrifice_batch(&[(p, bifur), (p, staff), (p, lookout)]);
        assert_eq!(
            g.state.pending_triggers.len(),
            1,
            "metadata still attaches to the original packet"
        );
        assert_eq!(
            g.state.pending_triggers[0].additional_triggers, 2,
            "both departed providers were active in source LKI"
        );
        let saved = g.snapshot();
        let mut child = {
            let parent = g.clone();
            parent.clone()
        };
        child.prepare_triggers();
        assert_eq!(items(&child, lookout).len(), 3);
        g.restore(&saved).unwrap();
        g.prepare_triggers();
        assert_eq!(items(&g, lookout).len(), 3);
        assert_eq!(g.snapshot(), child.snapshot());
        for i in 0..3 {
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::EffectDiscard { .. })
            });
            let private = g
                .legal_actions()
                .iter()
                .find_map(|a| match a {
                    Action::Discard { card } => Some(*card),
                    _ => None,
                })
                .unwrap();
            assert!(g.observe(p).objects.iter().any(|o| o.id == private));
            assert!(g.observe(p.other()).objects.iter().all(|o| o.id != private));
            assert!(g.observe(p.other()).effect_discards.is_empty());
            if i == 0 {
                super::contracts::branch_contract(g.clone());
            }
            let card = if i == 0 {
                discard
            } else {
                g.legal_actions()
                    .into_iter()
                    .find_map(|a| {
                        if let Action::Discard { card } = a {
                            Some(card)
                        } else {
                            None
                        }
                    })
                    .unwrap()
            };
            act(&mut g, Action::Discard { card });
        }
        drive(&mut g, &[]);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 3);
        assert!(g.state.objects.iter().any(|o| o.zone == Zone::Battlefield
            && o.is_token()
            && o.card.info().type_line == "Creature — Human Soldier"));
    }
}

#[test]
fn hob_extra_triggers_ward_metadata_and_separate_payments() {
    for p in [Player::P0, Player::P1] {
        let mut g = game(p);
        let target = put(&mut g, p.other(), "Goldvein Pick", Zone::Battlefield);
        let creature = put(&mut g, p.other(), "Tolarian Terror", Zone::Battlefield);
        let staff = staff(&mut g, p.other(), creature);
        let spell = put(&mut g, p, "Unsummon", Zone::Hand);
        start(&mut g, spell, Some(creature));
        assert_eq!(items(&g, creature).len(), 2);
        let link = items(&g, creature)[0].related_stack;
        assert!(link.is_some());
        assert!(
            items(&g, creature)
                .iter()
                .all(|s| s.related_stack == link && !s.is_copy)
        );
        let _ = (target, staff);
        for _ in 0..2 {
            until(&mut g, |g| {
                g.observe(g.actor().unwrap()).resolution_payment.is_some()
            });
            choose(&mut g, |a| matches!(a, Action::PayResolutionMana { .. }));
        }
        drive(&mut g, &[]);
        assert_eq!(g.object_view(creature).zone, Zone::Hand);
    }
}

#[test]
fn hob_extra_triggers_explicit_limit_and_optional_resolution_limit() {
    let p = Player::P0;
    let mut g = game(p);
    let investigator = put(&mut g, p, "Hostile Investigator", Zone::Battlefield);
    staff(&mut g, p, investigator);
    let one = put(&mut g, p, "Grizzly Bears", Zone::Hand);
    let two = put(&mut g, p, "Grizzly Bears", Zone::Hand);
    g.discard_batch(&[one], None);
    g.prepare_triggers();
    assert_eq!(items(&g, investigator).len(), 1, "printed trigger-once cap");
    drive(&mut g, &[]);
    g.discard_batch(&[two], None);
    g.prepare_triggers();
    assert!(items(&g, investigator).is_empty());
    assert_eq!(
        g.state
            .objects
            .iter()
            .filter(|o| o.zone == Zone::Battlefield && o.is_token() && o.card.has_subtype("Clue"))
            .count(),
        1
    );
    for decline in [false, true] {
        let mut g = game(p);
        let cornu = put(&mut g, p, "Ancient Cornucopia", Zone::Battlefield);
        animate(&mut g, cornu);
        staff(&mut g, p, cornu);
        let bear = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let spell = put(&mut g, p, "Giant Growth", Zone::Hand);
        start(&mut g, spell, Some(bear));
        assert_eq!(
            items(&g, cornu)
                .iter()
                .filter(|s| s.attachment_trigger.is_none())
                .count(),
            2
        );
        let life = g.state.players[p.index()].life;
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectOptional { .. })
        });
        act(&mut g, Action::ChooseOptional { accept: !decline });
        if decline {
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::EffectOptional { .. })
            });
            act(&mut g, Action::ChooseOptional { accept: true });
        }
        drive(&mut g, &[]);
        assert_eq!(g.state.players[p.index()].life, life + 1);
    }
}

#[test]
fn hob_extra_triggers_state_latch_and_reflexive_scheduler_do_not_redouble() {
    let p = Player::P0;
    let mut g = game(p);
    let bifur = story(&mut g, p);
    let tome = put(&mut g, p, "Mazemind Tome", Zone::Battlefield);
    animate(&mut g, tome);
    staff(&mut g, p, tome);
    g.add_counter(tome, CounterKind::Page, 4);
    g.prepare_triggers();
    assert_eq!(items(&g, tome).len(), 1);
    drive(&mut g, &[]);
    assert_eq!(g.object_view(tome).zone, Zone::Exile);
    let source = g.object_view(bifur);
    assert!(g.queue_trigger(&source, 0));
    g.expand_trigger_multiplicity();
    let mut packet = g.state.pending_triggers.pop().unwrap();
    assert_eq!(packet.additional_triggers, 0);
    packet.kind = StackKind::TriggeredAbility { ability: 0 };
    // Reflexive/delayed schedulers clone already expanded owned packets and do
    // not dispatch a fresh permanent-owned trigger event.
    let before = g.state.pending_triggers.len();
    g.queue_reflexive(&packet, 0, None);
    g.expand_trigger_multiplicity();
    assert_eq!(g.state.pending_triggers.len(), before + 1);
    assert!(
        g.state
            .pending_triggers
            .iter()
            .all(|s| s.additional_triggers == 0)
    );
}

#[test]
fn hob_extra_triggers_linked_exiles_preserve_every_instance() {
    let p = Player::P0;
    let mut g = game(p);
    story(&mut g, p);
    let hunter = put(&mut g, p, "Glass Casket", Zone::Battlefield);
    animate(&mut g, hunter);
    dwarf(&mut g, hunter);
    let one = put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
    let two = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
    let source = g.object_view(hunter);
    assert!(g.queue_trigger(&source, 0));
    g.prepare_triggers();
    assert_eq!(items(&g, hunter).len(), 2);
    drive(&mut g, &[one, two]);
    assert_eq!(g.object_view(one).zone, Zone::Exile);
    assert_eq!(g.object_view(two).zone, Zone::Exile);
    g.move_object(hunter, Zone::Exile);
    drive(&mut g, &[]);
    assert_eq!(g.object_view(one).zone, Zone::Battlefield);
    assert_eq!(g.object_view(two).zone, Zone::Battlefield);
}

#[test]
fn hob_extra_triggers_json_snapshot_handles_preserve_private_owned_witnesses() {
    use crate::protocol::Session;
    let p = Player::P0;
    let mut g = game(p);
    let bifur = story(&mut g, p);
    let source = g.object_view(bifur);
    assert_eq!(source.additional_triggers, 1);
    assert!(g.queue_trigger(&source, 0));
    let observed = serde_json::to_string(&g.observe(p)).unwrap();
    assert!(!observed.contains("additional_triggers"));
    assert!(!observed.contains("trigger_multiplicity"));
    let mut session = Session::from_game(g);
    let saved: serde_json::Value =
        serde_json::from_str(&session.json(r#"{"op":"snapshot"}"#)).unwrap();
    let handle = saved["snapshot"].as_u64().unwrap();
    let mut fork = session.fork();
    fn publish(session: &mut Session) -> serde_json::Value {
        let seen: serde_json::Value =
            serde_json::from_str(&session.json(r#"{"op":"observe","viewer":"P0"}"#)).unwrap();
        let input = serde_json::json!({"op":"act","actor":"P0","viewer":"P0","revision":seen["revision"],"action":Action::Pass});
        let mut out: serde_json::Value =
            serde_json::from_str(&session.json(&input.to_string())).unwrap();
        assert!(out["error"].is_null(), "{out}");
        out["observation"]
            .as_object_mut()
            .unwrap()
            .remove("revision");
        out["observation"].clone()
    }
    let first = publish(&mut session);
    assert_eq!(first, publish(&mut fork));
    let restored: serde_json::Value = serde_json::from_str(
        &session.json(&serde_json::json!({"op":"restore","snapshot":handle}).to_string()),
    )
    .unwrap();
    assert!(restored["error"].is_null());
    assert_eq!(first, publish(&mut session));
    assert_eq!(first["waiting_triggers"].as_array().unwrap().len(), 2);
    eprintln!(
        "extra-trigger layouts ObjectView={} SpellView={} State={}",
        std::mem::size_of::<ObjectView>(),
        std::mem::size_of::<SpellView>(),
        std::mem::size_of::<State>()
    );
}

#[test]
fn hob_extra_triggers_delayed_packets_and_real_stack_ability_copy_are_not_redoubled() {
    let p = Player::P0;
    let mut g = game(p);
    let duplicator = put(&mut g, p, "Esoteric Duplicator", Zone::Battlefield);
    animate(&mut g, duplicator);
    staff(&mut g, p, duplicator);
    act(
        &mut g,
        Action::AnnounceActivate {
            card: duplicator,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    pay(&mut g);
    for _ in 0..2 {
        until(&mut g, |g| g.observe(p).resolution_payment.is_some());
        choose(&mut g, |a| matches!(a, Action::PayResolutionMana { .. }));
    }
    drive(&mut g, &[]);
    let delayed = g.state.delayed_artifact_copies.as_ref().unwrap();
    assert_eq!(delayed.len(), 2);
    assert!(delayed.iter().all(|s| s.additional_triggers == 0));
    g.begin_step(Step::End);
    g.prepare_triggers();
    drive(&mut g, &[]);
    assert_eq!(
        g.state
            .objects
            .iter()
            .filter(|o| o.zone == Zone::Battlefield
                && o.is_copy_token()
                && o.card == Card::from_name("Esoteric Duplicator").unwrap())
            .count(),
        2
    );

    let mut g = game(p);
    let bifur = story(&mut g, p);
    let echo = put(&mut g, p, "Echo, Perceptive Prodigy", Zone::Battlefield);
    g.state.objects[echo as usize].summoning_sick = false;
    let source = g.object_view(bifur);
    assert!(g.queue_trigger(&source, 0));
    g.prepare_triggers();
    for _ in 0..16 {
        if g.state.pending == Pending::Priority {
            break;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::PutTrigger { .. }
                        | Action::ChooseTarget {
                            target: Target::Card(_)
                        }
                        | Action::FinishTargets
                )
            })
            .unwrap();
        act(&mut g, action);
    }
    let original = g.state.stack.last().unwrap().clone();
    assert_eq!(items(&g, bifur).len(), 2);
    act(
        &mut g,
        Action::AnnounceActivate {
            card: echo,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Stack(original.id),
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(&mut g, Action::FinishTargets);
    }
    pay(&mut g);
    until(&mut g, |g| g.observe(p).retarget.is_some());
    act(&mut g, Action::FinishStackTargets);
    assert_eq!(items(&g, bifur).len(), 3);
    assert_eq!(items(&g, bifur).iter().filter(|s| s.is_copy).count(), 1);
    assert!(items(&g, bifur).iter().all(|s| s.additional_triggers == 0));
    drive(&mut g, &[]);
    let total: u64 = g
        .state
        .objects
        .iter()
        .enumerate()
        .map(|(id, _)| g.counter_count(id as CardId, CounterKind::PlusOne))
        .sum();
    assert_eq!(total, 3);
}

#[test]
fn hob_extra_triggers_before_exile_self_leave_witness_survives_provider_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = game(p);
        let gambit = put(&mut g, p, "Greed's Gambit", Zone::Battlefield);
        animate(&mut g, gambit);
        let staff = staff(&mut g, p, gambit);
        let sources = g.death_trigger_sources();
        let events = g.move_objects_raw(&[gambit, staff], Zone::Exile);
        g.finish_zone_events(&events, &sources);
        assert_eq!(g.state.pending_triggers.len(), 1);
        assert_eq!(g.state.pending_triggers[0].additional_triggers, 1);
        assert!(
            g.state.trigger_multiplicity.is_none(),
            "cold index released"
        );
        g.prepare_triggers();
        assert_eq!(items(&g, gambit).len(), 2);
        let life = g.state.players[p.index()].life;
        drive(&mut g, &[]);
        assert_eq!(g.state.players[p.index()].life, life - 12);
    }
}
