//! Loyalty costs must complete before the activation is committed.
use super::*;

#[test]
fn loyalty_activation_waits_for_counter_replacement_choice() {
    for player in [Player::P0, Player::P1] {
        for direct in [false, true] {
            let mut g = scenario(player);
            let walker = put(
                &mut g,
                player,
                "Liliana, Dreadhorde General",
                Zone::Battlefield,
            );
            g.add_counter(walker, CounterKind::Loyalty, 6);
            put(
                &mut g,
                player,
                "Doc Samson, Super Psychiatrist",
                Zone::Battlefield,
            );
            put(&mut g, player, "Doubling Season", Zone::Battlefield);
            put(&mut g, player, "Doubling Season", Zone::Battlefield);
            let observer = put(&mut g, player, "Way of the Paradox", Zone::Battlefield);
            if direct {
                let action = g
                    .legal_actions()
                    .into_iter()
                    .find(|a| {
                        matches!(a,
                Action::Activate { card, ability: 0, .. } if *card == walker)
                    })
                    .unwrap();
                act(&mut g, action);
            } else {
                act(
                    &mut g,
                    Action::AnnounceActivate {
                        card: walker,
                        ability: 0,
                        counter_payment: Counters::default(),
                    },
                );
                pay(&mut g);
            }
            assert!(matches!(g.state.pending, Pending::Replacement { .. }));
            if direct {
                assert_eq!(
                    g.state
                        .stack
                        .iter()
                        .filter(|item| item.source.is_some_and(|s| s.id == walker))
                        .count(),
                    1
                );
                for viewer in [player, player.other()] {
                    assert_eq!(
                        g.observe(viewer)
                            .stack
                            .iter()
                            .filter(|item| item.source.is_some_and(|s| s.id == walker))
                            .count(),
                        1
                    );
                }
            }
            crate::engine::assurance::contracts::branch_contract(g.clone());
            assert_eq!(g.counter_count(walker, CounterKind::Loyalty), 6);
            assert!(!g.object_view(walker).loyalty_activated_this_turn);
            assert!(!g.state.players[player.index()].loyalty_ability_activated_this_turn);
            assert!(
                !g.state
                    .pending_triggers
                    .iter()
                    .any(|item| item.source.is_some_and(|s| s.id == observer))
            );
            let checkpoint = g.snapshot();
            while g
                .legal_actions()
                .iter()
                .any(|a| matches!(a, Action::ChooseReplacement { .. }))
            {
                choose(&mut g, |a| matches!(a, Action::ChooseReplacement { .. }));
            }
            assert_eq!(g.counter_count(walker, CounterKind::Loyalty), 14);
            assert!(g.object_view(walker).loyalty_activated_this_turn);
            assert!(g.state.players[player.index()].loyalty_ability_activated_this_turn);
            let complete = g.snapshot();
            g.restore(&checkpoint).unwrap();
            while g
                .legal_actions()
                .iter()
                .any(|a| matches!(a, Action::ChooseReplacement { .. }))
            {
                choose(&mut g, |a| matches!(a, Action::ChooseReplacement { .. }));
            }
            assert_eq!(g.snapshot(), complete);
        }
    }
}

