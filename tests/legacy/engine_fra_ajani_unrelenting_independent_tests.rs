use super::*;

#[test]
fn ajani_minus_three_uses_current_token_controller_and_prevents_damage() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let ajani = put(&mut g, player, "Ajani Unrelenting", Zone::Battlefield);
        g.state.objects[ajani as usize].counters.loyalty = 5;
        let shielded = put(&mut g, player, "Hill Giant", Zone::Battlefield);
        g.add_counter(shielded, CounterKind::Shield, 1);
        g.create_tokens(player, Card::test_index(5298), 1, false);
        let own = g
            .state
            .objects
            .iter()
            .enumerate()
            .find_map(|(id, o)| {
                (o.zone == Zone::Battlefield
                    && o.owner == player
                    && o.is_token()
                    && o.card == Card::test_index(5298))
                .then_some(id as CardId)
            })
            .unwrap();
        g.create_tokens(player.other(), Card::test_index(5298), 1, false);
        let theirs = g
            .state
            .objects
            .iter()
            .enumerate()
            .find_map(|(id, o)| {
                (o.zone == Zone::Battlefield
                    && o.owner == player.other()
                    && o.is_token()
                    && o.card == Card::test_index(5298))
                .then_some(id as CardId)
            })
            .unwrap();
        act(
            &mut g,
            Action::AnnounceActivate {
                card: ajani,
                ability: 2,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        let own_ref = g.object_ref(own);
        let their_ref = g.object_ref(theirs);
        g.change_controller(own_ref, player.other());
        g.change_controller(their_ref, player);
        let before = g.snapshot();
        for replay in 0..2 {
            if replay == 1 {
                g.restore(&before).unwrap();
            }
            settle(&mut g);
            assert_eq!(g.object_view(shielded).zone, Zone::Battlefield);
            assert_eq!(g.object_view(shielded).damage, 0);
            assert_eq!(g.counter_count(shielded, CounterKind::Shield), 0);
            assert_ne!(g.object_view(own).zone, Zone::Battlefield);
            assert_eq!(g.object_view(theirs).zone, Zone::Battlefield);
        }
    }
}
