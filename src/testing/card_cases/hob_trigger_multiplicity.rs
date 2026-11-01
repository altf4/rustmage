//! Focused printed outcomes; shared engine invariants live in core tests.
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
        .filter(|s| {
            s.source.is_some_and(|source| source.id == id)
                && matches!(s.kind, StackKind::TriggeredAbility { .. })
        })
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
