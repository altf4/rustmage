//! Existing live modifiers share actual and speculative resource layers.
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
fn msh_turn_facts_funding_iron_virtual_artifact_sacrifice_matches_actual() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let iron = named(&mut g, p, "Iron Man, Master of Machines", Zone::Battlefield);
        let artifact = named(&mut g, p, "Hedron Archive", Zone::Battlefield);
        let before = g.snapshot();
        let goal = goal(p);
        let mut w = FundingState::from_game(&g, &goal);
        w.prepare_entry_projection();
        assert_eq!(w.power(iron), 2);
        let mut actual = g.clone();
        actual.move_object(artifact, Zone::Graveyard);
        let expected = actual.object_view(iron);
        let mut calls = 0;
        w.cost_departures(&[g.object_ref(artifact)], Zone::Graveyard, &mut |next| {
            calls += 1;
            assert_eq!(next.power(iron), 1);
            assert_eq!(
                characteristic_query::CharacteristicQuery::in_world(next).object_view(iron),
                expected
            );
            false
        });
        assert_eq!(calls, 1);
        assert_eq!(g.snapshot(), before);
    }
}
#[test]
fn msh_turn_facts_funding_avengers_virtual_lord_departure_matches_actual() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let hero = named(&mut g, p, "Kid Loki", Zone::Battlefield);
        let av = named(&mut g, p, "Avengers Assemble!", Zone::Hand);
        cast(&mut g, av, None, &[]);
        settle(&mut g);
        let before = g.snapshot();
        let goal = goal(p);
        let mut w = FundingState::from_game(&g, &goal);
        w.prepare_entry_projection();
        assert_eq!(w.power(hero), 3);
        let mut actual = g.clone();
        actual.move_object(av, Zone::Graveyard);
        let expected = actual.object_view(hero);
        let mut calls = 0;
        w.cost_departures(&[g.object_ref(av)], Zone::Graveyard, &mut |next| {
            calls += 1;
            assert_eq!(next.power(hero), 1);
            assert_eq!(
                characteristic_query::CharacteristicQuery::in_world(next).object_view(hero),
                expected
            );
            false
        });
        assert_eq!(calls, 1);
        assert_eq!(g.snapshot(), before);
    }
}
