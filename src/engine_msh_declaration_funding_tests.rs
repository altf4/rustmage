//! Attuma's lord uses the same exact layer/filter rules in independent funding worlds.
use super::*;
use crate::engine::assurance::big_helpers::{cast, named, settle, setup};
fn power(world: &FundingState<'_>, id: CardId) -> i128 {
    characteristic_query::CharacteristicQuery::in_world(world)
        .stats(id)
        .power
}
#[test]
fn msh_declaration_funding_lord_departure_control_and_suppression_are_owned() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, "Attuma, Atlantean Warlord", Zone::Hand);
        cast(&mut g, source, None, &[]);
        settle(&mut g);
        let own = named(&mut g, p, "Atlantean Cavalry", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Atlantean Cavalry", Zone::Battlefield);
        let goal = FundingGoal {
            actor: p,
            required: [0; 6],
            generic: 0,
            reserve: None,
            source: None,
        };
        let before = g.snapshot();
        let mut world = FundingState::from_game(&g, &goal);
        world.prepare_entry_projection();
        assert_eq!(power(&world, own), 4);
        assert_eq!(power(&world, enemy), 3);
        let mut visits = 0;
        world.cost_departures(&[g.object_ref(source)], Zone::Graveyard, &mut |dead| {
            visits += 1;
            assert_eq!(power(dead, own), 3);
            assert_eq!(power(dead, enemy), 3);
            false
        });
        assert_eq!(visits, 1);
        let mut branch = world.clone();
        let index = branch
            .resources
            .iter()
            .position(|r| r.id == source)
            .unwrap();
        branch.resources[index].object.controller = p.other();
        assert_eq!(power(&branch, own), 3);
        assert_eq!(power(&branch, enemy), 4);
        assert_eq!(power(&world, own), 4);
        assert_eq!(g.snapshot(), before);
        drop(world);
        let moon = named(&mut g, p, "Imprisoned in the Moon", Zone::Hand);
        cast(&mut g, moon, None, &[Target::Card(source)]);
        settle(&mut g);
        let before = g.snapshot();
        let mut world = FundingState::from_game(&g, &goal);
        world.prepare_entry_projection();
        assert_eq!(power(&world, own), 3);
        world.cost_departures(&[g.object_ref(moon)], Zone::Graveyard, &mut |dead| {
            assert_eq!(power(dead, own), 4);
            false
        });
        assert_eq!(g.snapshot(), before);
    }
}
