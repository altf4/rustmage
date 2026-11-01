use super::*;
use super::*;
use ability_world::CharacteristicWorld;
pub use rustmage_core::engine::funding::funding_entries::*;
#[cfg(test)]
#[path = "assurance/hob_lonely_funding.rs"]
mod hob_lonely_funding;
#[cfg(test)]
mod great_arashin_tests {
    use super::*;
    use crate::engine::assurance::big_helpers::{named, setup};

    #[test]
    fn projected_entry_uses_effective_controller_and_nonbasic_subtypes() {
        for p in [Player::P0, Player::P1] {
            for (name, owner, expected) in [
                ("Lush Portico", p, false),
                ("Forest", p, false),
                ("Plains", p, false),
                ("Island", p, true),
                ("Forest", p.other(), true),
            ] {
                let mut g = setup(p);
                named(&mut g, owner, name, Zone::Battlefield);
                let city = named(&mut g, p, "Great Arashin City", Zone::Hand);
                let movement =
                    entries::EntryMove::from_view(g.object_view(city), Zone::Battlefield, false);
                let goal = FundingGoal {
                    actor: p,
                    required: [0; 6],
                    generic: 0,
                    reserve: None,
                    source: None,
                };
                let world = FundingState::from_game(&g, &goal);
                assert_eq!(
                    world.entry_tapped(&movement),
                    expected,
                    "{name} controlled by {owner:?}"
                );
            }
        }
    }
}
#[cfg(test)]
mod projected_life_loss_tests {
    use super::*;
    use crate::engine::assurance::big_helpers::{named, setup};

    #[test]
    fn opponent_life_loss_remains_true_after_equal_gain_and_branch_disposal() {
        for p in [Player::P0, Player::P1] {
            let mut g = setup(p);
            let source = named(&mut g, p, "Cindering Cutthroat", Zone::Hand);
            let movement =
                entries::EntryMove::from_view(g.object_view(source), Zone::Battlefield, false);
            let goal = FundingGoal {
                actor: p,
                required: [0; 6],
                generic: 0,
                reserve: None,
                source: None,
            };
            let saved = g.snapshot();
            let world = FundingState::from_game(&g, &goal);
            assert!(!world.entry_guard(StepGuard::OpponentLostLife, &movement));
            let mut branch = world.clone();
            branch.pay_projected_life(p.other(), 1, &[]);
            branch.other_life += 1;
            assert_eq!(branch.life(p.other()), world.life(p.other()));
            assert!(branch.entry_guard(StepGuard::OpponentLostLife, &movement));
            assert!(!world.entry_guard(StepGuard::OpponentLostLife, &movement));
            drop(branch);
            assert_eq!(g.snapshot(), saved);
        }
    }
}
#[cfg(test)]
mod starting_town_tests {
    use super::*;
    use crate::engine::assurance::big_helpers::{named, setup};

    #[test]
    fn projected_town_entry_matches_actual_controller_turn() {
        for p in [Player::P0, Player::P1] {
            for count in 0..=4 {
                for active in [p, p.other()] {
                    let mut g = setup(p);
                    g.state.players[p.index()].turns_started = count;
                    g.state.active = active;
                    let town = named(&mut g, p, "Starting Town", Zone::Hand);
                    let movement = entries::EntryMove::from_view(
                        g.object_view(town),
                        Zone::Battlefield,
                        false,
                    );
                    let goal = FundingGoal {
                        actor: p,
                        required: [0; 6],
                        generic: 0,
                        reserve: None,
                        source: None,
                    };
                    let world = FundingState::from_game(&g, &goal);
                    assert_eq!(world.entry_tapped(&movement), g.enters_tapped(town));
                    assert_eq!(
                        world.entry_tapped(&movement),
                        active != p || count == 0 || count > 3
                    );
                }
            }
        }
    }
}
#[cfg(test)]
#[path = "engine_requested_lunar_funding_tests.rs"]
mod requested_lunar_funding_tests;
#[cfg(test)]
mod multiversal_projection_tests {
    use super::*;
    use crate::engine::assurance::big_helpers::{named, setup};

