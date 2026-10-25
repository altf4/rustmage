//! Paid Aura and declared combat scenarios; bounded layer fixtures are labeled.
use super::assurance::big_helpers::{cast, checked, named, next_turn, settle, setup};
use super::msh_galactus_tests::{finish, reject, until};
use super::*;
const SERUM: &str = "Super-Soldier Serum";
fn priority(g: &mut Game, p: Player) {
    if g.actor() != Some(p) {
        checked(g, Action::Pass);
    }
    assert_eq!(g.actor(), Some(p));
}
fn packet(g: &mut Game, p: Player, name: &str, ts: &[CardId]) -> CardId {
    priority(g, p);
    let id = named(g, p, name, Zone::Hand);
    let (mut mana, generic) = g.state.objects[id as usize].card.cost();
    mana[5] += generic;
    g.state.players[p.index()].mana = mana;
    cast(
        g,
        id,
        None,
        &ts.iter().copied().map(Target::Card).collect::<Vec<_>>(),
    );
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    id
}
pub(super) fn paid(g: &mut Game, p: Player, name: &str, ts: &[CardId]) -> CardId {
    let id = packet(g, p, name, ts);
    settle(g);
    id
}
fn trigger(g: &mut Game, s: CardId) {
    until(g, |g| {
        g.state.announcing.as_ref().is_some_and(|a| {
            a.item.card == s && matches!(a.item.kind, StackKind::TriggeredAbility { .. })
        }) || g
            .state
            .stack
            .iter()
            .any(|item| item.card == s && matches!(item.kind, StackKind::TriggeredAbility { .. }))
    });
}
fn choose(g: &mut Game, ts: &[CardId]) {
    for &t in ts {
        checked(
            g,
            Action::ChooseTarget {
                target: Target::Card(t),
            },
        );
    }
    if g.state.announcing.is_some() {
        finish(g);
    }
}
fn attack(g: &mut Game, p: Player, t: CardId) {
    g.state.active = p;
    g.state.priority = p;
    g.state.objects[t as usize].summoning_sick = false;
    g.begin_step(Step::BeginCombat);
    settle(g);
    until(g, |g| g.state.pending == Pending::Attackers);
    checked(g, Action::Attack { card: t });
    checked(g, Action::FinishAttackers);
}
fn block(g: &mut Game, p: Player, t: CardId, attacker: CardId) {
    attack(g, p.other(), attacker);
    until(g, |g| g.state.pending == Pending::Blockers);
    checked(
        g,
        Action::Block {
            blocker: t,
            attacker,
        },
    );
    checked(g, Action::FinishBlockers);
}
#[test]
fn msh_serum_paid_aura_any_controller_full_static_obligations() {
    for p in [Player::P0, Player::P1] {
        for owner in [p, p.other()] {
            let mut g = setup(p);
            let bear = named(&mut g, owner, "Bear Cub", Zone::Battlefield);
            let s = paid(&mut g, p, SERUM, &[bear]);
            let v = g.object_view(bear);
            assert_eq!((v.power, v.toughness), (4, 4));
            assert!(v.has_subtype("Bear") && v.has_subtype("Soldier"));
            assert!(v.is_legendary());
            assert_eq!(v.effective_legendary, Some(true));
            assert!(
                v.keywords
                    .contains(Keywords::FIRST_STRIKE.union(Keywords::VIGILANCE))
            );
            assert_eq!(v.controller, owner);
            assert_eq!(g.object_view(s).attached_to, Some(v.reference));
            assert!(!g.copiable_recipe(v.reference).definition.is_legendary());
            paid(&mut g, p, "Disenchant", &[s]);
            let v = g.object_view(bear);
            assert_eq!((v.power, v.toughness), (2, 2));
            assert!(!v.has_subtype("Soldier") || v.card.has_subtype("Soldier"));
            assert!(!v.is_legendary());
            assert_eq!(v.effective_legendary, None);
        }
    }
}
#[test]
fn msh_serum_paid_spell_targets_timing_and_short_payment_atomic() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let land = named(&mut g, p, "Island", Zone::Battlefield);
        let s = named(&mut g, p, SERUM, Zone::Hand);
        g.state.objects[land as usize].tapped = true;
        g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 0];
        reject(&mut g, Action::AnnounceCast { card: s });
        g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 1];
        g.state.active = p.other();
        reject(&mut g, Action::AnnounceCast { card: s });
        g.state.active = p;
        checked(&mut g, Action::AnnounceCast { card: s });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(land),
            },
        );
        reject(&mut g, Action::FinishTargets);
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        reject(&mut g, Action::PayCosts { payment: [0; 6] });
        finish(&mut g);
        settle(&mut g);
        assert_eq!(g.stats(bear).power, 4);
    }
}
#[test]
fn msh_serum_paid_attack_zero_one_many_and_equipment_control() {
    for p in [Player::P0, Player::P1] {
        for n in 0..=3 {
            let mut g = setup(p);
            let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let other = named(&mut g, p, "Hill Giant", Zone::Battlefield);
            let s = paid(&mut g, p, SERUM, &[bear]);
            let tools: Vec<_> = (0..3)
                .map(|_| named(&mut g, p, "Lotus Ring", Zone::Battlefield))
                .collect();
            let foreign = named(&mut g, p.other(), "Lotus Ring", Zone::Battlefield);
            let non = named(&mut g, p, "Arc Reactor", Zone::Battlefield);
            let first = g.object_ref(tools[0]);
            let recipient = g.object_ref(other);
            assert!(g.attach_equipment(first, recipient));
            attack(&mut g, p, bear);
            trigger(&mut g, s);
            assert!(!g.object_view(bear).tapped);
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(foreign),
                },
            );
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(non),
                },
            );
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(bear),
                },
            );
            choose(&mut g, &tools[..n]);
            settle(&mut g);
            for (i, &tool) in tools.iter().enumerate() {
                assert_eq!(
                    g.object_view(tool).attached_to,
                    if i < n {
                        Some(g.object_ref(bear))
                    } else if i == 0 {
                        Some(g.object_ref(other))
                    } else {
                        None
                    }
                );
            }
            assert!(g.object_view(foreign).attached_to.is_none());
        }
    }
}
#[test]
fn msh_serum_paid_block_declaration_uses_aura_controller_equipment() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let attacker = named(&mut g, p, "Hill Giant", Zone::Battlefield);
        let s = paid(&mut g, p, SERUM, &[bear]);
        let ring = named(&mut g, p, "Lotus Ring", Zone::Battlefield);
        let foreign = named(&mut g, p.other(), "Lotus Ring", Zone::Battlefield);
        block(&mut g, p.other(), bear, attacker);
        trigger(&mut g, s);
        let event = g
            .state
            .announcing
            .as_ref()
            .unwrap()
            .item
            .attachment_trigger
            .as_ref()
            .unwrap();
        assert_eq!(event.recipient, g.object_ref(bear));
        assert_eq!(event.defender, None);
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(foreign),
            },
        );
        choose(&mut g, &[ring]);
        settle(&mut g);
        assert_eq!(g.object_view(ring).attached_to, Some(g.object_ref(bear)));
    }
}
#[test]
fn msh_serum_paid_queued_recipient_survives_aura_departure_or_reattachment() {
    for p in [Player::P0, Player::P1] {
        for reattach in [false, true] {
            let mut g = setup(p);
            let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let other = named(&mut g, p, "Hill Giant", Zone::Battlefield);
            let s = paid(&mut g, p, SERUM, &[bear]);
            let ring = named(&mut g, p, "Lotus Ring", Zone::Battlefield);
            let second = named(&mut g, p, "Lotus Ring", Zone::Battlefield);
            attack(&mut g, p, bear);
            trigger(&mut g, s);
            choose(&mut g, &[ring, second]);
            if reattach {
                // Bounded attachment change after the real trigger froze its event recipient.
                g.state.objects[s as usize].attached_to = Some(g.object_ref(other));
                g.retimestamp_live_static_source(g.object_ref(s));
            } else {
                paid(&mut g, p, "Disenchant", &[s]);
            }
            settle(&mut g);
            assert_eq!(g.object_view(ring).attached_to, Some(g.object_ref(bear)));
            assert_eq!(g.object_view(second).attached_to, Some(g.object_ref(bear)));
        }
    }
}
#[test]
fn msh_serum_paid_stale_recipient_incarnation_never_receives_equipment() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let s = paid(&mut g, p, SERUM, &[bear]);
        let ring = named(&mut g, p, "Lotus Ring", Zone::Battlefield);
        let other = named(&mut g, p, "Hill Giant", Zone::Battlefield);
        assert!(g.attach_equipment(g.object_ref(ring), g.object_ref(other)));
        attack(&mut g, p, bear);
        trigger(&mut g, s);
        choose(&mut g, &[ring]);
        priority(&mut g, p);
        let u = named(&mut g, p, "Unsummon", Zone::Hand);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 0];
        cast(&mut g, u, None, &[Target::Card(bear)]);
        until(&mut g, |g| g.object_view(bear).zone == Zone::Hand);
        g.move_object(bear, Zone::Battlefield); // Explicit new incarnation before the old packet resolves.
        settle(&mut g);
        assert_eq!(g.object_view(ring).attached_to, Some(g.object_ref(other)));
    }
}
#[test]
fn msh_serum_paid_partial_equipment_targets_and_changed_control() {
    for p in [Player::P0, Player::P1] {
        for stolen in [false, true] {
            let mut g = setup(p);
            let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let s = paid(&mut g, p, SERUM, &[bear]);
            let a = named(&mut g, p, "Basilisk Collar", Zone::Battlefield);
            let b = named(&mut g, p, "Lotus Ring", Zone::Battlefield);
            attack(&mut g, p, bear);
            trigger(&mut g, s);
            choose(&mut g, &[a, b]);
            if stolen {
                g.record_control(g.object_ref(a), p.other(), None);
            }
            // Bounded equipment control transition.
            else {
                paid(&mut g, p, "Disenchant", &[a]);
            }
            settle(&mut g);
            assert_eq!(g.object_view(a).attached_to, None);
            assert_eq!(g.object_view(b).attached_to, Some(g.object_ref(bear)));
        }
    }
}
#[test]
fn msh_serum_recipient_hexproof_is_not_a_target_but_protection_prevents_attach() {
    for p in [Player::P0, Player::P1] {
        for protection in [false, true] {
            let mut g = setup(p);
            let bear = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let s = paid(&mut g, p, SERUM, &[bear]);
            let ring = named(&mut g, p, "Lotus Ring", Zone::Battlefield);
            attack(&mut g, p.other(), bear);
            trigger(&mut g, s);
            choose(&mut g, &[ring]);
            // A later resolved keyword grant changes attachment legality, never target legality.
            let reference = g.object_ref(bear);
            g.grant_keyword_effects(
                &[bear],
                reference,
                if protection {
                    Keywords::PROTECTION_EVERYTHING
                } else {
                    Keywords::HEXPROOF
                },
            );
            settle(&mut g);
            assert_eq!(
                g.object_view(ring).attached_to,
                if protection { None } else { Some(reference) }
            );
        }
    }
}
#[test]
fn msh_serum_real_legend_rule_and_copiable_values_exclude_aura_overlay() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let a = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let b = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        paid(&mut g, p, SERUM, &[a]);
        packet(&mut g, p, SERUM, &[b]);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::Legend { .. })
        });
        let object = g.object_ref(b);
        checked(&mut g, Action::KeepLegend { object });
        settle(&mut g);
        assert_eq!(g.object_view(a).zone, Zone::Graveyard);
        assert!(g.object_view(b).is_legendary());
        paid(&mut g, p, "Self-Reflection", &[b]);
        let token = g
            .state
            .objects
            .iter()
            .enumerate()
            .find_map(|(i, o)| {
                (o.zone == Zone::Battlefield && o.is_copy_token()).then_some(i as CardId)
            })
            .unwrap();
        let v = g.object_view(token);
        assert_eq!((v.power, v.toughness), (2, 2));
        assert!(!v.is_legendary());
        assert!(!v.has_subtype("Soldier"));
    }
}
#[test]
fn msh_serum_paid_first_strike_combat_and_vigilance() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let blocker = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let s = paid(&mut g, p, SERUM, &[bear]);
        attack(&mut g, p, bear);
        trigger(&mut g, s);
        choose(&mut g, &[]);
        settle(&mut g);
        assert!(!g.object_view(bear).tapped);
        until(&mut g, |g| g.state.pending == Pending::Blockers);
        checked(
            &mut g,
            Action::Block {
                blocker,
                attacker: bear,
            },
        );
        checked(&mut g, Action::FinishBlockers);
        next_turn(&mut g);
        assert_eq!(g.object_view(blocker).zone, Zone::Graveyard);
        assert_eq!(g.object_view(bear).zone, Zone::Battlefield);
    }
}
#[test]
fn msh_serum_paid_later_recipient_ability_removal_keeps_earlier_layers() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        paid(&mut g, p, SERUM, &[bear]);
        let wasp = packet(&mut g, p, "The Wondrous Wasp", &[]);
        trigger(&mut g, wasp);
        choose(&mut g, &[bear]);
        settle(&mut g);
        let v = g.object_view(bear);
        assert_eq!((v.power, v.toughness), (4, 4));
        assert!(v.is_legendary() && v.has_subtype("Soldier"));
        assert!(
            !v.keywords
                .intersects(Keywords::FIRST_STRIKE.union(Keywords::VIGILANCE))
        );
    }
}
#[test]
fn msh_serum_public_legendary_view_privacy_orphan_and_restore() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let s = paid(&mut g, p, SERUM, &[bear]);
        let ring = named(&mut g, p, "Lotus Ring", Zone::Battlefield);
        for viewer in [p, p.other()] {
            let mut a = g.clone();
            let mut b = g.clone();
            named(&mut a, viewer.other(), "Forest", Zone::Hand);
            named(&mut b, viewer.other(), "Mountain", Zone::Hand);
            assert_eq!(
                serde_json::to_value(a.observe(viewer)).unwrap(),
                serde_json::to_value(b.observe(viewer)).unwrap()
            );
        }
        attack(&mut g, p, bear);
        trigger(&mut g, s);
        choose(&mut g, &[ring]);
        let actor = g.actor().unwrap();
        let action = Action::Pass;
        let index = g.legal_actions().iter().position(|a| *a == action).unwrap();
        g = super::msh_thirteenth_combined_tests::owned_checked_step(g, actor, index, action);
        settle(&mut g);
        assert_eq!(g.object_view(ring).attached_to, Some(g.object_ref(bear)));
    }
}

