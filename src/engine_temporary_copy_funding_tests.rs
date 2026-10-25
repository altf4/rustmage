//! Borrowed mana worlds retain owned blueprints and physical departure identity.
use super::*;
use crate::engine::assurance::big_helpers::{cast, checked, named, settle, setup};
fn goal(actor: Player) -> FundingGoal {
    FundingGoal {
        actor,
        required: [0; 6],
        generic: 0,
        reserve: None,
        source: None,
    }
}
#[test]
fn msh_temporary_copy_funding_full_departure_views_match_real_physical_and_token_objects() {
    for player in [Player::P0, Player::P1] {
        for kind in 0..3 {
            for (requested, replacement) in [
                (Zone::Graveyard, false),
                (Zone::Exile, false),
                (Zone::Graveyard, true),
            ] {
                let mut game = setup(player);
                let id = if kind == 2 {
                    let recipe = CopyRecipe::new(Card::from_name("Bear Cub").unwrap())
                        .with_exceptions(CopyExceptions::GOLEM);
                    let copies = game.create_copy_groups_inner(
                        player,
                        &[TokenGroupView {
                            token: recipe.definition,
                            amount: 1,
                            tapped: false,
                        }],
                        &[Some(recipe)],
                        None,
                    );
                    game.advance_entries();
                    settle(&mut game);
                    copies[0].card
                } else {
                    named(
                        &mut game,
                        player,
                        if kind == 0 {
                            "Hill Giant"
                        } else {
                            "@token/msh-moloid"
                        },
                        Zone::Battlefield,
                    )
                };
                let object = game.object_ref(id);
                let mut recipe = CopyRecipe::new(Card::from_name("Forest").unwrap());
                recipe.exceptions.base = Some((4, 4));
                recipe.overrides = Some(CopyOverrides {
                    name: Some("Absorbing Man"),
                    kinds: Some(3),
                    types: Some(CopyTypes {
                        human_villain: true,
                        ..CopyTypes::default()
                    }),
                    keywords: Keywords::VIGILANCE,
                    ..CopyOverrides::NONE
                });
                game.apply_temporary_copy(object, recipe, CopyDeadline::EndOfTurn);
                if replacement {
                    if game.actor() != Some(player) {
                        checked(&mut game, Action::Pass);
                    }
                    let rip = named(&mut game, player, "Rest in Peace", Zone::Hand);
                    cast(&mut game, rip, None, &[]);
                    settle(&mut game);
                }
                let secret = named(&mut game, player, "Cancel", Zone::Hand);
                let snapshot = game.snapshot();
                let mut actual = {
                    let parent = game.clone();
                    parent.clone()
                };
                let event = actual.move_object_raw(id, requested);
                let expected = actual.object_view(id);
                assert_eq!(event.before.name(), "Absorbing Man");
                assert_eq!(event.before.is_token(), kind != 0);
                assert_eq!(
                    event.destination,
                    if replacement { Zone::Exile } else { requested }
                );
                assert!(expected.temporary_copy.is_none());
                let mut world = FundingState::from_game(&game, &goal(player));
                world.prepare_entry_projection();
                assert_eq!(
                    characteristic_query::CharacteristicQuery::in_world(&world).object_view(id),
                    game.object_view(id)
                );
                let mut visits = 0;
                world.cost_departures(&[object], requested, &mut |departed| {
                    visits += 1;
                    assert_eq!(
                        characteristic_query::CharacteristicQuery::in_world(departed)
                            .object_view(id),
                        expected
                    );
                    false
                });
                assert_eq!(visits, 1);
                assert_eq!(game.snapshot(), snapshot);
                assert!(
                    !game
                        .observe(player.other())
                        .objects
                        .iter()
                        .any(|view| view.id == secret)
                );
            }
        }
    }
}
#[test]
fn msh_temporary_copy_funding_virtual_treasure_sacrifice_restores_physical_identity() {
    for player in [Player::P0, Player::P1] {
        let mut game = setup(player);
        let id = named(&mut game, player, "Hill Giant", Zone::Battlefield);
        let object = game.object_ref(id);
        game.apply_temporary_copy(
            object,
            CopyRecipe::new(Card::from_name("@token/b60f6cd3b44b22bd289fe4e2").unwrap()),
            CopyDeadline::EndOfTurn,
        );
        game.state.players[player.index()].mana = [0; 6];
        let snapshot = game.snapshot();
        let mut request = goal(player);
        request.required[4] = 1;
        assert!(game.can_fund(request.clone()));
        let mut world = FundingState::from_game(&game, &request);
        let step = world
            .moves(&request)
            .into_iter()
            .find(|step| {
                world.resources[step.resource].id == id && step.production == [0, 0, 0, 0, 1, 0]
            })
            .unwrap();
        world.apply(&step, false);
        let after = ability_world::CharacteristicWorld::object(&world, id);
        assert_eq!(after.zone, Zone::Graveyard);
        assert_eq!(after.card.name(), "Hill Giant");
        assert!(!after.is_token());
        assert_eq!(
            ability_world::CharacteristicWorld::stored_recipe(&world, id),
            None
        );
        assert_eq!(game.snapshot(), snapshot);
    }
}
#[test]
fn msh_temporary_copy_funding_replacement_types_remove_basic_land_mana_but_keep_printed_elf_mana() {
    for player in [Player::P0, Player::P1] {
        for (definition, available) in [("Forest", false), ("Llanowar Elves", true)] {
            let mut game = setup(player);
            let id = named(&mut game, player, "Hill Giant", Zone::Battlefield);
            let object = game.object_ref(id);
            // A counter keeps the copied Forest's otherwise 0/0 body alive.
            game.state.objects[id as usize].counters.plus_one = 1;
            game.state.objects[id as usize].summoning_sick = false;
            let mut recipe = CopyRecipe::new(Card::from_name(definition).unwrap());
            recipe.overrides = Some(CopyOverrides {
                kinds: Some(1),
                types: Some(CopyTypes {
                    replace: true,
                    human_villain: true,
                    mercenary: true,
                    ..CopyTypes::default()
                }),
                ..CopyOverrides::NONE
            });
            game.apply_temporary_copy(object, recipe, CopyDeadline::EndOfTurn);
            game.state.players[player.index()].mana = [0; 6];
            assert!(!game.has_subtype(id, "Forest"));
            assert!(!game.has_subtype(id, "Elf"));
            assert_eq!(
                game.legal_actions().contains(&Action::TapMana { card: id }),
                available
            );
            let mut request = goal(player);
            request.required[4] = 1;
            assert_eq!(game.can_fund(request.clone()), available);
            let world = FundingState::from_game(&game, &request);
            assert_eq!(
                world
                    .moves(&request)
                    .iter()
                    .any(|step| world.resources[step.resource].id == id && step.production[4] == 1),
                available
            );
            if available {
                checked(&mut game, Action::TapMana { card: id });
                assert_eq!(game.state.players[player.index()].mana[4], 1);
            } else {
                let before = game.snapshot();
                let views = [
                    serde_json::to_value(game.observe(Player::P0)).unwrap(),
                    serde_json::to_value(game.observe(Player::P1)).unwrap(),
                ];
                assert_eq!(
                    game.apply(player, Action::TapMana { card: id }),
                    Err(Error::IllegalAction)
                );
                assert_eq!(game.snapshot(), before);
                assert_eq!(
                    serde_json::to_value(game.observe(Player::P0)).unwrap(),
                    views[0]
                );
                assert_eq!(
                    serde_json::to_value(game.observe(Player::P1)).unwrap(),
                    views[1]
                );
            }
        }
    }
}
#[test]
fn msh_temporary_copy_funding_effective_copy_token_entry_kinds_govern_tap_counters_and_history() {
    for player in [Player::P0, Player::P1] {
        let mut game = setup(player);
        named(
            &mut game,
            player.other(),
            "Authority of the Consuls",
            Zone::Battlefield,
        );
        named(
            &mut game,
            player,
            "Michelangelo, Weirdness to 11",
            Zone::Battlefield,
        );
        // Bounded creation fixture using an existing physical definition and the
        // runtime token/recipe representation. No new token ID or card producer.
        let id = named(&mut game, player, "Forest", Zone::Outside);
        let before_ref = game.object_ref(id);
        let mut recipe = CopyRecipe::new(Card::from_name("Forest").unwrap());
        recipe.exceptions.base = Some((4, 4));
        recipe.overrides = Some(CopyOverrides {
            kinds: Some(1),
            types: Some(CopyTypes {
                replace: true,
                human_villain: true,
                mercenary: true,
                ..CopyTypes::default()
            }),
            ..CopyOverrides::NONE
        });
        game.state.objects[id as usize].copy_kind = CopyKind::Token;
        game.set_copy_recipe(before_ref, recipe);
        let mut movement = game.entry_move(id, Zone::Battlefield, false);
        movement.effect_counter = Some((CounterKind::PlusOne, 1));
        let snapshot = game.snapshot();
        let world = FundingState::from_game(&game, &goal(player));
        let mut outcomes = Vec::new();
        world.visit_entry_moves(vec![movement.clone()], true, None, &mut |next| {
            outcomes
                .push(characteristic_query::CharacteristicQuery::in_world(next).object_view(id));
            false
        });
        assert_eq!(game.snapshot(), snapshot);
        assert_eq!(outcomes.len(), 1);
        assert!(outcomes[0].tapped);
        assert_eq!(outcomes[0].counters.plus_one, 2);
        let history = game.state.players[player.index()].nonland_permanents_entered_this_turn;
        game.stage_entries(vec![movement], None);
        game.advance_entries();
        settle(&mut game);
        assert!(game.state.objects[id as usize].tapped);
        assert_eq!(game.counter_count(id, CounterKind::PlusOne), 2);
        assert_eq!(game.current_kinds(id), 1);
        assert_eq!(
            game.state.players[player.index()].nonland_permanents_entered_this_turn,
            history + 1
        );
        assert_eq!(game.object_view(id), outcomes.remove(0));
    }
}

