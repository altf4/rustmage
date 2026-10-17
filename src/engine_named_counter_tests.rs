//! Named counters use the ordinary replacement/payment paths and owned state.
use super::ability_timeline_tests::{checked, named, settle, setup};
use super::*;

const NAMED: [CounterKind; 9] = [
    CounterKind::Incubation,
    CounterKind::Bait,
    CounterKind::Page,
    CounterKind::Divinity,
    CounterKind::Revival,
    CounterKind::Soul,
    CounterKind::Fellowship,
    CounterKind::Wish,
    CounterKind::Stash,
];

#[test]
fn named_counters_storage_is_canonical_owned_and_sparse() {
    assert_eq!(std::mem::size_of::<crate::cards::InitialCounters>(), 40);
    assert_eq!(std::mem::size_of::<Counters>(), 48);
    assert_eq!(std::mem::size_of::<Object>(), 144);
    assert!(!std::mem::needs_drop::<Object>());
    let empty = Counters::default();
    assert_eq!(empty.owned_bytes(), 0);
    assert!(empty.is_empty());
    assert!(!empty.has_any());
    let single = Counters::of(CounterKind::Soul, 1);
    assert_eq!(single.allocated_bytes(), 40);
    assert!(single.has_any());
    assert!(!single.is_empty());
    assert_eq!(single.owned_bytes(), 40);
    assert!(serde_json::to_value(&empty).unwrap().get("named").is_none());
    let mut all = empty.clone();
    for kind in NAMED.into_iter().rev() {
        all.add(kind, 2);
    }
    assert_eq!(all.total(), 18);
    assert_eq!(all.owned_bytes(), 24 + 9 * 16);
    let frozen = all.clone();
    for kind in NAMED {
        all.remove(kind, 2);
    }
    assert_eq!(all, empty);
    assert_eq!(all.owned_bytes(), 0);
    assert_eq!(frozen.total(), 18);
    let baseline = Game::new(Config {
        decks: [vec![Card::Forest; 60], vec![Card::Forest; 60]],
        seed: 249,
        starting_player: Player::P0,
        deck_rules: DeckRules::Standard,
    })
    .unwrap();
    println!(
        "NAMED_COUNTER_MEMORY counters={} initial_counters={} object={} state={} initial_logical_bytes={} nine_kinds_logical_heap={} nine_kinds_allocated_heap={} empty_allocated_heap={}",
        std::mem::size_of::<Counters>(),
        std::mem::size_of::<crate::cards::InitialCounters>(),
        std::mem::size_of::<Object>(),
        std::mem::size_of::<State>(),
        baseline.state_bytes(),
        frozen.owned_bytes(),
        frozen.allocated_bytes(),
        empty.allocated_bytes()
    );
    let wire = serde_json::to_value(&frozen).unwrap();
    assert_eq!(
        serde_json::from_value::<Counters>(wire.clone()).unwrap(),
        frozen
    );
    let mut reversed = wire.clone();
    reversed["named"].as_array_mut().unwrap().reverse();
    assert_eq!(
        serde_json::from_value::<Counters>(reversed).unwrap(),
        frozen
    );
    for invalid in [
        serde_json::json!([]),
        serde_json::json!([["Soul", 0]]),
        serde_json::json!([["Soul", 1], ["Soul", 2]]),
        serde_json::json!([["PlusOne", 1]]),
    ] {
        let mut forged = wire.clone();
        forged["named"] = invalid;
        assert!(serde_json::from_value::<Counters>(forged).is_err());
    }
}

