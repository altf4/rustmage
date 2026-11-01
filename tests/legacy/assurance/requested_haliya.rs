//! Haliya's entry lifegain and end-step resolution condition.
use super::*;

#[test]
fn haliya_counts_each_controlled_entry_and_checks_life_on_resolution() {
    for p in [Player::P0, Player::P1] {
        for gain_after_trigger in [false, true] {
            let mut g = scenario(p);
            let source = put(&mut g, p, "Haliya, Guided by Light", Zone::Hand);
            let life = g.state.players[p.index()].life;
            g.move_object(source, Zone::Battlefield);
            g.prepare_triggers();
            big_helpers::settle(&mut g);
            assert_eq!(g.state.players[p.index()].life, life + 1);
            let own = put(&mut g, p, "Bear Cub", Zone::Hand);
            let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Hand);
            g.move_object(own, Zone::Battlefield);
            g.move_object(enemy, Zone::Battlefield);
            g.prepare_triggers();
            big_helpers::settle(&mut g);
            assert_eq!(g.state.players[p.index()].life, life + 2);
            let before = g.hand_count(p);
            g.begin_step(Step::End);
            g.give_priority();
            g.prepare_triggers();
            if gain_after_trigger {
                g.gain_life(p, 1);
                g.lose_life(p, 5);
            }
            big_helpers::settle(&mut g);
            assert_eq!(g.hand_count(p), before + usize::from(gain_after_trigger));
        }
    }
}
