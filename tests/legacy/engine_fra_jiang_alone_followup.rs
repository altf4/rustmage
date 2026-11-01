//! Independent Jiang player-only solo attack interactions.
use super::*;

#[test]
fn jiang_with_empty_hand_still_draws_and_counts_only_previous_discards() {
    for p in [Player::P0, Player::P1] {
        for previous in [false, true] {
            let mut g = scenario(p);
            let jiang = put(&mut g, p, "Jiang Yanggu, Alone", Zone::Battlefield);
            if previous {
                let old = put(&mut g, p, "Forest", Zone::Hand);
                g.discard_batch(&[old], None);
            }
            let before = g.state.players[p.index()].library.len();
            g.state.step = Step::Attackers;
            g.state.pending = Pending::Attackers;
            act(&mut g, Action::Attack { card: jiang });
            act(&mut g, Action::FinishAttackers);
            let saved = g.snapshot();
            super::super::contracts::branch_contract(g.clone());
            for replay in 0..2 {
                if replay == 1 {
                    g.restore(&saved).unwrap();
                }
                settle(&mut g);
                assert_eq!(g.state.players[p.index()].library.len(), before - 1);
                assert_eq!(
                    g.counter_count(jiang, CounterKind::PlusOne),
                    u64::from(previous)
                );
                assert!(g.keywords(jiang).contains(Keywords::MENACE));
            }
        }
    }
}

#[test]
fn jiang_trigger_survives_source_departure_and_attacker_control_change() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let jiang = put(&mut g, p, "Jiang Yanggu, Alone", Zone::Battlefield);
        let attacker = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let discarded = put(&mut g, p, "Forest", Zone::Hand);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: attacker });
        act(&mut g, Action::FinishAttackers);
        g.move_object(jiang, Zone::Exile);
        let attacker_ref = g.object_ref(attacker);
        g.change_controller(attacker_ref, p.other());
        for _ in 0..32 {
            if g.legal_actions()
                .contains(&Action::Discard { card: discarded })
            {
                break;
            }
            choose(&mut g, |a| {
                matches!(a, Action::Pass | Action::PutTrigger { .. })
            });
        }
        let saved = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        for replay in 0..2 {
            if replay == 1 {
                g.restore(&saved).unwrap();
            }
            act(&mut g, Action::Discard { card: discarded });
            settle(&mut g);
            assert_eq!(g.object_view(attacker).controller, p.other());
            assert_eq!(g.counter_count(attacker, CounterKind::PlusOne), 1);
        }
    }
}
