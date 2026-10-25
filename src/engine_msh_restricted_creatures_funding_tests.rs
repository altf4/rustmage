//! Exact virtual departures must agree with actual activation permission and attached costs.
use super::super::msh_restricted_creatures_tests::{game, paid};
use super::*;
#[test]
fn msh_restricted_creatures_funding_virtual_haste_provider_departure_matches_actual() {
    for p in [Player::P0, Player::P1] {
        let mut g = game(p);
        let s = paid(&mut g, p, "Shang-Chi, Master of Kung Fu", &[]);
        let elf = paid(&mut g, p, "Llanowar Elves", &[]);
        let goal = FundingGoal {
            actor: p,
            required: [0; 6],
            generic: 1,
            reserve: None,
            source: None,
        };
        let before = g.snapshot();
        let views = [Player::P0, Player::P1].map(|p| serde_json::to_value(g.observe(p)).unwrap());
        assert!(g.can_fund(goal.clone()));
        let mut w = FundingState::from_game(&g, &goal);
        w.prepare_entry_projection();
        assert!(
            ability_world::CharacteristicWorld::activation_keywords(&w, elf)
                .contains(Keywords::HASTE)
        );
        assert!(
            !characteristic_query::CharacteristicQuery::in_world(&w)
                .keywords(elf)
                .contains(Keywords::HASTE)
        );
        let mut visits = 0;
        w.cost_departures(&[g.object_ref(s)], Zone::Graveyard, &mut |departed| {
            visits += 1;
            assert!(
                !ability_world::CharacteristicWorld::activation_keywords(departed, elf)
                    .contains(Keywords::HASTE)
            );
            assert!(!departed.reachable(&goal, &mut HashSet::new()));
            false
        });
        assert_eq!(visits, 1);
        let mut actual = g.clone();
        actual.move_object(s, Zone::Graveyard);
        assert!(!actual.can_fund(goal));
        assert_eq!(g.snapshot(), before);
        assert_eq!(
            [Player::P0, Player::P1].map(|p| serde_json::to_value(g.observe(p)).unwrap()),
            views
        );
    }
}
#[test]
fn msh_restricted_creatures_funding_attached_equipment_exact_reservation_uses_source_incarnation() {
    for p in [Player::P0, Player::P1] {
        let mut g = game(p);
        let s = paid(&mut g, p, "Shang-Chi, Master of Kung Fu", &[]);
        let r = paid(&mut g, p, "Ronin, Shadow Stalker", &[]);
        let e = super::super::assurance::big_helpers::named(
            &mut g,
            p,
            "S.H.I.E.L.D. Spy Kit",
            Zone::Battlefield,
        );
        assert!(g.attach_equipment(g.object_ref(e), g.object_ref(r)));
        let definition = g.state.objects[r as usize].card;
        let spec = definition.activation(1).cost.additional.unwrap();
        let goal = FundingGoal {
            actor: p,
            required: [0; 6],
            generic: 0,
            reserve: Some((g.object_ref(e), spec)),
            source: Some((g.object_ref(r), 1, Counters::default(), definition, None)),
        };
        assert!(g.can_fund(goal.clone()));
        let before = g.snapshot();
        let mut w = FundingState::from_game(&g, &goal);
        w.prepare_entry_projection();
        assert!(w.reachable(&goal, &mut HashSet::new()));
        let pos = w.resources.iter().position(|x| x.id == e).unwrap();
        w.resources[pos].object.attached_to = Some(ObjectRef {
            card: r,
            generation: g.object_ref(r).generation + 1,
        });
        assert!(!w.reachable(&goal, &mut HashSet::new()));
        assert_eq!(g.snapshot(), before);
        assert_eq!(g.object_view(s).zone, Zone::Battlefield);
    }
}