#[test]
fn msh_copy_subtype_cda_funding_raw_and_evaluated_keywords_equal_paid_actual() {
    for player in [Player::P0, Player::P1] {
        for blueprint in ["Three Tree Mascot", "Barkform Harvester"] {
            for layer_six in [false, true] {
                let (game, task) =
                    super::super::msh_temporary_copy_cards_tests::paid_subtype_cda_fixture(
                        player, blueprint, layer_six,
                    );
                let snapshot = game.snapshot();
                let mut request = goal(player);
                request.source = Some((
                    game.object_ref(task),
                    0,
                    Counters::default(),
                    game.state.objects[task as usize].card,
                    game.copy_ability_origin(game.object_ref(task)),
                ));
                let world = FundingState::from_game(&game, &request);
                let resource = world.resources.iter().find(|r| r.id == task).unwrap();
                assert_eq!(world.keywords(resource), game.keywords(task));
                assert!(!world.keywords(resource).contains(Keywords::CHANGELING));
                let query = characteristic_query::CharacteristicQuery::in_world(&world);
                assert_eq!(query.object_view(task), game.object_view(task));
                assert!(
                    !ability_world::CharacteristicWorld::subtypes(&world, task).has(
                        ability_world::CharacteristicWorld::definition(&world, task),
                        "Merfolk"
                    )
                );
                assert_eq!(game.snapshot(), snapshot);
            }
        }
    }
}

