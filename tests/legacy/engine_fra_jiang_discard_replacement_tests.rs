use super::*;

#[test]
fn jiang_counts_discards_redirected_to_exile_and_replays() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Jiang Yanggu, Alone", Zone::Battlefield);
        put(&mut g, p, "Rest in Peace", Zone::Battlefield);
        let attacker = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let prior = put(&mut g, p, "Forest", Zone::Hand);
        let selected = put(&mut g, p, "Island", Zone::Hand);
        g.discard_batch(&[prior], None);
        assert_eq!(g.object_view(prior).zone, Zone::Exile);
        assert_eq!(g.state.players[p.index()].cards_discarded_this_turn, 1);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: attacker });
        act(&mut g, Action::FinishAttackers);
        for _ in 0..32 {
            if g.legal_actions()
                .contains(&Action::Discard { card: selected })
            {
                break;
            }
            choose(&mut g, |a| {
                matches!(a, Action::Pass | Action::PutTrigger { .. })
            });
        }
        assert!(
            g.legal_actions()
                .contains(&Action::Discard { card: selected })
        );
        let library = g.state.players[p.index()].library.len();
        let saved = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        for replay in 0..2 {
            if replay == 1 {
                g.restore(&saved).unwrap();
            }
            act(&mut g, Action::Discard { card: selected });
            settle(&mut g);
            assert_eq!(g.object_view(selected).zone, Zone::Exile);
            assert_eq!(g.state.players[p.index()].cards_discarded_this_turn, 2);
            assert_eq!(g.state.players[p.index()].library.len(), library - 1);
            assert_eq!(g.counter_count(attacker, CounterKind::PlusOne), 2);
        }
    }
}
