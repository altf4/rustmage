//! Long control is part of the existing actual and speculative layer-two timeline.
use super::super::assurance::big_helpers::{checked, named, next_turn, setup};
use super::super::msh_broker_thrall_tests::{packet, paid};
use super::*;
#[test]
fn msh_broker_thrall_funding_long_control_resumes_after_virtual_and_actual_aura_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, "Powerful Broker", &[]);
        let target = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        paid(&mut g, p, "Evil's Thrall", &[Target::Card(target)]);
        next_turn(&mut g);
        while g.state.step != Step::Main1 || g.state.pending != Pending::Priority {
            checked(&mut g, Action::Pass);
        }
        let aura = paid(&mut g, p.other(), "Confiscate", &[Target::Card(target)]);
        assert_eq!(g.object_view(target).controller, p.other());
        let goal = FundingGoal {
            actor: p.other(),
            required: [0; 6],
            generic: 0,
            reserve: None,
            source: None,
        };
        let snapshot = g.snapshot();
        let before_views =
            [Player::P0, Player::P1].map(|q| serde_json::to_value(g.observe(q)).unwrap());
        let mut world = FundingState::from_game(&g, &goal);
        world.prepare_entry_projection();
        assert_eq!(
            characteristic_query::CharacteristicQuery::in_world(&world)
                .object_view(target)
                .controller,
            p.other()
        );
        let mut visits = 0;
        world.cost_departures(&[g.object_ref(aura)], Zone::Graveyard, &mut |departed| {
            visits += 1;
            assert_eq!(
                characteristic_query::CharacteristicQuery::in_world(departed)
                    .object_view(target)
                    .controller,
                p
            );
            false
        });
        assert_eq!(visits, 1);
        assert_eq!(g.snapshot(), snapshot);
        assert_eq!(
            [Player::P0, Player::P1].map(|q| serde_json::to_value(g.observe(q)).unwrap()),
            before_views
        );
        packet(&mut g, p, "Disenchant", &[Target::Card(aura)]);
        super::super::assurance::big_helpers::settle(&mut g);
        assert_eq!(g.object_view(target).controller, p);
        next_turn(&mut g);
        assert_eq!(g.object_view(target).controller, p);
        next_turn(&mut g);
        assert_eq!(g.object_view(target).controller, p.other());
    }
}
