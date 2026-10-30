use super::*;

fn finish_ways(game: &mut Game) {
    for _ in 0..96 {
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
    panic!("Way triggers did not finish");
}

#[test]
fn mentor_uses_captured_controller_and_current_planeswalker_recipients() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let way = put(&mut game, player, "Way of the Mentor", Zone::Battlefield);
        let own = put(
            &mut game,
            player,
            "Liliana, Dreadhorde General",
            Zone::Battlefield,
        );
        let other = put(
            &mut game,
            player.other(),
            "Ajani, Caller of the Pride",
            Zone::Battlefield,
        );
        game.add_counter(own, CounterKind::Loyalty, 5);
        game.add_counter(other, CounterKind::Loyalty, 5);
        game.gain_life(player, 1);
        let own_ref = game.object_ref(own);
        let other_ref = game.object_ref(other);
        let way_ref = game.object_ref(way);
        game.change_controller(own_ref, player.other());
        game.change_controller(other_ref, player);
        game.change_controller(way_ref, player.other());
        let saved = game.snapshot();
        super::super::contracts::branch_contract(game.clone());
        for replay in 0..2 {
            if replay == 1 {
                game.restore(&saved).unwrap();
            }
            finish_ways(&mut game);
            assert_eq!(game.counter_count(own, CounterKind::Loyalty), 5);
            assert_eq!(game.counter_count(other, CounterKind::Loyalty), 6);
        }
    }
}

#[test]
fn necromancer_counts_simultaneous_friendly_token_deaths_and_replaces_each_counter() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let way = put(
            &mut game,
            player,
            "Way of the Necromancer",
            Zone::Battlefield,
        );
        let walker = put(
            &mut game,
            player,
            "Liliana, Dreadhorde General",
            Zone::Battlefield,
        );
        game.add_counter(walker, CounterKind::Loyalty, 5);
        let creature = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let opposing = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
        game.create_tokens(player, Card::test_index(5298), 1, false);
        let token = game
            .state
            .objects
            .iter()
            .enumerate()
            .find_map(|(id, o)| {
                (o.is_token()
                    && o.zone == Zone::Battlefield
                    && o.controller == player
                    && o.card == Card::test_index(5298))
                .then_some(id as CardId)
            })
            .unwrap();
        put(&mut game, player, "Doubling Season", Zone::Battlefield);
        let observers = game.death_trigger_sources();
        let events = game.move_objects_raw(&[creature, token, opposing], Zone::Graveyard);
        game.finish_zone_events(&events, &observers);
        game.move_object(way, Zone::Exile);
        let saved = game.snapshot();
        super::super::contracts::branch_contract(game.clone());
        for replay in 0..2 {
            if replay == 1 {
                game.restore(&saved).unwrap();
            }
            finish_ways(&mut game);
            assert_eq!(game.counter_count(walker, CounterKind::Loyalty), 9);
        }
    }
}

#[test]
fn paradox_trigger_resolves_after_loyalty_ability_is_countered_and_permits_two_land_plays() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        put(&mut game, player, "Way of the Paradox", Zone::Battlefield);
        game.create_tokens(player, Card::JacePlaneswalkerToken, 1, false);
        let jace = game
            .state
            .objects
            .iter()
            .enumerate()
            .find_map(|(id, o)| {
                (o.is_token()
                    && o.zone == Zone::Battlefield
                    && o.controller == player
                    && o.card == Card::JacePlaneswalkerToken)
                    .then_some(id as CardId)
            })
            .unwrap();
        game.state.objects[jace as usize].counters.loyalty = 5;
        let first = put(&mut game, player, "Forest", Zone::Hand);
        let second = put(&mut game, player, "Forest", Zone::Hand);
        let life = game.state.players[player.index()].life;
        act(
            &mut game,
            Action::AnnounceActivate {
                card: jace,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut game);
        let ability = game
            .state
            .stack
            .iter()
            .find(|item| {
                item.source.is_some_and(|source| source.id == jace)
                    && matches!(item.kind, StackKind::ActivatedAbility { .. })
            })
            .unwrap()
            .id;
        assert!(game.counter_stack_item(ability, false));
        let saved = game.snapshot();
        super::super::contracts::branch_contract(game.clone());
        for replay in 0..2 {
            if replay == 1 {
                game.restore(&saved).unwrap();
            }
            finish_ways(&mut game);
            assert_eq!(game.state.players[player.index()].life, life + 1);
            act(&mut game, Action::PlayLand { card: first });
            act(&mut game, Action::PlayLand { card: second });
            assert_eq!(game.object_view(first).zone, Zone::Battlefield);
            assert_eq!(game.object_view(second).zone, Zone::Battlefield);
        }
    }
}
