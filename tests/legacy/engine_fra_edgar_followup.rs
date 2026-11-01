//! Independent Edgar death and counter timing interactions.
use super::*;

#[test]
fn bloodlord_sees_other_simultaneous_deaths_even_when_he_dies() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let edgar = put(&mut g, p, "Edgar, Ancient Bloodlord", Zone::Battlefield);
        let ally = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let stolen = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let stolen_ref = g.object_ref(stolen);
        g.change_controller(stolen_ref, p);
        let life = g.state.players[p.index()].life;
        let observers = g.death_trigger_sources();
        let events = g.move_objects_raw(&[edgar, ally, stolen, enemy], Zone::Graveyard);
        g.finish_zone_events(&events, &observers);
        let saved = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        for replay in 0..2 {
            if replay == 1 {
                g.restore(&saved).unwrap();
            }
            for _ in 0..48 {
                g.prepare_triggers();
                if g.state.pending == Pending::Priority
                    && g.state.stack.is_empty()
                    && g.state.pending_triggers.is_empty()
                    && g.state.resolving.is_none()
                {
                    break;
                }
                choose(&mut g, |a| {
                    matches!(a, Action::Pass | Action::PutTrigger { .. })
                });
            }
            assert_eq!(g.state.players[p.index()].life, life + 2);
            assert_eq!(g.object_view(stolen).owner, p.other());
        }
    }
}

#[test]
fn moonlit_activated_counters_sample_live_eligibility_and_replacements() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let edgar = put(&mut g, p, "Edgar, Moonlit Sovereign", Zone::Battlefield);
        let loses = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let gains = put(&mut g, p, "Hill Giant", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.add_counter(loses, CounterKind::PlusOne, 1);
        g.add_counter(enemy, CounterKind::PlusOne, 1);
        act(
            &mut g,
            Action::AnnounceActivate {
                card: edgar,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        g.state.objects[loses as usize].counters.plus_one = 0;
        g.add_counter(gains, CounterKind::PlusOne, 1);
        put(&mut g, p, "Doubling Season", Zone::Battlefield);
        g.move_object(edgar, Zone::Graveyard);
        let saved = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        for replay in 0..2 {
            if replay == 1 {
                g.restore(&saved).unwrap();
            }
            settle(&mut g);
            assert_eq!(g.counter_count(loses, CounterKind::PlusOne), 0);
            assert_eq!(g.counter_count(gains, CounterKind::PlusOne), 3);
            assert_eq!(g.counter_count(enemy, CounterKind::PlusOne), 1);
        }
    }
}
