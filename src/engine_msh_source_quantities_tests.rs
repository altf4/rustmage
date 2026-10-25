//! Complete paid representatives for selected current/departure source quantities.
use super::ability_world::CharacteristicWorld;
use super::assurance::big_helpers::{cast, checked, named, next_turn, settle, setup};
use super::*;
const WINGS: &str = "Captain America, Wings of Freedom";
const WHIP: &str = "Whiplash, Vengeful Engineer";
fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let revision = g.revision();
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
}
fn priority(g: &mut Game, player: Player) {
    if g.actor() != Some(player) {
        checked(g, Action::Pass);
    }
    assert_eq!(g.actor(), Some(player));
}
fn paid(g: &mut Game, player: Player, name: &str, targets: &[Target]) -> CardId {
    priority(g, player);
    let id = named(g, player, name, Zone::Hand);
    let (colored, generic) = g.object_view(id).card.cost();
    g.state.players[player.index()].mana = colored;
    g.state.players[player.index()].mana[5] += generic;
    cast(g, id, None, targets);
    settle(g);
    id
}
fn finish(g: &mut Game) {
    for _ in 0..15 {
        if g.state.announcing.is_none() {
            return;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
            .unwrap();
        checked(g, action);
    }
    panic!("unfinished quantity scenario payment");
}
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..100 {
        if stop(g) {
            return;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass
                        | Action::PutTrigger { .. }
                        | Action::FinishTargets
                        | Action::FinishAttackers
                        | Action::FinishBlockers
                )
            })
            .unwrap();
        checked(g, action);
    }
    panic!("quantity scenario stalled {:?}", g.state.pending);
}
fn declare(g: &mut Game, p: Player, id: CardId) {
    g.state.active = p;
    g.state.priority = p;
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    // Existing combat scenario boundary; separate body test verifies actual untap.
    g.state.objects[id as usize].summoning_sick = false;
    g.state.objects[id as usize].tapped = false;
    checked(g, Action::Attack { card: id });
    checked(g, Action::FinishAttackers);
    g.prepare_triggers();
}
fn equip(g: &mut Game, p: Player, equipment: CardId, target: CardId) {
    priority(g, p);
    let definition = g.object_view(equipment).card.activation(0);
    g.state.players[p.index()].mana = definition.cost.colored;
    g.state.players[p.index()].mana[5] += definition.cost.generic;
    checked(
        g,
        Action::AnnounceActivate {
            card: equipment,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    checked(
        g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    finish(g);
    settle(g);
    assert_eq!(
        g.object_view(equipment).attached_to,
        Some(g.object_ref(target))
    );
}
fn armed(g: &mut Game, p: Player) -> (CardId, CardId) {
    let id = paid(g, p, WHIP, &[]);
    let equipment = paid(g, p, "Vibranium Energy Daggers", &[]);
    equip(g, p, equipment, id);
    (id, equipment)
}
fn hero(g: &mut Game, p: Player) -> CardId {
    named(g, p, "Brave Brawler", Zone::Battlefield)
}
fn types(g: &mut Game, id: CardId, names: &'static [&'static str]) {
    let recipient = g.object_ref(id);
    let timestamp = g.next_role_timestamp();
    g.state
        .roles
        .get_or_insert_default()
        .creature_types
        .push(subtypes::CreatureTypesEffectView {
            recipient,
            source: recipient,
            types: names,
            additional: false,
            timestamp,
        });
}
fn moved(g: &mut Game, ids: &[CardId], zone: Zone) {
    let events = g.move_objects_raw(ids, zone);
    g.finish_zone_events(&events, &[]);
}
fn packet(g: &Game, id: CardId) -> &SpellView {
    g.state
        .stack
        .iter()
        .chain(&g.state.pending_triggers)
        .chain(&g.state.ordering_triggers)
        .find(|item| item.card == id)
        .unwrap()
}
#[test]
fn msh_source_quantities_paid_bodies_entry_tap_exact_cost_and_real_ready_turn() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let c = paid(&mut g, p, WINGS, &[]);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        let v = g.object_view(c);
        assert_eq!((v.power, v.toughness, v.colors), (3, 1, 1));
        assert!(
            v.card.info().legendary
                && v.has_subtype("Human")
                && v.has_subtype("Soldier")
                && v.has_subtype("Hero")
        );
        assert!(
            v.keywords
                .contains(Keywords::FLYING.union(Keywords::FIRST_STRIKE))
        );
        assert!(!v.tapped);
        let w = paid(&mut g, p, WHIP, &[]);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        let v = g.object_view(w);
        assert_eq!((v.power, v.toughness, v.colors), (2, 2, 4));
        assert!(
            v.tapped
                && v.card.info().legendary
                && v.has_subtype("Human")
                && v.has_subtype("Artificer")
                && v.has_subtype("Villain")
        );
        assert!(g.state.tap_history.is_none());
        next_turn(&mut g);
        next_turn(&mut g);
        assert!(!g.object_view(w).tapped && !g.object_view(w).summoning_sick);
    }
}
#[test]
fn msh_source_quantities_wings_paid_ward_pay_decline_and_friendly_target() {
    for p in [Player::P0, Player::P1] {
        for pay in [false, true] {
            let mut g = setup(p);
            let c = paid(&mut g, p, WINGS, &[]);
            let secret = named(&mut g, p, "Divination", Zone::Hand);
            priority(&mut g, p.other());
            let spell = named(&mut g, p.other(), "Shock", Zone::Hand);
            g.state.players[p.other().index()].mana = [0, 0, 0, 1, 0, 1];
            cast(&mut g, spell, None, &[Target::Card(c)]);
            until(&mut g, |g| {
                g.legal_actions().contains(&Action::DeclineResolutionMana)
            });
            reject(&mut g, Action::PayResolutionMana { payment: [0; 6] });
            if pay {
                checked(
                    &mut g,
                    Action::PayResolutionMana {
                        payment: [0, 0, 0, 0, 0, 1],
                    },
                );
            } else {
                checked(&mut g, Action::DeclineResolutionMana);
            }
            settle(&mut g);
            assert_eq!(
                g.object_view(c).zone,
                if pay {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
            assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
            assert!(
                !g.observe(p.other())
                    .objects
                    .iter()
                    .any(|view| view.reference.card == secret)
            );
        }
        let mut g = setup(p);
        let c = paid(&mut g, p, WINGS, &[]);
        paid(&mut g, p, "Giant Growth", &[Target::Card(c)]);
        assert_eq!(g.stats(c).toughness, 4);
        assert!(g.state.stack.is_empty());
    }
}
#[test]
fn msh_source_quantities_wings_flying_and_first_strike_actual_block_and_damage() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let c = paid(&mut g, p, WINGS, &[]);
        let ground = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let reach = named(&mut g, p.other(), "Kree Sentinel", Zone::Battlefield);
        let flyer = named(&mut g, p.other(), "Healer's Hawk", Zone::Battlefield);
        declare(&mut g, p, c);
        settle(&mut g);
        until(&mut g, |g| g.state.pending == Pending::Blockers);
        reject(
            &mut g,
            Action::Block {
                blocker: ground,
                attacker: c,
            },
        );
        assert!(g.legal_actions().contains(&Action::Block {
            blocker: reach,
            attacker: c
        }));
        checked(
            &mut g,
            Action::Block {
                blocker: flyer,
                attacker: c,
            },
        );
        checked(&mut g, Action::FinishBlockers);
        until(&mut g, |g| g.state.step == Step::FirstStrikeDamage);
        until(&mut g, |g| g.state.step == Step::CombatDamage);
        assert_eq!(g.object_view(flyer).zone, Zone::Graveyard);
        assert_eq!(g.object_view(c).zone, Zone::Battlefield);
        assert_eq!(g.object_view(c).damage, 0);
    }
}
#[test]
fn msh_source_quantities_wings_live_toughness_frozen_amount_and_hero_permanent_set() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let c = paid(&mut g, p, WINGS, &[]);
        let h = hero(&mut g, p);
        let enemy = hero(&mut g, p.other());
        let ordinary = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let kindred = paid(&mut g, p, "Clachan Festival", &[]);
        types(&mut g, kindred, &["Hero"]);
        assert!(!g.is_creature(kindred));
        let before = g.stats(h);
        declare(&mut g, p, c);
        assert_eq!(
            packet(&g, c)
                .source_quantity_lki
                .as_ref()
                .unwrap()
                .toughness,
            Some(1)
        );
        priority(&mut g, p);
        let grow = named(&mut g, p, "Giant Growth", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 0];
        cast(&mut g, grow, None, &[Target::Card(c)]);
        let late = hero(&mut g, p);
        settle(&mut g);
        assert_eq!(g.stats(c).toughness, 4);
        assert_eq!(
            g.stats(h),
            Stats {
                power: before.power + 4,
                toughness: before.toughness + 4
            }
        );
        assert_eq!(
            g.state.objects[kindred as usize].growth,
            Stats {
                power: 4,
                toughness: 4
            }
        );
        assert_eq!(g.state.objects[enemy as usize].growth, Stats::default());
        assert_eq!(g.state.objects[ordinary as usize].growth, Stats::default());
        assert_eq!(g.state.objects[late as usize].growth.power, 4);
        let later = hero(&mut g, p);
        assert_eq!(g.state.objects[later as usize].growth, Stats::default());
        // A noncreature Hero has a real stored effect; the existing production animation layer consumes it.
        let object = g.object_ref(kindred);
        let timestamp = g.next_role_timestamp();
        g.state.roles.get_or_insert_default().animations.push(
            characteristic_layers::AnimationView {
                object,
                timestamp,
                artifact: false,
                preserve_card_types: false,
                colors: None,
                creature_types: Some(&["Hero"]),
                all_creature_types: false,
                base: Some((2, 2)),
                keywords: Keywords::NONE,
                expires_turn: Some(g.state.turn),
            },
        );
        assert_eq!(
            g.stats(kindred),
            Stats {
                power: 6,
                toughness: 6
            }
        );
        types(&mut g, h, &["Citizen"]);
        assert_eq!(g.state.objects[h as usize].growth.power, 4);
        moved(&mut g, &[h], Zone::Exile);
        moved(&mut g, &[h], Zone::Battlefield);
        assert_eq!(g.state.objects[h as usize].growth, Stats::default());
        next_turn(&mut g);
        assert_eq!(g.state.objects[late as usize].growth, Stats::default());
    }
}
#[test]
fn msh_source_quantities_wings_departure_uses_latest_toughness_and_reentry_is_other() {
    for p in [Player::P0, Player::P1] {
        for reenter in [false, true] {
            let mut g = setup(p);
            let c = paid(&mut g, p, WINGS, &[]);
            let h = hero(&mut g, p);
            let old = g.object_ref(c);
            declare(&mut g, p, c);
            g.effect_counter_by(
                c,
                CounterKind::PlusOne,
                3,
                p,
                replacements::ResumeMode::Pending,
            );
            moved(&mut g, &[c], Zone::Exile);
            assert_eq!(
                packet(&g, c)
                    .source_quantity_lki
                    .as_ref()
                    .unwrap()
                    .toughness,
                Some(4)
            );
            if reenter {
                moved(&mut g, &[c], Zone::Battlefield);
                assert_ne!(g.object_ref(c), old);
            }
            settle(&mut g);
            assert_eq!(g.state.objects[h as usize].growth.power, 4);
            if reenter {
                assert_eq!(
                    g.stats(c),
                    Stats {
                        power: 7,
                        toughness: 5
                    }
                );
            }
        }
    }
}
#[test]
fn msh_source_quantities_wings_simultaneous_departure_freezes_aura_equipment_boost_all_orders() {
    for p in [Player::P0, Player::P1] {
        for source_first in [false, true] {
            let mut g = setup(p);
            let c = paid(&mut g, p, WINGS, &[]);
            let h = hero(&mut g, p);
            let aura = paid(&mut g, p, "Super Strength", &[Target::Card(c)]);
            let equipment = paid(&mut g, p, "Vibranium Energy Daggers", &[]);
            equip(&mut g, p, equipment, c);
            assert_eq!(g.stats(c).toughness, 7);
            declare(&mut g, p, c);
            moved(
                &mut g,
                &if source_first {
                    vec![c, aura, equipment]
                } else {
                    vec![equipment, aura, c]
                },
                Zone::Exile,
            );
            settle(&mut g);
            assert_eq!(g.state.objects[h as usize].growth.power, 7);
        }
    }
}
#[test]
fn msh_source_quantities_wings_source_theft_preserves_trigger_controller_recipient_set() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let c = paid(&mut g, p, WINGS, &[]);
        let own = hero(&mut g, p);
        let enemy = hero(&mut g, p.other());
        declare(&mut g, p, c);
        g.record_control(g.object_ref(c), p.other(), None);
        g.effect_counter_by(
            c,
            CounterKind::PlusOne,
            2,
            p.other(),
            replacements::ResumeMode::Pending,
        );
        settle(&mut g);
        assert_eq!(g.state.objects[own as usize].growth.power, 3);
        assert_eq!(g.state.objects[enemy as usize].growth.power, 0);
        assert_eq!(g.state.objects[c as usize].growth.power, 0);
    }
}
#[test]
fn msh_source_quantities_wings_zero_negative_and_noncreature_source_have_zero_bonus() {
    for p in [Player::P0, Player::P1] {
        for value in [-2, 0] {
            let mut g = setup(p);
            let c = paid(&mut g, p, WINGS, &[]);
            let h = hero(&mut g, p);
            declare(&mut g, p, c);
            // A production simultaneous movement freezes the final signed battlefield value, before an SBA can remove it separately.
            g.state.objects[c as usize].growth.toughness = value - 1;
            moved(&mut g, &[c], Zone::Exile);
            assert_eq!(
                packet(&g, c)
                    .source_quantity_lki
                    .as_ref()
                    .unwrap()
                    .toughness,
                Some(value)
            );
            settle(&mut g);
            assert_eq!(g.state.objects[h as usize].growth, Stats::default());
        }
        let mut g = setup(p);
        let c = paid(&mut g, p, WINGS, &[]);
        let h = hero(&mut g, p);
        declare(&mut g, p, c);
        let aura = named(&mut g, p, "Imprisoned in the Moon", Zone::Battlefield);
        g.state.objects[aura as usize].attached_to = Some(g.object_ref(c));
        assert!(!g.is_creature(c));
        settle(&mut g);
        assert_eq!(g.state.objects[h as usize].growth, Stats::default());
    }
}
#[test]
fn msh_source_quantities_both_suppression_before_declaration_and_after_queue() {
    for p in [Player::P0, Player::P1] {
        for name in [WINGS, WHIP] {
            for after in [false, true] {
                let mut g = setup(p);
                let id = paid(&mut g, p, name, &[]);
                let h = hero(&mut g, p);
                if name == WHIP {
                    let e = paid(&mut g, p, "Vibranium Energy Daggers", &[]);
                    equip(&mut g, p, e, id);
                }
                if !after {
                    paid(&mut g, p, "Witness Protection", &[Target::Card(id)]);
                }
                declare(&mut g, p, id);
                if after {
                    priority(&mut g, p);
                    let aura = named(&mut g, p, "Eaten by Piranhas", Zone::Hand);
                    g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 2];
                    cast(&mut g, aura, None, &[Target::Card(id)]);
                } else {
                    assert!(g.state.stack.is_empty());
                }
                settle(&mut g);
                if name == WINGS {
                    assert_eq!(
                        g.state.objects[h as usize].growth.power,
                        if after { 1 } else { 0 }
                    );
                } else {
                    assert_eq!(g.life(p.other()), if after { 19 } else { 20 });
                }
            }
        }
    }
}
#[test]
fn msh_source_quantities_whiplash_unequipped_never_triggers_and_equipped_guard_rechecks() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, WHIP, &[]);
        declare(&mut g, p, id);
        assert!(g.state.stack.is_empty());
        settle(&mut g);
        assert_eq!((g.life(p), g.life(p.other())), (20, 20));
        for reattach in [false, true] {
            let mut g = setup(p);
            let (id, e) = armed(&mut g, p);
            declare(&mut g, p, id);
            g.state.objects[e as usize].attached_to = None;
            if reattach {
                assert!(g.attach_equipment(g.object_ref(e), g.object_ref(id)));
            }
            settle(&mut g);
            assert_eq!(
                (g.life(p), g.life(p.other())),
                if reattach { (21, 19) } else { (20, 20) }
            );
        }
    }
}
#[test]
fn msh_source_quantities_whiplash_live_equipment_count_changes_without_retargeting() {
    for p in [Player::P0, Player::P1] {
        for count in [1, 2, 3] {
            let mut g = setup(p);
            let (id, _) = armed(&mut g, p);
            let aura = paid(&mut g, p, "Super Strength", &[Target::Card(id)]);
            assert!(g.object_view(aura).attached_to.is_some());
            let mut extra = Vec::new();
            for _ in 1..count {
                let e = paid(&mut g, p, "Vibranium Energy Daggers", &[]);
                extra.push(e);
            }
            declare(&mut g, p, id);
            for e in extra {
                assert!(g.attach_equipment(g.object_ref(e), g.object_ref(id)));
            }
            settle(&mut g);
            assert_eq!((g.life(p), g.life(p.other())), (20 + count, 20 - count));
        }
    }
}
#[test]
fn msh_source_quantities_whiplash_different_equipment_controller_and_captured_ability_controller() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let (id, e) = armed(&mut g, p);
        let e2 = paid(&mut g, p, "Vibranium Energy Daggers", &[]);
        equip(&mut g, p, e2, id);
        g.record_control(g.object_ref(e), p.other(), None);
        declare(&mut g, p, id);
        g.record_control(g.object_ref(id), p.other(), None);
        settle(&mut g);
        assert_eq!((g.life(p), g.life(p.other())), (22, 18));
        assert_eq!(g.object_view(e).controller, p.other());
    }
}
#[test]
fn msh_source_quantities_whiplash_latest_departure_count_survives_equipment_moves_and_reentry() {
    for p in [Player::P0, Player::P1] {
        for reenter in [false, true] {
            let mut g = setup(p);
            let (id, e) = armed(&mut g, p);
            let extra = paid(&mut g, p, "Vibranium Energy Daggers", &[]);
            let old = g.object_ref(id);
            declare(&mut g, p, id);
            assert!(g.attach_equipment(g.object_ref(extra), g.object_ref(id)));
            moved(&mut g, &[id], Zone::Exile);
            assert_eq!(
                packet(&g, id)
                    .source_quantity_lki
                    .as_ref()
                    .unwrap()
                    .equipment_count,
                Some(2)
            );
            moved(&mut g, &[e, extra], Zone::Exile);
            if reenter {
                moved(&mut g, &[id], Zone::Battlefield);
                assert_ne!(g.object_ref(id), old);
            }
            settle(&mut g);
            assert_eq!((g.life(p), g.life(p.other())), (22, 18));
        }
    }
}
#[test]
fn msh_source_quantities_whiplash_simultaneous_departure_is_order_independent() {
    for p in [Player::P0, Player::P1] {
        for source_first in [false, true] {
            let mut g = setup(p);
            let (id, e) = armed(&mut g, p);
            let e2 = paid(&mut g, p, "Vibranium Energy Daggers", &[]);
            equip(&mut g, p, e2, id);
            declare(&mut g, p, id);
            moved(
                &mut g,
                &if source_first {
                    vec![id, e, e2]
                } else {
                    vec![e2, e, id]
                },
                Zone::Exile,
            );
            assert_eq!(
                packet(&g, id)
                    .source_quantity_lki
                    .as_ref()
                    .unwrap()
                    .equipment_count,
                Some(2)
            );
            settle(&mut g);
            assert_eq!((g.life(p), g.life(p.other())), (22, 18));
        }
    }
}
#[test]
fn msh_source_quantities_whiplash_current_effective_subtype_and_suppressed_equipment_count() {
    for p in [Player::P0, Player::P1] {
        for lost_type in [false, true] {
            let mut g = setup(p);
            let (id, e) = armed(&mut g, p);
            declare(&mut g, p, id);
            if lost_type {
                // Existing production type layer; the final land is a valid Moon recipient.
                let aura = named(&mut g, p, "Imprisoned in the Moon", Zone::Battlefield);
                g.state.objects[aura as usize].attached_to = Some(g.object_ref(e));
                assert!(!g.has_subtype(e, "Equipment"));
                let item = packet(&g, id).clone();
                assert_eq!(g.source_equipment_quantity(&item), 0);
                settle(&mut g);
            } else {
                // A real paid flash Aura removes equip/indestructible while preserving Equipment subtype.
                paid(&mut g, p, "Stop Cold", &[Target::Card(e)]);
                assert!(g.has_subtype(e, "Equipment"));
                assert!(!g.printed_abilities_available(e));
                assert_eq!(g.source_equipment_count(g.object_ref(id)), Some(1));
            }
            assert_eq!(g.life(p.other()), if lost_type { 20 } else { 19 });
        }
    }
}
#[test]
fn msh_source_quantities_whiplash_life_gain_prevention_and_replacement_keep_loss_fixed() {
    for p in [Player::P0, Player::P1] {
        for prevent in [false, true] {
            let mut g = setup(p);
            let (id, _) = armed(&mut g, p);
            named(
                &mut g,
                p,
                if prevent {
                    "Giant Cindermaw"
                } else {
                    "Angel of Vitality"
                },
                Zone::Battlefield,
            );
            named(&mut g, p.other(), "Crystal Barricade", Zone::Battlefield);
            declare(&mut g, p, id);
            settle(&mut g);
            assert_eq!(g.life(p.other()), 19);
            assert_eq!(g.life(p), if prevent { 20 } else { 22 });
        }
    }
}
#[test]
fn msh_source_quantities_whiplash_illegal_equip_and_payment_atomicity() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, WHIP, &[]);
        let e = paid(&mut g, p, "Vibranium Energy Daggers", &[]);
        let enemy = hero(&mut g, p.other());
        g.state.players[p.index()].mana = [0; 6];
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: e,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: e,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(id),
            },
        );
        finish(&mut g);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}