    #[test]
    fn two_projected_entries_choose_types_independently_under_competing_replacements() {
        for p in [Player::P0, Player::P1] {
            let mut g = setup(p);
            named(&mut g, p, "Spelunking", Zone::Battlefield);
            let first = named(&mut g, p, "Forest", Zone::Hand);
            let second = named(&mut g, p, "Forest", Zone::Hand);
            let card = Card::from_name("Multiversal Passage").unwrap();
            for id in [first, second] {
                g.state.objects[id as usize].card = card;
            }
            let moves = [first, second]
                .into_iter()
                .map(|id| {
                    entries::EntryMove::from_view(g.object_view(id), Zone::Battlefield, false)
                })
                .collect();
            let goal = FundingGoal {
                actor: p,
                required: [0; 6],
                generic: 0,
                reserve: None,
                source: None,
            };
            let before = g.snapshot();
            let mut world = FundingState::from_game(&g, &goal);
            for id in [first, second] {
                if !world.resources.iter().any(|r| r.id == id) {
                    world.resources.push(Resource {
                        id,
                        attacking: false,
                        object: g.state.objects[id as usize].clone(),
                    });
                }
            }
            world.prepare_entry_projection();
            let mut seen = [[false; 5]; 5];
            world.visit_entry_moves(moves, true, None, &mut |next| {
                let a = next.entry_value(next.reference(first));
                let b = next.entry_value(next.reference(second));
                if let (
                    Some(EntryChoiceValue::BasicLandType(a)),
                    Some(EntryChoiceValue::BasicLandType(b)),
                ) = (a, b)
                {
                    assert!(next.subtypes(first).has(next.object(first).card, a.name()));
                    assert!(
                        next.subtypes(second)
                            .has(next.object(second).card, b.name())
                    );
                    seen[a.index()][b.index()] = true;
                }
                false
            });
            assert!(seen.iter().all(|row| row.iter().all(|seen| *seen)));
            assert_eq!(g.snapshot(), before);
        }
    }

    #[test]
    fn projected_basic_type_choice_precedes_competing_entry_payments() {
        for p in [Player::P0, Player::P1] {
            let mut g = setup(p);
            named(&mut g, p, "Spelunking", Zone::Battlefield);
            let land = named(&mut g, p, "Forest", Zone::Hand);
            g.state.objects[land as usize].card = Card::from_name("Multiversal Passage").unwrap();
            let movement =
                entries::EntryMove::from_view(g.object_view(land), Zone::Battlefield, false);
            let goal = FundingGoal {
                actor: p,
                required: [0; 6],
                generic: 0,
                reserve: None,
                source: None,
            };
            let saved = g.snapshot();
            let mut world = FundingState::from_game(&g, &goal);
            if !world.resources.iter().any(|r| r.id == land) {
                world.resources.push(Resource {
                    id: land,
                    attacking: false,
                    object: g.state.objects[land as usize].clone(),
                });
            }
            world.prepare_entry_projection();
            let mut seen = [[false; 2]; 5];
            world.visit_entry_moves(vec![movement], true, None, &mut |next| {
                let object = next.object(land);
                if object.zone == Zone::Battlefield
                    && let Some(EntryChoiceValue::BasicLandType(kind)) =
                        next.entry_value(next.reference(land))
                {
                    let paid = next.life(p) < g.state.players[p.index()].life;
                    assert!(next.subtypes(land).has(object.card, kind.name()));
                    seen[kind.index()][usize::from(paid)] = true;
                }
                false
            });
            assert!(seen.iter().all(|row| row.iter().all(|seen| *seen)));
            assert_eq!(g.snapshot(), saved);
        }
    }
}
#[cfg(test)]
mod full_bore_warp_projection_tests {
    use super::*;
    use crate::engine::assurance::big_helpers::{named, setup};

    #[test]
    fn projected_cast_for_warp_is_incarnation_bound_and_branch_local() {
        for p in [Player::P0, Player::P1] {
            let mut g = setup(p);
            let card = named(&mut g, p, "Bear Cub", Zone::Hand);
            let goal = FundingGoal {
                actor: p,
                required: [0; 6],
                generic: 0,
                reserve: None,
                source: None,
            };
            let before = g.snapshot();
            let mut world = FundingState::from_game(&g, &goal);
            if !world.resources.iter().any(|r| r.id == card) {
                world.resources.push(Resource {
                    id: card,
                    attacking: false,
                    object: g.state.objects[card as usize].clone(),
                });
            }
            world.prepare_entry_projection();
            for (cast, warp, expected) in [
                (true, true, true),
                (true, false, false),
                (false, true, false),
            ] {
                let mut movement =
                    entries::EntryMove::from_view(g.object_view(card), Zone::Battlefield, false);
                movement.cast_origin = cast.then_some(Zone::Hand);
                movement.warp_controller = warp.then_some(p);
                let mut visited = false;
                world.visit_entry_moves(vec![movement], true, None, &mut |next| {
                    assert_eq!(next.object(card).cast_for_warp, expected);
                    visited = true;
                    false
                });
                assert!(visited);
                assert!(!world.object(card).cast_for_warp);
            }
            assert_eq!(g.snapshot(), before);
        }
    }
}
#[cfg(test)]
mod battle_projection_tests {
    use super::*;
    use crate::engine::assurance::big_helpers::{named, setup};

