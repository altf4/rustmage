//! Focused printed outcomes; shared engine invariants live in core tests.
use super::*;

fn garruk_curse_activate(g: &mut Game, source: CardId, ability: u16) {
    act(
        g,
        Action::AnnounceActivate {
            card: source,
            ability,
            counter_payment: Counters::default(),
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
    pay(g);
    settle(g);
}

#[test]
fn garruk_curse_beast_and_large_entry_draw() {
    {
        let player = Player::P0;
        let mut g = scenario(player);
        let source = put(&mut g, player, "Garruk, Curse Breaker", Zone::Battlefield);
        g.state.objects[source as usize].counters.loyalty = 5;
        garruk_curse_activate(&mut g, source, 1);
        let token = g
            .state
            .objects
            .iter()
            .enumerate()
            .find_map(|(id, object)| {
                (object.zone == Zone::Battlefield
                    && object.card == Card::from_name("@token/0bdfcac57503bc474764a02e").unwrap())
                .then_some(id as CardId)
            })
            .unwrap();
        assert_eq!(stats(&g, token), (4, 4));
        assert!(g.keywords(token).contains(Keywords::TRAMPLE));
        assert_eq!(g.state.players[player.index()].cards_drawn_this_turn, 1);
    }
}
