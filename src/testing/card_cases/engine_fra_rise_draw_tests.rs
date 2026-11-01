//! Focused printed outcomes; shared engine invariants live in core tests.
use super::*;

fn cast_mode(g: &mut Game, p: Player, mode: u16) -> CardId {
    let spell = put(g, p, "Rise of the Deathbringer", Zone::Hand);
    act(g, Action::AnnounceCast { card: spell });
    act(g, Action::ChooseMode { mode });
    pay(g);
    spell
}

#[test]
fn rise_zero_or_negative_power_draws_nothing_and_wipe_is_temporary() {
    {
        let p = Player::P0;
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
    {
        let p = Player::P0;
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
