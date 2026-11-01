//! Paid card outcomes for conditional Ward and optional nontargeting attachment.
use super::*;
fn finish(g: &mut Game) {
    for _ in 0..64 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty()
            && g.state.ordering_triggers.is_empty()
        {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("Vow/Thorin resolution budget {:?}", g.state.pending);
}
fn until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..48 {
        if ready(g) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("Vow/Thorin choice budget {:?}", g.state.pending);
}
fn paid_thorin(g: &mut Game, p: Player) -> CardId {
    let id = put(g, p, "Thorin Oakenshield", Zone::Hand);
    g.state.players[p.index()].mana = [1, 0, 0, 1, 0, 0];
    cast(g, id, None);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    finish(g);
    id
}
fn story(g: &mut Game, p: Player) -> (CardId, CardId, CardId) {
    let map = put(g, p, "Thrór's Map", Zone::Battlefield);
    let pick = put(g, p, "Goldvein Pick", Zone::Battlefield);
    let thorin = paid_thorin(g, p);
    assert!(g.state.players[p.index()].enduring_story);
    (thorin, map, pick)
}
fn vow(g: &mut Game, p: Player, target: CardId) -> CardId {
    let spell = put(g, p, "Vow to Erebor", Zone::Hand);
    g.state.priority = p;
    g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 1];
    cast(g, spell, Some(Target::Card(target)));
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    spell
}
fn attachment(g: &mut Game) {
    until(g, |g| {
        matches!(g.state.pending, Pending::EquipmentAttachment { .. })
    });
}
fn ward(g: &mut Game) {
    until(g, |g| {
        matches!(g.state.pending, Pending::ResolutionMana { .. })
    });
}
fn ward_count(g: &Game) -> usize {
    g.state
        .pending_triggers
        .iter()
        .chain(g.state.ordering_triggers.iter())
        .chain(g.state.stack.iter())
        .filter(|s| {
            matches!(s.kind, StackKind::TriggeredAbility { .. }) && s.related_stack.is_some()
        })
        .count()
}
fn shock(g: &mut Game, p: Player, target: CardId, reserve: u32) -> CardId {
    g.state.priority = p;
    g.state.players[p.index()].mana = [0, 0, 0, 1, 0, reserve];
    let spell = put(g, p, "Shock", Zone::Hand);
    cast(g, spell, Some(Target::Card(target)));
    spell
}
fn protection(g: &mut Game, p: Player, target: CardId) -> CardId {
    g.state.priority = p;
    g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 0];
    let spell = put(g, p, "Witness Protection", Zone::Hand);
    cast(g, spell, Some(Target::Card(target)));
    finish(g);
    spell
}
#[test]
fn vow_paid_instant_untaps_and_pumps_without_dwarf_and_expires() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
        g.state.objects[bear as usize].tapped = true;
        g.state.active = p.other();
        let spell = vow(&mut g, p, bear);
        finish(&mut g);
        assert!(!g.object_view(bear).tapped);
        assert_eq!(stats(&g, bear), (4, 4));
        assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
        assert!(!g.state.players[p.index()].equip_activated_this_turn);
        g.finish_cleanup();
        assert_eq!(stats(&g, bear), (2, 2));
    }
}
#[test]
fn vow_dwarf_may_attach_one_own_equipment_or_decline_without_equip_cost_or_targeting() {
    for p in [Player::P0, Player::P1] {
        for attach in [false, true] {
            let mut g = scenario(p);
            let dwarf = put(&mut g, p, "Dwarven Mauler", Zone::Battlefield);
            let old = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let pick = put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
            assert!(g.attach_equipment(g.object_ref(pick), g.object_ref(old)));
            let enemy = put(&mut g, p.other(), "Goldvein Pick", Zone::Battlefield);
            let map = put(&mut g, p, "Thrór's Map", Zone::Battlefield);
            g.state.objects[dwarf as usize].tapped = true;
            vow(&mut g, p, dwarf);
            attachment(&mut g);
            assert_eq!(stats(&g, dwarf), (4, 3));
            assert!(!g.object_view(dwarf).tapped);
            let choice = Action::ChooseEquipmentAttachment {
                equipment: Some(g.object_ref(pick)),
            };
            for viewer in [p, p.other()] {
                let view = g.observe(viewer).equipment_attachment.unwrap();
                assert_eq!(view.player, p);
                assert_eq!(view.recipient.reference, g.object_ref(dwarf));
                assert_eq!(view.candidates, vec![g.object_ref(pick)]);
            }
            let r = g.object_ref(enemy);
            reject(
                &mut g,
                Action::ChooseEquipmentAttachment { equipment: Some(r) },
            );
            let r = g.object_ref(map);
            reject(
                &mut g,
                Action::ChooseEquipmentAttachment { equipment: Some(r) },
            );
            contracts::branch_contract(g.clone());
            act(
                &mut g,
                if attach {
                    choice
                } else {
                    Action::ChooseEquipmentAttachment { equipment: None }
                },
            );
            finish(&mut g);
            assert_eq!(
                g.object_view(pick).attached_to,
                Some(g.object_ref(if attach { dwarf } else { old }))
            );
            assert_eq!(stats(&g, dwarf), if attach { (5, 4) } else { (4, 3) });
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert!(!g.state.players[p.index()].equip_activated_this_turn);
        }
    }
}
#[test]
fn vow_requires_own_current_creature_and_stale_or_transferred_target_fizzles() {
    for p in [Player::P0, Player::P1] {
        for stale in [false, true] {
            let mut g = scenario(p);
            let dwarf = put(&mut g, p, "Dwarven Mauler", Zone::Battlefield);
            let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let pick = put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
            let spell = put(&mut g, p, "Vow to Erebor", Zone::Hand);
            g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 1];
            act(&mut g, Action::AnnounceCast { card: spell });
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(enemy),
                },
            );
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(pick),
                },
            );
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(dwarf),
                },
            );
            if g.legal_actions().contains(&Action::FinishTargets) {
                act(&mut g, Action::FinishTargets);
            }
            pay(&mut g);
            g.state.objects[dwarf as usize].tapped = true;
            if stale {
                g.move_object(dwarf, Zone::Hand);
                g.move_object(dwarf, Zone::Battlefield);
            } else {
                g.state.objects[dwarf as usize].controller = p.other();
            }
            finish(&mut g);
            assert_eq!(stats(&g, dwarf), (2, 1));
            assert_eq!(g.object_view(pick).attached_to, None);
            if !stale {
                assert!(g.object_view(dwarf).tapped);
            }
        }
    }
}
#[test]
fn vow_checks_dwarf_on_resolution_and_cannot_attach_equipment_to_itself() {
    for p in [Player::P0, Player::P1] {
        for gain in [false, true] {
            let mut g = scenario(p);
            let target = put(
                &mut g,
                p,
                if gain { "Bear Cub" } else { "Dwarven Mauler" },
                Zone::Battlefield,
            );
            let pick = put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
            vow(&mut g, p, target);
            let item = g.state.stack.last().unwrap().clone();
            g.set_creature_types(
                &item,
                Subject::Targets,
                if gain { &["Dwarf"] } else { &["Goblin"] },
            );
            if gain {
                attachment(&mut g);
                let r = g.object_ref(pick);
                act(
                    &mut g,
                    Action::ChooseEquipmentAttachment { equipment: Some(r) },
                );
            }
            finish(&mut g);
            assert_eq!(
                g.object_view(pick).attached_to,
                if gain {
                    Some(g.object_ref(target))
                } else {
                    None
                }
            );
        }
        let mut g = scenario(p);
        let e = put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
        g.change_object_form(
            g.object_ref(e),
            crate::cards::AnimationSpec {
                artifact: true,
                creature_types: crate::cards::AnimationCreatureTypes::Named(&["Dwarf"]),
                base: Some((2, 2)),
                keywords: Keywords::NONE,
                duration: crate::cards::AnimationDuration::EndOfTurn,
            },
            characteristic_layers::AnimationTypeMode::AddCreature,
        );
        vow(&mut g, p, e);
        finish(&mut g);
        assert_eq!(stats(&g, e), (4, 4));
        assert_eq!(g.object_view(e).attached_to, None);
    }
}
#[test]
fn vow_attachment_rechecks_equipment_control_and_incarnation_and_exact_recipient() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..3 {
            let mut g = scenario(p);
            let target = put(&mut g, p, "Dwarven Mauler", Zone::Battlefield);
            let e = put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
            vow(&mut g, p, target);
            attachment(&mut g);
            let old = g.object_ref(e);
            if mode == 0 {
                g.state.objects[e as usize].controller = p.other();
            } else if mode == 1 {
                g.move_object(e, Zone::Hand);
                g.move_object(e, Zone::Battlefield);
            } else {
                g.move_object(target, Zone::Hand);
                g.move_object(target, Zone::Battlefield);
            }
            reject(
                &mut g,
                Action::ChooseEquipmentAttachment {
                    equipment: Some(old),
                },
            );
            act(
                &mut g,
                Action::ChooseEquipmentAttachment { equipment: None },
            );
            finish(&mut g);
            assert_eq!(g.object_view(e).attached_to, None);
        }
    }
}
#[test]
fn thorin_paid_body_union_story_threshold_permanence_and_own_target_exemption() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let map = put(&mut g, p, "Thrór's Map", Zone::Battlefield);
        let t = paid_thorin(&mut g, p);
        assert_eq!(stats(&g, t), (3, 2));
        assert!(g.keywords(t).contains(Keywords::TRAMPLE));
        assert!(
            !g.state.players[p.index()].enduring_story,
            "legendary artifact counts once, not twice"
        );
        let pick = put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
        g.refresh_enduring_stories();
        assert!(g.state.players[p.index()].enduring_story);
        g.move_object(map, Zone::Graveyard);
        g.move_object(pick, Zone::Graveyard);
        assert!(g.state.players[p.index()].enduring_story);
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.add_counter(bear, CounterKind::PlusOne, 1);
        shock(&mut g, p, bear, 0);
        assert_eq!(ward_count(&g), 0);
        finish(&mut g);
        assert_eq!(g.state.objects[bear as usize].damage, 2);
    }
}
#[test]
fn thorin_ward_one_actual_payment_decline_underpay_and_source_departure() {
    for p in [Player::P0, Player::P1] {
        for pay_tax in [false, true] {
            let mut g = scenario(p);
            let (t, _, _) = story(&mut g, p);
            let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            g.add_counter(bear, CounterKind::PlusOne, 1);
            let spell = shock(&mut g, p.other(), bear, 1);
            assert_eq!(ward_count(&g), 1);
            if !pay_tax {
                g.move_object(t, Zone::Graveyard);
            }
            ward(&mut g);
            assert_eq!(g.actor(), Some(p.other()));
            assert_eq!(g.observe(p).resolution_payment.unwrap().cost.generic, 1);
            reject(&mut g, Action::PayResolutionMana { payment: [0; 6] });
            reject(
                &mut g,
                Action::PayResolutionMana {
                    payment: [0, 0, 0, 0, 0, 2],
                },
            );
            contracts::branch_contract(g.clone());
            act(
                &mut g,
                if pay_tax {
                    Action::PayResolutionMana {
                        payment: [0, 0, 0, 0, 0, 1],
                    }
                } else {
                    Action::DeclineResolutionMana
                },
            );
            finish(&mut g);
            assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
            assert_eq!(
                g.state.objects[bear as usize].damage,
                if pay_tax { 2 } else { 0 }
            );
            assert_eq!(
                g.state.players[p.other().index()].mana,
                [0, 0, 0, 0, 0, u32::from(!pay_tax)]
            );
        }
    }
}
#[test]
fn thorin_ward_artifact_or_creature_once_excludes_other_enchantments_and_enemy_objects() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..4 {
            let mut g = scenario(p);
            let (_, _, pick) = story(&mut g, p);
            let target = match mode {
                0 | 1 => pick,
                2 => put(&mut g, p, "Beorn's Hospitality", Zone::Battlefield),
                _ => put(&mut g, p.other(), "Bear Cub", Zone::Battlefield),
            };
            if mode == 1 {
                g.change_object_form(
                    g.object_ref(target),
                    crate::cards::AnimationSpec {
                        artifact: true,
                        creature_types: crate::cards::AnimationCreatureTypes::Unchanged,
                        base: Some((2, 2)),
                        keywords: Keywords::NONE,
                        duration: crate::cards::AnimationDuration::EndOfTurn,
                    },
                    characteristic_layers::AnimationTypeMode::AddCreature,
                );
            }
            let caster = if mode == 3 { p } else { p.other() };
            g.state.priority = caster;
            let spell = put(
                &mut g,
                caster,
                if mode < 3 { "Disenchant" } else { "Shock" },
                Zone::Hand,
            );
            g.state.players[caster.index()].mana = if mode < 3 {
                [1, 0, 0, 0, 0, 1]
            } else {
                [0, 0, 0, 1, 0, 0]
            };
            cast(&mut g, spell, Some(Target::Card(target)));
            assert_eq!(ward_count(&g), usize::from(mode < 2));
            if mode < 2 {
                ward(&mut g);
                act(&mut g, Action::DeclineResolutionMana);
            }
            finish(&mut g);
            if mode < 2 {
                assert_eq!(g.object_view(target).zone, Zone::Battlefield);
            } else if mode == 2 {
                assert_eq!(g.object_view(target).zone, Zone::Graveyard);
            }
        }
    }
}
#[test]
fn thorin_ward_targets_opponent_activated_ability_even_after_its_source_sacrificed() {
    for p in [Player::P0, Player::P1] {
        for pay_tax in [false, true] {
            let mut g = scenario(p);
            story(&mut g, p);
            let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let fire = put(&mut g, p.other(), "Fanatical Firebrand", Zone::Battlefield);
            g.state.priority = p.other();
            g.state.players[p.other().index()].mana = [0, 0, 0, 0, 0, 1];
            act(
                &mut g,
                Action::AnnounceActivate {
                    card: fire,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(bear),
                },
            );
            if g.legal_actions().contains(&Action::FinishTargets) {
                act(&mut g, Action::FinishTargets);
            }
            pay(&mut g);
            assert_eq!(g.object_view(fire).zone, Zone::Graveyard);
            assert_eq!(ward_count(&g), 1);
            ward(&mut g);
            act(
                &mut g,
                if pay_tax {
                    Action::PayResolutionMana {
                        payment: [0, 0, 0, 0, 0, 1],
                    }
                } else {
                    Action::DeclineResolutionMana
                },
            );
            finish(&mut g);
            assert_eq!(
                g.state.objects[bear as usize].damage,
                if pay_tax { 1 } else { 0 }
            );
            assert_eq!(g.object_view(fire).zone, Zone::Graveyard);
        }
    }
}
#[test]
fn thorin_story_gained_after_target_acquisition_does_not_retroactively_trigger_ward() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        paid_thorin(&mut g, p);
        put(&mut g, p, "Thrór's Map", Zone::Battlefield);
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.add_counter(bear, CounterKind::PlusOne, 1);
        let spell = put(&mut g, p.other(), "Shock", Zone::Hand);
        g.state.priority = p.other();
        g.state.players[p.other().index()].mana = [0, 0, 0, 1, 0, 0];
        act(&mut g, Action::AnnounceCast { card: spell });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        assert_eq!(ward_count(&g), 0);
        put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
        g.refresh_enduring_stories();
        assert!(g.state.players[p.index()].enduring_story);
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        pay(&mut g);
        assert_eq!(ward_count(&g), 0);
        finish(&mut g);
        assert_eq!(g.state.objects[bear as usize].damage, 2);
    }
}
#[test]
fn thorin_and_attached_ward_are_separate_payments_with_exact_related_stack() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        story(&mut g, p);
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let mattock = put(&mut g, p, "Dwarven Mattock", Zone::Battlefield);
        assert!(g.attach_equipment(g.object_ref(mattock), g.object_ref(bear)));
        let spell = shock(&mut g, p.other(), bear, 2);
        assert_eq!(ward_count(&g), 2);
        for remaining in [1, 0] {
            ward(&mut g);
            act(
                &mut g,
                Action::PayResolutionMana {
                    payment: [0, 0, 0, 0, 0, 1],
                },
            );
            assert_eq!(g.state.players[p.other().index()].mana[5], remaining);
        }
        finish(&mut g);
        assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
        assert_eq!(g.state.objects[bear as usize].damage, 2);
    }
}
#[test]
fn thorin_current_control_requires_current_controllers_story_and_protects_current_objects() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let (t, _, _) = story(&mut g, p);
        let bear = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.add_counter(bear, CounterKind::PlusOne, 2);
        g.state.objects[t as usize].controller = p.other();
        shock(&mut g, p, bear, 0);
        assert_eq!(ward_count(&g), 0);
        finish(&mut g);
        put(&mut g, p.other(), "Goldvein Pick", Zone::Battlefield);
        put(&mut g, p.other(), "Thrór's Map", Zone::Battlefield);
        g.refresh_enduring_stories();
        assert!(g.state.players[p.other().index()].enduring_story);
        shock(&mut g, p, bear, 1);
        assert_eq!(ward_count(&g), 1);
        ward(&mut g);
        act(&mut g, Action::DeclineResolutionMana);
        finish(&mut g);
        assert_eq!(g.state.objects[bear as usize].damage, 2);
    }
}
#[test]
fn thorin_ward_provider_ability_removal_suppresses_and_departure_restores_grant() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let (t, _, _) = story(&mut g, p);
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.add_counter(bear, CounterKind::PlusOne, 2);
        let aura = protection(&mut g, p, t);
        assert!(!g.printed_abilities_available(t));
        assert!(g.state.players[p.index()].enduring_story);
        shock(&mut g, p.other(), bear, 0);
        assert_eq!(ward_count(&g), 0);
        finish(&mut g);
        assert_eq!(g.state.objects[bear as usize].damage, 2);
        g.move_object(aura, Zone::Graveyard);
        shock(&mut g, p.other(), bear, 1);
        assert_eq!(ward_count(&g), 1);
        ward(&mut g);
        act(&mut g, Action::DeclineResolutionMana);
        finish(&mut g);
        assert_eq!(g.state.objects[bear as usize].damage, 2);
    }
}
#[test]
fn thorin_granted_ward_and_recipient_removal_follow_layer6_chronology() {
    for p in [Player::P0, Player::P1] {
        for earlier in [false, true] {
            let mut g = scenario(p);
            let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            g.add_counter(bear, CounterKind::PlusOne, 3);
            if earlier {
                protection(&mut g, p, bear);
            }
            story(&mut g, p);
            if !earlier {
                protection(&mut g, p, bear);
            }
            shock(&mut g, p.other(), bear, 0);
            assert_eq!(ward_count(&g), usize::from(earlier));
            if earlier {
                ward(&mut g);
                act(&mut g, Action::DeclineResolutionMana);
            }
            finish(&mut g);
            assert_eq!(
                g.state.objects[bear as usize].damage,
                if earlier { 0 } else { 2 }
            );
        }
    }
}
#[test]
fn thorin_copied_provider_acquires_ward_and_queued_packet_survives_later_copy_change() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let (t, _, _) = story(&mut g, p);
        g.move_object(t, Zone::Graveyard);
        let provider = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.apply_temporary_copy(
            g.object_ref(provider),
            CopyRecipe::new(Card::from_name("Thorin Oakenshield").unwrap()),
            CopyDeadline::EndOfTurn,
        );
        let spell = shock(&mut g, p.other(), bear, 1);
        assert_eq!(ward_count(&g), 1);
        g.apply_temporary_copy(
            g.object_ref(provider),
            CopyRecipe::new(Card::from_name("Bear Cub").unwrap()),
            CopyDeadline::EndOfTurn,
        );
        ward(&mut g);
        act(&mut g, Action::DeclineResolutionMana);
        finish(&mut g);
        assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
        assert_eq!(g.state.objects[bear as usize].damage, 0);
        g.add_counter(bear, CounterKind::PlusOne, 1);
        shock(&mut g, p.other(), bear, 0);
        assert_eq!(ward_count(&g), 0);
        finish(&mut g);
        assert_eq!(g.state.objects[bear as usize].damage, 2);
    }
}
