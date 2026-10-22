//! Shared subtype regressions. Immutable programs use the normal interpreter.
use super::ability_timeline_tests::{cast, checked, named, settle, setup};
use super::tests::spawn;
use super::*;

const WEREWOLF: &[Instruction] = &[Instruction::SetCreatureTypes {
    subject: Subject::Targets,
    types: &["Werewolf"],
}];

fn program(g: &mut Game, player: Player, target: CardId, code: &'static [Instruction]) {
    let source = spawn(g, player, Card::Shock, Zone::Hand);
    let mut item = g.cast_item(player, source, [0; 6]);
    item.targets.push(BoundTarget {
        group: 0,
        target: Target::Card(target),
        object: Some(g.object_ref(target)),
    });
    g.state.resolving = Some(Resolution {
        continuation: None,
        chosen_creature_type: None,
        chosen_color: None,
        connive: None,
        explore: None,
        evidence: None,
        item,
        mana_payment: None,
        forage: None,
        target_library: None,
        program: code,
        pc: 0,
        target_group: 0,
        moved_count: None,
        discards: [0; 2],
        selected_discards: [Vec::new(), Vec::new()],
        milled: Vec::new(),
        milled_selection: None,
        library: None,
        selection: None,
        search: None,
        optional_searches: None,
        choice: None,
    });
    g.run_program();
}

#[test]
fn subtype_layers_replacement_is_complete_preserves_noncreature_types_and_frozen_lki() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        for name in ["Bear Cub", "Chitinous Graspling"] {
            let target = named(&mut g, p, name, Zone::Battlefield);
            let old = g.object_view(target);

            program(&mut g, p, target, WEREWOLF);
            let changed = g.object_view(target);
            assert!(changed.has_subtype("Werewolf"));
            // The overlay preserves other subtype classes in mixed physical
            // identities too, without admitting that gated card's abilities.
            let food = Card::from_name("Gingerbrute").unwrap();
            assert!(changed.granted_subtypes.has(food, "Food"));
            assert!(!changed.granted_subtypes.has(food, "Golem"));
            for typ in crate::CREATURE_TYPES {
                assert_eq!(
                    changed.has_subtype(typ),
                    *typ == "Werewolf",
                    "{name}: {typ}"
                );
            }
            let facts = g.damage_facts(target);
            assert!(facts.has_subtype("Werewolf"));
            assert!(!facts.has_subtype("Golem"));
            for viewer in [p, p.other()] {
                let visible = g
                    .observe(viewer)
                    .objects
                    .into_iter()
                    .find(|v| v.id == target)
                    .unwrap();
                assert!(visible.has_subtype("Werewolf"));
                let wire = serde_json::to_value(visible).unwrap();
                assert_eq!(
                    wire["granted_subtypes"]["creature_types"],
                    serde_json::json!(["Werewolf"])
                );
            }
            let bounce = named(&mut g, p, "Unsummon", Zone::Hand);
            cast(&mut g, bounce, None, &[Target::Card(target)]);
            settle(&mut g);
            assert_eq!(g.object_view(target).zone, Zone::Hand);
            assert_eq!(
                g.object_view(target).has_subtype("Golem"),
                old.has_subtype("Golem")
            );
            assert!(changed.has_subtype("Werewolf"));
            assert!(facts.has_subtype("Werewolf"));
            assert!(g.state.roles.as_ref().unwrap().creature_types.is_empty());
        }
    }
}

#[test]
fn subtype_layers_static_threshold_is_reversible_and_starts_before_ability_removal() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let card = Card::from_name("Skyknight Squire").unwrap();
        assert!(card.is_supported());
        let squire = spawn(&mut g, p, card, Zone::Battlefield);
        g.state.objects[squire as usize].counters.plus_one = 2;
        assert!(!g.has_subtype(squire, "Knight"));
        assert!(!g.keywords(squire).contains(Keywords::FLYING));
        g.state.objects[squire as usize].counters.plus_one = 3;
        assert!(g.has_subtype(squire, "Cat"));
        assert!(g.has_subtype(squire, "Scout"));
        assert!(g.has_subtype(squire, "Knight"));
        assert!(g.keywords(squire).contains(Keywords::FLYING));
        g.state.objects[squire as usize].counters.minus_one = 1;
        g.add_counter(squire, CounterKind::Page, 2);
        let stats_before = g.stats(squire);
        let before_sba = g.snapshot();
        let mut branch = {
            let parent = g.clone();
            parent.clone()
        };
        branch.state_based_actions();
        g.state_based_actions();
        assert_eq!(g.snapshot(), branch.snapshot());
        assert_eq!(g.stats(squire), stats_before);
        assert_eq!(g.object_view(squire).counters.plus_one, 2);
        assert_eq!(g.object_view(squire).counters.minus_one, 0);
        assert_eq!(g.object_view(squire).counters.get(CounterKind::Page), 2);
        assert!(!g.has_subtype(squire, "Knight"));
        assert!(!g.keywords(squire).contains(Keywords::FLYING));
        let after_sba = g.snapshot();
        g.restore(&before_sba).unwrap();
        g.state_based_actions();
        assert_eq!(g.snapshot(), after_sba);
        g.state.objects[squire as usize].counters.plus_one = 3;
        let aura = named(&mut g, p, "Stop Cold", Zone::Hand);
        cast(&mut g, aura, None, &[Target::Card(squire)]);
        settle(&mut g);
        assert!(!g.printed_abilities_available(squire));
        // Layer four starts before layer-six removal. Its Knight fragment still
        // applies; the older flying grant is overwritten by newer Stop Cold.
        assert!(g.has_subtype(squire, "Knight"));
        assert!(!g.keywords(squire).contains(Keywords::FLYING));
        g.state.objects[squire as usize].counters.plus_one = 2;
        assert!(!g.has_subtype(squire, "Knight"));
        let bounce = named(&mut g, p, "Disenchant", Zone::Hand);
        cast(&mut g, bounce, None, &[Target::Card(aura)]);
        settle(&mut g);
        g.state.objects[squire as usize].counters.plus_one = 3;
        assert!(g.keywords(squire).contains(Keywords::FLYING));
    }
}

