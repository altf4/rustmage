//! Namor CDA follows borrowed actual/funding worlds without granting mana abilities.
use super::*;
use crate::engine::assurance::big_helpers::{cast, named, settle, setup};
fn goal(actor: Player) -> FundingGoal {
    FundingGoal {
        actor,
        required: [0; 6],
        generic: 0,
        reserve: None,
        source: None,
    }
}
#[test]
fn msh_namor_funding_virtual_merfolk_departure_matches_actual_view_and_mana_quantity() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let n = named(&mut g, p, "Namor the Sub-Mariner", Zone::Battlefield);
        let w = named(&mut g, p, "Barkform Harvester", Zone::Battlefield);
        let before = g.snapshot();
        let goal = goal(p);
        let mut world = FundingState::from_game(&g, &goal);
        world.prepare_entry_projection();
        assert_eq!(world.power(n), 2);
        let mut actual = {
            let parent = g.clone();
            parent.clone()
        };
        actual.move_object(w, Zone::Graveyard);
        let expected = actual.object_view(n);
        let mut calls = 0;
        world.cost_departures(&[g.object_ref(w)], Zone::Graveyard, &mut |after| {
            calls += 1;
            assert_eq!(after.power(n), 1);
            assert_eq!(
                characteristic_query::CharacteristicQuery::in_world(after).object_view(n),
                expected
            );
            assert_eq!(
                ManaProduction::SourcePower { color: 1 }.in_world(after, n),
                ManaProduction::Fixed([0, 1, 0, 0, 0, 0])
            );
            false
        });
        assert_eq!(calls, 1);
        assert_eq!(g.snapshot(), before);
        assert!(
            Card::from_name("Namor the Sub-Mariner")
                .unwrap()
                .activations()
                .is_empty()
        );
    }
}
#[test]
fn msh_namor_funding_projected_entry_counts_new_self_without_mutating_parent() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let n = named(&mut g, p, "Namor the Sub-Mariner", Zone::Hand);
        named(&mut g, p, "Barkform Harvester", Zone::Battlefield);
        let before = g.snapshot();
        let goal = goal(p);
        let w = FundingState::from_game(&g, &goal);
        assert_eq!(w.power(n), 1);
        let movement = g.entry_move(n, Zone::Battlefield, false);
        let mut calls = 0;
        w.commit_projected_entries(vec![movement], &[0], &mut |next| {
            calls += 1;
            assert_eq!(next.power(n), 2);
            false
        });
        assert_eq!(calls, 1);
        assert_eq!(g.snapshot(), before);
    }
}
#[test]
fn msh_namor_funding_control_source_departure_uses_new_controller_merfolk() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let n = named(
            &mut g,
            p.other(),
            "Namor the Sub-Mariner",
            Zone::Battlefield,
        );
        g.create_tokens(p, Card::MshBlueMerfolkToken, 2, false);
        let aura = named(&mut g, p, "Confiscate", Zone::Hand);
        cast(&mut g, aura, None, &[Target::Card(n)]);
        settle(&mut g);
        assert_eq!(g.stats(n).power, 3);
        let before = g.snapshot();
        let goal = goal(p);
        let mut world = FundingState::from_game(&g, &goal);
        world.prepare_entry_projection();
        let mut actual = g.clone();
        actual.move_object(aura, Zone::Graveyard);
        let expected = actual.object_view(n);
        let mut calls = 0;
        world.cost_departures(&[g.object_ref(aura)], Zone::Graveyard, &mut |after| {
            calls += 1;
            assert_eq!(after.power(n), 1);
            assert_eq!(
                characteristic_query::CharacteristicQuery::in_world(after).object_view(n),
                expected
            );
            false
        });
        assert_eq!(calls, 1);
        assert_eq!(g.snapshot(), before);
    }
}
