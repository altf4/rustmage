use super::*;

fn activate_repentant(game: &mut Game, source: CardId, target: CardId) {
    act(
        game,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    act(
        game,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    pay(game);
}
fn finish_repentant(game: &mut Game) {
    for _ in 0..80 {
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
    panic!("Liliana continuation did not finish");
}

#[test]
fn repentant_blinked_source_does_not_receive_the_old_activation_counter() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let source = put(
            &mut game,
            player,
            "Liliana the Repentant",
            Zone::Battlefield,
        );
        let target = put(&mut game, player, "Bear Cub", Zone::Graveyard);
        activate_repentant(&mut game, source, target);
        let old = game.object_ref(source);
        game.move_object(source, Zone::Exile);
        game.move_object(source, Zone::Battlefield);
        assert_ne!(game.object_ref(source), old);
        let saved = game.snapshot();
        super::super::contracts::branch_contract(game.clone());
        for replay in 0..2 {
            if replay == 1 {
                game.restore(&saved).unwrap();
            }
            finish_repentant(&mut game);
            assert_eq!(game.object_view(target).zone, Zone::Battlefield);
            assert_eq!(game.counter_count(source, CounterKind::PlusOne), 0);
            assert_eq!(game.state.players[player.index()].library.len(), 6);
        }
    }
}

#[test]
fn repentant_prevented_entry_still_places_counter_on_the_original_source() {
    // No admitted library/graveyard entry prohibition exists; inject a skipped
    // staged entry after a real target and entry choice, as the shared engine tests do.
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let source = put(
            &mut game,
            player,
            "Liliana the Repentant",
            Zone::Battlefield,
        );
        let target = put(&mut game, player, "Adaptive Automaton", Zone::Graveyard);
        activate_repentant(&mut game, source, target);
        for _ in 0..48 {
            if matches!(game.state.pending, Pending::EntryValue { .. }) {
                break;
            }
            choose(&mut game, |a| {
                matches!(a, Action::Pass | Action::PutTrigger { .. })
            });
        }
        assert!(matches!(game.state.pending, Pending::EntryValue { .. }));
        let target_ref = game.object_ref(target);
        let batch = game.state.entry_batch.as_mut().unwrap();
        assert_eq!(batch.moves[0].before.reference, target_ref);
        batch.moves[0].skipped = true;
        let saved = game.snapshot();
        super::super::contracts::branch_contract(game.clone());
        for replay in 0..2 {
            if replay == 1 {
                game.restore(&saved).unwrap();
            }
            choose(
                &mut game,
                |a| matches!(a, Action::ChooseEntryValue { value: EntryChoiceValue::CreatureType(t) } if t.name() == "Dwarf"),
            );
            finish_repentant(&mut game);
            assert_eq!(game.object_view(target).zone, Zone::Graveyard);
            assert_eq!(game.object_ref(target), target_ref);
            assert_eq!(game.counter_count(source, CounterKind::PlusOne), 1);
            assert_eq!(game.state.players[player.index()].library.len(), 8);
        }
    }
}

#[test]
fn copied_repentant_ability_uses_copys_target_and_original_source() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let source = put(
            &mut game,
            player,
            "Liliana the Repentant",
            Zone::Battlefield,
        );
        let own = put(&mut game, player, "Bear Cub", Zone::Graveyard);
        let enemy = put(&mut game, player.other(), "Hill Giant", Zone::Graveyard);
        activate_repentant(&mut game, source, own);
        let mut copy = game.state.stack.last().unwrap().clone();
        copy.id = game.next_stack_id();
        copy.controller = player.other();
        copy.is_copy = true;
        copy.targets[0].target = Target::Card(enemy);
        copy.targets[0].object = Some(game.object_ref(enemy));
        game.push_stack_item(copy);
        let saved = game.snapshot();
        super::super::contracts::branch_contract(game.clone());
        for replay in 0..2 {
            if replay == 1 {
                game.restore(&saved).unwrap();
            }
            finish_repentant(&mut game);
            assert_eq!(game.object_view(own).zone, Zone::Battlefield);
            assert_eq!(game.object_view(own).controller, player);
            assert_eq!(game.object_view(enemy).zone, Zone::Battlefield);
            assert_eq!(game.object_view(enemy).controller, player.other());
            assert_eq!(game.counter_count(source, CounterKind::PlusOne), 2);
            assert_eq!(game.state.players[player.index()].library.len(), 6);
            assert_eq!(game.state.players[player.other().index()].library.len(), 8);
        }
    }
}