#[test]
fn subtype_layers_replacement_orders_equipment_and_static_additions_by_shared_timestamp() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let target = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let sword = named(&mut g, p, "Warrior's Sword", Zone::Battlefield);
        assert!(g.attach_equipment(g.object_ref(sword), g.object_ref(target)));
        assert!(g.has_subtype(target, "Warrior"));
        program(&mut g, p, target, WEREWOLF);
        assert!(!g.has_subtype(target, "Bear"));
        assert!(!g.has_subtype(target, "Warrior"));
        let other = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        assert!(g.attach_equipment(g.object_ref(sword), g.object_ref(other)));
        assert!(g.attach_equipment(g.object_ref(sword), g.object_ref(target)));
        assert!(g.has_subtype(target, "Warrior"));
        assert!(g.has_subtype(target, "Werewolf"));
        let squire = spawn(
            &mut g,
            p,
            Card::from_name("Skyknight Squire").unwrap(),
            Zone::Battlefield,
        );
        g.state.objects[squire as usize].counters.plus_one = 2;
        program(&mut g, p, squire, WEREWOLF);
        g.state.objects[squire as usize].counters.plus_one = 3;
        assert!(
            !g.has_subtype(squire, "Knight"),
            "condition changes do not retimestamp the older static effect"
        );
        assert!(g.has_subtype(squire, "Werewolf"));
        assert!(g.keywords(squire).contains(Keywords::FLYING));
    }
}

#[test]
fn subtype_layers_target_legality_uses_replaced_types_and_errors_are_transactional() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let target = named(&mut g, p.other(), "Chitinous Graspling", Zone::Battlefield);
        program(&mut g, p, target, WEREWOLF);
        let bounce = named(&mut g, p, "Unsummon", Zone::Hand);
        let source = g.object_ref(bounce);
        for (subtype, expected) in [("Human", false), ("Werewolf", true)] {
            let predicate = if subtype == "Human" {
                &TargetPredicate::Subtype("Human")
            } else {
                &TargetPredicate::Subtype("Werewolf")
            };
            assert_eq!(
                g.target_matches(
                    TargetRule::Filtered {
                        base: &TargetRule::Creature,
                        predicate
                    },
                    p,
                    source,
                    Target::Card(target)
                ),
                expected
            );
        }
        assert!(
            !g.observe(p.other())
                .objects
                .iter()
                .any(|object| object.id == bounce)
        );
        checked(&mut g, Action::AnnounceCast { card: bounce });
        let saved = g.snapshot();
        let revision = g.revision();
        assert_eq!(
            g.apply(
                p,
                Action::ChooseTarget {
                    target: Target::Player(p)
                }
            ),
            Err(Error::IllegalAction)
        );
        assert_eq!(saved, g.snapshot());
        assert_eq!(revision, g.revision());
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        while g.state.announcing.is_some() {
            let action = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PayCosts { .. } | Action::FinishTargets))
                .unwrap();
            checked(&mut g, action);
        }
        settle(&mut g);
        assert!(g.object_view(target).has_subtype("Human"));
        assert!(!g.object_view(target).granted_subtypes.contains("Werewolf"));
    }
}

#[test]
fn subtype_layers_lasting_replacement_survives_cleanup_but_never_rebinds_old_targets() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let target = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let source = named(&mut g, p, "Shock", Zone::Hand);
        let mut item = g.cast_item(p, source, [0; 6]);
        item.targets.push(BoundTarget {
            group: 0,
            target: Target::Card(target),
            object: Some(g.object_ref(target)),
        });
        g.set_creature_types(&item, Subject::Targets, &["Werewolf"]);
        let snap = g.snapshot();
        g.finish_cleanup();
        assert!(g.has_subtype(target, "Werewolf"));
        g.restore(&snap).unwrap();
        let old = g.object_ref(target);
        g.move_object(target, Zone::Hand);
        g.move_object(target, Zone::Battlefield);
        assert_ne!(old, g.object_ref(target));
        g.set_creature_types(&item, Subject::Targets, &["Werewolf"]);
        assert!(g.has_subtype(target, "Bear"));
        assert!(!g.has_subtype(target, "Werewolf"));
        assert!(g.state.roles.as_ref().unwrap().creature_types.is_empty());
    }
}
