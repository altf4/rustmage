use super::*;

#[test]
fn ajani_zero_gains_life_and_triggers_loyalty_once_with_replay() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let ajani = put(&mut g, player, "Ajani Resolute", Zone::Battlefield);
        g.state.objects[ajani as usize].counters.loyalty = 2;
        let life = g.state.players[player.index()].life;
        act(
            &mut g,
            Action::AnnounceActivate {
                card: ajani,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        let saved = g.snapshot();
        for _ in 0..2 {
            settle(&mut g);
            assert_eq!(g.state.players[player.index()].life, life + 1);
            assert_eq!(g.counter_count(ajani, CounterKind::Loyalty), 3);
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn ajani_pridemate_token_tracks_controller_life_gain() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let ajani = put(&mut g, player, "Ajani Resolute", Zone::Battlefield);
        g.state.objects[ajani as usize].counters.loyalty = 5;
        act(
            &mut g,
            Action::AnnounceActivate {
                card: ajani,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        settle(&mut g);
        let token = g
            .state
            .objects
            .iter()
            .enumerate()
            .find_map(|(id, object)| {
                (object.zone == Zone::Battlefield && object.card == Card::test_index(5462))
                    .then_some(id as CardId)
            })
            .unwrap();
        assert_eq!(g.object_view(token).controller, player);
        assert_eq!(g.stats(token).power, 2);
        g.gain_life(player.other(), 1);
        g.prepare_triggers();
        settle(&mut g);
        assert_eq!(g.counter_count(token, CounterKind::PlusOne), 0);
        let saved = g.snapshot();
        for _ in 0..2 {
            g.gain_life(player, 1);
            g.prepare_triggers();
            // Ajani and the Pridemate both trigger on the same life gain.
            while matches!(g.state.pending, Pending::OrderTriggers { .. }) {
                choose(&mut g, |action| matches!(action, Action::PutTrigger { .. }));
            }
            settle(&mut g);
            assert_eq!(g.counter_count(token, CounterKind::PlusOne), 1);
            assert_eq!(g.counter_count(ajani, CounterKind::Loyalty), 2);
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn ajani_emblem_persists_after_source_departure_and_affects_only_controller() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let ajani = put(&mut g, player, "Ajani Resolute", Zone::Battlefield);
        let own = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        let other = put(&mut g, player.other(), "Bear Cub", Zone::Battlefield);
        g.state.objects[ajani as usize].counters.loyalty = 11;
        act(
            &mut g,
            Action::AnnounceActivate {
                card: ajani,
                ability: 2,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        settle(&mut g);
        let saved = g.snapshot();
        for _ in 0..2 {
            g.move_object(ajani, Zone::Graveyard);
            assert_eq!(g.stats(own).power, 4);
            assert_eq!(g.stats(own).toughness, 4);
            assert_eq!(g.stats(other).power, 2);
            g.restore(&saved).unwrap();
        }
    }
}