#[test]
fn named_counters_replacement_choices_freeze_all_kinds_and_public_observations() {
    for player in [Player::P0, Player::P1] {
        let mut game = setup(player);
        named(&mut game, player, "Doubling Season", Zone::Battlefield);
        named(&mut game, player, "Doubling Season", Zone::Battlefield);
        let bear = named(&mut game, player, "Bear Cub", Zone::Battlefield);
        let enemy = named(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
        let mut counters = Counters::default();
        for kind in NAMED {
            counters.add(kind, 1);
        }
        let empty_bytes = game.state_bytes();
        game.effect_counters(
            vec![
                (game.object_ref(bear), counters.clone()),
                (game.object_ref(enemy), counters),
            ],
            replacements::ResumeMode::Pending,
        );
        assert!(game.advance_replacements(false));
        assert!(game.state_bytes() > empty_bytes);
        while game.state.replacements.is_some() {
            let action = game
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::ChooseReplacement { .. }))
                .unwrap();
            checked(&mut game, action);
        }
        for viewer in [player, player.other()] {
            let observation = game.observe(viewer);
            for (id, amount) in [(bear, 4), (enemy, 1)] {
                let view = observation.objects.iter().find(|o| o.id == id).unwrap();
                for kind in NAMED {
                    assert_eq!(view.counters.get(kind), amount);
                }
            }
        }
        assert!(game.is_modified(bear));
        assert!(game.damage_facts(bear).counter);
        assert!(game.damage_facts(bear).modified);
        assert_eq!(game.object_view(bear).power, 2);
        assert_eq!(game.state_bytes() - empty_bytes, 24 + 2 * (40 + 9 * 16));
    }
}

#[test]
fn named_counter_removal_costs_are_complete_atomic_and_survive_owned_branches() {
    for player in [Player::P0, Player::P1] {
        let mut game = setup(player);
        let elm = named(&mut game, player, "Gnarlbark Elm", Zone::Battlefield);
        game.set_counters(elm, Counters::of(CounterKind::Incubation, 2));
        game.add_counter(elm, CounterKind::Soul, 2);
        let choices: Vec<_> = game
            .legal_actions()
            .into_iter()
            .filter(|a| {
                matches!(a,
            Action::BeginActivate { card, .. } if *card == elm)
            })
            .collect();
        let mut allocations: Vec<_> = choices
            .iter()
            .map(|a| {
                let Action::BeginActivate {
                    counter_payment, ..
                } = a
                else {
                    unreachable!()
                };
                (
                    counter_payment.get(CounterKind::Incubation),
                    counter_payment.get(CounterKind::Soul),
                )
            })
            .collect();
        allocations.sort();
        allocations.dedup();
        assert_eq!(allocations, vec![(0, 2), (1, 1), (2, 0)]);
        let action = choices.into_iter().find(|a| matches!(a,
            Action::BeginActivate { counter_payment, .. } if counter_payment.get(CounterKind::Soul) == 1)).unwrap();
        let mut forged = action.clone();
        let Action::BeginActivate {
            counter_payment, ..
        } = &mut forged
        else {
            unreachable!()
        };
        *counter_payment = Counters::of(CounterKind::Page, 2);
        let saved = game.snapshot();
        let revision = game.revision();
        assert_eq!(game.apply(player, forged), Err(Error::IllegalAction));
        assert_eq!(game.snapshot(), saved);
        assert_eq!(game.revision(), revision);
        checked(&mut game, action);
        checked(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(elm),
            },
        );
        assert_eq!(game.object_view(elm).counters.get(CounterKind::Soul), 1);
        assert_eq!(
            game.object_view(elm).counters.get(CounterKind::Incubation),
            1
        );
        settle(&mut game);
        let frozen = game.object_view(elm);
        let saved = game.snapshot();
        let mut child = {
            let parent = game.clone();
            parent.clone()
        };
        drop(game);
        child.move_object(elm, Zone::Hand);
        assert_eq!(child.object_view(elm).counters, Counters::default());
        assert_eq!(frozen.counters.total(), 2);
        child.move_object(elm, Zone::Battlefield);
        assert_eq!(child.object_view(elm).counters.get(CounterKind::Soul), 0);
        child.restore(&saved).unwrap();
        assert_eq!(child.object_view(elm).counters, frozen.counters);
        assert_eq!(child.snapshot(), saved);
    }
}

