//! Rules names are shared strings; catalogue identities remain unambiguous.
use super::ability_timeline_tests::{cast, checked, named, settle, setup};
use super::tests::spawn;
use super::*;
use crate::SearchFilter;

#[test]
fn paid_crypt_feaster_retains_its_string_name_on_the_battlefield_in_both_seats() {
    for player in [Player::P0, Player::P1] {
        let mut game = setup(player);
        let creature = named(&mut game, player, "Crypt Feaster", Zone::Hand);
        assert!(
            !game
                .observe(player.other())
                .objects
                .iter()
                .any(|object| object.id == creature)
        );
        assert_eq!(
            game.observe(player)
                .objects
                .iter()
                .find(|object| object.id == creature)
                .unwrap()
                .name(),
            "Crypt Feaster"
        );
        game.state.players[player.index()].mana = [0, 0, 1, 0, 0, 3];
        cast(&mut game, creature, None, &[]);
        settle(&mut game);
        let object = game.object_view(creature);
        assert_eq!(object.zone, Zone::Battlefield);
        let name: &str = object.name();
        assert_eq!(name, "Crypt Feaster");
        assert_eq!(object.card.name(), name);
        for viewer in [player, player.other()] {
            let observation = game.observe(viewer);
            assert_eq!(observation.schema, OBSERVATION_SCHEMA);
            let visible = observation
                .objects
                .iter()
                .find(|object| object.id == creature)
                .unwrap();
            assert_eq!(
                serde_json::to_value(visible).unwrap()["name"],
                "Crypt Feaster"
            );
        }
        assert_eq!(game.state.players[player.index()].mana, [0; 6]);
        let saved = game.snapshot();
        let mut owned = {
            let parent = game.clone();
            parent.clone()
        };
        drop(game);
        assert_eq!(owned.object_view(creature).name(), name);
        owned.restore(&saved).unwrap();
        assert_eq!(owned.object_view(creature).name(), name);
    }
}

#[test]
fn every_token_recipe_exposes_rules_name_without_losing_lookup_or_serde_identity() {
    for token in Card::catalogue().filter(|card| card.is_token()) {
        let info = token.token_info().unwrap();
        assert_eq!(token.name(), info.name);
        assert_eq!(token.identity(), info.factory_identity);
        assert_eq!(Card::from_name(token.identity()), Some(token));
        let wire = serde_json::to_value(token).unwrap();
        assert_eq!(wire, info.factory_identity);
        assert_eq!(serde_json::from_value::<Card>(wire).unwrap(), token);
        assert!(SearchFilter::Name(info.name).matches(token));
        if info.factory_identity != info.name {
            assert!(!SearchFilter::Name(info.factory_identity).matches(token));
        }
    }
    let original = Card::from_name("Coruscation Mage").unwrap();
    let copy = original.offspring_copy().unwrap();
    assert_eq!(copy.name(), original.name());
    assert_ne!(copy.identity(), original.identity());
}

#[test]
fn different_factories_with_one_rules_name_are_destroyed_together_after_paid_cast() {
    let recipes: Vec<_> = Card::catalogue()
        .filter(|card| card.is_token() && card.name() == "Goblin Token")
        .collect();
    assert!(recipes.len() >= 2);
    let left = recipes[0];
    let right = recipes[1];
    assert_ne!(left.identity(), right.identity());
    for player in [Player::P0, Player::P1] {
        let mut game = setup(player);
        let a = spawn(&mut game, player, left, Zone::Battlefield);
        let b = spawn(&mut game, player.other(), right, Zone::Battlefield);
        let other = named(
            &mut game,
            player.other(),
            "Crypt Feaster",
            Zone::Battlefield,
        );
        assert_eq!(game.object_view(a).name(), game.object_view(b).name());
        assert_eq!(game.object_view(a).name(), "Goblin Token");
        let pulse = named(&mut game, player, "Maelstrom Pulse", Zone::Hand);
        cast(&mut game, pulse, None, &[Target::Card(a)]);
        settle(&mut game);
        assert_eq!(game.object_view(a).zone, Zone::Outside);
        assert_eq!(game.object_view(b).zone, Zone::Outside);
        assert_eq!(game.object_view(other).zone, Zone::Battlefield);
    }
}

#[test]
fn named_and_copied_tokens_keep_string_names_in_owned_views() {
    for player in [Player::P0, Player::P1] {
        let mut game = setup(player);
        let scion = Card::catalogue()
            .find(|card| card.is_token() && card.name() == "Scion of the Deep")
            .unwrap();
        let original = Card::from_name("Coruscation Mage").unwrap();
        let copy = original.offspring_copy().unwrap();
        let named_token = spawn(&mut game, player, scion, Zone::Battlefield);
        let copied_token = spawn(&mut game, player, copy, Zone::Battlefield);
        let normal = spawn(&mut game, player, original, Zone::Battlefield);
        assert_eq!(game.object_view(named_token).name(), "Scion of the Deep");
        assert_eq!(
            game.object_view(copied_token).name(),
            game.object_view(normal).name()
        );
        let frozen = game.object_view(copied_token);
        let saved = game.snapshot();
        let branch = {
            let parent = game.clone();
            parent.clone()
        };
        game.move_object(copied_token, Zone::Graveyard);
        game.state_based_actions();
        let goblin = Card::catalogue()
            .find(|card| card.is_token() && card.name() == "Goblin Token")
            .unwrap();
        game.create_tokens(player, goblin, 1, false);
        assert_eq!(game.object_view(copied_token).name(), "Goblin Token");
        assert_ne!(game.object_view(copied_token).reference, frozen.reference);
        assert_eq!(frozen.name(), "Coruscation Mage");
        assert_eq!(branch.object_view(copied_token).name(), frozen.name());
        game.restore(&saved).unwrap();
        assert_eq!(game.object_view(copied_token).name(), frozen.name());
    }
}

#[test]
fn adventure_object_name_tracks_the_chosen_spell_face_then_the_physical_card() {
    for player in [Player::P0, Player::P1] {
        let mut game = setup(player);
        let target = named(&mut game, player, "Crypt Feaster", Zone::Battlefield);
        let adventure = named(&mut game, player, "Intrepid Trufflesnout", Zone::Hand);
        assert_eq!(game.object_view(adventure).name(), "Intrepid Trufflesnout");
        checked(&mut game, Action::AnnounceAdventure { card: adventure });
        checked(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        for _ in 0..8 {
            if game.state.announcing.is_none() {
                break;
            }
            let action = game
                .legal_actions()
                .into_iter()
                .find(|action| matches!(action, Action::FinishTargets | Action::PayCosts { .. }))
                .unwrap();
            checked(&mut game, action);
        }
        assert!(game.state.announcing.is_none());
        let stack_view = game.object_view(adventure);
        assert_eq!(stack_view.zone, Zone::Stack);
        assert_eq!(stack_view.name(), "Go Hog Wild");
        assert_eq!(
            game.spell_characteristics().last().unwrap().name,
            stack_view.name()
        );
        let saved = game.snapshot();
        settle(&mut game);
        assert_eq!(game.object_view(adventure).zone, Zone::Exile);
        assert_eq!(game.object_view(adventure).name(), "Intrepid Trufflesnout");
        assert_eq!(stack_view.name(), "Go Hog Wild");
        game.restore(&saved).unwrap();
        assert_eq!(game.object_view(adventure).name(), "Go Hog Wild");
        settle(&mut game);
        assert_eq!(game.object_view(adventure).name(), "Intrepid Trufflesnout");
    }
}
