//! Controller life history and frozen death counters for Essence Channeler.
use super::*;

#[test]
fn channeler_life_history_and_death_transfer_use_frozen_source() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Essence Channeler", Zone::Battlefield);
        let target = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        assert!(!g.keywords(source).contains(Keywords::FLYING));
        g.lose_life(p.other(), 1);
        assert!(!g.keywords(source).contains(Keywords::FLYING));
        g.lose_life(p, 1);
        assert!(g.keywords(source).contains(Keywords::FLYING));
        assert!(g.keywords(source).contains(Keywords::VIGILANCE));
        g.gain_life(p, 3);
        big_helpers::settle(&mut g);
        assert_eq!(g.counter_count(source, CounterKind::PlusOne), 1);
        g.add_counter(source, CounterKind::Stun, 2);
        g.move_object(source, Zone::Graveyard);
        g.move_object(source, Zone::Battlefield);
        g.add_counter(source, CounterKind::PlusOne, 5);
        for _ in 0..100 {
            g.prepare_triggers();
            if g.state.pending == Pending::Priority
                && g.state.stack.is_empty()
                && g.state.pending_triggers.is_empty()
                && g.state.resolving.is_none()
                && g.state.announcing.is_none()
            {
                break;
            }
            let actions = g.legal_actions();
            let select = Action::ChooseTarget {
                target: Target::Card(target),
            };
            let action = if actions.contains(&select) {
                select
            } else if g.state.pending == Pending::Priority {
                Action::Pass
            } else {
                actions
                    .into_iter()
                    .find(|a| !matches!(a, Action::Concede))
                    .unwrap()
            };
            big_helpers::checked(&mut g, action);
        }
        assert_eq!(g.counter_count(target, CounterKind::PlusOne), 1);
        assert_eq!(g.counter_count(target, CounterKind::Stun), 2);
        assert_eq!(g.counter_count(source, CounterKind::PlusOne), 5);
        g.finish_cleanup();
        assert!(!g.keywords(source).contains(Keywords::FLYING));
    }
}

#[test]
fn channeler_death_triggers_even_without_counters() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Essence Channeler", Zone::Battlefield);
        put(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.move_object(source, Zone::Graveyard);
        assert!(!g.state.pending_triggers.is_empty());
    }
}
