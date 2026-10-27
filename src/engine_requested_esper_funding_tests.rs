//! Projected linked-copy expiry must use the borrowed world's mana value.
use super::*;
use crate::engine::ability_world::CharacteristicWorld;
use crate::engine::assurance::big_helpers::{named, setup};

#[test]
fn projected_source_sacrifice_restores_transformed_saga_mana_value() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let saga = named(&mut g, p, "Summon: Esper Maduin", Zone::Battlefield);
        let elf = named(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        let ring = named(&mut g, p, "Lotus Ring", Zone::Battlefield);
        assert!(g.attach_equipment(g.object_ref(ring), g.object_ref(elf)));
        g.apply_temporary_copy(
            g.object_ref(saga),
            CopyRecipe::new(Card::from_name("Summon: Esper Maduin").unwrap()),
            CopyDeadline::WhileSource {
                source: g.object_ref(elf),
            },
        );
        let filter = CountFilter {
            zone: CountZone::Battlefield,
            relation: 0,
            kind: 1,
            exclude: 0,
            subtype: Some("Saga"),
            basic: false,
            other: false,
            cards_only: false,
        };
        let goal = FundingGoal {
            actor: p,
            required: [0; 6],
            generic: 1,
            reserve: None,
            source: None,
        };
        let before = g.snapshot();
        let world = FundingState::from_game(&g, &goal);
        assert_eq!(world.mana_value(saga), 0);
        assert_eq!(world.value(StatValue::GreatestManaValue(filter), saga), 0);
        let step = world
            .moves(&goal)
            .into_iter()
            .find(|m| {
                world.resources[m.resource].id == elf && m.grantor == Some(g.object_ref(ring))
            })
            .unwrap();
        let mut after = None;
        world.visit_applied(&step, &mut |next| {
            after = Some(next.clone());
            true
        });
        let after = after.unwrap();
        assert_eq!(after.object(elf).zone, Zone::Graveyard);
        assert_eq!(after.mana_value(saga), 2);
        assert_eq!(after.value(StatValue::GreatestManaValue(filter), saga), 2);
        assert_eq!(
            characteristic_query::CharacteristicQuery::in_world(&after)
                .value(StatValue::GreatestManaValue(filter), saga),
            2
        );
        assert_eq!(g.snapshot(), before);
    }
}