#[test]
fn creature_activation_observer_waits_for_counter_cost_replacements() {
    for player in [Player::P0, Player::P1] {
        for direct in [false, true] {
            let mut g = scenario(player);
            let source = put(
                &mut g,
                player,
                "Jessica Jones, Private Eye",
                Zone::Battlefield,
            );
            let observer = put(&mut g, player, "Elrond, Moon-Reader", Zone::Battlefield);
            put(
                &mut g,
                player,
                "Doc Samson, Super Psychiatrist",
                Zone::Battlefield,
            );
            put(&mut g, player, "Loading Zone", Zone::Battlefield);
            if direct {
                let action = g
                    .legal_actions()
                    .into_iter()
                    .find(|a| {
                        matches!(a,
                    Action::Activate { card, ability: 0, .. } if *card == source)
                    })
                    .unwrap();
                act(&mut g, action);
            } else {
                act(
                    &mut g,
                    Action::AnnounceActivate {
                        card: source,
                        ability: 0,
                        counter_payment: Counters::default(),
                    },
                );
                pay(&mut g);
            }
            assert!(matches!(g.state.pending, Pending::Replacement { .. }));
            assert!(g.object_view(source).tapped);
            assert_eq!(g.counter_count(source, CounterKind::Stun), 0);
            assert!(
                !g.state
                    .pending_triggers
                    .iter()
                    .any(|item| item.source.is_some_and(|s| s.id == observer))
            );
            let checkpoint = g.snapshot();
            while g
                .legal_actions()
                .iter()
                .any(|a| matches!(a, Action::ChooseReplacement { .. }))
            {
                choose(&mut g, |a| matches!(a, Action::ChooseReplacement { .. }));
            }
            assert!(g.counter_count(source, CounterKind::Stun) > 0);
            assert!(
                g.state
                    .pending_triggers
                    .iter()
                    .chain(&g.state.stack)
                    .chain(&g.state.ordering_triggers)
                    .any(|item| item.source.is_some_and(|s| s.id == observer))
            );
            let complete = g.snapshot();
            g.restore(&checkpoint).unwrap();
            while g
                .legal_actions()
                .iter()
                .any(|a| matches!(a, Action::ChooseReplacement { .. }))
            {
                choose(&mut g, |a| matches!(a, Action::ChooseReplacement { .. }));
            }
            assert_eq!(g.snapshot(), complete);
        }
    }
}

