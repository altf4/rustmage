//! One-recipient keyword projections preserve the existing Some/None query paths.
use super::*;
use crate::engine::ability_timeline_tests::{cast, checked, named, next_turn, settle, setup};
use std::time::Instant;
fn same(g: &Game) {
    let before = g.snapshot();
    let query = g.characteristic_query();
    for (id, _) in g.state.objects.iter().enumerate() {
        let id = id as CardId;
        assert_eq!(
            g.keywords(id),
            query.keywords(id),
            "{} {:?}",
            id,
            g.state.objects[id as usize].zone
        );
    }
    assert_eq!(before, g.snapshot());
}
fn family(fragment: &StaticAbilityFragment) -> Option<usize> {
    Some(match fragment {
        StaticAbilityFragment::SourceCounterKeywords { .. } => 0,
        StaticAbilityFragment::SourceCounterSubtype { .. } => 1,
        StaticAbilityFragment::AttachmentKeywords(_) => 2,
        StaticAbilityFragment::AttachedTurnKeywords(_) => 3,
        StaticAbilityFragment::AttachmentMana(_) => 4,
        StaticAbilityFragment::ControlledCreatureTrigger { .. } => 5,
        StaticAbilityFragment::AttachmentTrigger(_) => 6,
        StaticAbilityFragment::AttachmentActivation(_) => 7,
        StaticAbilityFragment::ChosenCreatureKeywords(_) => 8,
        StaticAbilityFragment::DeathBaronLord => 9,
        StaticAbilityFragment::CounterKeywordOwnTurn(..) => 10,
        StaticAbilityFragment::CounterKeyword(..) => 11,
        StaticAbilityFragment::ControlledCreatureCounterRule { .. } => 12,
        StaticAbilityFragment::ControlledTappedKeywords { .. } => 13,
        StaticAbilityFragment::KeywordGrant(..) => 14,
        StaticAbilityFragment::StaticRuleGrant(..) => 15,
        StaticAbilityFragment::LiveKeywords(..) => 16,
        StaticAbilityFragment::AttachmentRule(..) => 17,
        _ => return None,
    })
}
#[test]
fn scalar_keywords_all_authored_fragment_families_both_controls_and_active_players() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let skeleton = named(&mut g, p.other(), "Skeleton Archer", Zone::Battlefield);
        g.state.objects[bear as usize].tapped = true;
        g.add_counter(bear, CounterKind::PlusOne, 1);
        g.grant_keyword_effects(&[bear], g.object_ref(bear), Keywords::INDESTRUCTIBLE);
        let mut seen = [false; 18];
        for definition in Card::catalogue().filter(|c| {
            c.is_supported()
                && !c.is_token()
                && !c.is_emblem()
                && !c.static_ability_groups().iter().any(|group| {
                    group
                        .fragments
                        .iter()
                        .any(|f| matches!(f, StaticAbilityFragment::RemoveAttachedAbilities))
                })
        }) {
            let wanted = definition
                .static_ability_groups()
                .iter()
                .flat_map(|g| g.fragments)
                .filter_map(family)
                .any(|f| !seen[f]);
            if !wanted {
                continue;
            }
            let id = crate::engine::tests::spawn(&mut g, p, definition, Zone::Battlefield);
            g.state.objects[id as usize].attached_to = Some(g.object_ref(bear));
            for group in definition.static_ability_groups() {
                for fragment in group.fragments {
                    if let Some(f) = family(fragment) {
                        seen[f] = true;
                    }
                    match *fragment {
                        StaticAbilityFragment::SourceCounterKeywords { kind, minimum, .. }
                        | StaticAbilityFragment::SourceCounterSubtype { kind, minimum, .. } => {
                            g.add_counter(id, kind, minimum.max(1))
                        }
                        StaticAbilityFragment::CounterKeyword(_, kind, _)
                        | StaticAbilityFragment::CounterKeywordOwnTurn(kind, _) => {
                            g.add_counter(bear, kind, 1)
                        }
                        _ => (),
                    }
                }
            }
            assert!(
                scalar_keywords_without_removal(&g, bear).is_some(),
                "fast family provider {}",
                definition.name()
            );
            same(&g);
            g.state.objects[id as usize].controller = p.other();
            assert!(scalar_keywords_without_removal(&g, bear).is_some());
            same(&g);
            g.state.objects[id as usize].attached_to = Some(g.object_ref(skeleton));
            assert!(scalar_keywords_without_removal(&g, bear).is_some());
            same(&g);
            g.move_object(id, Zone::Exile);
            same(&g);
        }
        assert!(
            seen[..17].iter().all(|v| *v),
            "missing authored families {seen:?}"
        );
        // AttachmentRule has no authored physical consumer yet. Exercise its
        // exact attachment/rules grant through the same primitive used above.
        let equipment = named(&mut g, p, "Skateboard", Zone::Battlefield);
        assert!(g.attach_equipment(g.object_ref(equipment), g.object_ref(bear)));
        assert!(scalar_keywords_without_removal(&g, bear).is_some());
        let fragment = StaticAbilityFragment::AttachmentRule(Keywords::CANT_BLOCK);
        let source = g.object_ref(equipment);
        assert_eq!(
            grant_fragment(&g, source, &fragment, bear, 0)
                .unwrap()
                .keywords,
            Keywords::CANT_BLOCK
        );
        assert!(grant_fragment(&g, source, &fragment, skeleton, 0).is_none());
        g.move_object(bear, Zone::Exile);
        g.move_object(bear, Zone::Battlefield);
        assert!(grant_fragment(&g, source, &fragment, bear, 0).is_none());
        seen[family(&fragment).unwrap()] = true;
        assert!(
            seen.iter().all(|v| *v),
            "missing all-family coverage {seen:?}"
        );
        g.state.active = p.other();
        same(&g);
    }
}
#[test]
fn scalar_keywords_none_path_and_temporary_rule_counter_suspect_animation_expiry() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let source = g.object_ref(bear);
        assert!(scalar_keywords_without_removal(&g, bear).is_none());
        same(&g);
        g.grant_keyword_effects(
            &[bear],
            source,
            Keywords::TRAMPLE.union(Keywords::CANT_BLOCK),
        );
        g.record_suspect_keywords(source);
        let timestamp = g.next_role_timestamp();
        let turn = g.state.turn;
        g.state
            .roles
            .as_mut()
            .unwrap()
            .keyword_effects
            .push(KeywordEffectView {
                recipient: source,
                origin: KeywordEffectOrigin::Resolution { source },
                keywords: Keywords::NONE,
                hexproof_colors: 1,
                rules: Keywords::NONE,
                timestamp,
                expires_turn: Some(turn),
                expires_before_turn: None,
            });
        assert_eq!(g.characteristic_query().hexproof_colors(bear), 1);
        assert!(!g.keywords(bear).contains(Keywords::HEXPROOF));
        g.state
            .roles
            .as_mut()
            .unwrap()
            .keyword_effects
            .push(KeywordEffectView {
                recipient: source,
                origin: KeywordEffectOrigin::Resolution { source },
                keywords: Keywords::DEATHTOUCH,
                hexproof_colors: 0,
                rules: Keywords::NONE,
                timestamp,
                expires_turn: Some(turn + 1),
                expires_before_turn: None,
            });
        assert!(!g.keywords(bear).contains(Keywords::DEATHTOUCH));
        assert!(scalar_keywords_without_removal(&g, bear).is_some());
        same(&g);
        g.effect_counter(
            bear,
            CounterKind::Flying,
            1,
            replacements::ResumeMode::Pending,
        );
        same(&g);
        g.set_counters(bear, Counters::default());
        same(&g);
        let item = g.activation_item(p, bear, 0);
        g.animate_source(
            &item,
            crate::cards::AnimationSpec {
                artifact: false,
                base: Some((3, 3)),
                creature_types: crate::cards::AnimationCreatureTypes::Unchanged,
                keywords: Keywords::LIFELINK,
                duration: crate::cards::AnimationDuration::EndOfTurn,
            },
        );
        same(&g);
        next_turn(&mut g);
        same(&g);
        let old = g.object_ref(bear);
        g.move_object(bear, Zone::Exile);
        g.move_object(bear, Zone::Battlefield);
        assert_ne!(old, g.object_ref(bear));
        same(&g);
    }
}
#[test]
fn scalar_keywords_paid_stack_copy_control_departure_and_ordered_removal_fallback() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        g.grant_keyword_effects(&[bear], g.object_ref(bear), Keywords::INDESTRUCTIBLE);
        let hill = named(&mut g, p, "Hill Giant", Zone::Hand);
        cast(&mut g, hill, None, &[]);
        g.grant_stack_keywords(g.object_ref(hill), Keywords::HASTE);
        same(&g);
        settle(&mut g);
        same(&g);
        let copy = named(&mut g, p, "Self-Reflection", Zone::Hand);
        cast(&mut g, copy, None, &[Target::Card(hill)]);
        same(&g);
        settle(&mut g);
        same(&g);
        let item = g.activation_item(p, hill, 0);
        g.gain_control(&item, Subject::Source, PlayerSet::Opponent);
        same(&g);
        let aura = named(&mut g, p, "Witness Protection", Zone::Hand);
        cast(&mut g, aura, None, &[Target::Card(bear)]);
        settle(&mut g);
        assert!(scalar_keywords_without_removal(&g, bear).is_none());
        same(&g);
        g.grant_keyword_effects(&[bear], g.object_ref(aura), Keywords::LIFELINK);
        same(&g);
        g.move_object(aura, Zone::Graveyard);
        same(&g);
        let before = g.snapshot();
        let mut branch = {
            let parent = g.clone();
            parent.clone()
        };
        checked(&mut g, Action::Pass);
        let actor = branch.actor().unwrap();
        branch.apply(actor, Action::Pass).unwrap();
        assert_eq!(g.snapshot(), branch.snapshot());
        g.restore(&before).unwrap();
        same(&g);
        assert_eq!(before, g.snapshot());
    }
}
#[test]
fn scalar_keywords_stack_inset_effective_definition_and_copied_exceptions() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        g.grant_keyword_effects(&[bear], g.object_ref(bear), Keywords::INDESTRUCTIBLE);
        // Actual inset spell definitions share their physical object's slot.
        let id = named(&mut g, p, "Hollow Scavenger // Bakery Raid", Zone::Hand);
        checked(&mut g, Action::AnnounceAdventure { card: id });
        for _ in 0..4 {
            if g.state.announcing.is_none() {
                break;
            }
            let a = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PayCosts { .. } | Action::FinishTargets))
                .unwrap();
            checked(&mut g, a);
        }
        assert!(g.effective_spell_card(id).adventure_parent().is_some());
        same(&g);
        let recipe = CopyRecipe {
            definition: Card::from_name("Grizzly Bears").unwrap(),
            faces: copies::CopyFaces::Definition,
            exceptions: CopyExceptions::HASTE_AND_SACRIFICE,
        };
        g.stage_copy_creation(p, recipe, 1);
        settle(&mut g);
        same(&g);
    }
}
#[test]
fn scalar_keywords_dense_single_recipient_benchmark_and_observation_snapshot_parity() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let mut ids = Vec::new();
        for _ in 0..100 {
            ids.push(named(&mut g, p, "Grizzly Bears", Zone::Battlefield));
        }
        for i in 0..24 {
            let e = named(&mut g, p, "Quintessential Katana", Zone::Battlefield);
            assert!(g.attach_equipment(g.object_ref(e), g.object_ref(ids[i])));
        }
        for &id in ids.iter().take(36) {
            g.grant_keyword_effects(&[id], g.object_ref(id), Keywords::INDESTRUCTIBLE);
        }
        let item = g.activation_item(p, ids[0], 0);
        g.animate_source(
            &item,
            crate::cards::AnimationSpec {
                artifact: false,
                base: Some((2, 2)),
                creature_types: crate::cards::AnimationCreatureTypes::Unchanged,
                keywords: Keywords::FLYING,
                duration: crate::cards::AnimationDuration::Permanent,
            },
        );
        same(&g);
        assert!(
            ids.iter()
                .all(|&id| scalar_keywords_without_removal(&g, id).is_some())
        );
        let snapshot = g.snapshot();
        let elapsed = Instant::now();
        for i in 0..256 {
            std::hint::black_box(g.characteristic_query().keywords(ids[i % ids.len()]));
        }
        let full = elapsed.elapsed();
        let elapsed = Instant::now();
        for i in 0..256 {
            std::hint::black_box(g.keywords(ids[i % ids.len()]));
        }
        let scalar = elapsed.elapsed();
        eprintln!(
            "scalar keywords seat={p:?} battlefield=124 queries=256 full_query={full:?} scalar={scalar:?}"
        );
        for viewer in [Player::P0, Player::P1] {
            let before = serde_json::to_value(g.observe(viewer)).unwrap();
            same(&g);
            assert_eq!(before, serde_json::to_value(g.observe(viewer)).unwrap());
        }
        assert_eq!(snapshot, g.snapshot());
    }
}
