//! Paid attachment, continuous effects and owned rules interactions.
use super::ability_timeline_tests::{cast, checked, named, settle, setup};
use super::tmt_departure_tests::{candidate, cast_candidate, reject};
use super::*;
fn announce(g: &mut Game, id: CardId, ability: u16, target: Option<CardId>) {
    checked(
        g,
        Action::AnnounceActivate {
            card: id,
            ability,
            counter_payment: Counters::default(),
        },
    );
    if let Some(id) = target {
        checked(
            g,
            Action::ChooseTarget {
                target: Target::Card(id),
            },
        );
    }
}
fn finish(g: &mut Game) {
    for _ in 0..10 {
        if g.state.announcing.is_none() {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. } | Action::FinishTargets))
            .unwrap();
        checked(g, a);
    }
    panic!("unfinished equip");
}
fn equip(g: &mut Game, id: CardId, ability: u16, target: CardId) {
    announce(g, id, ability, Some(target));
    finish(g);
    settle(g);
}
#[test]
fn tmt_equipment_arsenal_live_own_artifacts_includes_itself_and_copy_tokens() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let arsenal = candidate(&mut g, p, "Improvised Arsenal", Zone::Hand);
        cast_candidate(&mut g, arsenal, &[]);
        settle(&mut g);
        equip(&mut g, arsenal, 1, bear);
        assert_eq!(g.stats(bear).power, 3);
        let own = named(&mut g, p, "Three Tree Mascot", Zone::Battlefield);
        named(&mut g, p.other(), "Three Tree Mascot", Zone::Battlefield);
        assert_eq!(g.stats(bear).power, 4);
        announce(&mut g, arsenal, 0, None);
        finish(&mut g);
        settle(&mut g);
        assert_eq!(g.stats(bear).power, 5);
        let token = g
            .state
            .copy_recipes
            .as_ref()
            .unwrap()
            .iter()
            .find(|r| r.recipe.definition == Card::from_name("Improvised Arsenal").unwrap())
            .unwrap()
            .object
            .card;
        assert!(g.state.objects[token as usize].is_token());
        assert_eq!(g.state.objects[token as usize].attached_to, None);
        equip(&mut g, token, 1, bear);
        assert_eq!(g.stats(bear).power, 8);
        g.move_object(own, Zone::Graveyard);
        assert_eq!(g.stats(bear).power, 6);
    }
}