#[test]
fn msh_source_quantities_packets_private_hidden_cards_and_owned_departure_restore() {
    for p in [Player::P0, Player::P1] {
        for name in [WINGS, WHIP] {
            let mut g = setup(p);
            let id = paid(&mut g, p, name, &[]);
            if name == WHIP {
                let e = paid(&mut g, p, "Vibranium Energy Daggers", &[]);
                equip(&mut g, p, e, id);
            }
            let hidden = named(&mut g, p, "Divination", Zone::Hand);
            let h = hero(&mut g, p);
            declare(&mut g, p, id);
            let mut different_secret = g.clone();
            different_secret.state.objects[hidden as usize].card =
                Card::from_name("Murder").unwrap();
            assert_eq!(
                serde_json::to_value(g.observe(p.other())).unwrap(),
                serde_json::to_value(different_secret.observe(p.other())).unwrap()
            );
            let before = g.snapshot();
            let bytes = g.state_bytes();
            let mut child = {
                let parent = g.clone();
                parent.clone()
            };
            if name == WINGS {
                child.effect_counter_by(
                    id,
                    CounterKind::PlusOne,
                    3,
                    p,
                    replacements::ResumeMode::Pending,
                );
            } else {
                let extra = named(&mut child, p, "Vibranium Energy Daggers", Zone::Battlefield);
                assert!(child.attach_equipment(child.object_ref(extra), child.object_ref(id)));
            }
            moved(&mut child, &[id], Zone::Exile);
            let original = packet(&g, id).source_quantity_lki.as_ref().unwrap();
            let orphan = packet(&child, id).source_quantity_lki.as_ref().unwrap();
            if name == WINGS {
                assert_eq!((original.toughness, orphan.toughness), (Some(1), Some(4)));
            } else {
                assert_eq!(
                    (original.equipment_count, orphan.equipment_count),
                    (Some(1), Some(2))
                );
            }
            let departed = child.snapshot();
            assert_eq!(g.snapshot(), before);
            assert!(child.state_bytes() > 0 && bytes > 0);
            assert!(
                !child
                    .observe(p.other())
                    .objects
                    .iter()
                    .any(|o| o.reference.card == hidden)
            );
            assert_eq!(child.observe(p).stack, child.observe(p.other()).stack);
            settle(&mut child);
            let result = child.snapshot();
            child.restore(&departed).unwrap();
            settle(&mut child);
            assert_eq!(child.snapshot(), result);
            if name == WINGS {
                assert_eq!(child.state.objects[h as usize].growth.power, 4);
            } else {
                assert_eq!((child.life(p), child.life(p.other())), (22, 18));
            }
        }
    }
}
#[test]
fn msh_source_quantities_copy_recipes_have_independent_source_facts_and_controllers() {
    for p in [Player::P0, Player::P1] {
        for name in [WINGS, WHIP] {
            let mut g = setup(p);
            let id = paid(&mut g, p, name, &[]);
            let definition = g.object_view(id).card;
            let copy = g.create_copy_groups_inner(
                p.other(),
                &[TokenGroupView {
                    token: definition,
                    amount: 1,
                    tapped: false,
                }],
                &[Some(CopyRecipe::new(definition))],
                None,
            )[0]
            .card;
            g.advance_entries();
            let h = hero(&mut g, p.other());
            if name == WHIP {
                let e = named(&mut g, p, "Vibranium Energy Daggers", Zone::Battlefield);
                assert!(g.attach_equipment(g.object_ref(e), g.object_ref(copy)));
            }
            declare(&mut g, p.other(), copy);
            moved(&mut g, &[copy], Zone::Exile);
            settle(&mut g);
            if name == WINGS {
                assert_eq!(g.state.objects[h as usize].growth.power, 1);
            } else {
                assert_eq!((g.life(p.other()), g.life(p)), (21, 19));
            }
            assert_eq!(g.object_view(id).zone, Zone::Battlefield);
        }
    }
}

