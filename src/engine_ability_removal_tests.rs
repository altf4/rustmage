use super::ability_timeline_tests::{cast, checked, named, next_turn, settle, setup};
use super::*;

fn activate(g: &mut Game, card: CardId, ability: u16, target: CardId) {
    checked(
        g,
        Action::AnnounceActivate {
            card,
            ability,
            counter_payment: Counters::default(),
        },
    );
    checked(
        g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    for _ in 0..12 {
        if g.state.announcing.is_none() {
            break;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::FinishTargets))
            .or_else(|| {
                g.legal_actions()
                    .into_iter()
                    .find(|a| matches!(a, Action::PayCosts { .. }))
            })
            .unwrap();
        checked(g, action);
    }
    settle(g);
}

#[test]
fn ability_removal_paid_all_four_complete_bodies_counters_untap_and_real_activated_effects() {
    for p in [Player::P0, Player::P1] {
        for name in [
            "Fresh Start",
            "Frozen in Ice",
            "Stop Cold",
            "Enchanted River's Grasp",
        ] {
            let mut g = setup(p);
            let elf = named(&mut g, p, "Llanowar Elves", Zone::Battlefield);
            g.state.objects[elf as usize].summoning_sick = false;
            let researcher = named(&mut g, p, "Forensic Researcher", Zone::Battlefield);
            g.state.objects[researcher as usize].summoning_sick = false;
            if name == "Enchanted River's Grasp" {
                g.state.objects[elf as usize].counters = Counters {
                    plus_one: 4,
                    minus_one: 1,
                    stun: 3,
                    lore: 4,
                    loyalty: 5,
                };
            } else if name == "Frozen in Ice" {
                g.state.objects[elf as usize].counters.stun = 3;
            }
            let aura = named(&mut g, p, name, Zone::Hand);
            cast(&mut g, aura, None, &[Target::Card(elf)]);
            settle(&mut g);
            assert_eq!(
                g.object_view(aura).attached_to,
                Some(g.object_ref(elf)),
                "{name}"
            );
            assert!(!g.object_view(elf).abilities.unwrap().printed, "{name}");
            assert_eq!(
                g.stats(elf),
                Stats {
                    power: if name == "Fresh Start" { -4 } else { 1 },
                    toughness: 1
                },
                "{name}"
            );
            if name == "Enchanted River's Grasp" {
                assert_eq!(g.state.objects[elf as usize].counters, Counters::default());
            }
            assert_eq!(g.object_view(elf).tapped, name != "Fresh Start", "{name}");
            assert!(!g.legal_actions().iter().any(|action| matches!(action,
                Action::TapMana { card } | Action::AnnounceActivate { card, .. } if *card == elf)));
            if name != "Fresh Start" {
                let after = g.snapshot();
                activate(&mut g, researcher, 0, elf);
                assert_eq!(
                    g.object_view(elf).tapped,
                    name == "Frozen in Ice",
                    "effect untap {name}"
                );
                assert_eq!(
                    g.state.objects[elf as usize].counters.stun,
                    if name == "Frozen in Ice" { 3 } else { 0 }
                );
                g.restore(&after).unwrap();
                next_turn(&mut g);
                next_turn(&mut g);
                assert_eq!(g.state.active, p);
                assert!(g.object_view(elf).tapped, "controller untap {name}");
                assert_eq!(
                    g.state.objects[elf as usize].counters.stun,
                    if name == "Frozen in Ice" { 3 } else { 0 }
                );
            }
        }
    }
}

#[test]
fn ability_removal_paid_stop_cold_keeps_earlier_job_effect_but_removes_equipment_activation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let equipment = named(&mut g, p, "Samurai's Katana", Zone::Hand);
        cast(&mut g, equipment, None, &[]);
        settle(&mut g);
        let hero = g.state.objects[equipment as usize]
            .attached_to
            .unwrap()
            .card;
        let before = g.object_view(hero);
        assert!(before.has_subtype("Samurai"));
        let aura = named(&mut g, p, "Stop Cold", Zone::Hand);
        cast(&mut g, aura, None, &[Target::Card(equipment)]);
        settle(&mut g);
        assert!(g.object_view(equipment).tapped);
        assert!(!g.object_view(equipment).abilities.unwrap().printed);
        let after = g.object_view(hero);
        assert!(after.has_subtype("Samurai"));
        assert_eq!(
            (after.power, after.toughness, after.keywords),
            (before.power, before.toughness, before.keywords)
        );
        assert!(!g.legal_actions().iter().any(
            |action| matches!(action, Action::AnnounceActivate { card, .. } if *card == equipment)
        ));
    }
}

#[test]
fn ability_removal_paid_external_blocking_rules_survive_real_removal_aura() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enigma = named(&mut g, p, "Enter the Enigma", Zone::Hand);
        cast(&mut g, enigma, None, &[Target::Card(bear)]);
        settle(&mut g);
        let aura = named(&mut g, p, "Stop Cold", Zone::Hand);
        cast(&mut g, aura, None, &[Target::Card(bear)]);
        settle(&mut g);
        assert!(
            g.object_view(bear)
                .keywords
                .contains(Keywords::CANT_BE_BLOCKED)
        );
        assert!(!g.object_view(bear).abilities.unwrap().printed);
    }
}

