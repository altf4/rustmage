//! Issue257 whole-body and exact recipient grant contracts. Native setup fixtures
//! complement separate real paid public-library games; setup is not a public play claim.
use super::tests::{act, cast, pass_pair, scenario, spawn};
use super::*;
fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let card = Card::from_name(name).unwrap();
    assert!(card.is_supported(), "{name}");
    spawn(g, p, card, zone)
}
fn replay(g: &mut Game, action: Action) {
    let saved = g.snapshot();
    let mut fork = g.clone();
    act(&mut fork, action.clone());
    act(g, action.clone());
    assert_eq!(g.snapshot(), fork.snapshot());
    let after = g.snapshot();
    g.restore(&saved).unwrap();
    act(g, action);
    assert_eq!(after, g.snapshot());
}
fn aura(g: &mut Game, p: Player, name: &str, target: CardId) -> CardId {
    let id = named(g, p, name, Zone::Hand);
    let mut movement = g.entry_move(id, Zone::Battlefield, false);
    movement.attachment = Some(g.object_ref(target));
    g.stage_entries(vec![movement], None);
    id
}
#[test]
fn complete_attachment_ward_definitions_have_all_residuals() {
    for name in [
        "Dusk Rose Reliquary",
        "Trapped in the Screen",
        "Dwarven Mattock",
        "Falcon's Wing Harness",
        "Lavaspur Boots",
        "Hardlight Containment",
        "Sheltered by Ghosts",
        "Super Strength",
    ] {
        let c = Card::from_name(name).unwrap();
        assert!(c.is_supported(), "{name}");
        assert!(!c.triggers().is_empty(), "{name}");
    }
    let relic = Card::from_name("Dusk Rose Reliquary").unwrap();
    assert!(relic.additional_cost().is_some());
    for (name, p, t, k) in [
        ("Dwarven Mattock", 2, 2, Keywords::NONE),
        ("Falcon's Wing Harness", 1, 1, Keywords::FLYING),
        ("Lavaspur Boots", 1, 0, Keywords::HASTE),
        ("Sheltered by Ghosts", 1, 0, Keywords::LIFELINK),
        ("Super Strength", 4, 4, Keywords::TRAMPLE),
    ] {
        let c = Card::from_name(name).unwrap();
        let b = c.attachment_bonus().unwrap();
        assert_eq!(
            b.stats,
            Stats {
                power: p,
                toughness: t
            }
        );
        assert!(b.keywords.contains(k));
        assert_eq!(
            c.triggers()
                .iter()
                .filter(
                    |a| a.condition.is_granted() && a.condition.event() == TriggerCondition::Ward
                )
                .count(),
            1
        );
    }
}
#[test]
fn unrestricted_aura_ward_uses_recipient_controller_and_survives_detachment() {
    for p in [Player::P0, Player::P1] {
        for pay in [false, true] {
            let mut g = scenario();
            let bear = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
            let e = aura(&mut g, p.other(), "Super Strength", bear);
            assert_eq!(
                g.stats(bear),
                Stats {
                    power: 6,
                    toughness: 6
                }
            );
            let shock = named(&mut g, p.other(), "Shock", Zone::Hand);
            g.state.priority = p.other();
            g.state.players[p.other().index()].mana = [10; 6];
            cast(&mut g, shock, Some(Target::Card(bear)));
            assert_eq!(g.state.stack.len(), 2);
            let packet = g.state.stack.last().unwrap();
            assert_eq!(packet.controller, p);
            assert_eq!(packet.object, g.object_ref(bear));
            assert_eq!(packet.related_stack.unwrap().controller, p.other());
            g.move_object(e, Zone::Graveyard);
            pass_pair(&mut g);
            assert_eq!(g.actor(), Some(p.other()));
            assert!(g.observe(p).actions.is_empty());
            let action = g
                .legal_actions()
                .into_iter()
                .find(|a| {
                    if pay {
                        matches!(a, Action::PayResolutionMana { .. })
                    } else {
                        matches!(a, Action::DeclineResolutionMana)
                    }
                })
                .unwrap();
            replay(&mut g, action);
            if pay {
                pass_pair(&mut g);
            }
            assert_eq!(
                g.object_view(bear).zone,
                if pay {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
        }
    }
}
#[test]
fn recipient_own_spell_does_not_trigger_aura_owner_ward() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario();
        let bear = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        aura(&mut g, p.other(), "Super Strength", bear);
        let spell = named(&mut g, p, "Giant Growth", Zone::Hand);
        g.state.priority = p;
        g.state.players[p.index()].mana = [10; 6];
        cast(&mut g, spell, Some(Target::Card(bear)));
        assert_eq!(g.state.stack.len(), 1);
    }
}
#[test]
fn equal_granted_instances_and_printed_ward_remain_independent() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario();
        let target = named(&mut g, p, "Toadstool Admirer", Zone::Battlefield);
        for _ in 0..2 {
            let e = named(&mut g, p, "Lavaspur Boots", Zone::Battlefield);
            assert!(g.attach_equipment(g.object_ref(e), g.object_ref(target)));
        }
        let shock = named(&mut g, p.other(), "Shock", Zone::Hand);
        g.state.priority = p.other();
        g.state.players[p.other().index()].mana = [10; 6];
        cast(&mut g, shock, Some(Target::Card(target)));
        while let Some(a) = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PutTrigger { .. }))
        {
            replay(&mut g, a);
        }
        assert_eq!(g.state.stack.len(), 4);
        let victim = g.state.stack[0].id;
        assert!(
            g.state.stack[1..]
                .iter()
                .all(|x| x.controller == p && x.related_stack.unwrap().id == victim)
        );
    }
}
#[test]
fn hardlight_enchant_predicate_accepts_noncreature_artifact_and_exact_refs() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario();
        let artifact = named(&mut g, p, "Candy Trail", Zone::Battlefield);
        let own = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let foreign = named(&mut g, p.other(), "Candy Trail", Zone::Battlefield);
        let hard = Card::from_name("Hardlight Containment").unwrap();
        assert!(g.aura_can_enchant(hard, p, g.object_ref(artifact)));
        assert!(!g.aura_can_enchant(hard, p, g.object_ref(own)));
        assert!(!g.aura_can_enchant(hard, p, g.object_ref(foreign)));
        let e = aura(&mut g, p, "Hardlight Containment", artifact);
        // Clear the targetless ETB trigger fixture: grants are independently tested below.
        g.state.pending_triggers.clear();
        assert_eq!(g.attachment_recipient(e), Some(g.object_ref(artifact)));
        let old = g.object_ref(artifact);
        g.move_object(artifact, Zone::Hand);
        g.move_object(artifact, Zone::Battlefield);
        assert!(!g.aura_can_enchant(hard, p, old));
        assert_eq!(g.attachment_recipient(e), None);
    }
}

