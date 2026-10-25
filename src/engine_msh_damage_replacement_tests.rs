//! Individual paid damage/replacement cards; no aggregate set certification.
use super::assurance::big_helpers::{cast, checked, named, next_turn, settle, setup};
use super::msh_thirteenth_combined_tests::owned_checked_step;
use super::*;
const HAWK: &str = "Hawkeye, Young Avenger";
const WOLF: &str = "Wolverine, Fierce Fighter";
const HAMMER: &str = "Mjölnir, Hammer of Thor";
fn seed(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let card = Card::from_name(name).unwrap();
    if card.is_supported() {
        return named(g, p, name, zone);
    }
    assert!([HAWK, WOLF, HAMMER].contains(&name));
    let id = named(g, p, "Hill Giant", zone);
    g.state.objects[id as usize].card = card;
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
    for _ in 0..220 {
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
                        | Action::AssignDamage { .. }
                )
            })
            .unwrap_or_else(|| panic!("unexpected {:?}", g.state.pending));
        checked(g, a);
    }
    panic!("stalled {:?}", g.state.pending);
}
fn targets(g: &mut Game, id: CardId) {
    until(g, |g| {
        matches!(g.state.pending, Pending::Targets { .. })
            && g.state
                .announcing
                .as_ref()
                .is_some_and(|a| a.item.card == id)
    });
}
fn finish(g: &mut Game) {
    while g.state.announcing.is_some() {
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
            .unwrap();
        checked(g, a);
    }
}
fn paid_entry(g: &mut Game, p: Player, name: &str, target: Option<CardId>) -> CardId {
    let id = packet(g, p, name, &[]);
    if let Some(target) = target {
        targets(g, id);
        checked(
            g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        finish(g);
    }
    settle(g);
    id
}
fn activation(id: CardId, ability: u16) -> Action {
    Action::AnnounceActivate {
        card: id,
        ability,
        counter_payment: Counters::default(),
    }
}
fn activate(
    g: &mut Game,
    p: Player,
    id: CardId,
    ability: u16,
    mana: [u32; 6],
    target: Option<Target>,
) {
    priority(g, p);
    g.state.players[p.index()].mana = mana;
    checked(g, activation(id, ability));
    if let Some(target) = target {
        checked(g, Action::ChooseTarget { target });
    }
    finish(g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
}
fn reject(g: &mut Game, a: Action) {
    let s = g.snapshot();
    let bytes = g.state_bytes();
    let v = [Player::P0, Player::P1].map(|p| serde_json::to_value(g.observe(p)).unwrap());
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), s);
    assert_eq!(g.state_bytes(), bytes);
    assert_eq!(
        [Player::P0, Player::P1].map(|p| serde_json::to_value(g.observe(p)).unwrap()),
        v
    );
}
fn orphan(g: Game, a: Action) -> Game {
    let p = g.actor().unwrap();
    let i = g.legal_actions().iter().position(|v| *v == a).unwrap();
    owned_checked_step(g, p, i, a)
}
fn main(g: &mut Game, p: Player) {
    until(g, |g| {
        g.state.active == p && g.state.step == Step::Main1 && g.state.pending == Pending::Priority
    });
    priority(g, p);
}
fn next_main(g: &mut Game, p: Player) {
    next_turn(g);
    next_turn(g);
    main(g, p);
}
#[test]
fn msh_damage_hawkeye_paid_costs_reach_and_own_opponent_recipient_filter() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = seed(&mut g, p, HAWK, Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 3];
        reject(&mut g, Action::AnnounceCast { card: id });
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 3];
        cast(&mut g, id, None, &[]);
        settle(&mut g);
        assert_eq!(
            g.stats(id),
            Stats {
                power: 2,
                toughness: 4
            }
        );
        assert!(g.keywords(id).contains(Keywords::REACH));
        let before = g.state.players[p.other().index()].life;
        paid(&mut g, p, "Shock", &[Target::Player(p.other())]);
        assert_eq!(g.state.players[p.other().index()].life, before - 4);
        paid(&mut g, p, "Shock", &[Target::Card(id)]);
        assert_eq!(g.object_view(id).damage, 2);
        paid(&mut g, p.other(), "Shock", &[Target::Card(id)]);
        assert_eq!(g.object_view(id).zone, Zone::Graveyard);
    }
}
#[test]
fn msh_damage_hawkeye_signed_power_clamps_only_final_damage() {
    for p in [Player::P0, Player::P1] {
        for minuses in [0, 2] {
            let mut g = setup(p);
            let id = paid(&mut g, p, HAWK, &[]);
            // Bounded actual counter storage fixture supplies a negative current power without lethal toughness.
            g.set_counters(id, Counters::of(CounterKind::MinusOne, 3));
            assert_eq!(
                g.stats(id),
                Stats {
                    power: -1,
                    toughness: 1
                }
            );
            for _ in 0..minuses {
                paid(&mut g, p, "Fleeting Distraction", &[Target::Card(id)]);
            }
            let before = g.state.players[p.other().index()].life;
            paid(&mut g, p, "Shock", &[Target::Player(p.other())]);
            assert_eq!(
                g.state.players[p.other().index()].life,
                before - if minuses == 0 { 1 } else { 0 }
            );
        }
    }
}
#[test]
fn msh_damage_hawkeye_current_power_at_resolution_and_departed_provider() {
    for p in [Player::P0, Player::P1] {
        for kill in [false, true] {
            let mut g = setup(p);
            let id = paid(&mut g, p, HAWK, &[]);
            let before = g.state.players[p.other().index()].life;
            packet(&mut g, p, "Shock", &[Target::Player(p.other())]);
            if kill {
                packet(&mut g, p.other(), "Murder", &[Target::Card(id)]);
            } else {
                packet(&mut g, p, "Thwip!", &[Target::Card(id)]);
            }
            settle(&mut g);
            assert_eq!(
                g.state.players[p.other().index()].life,
                before - if kill { 2 } else { 6 }
            );
        }
    }
}
#[test]
fn msh_damage_hawkeye_ability_removal_and_restoration() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, HAWK, &[]);
        let aura = paid(&mut g, p, "Witness Protection", &[Target::Card(id)]);
        let before = g.state.players[p.other().index()].life;
        paid(&mut g, p, "Shock", &[Target::Player(p.other())]);
        assert_eq!(g.state.players[p.other().index()].life, before - 2);
        paid(&mut g, p, "Disenchant", &[Target::Card(aura)]);
        paid(&mut g, p, "Shock", &[Target::Player(p.other())]);
        assert_eq!(g.state.players[p.other().index()].life, before - 6);
    }
}
#[test]
fn msh_damage_mjolnir_paid_entry_optional_four_and_target_departure() {
    for p in [Player::P0, Player::P1] {
        for choose in [false, true] {
            let mut g = setup(p);
            let bear = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let id = paid_entry(&mut g, p, HAMMER, choose.then_some(bear));
            assert_eq!(g.object_view(id).zone, Zone::Battlefield);
            assert_eq!(
                g.object_view(bear).zone,
                if choose {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
        }
        let mut g = setup(p);
        let bear = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let id = packet(&mut g, p, HAMMER, &[]);
        targets(&mut g, id);
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        finish(&mut g);
        packet(&mut g, p.other(), "Unsummon", &[Target::Card(bear)]);
        settle(&mut g);
        assert_eq!(g.object_view(bear).zone, Zone::Hand);
    }
}
#[test]
fn msh_damage_mjolnir_worthy_equip_checks_all_current_restrictions_and_atomicity() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid_entry(&mut g, p, HAMMER, None);
        let hawk = paid(&mut g, p, HAWK, &[]);
        let winter = paid(&mut g, p, "Winter Soldier, Icy Assassin", &[]);
        let invalid = [
            named(&mut g, p, "Serra Angel", Zone::Battlefield),
            named(&mut g, p, "Doc Samson", Zone::Battlefield),
            named(&mut g, p, "Thanos, the Mad Titan", Zone::Battlefield),
            named(
                &mut g,
                p.other(),
                "Winter Soldier, Icy Assassin",
                Zone::Battlefield,
            ),
        ];
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        checked(&mut g, activation(id, 0));
        for bad in invalid {
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(bad),
                },
            );
        }
        reject(&mut g, Action::FinishTargets);
        g = orphan(
            g,
            Action::ChooseTarget {
                target: Target::Card(hawk),
            },
        );
        finish(&mut g);
        settle(&mut g);
        assert_eq!(
            g.state.objects[id as usize].attached_to,
            Some(g.object_ref(hawk))
        );
        activate(
            &mut g,
            p,
            id,
            0,
            [0, 0, 0, 0, 0, 1],
            Some(Target::Card(winter)),
        );
        settle(&mut g);
        assert_eq!(
            g.state.objects[id as usize].attached_to,
            Some(g.object_ref(winter))
        );
    }
}
#[test]
fn msh_damage_mjolnir_equip_sorcery_costs_target_rechecked_after_announcement() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid_entry(&mut g, p, HAMMER, None);
        let hawk = paid(&mut g, p, HAWK, &[]);
        g.state.players[p.index()].mana = [0; 6];
        reject(&mut g, activation(id, 0));
        paid(&mut g, p, "High Fae Trickster", &[]);
        activate(
            &mut g,
            p,
            id,
            0,
            [0, 0, 0, 0, 0, 1],
            Some(Target::Card(hawk)),
        );
        packet(&mut g, p, "Witness Protection", &[Target::Card(hawk)]);
        settle(&mut g);
        assert_eq!(g.state.objects[id as usize].attached_to, None);
        next_turn(&mut g);
        main(&mut g, p.other());
        priority(&mut g, p);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        reject(&mut g, activation(id, 0));
    }
}
fn unworthy_equipped(g: &mut Game, p: Player) -> (CardId, CardId, CardId) {
    let sword = paid(g, p, "Swordsman, Sharp Scoundrel", &[]);
    let hammer = paid_entry(g, p, HAMMER, None);
    let skrull = packet(g, p, "Super-Skrull", &[]);
    targets(g, sword);
    checked(
        g,
        Action::ChooseTarget {
            target: Target::Card(hammer),
        },
    );
    checked(
        g,
        Action::ChooseTarget {
            target: Target::Card(skrull),
        },
    );
    finish(g);
    settle(g);
    assert_eq!(
        g.state.objects[hammer as usize].attached_to,
        Some(g.object_ref(skrull))
    );
    (sword, hammer, skrull)
}
#[test]
fn msh_damage_mjolnir_unworthy_actual_attachment_noncombat_order_lifelink_owned_choices() {
    for p in [Player::P0, Player::P1] {
        for double_first in [false, true] {
            let mut g = setup(p);
            let (_, hammer, skrull) = unworthy_equipped(&mut g, p);
            let hawk = paid(&mut g, p, HAWK, &[]);
            let collar = paid(&mut g, p, "Basilisk Collar", &[]);
            activate(
                &mut g,
                p,
                collar,
                0,
                [0, 0, 0, 0, 0, 2],
                Some(Target::Card(skrull)),
            );
            settle(&mut g);
            let victim = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            let before = g.state.players[p.index()].life;
            activate(
                &mut g,
                p,
                skrull,
                2,
                [0, 0, 0, 1, 0, 4],
                Some(Target::Card(victim)),
            );
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::Replacement { .. })
            });
            assert_eq!(g.actor(), Some(p.other()));
            let view = g.observe(p.other()).replacement_choice.unwrap();
            assert_eq!(view.effects.len(), 2);
            assert!(view.damage_preventions.is_empty());
            let invalid = Action::ChooseReplacement {
                source: g.object_ref(skrull),
                ability: 0,
            };
            reject(&mut g, invalid);
            let source = g.object_ref(if double_first { hammer } else { hawk });
            g = orphan(g, Action::ChooseReplacement { source, ability: 0 });
            settle(&mut g);
            assert_eq!(
                g.state.players[p.index()].life,
                before + if double_first { 10 } else { 12 }
            );
            assert_eq!(g.object_view(victim).zone, Zone::Graveyard);
        }
    }
}
#[test]
fn msh_damage_mjolnir_foreign_equipment_controller_still_doubles_creature_damage() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let (_, hammer, skrull) = unworthy_equipped(&mut g, p);
        paid(
            &mut g,
            p,
            "Harmless Offering",
            &[Target::Player(p.other()), Target::Card(hammer)],
        );
        let victim = named(&mut g, p.other(), "Colossal Dreadmaw", Zone::Battlefield);
        activate(
            &mut g,
            p,
            skrull,
            2,
            [0, 0, 0, 1, 0, 4],
            Some(Target::Card(victim)),
        );
        settle(&mut g);
        assert_eq!(g.object_view(victim).zone, Zone::Graveyard);
    }
}
#[test]
fn msh_damage_mjolnir_channel_actual_discard_colored_generic_hand_only_and_instant() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = seed(&mut g, p, HAMMER, Zone::Hand);
        for mana in [[0, 0, 0, 0, 1, 2], [0, 0, 0, 1, 0, 1]] {
            g.state.players[p.index()].mana = mana;
            reject(&mut g, activation(id, 1));
        }
        let a = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let b = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let c = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        next_turn(&mut g);
        main(&mut g, p.other());
        activate(&mut g, p, id, 1, [0, 0, 0, 1, 0, 2], None);
        assert_eq!(g.object_view(id).zone, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.object_view(a).zone, Zone::Graveyard);
        assert_eq!(g.object_view(b).zone, Zone::Graveyard);
        assert_eq!(g.object_view(c).damage, 2);
    }
}
#[test]
fn msh_damage_wolverine_optional_other_fight_and_lethal_damage_not_prevention() {
    for p in [Player::P0, Player::P1] {
        for victim_name in ["Bear Cub", "Colossal Dreadmaw"] {
            let mut g = setup(p);
            let victim = named(&mut g, p.other(), victim_name, Zone::Battlefield);
            let id = packet(&mut g, p, WOLF, &[]);
            targets(&mut g, id);
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(id),
                },
            );
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(victim),
                },
            );
            finish(&mut g);
            settle(&mut g);
            assert!(g.keywords(id).contains(Keywords::HASTE));
            assert_eq!(
                g.object_view(id).zone,
                if victim_name == "Bear Cub" {
                    Zone::Battlefield
                } else {
                    Zone::Graveyard
                }
            );
            if victim_name == "Bear Cub" {
                assert_eq!(g.object_view(id).damage, 2);
                assert_eq!(g.object_view(victim).zone, Zone::Graveyard);
            }
        }
        let mut g = setup(p);
        let id = paid_entry(&mut g, p, WOLF, None);
        assert_eq!(
            g.stats(id),
            Stats {
                power: 3,
                toughness: 5
            }
        );
        assert_eq!(g.object_view(id).damage, 0);
    }
}
#[test]
fn msh_damage_wolverine_sequential_heals_only_old_marks_keeps_damage_history() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid_entry(&mut g, p, WOLF, None);
        paid(&mut g, p.other(), "Shock", &[Target::Card(id)]);
        assert_eq!(g.object_view(id).damage, 2);
        paid(&mut g, p.other(), "Shock", &[Target::Card(id)]);
        assert_eq!(g.object_view(id).damage, 2);
        assert!(g.state.objects[id as usize].damaged_this_turn);
        paid(&mut g, p.other(), "Murder", &[Target::Card(id)]);
        assert_eq!(g.object_view(id).zone, Zone::Graveyard);
    }
}
#[test]
fn msh_damage_wolverine_fight_source_or_target_departure_never_rebinds() {
    for p in [Player::P0, Player::P1] {
        for source_gone in [false, true] {
            let mut g = setup(p);
            let victim = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            let id = packet(&mut g, p, WOLF, &[]);
            targets(&mut g, id);
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(victim),
                },
            );
            finish(&mut g);
            let gone = if source_gone { id } else { victim };
            packet(&mut g, p.other(), "Unsummon", &[Target::Card(gone)]);
            settle(&mut g);
            assert_eq!(g.object_view(gone).zone, Zone::Hand);
            let stayed = if source_gone { victim } else { id };
            assert_eq!(g.object_view(stayed).damage, 0);
        }
    }
}
fn attack_blocks(g: &mut Game, attacker: CardId, blockers: &[CardId]) {
    until(g, |g| g.state.pending == Pending::Attackers);
    checked(g, Action::Attack { card: attacker });
    checked(g, Action::FinishAttackers);
    until(g, |g| g.state.pending == Pending::Blockers);
    for &blocker in blockers {
        checked(g, Action::Block { blocker, attacker });
    }
    checked(g, Action::FinishBlockers);
}
#[test]
fn msh_damage_wolverine_simultaneous_combat_keeps_entire_new_batch_after_one_heal() {
    for p in [Player::P0, Player::P1] {
        for name in ["Bear Cub", "Hill Giant"] {
            let mut g = setup(p);
            let id = paid_entry(&mut g, p, WOLF, None);
            paid(&mut g, p.other(), "Shock", &[Target::Card(id)]);
            let a = named(&mut g, p.other(), name, Zone::Battlefield);
            let b = named(&mut g, p.other(), name, Zone::Battlefield);
            attack_blocks(&mut g, id, &[a, b]);
            until(&mut g, |g| {
                g.state.step == Step::Main2 && g.state.pending == Pending::Priority
            });
            assert_eq!(
                g.object_view(id).zone,
                if name == "Bear Cub" {
                    Zone::Battlefield
                } else {
                    Zone::Graveyard
                }
            );
            if name == "Bear Cub" {
                assert_eq!(g.object_view(id).damage, 4);
            }
        }
    }
}
#[test]
fn msh_damage_wolverine_new_deathtouch_is_not_erased_by_healing() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid_entry(&mut g, p, WOLF, None);
        let fynn = named(&mut g, p.other(), "Fynn, the Fangbearer", Zone::Battlefield);
        attack_blocks(&mut g, id, &[fynn]);
        until(&mut g, |g| {
            g.state.step == Step::Main2 && g.state.pending == Pending::Priority
        });
        assert_eq!(g.object_view(id).zone, Zone::Graveyard);
        assert_eq!(g.object_view(fynn).zone, Zone::Graveyard);
    }
}
fn slagstorm(g: &mut Game, p: Player) {
    priority(g, p);
    let id = named(g, p, "Slagstorm", Zone::Hand);
    g.state.players[p.index()].mana = [0, 0, 0, 2, 0, 1];
    cast(g, id, Some(0), &[]);
}
#[test]
fn msh_damage_wolverine_prevention_order_exposes_owned_queued_heal_truthfully() {
    for p in [Player::P0, Player::P1] {
        for heal_first in [false, true] {
            let mut g = setup(p);
            let id = paid_entry(&mut g, p, WOLF, None);
            paid(&mut g, p.other(), "Shock", &[Target::Card(id)]);
            let sword = paid(&mut g, p, "Sword of Wealth and Power", &[]);
            activate(
                &mut g,
                p,
                sword,
                0,
                [0, 0, 0, 0, 0, 2],
                Some(Target::Card(id)),
            );
            settle(&mut g);
            next_turn(&mut g);
            main(&mut g, p.other());
            // Cleanup removed old marks. Establish current-turn marks with a creature ability, which protection permits.
            let skrull = paid(&mut g, p.other(), "Super-Skrull", &[]);
            activate(
                &mut g,
                p.other(),
                skrull,
                2,
                [0, 0, 0, 1, 0, 4],
                Some(Target::Card(id)),
            );
            settle(&mut g);
            assert_eq!(g.object_view(id).damage, 4);
            paid(&mut g, p.other(), HAWK, &[]);
            slagstorm(&mut g, p.other());
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::Replacement { .. })
            });
            assert_eq!(g.actor(), Some(p));
            let reference = g.object_ref(id);
            let view = g.observe(p).replacement_choice.unwrap();
            assert_eq!(view.effects.len(), 2);
            assert_eq!(view.damage_preventions.len(), 1);
            assert!(view.heals_before_damage.is_empty());
            if heal_first {
                g = orphan(
                    g,
                    Action::ChooseReplacement {
                        source: reference,
                        ability: 0,
                    },
                );
                let v = g.observe(p).replacement_choice.unwrap();
                assert_eq!(v.heals_before_damage, vec![reference]);
                assert_eq!(v.damage_preventions.len(), 1);
            }
            let action = Action::ChooseDamagePrevention {
                source: reference,
                prevention: DamagePreventionKind::ProtectionInstantSorcery,
            };
            g = orphan(g, action);
            settle(&mut g);
            assert_eq!(g.object_view(id).damage, if heal_first { 0 } else { 4 });
        }
    }
}
