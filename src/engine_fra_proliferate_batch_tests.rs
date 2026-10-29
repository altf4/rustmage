use super::*;

#[test]
fn mixed_player_and_permanent_counters_wait_for_replacements_and_restore() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let walker = put(&mut g, p, "Jace, the Perfected Mind", Zone::Battlefield);
        put(&mut g, p, "Doubling Season", Zone::Battlefield);
        put(&mut g, p, "Doc Samson, Super Psychiatrist", Zone::Battlefield);
        let reference = g.object_ref(walker);
        let before = g.counter_count(walker, CounterKind::Loyalty);
        g.state.players[p.other().index()].poison_counters = 1;
        let mut poison = [0; 2];
        poison[p.other().index()] = 1;
        g.effect_counters_with_poison(
            vec![(reference, Counters::of(CounterKind::Loyalty, 1))],
            poison, p, replacements::ResumeMode::Pending,
        );
        assert!(g.advance_replacements(false));
        assert!(matches!(g.state.pending, Pending::Replacement { .. }));
        assert_eq!(g.state.players[p.other().index()].poison_counters, 1);
        assert_eq!(g.counter_count(walker, CounterKind::Loyalty), before);
        let saved = g.snapshot();
        let choices: Vec<_> = g.legal_actions().into_iter()
            .filter(|a| matches!(a, Action::ChooseReplacement { .. })).collect();
        assert_eq!(choices.len(), 2);
        for choice in choices {
            g.restore(&saved).unwrap();
            act(&mut g, choice);
            assert_eq!(g.state.players[p.other().index()].poison_counters, 2);
            let added = g.counter_count(walker, CounterKind::Loyalty) - before;
            assert!(added == 3 || added == 4);
        }
    }
}

#[test]
fn mixed_counter_batch_handles_player_only_and_empty_events() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let before = g.snapshot();
        g.effect_counters_with_poison(vec![], [0; 2], p, replacements::ResumeMode::Pending);
        assert_eq!(g.snapshot(), before);
        let mut poison = [0; 2];
        poison[p.index()] = 1;
        g.effect_counters_with_poison(vec![], poison, p, replacements::ResumeMode::Pending);
        assert_eq!(g.state.players[p.index()].poison_counters, 1);
    }
}
