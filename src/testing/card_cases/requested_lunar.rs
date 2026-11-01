//! Focused printed outcomes; shared engine invariants live in core tests.
//! Lunar Convocation's intervening-if triggers and paid draw.
use super::*;

#[test]
fn lunar_life_paid_draw_and_late_gain_do_not_retroactively_trigger() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let source = put(&mut g, p, "Lunar Convocation", Zone::Battlefield);
        let life = g.state.players[p.index()].life;
        let hand = g.hand_count(p);
        big_helpers::checked(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        assert_eq!(g.state.players[p.index()].life, life - 2);
        big_helpers::settle(&mut g);
        assert_eq!(g.hand_count(p), hand + 1);
        let enemy_life = g.state.players[p.other().index()].life;
        g.begin_step(Step::End);
        g.give_priority();
        g.prepare_triggers();
        g.gain_life(p, 1);
        big_helpers::settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, enemy_life);
        assert!(!g.state.objects.iter().any(|o| o.zone == Zone::Battlefield
            && o.controller == p
            && o.card == Card::from_name("@token/6570bb4cb1a0220e62f056e8").unwrap()));
    }
}

#[test]
fn lunar_prevented_gain_does_not_satisfy_its_conditions() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        put(&mut g, p, "Lunar Convocation", Zone::Battlefield);
        put(&mut g, p.other(), "Giant Cindermaw", Zone::Battlefield);
        let life = g.state.players[p.index()].life;
        let enemy = g.state.players[p.other().index()].life;
        g.gain_life(p, 3);
        g.lose_life(p, 1);
        assert_eq!(g.state.players[p.index()].life, life - 1);
        assert_eq!(g.state.players[p.index()].life_gained_this_turn, 0);
        g.begin_step(Step::End);
        g.give_priority();
        g.prepare_triggers();
        big_helpers::settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, enemy);
        assert!(!g.state.objects.iter().any(|o| o.zone == Zone::Battlefield
            && o.controller == p
            && o.card == Card::from_name("@token/6570bb4cb1a0220e62f056e8").unwrap()));
    }
}
