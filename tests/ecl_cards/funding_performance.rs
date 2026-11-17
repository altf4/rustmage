//! Restricted mana stays exact at payment while declarations avoid future searches.
use crate::ecl_scenarios::{cast, checked, named, settle, setup};
use crate::support::admit;
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
    // An unrelated real attachment does not change mana eligibility.
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
        assert!(world.reachable(&goal, &mut visited));
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
            world.reachable(&goal, &mut HashSet::new()),
            "declaration admission does not spend restricted lots"
        );
        admit(&g, Action::AnnounceCast { card: aurora });
        admit(&g, Action::AnnounceCast { card: prism });
        let mut failed = g.clone();
        checked(&mut failed, Action::AnnounceCast { card: aurora });
        assert!(
            !failed
                .legal_actions()
                .iter()
                .any(|a| matches!(a, Action::PayCosts { .. }))
        );
        checked(&mut failed, Action::EndManaWindow);
        checked(&mut failed, Action::SubmitAttempt);
        let outcome = failed.last_attempt_outcome_view(p).unwrap();
        assert_eq!(outcome.outcome, AttemptOutcome::Failed);
        assert_eq!(outcome.reason, Some(AttemptFailureReason::UnsatisfiedMana));
        assert_eq!(failed.object_view(aurora).zone, Zone::Hand);
        assert_eq!(
            failed.state.players[p.index()].mana,
            g.state.players[p.index()].mana
        );
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
        let flame = g
            .state
            .objects
            .iter()
            .position(|o| o.card == Card::from_name("Flamebraider").unwrap())
            .unwrap() as CardId;
        let converter = g
            .state
            .objects
            .iter()
            .position(|o| o.card == Card::from_name("Foraging Wickermaw").unwrap())
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
        checked(
            &mut g,
            Action::ActivateMana {
                card: converter,
                ability: 0,
                payment: [0, 0, 0, 1, 0, 0],
                counter_payment: Counters::default(),
                production: [0, 0, 0, 0, 1, 0],
            },
        );
        assert_eq!(
            g.available_mana(
                p,
                g.spell_mana_purpose_for_card(p, aurora, g.state.objects[aurora as usize].card)
            ),
            [0, 0, 0, 0, 7, 0]
        );
        checked(
            &mut g,
            Action::PayCosts {
                payment: [0, 0, 0, 0, 7, 0],
            },
        );
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
        assert!(world.reachable(&goal, &mut visited));
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
        admit(&g, Action::AnnounceCast { card: spell });
        let mut failed = g.clone();
        checked(&mut failed, Action::AnnounceCast { card: spell });
        assert!(
            !failed
                .legal_actions()
                .iter()
                .any(|a| matches!(a, Action::PayCosts { .. }))
        );
        checked(&mut failed, Action::EndManaWindow);
        checked(&mut failed, Action::SubmitAttempt);
        let outcome = failed.last_attempt_outcome_view(p).unwrap();
        assert_eq!(outcome.outcome, AttemptOutcome::Failed);
        assert_eq!(outcome.reason, Some(AttemptFailureReason::UnsatisfiedMana));
        assert_eq!(failed.object_view(spell).zone, Zone::Hand);
        assert_eq!(
            failed.state.players[p.index()].mana,
            g.state.players[p.index()].mana
        );
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
        let converter = g
            .state
            .objects
            .iter()
            .position(|o| o.card == Card::from_name("Foraging Wickermaw").unwrap())
            .unwrap() as CardId;
        checked(
            &mut g,
            Action::ActivateMana {
                card: converter,
                ability: 0,
                payment: [1, 0, 0, 0, 0, 0],
                counter_payment: Counters::default(),
                production: [1, 0, 0, 0, 0, 0],
            },
        );
        // If the lot allocation is ambiguous, spend the restricted Flamebraider
        // lot on the Elemental's activation and retain ordinary white for the spell.
        while let Some(choice) = g.observe(p).mana_lot_spend {
            let quality = g.state.mana_lots.as_ref().unwrap().pools[p.index()][choice.lot].quality;
            let amount = if quality.eligible(mana_lots::Purpose::Other) {
                choice.minimum
            } else {
                choice.maximum
            };
            checked(&mut g, Action::ChooseManaLotSpend { amount });
        }
        assert_eq!(
            g.available_mana(
                p,
                g.spell_mana_purpose_for_card(p, spell, g.state.objects[spell as usize].card)
            ),
            [2, 0, 0, 0, 0, 3]
        );
        checked(
            &mut g,
            Action::PayCosts {
                payment: [2, 0, 0, 0, 0, 3],
            },
        );
        assert!(g.state.announcing.is_none());
        settle(&mut g);
        assert_eq!(g.state.objects[spell as usize].zone, Zone::Battlefield);
    }
}

