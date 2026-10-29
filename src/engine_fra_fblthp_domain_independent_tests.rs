use super::*;

#[test]
fn fblthp_domain_tracks_layered_types_control_and_ability_loss() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let fblthp = put(
            &mut game,
            player,
            "Fblthp, Knows the Way",
            Zone::Battlefield,
        );
        let dual = put(&mut game, player, "Hedge Maze", Zone::Battlefield);
        let enemy = put(&mut game, player.other(), "Swamp", Zone::Battlefield);
        assert_eq!(stats(&game, fblthp), (2, 2));
        let original = game.object_ref(dual);
        game.apply_temporary_copy(
            original,
            CopyRecipe::new(Card::from_name("Mountain").unwrap()),
            CopyDeadline::EndOfTurn,
        );
        assert_eq!(stats(&game, fblthp), (1, 2));
        let enemy_ref = game.object_ref(enemy);
        game.change_controller(enemy_ref, player);
        assert_eq!(stats(&game, fblthp), (2, 2));
        let aura = put(
            &mut game,
            player.other(),
            "Witness Protection",
            Zone::Battlefield,
        );
        game.state.objects[aura as usize].attached_to = Some(game.object_ref(fblthp));
        assert_eq!(stats(&game, fblthp), (1, 1));
        game.move_object(aura, Zone::Graveyard);
        assert_eq!(stats(&game, fblthp), (2, 2));
        let saved = game.snapshot();
        game.move_object(fblthp, Zone::Graveyard);
        assert_eq!(game.stats(fblthp).power, 2);
        game.restore(&saved).unwrap();
        assert_eq!(stats(&game, fblthp), (2, 2));
    }
}

#[test]
fn copied_fblthp_spell_carries_x_into_its_entry_search() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let source = put(&mut game, player, "Fblthp, Knows the Way", Zone::Hand);
        let forest = game.state.players[player.index()].library[0];
        let island = put(&mut game, player, "Island", Zone::Library);
        act(&mut game, Action::AnnounceCast { card: source });
        act(&mut game, Action::ChooseX { value: 2 });
        pay(&mut game);
        let original = game.state.stack.last().unwrap().clone();
        let copy = game.allocate_spell_copy(&original, player);
        assert_eq!(copy.x, Some(2));
        let copy_card = copy.source.unwrap().id;
        game.push_stack_item(copy);
        assert!(game.counter_stack_item(original.id, false));
        for _ in 0..64 {
            game.prepare_triggers();
            if matches!(game.state.pending, Pending::EffectSearch { .. }) {
                break;
            }
            choose(&mut game, |a| {
                matches!(a, Action::Pass | Action::PutTrigger { .. })
            });
        }
        assert!(matches!(game.state.pending, Pending::EffectSearch { .. }));
        assert!(game.object_view(copy_card).is_token());
        assert_eq!(game.object_view(copy_card).zone, Zone::Battlefield);
        let before = game.snapshot();
        super::super::contracts::branch_contract(game.clone());
        for replay in 0..2 {
            if replay == 1 {
                game.restore(&before).unwrap();
            }
            act(&mut game, Action::ChooseSearch { card: forest });
            act(&mut game, Action::ChooseSearch { card: island });
            act(&mut game, Action::FinishSearch);
            settle(&mut game);
            assert_eq!(game.object_view(forest).zone, Zone::Hand);
            assert_eq!(game.object_view(island).zone, Zone::Hand);
        }
    }
}
