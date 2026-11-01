//! Projection must normalize a departing manifested mana source exactly as the owned move does.
use super::*;
use crate::engine::assurance::big_helpers::{named, setup};

#[test]
fn slow_sacrifice_of_manifest_and_copied_manifest_matches_owned_departure_state() {
    for player in [Player::P0, Player::P1] {
        for copied in [false, true] {
            let mut game = setup(player);
            let thranduil = named(
                &mut game,
                player,
                "Thranduil, the Elvenking",
                Zone::Battlefield,
            );
            let elf = named(&mut game, player, "Llanowar Elves", Zone::Library);
            let movement = game.manifest_entry(game.object_ref(elf), player).unwrap();
            game.stage_entries(vec![movement], None);
            assert!(!game.advance_entries());
            let old = game.object_ref(elf);
            if copied {
                game.apply_temporary_copy(
                    old,
                    CopyRecipe::new(Card::from_name("Bear Cub").unwrap()),
                    CopyDeadline::EndOfTurn,
                );
            }
            game.state.objects[elf as usize].summoning_sick = false;
            game.state.objects[elf as usize].class_level = 3;
            game.state.objects[elf as usize].cast_for_warp = true;
            game.add_counter(elf, CounterKind::Charge, 2);
            let ring = named(&mut game, player, "Lotus Ring", Zone::Battlefield);
            let grantor = game.object_ref(ring);
            assert!(game.attach_equipment(grantor, old));
            game.state.players[player.index()].mana = [0; 6];
            let goal = FundingGoal {
                actor: player,
                required: [0, 0, 0, 0, 4, 0],
                generic: 0,
                reserve: None,
                source: None,
            };
            let before = game.snapshot();
            let mut projected = FundingState::from_game(&game, &goal);
            // Force the entry-aware departure planner used by nested payments.
            projected.prepare_entry_projection();
            let step = projected
                .moves(&goal)
                .into_iter()
                .find(|s| {
                    projected.resources[s.resource].id == elf
                        && s.grantor == Some(grantor)
                        && s.production == [0, 0, 0, 0, 3, 0]
                })
                .expect("granted sacrifice mana");
            let mut expected = None;
            projected.visit_applied(&step, &mut |next| {
                let donor = borrowed_activations::abilities(next, thranduil)
                    .into_iter()
                    .find(|a| a.donor.card == elf && a.ability == u16::MAX)
                    .expect("physical Elf donor");
                assert!(
                    next.named_counters
                        .as_ref()
                        .is_none_or(|rows| rows.iter().all(|r| r.object() != old))
                );
                assert!(
                    next.entry_projection
                        .as_ref()
                        .unwrap()
                        .manifested
                        .iter()
                        .all(|(object, _)| *object != old)
                );
                let object = next.object(elf).clone();
                assert_eq!(object.class_level, 1);
                assert!(!object.cast_for_warp);
                expected = Some((object, donor.slot));
                true
            });
            drop(projected);
            game.apply(
                player,
                Action::TapGrantedMana {
                    card: elf,
                    grantor,
                    production: [0, 0, 0, 0, 3, 0],
                },
            )
            .unwrap();
            let (object, slot) = expected.expect("a legal projected branch");
            assert_eq!(object, game.state.objects[elf as usize]);
            assert_eq!(object.card.name(), "Llanowar Elves");
            assert!(
                game.state
                    .named_counters
                    .as_ref()
                    .is_none_or(|rows| rows.iter().all(|r| r.object() != old))
            );
            let actual = game
                .graveyard_borrowed_ability_views()
                .unwrap()
                .into_iter()
                .find(|a| {
                    a.source.card == thranduil && a.graveyard.card == elf && a.ability == u16::MAX
                })
                .unwrap();
            assert_eq!(actual.slot, slot);
            let after = game.snapshot();
            game.restore(&before).unwrap();
            game.apply(
                player,
                Action::TapGrantedMana {
                    card: elf,
                    grantor,
                    production: [0, 0, 0, 0, 3, 0],
                },
            )
            .unwrap();
            assert_eq!(game.snapshot(), after);
        }
    }
}
