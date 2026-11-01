//! Focused printed outcomes; shared engine invariants live in core tests.
//! Wild Ride's ordinary spell effect; harmonize tests require its shared cast path.
use super::big_helpers::{cast as paid_cast, settle, setup};
use super::*;

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
    {
        let p = Player::P0;
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
fn harmonize_can_decline_tapping_and_is_exiled_if_countered() {
    {
        let p = Player::P0;
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
fn harmonize_and_casualty_can_use_the_same_creature_in_order() {
    {
        let p = Player::P0;
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
fn harmonize_proposal_includes_burst_lightnings_printed_kicker() {
    {
        let p = Player::P0;
        let mut g = setup(p);
        let helper = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let spell = put(&mut g, p, "Wild Ride", Zone::Graveyard);
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 2];
        big_helpers::checked(&mut g, Action::AnnounceHarmonize { card: spell });
        let mut proposal = g.state.announcing.as_ref().unwrap().clone();
        // Compose the existing printed {4} kicker with the Harmonize cost in
        // an owned proposal; no currently supported printed card has both.
        proposal
            .item
            .set_source_definition(Card::from_name("Burst Lightning").unwrap());
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
