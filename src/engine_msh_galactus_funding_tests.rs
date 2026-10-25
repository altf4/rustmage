//! New shield bonuses share exact borrowed-world departure semantics.
use super::*;
use crate::engine::assurance::big_helpers::{named, setup};
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
fn msh_galactus_funding_shield_virtual_departure_matches_actual_full_view() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = named(&mut g, p, "U.S.Agent, John Walker", Zone::Battlefield);
        g.create_tokens(p, Card::MshSturdyShieldToken, 1, false);
        let sh = g.state.objects.len() as CardId - 1;
        assert!(g.attach_equipment(g.object_ref(sh), g.object_ref(s)));
        let before = g.snapshot();
        let goal = goal(p);
        let mut w = FundingState::from_game(&g, &goal);
        w.prepare_entry_projection();
        assert_eq!(w.power(s), 4);
        let mut actual = {
            let parent = g.clone();
            parent.clone()
        };
        actual.move_object(sh, Zone::Graveyard);
        let expected = actual.object_view(s);
        let mut calls = 0;
        w.cost_departures(&[g.object_ref(sh)], Zone::Graveyard, &mut |next| {
            calls += 1;
            assert_eq!(next.power(s), 3);
            assert_eq!(
                characteristic_query::CharacteristicQuery::in_world(next).object_view(s),
                expected
            );
            false
        });
        assert_eq!(calls, 1);
        assert_eq!(g.snapshot(), before);
    }
}
#[test]
fn msh_galactus_funding_recipient_departure_does_not_rebind_equipment_bonus() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = named(&mut g, p, "U.S.Agent, John Walker", Zone::Battlefield);
        g.create_tokens(p, Card::MshSturdyShieldToken, 1, false);
        let sh = g.state.objects.len() as CardId - 1;
        assert!(g.attach_equipment(g.object_ref(sh), g.object_ref(s)));
        let before = g.snapshot();
        let goal = goal(p);
        let mut w = FundingState::from_game(&g, &goal);
        w.prepare_entry_projection();
        let mut actual = g.clone();
        actual.move_object(s, Zone::Graveyard);
        let expected = actual.object_view(s);
        let mut calls = 0;
        w.cost_departures(&[g.object_ref(s)], Zone::Graveyard, &mut |next| {
            calls += 1;
            assert_eq!(next.power(s), 3);
            assert_eq!(
                characteristic_query::CharacteristicQuery::in_world(next).object_view(s),
                expected
            );
            false
        });
        assert_eq!(calls, 1);
        assert_eq!(g.snapshot(), before);
    }
}