#[test]
fn self_bound_mirrorform_copies_preserve_the_restricted_white_bound() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p.other());
        for _ in 0..4 {
            named(&mut g, p, "Forest", Zone::Battlefield);
            named(&mut g, p, "Swamp", Zone::Battlefield);
        }
        named(&mut g, p, "Great Forest Druid", Zone::Battlefield);
        let flame = named(&mut g, p, "Flamebraider", Zone::Battlefield);
        for _ in 0..2 {
            named(&mut g, p, "Flamebraider", Zone::Battlefield);
        }
        let copied = named(&mut g, p.other(), "Springleaf Drum", Zone::Battlefield);
        let mirrorform = named(&mut g, p.other(), "Mirrorform", Zone::Hand);
        cast(&mut g, mirrorform, None, &[Target::Card(flame)]);
        settle(&mut g);
        assert_eq!(g.object_view(copied).card, g.object_view(flame).card);

        g.state.active = p;
        g.state.priority = p;
        g.state.players[p.index()].mana = [0; 6];
        let kinbinding = named(&mut g, p, "Kinbinding", Zone::Hand);
        let (world, goal) = world_for(&g, p, kinbinding);
        let mut visited = HashSet::new();
        assert!(world.reachable(&goal, &mut visited));
        assert!(visited.is_empty());
        admit(&g, Action::AnnounceCast { card: kinbinding });
        named(&mut g, p, "Plains", Zone::Battlefield);
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceCast { card: kinbinding })
        );
    }
}

#[test]
fn celestial_reunion_x_counts_fixed_bloom_colors_and_ignores_an_opposing_land_bonus() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        for _ in 0..4 {
            named(&mut g, p, "Forest", Zone::Battlefield);
            named(&mut g, p, "Swamp", Zone::Battlefield);
        }
        named(&mut g, p, "Shadow Urchin", Zone::Battlefield);
        for _ in 0..2 {
            named(&mut g, p, "Bloom Tender", Zone::Battlefield);
        }
        let land = named(&mut g, p.other(), "Forest", Zone::Battlefield);
        let aura = named(&mut g, p.other(), "Shimmerwilds Growth", Zone::Battlefield);
        g.state.objects[aura as usize].attached_to = Some(g.object_ref(land));
        g.set_entry_value(
            g.object_ref(aura),
            EntryChoiceValue::Color(ChoiceColor::Blue),
        );
        assert_eq!(g.object_view(land).colors, 2);
        g.state.players[p.index()].mana = [0; 6];
        let spell = named(&mut g, p, "Celestial Reunion", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: spell });
        checked(
            &mut g,
            Action::ChooseBeholdType {
                creature_type: Some(CreatureType::from_name("Druid").unwrap()),
            },
        );
        // Eight lands plus black, red and green from each Tender, with one
        // mana for the printed green symbol. Tapped Druids still count.
        admit(&g, Action::ChooseX { value: 14 });
        checked(&mut g, Action::ChooseX { value: 13 });
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
        assert!(world.reachable(&goal, &mut visited));
        assert!(visited.is_empty());
        admit(&g, Action::AnnounceCast { card: rhys });

        g.state.objects[live as usize].tapped = false;
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceCast { card: rhys })
        );
        let (world, goal) = world_for(&g, p, eirdu);
        let mut visited = HashSet::new();
        assert_eq!(goal.required[0], 2);
        assert!(world.reachable(&goal, &mut visited));
        assert!(visited.is_empty());
        admit(&g, Action::AnnounceCast { card: eirdu });
        named(&mut g, p, "Plains", Zone::Battlefield);
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceCast { card: eirdu })
        );
    }
}

#[test]
fn end_blaze_keeps_x_admission_after_abigale_removes_abilities() {
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
        assert!(world.reachable(&goal, &mut visited));
        assert!(
            visited.is_empty(),
            "declaration admission does not search for the ninth mana"
        );

        checked(&mut g, Action::AnnounceCast { card: spell });
        admit(&g, Action::ChooseX { value: 8 });
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
fn absent_external_tap_witness_allows_declaration_but_fresh_creature_pays_actual_cost() {
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
        assert!(world.reachable(&goal, &mut visited));
        assert!(
            visited.is_empty(),
            "no creature can pay Drum's separate tap cost"
        );
        for &meek in &meeks {
            admit(
                &g,
                Action::AnnounceActivate {
                    card: meek,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
        }
        let action = Action::AnnounceActivate {
            card: meek,
            ability: 0,
            counter_payment: Counters::default(),
        };
        let mut failed = g.clone();
        checked(&mut failed, action.clone());
        checked(&mut failed, Action::EndManaWindow);
        checked(&mut failed, Action::SubmitAttempt);
        let outcome = failed.last_attempt_outcome_view(p).unwrap();
        assert_eq!(outcome.outcome, AttemptOutcome::Failed);
        assert_eq!(outcome.reason, Some(AttemptFailureReason::UnsatisfiedMana));
        assert!(!failed.object_view(meek).tapped);
        assert!(!failed.object_view(drum).tapped);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.state.objects[bear as usize].summoning_sick = true;
        checked(&mut g, action);
        checked(
            &mut g,
            Action::AnnounceMana {
                card: drum,
                ability: 0,
                counter_payment: Counters::default(),
                production: [0, 0, 0, 1, 0, 0],
            },
        );
        let object = g.object_ref(bear);
        checked(&mut g, Action::ChooseAdditionalCost { object });
        checked(&mut g, Action::PayCosts { payment: [0; 6] });
        let forest = g
            .state
            .objects
            .iter()
            .position(|o| {
                o.card == Card::Forest && o.zone == Zone::Battlefield && o.controller == p
            })
            .unwrap() as CardId;
        checked(&mut g, Action::TapMana { card: forest });
        checked(
            &mut g,
            Action::PayCosts {
                payment: [0, 0, 0, 1, 1, 0],
            },
        );
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
        admit(&tapped_control, Action::AnnounceCast { card: spell });
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
