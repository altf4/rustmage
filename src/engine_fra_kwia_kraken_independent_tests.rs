use super::*;

fn count_token(g: &Game, player: Player, card: u16) -> usize {
    g.state
        .objects
        .iter()
        .filter(|o| {
            o.is_token()
                && o.zone == Zone::Battlefield
                && o.controller == player
                && o.card == Card::test_index(card)
        })
        .count()
}

#[test]
fn kwia_countered_trigger_is_used_but_new_incarnation_can_trigger() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let kwia = put(&mut g, player, "Kwia Vigorbloom", Zone::Battlefield);
        g.gain_life(player, 1);
        g.prepare_triggers();
        for _ in 0..32 {
            if g.state.stack.iter().any(|item| item.card == kwia) {
                break;
            }
            choose(&mut g, |a| {
                matches!(a, Action::Pass | Action::PutTrigger { .. })
            });
        }
        let id = g
            .state
            .stack
            .iter()
            .find(|item| item.card == kwia)
            .unwrap()
            .id;
        assert!(g.counter_stack_item(id, false));
        g.gain_life(player, 1);
        g.prepare_triggers();
        settle(&mut g);
        assert_eq!(count_token(&g, player, 5453), 0);
        let previous = g.object_ref(kwia);
        g.move_object(kwia, Zone::Exile);
        g.move_object(kwia, Zone::Battlefield);
        assert_ne!(g.object_ref(kwia), previous);
        g.gain_life(player, 1);
        g.prepare_triggers();
        let before = g.snapshot();
        for replay in 0..2 {
            if replay == 1 {
                g.restore(&before).unwrap();
            }
            settle(&mut g);
            assert_eq!(count_token(&g, player, 5453), 1);
        }
    }
}

#[test]
fn kraken_opponents_upkeep_token_creation_is_replaced_for_controller() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player.other());
        put(&mut g, player, "Verdant Kraken", Zone::Battlefield);
        put(&mut g, player, "Doubling Season", Zone::Battlefield);
        g.begin_step(Step::Upkeep);
        g.prepare_triggers();
        settle(&mut g);
        assert_eq!(count_token(&g, player, 5454), 2);
        assert_eq!(count_token(&g, player.other(), 5454), 0);
        let tokens: Vec<_> = g
            .state
            .objects
            .iter()
            .enumerate()
            .filter_map(|(id, object)| {
                (object.is_token()
                    && object.zone == Zone::Battlefield
                    && object.card == Card::test_index(5454))
                .then_some(id as CardId)
            })
            .collect();
        for token in tokens {
            assert_eq!(stats(&g, token), (3, 3));
            assert!(g.has_subtype(token, "Forest") && g.has_subtype(token, "Tentacle"));
        }
    }
}
