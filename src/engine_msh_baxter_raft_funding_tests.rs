//! Shared actual/borrowed characteristic guards and net-zero color-conversion search.
use super::*;
use crate::engine::assurance::big_helpers::{named, setup};
#[test]
fn msh_baxter_raft_funding_toughness_guard_virtual_lord_departure_matches_actual() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let b = named(&mut g, p, "Baxter Building", Zone::Battlefield);
        let hero = named(&mut g, p, "Human Torch, Johnny Storm", Zone::Battlefield);
        let lord = named(&mut g, p, "Avengers Assemble!", Zone::Battlefield);
        let guard = ActivationGuard::ControlCreatureToughnessAtLeast(4);
        let goal = FundingGoal {
            actor: p,
            required: [0; 6],
            generic: 4,
            reserve: None,
            source: None,
        };
        let snapshot = g.snapshot();
        let mut w = FundingState::from_game(&g, &goal);
        w.prepare_entry_projection(); // Retain the explicitly departed nonmana layer provider.
        assert!(w.guard_available(guard, b, p));
        assert!(g.activation_guard_available(guard, b, p));
        let mut actual = g.clone();
        actual.move_object(lord, Zone::Graveyard);
        assert!(!actual.activation_guard_available(guard, b, p));
        let expected = actual.object_view(hero);
        let mut calls = 0;
        w.cost_departures(&[g.object_ref(lord)], Zone::Graveyard, &mut |next| {
            calls += 1;
            assert!(!next.guard_available(guard, b, p));
            assert_eq!(
                characteristic_query::CharacteristicQuery::in_world(next).object_view(hero),
                expected
            );
            false
        });
        assert_eq!(calls, 1);
        assert_eq!(g.snapshot(), snapshot);
    }
}
#[test]
fn msh_baxter_raft_funding_net_zero_converter_reaches_mixed_colors_not_more_quantity() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let b = named(&mut g, p, "Baxter Building", Zone::Battlefield);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 4];
        let goal = FundingGoal {
            actor: p,
            required: [1, 1, 1, 1, 0, 0],
            generic: 0,
            reserve: None,
            source: None,
        };
        let before = g.snapshot();
        assert!(g.can_fund(goal.clone()));
        let mut too_much = goal.clone();
        too_much.generic = 1;
        assert!(!g.can_fund(too_much));
        g.state.objects[b as usize].tapped = true;
        assert!(!g.can_fund(goal.clone()));
        g.state.objects[b as usize].tapped = false;
        g.state.players[p.index()].mana[5] = 3;
        assert!(!g.can_fund(goal));
        g.restore(&before).unwrap();
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 0, 0, 4]);
    }
}
#[test]
fn msh_baxter_raft_funding_locked_target_discount_survives_virtual_artifact_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let r = named(&mut g, p, "Raft Security Officer", Zone::Battlefield);
        g.state.objects[r as usize].summoning_sick = false;
        let iron = named(&mut g, p, "Iron Man, Master of Machines", Zone::Battlefield);
        for _ in 0..3 {
            named(&mut g, p, "Hedron Archive", Zone::Battlefield);
        }
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        let mut item = g.activation_item(p, r, 0);
        item.targets.push(g.bind_target(Target::Card(iron)));
        assert_eq!(g.stats(iron).power, 4);
        let locked = g.selected_activation_target_cost(
            &item,
            ManaCost {
                colored: [0; 6],
                generic: 2,
            },
        );
        assert_eq!(locked.generic, 2);
        let goal = FundingGoal {
            actor: p,
            required: [0; 6],
            generic: locked.generic,
            reserve: None,
            source: Some((
                g.object_ref(r),
                0,
                Counters::default(),
                item.source_card,
                None,
            )),
        };
        let w = FundingState::from_game(&g, &goal);
        let artifact = (g.state.objects.len() - 1) as CardId;
        let before = g.snapshot();
        w.cost_departures(&[g.object_ref(artifact)], Zone::Graveyard, &mut |next| {
            assert_eq!(next.power(iron), 3);
            assert_eq!(goal.generic, 2);
            false
        });
        assert_eq!(g.snapshot(), before);
    }
}