#[test]
fn msh_serum_earlier_static_group_continues_after_its_source_loses_abilities() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let serum = paid(&mut g, p, SERUM, &[bear]);
        let wasp = named(&mut g, p, "The Wondrous Wasp", Zone::Battlefield);
        // Bounded external all-ability-removal effect on a noncreature source tests
        // CR613.6 without claiming Wasp can legally target an ordinary Aura.
        let timestamp = g.next_role_timestamp();
        let source = g.object_ref(wasp);
        let recipient = g.object_ref(serum);
        g.state
            .roles
            .as_mut()
            .unwrap()
            .source_linked_effects
            .push(SourceLinkedEffectView {
                source,
                recipient,
                controller: p,
                kind: SourceLinkedEffectKind::RemoveAbilities,
                timestamp,
            });
        let v = g.object_view(bear);
        assert!(!g.printed_abilities_available(serum));
        assert_eq!((v.power, v.toughness), (4, 4));
        assert!(v.is_legendary() && v.has_subtype("Soldier"));
        assert!(
            v.keywords
                .contains(Keywords::FIRST_STRIKE.union(Keywords::VIGILANCE))
        );
    }
}

#[test]
fn msh_serum_same_attachment_keeps_timestamp_while_moved_equipment_is_later() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let other = named(&mut g, p, "Hill Giant", Zone::Battlefield);
        let s = paid(&mut g, p, SERUM, &[bear]);
        let old = named(&mut g, p, "Lotus Ring", Zone::Battlefield);
        let newer = named(&mut g, p, "Lotus Ring", Zone::Battlefield);
        assert!(g.attach_equipment(g.object_ref(old), g.object_ref(bear)));
        assert!(g.attach_equipment(g.object_ref(newer), g.object_ref(other)));
        let wasp = packet(&mut g, p, "The Wondrous Wasp", &[]);
        trigger(&mut g, wasp);
        choose(&mut g, &[bear]);
        settle(&mut g);
        assert!(!g.keywords(bear).contains(Keywords::VIGILANCE));
        g.untap_objects(&[bear], untap_rules::UntapCause::Effect);
        attack(&mut g, p, bear);
        trigger(&mut g, s);
        choose(&mut g, &[old, newer]);
        settle(&mut g);
        assert!(g.keywords(bear).contains(Keywords::VIGILANCE));
        // Bounded departure distinguishes the unchanged old attachment's timestamp.
        g.move_object(newer, Zone::Graveyard);
        assert!(!g.keywords(bear).contains(Keywords::VIGILANCE));
    }
}