    #[test]
    fn entry_type_replacement_removes_siege_protector_in_borrowed_world() {
        for p in [Player::P0, Player::P1] {
            let mut g = setup(p);
            let id = named(&mut g, p, "@fixture/siege-front", Zone::Graveyard);
            let mut movement =
                entries::EntryMove::from_view(g.object_view(id), Zone::Battlefield, false);
            movement.artifact_return = true;
            let goal = FundingGoal {
                actor: p,
                required: [0; 6],
                generic: 0,
                reserve: None,
                source: None,
            };
            let mut world = FundingState::from_game(&g, &goal);
            if !world.resources.iter().any(|r| r.id == id) {
                world.resources.push(Resource {
                    id,
                    attacking: false,
                    object: g.state.objects[id as usize].clone(),
                });
            }
            let mut seen = false;
            world.visit_entry_moves(vec![movement], true, None, &mut |next| {
                let reference = next.reference(id);
                assert_eq!(
                    CharacteristicWorld::kinds(next, id) & crate::cards::BATTLE_KIND,
                    0
                );
                assert_eq!(CharacteristicWorld::battle_protector(next, reference), None);
                seen = true;
                false
            });
            assert!(seen);
        }
    }

    #[test]
    fn different_projected_protectors_have_different_funding_keys() {
        let mut g = setup(Player::P0);
        let siege = named(&mut g, Player::P0, "@fixture/siege-front", Zone::Hand);
        let goal = FundingGoal {
            actor: Player::P0,
            required: [0; 6],
            generic: 0,
            reserve: None,
            source: None,
        };
        let mut world = FundingState::from_game(&g, &goal);
        world.prepare_entry_projection();
        let object = g.object_ref(siege);
        world
            .entry_projection
            .as_mut()
            .unwrap()
            .battle_protectors
            .push((object, Player::P0));
        let mut other = world.clone();
        other.entry_projection.as_mut().unwrap().battle_protectors[0].1 = Player::P1;
        assert!(world.key(&goal) != other.key(&goal));
    }

    #[test]
    fn siege_protector_survives_in_projected_entry_world() {
        for p in [Player::P0, Player::P1] {
            let mut g = setup(p);
            named(
                &mut g,
                p,
                "@fixture/battle-reserved-5433",
                Zone::Battlefield,
            );
            let id = named(&mut g, p, "@fixture/siege-front", Zone::Hand);
            let movement =
                entries::EntryMove::from_view(g.object_view(id), Zone::Battlefield, false);
            let goal = FundingGoal {
                actor: p,
                required: [0; 6],
                generic: 0,
                reserve: None,
                source: None,
            };
            let mut world = FundingState::from_game(&g, &goal);
            if !world.resources.iter().any(|r| r.id == id) {
                world.resources.push(Resource {
                    id,
                    attacking: false,
                    object: g.state.objects[id as usize].clone(),
                });
            }
            let before = g.snapshot();
            let mut seen = false;
            world.visit_entry_moves(vec![movement], true, None, &mut |next| {
                let reference = next.reference(id);
                assert_eq!(
                    CharacteristicWorld::counter_count(next, id, CounterKind::Defense),
                    10
                );
                assert_eq!(
                    CharacteristicWorld::battle_protector(next, reference),
                    Some(p.other())
                );
                assert_eq!(
                    characteristic_query::CharacteristicQuery::in_world(next)
                        .object_view(id)
                        .battle_protector,
                    Some(p.other())
                );
                seen = true;
                false
            });
            assert!(seen);
            assert_eq!(g.snapshot(), before);
        }
    }
}
#[cfg(test)]
mod loyalty_incarnation_tests {
    use super::*;
    use crate::engine::assurance::big_helpers::{named, setup};

    #[test]
    fn projected_departures_and_entries_reset_loyalty_use_without_mutating_branch() {
        for p in [Player::P0, Player::P1] {
            let mut g = setup(p);
            let walker = named(&mut g, p, "Ajani, Caller of the Pride", Zone::Battlefield);
            g.state.objects[walker as usize].loyalty_activated_this_turn = true;
            let old = g.object_ref(walker);
            let saved = g.snapshot();
            let goal = FundingGoal {
                actor: p,
                required: [0; 6],
                generic: 0,
                reserve: None,
                source: None,
            };
            let mut world = FundingState::from_game(&g, &goal);
            // A real cast-cost context retains candidates before projecting their
            // departure; this direct fixture must retain its candidate as well.
            if !world.resources.iter().any(|r| r.id == walker) {
                world.resources.push(Resource {
                    id: walker,
                    attacking: false,
                    object: g.state.objects[walker as usize].clone(),
                });
            }
            let mut seen = false;
            world.cost_departures(&[old], Zone::Graveyard, &mut |next| {
                assert!(!next.object(walker).loyalty_activated_this_turn);
                assert_ne!(next.reference(walker), old);
                seen = true;
                false
            });
            assert!(seen);
            let movement =
                entries::EntryMove::from_view(g.object_view(walker), Zone::Battlefield, false);
            assert!(
                !world
                    .preview_entry_view(&movement)
                    .unwrap()
                    .loyalty_activated_this_turn
            );
            assert!(world.object(walker).loyalty_activated_this_turn);
            assert_eq!(g.snapshot(), saved);
        }
    }
}
#[cfg(test)]
mod valkyrie_return_replacement_tests {
    use super::*;
    use crate::engine::assurance::big_helpers::{named, setup};