#[test]
fn direct_sacrifice_preserves_stack_source_copy_lki() {
    let _fixture = crate::cards::issue_93_copy_cost_fixture::Guard::new();
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = put(&mut g, player, "Fanatical Firebrand", Zone::Battlefield);
        let reference = g.object_ref(source);
        let mut item = g.activation_item(player, source, 0);
        g.begin_direct_activation_on_stack(&mut item);
        // Synthetic timing: a cost-side effect changes the source's copy
        // recipe after announcement but before its sacrifice.
        let recipe = CopyRecipe::new(Card::from_name("Bear Cub").unwrap());
        g.set_copy_recipe(reference, recipe);
        assert!(
            g.pay_activation(&mut item, [0; 6], Counters::default())
                .is_none()
        );
        assert_eq!(
            g.state.stack.last().unwrap().source_copy_recipe.as_deref(),
            Some(&recipe)
        );
        g.finish_direct_activation_on_stack(item);
        assert_eq!(
            g.state.stack.last().unwrap().source_copy_recipe.as_deref(),
            Some(&recipe)
        );
    }
}
#[test]
fn direct_loyalty_target_queues_ward_and_declining_counters_the_ability() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let walker = put(&mut g, player, "Professor Dellian Fel", Zone::Battlefield);
        g.add_counter(walker, CounterKind::Loyalty, 5);
        let target = put(
            &mut g,
            player.other(),
            "Colorstorm Stallion",
            Zone::Battlefield,
        );
        act(
            &mut g,
            Action::Activate {
                card: walker,
                ability: 2,
                target: Some(Target::Card(target)),
                payment: [0; 6],
                counter_payment: Counters::default(),
            },
        );
        let ability = g
            .state
            .stack
            .iter()
            .find(|item| item.source.is_some_and(|s| s.id == walker))
            .unwrap()
            .id;
        let ward = g
            .state
            .pending_triggers
            .iter()
            .chain(&g.state.ordering_triggers)
            .chain(&g.state.stack)
            .find(|item| {
                item.source.is_some_and(|s| s.id == target) && item.related_stack.is_some()
            })
            .unwrap();
        assert_eq!(ward.related_stack.unwrap().id, ability);
        for _ in 0..32 {
            if matches!(g.state.pending, Pending::ResolutionMana { .. }) {
                break;
            }
            choose(&mut g, |action| {
                matches!(action, Action::Pass | Action::PutTrigger { .. })
            });
        }
        assert!(matches!(g.state.pending, Pending::ResolutionMana { .. }));
        act(&mut g, Action::DeclineResolutionMana);
        settle(&mut g);
        assert_eq!(g.object_view(target).zone, Zone::Battlefield);
    }
}
#[test]
fn targeted_loyalty_pending_choice_preserves_direct_and_staged_views() {
    for player in [Player::P0, Player::P1] {
        for direct in [false, true] {
            let mut g = scenario(player);
            let walker = put(
                &mut g,
                player,
                "Ajani, Caller of the Pride",
                Zone::Battlefield,
            );
            g.add_counter(walker, CounterKind::Loyalty, 4);
            put(
                &mut g,
                player,
                "Doc Samson, Super Psychiatrist",
                Zone::Battlefield,
            );
            put(&mut g, player, "Doubling Season", Zone::Battlefield);
            put(&mut g, player, "Doubling Season", Zone::Battlefield);
            let target = put(&mut g, player, "Bear Cub", Zone::Battlefield);
            if direct {
                act(
                    &mut g,
                    Action::Activate {
                        card: walker,
                        ability: 0,
                        target: Some(Target::Card(target)),
                        payment: [0; 6],
                        counter_payment: Counters::default(),
                    },
                );
            } else {
                act(
                    &mut g,
                    Action::AnnounceActivate {
                        card: walker,
                        ability: 0,
                        counter_payment: Counters::default(),
                    },
                );
                act(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(target),
                    },
                );
                if g.legal_actions().contains(&Action::FinishTargets) {
                    act(&mut g, Action::FinishTargets);
                }
                pay(&mut g);
            }
            assert!(matches!(g.state.pending, Pending::Replacement { .. }));
            for viewer in [player, player.other()] {
                let view = g.observe(viewer);
                if direct {
                    let ability = view
                        .stack
                        .iter()
                        .find(|item| item.source.is_some_and(|s| s.id == walker))
                        .unwrap();
                    assert_eq!(
                        ability
                            .targets
                            .iter()
                            .any(|bound| bound.target == Target::Card(target)),
                        viewer == player
                    );
                } else {
                    let ability = view.announcing.unwrap();
                    assert_eq!(
                        ability
                            .targets
                            .iter()
                            .any(|bound| bound.target == Target::Card(target)),
                        viewer == player
                    );
                }
            }
            crate::engine::assurance::contracts::branch_contract(g.clone());
            while g
                .legal_actions()
                .iter()
                .any(|action| matches!(action, Action::ChooseReplacement { .. }))
            {
                choose(&mut g, |action| {
                    matches!(action, Action::ChooseReplacement { .. })
                });
            }
            let public = g.observe(player.other());
            let ability = public
                .stack
                .iter()
                .find(|item| item.source.is_some_and(|s| s.id == walker))
                .unwrap();
            assert!(
                ability
                    .targets
                    .iter()
                    .any(|bound| bound.target == Target::Card(target))
            );
        }
    }
}
#[test]
fn uncommitted_target_allocations_are_private_until_cast() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        g.state.players[player.index()].mana = [0, 0, 0, 0, 1, 3];
        let a = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        let b = put(&mut g, player, "Grizzly Bears", Zone::Battlefield);
        let spell = put(
            &mut g,
            player,
            "Elusive Otter // Grove's Bounty",
            Zone::Hand,
        );
        act(&mut g, Action::AnnounceAdventure { card: spell });
        act(&mut g, Action::ChooseX { value: 3 });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(a),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(b),
            },
        );
        act(&mut g, Action::FinishTargets);
        assert!(g.observe(player).target_amount.is_some());
        assert!(g.observe(player.other()).target_amount.is_none());
        act(&mut g, Action::AllocateTargetAmount { amount: 1 });
        let own = g.observe(player);
        assert!(own.target_amount.is_some());
        assert!(own.announcing.unwrap().target_allocations.is_some());
        let other = g.observe(player.other());
        assert!(other.target_amount.is_none());
        let hidden = other.announcing.as_ref().unwrap();
        assert!(hidden.targets.is_empty());
        assert!(hidden.target_allocations.is_none());
        let wire = serde_json::to_value(&other).unwrap();
        assert!(wire["target_amount"].is_null());
        assert!(wire["announcing"]["target_allocations"].is_null());
        act(&mut g, Action::AllocateTargetAmount { amount: 2 });
        pay(&mut g);
        let public = g.observe(player.other());
        let item = public
            .stack
            .iter()
            .find(|item| item.source.is_some_and(|s| s.id == spell))
            .unwrap();
        assert_eq!(item.targets.len(), 2);
        assert_eq!(item.target_allocations.as_ref().unwrap().len(), 2);
    }
}
