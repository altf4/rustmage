//! Dense layer-six parity against the exhaustive provider/recipient algorithm.
use super::*;
use crate::engine::ability_timeline_tests::{cast, checked, named, settle, settle_on, setup};
use std::time::Instant;
fn assert_same(g: &Game) {
    let before = g.snapshot();
    let fast = evaluate_abilities(g);
    let reference = evaluate_abilities_with::<Game, true>(g);
    assert_eq!(fast.result.objects, reference.result.objects);
    assert_eq!(fast.result.started, reference.result.started);
    assert_eq!(
        fast.groups
            .iter()
            .map(|v| (v.source, v.group, v.first_layer, v.effect))
            .collect::<Vec<_>>(),
        reference
            .groups
            .iter()
            .map(|v| (v.source, v.group, v.first_layer, v.effect))
            .collect::<Vec<_>>()
    );
    let reference_query =
        crate::engine::characteristic_query::CharacteristicQuery::with_evaluation(g, reference);
    for p in [Player::P0, Player::P1] {
        for view in g.observe(p).objects {
            assert_eq!(
                serde_json::to_value(&view).unwrap(),
                serde_json::to_value(reference_query.object_view(view.id)).unwrap()
            );
        }
    }
    assert_eq!(before, g.snapshot());
}
fn dense(p: Player) -> Game {
    let mut g = setup(p);
    let mut recipients = Vec::new();
    for i in 0..100 {
        let owner = if i % 3 == 0 { p.other() } else { p };
        let id = named(&mut g, owner, "Grizzly Bears", Zone::Battlefield);
        recipients.push(id);
    }
    for i in 0..24 {
        let name = [
            "Skateboard",
            "Quintessential Katana",
            "Hard-Won Jitte",
            "Shredder's Armor",
        ][i % 4];
        let id = named(&mut g, p, name, Zone::Battlefield);
        let source = g.object_ref(id);
        let recipient = g.object_ref(recipients[i]);
        assert!(g.attach_equipment(source, recipient));
    }
    for (i, &id) in recipients.iter().enumerate().take(36) {
        let source = g.object_ref(id);
        g.grant_keyword_effects(
            &[id],
            source,
            if i % 2 == 0 {
                Keywords::INDESTRUCTIBLE
            } else {
                Keywords::HASTE
            },
        );
    }
    g
}
#[test]
fn characteristic_performance_dense_equipment_temporary_grants_match_exhaustive_both_seats() {
    for p in [Player::P0, Player::P1] {
        let g = dense(p);
        assert_same(&g);
        let before = g.snapshot();
        let mut branch = {
            let parent = g.clone();
            parent.clone()
        };
        let elapsed = Instant::now();
        for _ in 0..8 {
            std::hint::black_box(evaluate_abilities_with::<Game, true>(&g));
        }
        let reference = elapsed.elapsed();
        let elapsed = Instant::now();
        for _ in 0..8 {
            std::hint::black_box(evaluate_abilities(&g));
        }
        let optimized = elapsed.elapsed();
        eprintln!(
            "dense characteristic benchmark seat={p:?} objects={} battlefield={} evaluations=8 exhaustive={reference:?} optimized={optimized:?}",
            g.state.objects.len(),
            g.state
                .objects
                .iter()
                .filter(|o| o.zone == Zone::Battlefield)
                .count()
        );
        branch.restore(&before).unwrap();
        assert_same(&branch);
        assert_eq!(g.snapshot(), branch.snapshot());
    }
}
#[test]
fn characteristic_performance_paid_equipment_control_removal_copy_and_stale_attachment_parity() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let sword = named(&mut g, p, "Skateboard", Zone::Hand);
        cast(&mut g, sword, None, &[]);
        settle_on(&mut g, Some(Target::Card(bear)));
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: sword,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
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
        settle(&mut g);
        g.grant_keyword_effects(&[bear], g.object_ref(sword), Keywords::INDESTRUCTIBLE);
        assert_same(&g);
        let aura = named(&mut g, p, "Witness Protection", Zone::Hand);
        cast(&mut g, aura, None, &[Target::Card(bear)]);
        settle(&mut g);
        assert_same(&g);
        let reflection = named(&mut g, p, "Self-Reflection", Zone::Hand);
        cast(&mut g, reflection, None, &[Target::Card(bear)]);
        settle(&mut g);
        assert_same(&g);
        let packet = g.activation_item(p, bear, 0);
        g.gain_control(&packet, Subject::Source, PlayerSet::Opponent);
        assert_same(&g);
        g.move_object(bear, Zone::Exile);
        g.move_object(bear, Zone::Battlefield);
        assert_same(&g);
    }
}
#[test]
fn characteristic_performance_bound_source_and_broadcast_fragments_keep_exact_order() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        g.add_counter(bear, CounterKind::PlusOne, 1);
        for name in [
            "Death Baron",
            "Dazzling Angel",
            "Mikey & Leo, Chaos & Order",
            "Splinter, Radical Rat",
            "Venus, Torn Between Worlds",
        ] {
            named(&mut g, p, name, Zone::Battlefield);
        }
        let source = g.object_ref(bear);
        g.grant_keyword_effects(&[bear], source, Keywords::TRAMPLE);
        assert_same(&g);
        // Every authored immutable fragment is checked against its exhaustive
        // candidate domain; unknown fragment kinds deliberately use fallback.
        for definition in Card::catalogue().filter(|c| c.is_supported()) {
            for group in definition.static_ability_groups() {
                let fast = group_grants(&g, source, group.fragments, 0);
                let mut expected = Vec::new();
                for (id, _) in g.state.objects.iter().enumerate() {
                    for fragment in group.fragments {
                        if let Some(row) = grant_fragment(&g, source, fragment, id as CardId, 0) {
                            expected.push(row);
                        }
                    }
                }
                assert_eq!(
                    format!("{fast:?}"),
                    format!("{expected:?}"),
                    "{}",
                    definition.name()
                );
            }
        }
    }
}
#[test]
fn characteristic_performance_grant_order_matches_dependency_reference_with_blank_providers_and_ties()
 {
    use crate::engine::ability_order::*;
    for seed in 0..64usize {
        let objects: Vec<_> = (0..8)
            .map(|i| LayerSixObject {
                printed: (i + seed) % 4 != 0,
                hexproof_colors: 0,
                keywords: Keywords::NONE,
                grants: Vec::new(),
                mana: Vec::new(),
            })
            .collect();
        let effects: Vec<_> = (0..24)
            .map(|i| {
                let recipient = ((i * 5 + seed) % 8) as CardId;
                let key = GrantedAbilityKey {
                    source: ObjectRef {
                        card: (i % 8) as CardId,
                        generation: 0,
                    },
                    granting_card: Card::GrizzlyBears,
                    grantor_controller: if i % 2 == 0 { Player::P0 } else { Player::P1 },
                    ability: i as u16,
                    timestamp: (i % 7) as u64,
                };
                let action = if seed % 8 == 0 && i == 7 {
                    LayerSixAction::Remove
                } else {
                    match i % 3 {
                        0 => LayerSixAction::Grant {
                            keywords: Keywords::HASTE,
                            abilities: vec![key],
                        },
                        1 => LayerSixAction::GrantHexproofColors(1 << (i % 5)),
                        _ => LayerSixAction::GrantEach(vec![LayerSixGrant {
                            recipient,
                            keywords: Keywords::TRAMPLE,
                            abilities: vec![key],
                            mana: Vec::new(),
                        }]),
                    }
                };
                LayerSixEffect {
                    provider: if i % 5 == 0 {
                        None
                    } else {
                        Some((i % 8) as CardId)
                    },
                    started_earlier: i % 7 == 0,
                    timestamp: ((23 - i + seed) % 9) as u64,
                    recipients: vec![recipient],
                    action,
                }
            })
            .collect();
        let before = format!("{effects:?}");
        let expected = apply_layer_six_exhaustive(objects.clone(), &effects);
        let actual = apply_layer_six(objects, &effects);
        assert_eq!(actual.objects, expected.objects);
        assert_eq!(actual.started, expected.started);
        assert_eq!(before, format!("{effects:?}"));
    }
}
