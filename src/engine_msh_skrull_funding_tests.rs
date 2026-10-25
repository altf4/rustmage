//! Conditional type/stat fragments read the same owned projected world as mana guards.
use super::*;
use crate::engine::assurance::big_helpers::{cast, named, settle, setup};
fn goal(actor: Player) -> FundingGoal {
    FundingGoal {
        actor,
        required: [0; 6],
        generic: 0,
        reserve: None,
        source: None,
    }
}
fn form(world: &FundingState<'_>, id: CardId) -> (i128, bool) {
    let q = characteristic_query::CharacteristicQuery::in_world(world);
    (q.stats(id).power, q.object_view(id).has_subtype("Elf"))
}
#[test]
fn msh_skrull_funding_virtual_death_and_graveyard_exile_match_owned_actual_branches() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, "Undercover Skrull", Zone::Hand);
        cast(&mut g, source, None, &[]);
        settle(&mut g);
        let fuel = named(&mut g, p, "Hill Giant", Zone::Graveyard);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let secret = named(&mut g, p, "Shock", Zone::Hand);
        let before = g.snapshot();
        let goal = goal(p);
        let mut world = FundingState::from_game(&g, &goal);
        world.prepare_entry_projection();
        assert_eq!(form(&world, source), (1, false));
        let mut actual = {
            let parent = g.clone();
            parent.clone()
        };
        actual.move_object(bear, Zone::Graveyard);
        let expected = actual.object_view(source);
        assert_eq!(expected.power, 3);
        let mut visits = 0;
        world.cost_departures(&[g.object_ref(bear)], Zone::Graveyard, &mut |dead| {
            visits += 1;
            assert_eq!(form(dead, source), (3, true));
            assert_eq!(
                characteristic_query::CharacteristicQuery::in_world(dead).object_view(source),
                expected
            );
            let mut exiled = 0;
            dead.cost_departures(&[dead.reference(fuel)], Zone::Exile, &mut |gone| {
                exiled += 1;
                assert_eq!(form(gone, source), (1, false));
                false
            });
            assert_eq!(exiled, 1);
            false
        });
        assert_eq!(visits, 1);
        assert_eq!(g.snapshot(), before);
        assert!(!g.observe(p.other()).objects.iter().any(|o| o.id == secret));
    }
}
#[test]
fn msh_skrull_funding_threshold_rechecks_projected_controller_after_control_source_departure() {
    for p in [Player::P0, Player::P1] {
        let q = p.other();
        let mut g = setup(q);
        let source = named(&mut g, q, "Undercover Skrull", Zone::Hand);
        cast(&mut g, source, None, &[]);
        settle(&mut g);
        for _ in 0..2 {
            named(&mut g, p, "Bear Cub", Zone::Graveyard);
        }
        g.state.active = p;
        g.state.priority = p;
        let aura = named(&mut g, p, "Confiscate", Zone::Hand);
        cast(&mut g, aura, None, &[Target::Card(source)]);
        settle(&mut g);
        assert_eq!(g.object_view(source).controller, p);
        assert_eq!(g.stats(source).power, 3);
        let before = g.snapshot();
        let mut actual = {
            let parent = g.clone();
            parent.clone()
        };
        actual.move_object(aura, Zone::Graveyard);
        let expected = actual.object_view(source);
        assert_eq!((expected.controller, expected.power), (q, 1));
        let goal = goal(p);
        let mut world = FundingState::from_game(&g, &goal);
        world.prepare_entry_projection();
        let mut visits = 0;
        world.cost_departures(&[g.object_ref(aura)], Zone::Graveyard, &mut |after| {
            visits += 1;
            let view =
                characteristic_query::CharacteristicQuery::in_world(after).object_view(source);
            assert_eq!(view, expected);
            assert_eq!(form(after, source), (1, false));
            false
        });
        assert_eq!(visits, 1);
        assert_eq!(g.snapshot(), before);
    }
}
#[test]
fn msh_skrull_funding_new_entry_registers_shared_type_timestamp_and_bonus_provider() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        for _ in 0..2 {
            named(&mut g, p, "Bear Cub", Zone::Graveyard);
        }
        let source = named(&mut g, p, "Undercover Skrull", Zone::Hand);
        let before = g.snapshot();
        let movement = g.entry_move(source, Zone::Battlefield, false);
        let world = FundingState::from_game(&g, &goal(p));
        let mut visits = 0;
        assert!(
            world.commit_projected_entries(vec![movement], &[0], &mut |next| {
                visits += 1;
                assert_eq!(form(next, source), (3, true));
                assert!(next.static_sources().contains(&source));
                assert!(
                    next.roles()
                        .unwrap()
                        .ability_timestamps
                        .iter()
                        .any(|r| r.source == next.reference(source))
                );
                true
            })
        );
        assert_eq!(visits, 1);
        assert_eq!(g.snapshot(), before);
    }
}
#[test]
fn msh_skrull_funding_later_type_setter_and_suppression_keep_same_layer_seven_bonus() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        for _ in 0..2 {
            named(&mut g, p, "Bear Cub", Zone::Graveyard);
        }
        let source = named(&mut g, p, "Undercover Skrull", Zone::Hand);
        cast(&mut g, source, None, &[]);
        settle(&mut g);
        let aura = named(&mut g, p, "Witness Protection", Zone::Hand);
        cast(&mut g, aura, None, &[Target::Card(source)]);
        settle(&mut g);
        let before = g.snapshot();
        let world = FundingState::from_game(&g, &goal(p));
        assert_eq!(form(&world, source), (3, false));
        assert!(!world.printed_abilities_available(source));
        let v = characteristic_query::CharacteristicQuery::in_world(&world).object_view(source);
        assert!(v.has_subtype("Citizen"));
        assert_eq!(v, g.object_view(source));
        assert_eq!(g.snapshot(), before);
    }
}
