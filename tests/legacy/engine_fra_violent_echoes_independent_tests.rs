use super::*;

#[test]
fn violent_echoes_empowers_for_replacement_increased_damage() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        put(&mut g, player, "Twinflame Tyrant", Zone::Battlefield);
        let target = put(&mut g, player.other(), "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, player, "Violent Echoes", Zone::Hand);
        cast(&mut g, spell, Some(Target::Card(target)));
        let saved = g.snapshot();
        for replay in 0..2 {
            if replay == 1 {
                g.restore(&saved).unwrap();
            }
            settle(&mut g);
            let jace = g
                .state
                .objects
                .iter()
                .enumerate()
                .find_map(|(id, object)| {
                    (object.zone == Zone::Battlefield
                        && object.controller == player
                        && object.is_token()
                        && object.card == Card::JacePlaneswalkerToken)
                        .then_some(id as CardId)
                })
                .unwrap();
            // Twinflame changes 6 to 12; a 2/2 requires 2 lethal damage.
            assert_eq!(g.counter_count(jace, CounterKind::Loyalty), 10);
        }
    }
}

#[test]
fn violent_echoes_counts_excess_after_wolverine_heals_previous_damage() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let target = put(
            &mut game,
            player.other(),
            "Wolverine, Fierce Fighter",
            Zone::Battlefield,
        );
        game.state.objects[target as usize].damage = 3;
        let spell = put(&mut game, player, "Violent Echoes", Zone::Hand);
        cast(&mut game, spell, Some(Target::Card(target)));
        let saved = game.snapshot();
        for replay in 0..2 {
            if replay == 1 {
                game.restore(&saved).unwrap();
            }
            settle(&mut game);
            let jace = game
                .state
                .objects
                .iter()
                .enumerate()
                .find_map(|(id, object)| {
                    (object.zone == Zone::Battlefield
                        && object.controller == player
                        && object.is_token()
                        && object.card == Card::JacePlaneswalkerToken)
                        .then_some(id as CardId)
                })
                .unwrap();
            // The replacement heals 3 marked damage first; 6 damage minus
            // Wolverine's 5 toughness is one excess, not four.
            assert_eq!(game.counter_count(jace, CounterKind::Loyalty), 1);
        }
    }
}
