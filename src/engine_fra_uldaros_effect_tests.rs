//! Uldaros remains gated until its complete interaction suite is validated.
use super::*;

#[test]
fn exile_targets_creates_independent_budgeted_card_copies_and_restores() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let source = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let first = put(&mut game, player, "Bear Cub", Zone::Graveyard);
        let second = put(&mut game, player, "Gilded Lotus", Zone::Graveyard);
        let stale = put(&mut game, player, "Shock", Zone::Graveyard);
        let mut item = game.activation_item(player, source, 0);
        item.targets = [first, second, stale].into_iter().map(|id| BoundTarget {
            group: 0, target: Target::Card(id), object: Some(game.object_ref(id)),
        }).collect();
        game.move_object(stale, Zone::Exile);
        let snapshot = game.snapshot();
        for _ in 0..2 {
            assert!(game.exile_targets_copy_free_with_budget(&item, 6));
            assert_eq!(game.state.objects[first as usize].zone, Zone::Exile);
            assert_eq!(game.state.objects[second as usize].zone, Zone::Exile);
            let offer = game.state.resolution_free_casts.as_ref().unwrap();
            assert_eq!(offer.player, player);
            assert_eq!(offer.remaining_mana_value, Some(6));
            assert_eq!(offer.remaining.len(), 2);
            for reference in &offer.remaining {
                let copy = &game.state.objects[reference.card as usize];
                assert_eq!(copy.copy_kind, CopyKind::Card);
                assert_eq!(copy.owner, player);
                assert_eq!(copy.zone, Zone::Exile);
            }
            game.restore(&snapshot).unwrap();
        }
    }
}

#[test]
fn cast_uldaros_resolves_selected_types_and_excludes_departed_target() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let creature = put(&mut game, player, "Bear Cub", Zone::Graveyard);
        let artifact = put(&mut game, player, "Gilded Lotus", Zone::Graveyard);
        let source = put(&mut game, player, "Shock", Zone::Hand);
        game.state.objects[source as usize].card = Card::test_index(4739);
        // Verify the gated fixture index rather than accidentally testing a different card.
        assert_eq!(game.state.objects[source as usize].card.identity(), "Uldaros Theorix");
        act(&mut game, Action::AnnounceCast { card: source });
        pay(&mut game);
        for _ in 0..40 {
            if matches!(game.state.pending, Pending::Targets { .. }) { break; }
            act(&mut game, Action::Pass);
        }
        act(&mut game, Action::ChooseTargetCardType { target: Target::Card(creature), type_mask: 1 });
        act(&mut game, Action::ChooseTargetCardType { target: Target::Card(artifact), type_mask: 16 });
        act(&mut game, Action::FinishTargets);
        game.move_object(artifact, Zone::Hand);
        let saved = game.snapshot();
        for _ in 0..2 {
            game.resolve_top();
            assert_eq!(game.state.objects[creature as usize].zone, Zone::Exile);
            assert_eq!(game.state.objects[artifact as usize].zone, Zone::Hand);
            let offer = game.state.resolution_free_casts.as_ref().unwrap();
            assert_eq!(offer.remaining_mana_value, Some(6));
            assert_eq!(offer.remaining.len(), 1);
            assert_eq!(game.state.objects[offer.remaining[0].card as usize].card.identity(), "Bear Cub");
            game.restore(&saved).unwrap();
        }
    }
}