#[test]
fn named_counters_frozen_lki_and_token_slot_reuse_do_not_alias() {
    for player in [Player::P0, Player::P1] {
        let mut game = setup(player);
        let token = Card::catalogue()
            .find(|c| c.token_info().is_some_and(|t| t.name == "Goblin Token"))
            .unwrap();
        game.create_tokens(player, token, 1, false);
        let id = game
            .state
            .objects
            .iter()
            .position(|o| o.card == token && o.zone == Zone::Battlefield)
            .unwrap() as CardId;
        game.set_counters(id, Counters::of(CounterKind::Revival, 8));
        let frozen = game.object_view(id);
        let saved = game.snapshot();
        let mut child = {
            let parent = game.clone();
            parent.clone()
        };
        drop(game);
        child.move_object(id, Zone::Graveyard);
        child.state_based_actions();
        assert_eq!(child.object_view(id).zone, Zone::Outside);
        assert_eq!(child.object_view(id).counters, Counters::default());
        child.create_tokens(player, token, 1, false);
        assert_eq!(child.object_view(id).zone, Zone::Battlefield);
        assert_ne!(child.object_ref(id), frozen.reference);
        assert_eq!(child.object_view(id).counters.total(), 0);
        assert_eq!(frozen.counters.get(CounterKind::Revival), 8);
        child.restore(&saved).unwrap();
        assert_eq!(child.object_view(id).counters, frozen.counters);
    }
}

