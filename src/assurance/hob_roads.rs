//! Search-linked Saga chapters and player-duration attack triggers.
use super::*;
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
