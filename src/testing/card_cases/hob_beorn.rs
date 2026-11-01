//! Focused printed outcomes; shared engine invariants live in core tests.
//! Printed outcomes and shared permanent self-grant/type contracts for both Beorns.
use super::*;
fn finish(g: &mut Game) {
    for _ in 0..64 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty()
            && g.state.ordering_triggers.is_empty()
        {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("Beorn resolution budget {:?}", g.state.pending);
}
fn until(g: &mut Game, predicate: impl Fn(&Action) -> bool) {
    for _ in 0..32 {
        if g.legal_actions().iter().any(&predicate) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("Beorn choice budget {:?}", g.state.pending);
}
fn paid(g: &mut Game, p: Player, name: &str, mana: Mana) -> CardId {
    let id = put(g, p, name, Zone::Hand);
    g.state.players[p.index()].mana = mana;
    cast(g, id, None);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    finish(g);
    id
}
fn fierce(g: &mut Game, p: Player) -> CardId {
    paid(g, p, "Beorn the Fierce", [0, 0, 0, 0, 2, 3])
}
fn hospitality(g: &mut Game, p: Player) -> CardId {
    paid(g, p, "Beorn's Hospitality", [0, 0, 0, 0, 1, 1])
}
fn combat(g: &mut Game, p: Player) {
    g.state.active = p;
    g.begin_step(Step::BeginCombat);
    g.give_priority();
    g.prepare_triggers();
}
fn select(g: &mut Game, id: Option<CardId>) {
    until(g, |a| {
        matches!(a, Action::ChooseTarget { .. } | Action::FinishTargets)
    });
    if let Some(id) = id {
        act(
            g,
            Action::ChooseTarget {
                target: Target::Card(id),
            },
        );
    }
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
    if g.legal_actions()
        .iter()
        .any(|a| matches!(a, Action::PayCosts { .. }))
    {
        pay(g);
    }
}
fn start(g: &mut Game, p: Player, id: CardId) {
    g.state.priority = p;
    g.state.players[p.index()].mana = [0, 0, 0, 0, 2, 5];
    act(
        g,
        Action::AnnounceActivate {
            card: id,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    pay(g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
}
fn lands(g: &mut Game, p: Player, n: usize) -> Vec<CardId> {
    (0..n)
        .map(|_| put(g, p, "Forest", Zone::Battlefield))
        .collect()
}

#[test]
fn beorn_fierce_paid_body_other_own_bear_bonus_and_current_controller() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let goblin = put(&mut g, p, "Goblin Piker", Zone::Battlefield);
        let b = fierce(&mut g, p);
        assert_eq!(stats(&g, b), (6, 6));
        assert!(g.keywords(b).contains(Keywords::TRAMPLE));
        assert_eq!(stats(&g, own), (4, 4));
        assert_eq!(stats(&g, enemy), (2, 2));
        assert_eq!(stats(&g, goblin), (2, 1));
        g.state.objects[enemy as usize].controller = p;
        assert_eq!(stats(&g, enemy), (4, 4));
        g.move_object(b, Zone::Hand);
        assert_eq!(stats(&g, own), (2, 2));
        assert_eq!(stats(&g, enemy), (2, 2));
    }
}
#[test]
fn beorn_fierce_combat_counter_adds_bear_before_current_threshold_draw_and_persists() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let b = fierce(&mut g, p);
        put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let goblin = put(&mut g, p, "Goblin Piker", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        combat(&mut g, p);
        until(&mut g, |a| {
            matches!(a, Action::ChooseTarget { .. } | Action::FinishTargets)
        });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );

        select(&mut g, Some(goblin));
        finish(&mut g);
        assert_eq!(g.counter_count(goblin, CounterKind::Trample), 1);
        assert!(g.keywords(goblin).contains(Keywords::TRAMPLE));
        assert!(g.has_subtype(goblin, "Goblin") && g.has_subtype(goblin, "Bear"));
        assert_eq!(stats(&g, goblin), (4, 3));
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 2);
        let hand: Vec<_> = g
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(_, o)| o.zone == Zone::Hand && o.owner == p)
            .map(|(i, _)| i as CardId)
            .collect();
        for id in hand {
            assert!(g.observe(p).objects.iter().any(|o| o.id == id));
            assert!(g.observe(p.other()).objects.iter().all(|o| o.id != id));
        }
        g.move_object(b, Zone::Graveyard);
        g.finish_cleanup();
        assert!(g.has_subtype(goblin, "Bear") && g.has_subtype(goblin, "Goblin"));
        assert!(g.keywords(goblin).contains(Keywords::TRAMPLE));
        assert_eq!(stats(&g, goblin), (2, 1));
        g.move_object(goblin, Zone::Hand);
        g.move_object(goblin, Zone::Battlefield);
        assert!(!g.has_subtype(goblin, "Bear"));
        assert_eq!(g.counter_count(goblin, CounterKind::Trample), 0);
    }
}

#[test]
fn beorn_hospitality_self_granted_layer7b_overrides_actual_cda_layer7a() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        lands(&mut g, p, 3);
        let h = hospitality(&mut g, p);
        start(&mut g, p, h);
        // Copy changes retain the exact object and queued original activation.
        // The copied definition's real CDA gives0/1 with empty graveyards.
        g.apply_temporary_copy(
            g.object_ref(h),
            CopyRecipe::new(Card::from_name("Tarmogoyf").unwrap()),
            CopyDeadline::EndOfTurn,
        );
        assert_eq!(stats(&g, h), (0, 1));
        finish(&mut g);
        assert_eq!(stats(&g, h), (3, 3));
        assert!(g.has_subtype(h, "Lhurgoyf") && g.has_subtype(h, "Bear"));
        g.add_counter(h, CounterKind::PlusOne, 1);
        assert_eq!(stats(&g, h), (4, 4));
        g.finish_cleanup();
        assert_eq!(
            g.object_view(h).card,
            Card::from_name("Beorn's Hospitality").unwrap()
        );
        assert_eq!(stats(&g, h), (4, 4));
        assert!(g.has_subtype(h, "Bear"));
    }
}
