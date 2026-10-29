//! Wild Ride's ordinary spell effect; harmonize tests require its shared cast path.
use super::big_helpers::{cast as paid_cast, settle, setup};
use super::*;
use crate::cards::{AnimationCreatureTypes, AnimationDuration, AnimationSpec};

fn choose_clause(g: &mut Game, clause: CastCostClause) {
    let index = g
        .state
        .announcing
        .as_ref()
        .unwrap()
        .cast_costs
        .as_ref()
        .unwrap()
        .clauses
        .iter()
        .position(|row| row.clause == clause && !row.paid)
        .unwrap();
    big_helpers::checked(
        g,
        Action::ChooseCastCost {
            clause: index as u16,
        },
    );
}

#[test]
fn wild_ride_paid_spell_boosts_and_grants_haste_until_end_of_turn() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let creature = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let spell = put(&mut g, p, "Wild Ride", Zone::Hand);
        let before = g.stats(creature);
        paid_cast(&mut g, spell, None, &[Target::Card(creature)]);
        settle(&mut g);
        assert_eq!(g.stats(creature).power, before.power + 3);
        assert_eq!(g.stats(creature).toughness, before.toughness);
        assert!(g.keywords(creature).contains(Keywords::HASTE));
        assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
        big_helpers::next_turn(&mut g);
        assert_eq!(g.stats(creature), before);
        assert!(!g.keywords(creature).contains(Keywords::HASTE));
    }
}

#[test]
fn harmonize_contribution_uses_current_power_and_exact_untapped_creature() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let creature = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        let land = put(&mut g, p, "Forest", Zone::Battlefield);
        let r = g.object_ref(creature);
        assert_eq!(harmonize::contribution(&g, p, r), Some(2));
        assert_eq!(harmonize::contribution(&g, p, g.object_ref(enemy)), None);
        assert_eq!(harmonize::contribution(&g, p, g.object_ref(land)), None);
        g.state.objects[creature as usize].tapped = true;
        assert_eq!(harmonize::contribution(&g, p, r), None);
        g.state.objects[creature as usize].tapped = false;
        g.state.objects[creature as usize].growth.power = -5;
        assert_eq!(harmonize::contribution(&g, p, r), Some(0));
        g.move_object(creature, Zone::Graveyard);
        assert_eq!(harmonize::contribution(&g, p, r), None);
        let cost = ManaCost {
            colored: [0, 0, 0, 1, 0, 0],
            generic: 4,
        };
        assert_eq!(harmonize::reduce(cost, 8), ManaCost { generic: 0, ..cost });
    }
}

#[test]
fn wild_ride_harmonize_pays_with_a_single_creature_and_exiles() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let helper = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let target = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, p, "Wild Ride", Zone::Graveyard);
        // A freshly controlled creature is still eligible: tapping is a cost,
        // not its own {T} ability.
        g.state.objects[helper as usize].summoning_sick = true;
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 2];
        let reference = g.object_ref(helper);
        big_helpers::checked(&mut g, Action::AnnounceHarmonize { card: spell });
        big_helpers::checked(
            &mut g,
            Action::ChooseHarmonizer {
                object: Some(reference),
            },
        );
        big_helpers::checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        assert!(!g.state.objects[helper as usize].tapped);
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        big_helpers::checked(&mut g, pay);
        choose_clause(&mut g, CastCostClause::Harmonize(reference));
        assert!(g.state.objects[helper as usize].tapped);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        settle(&mut g);
        assert_eq!(g.object_view(spell).zone, Zone::Exile);
        assert_eq!(g.stats(target).power, 5);
        assert!(g.keywords(target).contains(Keywords::HASTE));
    }
}

