//! Definition-specific outcomes for four inherited draw consumers.
use super::*;
fn drive(g: &mut Game) {
    for _ in 0..64 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty()
        {
            return;
        }
        choose(g, |a| matches!(a, Action::PutTrigger { .. } | Action::Pass));
    }
    panic!("draw trigger resolution budget");
}
fn paid(g: &mut Game, p: Player, name: &str, mana: Mana) -> CardId {
    let id = put(g, p, name, Zone::Hand);
    g.state.players[p.index()].mana = mana;
    cast(g, id, None);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    id
}
#[test]
fn hob_inherited_draw_paid_own_each_vs_second_global_history_and_reset() {
    for p in [Player::P0, Player::P1] {
        for (name, generic, keyword, expected) in [
            ("Lakeshore Apothecary", 1, Keywords::VIGILANCE, 1),
            ("Ravenhill Flock", 3, Keywords::FLYING, 2),
        ] {
            let mut g = scenario(p);
            g.draw(p); // first draw precedes the observer
            let source = paid(&mut g, p, name, [0, 1, 0, 0, 0, generic]);
            drive(&mut g);
            assert_eq!(stats(&g, source), (1, 2));
            assert!(g.keywords(source).contains(keyword));
            g.state.active = p.other();
            g.state.priority = p;
            paid(&mut g, p, "Quick Study", [0, 1, 0, 0, 0, 2]);
            drive(&mut g);
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 3);
            assert_eq!(g.counter_count(source, CounterKind::PlusOne), expected);
            g.draw(p.other());
            g.draw(p.other());
            g.prepare_triggers();
            drive(&mut g);
            assert_eq!(g.counter_count(source, CounterKind::PlusOne), expected);
            g.draw(p);
            g.prepare_triggers();
            drive(&mut g);
            let fourth = if name == "Ravenhill Flock" { 3 } else { 1 };
            assert_eq!(g.counter_count(source, CounterKind::PlusOne), fourth);
            g.begin_turn();
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
            g.draw(p);
            g.draw(p);
            g.prepare_triggers();
            drive(&mut g);
            assert_eq!(
                g.counter_count(source, CounterKind::PlusOne),
                fourth + expected
            );
        }
    }
}
#[test]
fn hob_inherited_draw_source_counters_never_rebind_reentered_source() {
    for p in [Player::P0, Player::P1] {
        for (name, generic) in [("Lakeshore Apothecary", 1), ("Ravenhill Flock", 3)] {
            let mut g = scenario(p);
            let source = paid(&mut g, p, name, [0, 1, 0, 0, 0, generic]);
            drive(&mut g);
            g.draw(p);
            g.draw(p);
            assert!(!g.state.pending_triggers.is_empty());
            g.move_object(source, Zone::Hand);
            g.move_object(source, Zone::Battlefield);
            g.give_priority();
            g.prepare_triggers();
            drive(&mut g);
            assert_eq!(g.counter_count(source, CounterKind::PlusOne), 0);
            assert_eq!(stats(&g, source), (1, 2));
        }
    }
}
#[test]
fn hob_inherited_draw_discovery_own_artifacts_tokens_and_departed_observer() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = paid(&mut g, p, "Fateful Discovery", [0, 2, 0, 0, 0, 3]);
        drive(&mut g);
        g.state.active = p.other();
        g.state.priority = p.other();
        paid(&mut g, p.other(), "Goldvein Pick", [0, 0, 0, 0, 0, 2]);
        drive(&mut g);
        assert_eq!(g.hand_count(p), 0);
        g.state.active = p;
        g.state.priority = p;
        paid(&mut g, p, "Grizzly Bears", [0, 0, 0, 0, 1, 1]);
        drive(&mut g);
        assert_eq!(g.hand_count(p), 0);
        paid(&mut g, p, "Goldvein Pick", [0, 0, 0, 0, 0, 2]);
        for _ in 0..16 {
            if g.state.stack.iter().any(|s| {
                s.source.is_some_and(|identity| identity.id == source)
                    && matches!(s.kind, StackKind::TriggeredAbility { .. })
            }) {
                break;
            }
            act(&mut g, Action::Pass);
        }
        assert!(
            g.state
                .stack
                .iter()
                .any(|s| s.source.is_some_and(|identity| identity.id == source)
                    && matches!(s.kind, StackKind::TriggeredAbility { .. }))
        );
        g.move_object(source, Zone::Graveyard);
        g.give_priority();
        drive(&mut g);
        assert_eq!(g.hand_count(p), 1);
        let mut g = scenario(p);
        paid(&mut g, p, "Fateful Discovery", [0, 2, 0, 0, 0, 3]);
        drive(&mut g);
        put(&mut g, p, "Smaug the Magnificent", Zone::Battlefield);
        g.begin_step(Step::Upkeep);
        g.give_priority();
        g.prepare_triggers();
        drive(&mut g);
        assert_eq!(g.hand_count(p), 1, "actual Treasure entry draws once");
        assert_eq!(
            g.state
                .objects
                .iter()
                .filter(|o| o.zone == Zone::Battlefield
                    && o.controller == p
                    && o.card.has_subtype("Treasure"))
                .count(),
            1
        );
    }
}
fn activate_splendor(g: &mut Game, p: Player, source: CardId) {
    g.state.priority = p;
    g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 2];
    act(
        g,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    act(
        g,
        Action::ChooseTarget {
            target: Target::Player(p),
        },
    );
    reject(g, Action::FinishTargets);
    reject(
        g,
        Action::ChooseTarget {
            target: Target::Player(p),
        },
    );
    act(
        g,
        Action::ChooseTarget {
            target: Target::Player(p.other()),
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
    pay(g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
}
fn treasures(g: &Game, p: Player) -> usize {
    g.state
        .objects
        .iter()
        .filter(|o| {
            o.zone == Zone::Battlefield && o.controller == p && o.card.has_subtype("Treasure")
        })
        .count()
}
#[test]
fn hob_inherited_draw_splendor_two_distinct_players_exact_opponent_second_draw() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = paid(&mut g, p, "Gleaming Splendor", [1, 0, 0, 0, 0, 1]);
        drive(&mut g);
        for ordinal in 1..=3 {
            activate_splendor(&mut g, p, source);
            drive(&mut g);
            for player in [p, p.other()] {
                assert_eq!(g.hand_count(player), ordinal);
                assert_eq!(
                    g.state.players[player.index()].cards_drawn_this_turn,
                    ordinal as u64
                );
            }
            assert_eq!(treasures(&g, p), usize::from(ordinal >= 2));
            assert_eq!(treasures(&g, p.other()), 0);
        }
        // A historical first draw before entry still makes the next one second.
        let mut g = scenario(p);
        g.draw(p.other());
        let source = paid(&mut g, p, "Gleaming Splendor", [1, 0, 0, 0, 0, 1]);
        drive(&mut g);
        g.draw(p.other());
        g.prepare_triggers();
        contracts::branch_contract(g.clone());
        g.move_object(source, Zone::Graveyard);
        g.give_priority();
        drive(&mut g);
        assert_eq!(treasures(&g, p), 1);
        g.draw(p.other());
        g.prepare_triggers();
        drive(&mut g);
        assert_eq!(treasures(&g, p), 1);
    }
}