    #[test]
    fn projected_valkyrie_return_does_not_make_giada_apply_early() {
        for player in [Player::P0, Player::P1] {
            for doubled in [false, true] {
                let game = &mut setup(player);
                let call = named(game, player, "Valkyrie's Call", Zone::Battlefield);
                named(game, player, "Giada, Font of Hope", Zone::Battlefield);
                if doubled {
                    named(game, player, "Doubling Season", Zone::Battlefield);
                }
                let bear = named(game, player, "Bear Cub", Zone::Graveyard);
                let saved = game.snapshot();
                let mut movement =
                    entries::EntryMove::from_view(game.object_view(bear), Zone::Battlefield, false);
                movement.return_modification = Some((ReturnKind::Angel, game.object_ref(call)));
                movement.effect_counter = Some((CounterKind::PlusOne, 1));
                let goal = FundingGoal {
                    actor: player,
                    required: [0; 6],
                    generic: 0,
                    reserve: None,
                    source: None,
                };
                let mut world = FundingState::from_game(game, &goal);
                if !world.resources.iter().any(|resource| resource.id == bear) {
                    world.resources.push(Resource {
                        id: bear,
                        attacking: false,
                        object: game.state.objects[bear as usize].clone(),
                    });
                }
                let mut seen = false;
                world.visit_entry_moves(vec![movement], true, None, &mut |projected| {
                    seen = true;
                    assert_eq!(projected.object(bear).zone, Zone::Battlefield);
                    assert_eq!(
                        projected.counter_count(bear, CounterKind::PlusOne),
                        if doubled { 2 } else { 1 }
                    );
                    assert!(
                        projected
                            .subtypes(bear)
                            .has(projected.object(bear).card, "Angel")
                    );
                    false
                });
                assert!(seen);
                assert_eq!(game.snapshot(), saved);
            }
        }
    }
}
#[cfg(test)]
mod skrull_entry_replacement_tests {
    use super::*;
    use crate::engine::assurance::big_helpers::{named, setup};

    #[test]
    fn projected_giada_skrull_entry_uses_post_move_graveyard_and_existing_angels() {
        for player in [Player::P0, Player::P1] {
            for other_creatures in [1, 2] {
                let game = &mut setup(player);
                named(game, player, "Giada, Font of Hope", Zone::Battlefield);
                for _ in 0..other_creatures {
                    named(game, player, "Bear Cub", Zone::Graveyard);
                }
                let skrull = named(game, player, "Undercover Skrull", Zone::Graveyard);
                let angel = named(game, player, "Serra Angel", Zone::Hand);
                let saved = game.snapshot();
                let goal = FundingGoal {
                    actor: player,
                    required: [0; 6],
                    generic: 0,
                    reserve: None,
                    source: None,
                };
                let mut world = FundingState::from_game(game, &goal);
                for id in [skrull, angel] {
                    if !world.resources.iter().any(|resource| resource.id == id) {
                        world.resources.push(Resource {
                            id,
                            attacking: false,
                            object: game.state.objects[id as usize].clone(),
                        });
                    }
                }
                let moves = vec![
                    entries::EntryMove::from_view(
                        game.object_view(skrull),
                        Zone::Battlefield,
                        false,
                    ),
                    entries::EntryMove::from_view(
                        game.object_view(angel),
                        Zone::Battlefield,
                        false,
                    ),
                ];
                let mut seen = false;
                world.visit_entry_moves(moves, true, None, &mut |projected| {
                    seen = true;
                    let eligible = other_creatures >= 2;
                    assert_eq!(
                        projected.counter_count(skrull, CounterKind::PlusOne),
                        u64::from(eligible)
                    );
                    assert_eq!(
                        projected
                            .subtypes(skrull)
                            .has(projected.object(skrull).card, "Angel"),
                        eligible
                    );
                    // The two entrants are simultaneous: only Giada was already an Angel.
                    assert_eq!(projected.counter_count(angel, CounterKind::PlusOne), 1);
                    false
                });
                assert!(seen);
                assert_eq!(game.snapshot(), saved);
            }
        }
    }
}