fn settle(g: &mut Game, preferred: CardId) {
    g.prepare_triggers();
    for _ in 0..100 {
        if g.state.stack.is_empty()
            && g.state.ordering_triggers.is_empty()
            && g.state.announcing.is_none()
        {
            return;
        }
        let actions = g.legal_actions();
        let a = actions
            .iter()
            .find(|a| matches!(a,Action::ChooseTarget{target:Target::Card(id)} if *id==preferred))
            .or_else(|| {
                actions.iter().find(|a| {
                    matches!(
                        a,
                        Action::PutTrigger { .. }
                            | Action::ChooseTarget { .. }
                            | Action::FinishTargets
                            | Action::PayCosts { .. }
                            | Action::Pass
                    )
                })
            })
            .expect("owned complete continuation")
            .clone();
        replay(g, a);
    }
    panic!("unsettled {:?}", g.observe(g.actor().unwrap()).decision);
}
#[test]
fn actual_paid_hardlight_cast_exiles_and_source_leave_returns_before_next_instruction() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario();
        let artifact = named(&mut g, p, "Candy Trail", Zone::Battlefield);
        let bear = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        let hard = named(&mut g, p, "Hardlight Containment", Zone::Hand);
        g.state.active = p;
        g.state.priority = p;
        g.state.players[p.index()].mana = [10; 6];
        cast(&mut g, hard, Some(Target::Card(artifact)));
        settle(&mut g, bear);
        assert_eq!(
            g.object_view(hard).attached_to,
            Some(g.object_ref(artifact))
        );
        assert_eq!(g.object_view(bear).zone, Zone::Exile);
        // Opponent targets a noncreature artifact that genuinely has granted Ward.
        let disenchant = named(&mut g, p.other(), "Disenchant", Zone::Hand);
        g.state.priority = p.other();
        g.state.players[p.other().index()].mana = [10; 6];
        cast(&mut g, disenchant, Some(Target::Card(artifact)));
        assert_eq!(g.state.stack.len(), 2);
        assert_eq!(g.state.stack.last().unwrap().controller, p);
        pass_pair(&mut g);
        replay(&mut g, Action::DeclineResolutionMana);
        assert_eq!(g.object_view(artifact).zone, Zone::Battlefield);
        let disenchant = named(&mut g, p.other(), "Disenchant", Zone::Hand);
        g.state.priority = p.other();
        cast(&mut g, disenchant, Some(Target::Card(hard)));
        settle(&mut g, hard);
        assert_eq!(g.object_view(hard).zone, Zone::Graveyard);
        assert_eq!(g.object_view(bear).zone, Zone::Battlefield);
        assert_eq!(g.attachment_recipient(hard), None);
    }
}
#[test]
fn actual_paid_equipment_entries_and_equip_keep_full_stats_keywords_and_subtype_filters() {
    for p in [Player::P0, Player::P1] {
        for (name, delta, keyword) in [
            ("Dwarven Mattock", 2, Keywords::NONE),
            ("Falcon's Wing Harness", 1, Keywords::FLYING),
            ("Lavaspur Boots", 1, Keywords::HASTE),
        ] {
            let mut g = scenario();
            g.state.active = p;
            g.state.priority = p;
            let bear = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
            let dwarf = named(&mut g, p, "Axgard Cavalry", Zone::Battlefield);
            let item = named(&mut g, p, name, Zone::Hand);
            g.state.players[p.index()].mana = [20; 6];
            cast(&mut g, item, None);
            settle(&mut g, dwarf);
            if name != "Lavaspur Boots" {
                assert_eq!(g.object_view(item).attached_to, Some(g.object_ref(dwarf)));
            }
            let equip=g.legal_actions().into_iter().find(|a|matches!(a,Action::Activate{card,target:Some(Target::Card(t)),..} if *card==item && *t==bear)).unwrap();
            replay(&mut g, equip);
            settle(&mut g, bear);
            assert_eq!(g.object_view(item).attached_to, Some(g.object_ref(bear)));
            assert_eq!(g.stats(bear).power, 2 + delta);
            assert!(g.keywords(bear).contains(keyword));
        }
    }
}
#[test]
fn actual_reliquary_additional_sacrifice_and_screen_union_targets_are_complete() {
    for p in [Player::P0, Player::P1] {
        for name in ["Dusk Rose Reliquary", "Trapped in the Screen"] {
            for enemy in ["Grizzly Bears", "Candy Trail", "Banishing Light"] {
                if name == "Dusk Rose Reliquary" && enemy == "Banishing Light" {
                    continue;
                }
                let mut g = scenario();
                g.state.active = p;
                g.state.priority = p;
                g.state.players[p.index()].mana = [20; 6];
                let sacrifice = named(&mut g, p, "Candy Trail", Zone::Battlefield);
                let target = named(&mut g, p.other(), enemy, Zone::Battlefield);
                let source = named(&mut g, p, name, Zone::Hand);
                replay(&mut g, Action::AnnounceCast { card: source });
                if name == "Dusk Rose Reliquary" {
                    let reference = g.object_ref(sacrifice);
                    replay(&mut g, Action::ChooseAdditionalCost { object: reference });
                }
                settle(&mut g, target);
                assert_eq!(g.object_view(target).zone, Zone::Exile);
                if name == "Dusk Rose Reliquary" {
                    assert_eq!(g.object_view(sacrifice).zone, Zone::Graveyard);
                }
                g.move_object(source, Zone::Graveyard);
                assert_eq!(g.object_view(target).zone, Zone::Battlefield);
            }
        }
    }
}

