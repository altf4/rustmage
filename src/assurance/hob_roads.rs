//! Search-linked Saga chapters and player-duration attack triggers.
use super::*;
use crate::engine::replacements::ResumeMode;
fn until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..100 {
        if ready(g) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("Roads decision budget {:?}", g.state.pending)
}
fn finish(g: &mut Game) {
    until(g, |g| {
        g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.ordering_triggers.is_empty()
            && g.state.resolving.is_none()
    });
}
fn step(g: &mut Game) {
    g.begin_step(Step::Main1);
    g.give_priority();
    g.prepare_triggers();
}
fn search(g: &mut Game) {
    until(g, |g| {
        matches!(g.state.pending, Pending::EffectSearch { .. })
    });
}
fn linked(g: &mut Game) {
    until(g, |g| {
        matches!(g.state.pending, Pending::LinkedExile { .. })
    });
}
fn cast_saga(g: &mut Game, p: Player) -> CardId {
    let id = put(g, p, "Roads Go Ever, Ever On", Zone::Hand);
    g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 1];
    cast(g, id, None);
    id
}
#[test]
fn hob_roads_paid_search_exact_links_and_individual_return() {
    for p in [Player::P0, Player::P1] {
        for n in [0, 1, 2] {
            let mut g = scenario(p);
            let plains = [
                put(&mut g, p, "Plains", Zone::Library),
                put(&mut g, p, "Plains", Zone::Library),
            ];
            let forest = put(&mut g, p, "Forest", Zone::Library);
            let id = cast_saga(&mut g, p);
            search(&mut g);
            assert!(g.observe(p).search_choice.is_some());
            assert!(g.observe(p.other()).search_choice.is_none());
            reject(&mut g, Action::ChooseSearch { card: forest });
            contracts::branch_contract(g.clone());
            for &card in plains.iter().take(n) {
                act(&mut g, Action::ChooseSearch { card });
            }
            act(&mut g, Action::FinishSearch);
            finish(&mut g);
            assert_eq!(g.state.players[p.index()].life, 22);
            assert_eq!(g.counter_count(id, CounterKind::Lore), 1);
            for &card in plains.iter().take(n) {
                assert_eq!(g.object_view(card).zone, Zone::Exile);
                assert!(g.observe(p.other()).objects.iter().any(|v| v.id == card));
            }
            for i in 0..2 {
                step(&mut g);
                if i < n {
                    linked(&mut g);
                    let card = plains[n - 1 - i];
                    let object = g.object_ref(card);
                    contracts::branch_contract(g.clone());
                    act(&mut g, Action::ChooseLinkedExile { object });
                }
                finish(&mut g);
                assert_eq!(
                    plains
                        .iter()
                        .filter(|&&card| g.object_view(card).zone == Zone::Hand)
                        .count(),
                    (i + 1).min(n)
                );
            }
        }
    }
}
#[test]
fn hob_roads_delayed_attack_survives_sacrifice_and_counts_plains_on_resolution() {
    for (p, remove_source) in [
        (Player::P0, false),
        (Player::P1, false),
        (Player::P0, true),
        (Player::P1, true),
    ] {
        let mut g = scenario(p);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let attacker = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let target = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let plains = [
            put(&mut g, p, "Plains", Zone::Battlefield),
            put(&mut g, p, "Hallowed Fountain", Zone::Battlefield),
        ];
        let id = cast_saga(&mut g, p);
        search(&mut g);
        act(&mut g, Action::FinishSearch);
        finish(&mut g);
        for _ in 0..2 {
            step(&mut g);
            finish(&mut g);
        }
        step(&mut g);
        if remove_source {
            until(&mut g, |g| {
                g.state
                    .stack
                    .iter()
                    .any(|item| matches!(item.kind, StackKind::TriggeredAbility { ability: 3 }))
            });
            g.move_object(id, Zone::Exile);
        }
        finish(&mut g);
        assert_eq!(
            g.object_view(id).zone,
            if remove_source {
                Zone::Exile
            } else {
                Zone::Graveyard
            }
        );
        assert_eq!(g.observe(p).delayed_attack_this_turn.len(), 1);
        g.state.active = p;
        g.state.priority = p;
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: attacker });
        act(&mut g, Action::FinishAttackers);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::Targets { .. })
        });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        contracts::branch_contract(g.clone());
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        g.move_object(plains[0], Zone::Graveyard);
        finish(&mut g);
        assert_eq!(stats(&g, target), (3, 3));
        g.state.objects[attacker as usize].tapped = false;
        g.state.pending = Pending::Attackers;
        g.state.step = Step::Attackers;
        g.state.priority = p;
        act(&mut g, Action::Attack { card: attacker });
        act(&mut g, Action::FinishAttackers);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::Targets { .. })
        });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        finish(&mut g);
        assert_eq!(stats(&g, target), (4, 4));
        g.finish_cleanup();
        assert!(g.observe(p).delayed_attack_this_turn.is_empty());
    }
}

