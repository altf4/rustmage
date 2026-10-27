//! Projected life payments update characteristic conditions without mutating the game.
use super::*;
use crate::engine::assurance::big_helpers::{named, setup};

#[test]
fn projected_life_payment_updates_channeler_condition_and_preserves_parent() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let channeler = named(&mut g, p, "Essence Channeler", Zone::Battlefield);
        let hall = named(&mut g, p, "Great Hall of the Biblioplex", Zone::Battlefield);
        let goal = FundingGoal {
            actor: p,
            required: [0; 6],
            generic: 1,
            reserve: None,
            source: None,
        };
        let before = g.snapshot();
        let world = FundingState::from_game(&g, &goal);
        assert_eq!(world.value(StatValue::LifeLostThisTurn, channeler), 0);
        assert!(
            !characteristic_query::CharacteristicQuery::in_world(&world)
                .keywords(channeler)
                .contains(Keywords::FLYING)
        );
        let zero_step = world
            .moves(&goal)
            .into_iter()
            .find(|m| world.resources[m.resource].id == hall && m.ability == Some(0))
            .expect("zero-life mana ability must be available");
        let mut zero = None;
        world.visit_applied(&zero_step, &mut |next| {
            zero = Some(next.clone());
            true
        });
        let zero = zero.expect("projected zero-life payment");
        assert!(!zero.lost_life(p));
        assert!(
            !characteristic_query::CharacteristicQuery::in_world(&zero)
                .keywords(channeler)
                .contains(Keywords::VIGILANCE)
        );
        let step = world
            .moves(&goal)
            .into_iter()
            .find(|m| world.resources[m.resource].id == hall && m.ability == Some(1))
            .expect("life-paying mana ability must be available");
        let mut after = None;
        world.visit_applied(&step, &mut |next| {
            after = Some(next.clone());
            true
        });
        let after = after.expect("projected life payment");
        assert_eq!(after.value(StatValue::LifeLostThisTurn, channeler), 1);
        let nested = [
            (StatValue::Not(&StatValue::LifeLostThisTurn), 0),
            (
                StatValue::AtLeast {
                    value: &StatValue::LifeLostThisTurn,
                    minimum: 1,
                },
                1,
            ),
            (
                StatValue::Sum(&[StatValue::Fixed(2), StatValue::LifeLostThisTurn]),
                3,
            ),
        ];
        let projected_query = characteristic_query::CharacteristicQuery::in_world(&after);
        for (value, expected) in nested {
            assert_eq!(after.value(value, channeler), expected);
            assert_eq!(projected_query.value(value, channeler), expected);
            assert_ne!(g.stat_value(value, g.object_ref(channeler), p), expected);
        }
        let query = characteristic_query::CharacteristicQuery::in_world(&after);
        assert!(query.keywords(channeler).contains(Keywords::FLYING));
        assert!(query.keywords(channeler).contains(Keywords::VIGILANCE));
        assert!(!after.lost_life(p.other()));
        assert_eq!(g.snapshot(), before);
        assert!(!g.keywords(channeler).contains(Keywords::FLYING));
    }
}

#[test]
fn unavailable_life_payment_does_not_change_projection_or_parent() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let channeler = named(&mut g, p, "Essence Channeler", Zone::Battlefield);
        let hall = named(&mut g, p, "Great Hall of the Biblioplex", Zone::Battlefield);
        g.state.players[p.index()].life = 0;
        let before = g.snapshot();
        let goal = FundingGoal {
            actor: p,
            required: [0; 6],
            generic: 1,
            reserve: None,
            source: None,
        };
        let world = FundingState::from_game(&g, &goal);
        assert!(
            !world
                .moves(&goal)
                .iter()
                .any(|m| { world.resources[m.resource].id == hall && m.ability == Some(1) })
        );
        assert!(!world.lost_life(p));
        assert_eq!(world.value(StatValue::LifeLostThisTurn, channeler), 0);
        assert_eq!(g.snapshot(), before);
    }
}
