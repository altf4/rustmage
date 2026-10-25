//! Borrowed counter reservations and virtual departures match actual current characteristics.
use super::super::assurance::big_helpers::setup;
use super::super::shield_harness_tests::paid;
use super::*;
#[test]
fn msh_shield_harness_funding_counter_removal_and_source_departure_do_not_mutate_parent() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let cap = paid(&mut g, p, "Captain America, Super-Soldier", &[]);
        let hero = paid(&mut g, p, "Hawkeye, Young Avenger", &[]);
        let goal = FundingGoal {
            actor: p,
            required: [0; 6],
            generic: 0,
            reserve: None,
            source: None,
        };
        let before = g.snapshot();
        let views = [Player::P0, Player::P1].map(|p| serde_json::to_value(g.observe(p)).unwrap());
        let mut world = FundingState::from_game(&g, &goal);
        world.prepare_entry_projection();
        assert!(
            characteristic_query::CharacteristicQuery::in_world(&world)
                .keywords(hero)
                .contains(Keywords::HEXPROOF)
        );
        assert!(world.remove_creature_counter(g.object_ref(cap), CounterKind::Shield));
        let projected =
            characteristic_query::CharacteristicQuery::in_world(&world).object_view(hero);
        assert!(!projected.keywords.contains(Keywords::HEXPROOF));
        let mut actual = g.clone();
        actual.set_counters(cap, Counters::default());
        assert_eq!(projected, actual.object_view(hero));
        let mut world = FundingState::from_game(&g, &goal);
        world.prepare_entry_projection();
        let mut visits = 0;
        world.cost_departures(&[g.object_ref(cap)], Zone::Graveyard, &mut |departed| {
            visits += 1;
            assert!(
                !characteristic_query::CharacteristicQuery::in_world(departed)
                    .keywords(hero)
                    .contains(Keywords::HEXPROOF)
            );
            false
        });
        assert_eq!(visits, 1);
        assert_eq!(g.snapshot(), before);
        assert_eq!(
            [Player::P0, Player::P1].map(|p| serde_json::to_value(g.observe(p)).unwrap()),
            views
        );
    }
}
