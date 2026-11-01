use super::*;

#[test]
fn tethermage_observes_one_doubled_loyalty_placement_as_one_event() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let mage = put(&mut g, player, "Inspired Tethermage", Zone::Battlefield);
        g.create_tokens(player, Card::JacePlaneswalkerToken, 1, false);
        let jace = g
            .state
            .objects
            .iter()
            .enumerate()
            .find_map(|(id, o)| {
                (o.zone == Zone::Battlefield
                    && o.controller == player
                    && o.is_token()
                    && o.card == Card::JacePlaneswalkerToken)
                    .then_some(id as CardId)
            })
            .unwrap();
        g.state.objects[jace as usize].counters.loyalty = 1;
        put(&mut g, player, "Doubling Season", Zone::Battlefield);
        g.state.players[player.index()].mana = [0, 0, 0, 0, 0, 6];
        act(
            &mut g,
            Action::AnnounceActivate {
                card: mage,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        let saved = g.snapshot();
        for replay in 0..2 {
            if replay == 1 {
                g.restore(&saved).unwrap();
            }
            settle(&mut g);
            assert_eq!(g.counter_count(jace, CounterKind::Loyalty), 5);
            assert_eq!(g.counter_count(mage, CounterKind::PlusOne), 2);
        }
    }
}
