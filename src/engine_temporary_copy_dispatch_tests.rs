//! Bounded core instructions and ordinary paid copy/death/attachment consumers.
use super::modal_origin_tests::announce;
use super::*;
use crate::cards::InPlaceCopySpec;
use crate::engine::assurance::big_helpers::{cast, checked, named, settle, setup};

fn bounded_program(game: &mut Game, item: SpellView, program: &'static [Instruction]) {
    // Owned continuation fixture for a private opcode whose producers remain
    // gated. Production dispatch, target-group selection and expiry are real.
    game.state.resolving = Some(programs::Resolution {
        cascade: None,
        target_card_facts: None,
        immediate_blink_return: None,
        explore: None,
        evidence: None,
        connive: None,
        item,
        program,
        pc: 0,
        target_group: 0,
        moved_count: None,
        discard_after: None,
        discards: [0; 2],
        mana_payment: None,
        forage: None,
        selected_discards: [Vec::new(), Vec::new()],
        milled: Vec::new(),
        milled_selection: None,
        library: None,
        selection: None,
        target_library: None,
        search: None,
        optional_searches: None,
        choice: None,
    });
    game.run_program();
}

fn creature_recipe(card: Card) -> CopyRecipe {
    CopyRecipe {
        definition: card,
        exceptions: CopyExceptions {
            base: Some((4, 4)),
            ..CopyExceptions::NONE
        },
        overrides: Some(CopyOverrides {
            kinds: Some(1),
            types: Some(CopyTypes {
                replace: true,
                human_villain: true,
                mercenary: true,
                ..CopyTypes::default()
            }),
            ..CopyOverrides::NONE
        }),
        faces: None,
    }
}
#[test]
fn msh_temporary_copy_bounded_paid_vaultborn_death_applies_later_artifact_exception() {
    for player in [Player::P0, Player::P1] {
        let mut game = setup(player);
        let id = named(&mut game, player, "Hill Giant", Zone::Battlefield);
        game.apply_temporary_copy(
            game.object_ref(id),
            creature_recipe(Card::from_name("Vaultborn Tyrant").unwrap()),
            CopyDeadline::EndOfTurn,
        );
        assert_eq!(game.current_kinds(id), 1);
        announce(&mut game, player, "Murder", &[Target::Card(id)]);
        settle(&mut game);
        assert_eq!(game.state.objects[id as usize].zone, Zone::Graveyard);
        assert_eq!(game.state.objects[id as usize].card.name(), "Hill Giant");
        let copies: Vec<_> = game
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(_, object)| object.zone == Zone::Battlefield && object.is_copy_token())
            .map(|(id, _)| id as CardId)
            .collect();
        assert_eq!(copies.len(), 1);
        let token = game.object_view(copies[0]);
        assert!(token.is_token());
        assert_eq!(token.kinds & 17, 17);
        assert!(
            token.has_subtype("Human")
                && token.has_subtype("Mercenary")
                && token.has_subtype("Villain")
        );
        assert!(!token.has_subtype("Dinosaur"));
        assert!(token.copy_recipe.unwrap().exceptions.artifact);
    }
}
#[test]
fn msh_temporary_copy_bounded_paid_molten_copy_adds_artifact_after_replacement_types() {
    for player in [Player::P0, Player::P1] {
        let mut game = setup(player);
        let id = named(&mut game, player, "Hill Giant", Zone::Battlefield);
        game.apply_temporary_copy(
            game.object_ref(id),
            creature_recipe(Card::from_name("Bear Cub").unwrap()),
            CopyDeadline::EndOfTurn,
        );
        announce(&mut game, player, "Molten Duplication", &[Target::Card(id)]);
        settle(&mut game);
        let token = game
            .state
            .objects
            .iter()
            .enumerate()
            .find(|(_, object)| object.zone == Zone::Battlefield && object.is_copy_token())
            .map(|(id, _)| id as CardId)
            .unwrap();
        assert_eq!(game.current_kinds(token) & 17, 17);
        assert!(game.keywords(token).contains(Keywords::HASTE));
        assert_eq!(
            game.copiable_recipe(game.object_ref(token))
                .overrides
                .unwrap()
                .kinds,
            Some(17)
        );
        assert_eq!(game.current_kinds(id), 1);
    }
}
#[test]
fn msh_temporary_copy_bounded_paid_equipment_losing_equipment_detaches_without_rebinding() {
    for player in [Player::P0, Player::P1] {
        let mut game = setup(player);
        let pole = named(&mut game, player, "Fishing Pole", Zone::Battlefield);
        let bear = named(&mut game, player, "Bear Cub", Zone::Battlefield);
        checked(
            &mut game,
            Action::AnnounceActivate {
                card: pole,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        checked(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        while game.state.announcing.is_some() {
            let action = game
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PayCosts { .. } | Action::FinishTargets))
                .unwrap();
            checked(&mut game, action);
        }
        settle(&mut game);
        let reference = game.object_ref(pole);
        game.apply_temporary_copy(
            reference,
            CopyRecipe::new(Card::from_name("Arc Reactor").unwrap()),
            CopyDeadline::EndOfTurn,
        );
        assert!(game.attachment_recipient(pole).is_none());
        game.state_based_actions();
        assert!(game.on_battlefield(pole));
        assert_eq!(game.state.objects[pole as usize].attached_to, None);
        assert!(game.expire_temporary_copies(true));
        assert_eq!(game.object_ref(pole), reference);
        assert_eq!(game.state.objects[pole as usize].attached_to, None);
        assert!(!game.legal_actions().iter().any(|action| matches!(action,
            Action::AnnounceAttachedAbility { grantor, .. } if *grantor == reference)));
    }
}
#[test]
fn msh_temporary_copy_bounded_paid_aura_distinguishes_creature_aura_from_removed_aura_type() {
    for player in [Player::P0, Player::P1] {
        for retain_aura in [false, true] {
            let mut game = setup(player);
            let bear = named(&mut game, player, "Hill Giant", Zone::Battlefield);
            let aura = named(&mut game, player, "Witness Protection", Zone::Hand);
            cast(&mut game, aura, None, &[Target::Card(bear)]);
            settle(&mut game);
            let reference = game.object_ref(aura);
            let mut recipe = creature_recipe(Card::from_name("Witness Protection").unwrap());
            if retain_aura {
                let value = recipe.overrides.as_mut().unwrap();
                value.kinds = Some(33);
                value.types = Some(CopyTypes {
                    human_villain: true,
                    ..CopyTypes::default()
                });
            }
            game.apply_temporary_copy(reference, recipe, CopyDeadline::EndOfTurn);
            assert_eq!(game.is_enchanted(bear), retain_aura);
            game.state_based_actions();
            assert_eq!(
                game.state.objects[aura as usize].zone,
                if retain_aura {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
            assert_eq!(game.state.objects[aura as usize].attached_to, None);
            assert!(!game.is_enchanted(bear));
            assert_eq!(game.object_view(bear).name(), "Hill Giant");
        }
    }
}

#[test]
fn msh_temporary_copy_bounded_typed_instruction_composes_groups_source_and_flat_exceptions() {
    for player in [Player::P0, Player::P1] {
        let mut game = setup(player);
        let source = named(&mut game, player, "Hill Giant", Zone::Battlefield);
        let blueprint = named(&mut game, player, "Fishing Pole", Zone::Battlefield);
        let mut item = game.activation_item(player, source, 0);
        item.targets.push(BoundTarget {
            group: 1,
            target: Target::Card(blueprint),
            object: Some(game.object_ref(blueprint)),
        });
        const SPEC: InPlaceCopySpec = InPlaceCopySpec {
            recipient_group: None,
            blueprint_group: 1,
            overrides: CopyOverrides {
                name: Some("Absorbing Man"),
                legendary: Some(true),
                types: Some(CopyTypes {
                    replace: false,
                    human_villain: true,
                    mercenary: false,
                    nightmare: false,
                    golem: false,
                    fractal: false,
                }),
                keywords: Keywords::VIGILANCE,
                ..CopyOverrides::NONE
            },
            add_kinds: 1,
            base: Some((4, 4)),
            next_turn: true,
        };
        // A bounded immutable opcode fixture; no producer definition is admitted.
        bounded_program(&mut game, item.clone(), &[Instruction::BecomeCopy(SPEC)]);
        let view = game.object_view(source);
        assert_eq!(view.name(), "Absorbing Man");
        assert!(view.is_legendary() && view.has_subtype("Equipment") && view.has_subtype("Human"));
        assert_eq!((view.power, view.toughness), (4, 4));
        assert_eq!(view.kinds & 17, 17);
        assert!(view.keywords.contains(Keywords::VIGILANCE));
        let old = game.copiable_recipe(view.reference);
        // Both required groups may name one object, and no prior copy layer is nested.
        item.targets.push(BoundTarget {
            group: 0,
            target: Target::Card(source),
            object: Some(view.reference),
        });
        item.targets[0] = BoundTarget {
            group: 1,
            target: Target::Card(source),
            object: Some(view.reference),
        };
        const SELF_SPEC: InPlaceCopySpec = InPlaceCopySpec {
            recipient_group: Some(0),
            blueprint_group: 1,
            overrides: CopyOverrides {
                legendary: Some(false),
                ..CopyOverrides::NONE
            },
            add_kinds: 0,
            base: None,
            next_turn: false,
        };
        bounded_program(
            &mut game,
            item.clone(),
            &[Instruction::BecomeCopy(SELF_SPEC)],
        );
        assert!(!game.object_view(source).is_legendary());
        assert_eq!(
            game.copiable_recipe(view.reference).definition,
            old.definition
        );
        assert_eq!(
            game.copiable_recipe(view.reference).overrides.unwrap().name,
            old.overrides.unwrap().name
        );
        let before = game.object_view(source);
        item.targets.retain(|bound| bound.group != 1);
        bounded_program(&mut game, item, &[Instruction::BecomeCopy(SPEC)]);
        assert_eq!(game.object_view(source), before);
    }
}

#[test]
fn msh_temporary_copy_bounded_copied_land_providers_refresh_without_creating_entry_events() {
    for player in [Player::P0, Player::P1] {
        let mut game = setup(player);
        let id = named(&mut game, player, "Hill Giant", Zone::Battlefield);
        let library = named(&mut game, player, "Forest", Zone::Library);
        let graveyard = named(&mut game, player, "Forest", Zone::Graveyard);
        let reference = game.object_ref(id);
        game.apply_temporary_copy(
            reference,
            CopyRecipe::new(Card::from_name("Ka-Zar of the Savage Land").unwrap()),
            CopyDeadline::NextTurn {
                player,
                created_turn: game.state.turn,
            },
        );
        assert!(game.state.stack.is_empty() && game.state.pending_triggers.is_empty());
        assert!(
            !game
                .state
                .objects
                .iter()
                .any(|object| object.zone == Zone::Battlefield && object.is_token())
        );
        assert_eq!(
            game.observe(player).known_library_top[player.index()]
                .unwrap()
                .object,
            game.object_ref(library)
        );
        assert!(game.observe(player.other()).known_library_top[player.index()].is_none());
        assert!(
            game.legal_actions()
                .contains(&Action::PlayLand { card: library })
        );
        game.apply_temporary_copy(
            reference,
            CopyRecipe::new(Card::from_name("Mole Man, Moloid Master").unwrap()),
            CopyDeadline::EndOfTurn,
        );
        assert!(game.observe(player).known_library_top[player.index()].is_none());
        assert!(
            !game
                .legal_actions()
                .contains(&Action::PlayLand { card: library })
        );
        checked(&mut game, Action::PlayLand { card: graveyard });
        settle(&mut game);
        assert!(
            game.state
                .objects
                .iter()
                .any(|object| object.zone == Zone::Battlefield
                    && object.card == Card::from_name("@token/msh-moloid").unwrap())
        );
        assert!(game.expire_temporary_copies(true));
        assert_eq!(game.object_ref(id), reference);
        assert_eq!(
            game.observe(player).known_library_top[player.index()]
                .unwrap()
                .object,
            game.object_ref(library)
        );
        // The ordinary global land allowance remains consumed after restoration.
        assert!(
            !game
                .legal_actions()
                .contains(&Action::PlayLand { card: library })
        );
    }
}
#[test]
fn msh_temporary_copy_bounded_copied_cost_provider_actual_payment_and_funding_are_live() {
    for player in [Player::P0, Player::P1] {
        let mut game = setup(player);
        let id = named(&mut game, player, "Hill Giant", Zone::Battlefield);
        let victim = named(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
        let murder = named(&mut game, player, "Murder", Zone::Hand);
        game.state.players[player.index()].mana = [0, 0, 2, 0, 0, 0];
        let before = game.snapshot();
        assert_eq!(
            game.apply(player, Action::AnnounceCast { card: murder }),
            Err(Error::IllegalAction)
        );
        assert_eq!(game.snapshot(), before);
        game.apply_temporary_copy(
            game.object_ref(id),
            CopyRecipe::new(Card::from_name("Mocking Sprite").unwrap()),
            CopyDeadline::EndOfTurn,
        );
        assert!(
            game.legal_actions()
                .contains(&Action::AnnounceCast { card: murder })
        );
        cast(&mut game, murder, None, &[Target::Card(victim)]);
        settle(&mut game);
        assert_eq!(game.state.players[player.index()].mana, [0; 6]);
        assert_eq!(game.state.objects[victim as usize].zone, Zone::Graveyard);
        assert!(game.expire_temporary_copies(true));
        let another = named(&mut game, player, "Murder", Zone::Hand);
        game.state.players[player.index()].mana = [0, 0, 2, 0, 0, 0];
        if game.actor() != Some(player) {
            checked(&mut game, Action::Pass);
        }
        let before = game.snapshot();
        assert_eq!(
            game.apply(player, Action::AnnounceCast { card: another }),
            Err(Error::IllegalAction)
        );
        assert_eq!(game.snapshot(), before);
    }
}
#[test]
fn msh_temporary_copy_bounded_paid_forge_links_return_only_to_the_original_intrinsic_origin() {
    use crate::engine::territory_forge_tests::{finish, forge};
    for player in [Player::P0, Player::P1] {
        let mut game = setup(player);
        let land = named(&mut game, player.other(), "Forest", Zone::Battlefield);
        let id = forge(&mut game, player, land);
        let reference = game.object_ref(id);
        let exiled = game.object_ref(land);
        assert!(
            game.borrowed_ability_views()
                .iter()
                .any(|ability| ability.source == reference)
        );
        game.apply_temporary_copy(
            reference,
            CopyRecipe::new(Card::from_name("Territory Forge").unwrap()),
            CopyDeadline::EndOfTurn,
        );
        assert!(game.borrowed_ability_views().is_empty());
        assert!(game.state.pending_triggers.is_empty());
        assert!(game.expire_temporary_copies(true));
        assert!(
            game.borrowed_ability_views()
                .iter()
                .any(|ability| ability.source == reference)
        );
        game.state.players[player.index()].mana = [0; 6];
        assert!(game.can_fund(FundingGoal {
            actor: player,
            required: [0, 0, 0, 0, 1, 0],
            generic: 0,
            reserve: None,
            source: None
        }));
        if game.actor() != Some(player) {
            checked(&mut game, Action::Pass);
        }
        checked(
            &mut game,
            Action::AnnounceBorrowedMana {
                card: id,
                exiled,
                ability: u16::MAX,
                counter_payment: Counters::default(),
                production: [0, 0, 0, 0, 1, 0],
            },
        );
        finish(&mut game);
        assert_eq!(game.state.players[player.index()].mana, [0, 0, 0, 0, 1, 0]);
        assert_eq!(game.object_ref(id), reference);
    }
}
#[test]
fn msh_temporary_copy_bounded_saga_acquisition_has_no_entry_lore_and_queued_chapter_survives_expiry()
 {
    for player in [Player::P0, Player::P1] {
        let mut game = setup(player);
        let id = named(&mut game, player, "Arc Reactor", Zone::Battlefield);
        let reference = game.object_ref(id);
        let recipe = CopyRecipe::new(Card::from_name("Avengers: Under Siege").unwrap())
            .with_exceptions(CopyExceptions::ARTIFACT);
        game.apply_temporary_copy(reference, recipe, CopyDeadline::EndOfTurn);
        assert_eq!(game.counter_count(id, CounterKind::Lore), 0);
        assert!(game.state.pending_triggers.is_empty());
        // Explicit bounded lore placement uses the ordinary replacement/trigger
        // pipeline; it does not claim an additional admitted lore-placing card.
        game.effect_counter(id, CounterKind::Lore, 1, replacements::ResumeMode::Pending);
        assert_eq!(game.state.pending_triggers.len(), 1);
        let packet = game.state.pending_triggers[0].clone();
        assert_eq!(
            packet.source_ability_origin,
            game.copy_ability_origin(reference)
        );
        assert!(game.expire_temporary_copies(true));
        assert_eq!(game.object_view(id).name(), "Arc Reactor");
        settle(&mut game);
        assert_eq!(
            game.state
                .objects
                .iter()
                .filter(|object| object.zone == Zone::Battlefield && object.is_token())
                .count(),
            2
        );
        assert_eq!(game.counter_count(id, CounterKind::Lore), 1);
        assert_eq!(game.object_ref(id), reference);
    }
}