fn until(g: &mut Game, done: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..80 {
        if done(g) {
            return;
        }
        let actions = g.legal_actions();
        let a = if g.state.pending == Pending::Priority {
            Action::Pass
        } else {
            actions
                .into_iter()
                .find(|a| {
                    matches!(
                        a,
                        Action::PutTrigger { .. } | Action::PayCosts { .. } | Action::FinishTargets
                    )
                })
                .unwrap_or_else(|| panic!("continuation {:?}", g.state.pending))
        };
        checked(g, a);
    }
    panic!("unreached {:?}", g.state.pending);
}
fn etb_target(g: &mut Game, name: &str, target: CardId) -> CardId {
    let p = g.actor().unwrap();
    let id = candidate(g, p, name, Zone::Hand);
    cast_candidate(g, id, &[]);
    until(g, |g| matches!(g.state.pending, Pending::Targets { .. }));
    checked(
        g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    finish(g);
    id
}
fn choose_sac(g: &mut Game, id: CardId) {
    let object = g.object_ref(id);
    checked(g, Action::ChooseAdditionalCost { object });
}
#[test]
fn tmt_equipment_arsenal_copy_uses_departed_source_lki_and_is_unattached() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let arsenal = candidate(&mut g, p, "Improvised Arsenal", Zone::Hand);
        cast_candidate(&mut g, arsenal, &[]);
        settle(&mut g);
        announce(&mut g, arsenal, 0, None);
        finish(&mut g);
        g.move_object(arsenal, Zone::Hand);
        settle(&mut g);
        let row = g.state.copy_recipes.as_ref().unwrap().last().unwrap();
        assert_eq!(
            row.recipe.definition,
            Card::from_name("Improvised Arsenal").unwrap()
        );
        assert!(g.on_battlefield(row.object.card));
        assert_eq!(g.state.objects[row.object.card as usize].attached_to, None);
    }
}
#[test]
fn tmt_equipment_equip_timing_foreign_noncreature_and_stale_incarnation() {
    for (name, ability) in [
        ("Improvised Arsenal", 1),
        ("Novel Nunchaku", 0),
        ("Quintessential Katana", 0),
    ] {
        let p = Player::P0;
        let mut g = setup(p);
        let e = candidate(&mut g, p, name, Zone::Hand);
        cast_candidate(&mut g, e, &[]);
        settle(&mut g);
        let bear = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        let land = named(&mut g, p, "Forest", Zone::Battlefield);
        g.state.step = Step::End;
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: e,
                ability,
                counter_payment: Counters::default(),
            },
        );
        g.state.step = Step::Main1;
        announce(&mut g, e, ability, None);
        for id in [enemy, land] {
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(id),
                },
            );
        }
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        finish(&mut g);
        g.move_object(bear, Zone::Hand);
        g.move_object(bear, Zone::Battlefield);
        settle(&mut g);
        assert_eq!(g.state.objects[e as usize].attached_to, None);
    }
}
#[test]
fn tmt_equipment_armor_paid_entry_nonland_sacrifice_and_once_per_turn() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let other = named(&mut g, p, "Hill Giant", Zone::Battlefield);
        let armor = etb_target(&mut g, "Shredder's Armor", bear);
        settle(&mut g);
        assert_eq!((g.stats(bear).power, g.stats(bear).toughness), (4, 3));
        let token = super::tmt_departure_tests::food(&mut g, p);
        let land = named(&mut g, p, "Forest", Zone::Battlefield);
        let enemy = super::tmt_departure_tests::food(&mut g, p.other());
        announce(&mut g, armor, 0, Some(other));
        for id in [armor, land, enemy] {
            let object = g.object_ref(id);
            reject(&mut g, Action::ChooseAdditionalCost { object });
        }
        choose_sac(&mut g, token);
        finish(&mut g);
        assert!(!g.on_battlefield(token));
        settle(&mut g);
        assert_eq!(
            g.state.objects[armor as usize].attached_to,
            Some(g.object_ref(other))
        );
        assert_eq!(g.stats(bear).power, 2);
        assert_eq!(g.stats(other).power, 5);
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: armor,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        super::ability_timeline_tests::next_turn(&mut g);
        super::ability_timeline_tests::next_turn(&mut g);
        until(&mut g, |g| g.state.step == Step::Main1);
        announce(&mut g, armor, 0, Some(bear));
        choose_sac(&mut g, other);
        finish(&mut g);
        settle(&mut g);
        assert_eq!(
            g.state.objects[armor as usize].attached_to,
            Some(g.object_ref(bear))
        );
    }
}
#[test]
fn tmt_equipment_armor_sacrificed_target_fizzles_without_refund_or_limit_reset() {
    let p = Player::P0;
    let mut g = setup(p);
    let bear = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let armor = etb_target(&mut g, "Shredder's Armor", bear);
    settle(&mut g);
    announce(&mut g, armor, 0, Some(bear));
    choose_sac(&mut g, bear);
    // The selected target leaves as the sacrifice cost commits.
    finish(&mut g);
    settle(&mut g);
    assert_eq!(g.state.objects[bear as usize].zone, Zone::Graveyard);
    assert_eq!(g.state.objects[armor as usize].attached_to, None);
    named(&mut g, p, "Hill Giant", Zone::Battlefield);
    reject(
        &mut g,
        Action::AnnounceActivate {
            card: armor,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
}
#[test]
fn tmt_equipment_nunchaku_reflexive_fight_is_separate_and_damage_simultaneous() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let e = etb_target(&mut g, "Novel Nunchaku", bear);
        assert!(
            g.state
                .stack
                .iter()
                .all(|s| !s.targets.iter().any(|t| t.target == Target::Card(enemy)))
        );
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::Targets { .. })
        });
        assert_eq!(
            g.state.objects[e as usize].attached_to,
            Some(g.object_ref(bear))
        );
        assert!(g.keywords(bear).contains(Keywords::TRAMPLE));
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        finish(&mut g);
        assert!(g.on_battlefield(bear) && g.on_battlefield(enemy));
        settle(&mut g);
        assert_eq!(g.state.objects[bear as usize].zone, Zone::Graveyard);
        assert_eq!(g.state.objects[enemy as usize].zone, Zone::Graveyard);
    }
}
#[test]
fn tmt_equipment_nunchaku_can_choose_zero_and_equip_does_not_repeat_fight() {
    let p = Player::P0;
    let mut g = setup(p);
    let bear = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let enemy = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
    let e = etb_target(&mut g, "Novel Nunchaku", bear);
    until(&mut g, |g| {
        matches!(g.state.pending, Pending::Targets { .. })
    });
    checked(&mut g, Action::FinishTargets);
    settle(&mut g);
    assert_eq!(g.state.objects[bear as usize].damage, 0);
    assert_eq!(g.state.objects[enemy as usize].damage, 0);
    equip(&mut g, e, 0, bear);
    assert_eq!(g.state.objects[enemy as usize].damage, 0);
}
#[test]
fn tmt_equipment_nunchaku_failed_attachment_has_no_reflexive_and_fight_uses_current_equipped_creature()
 {
    for failed_source in [false, true] {
        let p = Player::P0;
        let mut g = setup(p);
        let bear = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let e = etb_target(&mut g, "Novel Nunchaku", bear);
        g.move_object(if failed_source { e } else { bear }, Zone::Hand);
        settle(&mut g);
        assert!(g.state.stack.is_empty());
        assert!(g.state.pending_triggers.is_empty());
    }
    let p = Player::P0;
    let mut g = setup(p);
    let bear = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let giant = named(&mut g, p, "Hill Giant", Zone::Battlefield);
    let enemy = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
    let e = etb_target(&mut g, "Novel Nunchaku", bear);
    until(&mut g, |g| {
        matches!(g.state.pending, Pending::Targets { .. })
    });
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(enemy),
        },
    );
    finish(&mut g);
    // An independent attachment effect in the response window changes "equipped creature".
    assert!(g.attach_equipment(g.object_ref(e), g.object_ref(giant)));
    settle(&mut g);
    assert_eq!(g.state.objects[bear as usize].damage, 0);
    assert_eq!(g.state.objects[giant as usize].damage, 2);
    assert_eq!(g.state.objects[enemy as usize].zone, Zone::Graveyard);
}
#[test]
fn tmt_equipment_nunchaku_departed_equipment_lki_fights_without_bonus_and_illegal_enemy_fizzles() {
    for remove_enemy in [false, true] {
        let p = Player::P0;
        let mut g = setup(p);
        let giant = named(&mut g, p, "Hill Giant", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        let e = etb_target(&mut g, "Novel Nunchaku", giant);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::Targets { .. })
        });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        finish(&mut g);
        g.move_object(if remove_enemy { enemy } else { e }, Zone::Hand);
        settle(&mut g);
        assert_eq!(
            g.state.objects[giant as usize].damage,
            if remove_enemy { 0 } else { 2 }
        );
        if !remove_enemy {
            assert_eq!(g.state.objects[enemy as usize].zone, Zone::Graveyard);
        }
    }
}
#[test]
fn tmt_equipment_katana_actual_combat_damage_untaps_and_gains_two() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let e = candidate(&mut g, p, "Quintessential Katana", Zone::Hand);
        cast_candidate(&mut g, e, &[]);
        settle(&mut g);
        equip(&mut g, e, 0, bear);
        super::tmt_sneak_tests::unblocked(&mut g, bear);
        assert!(g.state.objects[bear as usize].tapped);
        let before = g.state.players[p.index()].life;
        until(&mut g, |g| g.state.players[p.index()].life > before);
        assert_eq!(g.state.players[p.index()].life, before + 2);
        assert!(!g.state.objects[bear as usize].tapped);
        settle(&mut g);
    }
}
fn dealt(g: &mut Game, dealer: CardId, targets: &[CardId], combat: bool, amount: i128) {
    let source = g.damage_source(dealer);
    let events: Vec<_> = targets
        .iter()
        .map(|id| DamageEvent {
            preventable: true,
            source,
            target: Target::Card(*id),
            amount,
            combat,
        })
        .collect();
    g.apply_damage_batch(&events);
    g.state_based_actions();
    g.prepare_triggers();
}
#[test]
fn tmt_equipment_katana_one_trigger_per_simultaneous_batch_and_independent_grants() {
    for count in [1, 2] {
        let p = Player::P0;
        let mut g = setup(p);
        let bear = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let first = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let second = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        for _ in 0..count {
            let e = candidate(&mut g, p, "Quintessential Katana", Zone::Hand);
            cast_candidate(&mut g, e, &[]);
            settle(&mut g);
            equip(&mut g, e, 0, bear);
        }
        let before = g.state.players[p.index()].life;
        dealt(&mut g, bear, &[first, second], true, 1);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, before + 2 * count);
        dealt(&mut g, bear, &[first], false, 1);
        settle(&mut g);
        dealt(&mut g, bear, &[second], true, 0);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, before + 2 * count);
    }
}
#[test]
fn tmt_equipment_katana_granted_controller_is_recipient_and_trigger_survives_departure() {
    for depart in [false, true] {
        let p = Player::P0;
        let mut g = setup(p);
        let bear = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let e = candidate(&mut g, p, "Quintessential Katana", Zone::Hand);
        cast_candidate(&mut g, e, &[]);
        settle(&mut g);
        equip(&mut g, e, 0, bear);
        if !depart {
            let theft = named(&mut g, p.other(), "Confiscate", Zone::Hand);
            g.state.priority = p.other();
            g.state.active = p.other();
            cast(&mut g, theft, None, &[Target::Card(bear)]);
            settle(&mut g);
        }
        let controller = g.state.objects[bear as usize].controller;
        let before = g.state.players[controller.index()].life;
        dealt(&mut g, bear, &[enemy], true, 1);
        if depart {
            g.move_object(e, Zone::Hand);
            g.move_object(bear, Zone::Hand);
            g.move_object(bear, Zone::Battlefield);
        }
        settle(&mut g);
        assert_eq!(g.state.players[controller.index()].life, before + 2);
    }
}
#[test]
fn tmt_equipment_katana_optional_ninja_entry_is_untargeted_and_exact_incarnation() {
    for p in [Player::P0, Player::P1] {
        for accept in [false, true] {
            let mut g = setup(p);
            let e = candidate(&mut g, p, "Quintessential Katana", Zone::Hand);
            cast_candidate(&mut g, e, &[]);
            settle(&mut g);
            let ninja = named(&mut g, p, "Foot Ninjas", Zone::Hand);
            cast_candidate(&mut g, ninja, &[]);
            until(&mut g, |g| {
                g.legal_actions()
                    .contains(&Action::ChooseOptional { accept })
            });
            checked(&mut g, Action::ChooseOptional { accept });
            settle(&mut g);
            assert_eq!(
                g.state.objects[e as usize].attached_to,
                if accept {
                    Some(g.object_ref(ninja))
                } else {
                    None
                }
            );
            let other = named(&mut g, p, "Foot Ninjas", Zone::Hand);
            cast_candidate(&mut g, other, &[]);
            until(&mut g, |g| {
                g.legal_actions()
                    .contains(&Action::ChooseOptional { accept: true })
            });
            g.move_object(other, Zone::Hand);
            g.move_object(other, Zone::Battlefield);
            checked(&mut g, Action::ChooseOptional { accept: true });
            assert_ne!(
                g.state.objects[e as usize].attached_to,
                Some(g.object_ref(other))
            );
            // Re-entry legitimately created a separate new Ninja trigger. Decline it.
            until(&mut g, |g| {
                g.legal_actions()
                    .contains(&Action::ChooseOptional { accept: false })
            });
            checked(&mut g, Action::ChooseOptional { accept: false });
            settle(&mut g);
            assert_ne!(
                g.state.objects[e as usize].attached_to,
                Some(g.object_ref(other))
            );
        }
    }
}

