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

#[test]
fn proliferate_is_non_targeted_repeatable_and_snapshot_owned() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let victim = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.add_counter(victim, CounterKind::PlusOne, 1);
        g.add_counter(victim, CounterKind::Stun, 1);
        g.state.players[p.other().index()].poison_counters = 1;
        g.state.players[p.index()].poison_counters = 1;
        let item = g.activation_item(p, source, 0);
        g.state.resolving = Some(Resolution {
            chosen_creature_type: None, target_counter_removal: None,
            linked_copy: None, cascade: None, target_card_facts: None,
            sacrificed_power: None, immediate_blink_return: None,
            connive: None, proliferate: None, explore: None, evidence: None,
            item, mana_payment: None, forage: None, target_library: None,
            program: &[Instruction::Proliferate(AmountValue::Fixed(2))],
            pc: 0, target_group: 0, moved_count: None, remembered_draws: None,
            discard_after: None, discards: [0; 2], selected_discards: [vec![], vec![]],
            milled: vec![], milled_selection: None, library: None, selection: None,
            return_attachment: None, search: None, search_result: None,
            optional_searches: None, choice: None,
        });
        g.run_program();
        let object = g.object_ref(victim);
        let permanent = Action::ProliferatePermanent { object };
        let player = Action::ProliferatePlayer { player: p.other() };
        let before = g.snapshot();
        assert!(g.apply(p.other(), permanent.clone()).is_err());
        assert_eq!(g.snapshot(), before);
        act(&mut g, permanent.clone());
        reject(&mut g, permanent.clone());
        act(&mut g, player.clone());
        reject(&mut g, player.clone());
        act(&mut g, Action::ProliferatePlayer { player: p });
        let observed = g.observe(p.other()).proliferate_choice.unwrap();
        assert_eq!(observed.selected_permanents, vec![object]);
        assert_eq!(observed.selected_players, vec![Player::P0, Player::P1]);
        act(&mut g, Action::FinishProliferate);
        assert_eq!(g.counter_count(victim, CounterKind::PlusOne), 2);
        assert_eq!(g.counter_count(victim, CounterKind::Stun), 2);
        assert_eq!(g.state.players[p.other().index()].poison_counters, 2);
        assert_eq!(g.state.players[p.index()].poison_counters, 2);
        act(&mut g, permanent);
        act(&mut g, player);
        act(&mut g, Action::FinishProliferate);
        assert_eq!(g.counter_count(victim, CounterKind::PlusOne), 3);
        assert_eq!(g.counter_count(victim, CounterKind::Stun), 3);
        assert_eq!(g.state.players[p.other().index()].poison_counters, 3);
        g.restore(&before).unwrap();
        assert_eq!(g.snapshot(), before);
        act(&mut g, Action::FinishProliferate);
        act(&mut g, Action::FinishProliferate);
        assert_eq!(g.counter_count(victim, CounterKind::PlusOne), 1);
        assert_eq!(g.state.players[p.other().index()].poison_counters, 1);
    }
}
