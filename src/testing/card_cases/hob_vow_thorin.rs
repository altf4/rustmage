//! Focused printed outcomes; shared engine invariants live in core tests.
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

#[test]
fn vow_paid_instant_untaps_and_pumps_without_dwarf_and_expires() {
    {
        let p = Player::P0;
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
    {
        let p = Player::P0;
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
fn thorin_ward_artifact_or_creature_once_excludes_other_enchantments_and_enemy_objects() {
    {
        let p = Player::P0;
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
    {
        let p = Player::P0;
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
fn thorin_and_attached_ward_are_separate_payments_with_exact_related_stack() {
    {
        let p = Player::P0;
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