#[test]
fn tmt_equipment_nunchaku_already_attached_does_not_count_as_new_attachment() {
    let p = Player::P0;
    let mut g = setup(p);
    let bear = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let e = etb_target(&mut g, "Novel Nunchaku", bear);
    assert!(g.attach_equipment(g.object_ref(e), g.object_ref(bear)));
    settle(&mut g);
    assert_eq!(
        g.state.objects[e as usize].attached_to,
        Some(g.object_ref(bear))
    );
    assert!(g.state.stack.is_empty());
    assert_eq!(g.state.objects[bear as usize].damage, 0);
}
#[test]
fn tmt_equipment_katana_granted_ability_removed_before_combat_and_recipient_lethal_batch() {
    for lose_ability in [false, true] {
        let p = Player::P0;
        let mut g = setup(p);
        let bear = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let e = candidate(&mut g, p, "Quintessential Katana", Zone::Hand);
        cast_candidate(&mut g, e, &[]);
        settle(&mut g);
        equip(&mut g, e, 0, bear);
        if lose_ability {
            let aura = named(&mut g, p, "Witness Protection", Zone::Hand);
            cast(&mut g, aura, None, &[Target::Card(bear)]);
            settle(&mut g);
        }
        let before = g.state.players[p.index()].life;
        let dealer = g.damage_source(bear);
        let blocker = g.damage_source(enemy);
        g.apply_damage_batch(&[
            DamageEvent {
                preventable: true,
                source: dealer,
                target: Target::Card(enemy),
                amount: 1,
                combat: true,
            },
            DamageEvent {
                preventable: true,
                source: blocker,
                target: Target::Card(bear),
                amount: 8,
                combat: true,
            },
        ]);
        g.state_based_actions();
        g.prepare_triggers();
        settle(&mut g);
        assert_eq!(g.state.objects[bear as usize].zone, Zone::Graveyard);
        assert_eq!(
            g.state.players[p.index()].life,
            before + if lose_ability { 0 } else { 2 }
        );
    }
}