#[test]
fn harmonize_can_decline_tapping_and_is_exiled_if_countered() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let target = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let spell = put(&mut g, p, "Wild Ride", Zone::Graveyard);
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 4];
        big_helpers::checked(&mut g, Action::AnnounceHarmonize { card: spell });
        big_helpers::checked(&mut g, Action::ChooseHarmonizer { object: None });
        big_helpers::checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        big_helpers::checked(&mut g, pay);
        assert!(!g.state.objects[target as usize].tapped);
        big_helpers::checked(&mut g, Action::Pass);
        let counter = put(&mut g, p.other(), "Cancel", Zone::Hand);
        paid_cast(&mut g, counter, None, &[Target::Spell(spell)]);
        settle(&mut g);
        assert_eq!(g.object_view(spell).zone, Zone::Exile);
        assert_eq!(g.stats(target).power, 2);
    }
}

#[test]
fn harmonize_cannot_use_same_creature_for_mana_and_discount() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let elf = put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        let spell = put(&mut g, p, "Wild Ride", Zone::Graveyard);
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 2];
        assert!(
            !g.legal_actions()
                .contains(&Action::AnnounceHarmonize { card: spell })
        );
        g.state.players[p.index()].mana[5] = 3;
        big_helpers::checked(&mut g, Action::AnnounceHarmonize { card: spell });
        let reference = g.object_ref(elf);
        big_helpers::checked(
            &mut g,
            Action::ChooseHarmonizer {
                object: Some(reference),
            },
        );
        big_helpers::checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(elf),
            },
        );
        assert!(
            !g.legal_actions().iter().any(|action| matches!(action,
            Action::ActivateMana { card, .. } | Action::AnnounceMana { card, .. } if *card == elf))
        );
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        big_helpers::checked(&mut g, pay);
        choose_clause(&mut g, CastCostClause::Harmonize(reference));
        assert!(g.state.objects[elf as usize].tapped);
        settle(&mut g);
        assert_eq!(g.object_view(spell).zone, Zone::Exile);
    }
}

#[test]
fn harmonize_and_casualty_can_use_the_same_creature_in_order() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        put(&mut g, p, "Silverquill, the Disputant", Zone::Battlefield);
        let helper = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, p, "Wild Ride", Zone::Graveyard);
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 2];
        let reference = g.object_ref(helper);
        big_helpers::checked(&mut g, Action::AnnounceHarmonize { card: spell });
        big_helpers::checked(&mut g, Action::ChooseCasualty { pay: true });
        big_helpers::checked(
            &mut g,
            Action::ChooseHarmonizer {
                object: Some(reference),
            },
        );
        big_helpers::checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        big_helpers::checked(&mut g, pay);
        choose_clause(&mut g, CastCostClause::Harmonize(reference));
        assert!(g.state.objects[helper as usize].tapped);
        choose_clause(&mut g, CastCostClause::Casualty(0));
        big_helpers::checked(&mut g, Action::PayCasualty { object: reference });
        assert_eq!(g.object_view(helper).zone, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.object_view(spell).zone, Zone::Exile);
    }
}

#[test]
fn harmonize_foreign_replacement_can_force_full_cost_rollback() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let aura = put(
            &mut g,
            p.other(),
            "Imprisoned in the Moon",
            Zone::Battlefield,
        );
        g.state.objects[aura as usize].attached_to = Some(g.object_ref(enemy));
        let prison = put(&mut g, p, "Banishing Light", Zone::Hand);
        paid_cast(&mut g, prison, None, &[]);
        for _ in 0..40 {
            if g.state.pending != Pending::Priority {
                break;
            }
            big_helpers::checked(&mut g, Action::Pass);
        }
        big_helpers::checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(aura),
            },
        );
        settle(&mut g);
        assert_eq!(g.object_view(aura).zone, Zone::Exile);
        g.change_object_form(
            g.object_ref(prison),
            AnimationSpec {
                artifact: false,
                creature_types: AnimationCreatureTypes::Unchanged,
                base: Some((1, 1)),
                keywords: Keywords::NONE,
                duration: AnimationDuration::EndOfTurn,
            },
            characteristic_layers::AnimationTypeMode::AddCreature,
        );
        put(&mut g, p, "Silverquill, the Disputant", Zone::Battlefield);
        let helper = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let spell = put(&mut g, p, "Wild Ride", Zone::Graveyard);
        let before = g.snapshot();
        let reference = g.object_ref(helper);
        big_helpers::checked(&mut g, Action::AnnounceHarmonize { card: spell });
        big_helpers::checked(&mut g, Action::ChooseCasualty { pay: true });
        big_helpers::checked(
            &mut g,
            Action::ChooseHarmonizer {
                object: Some(reference),
            },
        );
        big_helpers::checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        big_helpers::checked(&mut g, pay);
        choose_clause(&mut g, CastCostClause::Casualty(0));
        let prison_ref = g.object_ref(prison);
        big_helpers::checked(&mut g, Action::PayCasualty { object: prison_ref });
        assert_eq!(g.actor(), Some(p.other()));
        big_helpers::checked(&mut g, Action::ChooseAuraAttachment { object: reference });
        assert_eq!(g.snapshot(), before);
        assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
        assert!(!g.object_view(helper).tapped);
        assert_eq!(g.object_view(prison).zone, Zone::Battlefield);
    }
}

