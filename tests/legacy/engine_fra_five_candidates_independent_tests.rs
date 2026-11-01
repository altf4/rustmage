use super::*;

#[test]
fn silence_alternatives_keep_taxes_and_reductions_under_normal_and_free_casts() {
    for player in [Player::P0, Player::P1] {
        for free in [false, true] {
            for discount in [false, true] {
                for route in [0u16, 1] {
                    let mut game = scenario(player);
                    put(
                        &mut game,
                        player.other(),
                        "Thalia, the Survivor",
                        Zone::Battlefield,
                    );
                    if discount {
                        put(
                            &mut game,
                            player,
                            "Geist of Saint Thalia",
                            Zone::Battlefield,
                        );
                    }
                    if free {
                        put(&mut game, player, "Omniscience", Zone::Battlefield);
                    }
                    let sacrifice = put(&mut game, player, "Bear Cub", Zone::Battlefield);
                    let target = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
                    let spell = put(&mut game, player, "Silence the Echo", Zone::Hand);
                    game.state.players[player.index()].mana = [0, 0, u32::from(!free), 0, 0, 5];
                    let action = if free {
                        Action::AnnounceFree {
                            card: spell,
                            face: CastFace::Normal,
                            flash_surcharge: false,
                        }
                    } else {
                        Action::AnnounceCast { card: spell }
                    };
                    act(&mut game, action);
                    act(&mut game, Action::ChooseSpellCost { cost: route });
                    act(
                        &mut game,
                        Action::ChooseTarget {
                            target: Target::Card(target),
                        },
                    );
                    if route == 0 {
                        let object = game.object_ref(sacrifice);
                        act(&mut game, Action::ChooseAdditionalCost { object });
                    }
                    if game.legal_actions().contains(&Action::FinishTargets) {
                        act(&mut game, Action::FinishTargets);
                    }
                    let total = game.state.announcing.as_ref().unwrap().total_cost.unwrap();
                    let generic = (u32::from(!free) + 1 + 3 * u32::from(route))
                        .saturating_sub(u32::from(discount));
                    assert_eq!(total.generic, generic);
                    assert_eq!(total.colored[2], u32::from(!free));
                    let saved = game.snapshot();
                    super::super::contracts::branch_contract(game.clone());
                    for replay in 0..2 {
                        if replay == 1 {
                            game.restore(&saved).unwrap();
                        }
                        pay(&mut game);
                        assert_eq!(game.state.players[player.index()].mana[5], 5 - generic);
                        assert_eq!(
                            game.object_view(sacrifice).zone,
                            if route == 0 {
                                Zone::Graveyard
                            } else {
                                Zone::Battlefield
                            }
                        );
                        settle(&mut game);
                        assert_eq!(game.object_view(target).zone, Zone::Graveyard);
                    }
                }
            }
        }
    }
}

#[test]
fn silence_selected_sacrifice_restores_when_transaction_is_rolled_back() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let sacrifice = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let target = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
        let spell = put(&mut game, player, "Silence the Echo", Zone::Hand);
        act(&mut game, Action::AnnounceCast { card: spell });
        act(&mut game, Action::ChooseSpellCost { cost: 0 });
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        let original = game.object_ref(sacrifice);
        act(&mut game, Action::ChooseAdditionalCost { object: original });
        // Inject the transaction failure after the real selected sacrifice commits.
        game.begin_attack_rollback(player);
        let mut announcement = game.state.announcing.as_ref().unwrap().clone();
        game.pay_additional_cost(&mut announcement);
        assert_eq!(game.object_view(sacrifice).zone, Zone::Graveyard);
        game.finish_cast_rollback(false);
        assert_eq!(game.object_view(sacrifice).zone, Zone::Battlefield);
        assert_eq!(game.object_ref(sacrifice), original);
        assert_eq!(game.object_view(target).zone, Zone::Battlefield);
        super::super::contracts::branch_contract(game);
    }
}

#[test]
fn charge_snapshots_global_recipients_and_counter_replacement() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let own = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let spell = put(&mut game, player, "Charge the Sanctum", Zone::Hand);
        act(&mut game, Action::AnnounceCast { card: spell });
        act(&mut game, Action::ChooseMode { mode: 0 });
        if game
            .legal_actions()
            .iter()
            .any(|a| matches!(a, Action::ChooseSpellCost { .. }))
        {
            choose(&mut game, |a| matches!(a, Action::ChooseSpellCost { .. }));
        }
        pay(&mut game);
        settle(&mut game);
        let later = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        assert_eq!(stats(&game, own), (4, 2));
        assert_eq!(stats(&game, later), (2, 2));
        let saved = game.snapshot();
        game.finish_cleanup();
        assert_eq!(stats(&game, own), (2, 2));
        game.restore(&saved).unwrap();
        assert_eq!(stats(&game, own), (4, 2));

        let mut game = scenario(player);
        let target = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
        put(
            &mut game,
            player.other(),
            "Doubling Season",
            Zone::Battlefield,
        );
        let spell = put(&mut game, player, "Charge the Sanctum", Zone::Hand);
        act(&mut game, Action::AnnounceCast { card: spell });
        act(&mut game, Action::ChooseMode { mode: 1 });
        if game
            .legal_actions()
            .iter()
            .any(|a| matches!(a, Action::ChooseSpellCost { .. }))
        {
            choose(&mut game, |a| matches!(a, Action::ChooseSpellCost { .. }));
        }
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        pay(&mut game);
        settle(&mut game);
        assert_eq!(game.counter_count(target, CounterKind::PlusOne), 2);
        assert_eq!(stats(&game, target), (6, 4));
        assert!(game.keywords(target).contains(Keywords::FIRST_STRIKE));
    }
}

#[test]
fn arni_counts_entering_token_and_keeps_source_through_control_change() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let arni = put(
            &mut game,
            player,
            "Arni, Renowned Champion",
            Zone::Battlefield,
        );
        game.create_tokens(player, Card::test_index(5298), 1, false);
        game.prepare_triggers();
        let reference = game.object_ref(arni);
        game.change_controller(reference, player.other());
        let saved = game.snapshot();
        super::super::contracts::branch_contract(game.clone());
        for replay in 0..2 {
            if replay == 1 {
                game.restore(&saved).unwrap();
            }
            settle(&mut game);
            assert_eq!(stats(&game, arni), (3, 5));
            assert_eq!(game.object_view(arni).controller, player.other());
        }
    }
}
