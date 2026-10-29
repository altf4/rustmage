use super::*;

#[test]
fn rise_counts_only_its_own_draws_when_an_independent_request_is_queued() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        put(&mut g, player, "Lorehold, the Historian", Zone::Battlefield);
        let rise = put(&mut g, player, "Rise of the Deathbringer", Zone::Hand);
        let before = g.state.players[player.index()].life;
        act(&mut g, Action::AnnounceCast { card: rise });
        act(&mut g, Action::ChooseMode { mode: 0 });
        pay(&mut g);
        for _ in 0..24 {
            if matches!(g.state.pending, Pending::MiracleReveal { .. }) {
                break;
            }
            act(&mut g, Action::Pass);
        }
        assert!(matches!(g.state.pending, Pending::MiracleReveal { .. }));
        // Synthetic independent request: the current public actions do not expose
        // a way to initiate a second draw while this private choice is pending.
        g.draw_many(player, 1);
        let saved = g.snapshot();
        for replay in 0..2 {
            if replay == 1 {
                g.restore(&saved).unwrap();
            }
            act(&mut g, Action::DeclineMiracle);
            settle(&mut g);
            assert_eq!(g.state.players[player.index()].cards_drawn_this_turn, 6);
            assert_eq!(g.state.players[player.index()].life, before - 5);
        }
    }
}
