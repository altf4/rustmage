//! Gated attachment core fixtures: real immutable definitions and paid actions.
//! Paid complete printed-clause fixtures for both attachment consumers.
use super::*;
use crate::engine::assurance::big_helpers::{cast, checked, named, next_turn, settle, setup};
use crate::engine::msh_thirteenth_combined_tests::owned_checked_step;
const SWORD: &str = "Swordsman, Sharp Scoundrel";
const WINTER: &str = "Winter Soldier, Icy Assassin";
fn seed(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let card = Card::from_name(name).unwrap();
    if card.is_supported() {
        return named(g, p, name, zone);
    }
    assert!([SWORD, WINTER].contains(&name));
    let id = CardId::try_from(g.state.objects.len()).unwrap();
    g.state.objects.push(Object {
        card,
        copy_kind: CopyKind::None,
        owner: p,
        controller: p,
        zone,
        generation: 0,
        tapped: false,
        summoning_sick: false,
        flashback: false,
        damage: 0,
        damaged_this_turn: false,
        dealt_damage_this_turn: false,
        deathtouch_since_sba: false,
        growth: Stats::default(),
        gained_keywords: Keywords::NONE,
        counters: InitialCounters::default(),
        used_abilities: 0,
        attached_to: None,
    });
    match zone {
        Zone::Battlefield => g.register_battlefield_sources(id),
        Zone::Library => g.state.players[p.index()].library.push(id),
        Zone::Graveyard => g.register_graveyard_sources(id),
        _ => (),
    }
    id
}
fn priority(g: &mut Game, p: Player) {
    if g.actor() != Some(p) {
        checked(g, Action::Pass);
    }
    assert_eq!(g.actor(), Some(p));
}
fn packet(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    priority(g, p);
    let id = seed(g, p, name, Zone::Hand);
    let (mut mana, generic) = Card::from_name(name).unwrap().cost();
    mana[5] += generic;
    g.state.players[p.index()].mana = mana;
    cast(g, id, None, targets);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    id
}
fn paid(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    let id = packet(g, p, name, targets);
    settle(g);
    id
}
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..240 {
        if stop(g) {
            return;
        }
        let a = g
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
                        | Action::ChooseMode { mode: 0 }
                )
            })
            .unwrap_or_else(|| panic!("unexpected {:?}", g.state.pending));
        checked(g, a);
    }
    panic!("stalled {:?}", g.state.pending);
}
fn finish(g: &mut Game) {
    for _ in 0..12 {
        if g.state.announcing.is_none() {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
            .unwrap();
        checked(g, a);
    }
    panic!("unfinished");
}
fn reject(g: &mut Game, a: Action) {
    let s = g.snapshot();
    let r = g.revision();
    let seen = [Player::P0, Player::P1].map(|p| serde_json::to_value(g.observe(p)).unwrap());
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), s);
    assert_eq!(g.revision(), r);
    assert_eq!(
        [Player::P0, Player::P1].map(|p| serde_json::to_value(g.observe(p)).unwrap()),
        seen
    );
}
fn orphan(g: Game, a: Action) -> Game {
    let p = g.actor().unwrap();
    let i = g.legal_actions().iter().position(|v| *v == a).unwrap();
    owned_checked_step(g, p, i, a)
}
fn entry_targets(g: &mut Game, sword: CardId) {
    until(g, |g| {
        matches!(g.state.pending, Pending::Targets { .. })
            && g.state
                .announcing
                .as_ref()
                .is_some_and(|a| a.item.card == sword)
    });
}
fn next_main(g: &mut Game, p: Player) {
    next_turn(g);
    next_turn(g);
    until(g, |g| {
        g.state.active == p && g.state.step == Step::Main1 && g.state.pending == Pending::Priority
    });
    priority(g, p);
}
fn equip(g: &mut Game, p: Player, pole: CardId, bear: CardId) {
    priority(g, p);
    g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
    checked(
        g,
        Action::AnnounceActivate {
            card: pole,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    checked(
        g,
        Action::ChooseTarget {
            target: Target::Card(bear),
        },
    );
    finish(g);
    settle(g);
}
#[test]
fn msh_attachment_core_paid_swordsman_optional_equipment_mandatory_creature_and_real_move() {
    for p in [Player::P0, Player::P1] {
        for choose in [false, true] {
            let mut g = setup(p);
            let sword = paid(&mut g, p, SWORD, &[]);
            let pole = paid(&mut g, p, "Fishing Pole", &[]);
            let bear = paid(&mut g, p, "Bear Cub", &[]);
            let foreign = named(&mut g, p.other(), "Fishing Pole", Zone::Battlefield);
            packet(&mut g, p, "Mister Hyde, Monster Within", &[]);
            entry_targets(&mut g, sword);
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(bear),
                },
            );
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(foreign),
                },
            );
            g = orphan(
                g,
                if choose {
                    Action::ChooseTarget {
                        target: Target::Card(pole),
                    }
                } else {
                    Action::FinishTargets
                },
            );
            reject(&mut g, Action::FinishTargets);
            g = orphan(
                g,
                Action::ChooseTarget {
                    target: Target::Card(bear),
                },
            );
            finish(&mut g);
            settle(&mut g);
            assert_eq!(
                g.state.objects[pole as usize].attached_to,
                choose.then(|| g.object_ref(bear))
            );
            assert_eq!(g.object_view(pole).controller, p);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        }
    }
}
#[test]
fn msh_attachment_core_paid_swordsman_invalidated_target_preserves_old_attachment_and_source_independence()
 {
    for p in [Player::P0, Player::P1] {
        for departure in 0..3 {
            let mut g = setup(p);
            let sword = paid(&mut g, p, SWORD, &[]);
            let pole = paid(&mut g, p, "Fishing Pole", &[]);
            let old = paid(&mut g, p, "Bear Cub", &[]);
            let new = named(&mut g, p, "Hill Giant", Zone::Battlefield);
            equip(&mut g, p, pole, old);
            packet(&mut g, p, "Mister Hyde, Monster Within", &[]);
            entry_targets(&mut g, sword);
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(pole),
                },
            );
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(new),
                },
            );
            finish(&mut g);
            let removed = match departure {
                0 => new,
                1 => pole,
                _ => sword,
            };
            paid(
                &mut g,
                p,
                if departure == 1 {
                    "Disenchant"
                } else {
                    "Unsummon"
                },
                &[Target::Card(removed)],
            );
            settle(&mut g);
            assert_eq!(
                g.state.objects[pole as usize].attached_to,
                match departure {
                    0 => Some(g.object_ref(old)),
                    1 => None,
                    _ => Some(g.object_ref(new)),
                }
            );
        }
    }
}
#[test]
fn msh_attachment_core_paid_swordsman_declared_attack_connives_once_with_two_equipment() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let sword = paid(&mut g, p, SWORD, &[]);
        let a = paid(&mut g, p, "Fishing Pole", &[]);
        let b = paid(&mut g, p, "Fishing Pole", &[]);
        let bear = paid(&mut g, p, "Bear Cub", &[]);
        equip(&mut g, p, a, bear);
        equip(&mut g, p, b, bear);
        next_main(&mut g, p);
        let discard = named(&mut g, p, "Shock", Zone::Hand);
        let n = g.state.players[p.index()].library.len();
        until(&mut g, |g| g.state.pending == Pending::Attackers);
        checked(&mut g, Action::Attack { card: bear });
        checked(&mut g, Action::FinishAttackers);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectDiscard { .. })
        });
        assert_eq!(g.state.players[p.index()].library.len(), n - 1);
        assert_eq!(g.state.resolving.as_ref().unwrap().item.card, sword);
        assert!(g.state.resolving.as_ref().unwrap().item.targets.is_empty());
        g = orphan(g, Action::Discard { card: discard });
        settle(&mut g);
        assert_eq!(g.counter_count(bear, CounterKind::PlusOne), 1);
        assert_eq!(g.state.players[p.index()].library.len(), n - 1);
    }
}
#[test]
fn msh_attachment_core_paid_swordsman_queued_attacker_current_or_departed_controller_connives() {
    for p in [Player::P0, Player::P1] {
        for leave in [false, true] {
            let mut g = setup(p);
            let sword = paid(&mut g, p, SWORD, &[]);
            let pole = paid(&mut g, p, "Fishing Pole", &[]);
            let bear = paid(&mut g, p, "Bear Cub", &[]);
            equip(&mut g, p, pole, bear);
            next_main(&mut g, p);
            until(&mut g, |g| g.state.pending == Pending::Attackers);
            checked(&mut g, Action::Attack { card: bear });
            checked(&mut g, Action::FinishAttackers);
            until(&mut g, |g| {
                g.state.stack.iter().any(|s| {
                    s.card == sword && matches!(s.kind, StackKind::TriggeredAbility { .. })
                })
            });
            let old = g.object_ref(bear);
            g.change_controller(old, p.other());
            if leave {
                packet(&mut g, p, "Unsummon", &[Target::Card(bear)]);
            }
            let discard = named(&mut g, p.other(), "Shock", Zone::Hand);
            let n = g.state.players[p.other().index()].library.len();
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::EffectDiscard { .. })
            });
            assert_eq!(g.actor(), Some(p.other()));
            assert_eq!(g.state.players[p.other().index()].library.len(), n - 1);
            g = orphan(g, Action::Discard { card: discard });
            settle(&mut g);
            assert_eq!(
                g.counter_count(bear, CounterKind::PlusOne),
                u64::from(!leave)
            );
            assert_eq!(
                g.object_view(bear).zone,
                if leave { Zone::Hand } else { Zone::Battlefield }
            );
        }
    }
}
#[test]
fn msh_attachment_core_paid_winter_live_equipment_count_same_in_virtual_world() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let winter = paid(&mut g, p, WINTER, &[]);
        let a = paid(&mut g, p, "Fishing Pole", &[]);
        let b = paid(&mut g, p, "Fishing Pole", &[]);
        assert_eq!(g.stats(winter).power, 2);
        equip(&mut g, p, a, winter);
        equip(&mut g, p, b, winter);
        let base = Card::from_name("Fishing Pole")
            .unwrap()
            .equipment()
            .unwrap()
            .stats
            .power;
        assert_eq!(g.stats(winter).power, 2 + 4 + base * 2);
        let goal = FundingGoal {
            actor: p,
            required: [0; 6],
            generic: 0,
            source: None,
            reserve: None,
        };
        let mut w = FundingState::from_game(&g, &goal);
        assert_eq!(w.value(StatValue::SourceEquipmentCount, winter), 2);
        assert_eq!(w.power(winter), g.stats(winter).power);
        let i = w.resources.iter().position(|r| r.id == a).unwrap();
        w.resources[i].object.zone = Zone::Graveyard;
        assert_eq!(w.value(StatValue::SourceEquipmentCount, winter), 1);
        assert_eq!(w.power(winter), 2 + 2 + base);
        assert_eq!(g.stats(winter).power, 2 + 4 + base * 2);
    }
}