fn lore(g: &mut Game, id: CardId) {
    g.effect_counters(
        vec![(g.object_ref(id), Counters::of(CounterKind::Lore, 1))],
        ResumeMode::Pending,
    );
    g.prepare_triggers();
}

#[test]
fn hob_roads_two_sagas_and_reentry_keep_exact_links() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let a = put(&mut g, p, "Plains", Zone::Library);
        let stale = put(&mut g, p, "Plains", Zone::Library);
        let b = put(&mut g, p, "Plains", Zone::Library);
        let first = cast_saga(&mut g, p);
        search(&mut g);
        act(&mut g, Action::ChooseSearch { card: a });
        act(&mut g, Action::ChooseSearch { card: stale });
        act(&mut g, Action::FinishSearch);
        finish(&mut g);
        let old_source = g.object_ref(first);
        let second = cast_saga(&mut g, p);
        search(&mut g);
        act(&mut g, Action::ChooseSearch { card: b });
        act(&mut g, Action::FinishSearch);
        finish(&mut g);
        let b_ref = g.object_ref(b);
        let stale_ref = g.object_ref(stale);
        g.move_object(stale, Zone::Hand);
        g.move_object(stale, Zone::Exile);
        assert_ne!(stale_ref, g.object_ref(stale));
        lore(&mut g, first);
        linked(&mut g);
        assert_eq!(g.actor(), Some(p));
        reject(&mut g, Action::ChooseLinkedExile { object: b_ref });
        reject(&mut g, Action::ChooseLinkedExile { object: stale_ref });
        let stale_new = g.object_ref(stale);
        reject(&mut g, Action::ChooseLinkedExile { object: stale_new });
        contracts::branch_contract(g.clone());
        let a_ref = g.object_ref(a);
        act(&mut g, Action::ChooseLinkedExile { object: a_ref });
        finish(&mut g);
        assert_eq!(g.object_view(a).zone, Zone::Hand);
        assert_eq!(g.object_view(a).owner, p);
        g.move_object(first, Zone::Exile);
        let movement = g.entry_move(first, Zone::Battlefield, false);
        g.stage_entries(vec![movement], None);
        g.prepare_triggers();
        search(&mut g);
        act(&mut g, Action::FinishSearch);
        finish(&mut g);
        assert_ne!(old_source, g.object_ref(first));
        lore(&mut g, first);
        finish(&mut g);
        assert_eq!(g.object_view(stale).zone, Zone::Exile);
        lore(&mut g, second);
        linked(&mut g);
        let stale_new = g.object_ref(stale);
        reject(&mut g, Action::ChooseLinkedExile { object: stale_new });
        act(&mut g, Action::ChooseLinkedExile { object: b_ref });
        finish(&mut g);
        assert_eq!(g.object_view(b).zone, Zone::Hand);
        assert_eq!(g.object_view(b).owner, p);
    }
}

fn declare(g: &mut Game, p: Player, attacker: CardId) {
    g.state.objects[attacker as usize].tapped = false;
    g.state.active = p;
    g.state.priority = p;
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    act(g, Action::Attack { card: attacker });
    act(g, Action::FinishAttackers);
}

