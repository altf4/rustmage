//! A real vivid-deck negative funding search and a legal restricted-mana converter.
use crate::ecl_scenarios::{cast, checked, named, settle, setup};
use rustmage_core::cards::*;
use rustmage_core::engine::ability_world::CharacteristicWorld;
use rustmage_core::engine::funding::{FundingGoal, FundingState};
use rustmage_core::engine::*;
use std::collections::HashSet;

fn position(p: Player, converter: bool) -> (Game, CardId, CardId) {
    let mut g = setup(p);
    for name in [
        "Forest",
        "Swamp",
        "Plains",
        "Great Forest Druid",
        "Great Forest Druid",
        "Flamebraider",
    ] {
        let id = named(&mut g, p, name, Zone::Battlefield);
        g.state.objects[id as usize].summoning_sick = false;
    }
    let realms = named(&mut g, p, "Eclipsed Realms", Zone::Battlefield);
    g.set_entry_value(
        g.object_ref(realms),
        EntryChoiceValue::CreatureType(
            rustmage_core::cards::CreatureType::from_name("Elf").unwrap(),
        ),
    );
    let druid = if converter {
        named(&mut g, p, "Foraging Wickermaw", Zone::Battlefield)
    } else {
        named(&mut g, p, "Bear Cub", Zone::Battlefield)
    };
    // An unrelated real attachment must not disable the negative shortcut.
    // On the converter it makes the activated source an Elemental, so the
    // restricted lot can pay that activation and become unrestricted mana.
    let dagger = named(&mut g, p, "Stalactite Dagger", Zone::Battlefield);
    assert!(g.attach_equipment(g.object_ref(dagger), g.object_ref(druid)));
    let aurora = named(&mut g, p, "Aurora Awakener", Zone::Hand);
    let prism = named(&mut g, p, "Prismabasher", Zone::Hand);
    g.state.players[p.index()].mana = [0; 6];
    (g, aurora, prism)
}
fn world_for(g: &Game, p: Player, card: CardId) -> (FundingState<'_>, FundingGoal) {
    let definition = g.state.objects[card as usize].card;
    let (required, generic) = definition.cost();
    let goal = FundingGoal {
        actor: p,
        required,
        generic,
        reserve: None,
        source: None,
    };
    let mut world = FundingState::from_game(g, &goal);
    world.set_purpose(g.spell_mana_purpose_for_card(p, card, definition));
    (world, goal)
}
#[test]
fn restricted_mana_rejects_impossible_aurora_and_keeps_elemental() {
    for p in [Player::P0, Player::P1] {
        let (mut g, aurora, prism) = position(p, false);
        named(&mut g, p.other(), "Firdoch Core", Zone::Battlefield);
        named(
            &mut g,
            p.other(),
            "Prismatic Undercurrents",
            Zone::Battlefield,
        );
        let (world, goal) = world_for(&g, p, aurora);
        assert!(world.total_mana_upper() >= 7);
        let mut visited = HashSet::new();
        assert!(!world.reachable(&goal, &mut visited));
        assert!(visited.is_empty());
        let flame = g
            .state
            .objects
            .iter()
            .position(|object| {
                object.zone == Zone::Battlefield
                    && object.card == Card::from_name("Flamebraider").unwrap()
            })
            .unwrap() as CardId;
        checked(
            &mut g,
            Action::ActivateMana {
                card: flame,
                ability: 0,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production: [0, 0, 0, 2, 0, 0],
            },
        );
        let (world, goal) = world_for(&g, p, aurora);
        assert!(
            !world.reachable(&goal, &mut HashSet::new()),
            "existing restricted lots remain ineligible"
        );
        let before = g.snapshot();
        let actions = g.legal_actions();
        assert!(!actions.contains(&Action::AnnounceCast { card: aurora }));
        assert!(actions.contains(&Action::AnnounceCast { card: prism }));
        assert_eq!(g.snapshot(), before);
        let revision = g.revision();
        let mut branch = {
            let parent = g.clone();
            parent.clone()
        };
        assert_eq!(
            branch.apply(p, Action::AnnounceCast { card: aurora }),
            Err(Error::IllegalAction)
        );
        assert_eq!(branch.snapshot(), before);
        assert_eq!(branch.revision(), revision);
        assert_eq!(
            g.apply(p, Action::AnnounceCast { card: aurora }),
            Err(Error::IllegalAction)
        );
        assert_eq!(g.snapshot(), before);
    }
}
#[test]
fn restricted_mana_converter_fallback_preserves_paid_aurora_and_owned_actions() {
    for p in [Player::P0, Player::P1] {
        let (mut g, aurora, _) = position(p, true);
        // Freeze the six unrestricted contributions in the pool; the positive
        // regression exercises conversion rather than unrelated color ordering.
        for object in &mut g.state.objects {
            if object.zone == Zone::Battlefield
                && object.controller == p
                && object.card != Card::from_name("Flamebraider").unwrap()
            {
                object.tapped = true;
            }
        }
        g.state.players[p.index()].mana = [0, 0, 0, 0, 6, 0];
        let (world, goal) = world_for(&g, p, aurora);
        assert!(world.reachable(&goal, &mut HashSet::new()));
        checked(&mut g, Action::AnnounceCast { card: aurora });
        for _ in 0..20 {
            if g.state.announcing.is_none() {
                break;
            }
            let actions = g.legal_actions();
            let action = actions
                .iter()
                .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
                .or_else(|| {
                    actions
                        .iter()
                        .find(|a| matches!(a, Action::TapMana { .. } | Action::ActivateMana { .. }))
                })
                .expect("legal funding plan has a next action")
                .clone();
            checked(&mut g, action);
        }
        assert!(g.state.announcing.is_none());
        settle(&mut g);
        assert_eq!(g.state.objects[aurora as usize].zone, Zone::Battlefield);
    }
}

