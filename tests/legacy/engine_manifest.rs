use super::*;
use super::*;
pub use rustmage_core::engine::manifest::*;
#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::assurance::big_helpers::{named, setup};

    #[test]
    fn bubble_smuggler_face_up_counters_obey_removal_and_counter_replacement() {
        for (blanked, has_replacement, expected) in
            [(false, false, 4), (true, false, 0), (false, true, 5)]
        {
            let mut game = setup(Player::P0);
            let id = named(&mut game, Player::P0, "Bear Cub", Zone::Library);
            // Bubble Smuggler is a fixture here: its full disguise behavior is
            // deliberately still gated from deck admission.
            game.state.objects[id as usize].card =
                Card::from_name("Bubble Smuggler").expect("pinned catalogue identity");
            let movement = game
                .manifest_entry(game.object_ref(id), Player::P0)
                .unwrap();
            game.stage_entries(vec![movement], None);
            assert!(!game.advance_entries());
            let object = game.object_ref(id);
            if blanked {
                let aura = named(
                    &mut game,
                    Player::P1,
                    "Witness Protection",
                    Zone::Battlefield,
                );
                game.state.objects[aura as usize].attached_to = Some(object);
            }
            if has_replacement {
                named(
                    &mut game,
                    Player::P0,
                    "Doc Samson, Super Psychiatrist",
                    Zone::Battlefield,
                );
            }
            assert!(game.turn_manifest_face_up(object));
            assert!(!game.advance_replacements(false));
            assert_eq!(game.state.objects[id as usize].counters.plus_one, expected);
            assert_eq!(
                game.state.manifest.as_ref().unwrap().last_face_up,
                Some(object)
            );
        }
    }

    #[test]
    fn turn_up_event_waits_for_counter_replacement_choice() {
        let mut game = setup(Player::P0);
        let id = named(&mut game, Player::P0, "Bear Cub", Zone::Library);
        game.state.objects[id as usize].card =
            Card::from_name("Bubble Smuggler").expect("pinned catalogue identity");
        let movement = game
            .manifest_entry(game.object_ref(id), Player::P0)
            .unwrap();
        game.stage_entries(vec![movement], None);
        assert!(!game.advance_entries());
        let object = game.object_ref(id);
        for _ in 0..2 {
            named(
                &mut game,
                Player::P0,
                "Doc Samson, Super Psychiatrist",
                Zone::Battlefield,
            );
        }
        assert!(game.turn_manifest_face_up(object));
        assert!(game.advance_replacements(false));
        assert!(game.is_face_down(object));
        assert_eq!(
            game.object_view(id).card,
            Card::from_name("@face-down/manifest").expect("pinned catalogue identity")
        );
        assert_eq!(game.state.manifest.as_ref().unwrap().last_face_up, None);
        assert_eq!(game.state.objects[id as usize].counters.plus_one, 0);
        let action = game
            .legal_actions()
            .into_iter()
            .find(|action| matches!(action, Action::ChooseReplacement { .. }))
            .unwrap();
        game.apply(Player::P0, action).unwrap();
        assert_eq!(game.state.objects[id as usize].counters.plus_one, 6);
        assert_eq!(
            game.state.manifest.as_ref().unwrap().last_face_up,
            Some(object)
        );
    }

    #[test]
    fn face_up_replacement_uses_prospective_copy_characteristics() {
        for (physical, copy, expected) in [
            (
                Card::from_name("Bubble Smuggler").expect("pinned catalogue identity"),
                Card::Forest,
                0,
            ),
            (
                Card::Forest,
                Card::from_name("Bubble Smuggler").expect("pinned catalogue identity"),
                4,
            ),
        ] {
            let mut game = setup(Player::P0);
            let id = named(&mut game, Player::P0, "Bear Cub", Zone::Library);
            game.state.objects[id as usize].card = physical;
            let movement = game
                .manifest_entry(game.object_ref(id), Player::P0)
                .unwrap();
            game.stage_entries(vec![movement], None);
            assert!(!game.advance_entries());
            let object = game.object_ref(id);
            game.apply_temporary_copy(object, CopyRecipe::new(copy), CopyDeadline::EndOfTurn);
            assert!(game.turn_manifest_face_up(object));
            assert!(!game.advance_replacements(false));
            assert_eq!(game.state.objects[id as usize].counters.plus_one, expected);
            assert_eq!(game.object_view(id).card, copy);
        }
    }

    #[test]
    fn paid_face_up_waits_for_replacement_choice_and_restores_snapshot() {
        let mut game = setup(Player::P0);
        let id = named(&mut game, Player::P0, "Bear Cub", Zone::Library);
        game.state.objects[id as usize].card =
            Card::from_name("Bubble Smuggler").expect("pinned catalogue identity");
        let movement = game
            .manifest_entry(game.object_ref(id), Player::P0)
            .unwrap();
        game.stage_entries(vec![movement], None);
        assert!(!game.advance_entries());
        let object = game.object_ref(id);
        for _ in 0..2 {
            named(
                &mut game,
                Player::P0,
                "Doc Samson, Super Psychiatrist",
                Zone::Battlefield,
            );
        }
        let begin = game
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(a,
            Action::BeginFaceUp { object: target, cost } if *target == object && cost.generic == 1)
            })
            .unwrap();
        game.apply(Player::P0, begin).unwrap();
        let payment = game
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayFaceUp { .. }))
            .unwrap();
        let before_payment = game.snapshot();
        let mut branch = game.clone();
        game.apply(Player::P0, payment.clone()).unwrap();
        branch.apply(Player::P0, payment.clone()).unwrap();
        assert_eq!(game.snapshot(), branch.snapshot());
        assert!(matches!(
            game.state.pending,
            Pending::Replacement { player: Player::P0 }
        ));
        assert!(game.is_face_down(object));
        assert_eq!(game.state.objects[id as usize].counters.plus_one, 0);
        let pending = game.snapshot();
        let choice = game
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::ChooseReplacement { .. }))
            .unwrap();
        game.apply(Player::P0, choice.clone()).unwrap();
        let finished = game.snapshot();
        assert!(!game.is_face_down(object));
        assert_eq!(game.state.objects[id as usize].counters.plus_one, 6);
        game.restore(&pending).unwrap();
        game.apply(Player::P0, choice).unwrap();
        assert_eq!(game.snapshot(), finished);
        game.restore(&before_payment).unwrap();
        game.apply(Player::P0, payment).unwrap();
        assert!(matches!(
            game.state.pending,
            Pending::Replacement { player: Player::P0 }
        ));
    }

    #[test]
    fn manifested_disguise_card_offers_its_mana_and_disguise_costs() {
        let mut game = setup(Player::P0);
        let id = named(&mut game, Player::P0, "Bear Cub", Zone::Library);
        game.state.objects[id as usize].card =
            Card::from_name("Bubble Smuggler").expect("pinned catalogue identity");
        let movement = game
            .manifest_entry(game.object_ref(id), Player::P0)
            .unwrap();
        game.stage_entries(vec![movement], None);
        assert!(!game.advance_entries());
        let object = game.object_ref(id);
        let natural = ManaCost {
            colored: [0, 1, 0, 0, 0, 0],
            generic: 1,
        };
        let disguise = ManaCost {
            colored: [0, 1, 0, 0, 0, 0],
            generic: 5,
        };
        let actions = game.legal_actions();
        assert!(actions.contains(&Action::BeginFaceUp {
            object,
            cost: natural
        }));
        assert!(actions.contains(&Action::BeginFaceUp {
            object,
            cost: disguise
        }));
        game.apply(
            Player::P0,
            Action::BeginFaceUp {
                object,
                cost: disguise,
            },
        )
        .unwrap();
        assert_eq!(game.face_up_payment().unwrap().cost, disguise);
        let payment = game
            .legal_actions()
            .into_iter()
            .find(|action| matches!(action, Action::PayFaceUp { .. }))
            .unwrap();
        game.apply(Player::P0, payment).unwrap();
        assert!(!game.is_face_down(object));
        assert_eq!(game.state.objects[id as usize].counters.plus_one, 4);
    }

    #[test]
    fn disguised_face_down_ward_is_copiable_and_removed_by_blanking() {
        use crate::engine::assurance::big_helpers::{cast, checked, settle};
        for blanked in [false, true] {
            let mut game = setup(Player::P0);
            let id = named(&mut game, Player::P0, "Bear Cub", Zone::Library);
            let mut movement = game
                .manifest_entry(game.object_ref(id), Player::P0)
                .unwrap();
            movement.before.card =
                Card::from_name("@face-down/disguise").expect("pinned catalogue identity");
            game.stage_entries(vec![movement], None);
            assert!(!game.advance_entries());
            let object = game.object_ref(id);
            assert_eq!(
                game.copiable_recipe(object).definition,
                Card::from_name("@face-down/disguise").expect("pinned catalogue identity")
            );
            if blanked {
                let aura = named(
                    &mut game,
                    Player::P1,
                    "Witness Protection",
                    Zone::Battlefield,
                );
                game.state.objects[aura as usize].attached_to = Some(object);
            }
            game.state.priority = Player::P1;
            let spell = named(&mut game, Player::P1, "Unsummon", Zone::Hand);
            cast(&mut game, spell, None, &[Target::Card(id)]);
            if !blanked {
                for _ in 0..16 {
                    if matches!(game.state.pending, Pending::ResolutionMana { .. }) {
                        break;
                    }
                    checked(&mut game, Action::Pass);
                }
                assert!(matches!(game.state.pending, Pending::ResolutionMana { .. }));
                checked(&mut game, Action::DeclineResolutionMana);
            }
            settle(&mut game);
            assert_eq!(game.state.objects[spell as usize].zone, Zone::Graveyard);
            assert_eq!(game.on_battlefield(id), !blanked);
        }
    }

    #[test]
    fn disguise_cast_keeps_identity_private_and_transfers_to_battlefield() {
        use crate::engine::assurance::big_helpers::settle;
        let mut game = setup(Player::P0);
        let id = named(&mut game, Player::P0, "Bear Cub", Zone::Hand);
        game.state.objects[id as usize].card =
            Card::from_name("Bubble Smuggler").expect("pinned catalogue identity");
        let announce = Action::AnnounceDisguise { card: id };
        assert!(game.legal_actions().contains(&announce));
        let before = game.snapshot();
        game.apply(Player::P0, announce.clone()).unwrap();
        assert_eq!(
            Game::announcement_base_cost(game.state.announcing.as_ref().unwrap()),
            ManaCost {
                colored: [0; 6],
                generic: 3
            }
        );
        assert_eq!(
            game.face_down_spell_views(Player::P0)[0].underlying,
            Some(Card::from_name("Bubble Smuggler").expect("pinned catalogue identity"))
        );
        assert_eq!(game.face_down_spell_views(Player::P1)[0].underlying, None);
        assert_eq!(
            game.object_view(id).card,
            Card::from_name("@face-down/disguise").expect("pinned catalogue identity")
        );
        let payment = game
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        game.apply(Player::P0, payment).unwrap();
        settle(&mut game);
        let object = game.object_ref(id);
        assert!(game.is_face_down(object));
        assert_eq!(
            game.object_view(id).card,
            Card::from_name("@face-down/disguise").expect("pinned catalogue identity")
        );
        assert_eq!(game.face_down_views(Player::P1)[0].underlying, None);
        game.restore(&before).unwrap();
        assert_eq!(
            game.state.objects[id as usize].card,
            Card::from_name("Bubble Smuggler").expect("pinned catalogue identity")
        );
        assert!(game.face_down_spell_views(Player::P0).is_empty());
        assert!(game.legal_actions().contains(&announce));
    }

    #[test]
    fn manifested_noncreature_with_disguise_can_pay_only_its_disguise_cost() {
        let mut game = setup(Player::P0);
        let id = named(&mut game, Player::P0, "Bear Cub", Zone::Library);
        let physical = Card::from_name("Concealed Weapon").unwrap();
        game.state.objects[id as usize].card = physical;
        let movement = game
            .manifest_entry(game.object_ref(id), Player::P0)
            .unwrap();
        game.stage_entries(vec![movement], None);
        assert!(!game.advance_entries());
        let object = game.object_ref(id);
        let printed = ManaCost {
            colored: [0, 0, 0, 1, 0, 0],
            generic: 1,
        };
        let disguise = ManaCost {
            colored: [0, 0, 0, 1, 0, 0],
            generic: 2,
        };
        let actions = game.legal_actions();
        assert!(!actions.contains(&Action::BeginFaceUp {
            object,
            cost: printed
        }));
        assert!(actions.contains(&Action::BeginFaceUp {
            object,
            cost: disguise
        }));
        game.apply(
            Player::P0,
            Action::BeginFaceUp {
                object,
                cost: disguise,
            },
        )
        .unwrap();
        let payment = game
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayFaceUp { .. }))
            .unwrap();
        game.apply(Player::P0, payment).unwrap();
        assert!(!game.is_face_down(object));
        assert_eq!(game.object_view(id).card, physical);
        assert_ne!(game.current_kinds(id) & 16, 0);
    }

    #[test]
    fn flash_permission_allows_disguise_cast_at_instant_speed() {
        let mut game = setup(Player::P0);
        let id = named(&mut game, Player::P0, "Bear Cub", Zone::Hand);
        game.state.objects[id as usize].card = Card::from_name("Bubble Smuggler").unwrap();
        game.state.active = Player::P1;
        game.state.priority = Player::P0;
        assert!(
            !game
                .legal_actions()
                .contains(&Action::AnnounceDisguise { card: id })
        );
        named(
            &mut game,
            Player::P0,
            "High Fae Trickster",
            Zone::Battlefield,
        );
        assert!(
            game.legal_actions()
                .contains(&Action::AnnounceDisguise { card: id })
        );
        game.apply(Player::P0, Action::AnnounceDisguise { card: id })
            .unwrap();
        assert_eq!(game.face_down_spell_views(Player::P1)[0].underlying, None);
    }

    #[test]
    fn disguise_from_public_zones_requires_a_paid_cast_permission() {
        use crate::engine::assurance::big_helpers::settle;
        for origin in [Zone::Graveyard, Zone::Exile] {
            let mut game = setup(Player::P0);
            let id = named(&mut game, Player::P0, "Bear Cub", origin);
            let physical = Card::from_name("Bubble Smuggler").unwrap();
            game.state.objects[id as usize].card = physical;
            let reference = game.object_ref(id);
            let action = Action::AnnounceDisguise { card: id };
            assert!(!game.legal_actions().contains(&action));
            match origin {
                Zone::Graveyard => {
                    game.grant_graveyard_permission(Player::P0, reference, GraveyardCastKind::Cast)
                }
                Zone::Exile => game.grant_exile_play_permission(Player::P0, reference, false),
                _ => unreachable!(),
            }
            assert!(game.legal_actions().contains(&action));
            game.apply(Player::P0, action).unwrap();
            assert_eq!(
                game.face_down_spell_views(Player::P1)[0].underlying,
                Some(physical)
            );
            let pay = game
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PayCosts { .. }))
                .unwrap();
            game.apply(Player::P0, pay).unwrap();
            settle(&mut game);
            assert_eq!(
                game.face_down_views(Player::P1)[0].underlying,
                Some(physical)
            );
        }
    }

    #[test]
    fn disguise_uses_and_consumes_creature_graveyard_type_permission() {
        use crate::engine::assurance::big_helpers::settle;
        let mut game = setup(Player::P0);
        let first = named(&mut game, Player::P0, "Bear Cub", Zone::Graveyard);
        let second = named(&mut game, Player::P0, "Bear Cub", Zone::Graveyard);
        let physical = Card::from_name("Bubble Smuggler").unwrap();
        game.state.objects[first as usize].card = physical;
        game.state.objects[second as usize].card = physical;
        let muldrotha = named(
            &mut game,
            Player::P0,
            "Muldrotha, the Gravetide",
            Zone::Battlefield,
        );
        let permission = game.object_ref(muldrotha);
        let action = Action::AnnounceDisguiseGraveyard {
            card: first,
            permission,
            permanent_type: PermanentType::Creature,
        };
        assert!(game.legal_actions().contains(&action));
        game.apply(Player::P0, action).unwrap();
        let pay = game
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        game.apply(Player::P0, pay).unwrap();
        settle(&mut game);
        assert!(game.is_face_down(game.object_ref(first)));
        assert!(
            !game
                .legal_actions()
                .contains(&Action::AnnounceDisguiseGraveyard {
                    card: second,
                    permission,
                    permanent_type: PermanentType::Creature
                })
        );
    }

    #[test]
    fn disguise_preserves_exile_permission_modifiers() {
        let mut game = setup(Player::P0);
        let id = named(&mut game, Player::P1, "Bear Cub", Zone::Exile);
        let physical = Card::from_name("Bubble Smuggler").unwrap();
        game.state.objects[id as usize].card = physical;
        named(&mut game, Player::P0, "Bear Cub", Zone::Battlefield);
        let reference = game.object_ref(id);
        game.grant_exile_play_permission(Player::P0, reference, false);
        let permission = game
            .state
            .exile_permissions
            .iter_mut()
            .find(|row| row.object == reference)
            .unwrap();
        permission.requires_subtype = Some("Bear");
        permission.spend_any_mana = true;
        permission.exile_on_graveyard = true;
        let action = Action::AnnounceDisguise { card: id };
        assert!(game.legal_actions().contains(&action));
        game.apply(Player::P0, action).unwrap();
        let info = game.state.announcing.as_ref().unwrap().item.cast.unwrap();
        assert!(info.spend_any_mana);
        assert!(info.exile_on_graveyard);
        assert_eq!(info.origin, Zone::Exile);
        assert_eq!(
            game.face_down_spell_views(Player::P1)[0].underlying,
            Some(physical)
        );
    }

    #[test]
    fn disguise_does_not_combine_with_a_free_only_exile_permission() {
        let mut game = setup(Player::P0);
        let id = named(&mut game, Player::P0, "Bear Cub", Zone::Exile);
        game.state.objects[id as usize].card = Card::from_name("Bubble Smuggler").unwrap();
        let reference = game.object_ref(id);
        game.grant_exile_play_permission(Player::P0, reference, false);
        game.state
            .exile_permissions
            .iter_mut()
            .find(|row| row.object == reference)
            .unwrap()
            .without_paying_mana_cost = true;
        assert!(
            !game
                .legal_actions()
                .contains(&Action::AnnounceDisguise { card: id })
        );
    }

    #[test]
    fn disguise_from_library_top_obeys_creature_cast_permission() {
        let mut game = setup(Player::P0);
        let id = named(&mut game, Player::P0, "Bear Cub", Zone::Library);
        game.state.objects[id as usize].card = Card::from_name("Bubble Smuggler").unwrap();
        let action = Action::AnnounceDisguise { card: id };
        assert!(!game.legal_actions().contains(&action));
        named(
            &mut game,
            Player::P0,
            "Vizier of the Menagerie",
            Zone::Battlefield,
        );
        assert!(game.legal_actions().contains(&action));
    }

    #[test]
    fn recognized_disguise_remains_public_knowledge_across_its_cast() {
        use crate::engine::assurance::big_helpers::settle;
        let mut game = setup(Player::P0);
        let id = named(&mut game, Player::P0, "Bear Cub", Zone::Hand);
        game.state.objects[id as usize].card =
            Card::from_name("Bubble Smuggler").expect("pinned catalogue identity");
        let reference = game.object_ref(id);
        game.remember_public_hand(reference);
        game.apply(Player::P0, Action::AnnounceDisguise { card: id })
            .unwrap();
        assert_eq!(
            game.face_down_spell_views(Player::P1)[0].underlying,
            Some(Card::from_name("Bubble Smuggler").expect("pinned catalogue identity"))
        );
        let payment = game
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        game.apply(Player::P0, payment).unwrap();
        settle(&mut game);
        assert_eq!(
            game.face_down_views(Player::P1)[0].underlying,
            Some(Card::from_name("Bubble Smuggler").expect("pinned catalogue identity"))
        );
    }

    #[test]
    fn copy_of_disguise_spell_becomes_face_up_ward_creature_token() {
        use crate::engine::assurance::big_helpers::settle;
        let mut game = setup(Player::P0);
        let id = named(&mut game, Player::P0, "Bear Cub", Zone::Hand);
        game.state.objects[id as usize].card =
            Card::from_name("Bubble Smuggler").expect("pinned catalogue identity");
        game.apply(Player::P0, Action::AnnounceDisguise { card: id })
            .unwrap();
        let payment = game
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        game.apply(Player::P0, payment).unwrap();
        let original = game.state.stack.last().unwrap().clone();
        let copy = game.allocate_spell_copy(&original, Player::P0);
        let copied_id = copy.source.unwrap().id;
        game.state.stack.push(copy);
        settle(&mut game);
        assert!(game.is_face_down(game.object_ref(id)));
        assert!(!game.is_face_down(game.object_ref(copied_id)));
        assert!(game.state.objects[copied_id as usize].is_token());
        assert_eq!(
            game.object_view(copied_id).card,
            Card::from_name("@face-down/disguise").expect("pinned catalogue identity")
        );
        assert!(
            game.object_view(copied_id)
                .card
                .triggers()
                .iter()
                .any(|a| a.condition.event() == TriggerCondition::Ward)
        );
    }

    #[test]
    fn countered_disguise_is_revealed_and_restores_physical_card() {
        use crate::engine::assurance::big_helpers::{cast, settle};
        let mut game = setup(Player::P0);
        let id = named(&mut game, Player::P0, "Bear Cub", Zone::Hand);
        game.state.objects[id as usize].card =
            Card::from_name("Bubble Smuggler").expect("pinned catalogue identity");
        game.apply(Player::P0, Action::AnnounceDisguise { card: id })
            .unwrap();
        let payment = game
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        game.apply(Player::P0, payment).unwrap();
        game.apply(Player::P0, Action::Pass).unwrap();
        let cancel = named(&mut game, Player::P1, "Cancel", Zone::Hand);
        cast(&mut game, cancel, None, &[Target::Spell(id)]);
        settle(&mut game);
        assert_eq!(game.state.objects[id as usize].zone, Zone::Graveyard);
        assert_eq!(
            game.state.objects[id as usize].card,
            Card::from_name("Bubble Smuggler").expect("pinned catalogue identity")
        );
        assert!(game.face_down_spell_views(Player::P1).is_empty());
    }

    #[test]
    fn turn_inside_out_boost_and_delayed_manifest_obey_death_destination() {
        use crate::engine::assurance::big_helpers::{checked, settle};
        for destination in [Zone::Graveyard, Zone::Exile] {
            let mut game = setup(Player::P0);
            game.state.players[Player::P0.index()].library.clear();
            named(&mut game, Player::P0, "Island", Zone::Library);
            named(&mut game, Player::P0, "Bear Cub", Zone::Library);
            let target = named(&mut game, Player::P0, "Bear Cub", Zone::Battlefield);
            let spell = named(&mut game, Player::P0, "Shock", Zone::Hand);
            game.state.objects[spell as usize].card = Card::from_name("Turn Inside Out").unwrap();
            // The card is deliberately gated; exercise its authored rules by
            // starting its normal paid announcement directly in this fixture.
            let item = game.cast_item(Player::P0, spell, [0; 6]);
            game.open_payment_announcement(item, Counters::default(), None);
            checked(
                &mut game,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
            let pay = game
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PayCosts { .. }))
                .unwrap();
            checked(&mut game, pay);
            settle(&mut game);
            assert_eq!(game.object_view(target).power, 5);
            game.move_object(target, destination);
            settle(&mut game);
            assert_eq!(
                game.observe(Player::P0).last_manifest_dread.is_some(),
                destination == Zone::Graveyard
            );
        }
    }

    #[test]
    fn turn_inside_out_does_not_affect_a_new_target_incarnation() {
        use crate::engine::assurance::big_helpers::{checked, settle};
        let mut game = setup(Player::P0);
        let target = named(&mut game, Player::P0, "Bear Cub", Zone::Battlefield);
        let spell = named(&mut game, Player::P0, "Shock", Zone::Hand);
        game.state.objects[spell as usize].card = Card::from_name("Turn Inside Out").unwrap();
        let item = game.cast_item(Player::P0, spell, [0; 6]);
        game.open_payment_announcement(item, Counters::default(), None);
        checked(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        let pay = game
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut game, pay);
        game.move_object(target, Zone::Graveyard);
        game.move_object(target, Zone::Battlefield);
        settle(&mut game);
        assert_eq!(game.object_view(target).power, 2);
        assert!(
            game.state
                .manifest
                .as_ref()
                .is_none_or(|state| state.deaths.is_empty())
        );
    }

    #[test]
    fn rollback_retains_public_identity_of_revealed_disguise_spell() {
        let mut game = setup(Player::P0);
        let id = named(&mut game, Player::P0, "Bear Cub", Zone::Hand);
        let physical = Card::from_name("Bubble Smuggler").unwrap();
        game.state.objects[id as usize].card = physical;
        game.begin_attack_rollback(Player::P0);
        let item = game.cast_disguised_item(Player::P0, id);
        assert_eq!(game.face_down_spell_views(Player::P1)[0].underlying, None);
        game.move_object(id, Zone::Graveyard);
        assert_eq!(game.state.objects[id as usize].card, physical);
        game.finish_cast_rollback(false);
        assert_eq!(game.state.objects[id as usize].zone, Zone::Hand);
        assert!(game.state.known_hand.contains(&game.object_ref(id)));
        assert!(
            game.observe(Player::P1)
                .objects
                .iter()
                .any(|row| row.id == id && row.card == physical)
        );
        assert_ne!(item.source.unwrap().object, game.object_ref(id));
    }

    #[test]
    fn rollback_retains_revealed_disguise_identity_on_library_top() {
        let mut game = setup(Player::P0);
        let id = named(&mut game, Player::P0, "Bear Cub", Zone::Library);
        let physical = Card::from_name("Bubble Smuggler").unwrap();
        game.state.objects[id as usize].card = physical;
        game.begin_attack_rollback(Player::P0);
        // This checkpoint models an actual library-origin spell proposal.
        game.state.cast_rollback.as_mut().unwrap().spell = id;
        game.cast_disguised_item(Player::P0, id);
        game.move_object(id, Zone::Graveyard);
        game.finish_cast_rollback(false);
        assert_eq!(game.state.objects[id as usize].zone, Zone::Library);
        assert_eq!(
            game.public_library_top(Player::P0),
            Some(game.object_ref(id))
        );
        let top = game.observe(Player::P1).known_library_top[Player::P0.index()].unwrap();
        assert_eq!(top.object, game.object_ref(id));
        assert_eq!(top.card, physical);
    }

    #[test]
    fn rollback_does_not_link_revealed_disguise_through_hidden_library_move() {
        let mut game = setup(Player::P0);
        let id = named(&mut game, Player::P0, "Bear Cub", Zone::Library);
        game.state.objects[id as usize].card = Card::from_name("Bubble Smuggler").unwrap();
        game.begin_attack_rollback(Player::P0);
        // This checkpoint models an actual library-origin spell proposal.
        game.state.cast_rollback.as_mut().unwrap().spell = id;
        game.cast_disguised_item(Player::P0, id);
        game.move_object(id, Zone::Library);
        // A later public move must not reconnect the checkpoint card through
        // the hidden library incarnation.
        game.move_object(id, Zone::Graveyard);
        game.finish_cast_rollback(false);
        assert_eq!(game.state.objects[id as usize].zone, Zone::Library);
        assert_ne!(
            game.public_library_top(Player::P0),
            Some(game.object_ref(id))
        );
    }

    #[test]
    fn foreign_choice_rollback_does_not_reveal_disguised_hand_card() {
        use crate::engine::assurance::big_helpers::{cast, checked, settle};
        for keep in [false, true] {
            let player = Player::P0;
            let mut game = setup(player);
            let enemy = named(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
            let aura = named(
                &mut game,
                player.other(),
                "Witness Protection",
                Zone::Battlefield,
            );
            game.state.objects[aura as usize].attached_to = Some(game.object_ref(enemy));
            let prison = named(&mut game, player, "Banishing Light", Zone::Hand);
            cast(&mut game, prison, None, &[]);
            for _ in 0..40 {
                if game.state.pending != Pending::Priority {
                    break;
                }
                checked(&mut game, Action::Pass);
            }
            checked(
                &mut game,
                Action::ChooseTarget {
                    target: Target::Card(aura),
                },
            );
            settle(&mut game);
            game.apply_temporary_copy(
                game.object_ref(prison),
                CopyRecipe::new(
                    Card::from_name("@token/b60f6cd3b44b22bd289fe4e2")
                        .expect("pinned catalogue identity"),
                ),
                CopyDeadline::EndOfTurn,
            );
            let forest = named(&mut game, player, "Forest", Zone::Battlefield);
            game.change_object_form(
                game.object_ref(forest),
                crate::cards::AnimationSpec {
                    artifact: false,
                    creature_types: crate::cards::AnimationCreatureTypes::Unchanged,
                    base: Some((1, 1)),
                    keywords: Keywords::NONE,
                    duration: crate::cards::AnimationDuration::EndOfTurn,
                },
                characteristic_layers::AnimationTypeMode::AddCreature,
            );
            let spell = named(&mut game, player, "Bear Cub", Zone::Hand);
            game.state.objects[spell as usize].card =
                Card::from_name("Bubble Smuggler").expect("pinned catalogue identity");
            game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 1];
            checked(&mut game, Action::AnnounceDisguise { card: spell });
            assert_eq!(
                game.face_down_spell_views(player.other())[0].underlying,
                None
            );
            let tap = game.legal_actions().into_iter().find(|action|
                matches!(action, Action::ActivateMana { card, production: [0, 0, 0, 0, 1, 0], .. } if *card == prison)).unwrap();
            checked(&mut game, tap);
            assert_eq!(game.actor(), Some(player.other()));
            let forest_ref = game.object_ref(forest);
            checked(
                &mut game,
                Action::ChooseAuraAttachment { object: forest_ref },
            );
            assert!(game.observe(player).mana_rollback.is_some());
            checked(&mut game, Action::ChooseManaRollback { keep });
            assert_eq!(game.state.objects[spell as usize].zone, Zone::Hand);
            assert_eq!(
                game.state.objects[spell as usize].card,
                Card::from_name("Bubble Smuggler").expect("pinned catalogue identity")
            );
            assert!(!game.state.known_hand.contains(&game.object_ref(spell)));
            assert!(
                !game
                    .observe(player.other())
                    .objects
                    .iter()
                    .any(|object| object.id == spell)
            );
            assert!(game.face_down_spell_views(player.other()).is_empty());
        }
    }

    #[test]
    fn face_down_spell_registry_shares_identity_only_with_authorized_viewers() {
        let mut game = setup(Player::P0);
        let id = named(&mut game, Player::P0, "Bear Cub", Zone::Stack);
        let physical = game.state.objects[id as usize].card;
        game.state.objects[id as usize].card =
            Card::from_name("@face-down/disguise").expect("pinned catalogue identity");
        let object = game.object_ref(id);
        game.state
            .manifest
            .get_or_insert_with(Default::default)
            .spells
            .push(FaceDownPermanent {
                object,
                underlying: physical,
                known: false,
                known_to: 1 << Player::P0.index(),
            });
        assert_eq!(
            game.face_down_spell_views(Player::P0)[0].underlying,
            Some(physical)
        );
        assert_eq!(game.face_down_spell_views(Player::P1)[0].underlying, None);
        let snapshot = game.snapshot();
        game.state.manifest.as_mut().unwrap().spells[0].known = true;
        assert_eq!(
            game.face_down_spell_views(Player::P1)[0].underlying,
            Some(physical)
        );
        game.restore(&snapshot).unwrap();
        assert_eq!(game.face_down_spell_views(Player::P1)[0].underlying, None);
    }

    #[test]
    fn game_end_reveals_face_down_permanents_to_every_player() {
        let mut game = setup(Player::P0);
        let id = named(&mut game, Player::P0, "Bear Cub", Zone::Library);
        let original = game.state.objects[id as usize].card;
        let movement = game
            .manifest_entry(game.object_ref(id), Player::P0)
            .unwrap();
        game.stage_entries(vec![movement], None);
        assert!(!game.advance_entries());
        assert_eq!(game.face_down_views(Player::P1)[0].underlying, None);
        game.state.outcome = Some(Outcome::Winner(Player::P1));
        assert_eq!(
            game.face_down_views(Player::P1)[0].underlying,
            Some(original)
        );
    }

    #[test]
    fn previously_revealed_library_top_stays_known_when_manifested() {
        for player in [Player::P0, Player::P1] {
            let mut game = setup(player);
            let id = named(&mut game, player, "Bear Cub", Zone::Library);
            let original = game.state.objects[id as usize].card;
            let reference = game.object_ref(id);
            game.remember_library_top(player, reference);
            let movement = game.manifest_entry(reference, player).unwrap();
            game.stage_entries(vec![movement], None);
            assert!(!game.advance_entries());
            assert_eq!(
                game.face_down_views(player.other())[0].underlying,
                Some(original)
            );
        }
    }

    #[test]
    fn face_up_payment_excludes_mana_restricted_to_artifact_spells() {
        let mut game = setup(Player::P0);
        let id = named(&mut game, Player::P0, "Bear Cub", Zone::Library);
        let movement = game
            .manifest_entry(game.object_ref(id), Player::P0)
            .unwrap();
        game.stage_entries(vec![movement], None);
        assert!(!game.advance_entries());
        let object = game.object_ref(id);
        game.state.players[Player::P0.index()].mana = [0; 6];
        game.add_quality_mana(
            Player::P0,
            [0, 0, 0, 0, 2, 0],
            Some(mana_lots::Quality {
                restriction: ManaRestrictionView::ArtifactSpell,
                spell_bonus: ManaSpellBonusView::None,
                retention: crate::cards::ManaRetention::Step,
                copy_source: None,
                treasure_source: false,
            }),
        );
        assert!(!game.face_up_payment_available(Player::P0, object));
        game.add_mana(Player::P0, [0, 0, 0, 0, 2, 0]);
        let begin = game.legal_actions().into_iter().find(|action|
            matches!(action, Action::BeginFaceUp { object: candidate, .. } if *candidate == object)).unwrap();
        game.apply(Player::P0, begin).unwrap();
        let pay = game
            .legal_actions()
            .into_iter()
            .find(|action| matches!(action, Action::PayFaceUp { .. }))
            .unwrap();
        game.apply(Player::P0, pay).unwrap();
        assert_eq!(game.state.players[Player::P0.index()].mana[4], 2);
        assert_eq!(game.mana_lots(Player::P0)[0].amounts[4], 2);
    }

    #[test]
    fn ability_removal_does_not_remove_the_face_up_special_action() {
        let mut game = setup(Player::P0);
        let id = named(&mut game, Player::P0, "Inspiring Overseer", Zone::Library);
        let movement = game
            .manifest_entry(game.object_ref(id), Player::P0)
            .unwrap();
        game.stage_entries(vec![movement], None);
        assert!(!game.advance_entries());
        let object = game.object_ref(id);
        let aura = named(
            &mut game,
            Player::P1,
            "Witness Protection",
            Zone::Battlefield,
        );
        game.state.objects[aura as usize].attached_to = Some(object);
        let begin = game.legal_actions().into_iter().find(|action|
            matches!(action, Action::BeginFaceUp { object: target, .. } if *target == object)).unwrap();
        game.apply(Player::P0, begin).unwrap();
        let pay = game
            .legal_actions()
            .into_iter()
            .find(|action| matches!(action, Action::PayFaceUp { .. }))
            .unwrap();
        game.apply(Player::P0, pay).unwrap();
        assert!(!game.is_face_down(object));
        assert_eq!(game.object_ref(id), object);
        assert!(!game.keywords(id).contains(Keywords::FLYING));
    }

    #[test]
    fn an_effect_can_turn_a_manifested_land_face_up_without_a_special_action() {
        let mut game = setup(Player::P0);
        let id = named(&mut game, Player::P0, "Forest", Zone::Library);
        let movement = game
            .manifest_entry(game.object_ref(id), Player::P0)
            .unwrap();
        game.stage_entries(vec![movement], None);
        assert!(!game.advance_entries());
        let object = game.object_ref(id);
        assert!(!game.face_up_payment_available(Player::P0, object));
        assert!(game.turn_manifest_face_up(object));
        assert_eq!(game.object_ref(id), object);
        assert_eq!(game.object_view(id).card, Card::Forest);
        assert_eq!(game.observe(Player::P1).last_turned_face_up, Some(object));
    }

    #[test]
    fn effect_turning_an_instant_face_up_reveals_it_but_leaves_it_face_down() {
        let mut game = setup(Player::P0);
        let id = named(&mut game, Player::P0, "Shock", Zone::Library);
        let physical = game.state.objects[id as usize].card;
        let movement = game
            .manifest_entry(game.object_ref(id), Player::P0)
            .unwrap();
        game.stage_entries(vec![movement], None);
        assert!(!game.advance_entries());
        let object = game.object_ref(id);
        assert!(!game.turn_manifest_face_up(object));
        assert!(game.is_face_down(object));
        assert_eq!(
            game.object_view(id).card,
            Card::from_name("@face-down/manifest").expect("pinned catalogue identity")
        );
        assert_eq!(
            game.face_down_views(Player::P1)[0].underlying,
            Some(physical)
        );
        assert!(game.observe(Player::P1).last_turned_face_up.is_none());
    }

    #[test]
    fn manifest_observer_with_removed_abilities_does_not_trigger() {
        let mut game = setup(Player::P0);
        let observer = named(&mut game, Player::P0, "Bear Cub", Zone::Battlefield);
        game.remove_face_providers(observer);
        game.state.objects[observer as usize].card =
            Card::from_name("@test/manifest-observer").expect("pinned catalogue identity");
        game.register_battlefield_providers(observer);
        let aura = named(
            &mut game,
            Player::P1,
            "Witness Protection",
            Zone::Battlefield,
        );
        game.state.objects[aura as usize].attached_to = Some(game.object_ref(observer));
        game.state.players[Player::P0.index()].library.clear();
        assert!(!game.start_manifest_dread(Player::P0));
        assert!(game.state.pending_triggers.is_empty());
        assert!(game.observe(Player::P0).last_manifest_dread.is_some());
    }

    #[test]
    fn actual_observers_receive_completed_manifest_and_turn_face_up_events() {
        for player in [Player::P0, Player::P1] {
            let mut game = setup(player);
            let observer = named(&mut game, player, "Bear Cub", Zone::Battlefield);
            game.remove_face_providers(observer);
            game.state.objects[observer as usize].card =
                Card::from_name("@test/manifest-observer").expect("pinned catalogue identity");
            game.register_battlefield_providers(observer);
            let opponent = named(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
            game.remove_face_providers(opponent);
            game.state.objects[opponent as usize].card =
                Card::from_name("@test/manifest-observer").expect("pinned catalogue identity");
            game.register_battlefield_providers(opponent);

            game.state.players[player.index()].library.clear();
            assert!(!game.start_manifest_dread(player));
            assert_eq!(game.state.pending_triggers.len(), 1);
            let trigger = game.state.pending_triggers.pop().unwrap();
            assert_eq!(trigger.source.unwrap().object, game.object_ref(observer));
            assert_eq!(trigger.kind, StackKind::TriggeredAbility { ability: 0 });
            let event = trigger.manifest_event.unwrap();
            assert_eq!(event.player, player);
            assert!(event.manifested.is_none());
            assert!(event.graveyard_cards.is_empty());

            let remainder = named(&mut game, player, "Island", Zone::Library);
            let concealed = named(&mut game, player, "Bear Cub", Zone::Library);
            game.state.objects[concealed as usize].card =
                Card::from_name("@test/manifest-observer").expect("pinned catalogue identity");
            assert!(game.start_manifest_dread(player));
            game.choose_manifest(game.object_ref(concealed));
            assert!(!game.advance_entries());
            assert_eq!(game.state.pending_triggers.len(), 1);
            let trigger = game.state.pending_triggers.pop().unwrap();
            let event = trigger.manifest_event.unwrap();
            let permanent = game.object_ref(concealed);
            assert_eq!(event.manifested, Some(permanent));
            assert_eq!(event.graveyard_cards, vec![game.object_ref(remainder)]);

            assert!(game.turn_manifest_face_up(permanent));
            let triggers = &game.state.pending_triggers;
            assert_eq!(triggers.len(), 3);
            assert!(
                triggers
                    .iter()
                    .any(|trigger| trigger.source.unwrap().object == permanent
                        && trigger.kind == StackKind::TriggeredAbility { ability: 1 })
            );
            assert!(
                triggers
                    .iter()
                    .any(|trigger| trigger.source.unwrap().object == permanent
                        && trigger.kind == StackKind::TriggeredAbility { ability: 2 })
            );
            assert!(triggers.iter().any(|trigger| trigger.source.unwrap().object
                == game.object_ref(observer)
                && trigger.kind == StackKind::TriggeredAbility { ability: 2 }));
            assert!(triggers.iter().all(|trigger| {
                trigger
                    .event_object
                    .as_ref()
                    .is_some_and(|event| event.object == permanent)
            }));
            assert!(
                !triggers
                    .iter()
                    .any(|trigger| trigger.source.unwrap().object == game.object_ref(opponent))
            );
        }
    }

    #[test]
    fn foreign_aura_choice_can_abort_face_up_payment_and_replay_mana() {
        use crate::engine::assurance::big_helpers::{cast, checked, settle};
        for player in [Player::P0, Player::P1] {
            for keep in [false, true] {
                let mut game = setup(player);
                let enemy = named(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
                let aura = named(
                    &mut game,
                    player.other(),
                    "Witness Protection",
                    Zone::Battlefield,
                );
                game.state.objects[aura as usize].attached_to = Some(game.object_ref(enemy));
                let prison = named(&mut game, player, "Banishing Light", Zone::Hand);
                cast(&mut game, prison, None, &[]);
                for _ in 0..40 {
                    if game.state.pending != Pending::Priority {
                        break;
                    }
                    checked(&mut game, Action::Pass);
                }
                checked(
                    &mut game,
                    Action::ChooseTarget {
                        target: Target::Card(aura),
                    },
                );
                settle(&mut game);
                assert_eq!(game.object_view(aura).zone, Zone::Exile);

                // The linked permanent becomes a Treasure without becoming a token;
                // sacrificing it for mana returns the opposing player's Aura.
                game.apply_temporary_copy(
                    game.object_ref(prison),
                    CopyRecipe::new(
                        Card::from_name("@token/b60f6cd3b44b22bd289fe4e2")
                            .expect("pinned catalogue identity"),
                    ),
                    CopyDeadline::EndOfTurn,
                );
                let forest = named(&mut game, player, "Forest", Zone::Battlefield);
                game.change_object_form(
                    game.object_ref(forest),
                    crate::cards::AnimationSpec {
                        artifact: false,
                        creature_types: crate::cards::AnimationCreatureTypes::Unchanged,
                        base: Some((1, 1)),
                        keywords: Keywords::NONE,
                        duration: crate::cards::AnimationDuration::EndOfTurn,
                    },
                    characteristic_layers::AnimationTypeMode::AddCreature,
                );
                let creature = named(&mut game, player, "Bear Cub", Zone::Library);
                let physical = game.state.objects[creature as usize].card;
                let movement = game
                    .manifest_entry(game.object_ref(creature), player)
                    .unwrap();
                game.stage_entries(vec![movement], None);
                assert!(!game.advance_entries());
                let face_down = game.object_ref(creature);
                game.state.players[player.index()].mana = [0; 6];
                let begin = game.legal_actions().into_iter().find(|action|
                    matches!(action, Action::BeginFaceUp { object, .. } if *object == face_down)).unwrap();
                checked(&mut game, begin);
                let tap = game.legal_actions().into_iter().find(|action|
                    matches!(action, Action::ActivateMana { card, production: [0, 0, 0, 0, 1, 0], .. } if *card == prison)).unwrap();
                checked(&mut game, tap);
                assert_eq!(game.actor(), Some(player.other()));
                let forest_ref = game.object_ref(forest);
                checked(
                    &mut game,
                    Action::ChooseAuraAttachment { object: forest_ref },
                );
                assert_eq!(game.actor(), Some(player));
                assert!(game.observe(player).mana_rollback.is_some());
                checked(&mut game, Action::ChooseManaRollback { keep });
                assert!(game.face_up_payment().is_none());
                assert!(game.is_face_down(face_down));
                assert_eq!(
                    game.face_down_views(player.other())[0].underlying,
                    Some(physical)
                );
                assert_eq!(
                    game.object_view(prison).zone,
                    if keep {
                        Zone::Graveyard
                    } else {
                        Zone::Battlefield
                    }
                );
            }
        }
    }

    #[test]
    fn failed_face_up_payment_restores_permanent_but_keeps_public_reveal() {
        let mut game = setup(Player::P0);
        let id = named(&mut game, Player::P0, "Bear Cub", Zone::Library);
        let original = game.state.objects[id as usize].card;
        let movement = game
            .manifest_entry(game.object_ref(id), Player::P0)
            .unwrap();
        game.stage_entries(vec![movement], None);
        assert!(!game.advance_entries());
        let object = game.object_ref(id);
        let begin = game.legal_actions().into_iter().find(|action|
            matches!(action, Action::BeginFaceUp { object: candidate, .. } if *candidate == object)).unwrap();
        game.apply(Player::P0, begin).unwrap();
        game.apply(Player::P0, Action::AbortFaceUpPayment).unwrap();
        assert_eq!(game.object_ref(id), object);
        assert_eq!(
            game.object_view(id).card,
            Card::from_name("@face-down/manifest").expect("pinned catalogue identity")
        );
        assert!(game.face_up_payment().is_none());
        assert_eq!(
            game.observe(Player::P1).face_down_permanents[0].underlying,
            Some(original)
        );
    }

    #[test]
    fn rollback_forgets_face_down_identity_if_hidden_library_movement_obscures_it() {
        let mut game = setup(Player::P0);
        let id = named(&mut game, Player::P0, "Bear Cub", Zone::Library);
        let movement = game
            .manifest_entry(game.object_ref(id), Player::P0)
            .unwrap();
        game.stage_entries(vec![movement], None);
        assert!(!game.advance_entries());
        let object = game.object_ref(id);
        game.state.manifest.as_mut().unwrap().permanents[0].known = true;
        let begin = game.legal_actions().into_iter().find(|action|
            matches!(action, Action::BeginFaceUp { object: candidate, .. } if *candidate == object)).unwrap();
        game.apply(Player::P0, begin).unwrap();
        game.obscure_rollback_face_down(id);
        game.finish_cast_rollback(false);
        assert_eq!(
            game.observe(Player::P1).face_down_permanents[0].underlying,
            None
        );
    }

    #[test]
    fn delayed_manifest_follows_exact_target_death_and_expires() {
        let mut game = setup(Player::P0);
        let source = named(&mut game, Player::P0, "Lotus Ring", Zone::Battlefield);
        let victim = named(&mut game, Player::P1, "Bear Cub", Zone::Battlefield);
        let mut item = game.activation_item(Player::P0, source, 0);
        let original = game.object_ref(victim);
        item.targets.push(BoundTarget {
            group: 0,
            target: Target::Card(victim),
            object: Some(original),
        });
        game.register_manifest_death(&item);
        let checkpoint = game.snapshot();
        let mut branch = game.clone();
        branch.move_object(victim, Zone::Graveyard);
        assert!(
            branch
                .state
                .pending_triggers
                .iter()
                .any(|item| item.kind == StackKind::DelayedManifestDread
                    && item.controller == Player::P0)
        );
        assert!(branch.state.manifest.as_ref().unwrap().deaths.is_empty());
        branch.restore(&checkpoint).unwrap();
        branch.move_object(victim, Zone::Exile);
        assert!(
            !branch
                .state
                .pending_triggers
                .iter()
                .any(|item| item.kind == StackKind::DelayedManifestDread)
        );
        branch.expire_manifest_deaths();
        assert!(branch.state.manifest.as_ref().unwrap().deaths.is_empty());
    }

    #[test]
    fn turning_face_up_does_not_trigger_an_enter_the_battlefield_ability() {
        let mut game = setup(Player::P0);
        let id = named(&mut game, Player::P0, "Inspiring Overseer", Zone::Library);
        let movement = game
            .manifest_entry(game.object_ref(id), Player::P0)
            .unwrap();
        game.stage_entries(vec![movement], None);
        assert!(!game.advance_entries());
        let reference = game.object_ref(id);
        let pending_before = game.state.pending_triggers.len();
        assert!(game.turn_manifest_face_up(reference));
        assert_eq!(game.object_ref(id), reference);
        assert_eq!(game.state.pending_triggers.len(), pending_before);
        assert_eq!(
            game.observe(Player::P1).last_turned_face_up,
            Some(reference)
        );
    }

    #[test]
    fn controller_can_pay_to_turn_a_manifested_creature_face_up() {
        let mut game = setup(Player::P0);
        let id = named(&mut game, Player::P0, "Bear Cub", Zone::Library);
        let physical = game.state.objects[id as usize].card;
        let movement = game
            .manifest_entry(game.object_ref(id), Player::P0)
            .unwrap();
        game.stage_entries(vec![movement], None);
        assert!(!game.advance_entries());
        let object = game.object_ref(id);
        let begin = game.legal_actions().into_iter().find(|action|
            matches!(action, Action::BeginFaceUp { object: candidate, .. } if *candidate == object)).unwrap();
        let checkpoint = game.snapshot();
        game.apply(Player::P0, begin).unwrap();
        assert!(game.observe(Player::P1).face_up_payment.is_none());
        assert_eq!(
            game.observe(Player::P1).face_down_permanents[0].underlying,
            Some(physical)
        );
        let pay = game
            .legal_actions()
            .into_iter()
            .find(|action| matches!(action, Action::PayFaceUp { .. }))
            .unwrap();
        game.apply(Player::P0, pay).unwrap();
        assert_eq!(game.object_ref(id), object);
        assert_eq!(game.object_view(id).card, physical);
        assert!(game.observe(Player::P0).face_down_permanents.is_empty());
        game.restore(&checkpoint).unwrap();
        assert_eq!(game.snapshot(), checkpoint);
    }

    #[test]
    fn face_down_copy_is_masked_until_turned_up_without_a_zone_change() {
        let mut game = setup(Player::P0);
        let id = named(&mut game, Player::P0, "Bear Cub", Zone::Library);
        let physical = game.state.objects[id as usize].card;
        let movement = game
            .manifest_entry(game.object_ref(id), Player::P0)
            .unwrap();
        game.stage_entries(vec![movement], None);
        assert!(!game.advance_entries());
        let object = game.object_ref(id);
        game.apply_temporary_copy(
            object,
            CopyRecipe::new(Card::Forest),
            CopyDeadline::EndOfTurn,
        );
        assert_eq!(
            game.object_view(id).card,
            Card::from_name("@face-down/manifest").expect("pinned catalogue identity")
        );
        assert_eq!(
            game.copiable_recipe(object).definition,
            Card::from_name("@face-down/manifest").expect("pinned catalogue identity")
        );
        assert!(game.turn_manifest_face_up(object));
        assert_eq!(game.object_ref(id), object);
        assert_eq!(game.object_view(id).card, Card::Forest);
        assert!(
            game.observe(Player::P1)
                .zone_reveals
                .iter()
                .any(|reveal| reveal.card == physical)
        );
        game.expire_temporary_copies(true);
        assert_eq!(game.object_view(id).card, physical);
    }

    #[test]
    fn copy_of_face_down_permanent_remains_blank_after_original_turns_up() {
        let mut game = setup(Player::P0);
        let id = named(&mut game, Player::P0, "Bear Cub", Zone::Library);
        let movement = game
            .manifest_entry(game.object_ref(id), Player::P0)
            .unwrap();
        game.stage_entries(vec![movement], None);
        assert!(!game.advance_entries());
        let face_down = game.object_ref(id);
        let recipient_id = named(&mut game, Player::P1, "Bear Cub", Zone::Battlefield);
        let recipient = game.object_ref(recipient_id);
        game.apply_temporary_copy(
            recipient,
            game.copiable_recipe(face_down),
            CopyDeadline::EndOfTurn,
        );
        assert_eq!(
            game.object_view(recipient_id).card,
            Card::from_name("@face-down/manifest").expect("pinned catalogue identity")
        );
        assert!(game.turn_manifest_face_up(face_down));
        assert_eq!(
            game.object_view(recipient_id).card,
            Card::from_name("@face-down/manifest").expect("pinned catalogue identity")
        );
        assert!(!game.is_face_down(recipient));
        assert!(game.observe(Player::P1).face_down_permanents.is_empty());
    }

    #[test]
    fn copy_expiring_face_down_does_not_reappear_after_turning_up() {
        let mut game = setup(Player::P0);
        let id = named(&mut game, Player::P0, "Bear Cub", Zone::Library);
        let physical = game.state.objects[id as usize].card;
        let movement = game
            .manifest_entry(game.object_ref(id), Player::P0)
            .unwrap();
        game.stage_entries(vec![movement], None);
        assert!(!game.advance_entries());
        let object = game.object_ref(id);
        game.apply_temporary_copy(
            object,
            CopyRecipe::new(Card::Forest),
            CopyDeadline::EndOfTurn,
        );
        game.expire_temporary_copies(true);
        assert_eq!(
            game.object_view(id).card,
            Card::from_name("@face-down/manifest").expect("pinned catalogue identity")
        );
        assert!(game.turn_manifest_face_up(object));
        assert_eq!(game.object_view(id).card, physical);
    }

    #[test]
    fn unrelated_nested_entry_cannot_finish_pending_manifest_dread() {
        let mut game = setup(Player::P0);
        game.state.players[Player::P0.index()].library.clear();
        let remainder = named(&mut game, Player::P0, "Island", Zone::Library);
        let selected = named(&mut game, Player::P0, "Bear Cub", Zone::Library);
        assert!(game.start_manifest_dread(Player::P0));
        game.choose_manifest(game.object_ref(selected));
        // Simulate an owned nested return before the selected outer entry
        // commits, as can happen inside an entry replacement continuation.
        let outer = game.state.entry_batch.take().unwrap();
        game.state.pending = Pending::Priority;
        let other = named(&mut game, Player::P1, "Bear Cub", Zone::Graveyard);
        let movement = game.entry_move(other, Zone::Battlefield, false);
        game.stage_entries(vec![movement], None);
        assert!(!game.advance_entries());
        assert_eq!(game.state.objects[remainder as usize].zone, Zone::Library);
        assert!(game.state.manifest.as_ref().unwrap().choice.is_some());
        assert!(game.observe(Player::P0).last_manifest_dread.is_none());
        game.state.entry_batch = Some(outer);
        assert!(!game.advance_entries());
        assert_eq!(game.state.objects[remainder as usize].zone, Zone::Graveyard);
        assert!(game.is_face_down(game.object_ref(selected)));
        let event = game.observe(Player::P0).last_manifest_dread.unwrap();
        assert_eq!(event.manifested, Some(game.object_ref(selected)));
    }

    #[test]
    fn prevented_manifest_puts_both_looked_at_cards_through_graveyard_replacement() {
        for replace in [false, true] {
            let mut game = setup(Player::P0);
            game.state.players[Player::P0.index()].library.clear();
            let other = named(&mut game, Player::P0, "Island", Zone::Library);
            let selected = named(&mut game, Player::P0, "Bear Cub", Zone::Library);
            if replace {
                named(&mut game, Player::P1, "Rest in Peace", Zone::Battlefield);
            }
            assert!(game.start_manifest_dread(Player::P0));
            game.choose_manifest(game.object_ref(selected));
            // This models an entry prevention after selecting the card.
            game.state.entry_batch.as_mut().unwrap().moves[0].skipped = true;
            assert!(!game.advance_entries());
            let destination = if replace {
                Zone::Exile
            } else {
                Zone::Graveyard
            };
            assert_eq!(game.state.objects[selected as usize].zone, destination);
            assert_eq!(game.state.objects[other as usize].zone, destination);
            let event = game.observe(Player::P0).last_manifest_dread.unwrap();
            assert_eq!(event.manifested, None);
            assert_eq!(event.graveyard_cards.len(), if replace { 0 } else { 2 });
        }
    }

    #[test]
    fn manifest_replays_competing_graveyard_replacements_without_partial_entry() {
        for exile in [false, true] {
            let mut game = setup(Player::P0);
            game.state.players[Player::P0.index()].library.clear();
            let other = named(&mut game, Player::P0, "Darksteel Colossus", Zone::Library);
            let selected = named(&mut game, Player::P0, "Bear Cub", Zone::Library);
            named(&mut game, Player::P1, "Rest in Peace", Zone::Battlefield);
            assert!(game.start_manifest_dread(Player::P0));
            let selected_ref = game.object_ref(selected);
            game.apply(
                Player::P0,
                Action::ChooseManifest {
                    object: selected_ref,
                },
            )
            .unwrap();
            assert!(matches!(
                game.state.pending,
                Pending::ZoneReplacement { player: Player::P0 }
            ));
            // The chosen card enters before the remaining card is moved; the
            // latter's replacement choice suspends that subsequent move.
            assert!(game.is_face_down(game.object_ref(selected)));
            assert_eq!(game.state.objects[other as usize].zone, Zone::Library);
            let checkpoint = game.snapshot();
            let effect = if exile {
                ZoneReplacementKind::Exile
            } else {
                ZoneReplacementKind::RevealShuffle
            };
            game.apply(Player::P0, Action::ChooseZoneReplacement { effect })
                .unwrap();
            assert!(game.is_face_down(game.object_ref(selected)));
            assert_eq!(
                game.state.objects[other as usize].zone,
                if exile { Zone::Exile } else { Zone::Library }
            );
            assert_eq!(
                game.observe(Player::P0)
                    .last_manifest_dread
                    .unwrap()
                    .manifested,
                Some(game.object_ref(selected))
            );
            game.restore(&checkpoint).unwrap();
            assert!(matches!(
                game.state.pending,
                Pending::ZoneReplacement { .. }
            ));
        }
    }

    #[test]
    fn delayed_manifest_uses_spell_controller_after_target_changes_control() {
        let mut game = setup(Player::P0);
        let source = named(&mut game, Player::P0, "Lotus Ring", Zone::Battlefield);
        let target = named(&mut game, Player::P0, "Bear Cub", Zone::Battlefield);
        let object = game.object_ref(target);
        let mut item = game.activation_item(Player::P0, source, 0);
        item.targets.push(BoundTarget {
            group: 0,
            target: Target::Card(target),
            object: Some(object),
        });
        game.register_manifest_death(&item);
        game.change_controller(object, Player::P1);
        game.move_object(target, Zone::Graveyard);
        assert!(
            game.state
                .pending_triggers
                .iter()
                .any(|item| item.kind == StackKind::DelayedManifestDread
                    && item.controller == Player::P0)
        );
    }

    #[test]
    fn manifest_dread_zero_and_one_card_need_no_choice() {
        let mut game = setup(Player::P0);
        game.state.players[Player::P0.index()].library.clear();
        assert!(!game.start_manifest_dread(Player::P0));
        let empty = game.observe(Player::P1).last_manifest_dread.unwrap();
        assert_eq!(empty.player, Player::P0);
        assert!(empty.manifested.is_none());
        assert!(empty.graveyard_cards.is_empty());
        let id = named(&mut game, Player::P0, "Island", Zone::Library);
        assert!(game.start_manifest_dread(Player::P0));
        assert!(game.state.entry_batch.is_some());
        assert!(!game.advance_entries());
        assert_eq!(game.state.objects[id as usize].zone, Zone::Battlefield);
        assert!(game.is_face_down(game.object_ref(id)));
    }

    #[test]
    fn manifest_waits_for_entry_replacement_before_putting_other_card_in_graveyard() {
        let mut game = setup(Player::P0);
        let other = named(&mut game, Player::P0, "Island", Zone::Library);
        let chosen = named(&mut game, Player::P0, "Bear Cub", Zone::Library);
        assert!(game.start_manifest_dread(Player::P0));
        game.choose_manifest(game.object_ref(chosen));
        let source = game.object_ref(chosen);
        game.state.entry_batch.as_mut().unwrap().tap_events = vec![entry_tap::EntryTapEvent {
            index: 0,
            effects: vec![
                entry_tap::EntryTapEffect {
                    source,
                    intrinsic: false,
                    kind: entry_tap::EntryTapKind::Tapped,
                },
                entry_tap::EntryTapEffect {
                    source,
                    intrinsic: true,
                    kind: entry_tap::EntryTapKind::Untapped,
                },
            ],
            payment: None,
            behold: None,
        }];
        assert!(game.advance_entries());
        assert_eq!(game.state.objects[other as usize].zone, Zone::Library);
        assert!(game.observe(Player::P1).last_manifest_dread.is_none());
        let checkpoint = game.snapshot();
        game.choose_entry_tap_effect(source, false);
        assert!(!game.advance_entries());
        assert_eq!(game.state.objects[other as usize].zone, Zone::Graveyard);
        assert!(!game.state.objects[chosen as usize].tapped);
        assert_eq!(
            game.observe(Player::P1)
                .last_manifest_dread
                .unwrap()
                .graveyard_cards,
            vec![game.object_ref(other)]
        );
        game.restore(&checkpoint).unwrap();
        assert_eq!(game.snapshot(), checkpoint);
    }

    #[test]
    fn manifest_dread_selects_privately_and_moves_other_card_after_entry() {
        for player in [Player::P0, Player::P1] {
            let mut game = setup(player);
            let other = named(&mut game, player, "Island", Zone::Library);
            let chosen = named(&mut game, player, "Bear Cub", Zone::Library);
            assert!(game.start_manifest_dread(player));
            assert_eq!(
                game.manifest_choice_view(player).unwrap().candidates.len(),
                2
            );
            assert!(game.manifest_choice_view(player.other()).is_none());
            let checkpoint = game.snapshot();
            let mut branch = game.clone();
            branch.choose_manifest(branch.object_ref(chosen));
            assert_eq!(branch.state.objects[other as usize].zone, Zone::Library);
            assert!(!branch.advance_entries());
            assert_eq!(
                branch.state.objects[chosen as usize].zone,
                Zone::Battlefield
            );
            assert_eq!(branch.state.objects[other as usize].zone, Zone::Graveyard);
            let event = branch.observe(player.other()).last_manifest_dread.unwrap();
            assert_eq!(event.manifested, Some(branch.object_ref(chosen)));
            assert_eq!(event.graveyard_cards, vec![branch.object_ref(other)]);
            assert!(branch.manifest_choice_view(player).is_none());
            branch.restore(&checkpoint).unwrap();
            assert_eq!(branch.snapshot(), checkpoint);
        }
    }

    #[test]
    fn manifest_entry_uses_blank_copiable_values_and_private_physical_card() {
        for p in [Player::P0, Player::P1] {
            let mut g = setup(p);
            let id = named(&mut g, p, "Darksteel Colossus", Zone::Library);
            let original = g.state.objects[id as usize].card;
            let before = g.snapshot();
            let movement = g.manifest_entry(g.object_ref(id), p).unwrap();
            assert_eq!(g.state.objects[id as usize].card, original);
            assert_eq!(
                movement.before.card,
                Card::from_name("@face-down/manifest").expect("pinned catalogue identity")
            );
            assert_eq!(movement.before.name, "");
            assert_eq!((movement.before.power, movement.before.toughness), (2, 2));
            assert_eq!(movement.before.kinds, 1);
            assert_eq!(movement.before.colors, 0);
            assert_eq!(movement.before.keywords, Keywords::NONE);
            g.stage_entries(vec![movement], None);
            assert!(!g.advance_entries());
            assert_eq!(g.object_view(id).zone, Zone::Battlefield);
            assert_eq!(
                g.object_view(id).card,
                Card::from_name("@face-down/manifest").expect("pinned catalogue identity")
            );
            assert_eq!(
                g.stats(id),
                Stats {
                    power: 2,
                    toughness: 2
                }
            );
            assert_eq!(g.keywords(id), Keywords::NONE);
            assert_eq!(
                g.observe(p).face_down_permanents[0].underlying,
                Some(original)
            );
            assert_eq!(
                g.observe(p.other()).face_down_permanents[0].underlying,
                None
            );
            assert_eq!(
                g.observe(p.other())
                    .objects
                    .iter()
                    .find(|o| o.id == id)
                    .unwrap()
                    .card,
                Card::from_name("@face-down/manifest").expect("pinned catalogue identity")
            );
            let mut hand = g.clone();
            hand.move_object(id, Zone::Hand);
            assert_eq!(hand.object_view(id).card, original);
            assert!(
                hand.observe(p.other())
                    .objects
                    .iter()
                    .any(|o| o.id == id && o.card == original)
            );
            assert!(hand.observe(p.other()).face_down_permanents.is_empty());
            let mut graveyard = g.clone();
            graveyard.move_object(id, Zone::Graveyard);
            assert_eq!(graveyard.object_view(id).card, original);
            assert!(graveyard.observe(p.other()).face_down_permanents.is_empty());
            let mut library = g.clone();
            library.move_object(id, Zone::Library);
            assert!(
                library
                    .observe(p.other())
                    .zone_reveals
                    .iter()
                    .any(|reveal| { reveal.card == original && reveal.from == Zone::Battlefield })
            );
            let mut branch = g.clone();
            branch.restore(&before).unwrap();
            assert_eq!(branch.snapshot(), before);
        }
    }

    #[test]
    fn manifest_ignores_underlying_enter_tapped_ability() {
        let mut g = setup(Player::P0);
        let id = named(&mut g, Player::P0, "Temple of Enlightenment", Zone::Library);
        let movement = g.manifest_entry(g.object_ref(id), Player::P0).unwrap();
        g.stage_entries(vec![movement], None);
        assert!(!g.advance_entries());
        assert_eq!(g.object_view(id).zone, Zone::Battlefield);
        assert!(!g.object_view(id).tapped);
    }

    #[test]
    fn face_up_payment_checks_controller_physical_creature_and_exact_incarnation() {
        for p in [Player::P0, Player::P1] {
            let mut g = setup(p);
            let id = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let object = g.object_ref(id);
            let creature = g.state.objects[id as usize].card;
            g.state.manifest = Some(Box::new(ManifestState {
                permanents: vec![FaceDownPermanent {
                    object,
                    underlying: creature,
                    known: false,
                    known_to: 1 << p.index(),
                }],
                choice: None,
                payment: None,
                deaths: Vec::new(),
                last_event: None,
                last_face_up: None,
                spells: Vec::new(),
                face_up_completion: None,
            }));
            g.state.players[p.index()].mana = [0; 6];
            assert!(!g.face_up_payment_available(p, object));
            g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 1];
            assert!(g.face_up_payment_available(p, object));
            assert!(!g.face_up_payment_available(p.other(), object));
            let stale = ObjectRef {
                generation: object.generation + 1,
                ..object
            };
            assert!(!g.face_up_payment_available(p, stale));
            g.state.manifest.as_mut().unwrap().permanents[0].underlying = Card::Forest;
            assert!(!g.face_up_payment_available(p, object));
        }
    }

    #[test]
    fn manifest_look_is_only_visible_to_the_choosing_player() {
        for player in [Player::P0, Player::P1] {
            let mut game = setup(player);
            let a = named(&mut game, player, "Bear Cub", Zone::Library);
            let b = named(&mut game, player, "Island", Zone::Library);
            let candidates = vec![game.object_ref(a), game.object_ref(b)];
            game.state.manifest = Some(Box::new(ManifestState {
                permanents: Vec::new(),
                choice: Some(ManifestChoice {
                    player,
                    candidates,
                    selected: None,
                }),
                payment: None,
                deaths: Vec::new(),
                last_event: None,
                last_face_up: None,
                spells: Vec::new(),
                face_up_completion: None,
            }));
            let view = game.manifest_choice_view(player).unwrap();
            assert_eq!(view.candidates.len(), 2);
            assert!(game.manifest_choice_view(player.other()).is_none());
            assert!(game.observe(player.other()).manifest_choice.is_none());
            let snapshot = game.snapshot();
            let mut branch = game.clone();
            branch.move_object(a, Zone::Hand);
            assert_eq!(
                branch
                    .manifest_choice_view(player)
                    .unwrap()
                    .candidates
                    .len(),
                1
            );
            branch.restore(&snapshot).unwrap();
            assert_eq!(branch.snapshot(), snapshot);
        }
    }

    #[test]
    fn control_changes_preserve_private_face_down_knowledge_and_restore() {
        let mut game = setup(Player::P0);
        let id = named(&mut game, Player::P0, "Bear Cub", Zone::Library);
        let physical = game.state.objects[id as usize].card;
        let movement = game
            .manifest_entry(game.object_ref(id), Player::P0)
            .unwrap();
        game.stage_entries(vec![movement], None);
        assert!(!game.advance_entries());
        let object = game.object_ref(id);
        let before = game.snapshot();
        let mut branch = game.clone();
        branch.change_controller(object, Player::P1);
        assert_eq!(
            branch.face_down_views(Player::P0)[0].underlying,
            Some(physical)
        );
        assert_eq!(
            branch.face_down_views(Player::P1)[0].underlying,
            Some(physical)
        );
        branch.change_controller(object, Player::P0);
        assert_eq!(
            branch.face_down_views(Player::P1)[0].underlying,
            Some(physical)
        );
        branch.restore(&before).unwrap();
        assert_eq!(branch.snapshot(), before);
        assert_eq!(branch.face_down_views(Player::P1)[0].underlying, None);
    }

    #[test]
    fn rollback_retains_private_identity_learned_from_temporary_control() {
        let mut game = setup(Player::P0);
        let id = named(&mut game, Player::P0, "Bear Cub", Zone::Library);
        let physical = game.state.objects[id as usize].card;
        let movement = game
            .manifest_entry(game.object_ref(id), Player::P0)
            .unwrap();
        game.stage_entries(vec![movement], None);
        assert!(!game.advance_entries());
        let object = game.object_ref(id);
        game.begin_attack_rollback(Player::P0);
        game.change_controller(object, Player::P1);
        assert_eq!(
            game.face_down_views(Player::P1)[0].underlying,
            Some(physical)
        );
        game.finish_cast_rollback(false);
        assert_eq!(game.state.objects[id as usize].controller, Player::P0);
        assert_eq!(
            game.face_down_views(Player::P1)[0].underlying,
            Some(physical)
        );
    }

    #[test]
    fn player_control_retains_private_identity_after_control_ends() {
        let mut game = setup(Player::P0);
        let id = named(&mut game, Player::P0, "Bear Cub", Zone::Library);
        let physical = game.state.objects[id as usize].card;
        let movement = game
            .manifest_entry(game.object_ref(id), Player::P0)
            .unwrap();
        game.stage_entries(vec![movement], None);
        assert!(!game.advance_entries());
        let before = game.snapshot();
        game.schedule_player_control(Player::P0, Player::P1);
        game.begin_player_control_turn();
        assert_eq!(
            game.face_down_views(Player::P1)[0].underlying,
            Some(physical)
        );
        game.state.active = Player::P1;
        game.begin_player_control_turn();
        assert_eq!(
            game.face_down_views(Player::P1)[0].underlying,
            Some(physical)
        );
        game.restore(&before).unwrap();
        assert_eq!(game.face_down_views(Player::P1)[0].underlying, None);
    }

    #[test]
    fn face_down_view_gives_only_current_controller_the_underlying_card() {
        for owner in [Player::P0, Player::P1] {
            let mut game = setup(owner);
            // This tests the bookkeeping view independently of the unfinished
            // face-down battlefield entry and characteristic implementation.
            let id = named(&mut game, owner, "Bear Cub", Zone::Battlefield);
            let object = game.object_ref(id);
            let underlying = game.state.objects[id as usize].card;
            game.state.manifest = Some(Box::new(ManifestState {
                permanents: vec![FaceDownPermanent {
                    object,
                    underlying,
                    known: false,
                    known_to: 1 << owner.index(),
                }],
                choice: None,
                payment: None,
                deaths: Vec::new(),
                last_event: None,
                last_face_up: None,
                spells: Vec::new(),
                face_up_completion: None,
            }));
            let before = game.snapshot();
            assert_eq!(game.face_down_views(owner)[0].underlying, Some(underlying));
            assert_eq!(game.face_down_views(owner.other())[0].underlying, None);
            let mut branch = game.clone();
            branch.change_controller(object, owner.other());
            assert_eq!(
                branch.face_down_views(owner)[0].underlying,
                Some(underlying)
            );
            assert_eq!(
                branch.face_down_views(owner.other())[0].underlying,
                Some(underlying)
            );
            branch.restore(&before).unwrap();
            assert_eq!(branch.snapshot(), before);
        }
    }
}