#[test]
fn hob_roads_delayed_controller_and_invalidated_targets_both_seats() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let attacker = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let opposing_attacker = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let stolen_target = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let blink_target = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let good_target = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        put(&mut g, p, "Plains", Zone::Battlefield);
        let saga = cast_saga(&mut g, p);
        search(&mut g);
        act(&mut g, Action::FinishSearch);
        finish(&mut g);
        for _ in 0..2 {
            step(&mut g);
            finish(&mut g);
        }
        step(&mut g);
        until(&mut g, |g| {
            g.state
                .stack
                .iter()
                .any(|item| matches!(item.kind, StackKind::TriggeredAbility { ability: 3 }))
        });
        g.record_control(g.object_ref(saga), p.other(), None);
        finish(&mut g);
        assert_eq!(g.observe(p).delayed_attack_this_turn[0].controller, p);
        declare(&mut g, p.other(), opposing_attacker);
        finish(&mut g);
        assert!(g.state.stack.is_empty());
        for target in [stolen_target, blink_target, good_target] {
            declare(&mut g, p, attacker);
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::Targets { .. })
            });
            assert_eq!(g.actor(), Some(p));
            contracts::branch_contract(g.clone());
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
            if target == stolen_target {
                g.record_control(g.object_ref(target), p.other(), None);
            }
            if target == blink_target {
                let before = g.object_ref(target);
                g.move_object(target, Zone::Exile);
                let movement = g.entry_move(target, Zone::Battlefield, false);
                g.stage_entries(vec![movement], None);
                assert_ne!(before, g.object_ref(target));
            }
            finish(&mut g);
        }
        assert_eq!(stats(&g, stolen_target), (2, 2));
        assert_eq!(stats(&g, blink_target), (2, 2));
        assert_eq!(stats(&g, good_target), (3, 3));
    }
}

#[test]
fn hob_roads_old_incarnation_link_survives_reentry_and_queued_chapter() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let old_card = put(&mut g, p, "Plains", Zone::Library);
        let new_card = put(&mut g, p, "Plains", Zone::Library);
        let saga = cast_saga(&mut g, p);
        search(&mut g);
        act(&mut g, Action::ChooseSearch { card: old_card });
        act(&mut g, Action::FinishSearch);
        finish(&mut g);
        let old_source = g.object_ref(saga);
        let old_link = g.object_ref(old_card);
        lore(&mut g, saga);
        until(&mut g, |g| {
            g.state.pending == Pending::Priority
                && g.state.stack.iter().any(|item| {
                    item.object == old_source
                        && matches!(item.kind, StackKind::TriggeredAbility { ability: 1 })
                })
        });
        g.move_object(saga, Zone::Exile);
        let movement = g.entry_move(saga, Zone::Battlefield, false);
        g.stage_entries(vec![movement], None);
        g.prepare_triggers();
        search(&mut g);
        act(&mut g, Action::ChooseSearch { card: new_card });
        act(&mut g, Action::FinishSearch);
        until(&mut g, |g| {
            g.state.pending == Pending::Priority
                && g.state.resolving.is_none()
                && g.state.pending_triggers.is_empty()
                && g.state.stack.len() == 1
                && g.state.stack[0].object == old_source
        });
        assert_ne!(old_source, g.object_ref(saga));
        assert_eq!(g.object_view(old_card).zone, Zone::Exile);
        assert_eq!(g.object_ref(old_card), old_link);
        let new_link = g.object_ref(new_card);
        lore(&mut g, saga);
        linked(&mut g);
        assert_eq!(g.actor(), Some(p));
        reject(&mut g, Action::ChooseLinkedExile { object: old_link });
        contracts::branch_contract(g.clone());
        act(&mut g, Action::ChooseLinkedExile { object: new_link });
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::LinkedExile { .. })
        });
        assert_eq!(g.actor(), Some(p));
        assert_eq!(g.state.resolving.as_ref().unwrap().item.object, old_source);
        reject(&mut g, Action::ChooseLinkedExile { object: new_link });
        contracts::branch_contract(g.clone());
        act(&mut g, Action::ChooseLinkedExile { object: old_link });
        finish(&mut g);
        assert_eq!(g.object_view(old_card).zone, Zone::Hand);
        assert_eq!(g.object_view(new_card).zone, Zone::Hand);
        assert_eq!(g.object_view(old_card).owner, p);
        assert_eq!(g.object_view(new_card).owner, p);
    }
}