#[test]
fn named_counter_doubling_and_capacity_failure_use_paid_rules_transactions() {
    for player in [Player::P0, Player::P1] {
        for (amount, hot) in [(3, false), (3, true), (u64::MAX, true)] {
            let mut game = setup(player);
            let source = named(
                &mut game,
                player,
                "Zimone, Paradox Sculptor",
                Zone::Battlefield,
            );
            let target = named(&mut game, player, "Mox Jasper", Zone::Battlefield);
            game.set_counters(target, Counters::of(CounterKind::Soul, amount));
            if hot {
                game.state.objects[target as usize].counters.plus_one = 1;
            }
            checked(
                &mut game,
                Action::AnnounceActivate {
                    card: source,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            checked(
                &mut game,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
            checked(&mut game, Action::FinishTargets);
            let payment = game
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PayCosts { .. }))
                .unwrap();
            checked(&mut game, payment);
            checked(&mut game, Action::Pass);
            if amount == 3 {
                let objects = game.state.objects.clone();
                let names = game.state.named_counters.clone();
                checked(&mut game, Action::Pass);
                if !hot {
                    assert!(game.state.objects.ptr_eq(&objects));
                    assert!(!live_counters::same_storage(
                        &game.state.named_counters,
                        &names
                    ));
                }
                assert_eq!(game.object_view(target).counters.get(CounterKind::Soul), 6);
                assert_eq!(
                    game.object_view(target).counters.plus_one,
                    if hot { 2 } else { 0 }
                );
            } else {
                let saved = game.snapshot();
                let revision = game.revision();
                let actor = game.actor().unwrap();
                let index = game
                    .legal_actions()
                    .iter()
                    .position(|a| *a == Action::Pass)
                    .unwrap();
                let mut child = {
                    let parent = game.clone();
                    parent.clone()
                };
                let error = Err(Error::CapacityExceeded(crate::CapacityExceeded {
                    resource: "counter representation exhausted",
                }));
                assert_eq!(game.apply(actor, Action::Pass), error);
                assert_eq!(child.apply_index(actor, revision, index), error);
                assert_eq!(game.snapshot(), saved);
                assert_eq!(child.snapshot(), saved);
                assert_eq!(game.revision(), revision);
                for viewer in [player, player.other()] {
                    assert_eq!(
                        serde_json::to_value(game.observe(viewer)).unwrap(),
                        serde_json::to_value(child.observe(viewer)).unwrap()
                    );
                }
                assert_eq!(
                    game.object_view(target).counters.get(CounterKind::Soul),
                    amount
                );
                assert_eq!(game.object_view(target).counters.plus_one, 1);
            }
        }
    }
}

#[test]
fn named_counter_state_based_pair_cancellation_mutates_live_storage_only() {
    for player in [Player::P0, Player::P1] {
        let mut game = setup(player);
        let creature = named(&mut game, player, "Bear Cub", Zone::Battlefield);
        game.set_counters(
            creature,
            Counters::of(CounterKind::PlusOne, 3)
                .with(CounterKind::MinusOne, 2)
                .with(CounterKind::Soul, 4)
                .with(CounterKind::Incubation, 2),
        );
        let frozen = game.object_view(creature);
        checked(&mut game, Action::Pass);
        let after = game.object_view(creature);
        assert_eq!(after.counters.plus_one, 1);
        assert_eq!(after.counters.minus_one, 0);
        assert_eq!(after.counters.get(CounterKind::Soul), 4);
        assert_eq!(after.counters.get(CounterKind::Incubation), 2);
        assert_eq!(after.counters.total(), 7);
        assert_eq!(frozen.counters.plus_one, 3);
        assert_eq!(frozen.counters.minus_one, 2);
        assert_eq!(frozen.counters.get(CounterKind::Soul), 4);
        assert_eq!(after.zone, Zone::Battlefield);
    }
}

#[test]
fn named_counter_rows_follow_all_zone_incarnations_and_detach_owned_branches() {
    for player in [Player::P0, Player::P1] {
        let mut game = setup(player);
        let id = named(&mut game, player, "Mox Jasper", Zone::Exile);
        game.set_counters(id, Counters::of(CounterKind::Stash, 3));
        let saved = game.snapshot();
        let reference = game.object_ref(id);
        let mut child = {
            let parent = game.clone();
            parent.clone()
        };
        assert!(game.state.objects.ptr_eq(&child.state.objects));
        child.add_counter(id, CounterKind::Stash, 2);
        assert!(game.state.objects.ptr_eq(&child.state.objects));
        assert!(!live_counters::same_storage(
            &game.state.named_counters,
            &child.state.named_counters
        ));
        assert_eq!(game.counter_count(id, CounterKind::Stash), 3);
        assert_eq!(child.counter_count(id, CounterKind::Stash), 5);
        assert!(!child.is_modified(id));
        child.move_object(id, Zone::Graveyard);
        assert_ne!(child.object_ref(id), reference);
        assert!(child.state.named_counters.is_none());
        assert_eq!(child.counter_count(id, CounterKind::Stash), 0);
        assert_eq!(game.counter_count(id, CounterKind::Stash), 3);
        child.restore(&saved).unwrap();
        assert_eq!(child.counter_count(id, CounterKind::Stash), 3);
        assert_eq!(child.object_ref(id), reference);
        child.set_counters(id, Counters::default());
        assert!(child.state.named_counters.is_none());
        assert_eq!(game.counter_count(id, CounterKind::Stash), 3);
    }
}

#[test]
fn every_named_kind_pays_actual_remove_any_costs() {
    for player in [Player::P0, Player::P1] {
        for kind in NAMED {
            let mut game = setup(player);
            let source = named(&mut game, player, "Gnarlbark Elm", Zone::Battlefield);
            game.set_counters(source, Counters::of(kind, 2));
            let action = game.legal_actions().into_iter().find(|action| matches!(action,
                Action::BeginActivate { card, counter_payment, .. } if *card == source && counter_payment.get(kind) == 2)).unwrap();
            checked(&mut game, action);
            checked(
                &mut game,
                Action::ChooseTarget {
                    target: Target::Card(source),
                },
            );
            assert_eq!(game.counter_count(source, kind), 0);
            assert!(game.state.named_counters.is_none());
        }
    }
}