#[test]
fn restricted_color_bound_rejects_kinbinding_with_abundant_ineligible_white() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        for name in [
            "Forest",
            "Forest",
            "Forest",
            "Forest",
            "Island",
            "Island",
            "Swamp",
            "Mountain",
            "Great Forest Druid",
            "Flamebraider",
            "Flamebraider",
            "Flamebraider",
        ] {
            let id = named(&mut g, p, name, Zone::Battlefield);
            g.state.objects[id as usize].summoning_sick = false;
        }
        let spell = named(&mut g, p, "Kinbinding", Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        let (world, goal) = world_for(&g, p, spell);
        assert!(
            world.total_mana_upper() >= 5,
            "quantity alone cannot reject this spell"
        );
        let mut visited = HashSet::new();
        assert!(!world.reachable(&goal, &mut visited));
        let flame = g
            .state
            .objects
            .iter()
            .position(|o| o.card == Card::from_name("Flamebraider").unwrap())
            .unwrap() as CardId;
        checked(
            &mut g,
            Action::ActivateMana {
                card: flame,
                ability: 0,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production: [2, 0, 0, 0, 0, 0],
            },
        );
        // Also reject a prefunded eligible white plus plentiful restricted
        // white, after the only white-producing Druid has been used.
        let druid = g
            .state
            .objects
            .iter()
            .position(|o| o.card == Card::from_name("Great Forest Druid").unwrap())
            .unwrap();
        g.state.objects[druid].tapped = true;
        g.state.players[p.index()].mana[0] += 1;
        let before = g.snapshot();
        let revision = g.revision();
        assert!(
            !g.legal_actions()
                .contains(&Action::AnnounceCast { card: spell })
        );
        let mut branch = {
            let parent = g.clone();
            parent.clone()
        };
        assert_eq!(
            branch.apply(p, Action::AnnounceCast { card: spell }),
            Err(Error::IllegalAction)
        );
        assert_eq!(branch.snapshot(), before);
        assert_eq!(branch.revision(), revision);
        assert_eq!(
            g.apply(p, Action::AnnounceCast { card: spell }),
            Err(Error::IllegalAction)
        );
        assert_eq!(g.snapshot(), before);
    }
}

