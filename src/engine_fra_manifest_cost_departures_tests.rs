// Include as a child module of engine::funding so FundingState and its helpers are visible.
use super::*;
use crate::engine::assurance::big_helpers::{named, setup};

#[test]
fn projected_cost_departure_restores_manifest_physical_face_even_when_copied() {
    for player in [Player::P0, Player::P1] {
        for copied in [false, true] {
            let mut g = setup(player);
            let thranduil = named(
                &mut g,
                player,
                "Thranduil, the Elvenking",
                Zone::Battlefield,
            );
            let elf = named(&mut g, player, "Llanowar Elves", Zone::Library);
            let movement = g.manifest_entry(g.object_ref(elf), player).unwrap();
            g.stage_entries(vec![movement], None);
            assert!(!g.advance_entries());
            if copied {
                let reference = g.object_ref(elf);
                g.apply_temporary_copy(
                    reference,
                    CopyRecipe::new(Card::from_name("Bear Cub").unwrap()),
                    CopyDeadline::EndOfTurn,
                );
            }
            let before = g.object_ref(elf);
            let goal = FundingGoal {
                actor: player,
                required: [0; 6],
                generic: 0,
                reserve: None,
                source: None,
            };
            let mut world = FundingState::from_game(&g, &goal);
            world.prepare_entry_projection();
            let mut predicted = None;
            world.cost_departures(&[before], Zone::Graveyard, &mut |next| {
                let donor = borrowed_activations::abilities(next, thranduil)
                    .into_iter()
                    .find(|ability| ability.donor.card == elf && ability.ability == u16::MAX)
                    .expect("underlying Elf ability after projected sacrifice");
                predicted = Some((next.object(elf).clone(), donor.slot));
                true
            });
            drop(world);
            let saved = g.snapshot();
            g.move_object(elf, Zone::Graveyard);
            let (object, slot) = predicted.unwrap();
            assert_eq!(object, g.state.objects[elf as usize]);
            let actual = g
                .graveyard_borrowed_ability_views()
                .unwrap()
                .into_iter()
                .find(|ability| {
                    ability.source.card == thranduil
                        && ability.graveyard.card == elf
                        && ability.ability == u16::MAX
                })
                .unwrap();
            assert_eq!(actual.slot, slot);
            g.restore(&saved).unwrap();
            g.move_object(elf, Zone::Graveyard);
            assert_eq!(g.state.objects[elf as usize], object);
        }
    }
}