#[test]
fn harmonize_funding_includes_the_declared_mana_surcharge() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let helper = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let spell = put(&mut g, p, "Wild Ride", Zone::Graveyard);
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 2];
        big_helpers::checked(&mut g, Action::AnnounceHarmonize { card: spell });
        let mut proposal = g.state.announcing.as_ref().unwrap().clone();
        let reference = g.object_ref(helper);
        assert!(g.harmonize_choice_fundable(&proposal, Some(reference)));
        // A chosen equivalent may already include an announced generic surcharge.
        proposal.cost = Some(ManaCost {
            colored: [0, 0, 0, 1, 0, 0],
            generic: 5,
        });
        assert!(!g.harmonize_choice_fundable(&proposal, Some(reference)));
        g.state.players[p.index()].mana[5] += 1;
        assert!(g.harmonize_choice_fundable(&proposal, Some(reference)));
    }
}

#[test]
fn harmonize_proposal_includes_burst_lightnings_printed_kicker() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let helper = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let spell = put(&mut g, p, "Wild Ride", Zone::Graveyard);
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 2];
        big_helpers::checked(&mut g, Action::AnnounceHarmonize { card: spell });
        let mut proposal = g.state.announcing.as_ref().unwrap().clone();
        // Compose the existing printed {4} kicker with the Harmonize cost in
        // an owned proposal; no currently supported printed card has both.
        proposal.item.set_source_definition(Card::from_name("Burst Lightning").unwrap());
        proposal.item.cast.as_mut().unwrap().kicker = Some(true);
        proposal.cost = Some(ManaCost {
            colored: [0, 0, 0, 1, 0, 0],
            generic: 4,
        });
        let reference = g.object_ref(helper);
        assert!(!g.harmonize_choice_fundable(&proposal, Some(reference)));
        g.state.players[p.index()].mana[5] = 6;
        assert!(g.harmonize_choice_fundable(&proposal, Some(reference)));
    }
}

#[test]
fn harmonize_chooses_before_targets_locks_power_then_allows_tap_before_mana() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let helper = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let target = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, p, "Wild Ride", Zone::Graveyard);
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 2];
        let reference = g.object_ref(helper);
        big_helpers::checked(&mut g, Action::AnnounceHarmonize { card: spell });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        big_helpers::checked(
            &mut g,
            Action::ChooseHarmonizer {
                object: Some(reference),
            },
        );
        // Probe the determination boundary independently of the casting UI.
        g.state.objects[helper as usize].growth.power = 2;
        big_helpers::checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        assert_eq!(
            g.state
                .announcing
                .as_ref()
                .unwrap()
                .total_cost
                .unwrap()
                .generic,
            0
        );
        big_helpers::checked(&mut g, Action::BeginOrderedCosts);
        choose_clause(&mut g, CastCostClause::Harmonize(reference));
        assert!(g.state.objects[helper as usize].tapped);
        g.state.objects[helper as usize].growth.power = -5;
        choose_clause(&mut g, CastCostClause::Mana);
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        big_helpers::checked(&mut g, pay);
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 0, 0, 2]);
        settle(&mut g);
        assert_eq!(g.object_view(spell).zone, Zone::Exile);
    }
}