#[test]
fn restricted_white_converter_fallback_keeps_actual_double_white_payment() {
    for p in [Player::P0, Player::P1] {
        let (mut g, _, _) = position(p, true);
        for object in &mut g.state.objects {
            if object.zone == Zone::Battlefield
                && object.controller == p
                && object.card != Card::from_name("Flamebraider").unwrap()
            {
                object.tapped = true;
            }
        }
        let spell = named(&mut g, p, "Kinbinding", Zone::Hand);
        g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 3];
        let flame = g
            .state
            .objects
            .iter()
            .position(|o| o.card == Card::from_name("Flamebraider").unwrap())
            .unwrap() as CardId;
        checked(
            &mut g,
            Action::ActivateMana {
                card: flame,
                ability: 0,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production: [2, 0, 0, 0, 0, 0],
            },
        );
        let (world, goal) = world_for(&g, p, spell);
        assert!(world.reachable(&goal, &mut HashSet::new()));
        checked(&mut g, Action::AnnounceCast { card: spell });
        for _ in 0..20 {
            if g.state.announcing.is_none() {
                break;
            }
            let actions = g.legal_actions();
            let action = actions
                .iter()
                .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
                .or_else(|| {
                    actions
                        .iter()
                        .find(|a| matches!(a, Action::TapMana { .. } | Action::ActivateMana { .. }))
                })
                .expect("converter has a legal next action")
                .clone();
            checked(&mut g, action);
        }
        assert!(g.state.announcing.is_none());
        settle(&mut g);
        assert_eq!(g.state.objects[spell as usize].zone, Zone::Battlefield);
    }
}

#[test]
fn scaling_creatures_and_fixed_ability_removal_preserve_white_mana_bound() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        for _ in 0..4 {
            named(&mut g, p, "Forest", Zone::Battlefield);
            named(&mut g, p, "Swamp", Zone::Battlefield);
        }
        named(&mut g, p.other(), "Moon-Vigil Adherents", Zone::Battlefield);
        let abigale = named(
            &mut g,
            p.other(),
            "Abigale, Eloquent First-Year",
            Zone::Battlefield,
        );
        let blanked = named(&mut g, p, "Great Forest Druid", Zone::Battlefield);
        let live = named(&mut g, p, "Great Forest Druid", Zone::Battlefield);
        g.state.objects[live as usize].tapped = true;
        g.remove_abilities_exact(
            &[g.object_ref(blanked)],
            g.object_ref(abigale),
            p.other(),
            AnimationDuration::Permanent,
        );
        g.state.players[p.index()].mana = [0; 6];
        let rhys = named(&mut g, p, "Rhys, the Evermore", Zone::Hand);
        let eirdu = named(&mut g, p, "Eirdu, Carrier of Dawn", Zone::Hand);

        let (world, goal) = world_for(&g, p, rhys);
        let mut visited = HashSet::new();
        assert!(!world.reachable(&goal, &mut visited));
        assert!(visited.is_empty());
        assert!(
            !g.legal_actions()
                .contains(&Action::AnnounceCast { card: rhys })
        );

        g.state.objects[live as usize].tapped = false;
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceCast { card: rhys })
        );
        let (world, goal) = world_for(&g, p, eirdu);
        let mut visited = HashSet::new();
        assert_eq!(goal.required[0], 2);
        assert!(!world.reachable(&goal, &mut visited));
        assert!(visited.is_empty());
        assert!(
            !g.legal_actions()
                .contains(&Action::AnnounceCast { card: eirdu })
        );
        named(&mut g, p, "Plains", Zone::Battlefield);
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceCast { card: eirdu })
        );
    }
}