fn projected_paid_copy_faces_departure_matrix(double: bool) {
    for player in [Player::P0, Player::P1] {
        for (requested, replacement) in [
            (Zone::Graveyard, false),
            (Zone::Exile, false),
            (Zone::Graveyard, true),
        ] {
            let (mut game, id) =
                super::super::msh_copy_identity_regression_tests::paid_face_token_fixture(
                    player, double,
                );
            if replacement {
                if game.actor() != Some(player) {
                    checked(&mut game, Action::Pass);
                }
                let rip = named(&mut game, player, "Rest in Peace", Zone::Hand);
                game.state.players[player.index()].mana = [1, 0, 0, 0, 0, 1];
                cast(&mut game, rip, None, &[]);
                assert_eq!(game.state.players[player.index()].mana, [0; 6]);
                settle(&mut game);
            }
            let secret = named(&mut game, player, "Cancel", Zone::Hand);
            let object = game.object_ref(id);
            let snapshot = game.snapshot();
            let views =
                serde_json::to_value([game.observe(Player::P0), game.observe(Player::P1)]).unwrap();
            let mut actual = {
                let parent = game.clone();
                parent.clone()
            };
            let event = actual.move_object_raw(id, requested);
            let expected = actual.object_view(id);
            assert_ne!(expected.reference, object);
            assert_eq!(
                event.destination,
                if replacement { Zone::Exile } else { requested }
            );
            assert_eq!(expected.face.is_some(), double);
            assert!(expected.is_token());
            assert!(expected.temporary_copy.is_none());
            let mut request = goal(player);
            request.source = Some((
                object,
                0,
                Counters::default(),
                game.state.objects[id as usize].card,
                game.copy_ability_origin(object),
            ));
            let mut world = FundingState::from_game(&game, &request);
            world.prepare_entry_projection();
            assert!(world.resources.iter().any(|r| r.id == id));
            assert!(ability_world::CharacteristicWorld::stored_recipe(&world, id).is_some());
            assert_eq!(
                characteristic_query::CharacteristicQuery::in_world(&world).object_view(id),
                game.object_view(id)
            );
            let mut visits = 0;
            world.cost_departures(&[object], requested, &mut |departed| {
                visits += 1;
                assert_eq!(
                    ability_world::CharacteristicWorld::recipe(departed, id).face_structure(),
                    actual
                        .copy_recipe(expected.reference)
                        .unwrap()
                        .face_structure()
                );
                assert_eq!(
                    characteristic_query::CharacteristicQuery::in_world(departed).object_view(id),
                    expected
                );
                false
            });
            assert_eq!(visits, 1);
            assert_eq!(game.snapshot(), snapshot);
            assert_eq!(
                serde_json::to_value([game.observe(Player::P0), game.observe(Player::P1)]).unwrap(),
                views
            );
            assert!(
                !game
                    .observe(player.other())
                    .objects
                    .iter()
                    .any(|v| v.id == secret)
            );
            actual.restore(&snapshot).unwrap();
            assert_eq!(actual.object_view(id), game.object_view(id));
        }
    }
}
#[test]
fn msh_copy_projected_faces_paid_single_token_full_departure_views_match_actual() {
    projected_paid_copy_faces_departure_matrix(false);
}
#[test]
fn msh_copy_projected_faces_paid_double_token_full_departure_views_match_actual() {
    projected_paid_copy_faces_departure_matrix(true);
}
