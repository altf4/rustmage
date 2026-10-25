//! Exact mana quality and earlier-layer Doctor condition in the borrowed world.
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
fn ind(w: &FundingState<'_>, id: CardId) -> bool {
    ability_evaluation::evaluate_abilities(w).result.objects[id as usize]
        .keywords
        .contains(Keywords::INDESTRUCTIBLE)
}
#[test]
fn msh_doom_funding_restricted_castle_paths_require_artifact_spell_purpose() {
    for p in [Player::P0, Player::P1] {
        for color in 0..5 {
            let mut g = setup(p);
            named(&mut g, p, "Castle Doom", Zone::Battlefield);
            g.state.players[p.index()].mana = [0; 6];
            let art = named(&mut g, p, "Vibranium Energy Daggers", Zone::Hand);
            let non = named(&mut g, p, "Shock", Zone::Hand);
            let mut goal = goal(p);
            goal.required[color] = 1;
            for (id, yes) in [(art, true), (non, false)] {
                let mut world = FundingState::from_game(&g, &goal);
                world.purpose =
                    g.spell_mana_purpose_for_card(p, id, g.state.objects[id as usize].card);
                assert_eq!(world.reachable(&goal, &mut HashSet::new()), yes);
            }
            let world = FundingState::from_game(&g, &goal);
            assert!(!world.reachable(&goal, &mut HashSet::new()));
        }
    }
}
#[test]
fn msh_doom_funding_virtual_last_witness_departure_matches_actual_owned_branch() {
    for p in [Player::P0, Player::P1] {
        for plan in [false, true] {
            let mut g = setup(p);
            let d = named(&mut g, p, "Doctor Doom", Zone::Battlefield);
            let witness = named(
                &mut g,
                p,
                if plan {
                    "Claim the Kingdom"
                } else {
                    "Aerial Doombot"
                },
                Zone::Battlefield,
            );
            let before = g.snapshot();
            let goal = goal(p);
            let mut world = FundingState::from_game(&g, &goal);
            world.prepare_entry_projection();
            assert!(ind(&world, d));
            let mut actual = {
                let parent = g.clone();
                parent.clone()
            };
            actual.move_object(witness, Zone::Graveyard);
            assert!(!actual.keywords(d).contains(Keywords::INDESTRUCTIBLE));
            let expected = actual.object_view(d);
            let mut calls = 0;
            world.cost_departures(&[g.object_ref(witness)], Zone::Graveyard, &mut |after| {
                calls += 1;
                assert!(!ind(after, d));
                assert_eq!(
                    characteristic_query::CharacteristicQuery::in_world(after).object_view(d),
                    expected
                );
                false
            });
            assert_eq!(calls, 1);
            assert_eq!(g.snapshot(), before);
        }
    }
}
#[test]
fn msh_doom_funding_projected_doctor_entry_registers_conditional_provider() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        named(&mut g, p, "Aerial Doombot", Zone::Battlefield);
        let d = named(&mut g, p, "Doctor Doom", Zone::Hand);
        let before = g.snapshot();
        let movement = g.entry_move(d, Zone::Battlefield, false);
        let goal = goal(p);
        let w = FundingState::from_game(&g, &goal);
        let mut calls = 0;
        w.commit_projected_entries(vec![movement], &[0], &mut |next| {
            calls += 1;
            assert!(ind(next, d));
            assert!(
                next.roles()
                    .unwrap()
                    .ability_timestamps
                    .iter()
                    .any(|r| r.source == next.reference(d))
            );
            false
        });
        assert_eq!(calls, 1);
        assert_eq!(g.snapshot(), before);
    }
}
#[test]
fn msh_doom_funding_control_source_departure_rechecks_new_controller_witness() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let d = named(&mut g, p.other(), "Doctor Doom", Zone::Battlefield);
        named(&mut g, p, "Aerial Doombot", Zone::Battlefield);
        let aura = named(&mut g, p, "Confiscate", Zone::Hand);
        cast(&mut g, aura, None, &[Target::Card(d)]);
        settle(&mut g);
        assert_eq!(g.object_view(d).controller, p);
        let goal = goal(p);
        let mut w = FundingState::from_game(&g, &goal);
        w.prepare_entry_projection();
        assert!(ind(&w, d));
        let mut actual = g.clone();
        actual.move_object(aura, Zone::Graveyard);
        let expected = actual.object_view(d);
        assert!(!actual.keywords(d).contains(Keywords::INDESTRUCTIBLE));
        let mut calls = 0;
        w.cost_departures(&[g.object_ref(aura)], Zone::Graveyard, &mut |after| {
            calls += 1;
            assert!(!ind(after, d));
            assert_eq!(
                characteristic_query::CharacteristicQuery::in_world(after).object_view(d),
                expected
            );
            false
        });
        assert_eq!(calls, 1);
    }
}
