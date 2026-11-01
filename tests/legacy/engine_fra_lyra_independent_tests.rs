use super::*;

fn grant_draw(game: &mut Game, lyra: CardId) {
    act(
        game,
        Action::AnnounceActivate {
            card: lyra,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    pay(game);
    settle(game);
}
fn finish_triggers(game: &mut Game) {
    for _ in 0..64 {
        game.prepare_triggers();
        if game.state.pending == Pending::Priority
            && game.state.stack.is_empty()
            && game.state.pending_triggers.is_empty()
            && game.state.resolving.is_none()
        {
            return;
        }
        choose(game, |a| {
            matches!(a, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("trigger resolution did not finish");
}
fn damage_player(game: &mut Game, lyra: CardId, player: Player) {
    let event = DamageEvent {
        source: game.damage_source(lyra),
        target: Target::Player(player),
        amount: 1,
        combat: true,
        preventable: true,
    };
    game.apply_damage_batch(&[event]);
}

#[test]
fn lyra_each_activation_grants_exactly_one_independent_trigger_and_uses_captured_controller() {
    for player in [Player::P0, Player::P1] {
        for grants in [1, 2] {
            let mut game = scenario(player);
            let lyra = put(
                &mut game,
                player,
                "Lyra, Tolarian Archangel",
                Zone::Battlefield,
            );
            for _ in 0..grants {
                grant_draw(&mut game, lyra);
            }
            let drawn = game.state.players[player.index()].cards_drawn_this_turn;
            damage_player(&mut game, lyra, player.other());
            assert_eq!(game.state.pending_triggers.len(), grants);
            game.prepare_triggers();
            let source = game.object_ref(lyra);
            game.change_controller(source, player.other());
            game.move_object(lyra, Zone::Exile);
            let before = game.snapshot();
            super::super::contracts::branch_contract(game.clone());
            for replay in 0..2 {
                if replay == 1 {
                    game.restore(&before).unwrap();
                }
                finish_triggers(&mut game);
                assert_eq!(
                    game.state.players[player.index()].cards_drawn_this_turn,
                    drawn + 2 * grants as u64
                );
                assert_eq!(
                    game.state.players[player.other().index()].cards_drawn_this_turn,
                    0
                );
            }
        }
    }
}

#[test]
fn lyra_removal_after_grant_suppresses_until_aura_leaves() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let lyra = put(
            &mut game,
            player,
            "Lyra, Tolarian Archangel",
            Zone::Battlefield,
        );
        grant_draw(&mut game, lyra);
        let aura = put(
            &mut game,
            player.other(),
            "Witness Protection",
            Zone::Battlefield,
        );
        game.state.objects[aura as usize].attached_to = Some(game.object_ref(lyra));
        damage_player(&mut game, lyra, player.other());
        assert!(game.state.pending_triggers.is_empty());
        game.move_object(aura, Zone::Graveyard);
        damage_player(&mut game, lyra, player.other());
        assert_eq!(game.state.pending_triggers.len(), 1);
        game.prepare_triggers();
        settle(&mut game);
        assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 2);
    }
}

#[test]
fn lyra_grant_resolving_after_removal_is_newer_and_survives_copy_change() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let lyra = put(
            &mut game,
            player,
            "Lyra, Tolarian Archangel",
            Zone::Battlefield,
        );
        act(
            &mut game,
            Action::AnnounceActivate {
                card: lyra,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut game);
        let aura = put(
            &mut game,
            player.other(),
            "Witness Protection",
            Zone::Battlefield,
        );
        game.state.objects[aura as usize].attached_to = Some(game.object_ref(lyra));
        settle(&mut game);
        damage_player(&mut game, lyra, player.other());
        assert_eq!(game.state.pending_triggers.len(), 1);
        game.prepare_triggers();
        settle(&mut game);
        assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 2);
        let source = game.object_ref(lyra);
        game.apply_temporary_copy(
            source,
            CopyRecipe::new(Card::from_name("Bear Cub").unwrap()),
            CopyDeadline::EndOfTurn,
        );
        damage_player(&mut game, lyra, player.other());
        assert_eq!(game.state.pending_triggers.len(), 1);
    }
}

#[test]
fn lyra_end_step_angel_has_printed_name_owner_and_flying() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        put(
            &mut game,
            player,
            "Lyra, Tolarian Archangel",
            Zone::Battlefield,
        );
        game.draw_many(player, 3);
        game.begin_step(Step::End);
        game.prepare_triggers();
        settle(&mut game);
        let angel = game
            .state
            .objects
            .iter()
            .enumerate()
            .find_map(|(id, o)| {
                (o.is_token() && o.zone == Zone::Battlefield && o.card == Card::test_index(5457))
                    .then_some(id as CardId)
            })
            .unwrap();
        let view = game.object_view(angel);
        assert_eq!(view.name(), "Angel Token");
        assert_eq!(view.owner, player);
        assert_eq!(view.controller, player);
        assert_eq!(view.colors, 2);
        assert_eq!(stats(&game, angel), (3, 3));
        assert!(game.has_subtype(angel, "Angel"));
        assert!(game.keywords(angel).contains(Keywords::FLYING));
    }
}