#[test]
fn msh_source_quantities_paid_removal_preserves_latest_source_lki() {
    for p in [Player::P0, Player::P1] {
        for name in [WINGS, WHIP] {
            let mut g = setup(p);
            let id = paid(&mut g, p, name, &[]);
            let h = hero(&mut g, p);
            let e = paid(&mut g, p, "Vibranium Energy Daggers", &[]);
            equip(&mut g, p, e, id);
            let old = g.object_ref(id);
            declare(&mut g, p, id);
            paid(&mut g, p, "Murder", &[Target::Card(id)]);
            if name == WINGS {
                assert_eq!(g.state.objects[h as usize].growth.power, 3);
            } else {
                assert_eq!((g.life(p), g.life(p.other())), (21, 19));
            }
            assert_ne!(g.object_ref(id), old);
            assert_eq!(g.object_view(id).zone, Zone::Graveyard);
            assert_eq!(g.object_view(e).attached_to, None);
        }
    }
}
#[test]
fn msh_source_quantities_entering_attacking_does_not_dispatch_declaration_triggers() {
    for p in [Player::P0, Player::P1] {
        for name in [WINGS, WHIP] {
            let mut g = setup(p);
            let id = paid(&mut g, p, name, &[]);
            let h = hero(&mut g, p);
            moved(&mut g, &[id], Zone::Exile);
            g.state.step = Step::Attackers;
            let mut movement = g.entry_move(id, Zone::Battlefield, true);
            movement.attacking = Some(CombatDefender::Player(p.other()));
            g.stage_entries(vec![movement], None);
            g.advance_entries();
            assert!(g.object_view(id).attacking);
            assert!(g.state.pending_triggers.is_empty() && g.state.stack.is_empty());
            assert_eq!(g.state.objects[h as usize].growth, Stats::default());
            assert_eq!((g.life(p), g.life(p.other())), (20, 20));
        }
    }
}
#[test]
fn msh_source_quantities_whiplash_life_events_follow_existing_independent_trigger_paths() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let (id, _) = armed(&mut g, p);
        let observer = paid(&mut g, p, "Tigra, Feline Fury", &[]);
        declare(&mut g, p, id);
        settle(&mut g);
        assert_eq!((g.life(p), g.life(p.other())), (21, 19));
        assert_eq!(g.counter_count(observer, CounterKind::PlusOne), 1);
    }
}
#[test]
fn msh_source_quantities_whiplash_simultaneous_type_provider_departure_keeps_prebatch_count() {
    for p in [Player::P0, Player::P1] {
        for source_first in [false, true] {
            let mut g = setup(p);
            let (id, equipment) = armed(&mut g, p);
            declare(&mut g, p, id);
            let moon = named(&mut g, p, "Imprisoned in the Moon", Zone::Battlefield);
            g.state.objects[moon as usize].attached_to = Some(g.object_ref(equipment));
            assert!(!g.has_subtype(equipment, "Equipment"));
            moved(
                &mut g,
                &if source_first {
                    vec![id, moon, equipment]
                } else {
                    vec![moon, equipment, id]
                },
                Zone::Exile,
            );
            assert_eq!(
                packet(&g, id)
                    .source_quantity_lki
                    .as_ref()
                    .unwrap()
                    .equipment_count,
                Some(0)
            );
            settle(&mut g);
            assert_eq!((g.life(p), g.life(p.other())), (20, 20));
        }
    }
}

