//! Legendary CDA counts follow current speculative control/type/copy worlds.
use super::super::assurance::big_helpers::{cast, named, settle, setup};
use super::super::msh_super_adaptoid_tests::paid;
use super::*;
#[test]
fn msh_adaptoid_funding_virtual_legendary_departure_recomputes_without_parent_mutation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let donor = named(&mut g, p, "Thanos, the Mad Titan", Zone::Battlefield);
        let id = paid(&mut g, p, Some(donor));
        let goal = FundingGoal {
            actor: p,
            required: [0; 6],
            generic: 0,
            reserve: None,
            source: None,
        };
        let before = g.snapshot();
        let views = [Player::P0, Player::P1].map(|q| serde_json::to_value(g.observe(q)).unwrap());
        let mut world = FundingState::from_game(&g, &goal);
        world.prepare_entry_projection();
        assert_eq!(
            characteristic_query::CharacteristicQuery::in_world(&world)
                .object_view(id)
                .power,
            2
        );
        let mut visits = 0;
        world.cost_departures(&[g.object_ref(donor)], Zone::Graveyard, &mut |departed| {
            visits += 1;
            let view =
                characteristic_query::CharacteristicQuery::in_world(departed).object_view(id);
            assert_eq!((view.power, view.toughness), (1, 2));
            false
        });
        assert_eq!(visits, 1);
        assert_eq!(g.snapshot(), before);
        assert_eq!(
            [Player::P0, Player::P1].map(|q| serde_json::to_value(g.observe(q)).unwrap()),
            views
        );
        let spell = named(&mut g, p, "Murder", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 2, 0, 0, 1];
        cast(&mut g, spell, None, &[Target::Card(donor)]);
        settle(&mut g);
        assert_eq!(
            g.stats(id),
            Stats {
                power: 1,
                toughness: 2
            }
        );
    }
}