#[test]
fn end_blaze_keeps_a_bounded_x_search_after_abigale_removes_abilities() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        for name in [
            "Island", "Island", "Island", "Swamp", "Swamp", "Swamp", "Mountain", "Mountain",
        ] {
            named(&mut g, p, name, Zone::Battlefield);
        }
        let abigale = named(
            &mut g,
            p.other(),
            "Abigale, Eloquent First-Year",
            Zone::Battlefield,
        );
        let seeker = named(&mut g, p, "Soulbright Seeker", Zone::Battlefield);
        g.remove_abilities_exact(
            &[g.object_ref(seeker)],
            g.object_ref(abigale),
            p.other(),
            AnimationDuration::Permanent,
        );
        g.state.players[p.index()].mana = [0; 6];
        let spell = named(&mut g, p, "End-Blaze Epiphany", Zone::Hand);

        let (world, mut goal) = world_for(&g, p, spell);
        goal.generic = 8;
        let mut visited = HashSet::new();
        assert!(!world.reachable(&goal, &mut visited));
        assert!(
            visited.is_empty(),
            "the ninth mana is ruled out before search"
        );

        checked(&mut g, Action::AnnounceCast { card: spell });
        let before = g.snapshot();
        let revision = g.revision();
        assert_eq!(
            g.apply(p, Action::ChooseX { value: 8 }),
            Err(Error::IllegalAction)
        );
        assert_eq!(g.snapshot(), before);
        assert_eq!(g.revision(), revision);
        checked(&mut g, Action::ChooseX { value: 7 });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(abigale),
            },
        );
    }
}

#[test]
fn absent_external_tap_witness_prunes_many_meek_attacks_but_fresh_creature_funds_actual_cost() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        for _ in 0..8 {
            named(&mut g, p, "Forest", Zone::Battlefield);
            named(&mut g, p, "Island", Zone::Battlefield);
        }
        let mut meeks = Vec::new();
        for _ in 0..12 {
            meeks.push(named(&mut g, p, "Meek Attack", Zone::Battlefield));
        }
        let drum = named(&mut g, p, "Springleaf Drum", Zone::Battlefield);
        g.state.players[p.index()].mana = [0; 6];
        let meek = meeks[0];
        let goal = FundingGoal {
            actor: p,
            required: [0, 0, 0, 1, 0, 0],
            generic: 1,
            reserve: None,
            source: Some((
                g.object_ref(meek),
                0,
                Counters::default(),
                g.state.objects[meek as usize].card,
                None,
            )),
        };
        let world = FundingState::from_game(&g, &goal);
        let mut visited = HashSet::new();
        assert!(!world.reachable(&goal, &mut visited));
        assert!(
            visited.is_empty(),
            "no creature can pay Drum's separate tap cost"
        );
        let actions = g.legal_actions();
        assert!(
            !actions.iter().any(
                |a| matches!(a, Action::AnnounceActivate { card, .. } if meeks.contains(card))
            )
        );
        let before = g.snapshot();
        let action = Action::AnnounceActivate {
            card: meek,
            ability: 0,
            counter_payment: Counters::default(),
        };
        let mut child = {
            let parent = g.clone();
            parent.clone()
        };
        assert_eq!(child.apply(p, action.clone()), Err(Error::IllegalAction));
        assert_eq!(child.snapshot(), before);
        assert_eq!(g.apply(p, action.clone()), Err(Error::IllegalAction));
        assert_eq!(g.snapshot(), before);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.state.objects[bear as usize].summoning_sick = true;
        checked(&mut g, action);
        for _ in 0..20 {
            if g.state.announcing.is_none() {
                break;
            }
            let actions = g.legal_actions();
            let action = actions
                .iter()
                .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
                .or_else(|| {
                    actions.iter().find(|a| {
                        matches!(
                            a,
                            Action::TapMana { .. }
                                | Action::ActivateMana { .. }
                                | Action::AnnounceMana { .. }
                                | Action::ChooseAdditionalCost { .. }
                                | Action::ChooseMana { .. }
                        )
                    })
                })
                .expect("fresh creature supplies Drum witness")
                .clone();
            checked(&mut g, action);
        }
        assert!(g.state.announcing.is_none());
        assert!(g.state.objects[drum as usize].tapped);
        assert!(g.state.objects[bear as usize].tapped);
        assert!(g.state.objects[bear as usize].summoning_sick);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}

