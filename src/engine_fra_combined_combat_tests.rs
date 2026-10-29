use super::*;

#[test]
fn ingris_jiang_and_tetsuko_keep_distinct_attack_and_block_events() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let ingris = put(&mut g, p, "Ingris Stingerquill", Zone::Battlefield);
        let jiang = put(&mut g, p, "Jiang Yanggu, Alone", Zone::Battlefield);
        let tetsuko = put(&mut g, p, "Tetsuko Umezawa, Pursuer", Zone::Battlefield);
        let attacker = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let blocker = put(&mut g, p.other(), "Llanowar Elves", Zone::Battlefield);
        let discard = put(&mut g, p, "Forest", Zone::Hand);
        let own_life = g.state.players[p.index()].life;
        let enemy_life = g.state.players[p.other().index()].life;
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: attacker });
        act(&mut g, Action::FinishAttackers);
        g.prepare_triggers();
        assert!(matches!(g.state.pending, Pending::OrderTriggers { player } if player == p));
        let original = g.snapshot();
        for selected_first in [ingris, jiang] {
            g.restore(&original).unwrap();
            let first = g
                .state
                .ordering_triggers
                .iter()
                .find(|item| item.card == selected_first)
                .unwrap()
                .id;
            assert!(g.state.ordering_triggers.iter().any(|item| item.card
                == if selected_first == ingris {
                    jiang
                } else {
                    ingris
                }));
            act(&mut g, Action::PutTrigger { id: first });
            for _ in 0..96 {
                if g.state.pending == Pending::Blockers {
                    break;
                }
                let actions = g.legal_actions();
                let action = actions
                    .iter()
                    .find(|a| **a == Action::Discard { card: discard })
                    .or_else(|| {
                        actions
                            .iter()
                            .find(|a| matches!(a, Action::Pass | Action::PutTrigger { .. }))
                    })
                    .cloned()
                    .unwrap_or_else(|| panic!("unexpected combat state {:?}", g.state.pending));
                act(&mut g, action);
            }
            assert_eq!(g.state.pending, Pending::Blockers);
            assert_eq!(g.state.players[p.other().index()].life, enemy_life - 1);
            assert_eq!(g.counter_count(attacker, CounterKind::PlusOne), 1);
            assert_eq!(g.object_view(discard).zone, Zone::Graveyard);
            act(&mut g, Action::Block { blocker, attacker });
            act(&mut g, Action::FinishBlockers);
            assert!(
                g.state
                    .stack
                    .iter()
                    .chain(g.state.pending_triggers.iter())
                    .any(|item| item.card == tetsuko
                        && matches!(item.kind, StackKind::TriggeredAbility { .. }))
            );
            let old_blocker = g.object_ref(blocker);
            g.change_controller(old_blocker, p);
            g.move_object(blocker, Zone::Graveyard);
            let before_resolution = g.snapshot();
            for replay in 0..2 {
                if replay == 1 {
                    g.restore(&before_resolution).unwrap();
                }
                g.prepare_triggers();
                settle(&mut g);
                assert_eq!(g.state.players[p.index()].life, own_life - 1);
                assert_eq!(g.state.players[p.other().index()].life, enemy_life - 1);
                assert_eq!(g.counter_count(attacker, CounterKind::PlusOne), 1);
            }
        }
    }
}