#[test]
fn ability_removal_paid_layer_six_orders_resolution_grants_without_removing_counters() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let old = named(&mut g, p, "Take Up the Shield", Zone::Hand);
        cast(&mut g, old, None, &[Target::Card(bear)]);
        settle(&mut g);
        let aura = named(&mut g, p, "Stop Cold", Zone::Hand);
        cast(&mut g, aura, None, &[Target::Card(bear)]);
        settle(&mut g);
        assert!(!g.keywords(bear).contains(Keywords::INDESTRUCTIBLE));
        assert!(!g.keywords(bear).contains(Keywords::LIFELINK));
        assert_eq!(g.state.objects[bear as usize].counters.plus_one, 1);
        let later = named(&mut g, p, "Take Up the Shield", Zone::Hand);
        cast(&mut g, later, None, &[Target::Card(bear)]);
        settle(&mut g);
        assert!(g.keywords(bear).contains(Keywords::INDESTRUCTIBLE));
        assert!(g.keywords(bear).contains(Keywords::LIFELINK));
        assert_eq!(g.state.objects[bear as usize].counters.plus_one, 2);
        assert!(!g.object_view(bear).abilities.unwrap().printed);
    }
}

#[test]
fn ability_removal_paid_live_multipart_provider_dependency_and_pure_later_provider() {
    for p in [Player::P0, Player::P1] {
        for source_name in ["Archon of the Wild Rose", "Maha, Its Feathers Night"] {
            let mut g = setup(p);
            let source = named(&mut g, p, source_name, Zone::Battlefield);
            let recipient_owner = if source_name.starts_with("Maha") {
                p.other()
            } else {
                p
            };
            let recipient = named(&mut g, recipient_owner, "Bear Cub", Zone::Battlefield);
            if recipient_owner == p {
                let royal = named(&mut g, p, "Royal Treatment", Zone::Hand);
                cast(&mut g, royal, None, &[Target::Card(recipient)]);
                settle(&mut g);
                assert_eq!(
                    g.stats(recipient),
                    Stats {
                        power: 5,
                        toughness: 5
                    }
                );
                assert!(g.keywords(recipient).contains(Keywords::FLYING));
            } else {
                assert_eq!(g.stats(recipient).toughness, 1);
            }
            let aura = named(&mut g, p, "Stop Cold", Zone::Hand);
            cast(&mut g, aura, None, &[Target::Card(source)]);
            settle(&mut g);
            assert!(!g.object_view(source).abilities.unwrap().printed);
            if recipient_owner == p {
                assert_eq!(
                    g.stats(recipient),
                    Stats {
                        power: 3,
                        toughness: 3
                    }
                );
                assert!(!g.keywords(recipient).contains(Keywords::FLYING));
            } else {
                assert_eq!(g.stats(recipient).toughness, 2);
            }
            let remove = named(&mut g, p, "Disenchant", Zone::Hand);
            cast(&mut g, remove, None, &[Target::Card(aura)]);
            settle(&mut g);
            assert_eq!(g.state.objects[aura as usize].zone, Zone::Graveyard);
            if recipient_owner == p {
                assert_eq!(
                    g.stats(recipient),
                    Stats {
                        power: 5,
                        toughness: 5
                    }
                );
                assert!(g.keywords(recipient).contains(Keywords::FLYING));
            } else {
                assert_eq!(g.stats(recipient).toughness, 1);
            }
        }
    }
}

#[test]
fn ability_removal_paid_already_queued_trigger_keeps_its_own_program() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, "Aquatic Alchemist", Zone::Battlefield);
        let draw = named(&mut g, p, "Quick Study", Zone::Hand);
        cast(&mut g, draw, None, &[]);
        let aura = named(&mut g, p, "Stop Cold", Zone::Hand);
        cast(&mut g, aura, None, &[Target::Card(source)]);
        settle(&mut g);
        assert!(!g.object_view(source).abilities.unwrap().printed);
        assert_eq!(g.state.objects[source as usize].counters.plus_one, 0);
        assert_eq!(
            g.stats(source),
            Stats {
                power: 3,
                toughness: 3
            }
        );
    }
}

#[test]
fn ability_removal_paid_removed_minstrel_cannot_override_land_entry_then_restores() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let minstrel = named(&mut g, p, "The Wandering Minstrel", Zone::Hand);
        cast(&mut g, minstrel, None, &[]);
        settle(&mut g);
        let aura = named(&mut g, p, "Stop Cold", Zone::Hand);
        cast(&mut g, aura, None, &[Target::Card(minstrel)]);
        settle(&mut g);
        let first = named(&mut g, p, "Baron, Airship Kingdom", Zone::Hand);
        checked(&mut g, Action::PlayLand { card: first });
        settle(&mut g);
        assert!(g.object_view(first).tapped);
        let remove = named(&mut g, p, "Disenchant", Zone::Hand);
        cast(&mut g, remove, None, &[Target::Card(aura)]);
        settle(&mut g);
        next_turn(&mut g);
        next_turn(&mut g);
        while g.state.step != Step::Main1 {
            checked(&mut g, Action::Pass);
        }
        let second = named(&mut g, p, "Baron, Airship Kingdom", Zone::Hand);
        checked(&mut g, Action::PlayLand { card: second });
        let intrinsic = g
            .entry_tap_view(p)
            .unwrap()
            .effects
            .into_iter()
            .find(|effect| effect.intrinsic)
            .unwrap();
        checked(
            &mut g,
            Action::ChooseEntryReplacement {
                source: intrinsic.source,
                intrinsic: true,
            },
        );
        settle(&mut g);
        assert!(!g.object_view(second).tapped);
    }
}