#[test]
fn msh_source_quantities_wings_growth_updates_real_and_speculative_power_mana() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let captain = paid(&mut g, p, WINGS, &[]);
        let source = named(&mut g, p, "Topiary Lecturer", Zone::Battlefield);
        types(&mut g, source, &["Hero"]);
        g.state.objects[source as usize].summoning_sick = false;
        g.state.players[p.index()].mana = [0; 6];
        let goal = |amount| FundingGoal {
            actor: p,
            required: [0, 0, 0, 0, amount, 0],
            generic: 0,
            reserve: None,
            source: None,
        };
        assert!(!g.can_fund(goal(2)));
        declare(&mut g, p, captain);
        settle(&mut g);
        assert_eq!(g.stats(source).power, 2);
        let before = g.snapshot();
        assert!(g.can_fund(goal(2)));
        assert!(!g.can_fund(goal(3)));
        assert_eq!(g.snapshot(), before);
        priority(&mut g, p);
        reject(
            &mut g,
            Action::ActivateMana {
                card: source,
                ability: 0,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production: [0, 0, 0, 0, 3, 0],
            },
        );
        checked(
            &mut g,
            Action::ActivateMana {
                card: source,
                ability: 0,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production: [0, 0, 0, 0, 2, 0],
            },
        );
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 0, 2, 0]);
        assert!(g.object_view(source).tapped);
    }
}
