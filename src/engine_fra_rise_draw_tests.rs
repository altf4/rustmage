use super::*;

fn cast_mode(g: &mut Game, p: Player, mode: u16) -> CardId {
    let spell = put(g, p, "Rise of the Deathbringer", Zone::Hand);
    act(g, Action::AnnounceCast { card: spell });
    act(g, Action::ChooseMode { mode });
    pay(g);
    spell
}

#[test]
fn rise_counts_greatest_current_power_and_actual_replacement_draws() {
    for p in [Player::P0, Player::P1] {
        for doubled in [false, true] {
            let mut g = scenario(p);
            put(&mut g, p, "Bear Cub", Zone::Battlefield);
            if doubled {
                put(&mut g, p, "Bard, King of Dale", Zone::Battlefield);
            }
            let amount = if doubled { 6 } else { 2 };
            let before = g.state.players[p.index()].life;
            let drawn = g.state.players[p.index()].cards_drawn_this_turn;
            cast_mode(&mut g, p, 0);
            let saved = g.snapshot();
            super::super::contracts::branch_contract(g.clone());
            for replay in 0..2 {
                if replay == 1 {
                    g.restore(&saved).unwrap();
                }
                settle(&mut g);
                assert_eq!(
                    g.state.players[p.index()].cards_drawn_this_turn,
                    drawn + amount
                );
                assert_eq!(g.state.players[p.index()].life, before - i128::from(amount));
            }
        }
    }
}

#[test]
fn rise_defers_life_loss_until_all_miracle_interrupted_draws_finish() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Lorehold, the Historian", Zone::Battlefield);
        let before = g.state.players[p.index()].life;
        cast_mode(&mut g, p, 0);
        for _ in 0..24 {
            if matches!(g.state.pending, Pending::MiracleReveal { .. }) {
                break;
            }
            act(&mut g, Action::Pass);
        }
        assert!(matches!(g.state.pending, Pending::MiracleReveal { player } if player == p));
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
        assert_eq!(g.state.players[p.index()].life, before);
        assert!(g.observe(p).miracle.is_some());
        assert!(g.observe(p.other()).miracle.is_none());
        let saved = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        for replay in 0..2 {
            if replay == 1 {
                g.restore(&saved).unwrap();
            }
            act(&mut g, Action::DeclineMiracle);
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 5);
            assert_eq!(g.state.players[p.index()].life, before - 5);
        }
    }
}

#[test]
fn rise_zero_or_negative_power_draws_nothing_and_wipe_is_temporary() {
    for p in [Player::P0, Player::P1] {
        for creature in [false, true] {
            let mut g = scenario(p);
            if creature {
                let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
                g.state.objects[bear as usize].growth.power = -4;
            }
            let before = g.state.players[p.index()].life;
            cast_mode(&mut g, p, 0);
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
            assert_eq!(g.state.players[p.index()].life, before);
        }
        let mut g = scenario(p);
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let large = put(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
        cast_mode(&mut g, p, 1);
        settle(&mut g);
        assert_eq!(g.object_view(bear).zone, Zone::Graveyard);
        assert_eq!(stats(&g, large), (1, 1));
        g.finish_cleanup();
        assert_eq!(stats(&g, large), (4, 4));
    }
}

#[test]
fn rise_loses_only_life_for_successful_draws_when_library_runs_out() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Bear Cub", Zone::Battlefield);
        for id in g.state.players[p.index()].library.clone() {
            g.move_object(id, Zone::Outside);
        }
        put(&mut g, p, "Forest", Zone::Library);
        let life = g.state.players[p.index()].life;
        cast_mode(&mut g, p, 0);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
        assert_eq!(g.state.players[p.index()].life, life - 1);
        assert!(g.state.players[p.index()].attempted_empty_draw);
    }
}

#[test]
fn rise_counts_only_its_own_draw_request_across_a_miracle_pause() {
    for p in [Player::P0, Player::P1] {
        for doubled in [false, true] {
            let mut g = scenario(p);
            put(&mut g, p, "Lorehold, the Historian", Zone::Battlefield);
            if doubled {
                put(&mut g, p, "Bard, King of Dale", Zone::Battlefield);
            }
            for _ in 0..6 {
                put(&mut g, p, "Forest", Zone::Library);
            }
            let before = g.state.players[p.index()].life;
            cast_mode(&mut g, p, 0);
            for _ in 0..24 {
                if matches!(g.state.pending, Pending::MiracleReveal { .. }) {
                    break;
                }
                act(&mut g, Action::Pass);
            }
            assert!(matches!(g.state.pending, Pending::MiracleReveal { player } if player == p));
            assert_eq!(
                g.state.resolving.as_ref().unwrap().remembered_draws,
                Some(1)
            );
            // This separate draw queues behind Rise's remaining draws; it must not
            // become part of the spell's "drawn this way" life-loss quantity.
            g.draw_many(p, 1);
            let saved = g.snapshot();
            super::super::contracts::branch_contract(g.clone());
            for replay in 0..2 {
                if replay == 1 {
                    g.restore(&saved).unwrap();
                }
                act(&mut g, Action::DeclineMiracle);
                settle(&mut g);
                let multiplier = if doubled { 2 } else { 1 };
                assert_eq!(
                    g.state.players[p.index()].cards_drawn_this_turn,
                    6 * multiplier
                );
                assert_eq!(
                    g.state.players[p.index()].life,
                    before - 5 * i128::from(multiplier)
                );
            }
        }
    }
}