#[test]
fn wickermaw_recoloring_tender_funds_paid_double_blue_counterspell() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p.other());
        let victim = named(&mut g, p.other(), "Divination", Zone::Hand);
        cast(&mut g, victim, None, &[]);
        checked(&mut g, Action::Pass);
        let tender = named(&mut g, p, "Bloom Tender", Zone::Battlefield);
        g.state.objects[tender as usize].summoning_sick = false;
        let wicker = named(&mut g, p, "Foraging Wickermaw", Zone::Battlefield);
        let spell = named(&mut g, p, "Wild Unraveling", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        assert_eq!(g.colors(wicker), 0);
        // The only initial colored permanent is the green Tender. The spell
        // in hand/stack cannot contribute blue to its production.
        assert_eq!(g.colors(tender), 16);
        let (world, mut goal) = world_for(&g, p, spell);
        goal.generic = 1;
        assert!(world.optimistic(&goal));
        assert!(world.reachable(&goal, &mut HashSet::new()));
        let mut tapped_control = g.clone();
        tapped_control.state.objects[tender as usize].tapped = true;
        assert!(
            !tapped_control
                .legal_actions()
                .contains(&Action::AnnounceCast { card: spell })
        );
        let before = tapped_control.snapshot();
        let revision = tapped_control.revision();
        assert_eq!(
            tapped_control.apply(p, Action::AnnounceCast { card: spell }),
            Err(Error::IllegalAction)
        );
        assert_eq!(tapped_control.snapshot(), before);
        assert_eq!(tapped_control.revision(), revision);
        checked(&mut g, Action::AnnounceCast { card: spell });
        checked(&mut g, Action::ChooseSpellCost { cost: 1 });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Spell(victim),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            checked(&mut g, Action::FinishTargets);
        }
        let unpaid = g.snapshot();
        let revision = g.revision();
        assert_eq!(
            g.apply(
                p,
                Action::PayCosts {
                    payment: [0, 2, 0, 0, 1, 0]
                }
            ),
            Err(Error::IllegalAction)
        );
        assert_eq!(g.snapshot(), unpaid);
        assert_eq!(g.revision(), revision);
        checked(
            &mut g,
            Action::ActivateMana {
                card: wicker,
                ability: 0,
                payment: [0, 0, 0, 0, 0, 1],
                counter_payment: Counters::default(),
                production: [0, 1, 0, 0, 0, 0],
            },
        );
        assert_eq!(g.colors(wicker), 2);
        checked(
            &mut g,
            Action::ActivateMana {
                card: tender,
                ability: 0,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production: [0, 1, 0, 0, 1, 0],
            },
        );
        checked(
            &mut g,
            Action::PayCosts {
                payment: [0, 2, 0, 0, 1, 0],
            },
        );
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.state.objects[tender as usize].counters.minus_one, 0);
        assert_eq!(g.state.objects[wicker as usize].counters.minus_one, 0);
        let library = g
            .state
            .objects
            .iter()
            .filter(|o| o.owner == p.other() && o.zone == Zone::Library)
            .count();
        settle(&mut g);
        assert_eq!(g.state.objects[victim as usize].zone, Zone::Graveyard);
        assert_eq!(g.state.objects[spell as usize].zone, Zone::Graveyard);
        assert_eq!(
            g.state
                .objects
                .iter()
                .filter(|o| o.owner == p.other() && o.zone == Zone::Library)
                .count(),
            library
        );
    }
}
