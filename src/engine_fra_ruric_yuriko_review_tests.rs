use super::*;

fn finish_combat_trigger(g: &mut Game) {
    for _ in 0..96 {
        g.prepare_triggers();
        if g.state.pending == Pending::Priority && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty() && g.state.resolving.is_none() { return; }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("combat triggers did not finish: {:?}", g.state.pending);
}

#[test]
fn ruric_uncounterable_spell_survives_actual_cancel_and_enters_with_printed_keywords() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let ruric = put(&mut g, p, "Ruric Thar, Magecrusher", Zone::Hand);
        let cancel = put(&mut g, p.other(), "Cancel", Zone::Hand);
        cast(&mut g, ruric, None);
        act(&mut g, Action::Pass);
        assert_eq!(g.actor(), Some(p.other()));
        cast(&mut g, cancel, Some(Target::Spell(ruric)));
        let saved = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        for _ in 0..2 {
            settle(&mut g);
            assert_eq!(g.object_view(cancel).zone, Zone::Graveyard);
            assert_eq!(g.object_view(ruric).zone, Zone::Battlefield);
            assert_eq!(stats(&g, ruric), (7, 7));
            for keyword in [Keywords::REACH, Keywords::VIGILANCE, Keywords::TRAMPLE, Keywords::HEXPROOF] {
                assert!(g.keywords(ruric).contains(keyword));
            }
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn yuriko_never_grants_double_strike_when_two_creatures_attack() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let yuriko = put(&mut g, p, "Yuriko, Blade of the Mighty", Zone::Battlefield);
        let first = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let second = put(&mut g, p, "Hill Giant", Zone::Battlefield);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: first });
        act(&mut g, Action::Attack { card: second });
        act(&mut g, Action::FinishAttackers);
        assert!(!g.state.stack.iter().chain(g.state.pending_triggers.iter()).chain(g.state.ordering_triggers.iter())
            .any(|s| s.card == yuriko && matches!(s.kind, StackKind::TriggeredAbility { .. })));
        finish_combat_trigger(&mut g);
        assert!(!g.keywords(first).contains(Keywords::DOUBLE_STRIKE));
        assert!(!g.keywords(second).contains(Keywords::DOUBLE_STRIKE));
    }
}

#[test]
fn yuriko_trigger_does_not_grant_double_strike_to_reentered_attacker() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let yuriko = put(&mut g, p, "Yuriko, Blade of the Mighty", Zone::Battlefield);
        let attacker = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: attacker });
        act(&mut g, Action::FinishAttackers);
        assert!(g.state.stack.iter().chain(g.state.pending_triggers.iter()).chain(g.state.ordering_triggers.iter())
            .any(|s| s.card == yuriko && matches!(s.kind, StackKind::TriggeredAbility { .. })));
        let prior = g.object_ref(attacker);
        g.move_object(attacker, Zone::Exile);
        g.move_object(attacker, Zone::Battlefield);
        assert_ne!(prior, g.object_ref(attacker));
        let saved = g.snapshot();
        for _ in 0..2 {
            finish_combat_trigger(&mut g);
            assert!(!g.keywords(attacker).contains(Keywords::DOUBLE_STRIKE));
            g.restore(&saved).unwrap();
        }
    }
}
