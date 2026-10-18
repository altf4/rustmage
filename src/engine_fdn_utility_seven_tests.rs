use super::tests::{act, scenario, spawn};
use super::*;

fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let card = Card::from_name(name).unwrap();
    assert!(card.is_supported(), "{name}");
    spawn(g, p, card, zone)
}
fn setup(p: Player) -> Game {
    let mut g = scenario();
    g.state.active = p;
    g.state.priority = p;
    for player in &mut g.state.players {
        player.mana = [20; 6];
    }
    g
}
fn replay(g: &mut Game, action: Action) {
    let actor = g.actor().unwrap();
    let revision = g.revision();
    let index = g
        .legal_actions()
        .iter()
        .position(|a| a == &action)
        .expect("legal action");
    let before = g.snapshot();
    let mut child = {
        let parent = g.clone();
        parent.clone()
    };
    child.apply_index(actor, revision, index).unwrap();
    act(g, action.clone());
    let after = g.snapshot();
    assert_eq!(child.snapshot(), after);
    for viewer in [Player::P0, Player::P1] {
        assert_eq!(
            serde_json::to_value(child.observe(viewer)).unwrap(),
            serde_json::to_value(g.observe(viewer)).unwrap()
        );
    }
    g.restore(&before).unwrap();
    assert_eq!(
        g.apply_index(actor, revision, index),
        Err(Error::StaleRevision)
    );
    act(g, action);
    assert_eq!(g.snapshot(), after);
}
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
fn pay(g: &mut Game) {
    for _ in 0..10 {
        if g.state.announcing.is_none() {
            return;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
            .expect("payment/finish choice");
        replay(g, action);
    }
    panic!("announcement stalled");
}
fn announce(g: &mut Game, spell: CardId, targets: &[CardId]) {
    replay(g, Action::AnnounceCast { card: spell });
    for &id in targets {
        replay(
            g,
            Action::ChooseTarget {
                target: Target::Card(id),
            },
        );
    }
    pay(g);
}

fn settle(g: &mut Game) {
    for _ in 0..50 {
        if g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.resolving.is_none()
        {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::Pass | Action::PutTrigger { .. }))
            .unwrap();
        replay(g, a);
    }
    panic!("settle stalled");
}
fn activate(g: &mut Game, card: CardId, targets: &[CardId]) {
    replay(
        g,
        Action::AnnounceActivate {
            card,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    for &id in targets {
        replay(
            g,
            Action::ChooseTarget {
                target: Target::Card(id),
            },
        );
    }
    pay(g);
}
fn hit(g: &mut Game, source: CardId, target: CardId, combat: bool, amount: i128) {
    let e = DamageEvent {
        source: g.damage_source(source),
        target: Target::Card(target),
        amount,
        combat,
    };
    g.apply_damage_batch(&[e]);
}
#[test]
fn fdn_utility_bulk_up_signed_power_paid_and_flashback() {
    for p in [Player::P0, Player::P1] {
        for power in [-2, 0, 5] {
            for flash in [false, true] {
                let mut g = setup(p);
                let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
                g.state.objects[bear as usize].growth.power = power - 2;
                let spell = named(
                    &mut g,
                    p,
                    "Bulk Up",
                    if flash { Zone::Graveyard } else { Zone::Hand },
                );
                g.state.players[p.index()].mana = if flash {
                    [0, 0, 0, 2, 0, 4]
                } else {
                    [0, 0, 0, 1, 0, 1]
                };
                replay(
                    &mut g,
                    if flash {
                        Action::AnnounceFlashback { card: spell }
                    } else {
                        Action::AnnounceCast { card: spell }
                    },
                );
                replay(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(bear),
                    },
                );
                pay(&mut g);
                settle(&mut g);
                assert_eq!(g.stats(bear).power, power * 2);
                assert_eq!(g.stats(bear).toughness, 2);
                assert_eq!(
                    g.object_view(spell).zone,
                    if flash { Zone::Exile } else { Zone::Graveyard }
                );
                assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            }
        }
    }
}
#[test]
fn fdn_utility_flight_counter_flying_prevention_expiry_and_incarnation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let source = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.grant_keyword_effects(&[source], g.object_ref(source), Keywords::LIFELINK);
        let spell = named(&mut g, p, "Fleeting Flight", Zone::Hand);
        let land = named(&mut g, p, "Forest", Zone::Battlefield);
        replay(&mut g, Action::AnnounceCast { card: spell });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(land),
            },
        );
        replay(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.state.objects[bear as usize].counters.plus_one, 1);
        assert!(g.keywords(bear).contains(Keywords::FLYING));
        hit(&mut g, source, bear, true, 20);
        assert_eq!(g.object_view(bear).damage, 0);
        assert_eq!(g.state.players[p.other().index()].life, 20);
        assert!(!g.state.objects[source as usize].dealt_damage_this_turn);
        hit(&mut g, source, bear, false, 1);
        assert_eq!(g.object_view(bear).damage, 1);
        assert_eq!(g.state.players[p.other().index()].life, 21);
        let aura = named(&mut g, p, "Frozen in Ice", Zone::Hand);
        announce(&mut g, aura, &[bear]);
        settle(&mut g);
        assert!(!g.keywords(bear).contains(Keywords::FLYING));
        assert!(g.keywords(bear).contains(Keywords::PREVENT_COMBAT_TO));
        g.move_object(bear, Zone::Exile);
        g.move_object(bear, Zone::Battlefield);
        assert!(!g.keywords(bear).contains(Keywords::PREVENT_COMBAT_TO));
    }
}
#[test]
fn fdn_utility_fog_bank_paid_prevents_both_directions_not_noncombat() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let fog = named(&mut g, p, "Fog Bank", Zone::Hand);
        announce(&mut g, fog, &[]);
        settle(&mut g);
        let bear = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        assert_eq!(
            g.stats(fog),
            Stats {
                power: 0,
                toughness: 2
            }
        );
        assert!(g.keywords(fog).contains(Keywords::DEFENDER));
        assert!(g.keywords(fog).contains(Keywords::FLYING));
        g.state.objects[fog as usize].growth.power = 5;
        hit(&mut g, fog, bear, true, 5);
        hit(&mut g, bear, fog, true, 5);
        assert_eq!(g.object_view(fog).damage, 0);
        assert_eq!(g.object_view(bear).damage, 0);
        hit(&mut g, fog, bear, false, 1);
        hit(&mut g, bear, fog, false, 1);
        assert_eq!(g.object_view(fog).damage, 1);
        assert_eq!(g.object_view(bear).damage, 1);
        let aura = named(&mut g, p, "Frozen in Ice", Zone::Hand);
        announce(&mut g, aura, &[fog]);
        settle(&mut g);
        assert!(!g.keywords(fog).contains(Keywords::PREVENT_COMBAT_TO));
        assert!(!g.keywords(fog).contains(Keywords::PREVENT_COMBAT_BY));
    }
}
#[test]
fn fdn_utility_cane_exiles_as_cost_shuffles_only_own_graveyard_including_empty() {
    for p in [Player::P0, Player::P1] {
        for empty in [false, true] {
            let mut g = setup(p);
            let cane = named(&mut g, p, "Feldon's Cane", Zone::Hand);
            announce(&mut g, cane, &[]);
            settle(&mut g);
            let grave = (!empty).then(|| named(&mut g, p, "Bear Cub", Zone::Graveyard));
            let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Graveyard);
            let old = g.state.players[p.index()].library.len();
            activate(&mut g, cane, &[]);
            assert_eq!(g.object_view(cane).zone, Zone::Exile);
            settle(&mut g);
            assert_eq!(g.object_view(enemy).zone, Zone::Graveyard);
            assert_eq!(
                g.state.players[p.index()].library.len(),
                old + usize::from(!empty)
            );
            if let Some(id) = grave {
                assert_eq!(g.object_view(id).zone, Zone::Library);
                assert!(!g.observe(p.other()).objects.iter().any(|o| o.id == id));
            }
            reject(
                &mut g,
                Action::AnnounceActivate {
                    card: cane,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
        }
    }
}
#[test]
fn fdn_utility_smuggler_haste_another_any_controller_and_resolution_power_check() {
    for p in [Player::P0, Player::P1] {
        for enemy in [false, true] {
            let mut g = setup(p);
            let goblin = named(&mut g, p, "Goblin Smuggler", Zone::Hand);
            announce(&mut g, goblin, &[]);
            settle(&mut g);
            let target = named(
                &mut g,
                if enemy { p.other() } else { p },
                "Bear Cub",
                Zone::Battlefield,
            );
            replay(
                &mut g,
                Action::AnnounceActivate {
                    card: goblin,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(goblin),
                },
            );
            replay(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
            pay(&mut g);
            assert!(g.object_view(goblin).tapped);
            settle(&mut g);
            assert!(g.keywords(target).contains(Keywords::CANT_BE_BLOCKED));
            g.finish_cleanup();
            assert!(!g.keywords(target).contains(Keywords::CANT_BE_BLOCKED));
        }
    }
    let mut g = setup(Player::P0);
    let goblin = named(&mut g, Player::P0, "Goblin Smuggler", Zone::Battlefield);
    let bear = named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    activate(&mut g, goblin, &[bear]);
    g.state.objects[bear as usize].growth.power = 1;
    settle(&mut g);
    assert!(!g.keywords(bear).contains(Keywords::CANT_BE_BLOCKED));
}
#[test]
fn fdn_utility_inspiring_call_creature_count_snapshot_privacy_and_expiry() {
    for p in [Player::P0, Player::P1] {
        for amount in [0, 1, 2] {
            let mut g = setup(p);
            let a = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let b = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let land = named(&mut g, p, "Forest", Zone::Battlefield);
            g.state.objects[a as usize].counters.plus_one = if amount > 0 { 4 } else { 0 };
            g.state.objects[b as usize].counters.plus_one = if amount > 1 { 1 } else { 0 };
            g.state.objects[enemy as usize].counters.plus_one = 1;
            g.state.objects[land as usize].counters.plus_one = 1;
            for _ in 0..3 {
                let id = named(&mut g, p, "Giant Growth", Zone::Library);
                let _ = id;
            }
            let before = g.hand_count(p);
            let spell = named(&mut g, p, "Inspiring Call", Zone::Hand);
            announce(&mut g, spell, &[]);
            settle(&mut g);
            assert_eq!(g.hand_count(p), before + amount);
            assert_eq!(g.keywords(a).contains(Keywords::INDESTRUCTIBLE), amount > 0);
            assert_eq!(g.keywords(b).contains(Keywords::INDESTRUCTIBLE), amount > 1);
            assert!(!g.keywords(enemy).contains(Keywords::INDESTRUCTIBLE));
            assert!(!g.keywords(land).contains(Keywords::INDESTRUCTIBLE));
            assert!(
                !g.observe(p.other()).objects.iter().any(|o| o.owner == p
                    && o.zone == Zone::Hand
                    && o.card.name() == "Giant Growth")
            );
            g.state.objects[b as usize].counters.plus_one = 3;
            assert_eq!(g.keywords(b).contains(Keywords::INDESTRUCTIBLE), amount > 1);
            g.finish_cleanup();
            assert!(!g.keywords(a).contains(Keywords::INDESTRUCTIBLE));
        }
    }
}
#[test]
fn fdn_utility_dictate_flash_both_draw_steps_and_departed_source() {
    for p in [Player::P0, Player::P1] {
        for active in [Player::P0, Player::P1] {
            let mut g = setup(p);
            g.state.active = p.other();
            g.state.step = Step::End;
            let dictate = named(&mut g, p, "Dictate of Kruphix", Zone::Hand);
            announce(&mut g, dictate, &[]);
            settle(&mut g);
            for _ in 0..3 {
                let id = named(&mut g, active, "Giant Growth", Zone::Library);
                let _ = id;
            }
            let before = g.hand_count(active);
            g.state.active = active;
            g.begin_step(Step::Draw);
            g.give_priority();
            g.move_object(dictate, Zone::Graveyard);
            settle(&mut g);
            assert_eq!(g.hand_count(active), before + 1);
            assert!(
                !g.observe(active.other())
                    .objects
                    .iter()
                    .any(|o| o.owner == active && o.zone == Zone::Hand)
            );
        }
    }
}
#[test]
fn fdn_utility_prevention_real_combat_and_expiration() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let attacker = named(&mut g, p, "Vampire Nighthawk", Zone::Battlefield);
        let fog = named(&mut g, p.other(), "Fog Bank", Zone::Battlefield);
        g.state.attackers.push(Attacker {
            defender: Some(CombatDefender::Player(p.other())),
            card: attacker,
            blocked: true,
        });
        g.state.blocks.push((fog, attacker));
        g.state.blocking.push(fog);
        g.state.step = Step::Blockers;
        g.give_priority();
        replay(&mut g, Action::Pass);
        replay(&mut g, Action::Pass);
        assert_eq!(g.state.step, Step::CombatDamage);
        assert_eq!(g.object_view(fog).zone, Zone::Battlefield);
        assert_eq!(g.object_view(fog).damage, 0);
        assert_eq!(g.state.players[p.index()].life, 20);
        let mut g = setup(p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let spell = named(&mut g, p, "Fleeting Flight", Zone::Hand);
        announce(&mut g, spell, &[bear]);
        settle(&mut g);
        assert!(g.keywords(bear).contains(Keywords::PREVENT_COMBAT_TO));
        g.finish_cleanup();
        assert!(!g.keywords(bear).contains(Keywords::PREVENT_COMBAT_TO));
        assert!(!g.keywords(bear).contains(Keywords::FLYING));
        assert_eq!(g.state.objects[bear as usize].counters.plus_one, 1);
    }
}
#[test]
fn fdn_utility_capacity_failure_restores_complete_spell_resolution_and_views() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.state.objects[bear as usize].counters.plus_one = 1;
        let spell = named(&mut g, p, "Inspiring Call", Zone::Hand);
        announce(&mut g, spell, &[]);
        replay(&mut g, Action::Pass);
        g.state.players[p.index()].cards_drawn_this_turn = u64::MAX;
        let before = g.snapshot();
        let revision = g.revision();
        let actor = g.actor().unwrap();
        let views = [
            serde_json::to_value(g.observe(p)).unwrap(),
            serde_json::to_value(g.observe(p.other())).unwrap(),
        ];
        for _ in 0..2 {
            assert_eq!(
                g.apply(actor, Action::Pass),
                Err(Error::CapacityExceeded(crate::CapacityExceeded {
                    resource: "draw ordinal exhausted"
                }))
            );
            assert_eq!(g.snapshot(), before);
            assert_eq!(g.revision(), revision);
            assert_eq!(serde_json::to_value(g.observe(p)).unwrap(), views[0]);
            assert_eq!(
                serde_json::to_value(g.observe(p.other())).unwrap(),
                views[1]
            );
        }
        assert!(!g.keywords(bear).contains(Keywords::INDESTRUCTIBLE));
    }
}