#[test]
fn static_snare_actual_opposing_attackers_reduce_payment_and_preserve_owned_counts() {
    for p in [Player::P0, Player::P1] {
        for count in [1, 2] {
            let mut g = scenario();
            g.state.active = p.other();
            g.state.priority = p.other();
            g.state.step = Step::Attackers;
            g.state.pending = Pending::Attackers;
            let mut attacking = Vec::new();
            for _ in 0..count {
                attacking.push(named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield));
            }
            for &id in &attacking {
                replay(&mut g, Action::Attack { card: id });
            }
            replay(&mut g, Action::FinishAttackers);
            let snare = named(&mut g, p, "Static Snare", Zone::Hand);
            g.state.priority = p;
            let reference = g.object_ref(snare);
            assert_eq!(g.stat_value(StatValue::AttackingCreatures, reference, p), 0);
            assert_eq!(
                g.stat_value(StatValue::AllAttackingCreatures, reference, p),
                count
            );
            assert_eq!(g.spell_reduction(p, snare), count as u32);
            g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 4 - count as u32];
            cast(&mut g, snare, None);
            assert_eq!(
                g.state.stack.last().unwrap().cast.unwrap().payment,
                [1, 0, 0, 0, 0, 4 - count as u32]
            );
            settle(&mut g, attacking[0]);
            assert_eq!(g.object_view(attacking[0]).zone, Zone::Exile);
            g.move_object(snare, Zone::Graveyard);
            assert_eq!(g.object_view(attacking[0]).zone, Zone::Battlefield);
        }
    }
}
#[test]
fn mandatory_reliquary_cost_cannot_reuse_its_only_mana_sacrifice() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario();
        g.state.active = p;
        g.state.priority = p;
        let pizza = named(&mut g, p, "Omni-Cheese Pizza", Zone::Battlefield);
        let source = named(&mut g, p, "Dusk Rose Reliquary", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        let before = g.snapshot();
        assert!(
            !g.legal_actions()
                .contains(&Action::AnnounceCast { card: source })
        );
        assert_eq!(g.snapshot(), before);
        let bear = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        replay(&mut g, Action::AnnounceCast { card: source });
        let reference = g.object_ref(bear);
        replay(&mut g, Action::ChooseAdditionalCost { object: reference });
        let mana = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a,Action::ActivateMana{card,..}if *card==pizza))
            .unwrap();
        replay(&mut g, mana);
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        replay(&mut g, pay);
        assert_eq!(g.object_view(pizza).zone, Zone::Graveyard);
        assert_eq!(g.object_view(bear).zone, Zone::Graveyard);
        assert_eq!(g.object_view(source).zone, Zone::Stack);
    }
}
#[test]
fn noncast_opposing_ward_recipient_attachment_is_a_choice_not_a_target() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario();
        let recipient = named(&mut g, p.other(), "Toadstool Admirer", Zone::Battlefield);
        let source = named(&mut g, p, "Super Strength", Zone::Hand);
        g.stage_entries(vec![g.entry_move(source, Zone::Battlefield, false)], None);
        assert!(g.advance_entries());
        let old = g.object_ref(recipient);
        replay(&mut g, Action::ChooseAuraAttachment { object: old });
        assert_eq!(g.object_view(source).attached_to, Some(old));
        assert!(g.state.stack.is_empty());
        assert!(g.state.pending_triggers.is_empty());
        assert_eq!(g.attachment_recipient(source), Some(old));
    }
}