fn activate_return(g: &mut Game, p: Player, winter: CardId) {
    priority(g, p);
    g.state.players[p.index()].mana = [1, 0, 1, 0, 0, 3];
    checked(
        g,
        Action::AnnounceActivate {
            card: winter,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    finish(g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
}
#[test]
fn msh_attachment_core_paid_winter_return_finality_and_public_optional_nontargeting_attachment() {
    for p in [Player::P0, Player::P1] {
        for accept in [false, true] {
            let mut g = setup(p);
            let pole = paid(&mut g, p, "Fishing Pole", &[]);
            let old_bear = paid(&mut g, p, "Bear Cub", &[]);
            equip(&mut g, p, pole, old_bear);
            let winter = seed(&mut g, p, WINTER, Zone::Graveyard);
            let foreign = named(&mut g, p.other(), "Fishing Pole", Zone::Battlefield);
            let old = g.object_ref(winter);
            activate_return(&mut g, p, winter);
            let ability = g.state.stack.last().unwrap().clone();
            assert_eq!(ability.object, old);
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::EquipmentAttachment { .. })
            });
            assert_eq!(g.actor(), Some(p));
            assert_ne!(g.object_ref(winter), old);
            assert_eq!(g.counter_count(winter, CounterKind::Finality), 1);
            assert_eq!(g.state.resolving.as_ref().unwrap().item.object, old);
            assert_eq!(
                g.state.resolving.as_ref().unwrap().item.source_facts,
                ability.source_facts
            );
            let a = g.observe(p).equipment_attachment.unwrap();
            let b = g.observe(p.other()).equipment_attachment.unwrap();
            assert_eq!(
                serde_json::to_value(&a).unwrap(),
                serde_json::to_value(&b).unwrap()
            );
            assert_eq!(a.recipient.reference, g.object_ref(winter));
            assert_eq!(a.candidates, vec![g.object_ref(pole)]);
            assert!(g.state.resolving.as_ref().unwrap().item.targets.is_empty());
            for bad in [g.object_ref(foreign), g.object_ref(old_bear), old] {
                reject(
                    &mut g,
                    Action::ChooseEquipmentAttachment {
                        equipment: Some(bad),
                    },
                );
            }
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(pole),
                },
            );
            let equipment = accept.then(|| g.object_ref(pole));
            g = orphan(g, Action::ChooseEquipmentAttachment { equipment });
            settle(&mut g);
            assert_eq!(
                g.state.objects[pole as usize].attached_to,
                Some(g.object_ref(if accept { winter } else { old_bear }))
            );
            assert!(g.observe(p).equipment_attachment.is_none());
            assert!(g.state.resolving.is_none());
            paid(&mut g, p, "Murder", &[Target::Card(winter)]);
            assert_eq!(g.object_view(winter).zone, Zone::Exile);
            assert_eq!(g.counter_count(winter, CounterKind::Finality), 0);
            assert!(
                !g.legal_actions()
                    .iter()
                    .any(|a| matches!(a, Action::AnnounceActivate { card, .. } if *card==winter))
            );
        }
    }
}
#[test]
fn msh_attachment_core_paid_winter_two_queued_activations_only_successful_return_attaches() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let pole = paid(&mut g, p, "Fishing Pole", &[]);
        let winter = seed(&mut g, p, WINTER, Zone::Graveyard);
        let old = g.object_ref(winter);
        activate_return(&mut g, p, winter);
        activate_return(&mut g, p, winter);
        assert_eq!(g.state.stack.len(), 2);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EquipmentAttachment { .. })
        });
        let returned = g.object_ref(winter);
        assert_ne!(returned, old);
        g = orphan(g, Action::ChooseEquipmentAttachment { equipment: None });
        assert!(g.state.stack.iter().any(|s| s.object == old));
        until(&mut g, |g| {
            g.state.stack.is_empty()
                && g.state.resolving.is_none()
                && g.state.pending == Pending::Priority
        });
        assert_eq!(g.object_ref(winter), returned);
        assert_eq!(g.counter_count(winter, CounterKind::Finality), 1);
        assert_eq!(g.state.objects[pole as usize].attached_to, None);
    }
}
#[test]
fn msh_attachment_core_paid_winter_old_graveyard_incarnation_cannot_return_or_attach_new_one() {
    for p in [Player::P0, Player::P1] {
        for back_to_graveyard in [false, true] {
            let mut g = setup(p);
            let pole = paid(&mut g, p, "Fishing Pole", &[]);
            let winter = seed(&mut g, p, WINTER, Zone::Graveyard);
            let old = g.object_ref(winter);
            activate_return(&mut g, p, winter);
            packet(&mut g, p, "It Doesn't Add Up", &[Target::Card(winter)]);
            until(&mut g, |g| {
                g.on_battlefield(winter) && g.state.pending == Pending::Priority
            });
            assert_ne!(g.object_ref(winter), old);
            assert_eq!(g.counter_count(winter, CounterKind::Finality), 0);
            if back_to_graveyard {
                packet(&mut g, p, "Murder", &[Target::Card(winter)]);
                until(&mut g, |g| {
                    g.object_view(winter).zone == Zone::Graveyard
                        && g.state.pending == Pending::Priority
                });
            }
            let current = g.object_ref(winter);
            until(&mut g, |g| {
                g.state.stack.is_empty()
                    && g.state.resolving.is_none()
                    && g.state.pending == Pending::Priority
            });
            assert_eq!(g.object_ref(winter), current);
            assert_eq!(
                g.object_view(winter).zone,
                if back_to_graveyard {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
            assert_eq!(g.state.objects[pole as usize].attached_to, None);
            assert_eq!(g.counter_count(winter, CounterKind::Finality), 0);
        }
    }
}
fn two_seasons(g: &mut Game, p: Player) {
    paid(g, p, "Doubling Season", &[]);
    paid(g, p, "Doubling Season", &[]);
}
fn finish_replacements(g: &mut Game) {
    while matches!(g.state.pending, Pending::Replacement { .. }) {
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::ChooseReplacement { .. }))
            .unwrap();
        checked(g, a);
    }
}
#[test]
fn msh_attachment_core_paid_winter_counter_entry_pause_precedes_public_attachment_and_owned_resume()
{
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        two_seasons(&mut g, p);
        let pole = paid(&mut g, p, "Fishing Pole", &[]);
        let winter = seed(&mut g, p, WINTER, Zone::Graveyard);
        activate_return(&mut g, p, winter);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::Replacement { .. })
        });
        assert_eq!(g.object_view(winter).zone, Zone::Graveyard);
        assert!(g.observe(p).equipment_attachment.is_none());
        let choice = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::ChooseReplacement { .. }))
            .unwrap();
        g = orphan(g, choice);
        finish_replacements(&mut g);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EquipmentAttachment { .. })
        });
        assert_eq!(g.counter_count(winter, CounterKind::Finality), 4);
        let equipment = Some(g.object_ref(pole));
        g = orphan(g, Action::ChooseEquipmentAttachment { equipment });
        settle(&mut g);
        assert_eq!(
            g.state.objects[pole as usize].attached_to,
            Some(g.object_ref(winter))
        );
    }
}
#[test]
fn msh_attachment_core_winter_actual_entry_result_skipped_or_redirected_has_no_attachment_recipient()
 {
    for p in [Player::P0, Player::P1] {
        for skipped in [false, true] {
            let mut g = setup(p);
            two_seasons(&mut g, p);
            let pole = paid(&mut g, p, "Fishing Pole", &[]);
            let winter = seed(&mut g, p, WINTER, Zone::Graveyard);
            activate_return(&mut g, p, winter);
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::Replacement { .. })
            });
            // Bounded entry-transaction boundary: no claim that an admitted card
            // currently produces an entry prohibition or this redirection.
            let movement = &mut g.state.entry_batch.as_mut().unwrap().moves[0];
            assert!(movement.equipment_after);
            if skipped {
                movement.skipped = true;
            } else {
                movement.destination = Zone::Exile;
            }
            finish_replacements(&mut g);
            until(&mut g, |g| {
                g.state.stack.is_empty()
                    && g.state.resolving.is_none()
                    && g.state.pending == Pending::Priority
            });
            assert_eq!(
                g.object_view(winter).zone,
                if skipped {
                    Zone::Graveyard
                } else {
                    Zone::Exile
                }
            );
            assert_eq!(g.state.objects[pole as usize].attached_to, None);
            assert_eq!(g.counter_count(winter, CounterKind::Finality), 0);
        }
    }
}
#[test]
fn msh_attachment_card_winter_paid_vigilance_and_menace_single_block_illegal() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let winter = paid(&mut g, p, WINTER, &[]);
        let a = named(&mut g, p.other(), "Aegis Turtle", Zone::Battlefield);
        let b = named(&mut g, p.other(), "Aegis Turtle", Zone::Battlefield);
        next_main(&mut g, p);
        until(&mut g, |g| g.state.pending == Pending::Attackers);
        g = orphan(g, Action::Attack { card: winter });
        checked(&mut g, Action::FinishAttackers);
        assert!(!g.object_view(winter).tapped);
        until(&mut g, |g| g.state.pending == Pending::Blockers);
        checked(
            &mut g,
            Action::Block {
                blocker: a,
                attacker: winter,
            },
        );
        reject(&mut g, Action::FinishBlockers);
        checked(
            &mut g,
            Action::Block {
                blocker: b,
                attacker: winter,
            },
        );
        g = orphan(g, Action::FinishBlockers);
        assert!(!g.object_view(winter).tapped);
    }
}
#[test]
fn msh_attachment_card_winter_graveyard_only_colored_generic_and_opponent_turn_activation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let winter = paid(&mut g, p, WINTER, &[]);
        g.state.players[p.index()].mana = [30; 6];
        let action = Action::AnnounceActivate {
            card: winter,
            ability: 0,
            counter_payment: Counters::default(),
        };
        reject(&mut g, action.clone());
        paid(&mut g, p, "Murder", &[Target::Card(winter)]);
        for mana in [[1, 0, 0, 0, 0, 4], [1, 0, 1, 0, 0, 2]] {
            g.state.players[p.index()].mana = mana;
            reject(&mut g, action.clone());
        }
        next_turn(&mut g);
        until(&mut g, |g| {
            g.state.active == p.other() && g.state.step == Step::Main1
        });
        activate_return(&mut g, p, winter);
        settle(&mut g);
        assert_eq!(g.object_view(winter).zone, Zone::Battlefield);
        assert_eq!(g.counter_count(winter, CounterKind::Finality), 1);
        assert!(g.state.resolving.is_none());
    }
}
#[test]
fn msh_attachment_card_swordsman_self_foreign_and_nonvillain_excluded_zero_equipment_still_targets()
{
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let sword = paid(&mut g, p, SWORD, &[]);
        assert!(g.state.pending_triggers.is_empty());
        paid(&mut g, p, "Bear Cub", &[]);
        paid(&mut g, p.other(), "Doc Ock's Henchmen", &[]);
        assert!(g.state.pending_triggers.is_empty());
        packet(&mut g, p, "Mister Hyde, Monster Within", &[]);
        entry_targets(&mut g, sword);
        // No optional Equipment exists; the independent mandatory creature group
        // remains an actual targeted ability, and may select Swordsman himself.
        if g.observe(p).targeting_group == Some(0) {
            checked(&mut g, Action::FinishTargets);
        }
        assert_eq!(g.observe(p).targeting_group, Some(1));
        reject(&mut g, Action::FinishTargets);
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(sword),
            },
        );
        finish(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(sword).zone, Zone::Battlefield);
    }
}
#[test]
fn msh_attachment_card_swordsman_queued_changes_respect_current_attachment_legality() {
    for p in [Player::P0, Player::P1] {
        for change in 0..3 {
            let mut g = setup(p);
            let sword = paid(&mut g, p, SWORD, &[]);
            let pole = paid(&mut g, p, "Fishing Pole", &[]);
            let old = paid(&mut g, p, "Bear Cub", &[]);
            let new = paid(&mut g, p, "Hill Giant", &[]);
            equip(&mut g, p, pole, old);
            packet(&mut g, p, "Mister Hyde, Monster Within", &[]);
            entry_targets(&mut g, sword);
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(pole),
                },
            );
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(new),
                },
            );
            finish(&mut g);
            // Explicit earlier-layer boundary after the actual paid trigger and
            // both target declarations; no invented playable card is claimed.
            match change {
                0 => g.apply_temporary_copy(
                    g.object_ref(pole),
                    CopyRecipe::new(Card::from_name("Fishing Pole").unwrap())
                        .with_exceptions(CopyExceptions::GOLEM),
                    CopyDeadline::EndOfTurn,
                ),
                1 => g.apply_temporary_copy(
                    g.object_ref(new),
                    CopyRecipe::new(Card::from_name("Arc Reactor").unwrap()),
                    CopyDeadline::EndOfTurn,
                ),
                _ => g.change_controller(g.object_ref(new), p.other()),
            }
            settle(&mut g);
            assert_eq!(
                g.state.objects[pole as usize].attached_to,
                if change == 0 {
                    None
                } else {
                    Some(g.object_ref(old))
                }
            );
        }
    }
}
#[test]
fn msh_attachment_card_winter_foreign_equipment_count_and_actual_ability_suppression() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let winter = paid(&mut g, p, WINTER, &[]);
        let pole = paid(&mut g, p, "Fishing Pole", &[]);
        equip(&mut g, p, pole, winter);
        g.change_controller(g.object_ref(pole), p.other());
        assert_eq!(g.stats(winter).power, 4);
        let aura = paid(&mut g, p, "Witness Protection", &[Target::Card(winter)]);
        assert_eq!(g.stats(winter).power, 1);
        assert!(!g.keywords(winter).contains(Keywords::VIGILANCE));
        assert_eq!(
            g.state.objects[pole as usize].attached_to,
            Some(g.object_ref(winter))
        );
        paid(&mut g, p, "Disenchant", &[Target::Card(aura)]);
        assert_eq!(g.stats(winter).power, 4);
        assert!(
            g.keywords(winter)
                .contains(Keywords::VIGILANCE.union(Keywords::MENACE))
        );
    }
}
#[test]
fn msh_attachment_card_swordsman_two_equipped_attackers_connive_after_source_and_equipment_depart()
{
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let sword = paid(&mut g, p, SWORD, &[]);
        let a = paid(&mut g, p, "Bear Cub", &[]);
        let b = paid(&mut g, p, "Hill Giant", &[]);
        let pole_a = paid(&mut g, p, "Fishing Pole", &[]);
        let pole_b = paid(&mut g, p, "Fishing Pole", &[]);
        equip(&mut g, p, pole_a, a);
        equip(&mut g, p, pole_b, b);
        g.change_controller(g.object_ref(pole_b), p.other());
        next_main(&mut g, p);
        until(&mut g, |g| g.state.pending == Pending::Attackers);
        checked(&mut g, Action::Attack { card: a });
        checked(&mut g, Action::Attack { card: b });
        checked(&mut g, Action::FinishAttackers);
        until(&mut g, |g| {
            g.state.stack.iter().filter(|s| s.card == sword).count() == 2
        });
        packet(&mut g, p, "Unsummon", &[Target::Card(sword)]);
        until(&mut g, |g| {
            g.object_view(sword).zone == Zone::Hand && g.state.pending == Pending::Priority
        });
        packet(&mut g, p, "Disenchant", &[Target::Card(pole_a)]);
        let before = g.state.players[p.index()].library.len();
        let first = named(&mut g, p, "Shock", Zone::Hand);
        let second = named(&mut g, p, "Shock", Zone::Hand);
        for discard in [first, second] {
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::EffectDiscard { .. })
            });
            checked(&mut g, Action::Discard { card: discard });
        }
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].library.len(), before - 2);
        assert_eq!(g.counter_count(a, CounterKind::PlusOne), 1);
        assert_eq!(g.counter_count(b, CounterKind::PlusOne), 1);
        assert_eq!(g.object_view(sword).zone, Zone::Hand);
    }
}
