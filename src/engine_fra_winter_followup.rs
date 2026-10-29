//! Winter reflexive timing and planeswalker sacrifice.
use super::*;

fn wait_for_choice(g: &mut Game, p: Player) {
    for _ in 0..48 {
        if matches!(g.state.pending, Pending::EffectChoice { player } if player == p) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("Winter sacrifice choice unavailable: {:?}", g.state.pending);
}

#[test]
fn winter_planeswalker_sacrifice_to_exile_creates_opponents_reflexive_choice() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Rest in Peace", Zone::Battlefield);
        let walker = put(&mut g, p, "Ajani, Caller of the Pride", Zone::Battlefield);
        g.state.objects[walker as usize].counters.loyalty = 3;
        let first = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let second = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let winter = put(&mut g, p, "Winter, Tormented Loner", Zone::Hand);
        cast(&mut g, winter, None);
        wait_for_choice(&mut g, p);
        let walker_ref = g.object_ref(walker);
        act(&mut g, Action::Sacrifice { object: walker_ref });
        assert_eq!(g.object_view(walker).zone, Zone::Exile);
        // The already-created reflexive ability survives its source leaving.
        g.move_object(winter, Zone::Exile);
        wait_for_choice(&mut g, p.other());
        let saved = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        for replay in 0..2 {
            if replay == 1 {
                g.restore(&saved).unwrap();
            }
            let chosen = if replay == 0 { first } else { second };
            let other = if replay == 0 { second } else { first };
            let object = g.object_ref(chosen);
            act(&mut g, Action::Sacrifice { object });
            settle(&mut g);
            assert_eq!(g.object_view(chosen).zone, Zone::Exile);
            assert_eq!(g.object_view(other).zone, Zone::Battlefield);
        }
    }
}

#[test]
fn winter_opponent_with_no_creature_has_no_invalid_sacrifice_choice() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let walker = put(&mut g, p, "Ajani, Caller of the Pride", Zone::Battlefield);
        g.state.objects[walker as usize].counters.loyalty = 3;
        let opposing_walker = put(
            &mut g,
            p.other(),
            "Ajani, Caller of the Pride",
            Zone::Battlefield,
        );
        g.state.objects[opposing_walker as usize].counters.loyalty = 3;
        let winter = put(&mut g, p, "Winter, Tormented Loner", Zone::Hand);
        cast(&mut g, winter, None);
        wait_for_choice(&mut g, p);
        let object = g.object_ref(walker);
        act(&mut g, Action::Sacrifice { object });
        settle(&mut g);
        assert_eq!(g.object_view(opposing_walker).zone, Zone::Battlefield);
    }
}
