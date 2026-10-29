use super::*;

fn arc_entry_settle(g: &mut Game) {
    for _ in 0..128 {
        g.prepare_triggers();
        if g.state.pending == Pending::Priority && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty() && g.state.resolving.is_none() { return; }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("Arc of Fortune did not settle: {:?}", g.state.pending);
}

#[test]
fn arc_mixed_simultaneous_discard_waits_for_entry_timestamp_then_draws_both_hands() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Variable Chaser // Arc of Fortune", Zone::Hand);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 2];
        act(&mut g, Action::AnnounceCast { card: source });
        pay(&mut g);
        arc_entry_settle(&mut g);
        let permanent = g.object_ref(source);
        let prepared = g.state.preparations.as_ref().unwrap().iter()
            .find(|row| row.permanent == permanent).unwrap().spell_copy;
        put(&mut g, p, "Rest in Peace", Zone::Battlefield);
        let mine = put(&mut g, p, "Forest", Zone::Hand);
        let other = put(&mut g, p.other(), "Island", Zone::Hand);
        let first = put(&mut g, p.other(), "Wilt-Leaf Liege", Zone::Hand);
        let second = put(&mut g, p.other(), "Wilt-Leaf Liege", Zone::Hand);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 2];
        act(&mut g, Action::AnnounceExile { card: prepared.card });
        pay(&mut g);
        for _ in 0..64 {
            if matches!(g.state.pending, Pending::EffectOptional { .. }) { break; }
            choose(&mut g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
        }
        act(&mut g, Action::ChooseOptional { accept: true });
        let before_second_choice = g.snapshot();
        for _ in 0..2 {
            act(&mut g, Action::ChooseOptional { accept: true });
            // Each Liege's affected controller chooses its own replacement over
            // Rest in Peace. The other discarded cards still go to exile.
            for _ in 0..8 {
                if matches!(g.state.pending, Pending::EntryTimestamp { .. }) { break; }
                let choice = g.legal_actions().into_iter().find(|a| matches!(a,
                    Action::ChooseZoneReplacement { effect: ZoneReplacementKind::Battlefield }))
                    .unwrap_or_else(|| panic!("expected replacement choice, got {:?}", g.state.pending));
                assert_eq!(g.actor(), Some(p.other()));
                act(&mut g, choice);
            }
            assert!(matches!(g.state.pending, Pending::EntryTimestamp { player } if player == p.other()));
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
            assert_eq!(g.state.players[p.other().index()].cards_drawn_this_turn, 0);
            let checkpoint = g.snapshot();
            super::super::contracts::branch_contract(g.clone());
            let selected = g.object_ref(second);
            act(&mut g, Action::ChooseEntryTimestamp { object: selected });
            arc_entry_settle(&mut g);
            assert_eq!(g.object_view(first).zone, Zone::Battlefield);
            assert_eq!(g.object_view(second).zone, Zone::Battlefield);
            assert_eq!(g.object_view(first).controller, p.other());
            assert_eq!(g.object_view(second).controller, p.other());
            assert_eq!(g.object_view(mine).zone, Zone::Exile);
            assert_eq!(g.object_view(other).zone, Zone::Exile);
            assert_eq!(g.state.players[p.index()].cards_discarded_this_turn, 1);
            assert_eq!(g.state.players[p.other().index()].cards_discarded_this_turn, 3);
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 7);
            assert_eq!(g.state.players[p.other().index()].cards_drawn_this_turn, 7);
            let after = g.snapshot();
            g.restore(&checkpoint).unwrap();
            act(&mut g, Action::ChooseEntryTimestamp { object: selected });
            arc_entry_settle(&mut g);
            assert_eq!(g.snapshot(), after);
            g.restore(&before_second_choice).unwrap();
        }
    }
}
