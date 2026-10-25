//! Shared projected-world counter and single-color power mana rules; no admissions.
use super::*;
use crate::engine::assurance::big_helpers::{cast, settle, setup};
use crate::engine::msh_counter_quantity_core_tests::named;
const DOC: &str = "Doc Samson, Super Psychiatrist";
fn goal(p: Player) -> FundingGoal {
    FundingGoal {
        actor: p,
        required: [0; 6],
        generic: 0,
        reserve: None,
        source: None,
    }
}
fn row(g: &Game, id: CardId, p: Player) -> replacements::CounterPlacement {
    let o = &g.state.objects[id as usize];
    replacements::CounterPlacement {
        object: g.object_ref(id),
        placed_by: p,
        card: o.card,
        owner: o.owner,
        controller: o.controller,
        subtypes: g.granted_subtypes(id),
        kind: g.current_kinds(id),
        amount: Counters::of(CounterKind::Page, 1),
        effect: true,
        entry_index: None,
        private: false,
    }
}
#[test]
fn msh_counter_core_funding_virtual_provider_departure_matches_actual_counter_outcomes() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let d = named(&mut g, p, DOC, Zone::Battlefield);
        let r = named(&mut g, p, "Gilded Lotus", Zone::Battlefield);
        let before = g.snapshot();
        let placement = row(&g, r, p);
        let world = FundingState::from_game(&g, &goal(p));
        let mut got = Vec::new();
        replacements::counter_outcomes(&world, &placement, &mut Vec::new(), &mut |n| {
            got.push(n.get(CounterKind::Page));
            false
        });
        assert_eq!(got, vec![2]);
        world.cost_departures(&[g.object_ref(d)], Zone::Graveyard, &mut |after| {
            let mut actual = g.clone();
            actual.move_object(d, Zone::Graveyard);
            let mut a = Vec::new();
            replacements::counter_outcomes(after, &placement, &mut Vec::new(), &mut |n| {
                a.push(n.clone());
                false
            });
            let mut b = Vec::new();
            replacements::counter_outcomes(&actual, &placement, &mut Vec::new(), &mut |n| {
                b.push(n.clone());
                false
            });
            assert_eq!(a, b);
            assert_eq!(a[0].get(CounterKind::Page), 1);
            false
        });
        assert_eq!(g.snapshot(), before);
    }
}
#[test]
fn msh_counter_core_funding_restored_ability_and_source_power_use_virtual_aura_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let d = named(&mut g, p, DOC, Zone::Battlefield);
        let a = named(&mut g, p, "Witness Protection", Zone::Hand);
        cast(&mut g, a, None, &[Target::Card(d)]);
        settle(&mut g);
        g.state.players[p.index()].mana = [0; 6];
        let before = g.snapshot();
        let mut w = FundingState::from_game(&g, &goal(p));
        w.prepare_entry_projection();
        assert_eq!(w.power(d), 1);
        let production = ManaProduction::TapSourcePowerChoice { mask: 31 };
        w.cost_departures(&[g.object_ref(a)], Zone::Graveyard, &mut |after| {
            let mut actual = g.clone();
            actual.move_object(a, Zone::Graveyard);
            assert_eq!(after.power(d), 3);
            assert_eq!(
                production.in_world(after, d),
                production.in_world(&actual, d)
            );
            assert_eq!(
                production.in_world(after, d),
                ManaProduction::Choice {
                    mask: 31,
                    amount: 3
                }
            );
            false
        });
        assert_eq!(g.snapshot(), before);
    }
}
#[test]
fn msh_counter_core_funding_source_counter_reservations_zero_through_excess_match_actual_availability()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let a = named(&mut g, p, "The Astonishing Ant-Man", Zone::Battlefield);
        g.state.objects[a as usize].summoning_sick = false;
        g.set_counters(a, Counters::of(CounterKind::PlusOne, 5));
        named(&mut g, p, "Gilded Lotus", Zone::Battlefield);
        g.state.players[p.index()].mana = [0; 6];
        let before = g.snapshot();
        for n in 0..=6 {
            let mut request = goal(p);
            request.required[4] = 1;
            request.generic = 2;
            request.source = Some((
                g.object_ref(a),
                0,
                Counters::of(CounterKind::PlusOne, n),
                g.state.objects[a as usize].card,
                None,
            ));
            assert_eq!(g.can_fund(request), n <= 5);
        }
        assert_eq!(g.snapshot(), before);
    }
}
#[test]
fn msh_counter_core_funding_power_color_shortages_and_virtual_counters_are_exact() {
    for p in [Player::P0, Player::P1] {
        for count in 0..=4 {
            let mut g = setup(p);
            let d = named(&mut g, p, DOC, Zone::Battlefield);
            g.state.objects[d as usize].summoning_sick = false;
            g.set_counters(d, Counters::of(CounterKind::PlusOne, count));
            g.state.players[p.index()].mana = [0; 6];
            for color in 0..5 {
                let mut request = goal(p);
                request.required[color] = 3 + count as u32;
                assert!(g.can_fund(request.clone()));
                request.required[color] += 1;
                assert!(!g.can_fund(request));
            }
            let mut split = goal(p);
            split.required[0] = 1;
            split.required[1] = 1;
            assert!(!g.can_fund(split));
        }
    }
}
