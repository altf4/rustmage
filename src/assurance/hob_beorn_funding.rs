//! Land-count self setters must observe projected lands and recipient control.
use super::*;
use crate::engine::assurance::big_helpers::{named, setup};
#[test]
fn beorn_hospitality_funding_stats_use_projected_land_zone_control_and_owned_world() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let h = named(&mut g, p, "Beorn's Hospitality", Zone::Battlefield);
        let land = named(&mut g, p, "Forest", Zone::Battlefield);
        named(&mut g, p, "Forest", Zone::Battlefield);
        named(&mut g, p.other(), "Forest", Zone::Battlefield);
        let item = g.activation_item(p, h, 0);
        g.animate_source(
            &item,
            crate::cards::AnimationSpec {
                artifact: false,
                creature_types: crate::cards::AnimationCreatureTypes::Unchanged,
                base: None,
                keywords: Keywords::NONE,
                duration: crate::cards::AnimationDuration::Permanent,
            },
        );
        g.grant_permanent_self_ability(&item, SelfGrantedAbilityKind::ControllerLandStats);
        let goal = FundingGoal {
            actor: p,
            required: [0; 6],
            generic: 0,
            reserve: None,
            source: Some((
                g.object_ref(h),
                0,
                Counters::default(),
                Card::from_name("Beorn's Hospitality").unwrap(),
                None,
            )),
        };
        let world = FundingState::from_game(&g, &goal);
        assert_eq!(
            characteristic_query::CharacteristicQuery::in_world(&world)
                .stats(h)
                .power,
            2
        );
        let mut departed = world.clone();
        departed
            .resources
            .iter_mut()
            .find(|r| r.id == land)
            .unwrap()
            .object
            .zone = Zone::Graveyard;
        assert_eq!(
            characteristic_query::CharacteristicQuery::in_world(&departed)
                .stats(h)
                .power,
            1
        );
        let mut stolen = world.clone();
        stolen
            .resources
            .iter_mut()
            .find(|r| r.id == h)
            .unwrap()
            .object
            .controller = p.other();
        assert_eq!(
            characteristic_query::CharacteristicQuery::in_world(&stolen)
                .stats(h)
                .power,
            1
        );
        assert_eq!(
            characteristic_query::CharacteristicQuery::in_world(&world)
                .stats(h)
                .power,
            2
        );
        assert_eq!(g.stats(h).power, 2);
    }
}
