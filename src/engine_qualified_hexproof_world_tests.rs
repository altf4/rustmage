//! The same earlier-layer expressions must see speculative costs, not actual state.
use super::super::ability_timeline_tests::setup;
use super::super::ability_world::CharacteristicWorld;
use super::super::tests::spawn;
use super::*;
use crate::cards::ChoiceColor;
#[test]
fn qualified_hexproof_funding_uses_virtual_color_presence_and_life() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let grace = spawn(
            &mut g,
            p,
            Card::from_name("Knight of Grace").unwrap(),
            Zone::Battlefield,
        );
        let elenda = spawn(
            &mut g,
            p,
            Card::from_name("Elenda, Saint of Dusk").unwrap(),
            Zone::Battlefield,
        );
        let black = spawn(
            &mut g,
            p,
            Card::from_name("Vampire Nighthawk").unwrap(),
            Zone::Battlefield,
        );
        let goal = FundingGoal {
            actor: p,
            required: [0; 6],
            generic: 0,
            reserve: Some((
                g.object_ref(black),
                Card::from_name("Eaten Alive")
                    .unwrap()
                    .additional_cost()
                    .unwrap(),
            )),
            source: None,
        };
        let original = g.snapshot();
        let mut world = FundingState::from_game(&g, &goal);
        let has_black = StatValue::HasColor {
            filter: CountFilter {
                zone: CountZone::Battlefield,
                relation: -1,
                kind: 0,
                exclude: 0,
                subtype: None,
                basic: false,
                other: false,
                cards_only: false,
            },
            color: ChoiceColor::Black,
        };
        assert_eq!(world.value(has_black, grace), 1);
        // Elenda also counts as a black permanent; both virtual departures are necessary.
        for id in [black, elenda] {
            world
                .resources
                .iter_mut()
                .find(|r| r.id == id)
                .unwrap()
                .object
                .zone = Zone::Graveyard;
        }
        assert_eq!(world.value(has_black, grace), 0);
        world
            .resources
            .iter_mut()
            .find(|r| r.id == elenda)
            .unwrap()
            .object
            .zone = Zone::Battlefield;
        world.life = STARTING_LIFE + 1;
        let resource = world.resources.iter().find(|r| r.id == elenda).unwrap();
        assert!(world.keywords(resource).contains(Keywords::MENACE));
        assert_eq!(
            world.value(
                StatValue::ControllerLifeAboveStarting { minimum: 10 },
                elenda
            ),
            0
        );
        world.life = STARTING_LIFE + 10;
        assert_eq!(
            world.value(
                StatValue::ControllerLifeAboveStarting { minimum: 10 },
                elenda
            ),
            1
        );
        world.life = STARTING_LIFE;
        let resource = world.resources.iter().find(|r| r.id == elenda).unwrap();
        assert!(!world.keywords(resource).contains(Keywords::MENACE));
        assert_eq!(g.snapshot(), original);
        assert_eq!(g.stats(grace).power, 3);
        assert_eq!(g.stats(elenda).power, 4);
    }
}