#[test]
fn msh_serum_resolution_timestamp_order_is_mandatory_selected_and_batch_atomic() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let other = named(&mut g, p, "Hill Giant", Zone::Battlefield);
        let s = paid(&mut g, p, SERUM, &[bear]);
        let tools: Vec<_> = (0..3)
            .map(|_| named(&mut g, p, "Lotus Ring", Zone::Battlefield))
            .collect();
        let unselected = named(&mut g, p, "Lotus Ring", Zone::Battlefield);
        let unselected_ref = g.object_ref(unselected);
        let references: Vec<_> = tools.iter().map(|&id| g.object_ref(id)).collect();
        for &id in &tools {
            assert!(g.attach_equipment(g.object_ref(id), g.object_ref(other)));
        }
        attack(&mut g, p, bear);
        trigger(&mut g, s);
        choose(&mut g, &tools);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EquipmentAttachment { .. })
        });
        let view = g.equipment_attachment_view().unwrap();
        assert_eq!(view.ordering, Some(true));
        assert_eq!(view.candidates.len(), 3);
        reject(
            &mut g,
            Action::ChooseEquipmentAttachment { equipment: None },
        );
        reject(
            &mut g,
            Action::ChooseEquipmentAttachment {
                equipment: Some(unselected_ref),
            },
        );
        let action = Action::ChooseEquipmentAttachment {
            equipment: Some(references[2]),
        };
        let actor = g.actor().unwrap();
        let index = g.legal_actions().iter().position(|a| *a == action).unwrap();
        g = super::msh_thirteenth_combined_tests::owned_checked_step(g, actor, index, action);
        assert_eq!(g.equipment_attachment_view().unwrap().candidates.len(), 2);
        for &id in &tools {
            assert_eq!(g.object_view(id).attached_to, Some(g.object_ref(other)));
        }
        reject(
            &mut g,
            Action::ChooseEquipmentAttachment {
                equipment: Some(references[2]),
            },
        );
        checked(
            &mut g,
            Action::ChooseEquipmentAttachment {
                equipment: Some(references[1]),
            },
        );
        settle(&mut g);
        for &id in &tools {
            assert_eq!(g.object_view(id).attached_to, Some(g.object_ref(bear)));
        }
        let times: Vec<_> = tools
            .iter()
            .map(|id| {
                g.state
                    .roles
                    .as_ref()
                    .unwrap()
                    .ability_timestamps
                    .iter()
                    .find(|row| row.source == g.object_ref(*id))
                    .unwrap()
                    .timestamp
            })
            .collect();
        assert!(times[2] < times[1] && times[1] < times[0]);
        assert!(g.equipment_attachment_view().is_none());
    }
}
