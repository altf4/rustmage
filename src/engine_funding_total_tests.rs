use super::super::tests::{scenario, spawn};
use super::*;
fn goal(generic: u32) -> FundingGoal {
    FundingGoal {
        actor: Player::P0,
        required: [0; 6],
        generic,
        reserve: None,
        source: None,
    }
}
#[test]
fn neutral_filter_impossible_total_prunes_before_search() {
    let mut g = scenario();
    for _ in 0..4 {
        for card in [Card::Plains, Card::Island, Card::Swamp, Card::Mountain] {
            spawn(&mut g, Player::P0, card, Zone::Battlefield);
        }
    }
    spawn(
        &mut g,
        Player::P0,
        Card::from_name("Three Tree Mascot").unwrap(),
        Zone::Battlefield,
    );
    let target = goal(17);
    let state = FundingState::from_game(&g, target);
    let mut visited = HashSet::new();
    let start = std::time::Instant::now();
    assert!(!state.reachable(target, &mut visited));
    eprintln!(
        "neutral-filter impossible total: {} visited states in {:?}",
        visited.len(),
        start.elapsed()
    );
    assert!(
        visited.is_empty(),
        "an impossible total should be rejected before enumerating filter allocations"
    );
}

#[test]
fn net_gain_handles_choices_alternative_costs_and_finite_or_repeatable_production() {
    let mut a = Card::from_name("Three Tree Mascot").unwrap().activations()[0];
    a.once_per_turn = false;
    assert_eq!(activation_gain_upper(&a, Counters::default(), 20), 0); // Neutral even if repeatable.
    a.program = &[Instruction::AddMana(ManaProduction::Choice {
        mask: 31,
        amount: 3,
    })];
    assert_eq!(
        activation_gain_upper(&a, Counters::default(), 20),
        u128::MAX
    );
    a.cost.life = 2;
    assert_eq!(activation_gain_upper(&a, Counters::default(), 5), 4); // Two uses; three colors are not fifteen mana.
    a.cost.remove = Some((Some(CounterKind::PlusOne), 2));
    let counters = Counters {
        plus_one: 3,
        ..Counters::default()
    };
    assert_eq!(activation_gain_upper(&a, counters, 5), 2);
    a.cost.life = 0;
    a.cost.remove = None;
    a.once_per_turn = true;
    a.cost.generic = 3;
    assert_eq!(activation_gain_upper(&a, counters, 20), 0);
    a.cost.alternatives = &[ManaCost {
        colored: [0; 6],
        generic: 1,
    }];
    assert_eq!(activation_gain_upper(&a, counters, 20), 2); // Cheapest equivalent, not primary cost.
    a.once_per_turn = false;
    a.cost.tap = true;
    assert_eq!(activation_gain_upper(&a, counters, 20), 2);
    a.cost.tap = false;
    a.cost.sacrifice = true;
    assert_eq!(activation_gain_upper(&a, counters, 20), 2);
}

#[test]
fn neutral_conversion_retains_exact_colors_and_convoke_capacity() {
    let mut g = scenario();
    for card in [Card::Plains, Card::Plains, Card::Forest, Card::Forest] {
        spawn(&mut g, Player::P0, card, Zone::Battlefield);
    }
    spawn(
        &mut g,
        Player::P0,
        Card::from_name("Three Tree Mascot").unwrap(),
        Zone::Battlefield,
    );
    let target = FundingGoal {
        required: [0, 0, 1, 0, 0, 0],
        ..goal(3)
    };
    assert!(g.can_fund(target)); // Conversion is necessary, total remains four.
    assert!(!g.can_fund(FundingGoal {
        required: [0, 0, 2, 0, 0, 0],
        ..goal(2)
    })); // Only one conversion.
    assert!(g.can_fund_convoke(goal(5))); // Summoning sickness does not block convoke.
    assert!(!g.can_fund_convoke(goal(6)));
}

#[test]
fn positive_tap_production_and_enclosing_source_reservation_remain_exact() {
    let mut g = scenario();
    let card = Card::from_name("Hedron Archive").unwrap();
    let id = spawn(&mut g, Player::P0, card, Zone::Battlefield);
    assert!(g.can_fund(goal(2)));
    assert!(!g.can_fund(goal(3)));
    let reserved = FundingGoal {
        source: Some((g.object_ref(id), 1, Counters::default(), card)),
        ..goal(2)
    };
    assert!(!g.can_fund(reserved)); // Archive must remain untapped for its own tap/sac cost.
    for _ in 0..2 {
        spawn(&mut g, Player::P0, Card::Forest, Zone::Battlefield);
    }
    assert!(g.can_fund(reserved));
    assert!(g.can_fund(goal(4)));
    assert!(!g.can_fund(goal(5)));
}

// Independent small-state oracle: no optimistic bound, including the old color
// bound. All fixtures have finite uses, so exhaustive enumeration terminates.
fn exhaustive(
    state: &FundingState<'_>,
    target: FundingGoal,
    seen: &mut HashSet<FundingKey>,
) -> bool {
    if !state.obligations(target) {
        return false;
    }
    let paid = if state.convoke {
        convoke::payable(
            state.mana,
            target.required,
            target.generic,
            &state.convoke_masks(),
        )
    } else {
        payable(state.mana, target.required, target.generic)
    };
    if paid {
        return true;
    }
    if !seen.insert(state.key(target)) {
        return false;
    }
    state.moves(target).into_iter().any(|step| {
        let mut next = state.clone();
        next.apply(&step);
        exhaustive(&next, target, seen)
    })
}
#[test]
fn aggregate_bound_preserves_exhaustive_small_funding_domains() {
    for pool in 0..=1 {
        for lands in 0..=2 {
            for archive in [false, true] {
                for filter in [false, true] {
                    for convoke in [false, true] {
                        let mut g = scenario();
                        g.state.players[0].mana[4] = pool;
                        for _ in 0..lands {
                            spawn(&mut g, Player::P0, Card::Forest, Zone::Battlefield);
                        }
                        if archive {
                            spawn(
                                &mut g,
                                Player::P0,
                                Card::from_name("Hedron Archive").unwrap(),
                                Zone::Battlefield,
                            );
                        }
                        if filter {
                            spawn(
                                &mut g,
                                Player::P0,
                                Card::from_name("Three Tree Mascot").unwrap(),
                                Zone::Battlefield,
                            );
                        }
                        for generic in 0..=4 {
                            for black in 0..=1 {
                                let target = FundingGoal {
                                    required: [0, 0, black, 0, 0, 0],
                                    ..goal(generic)
                                };
                                let state =
                                    FundingState::from_game_with_convoke(&g, target, convoke);
                                assert_eq!(
                                    state.reachable(target, &mut HashSet::new()),
                                    exhaustive(&state, target, &mut HashSet::new()),
                                    "pool{pool}lands{lands}archive{archive}filter{filter}convoke{convoke}generic{generic}black{black}"
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn counterfactual_total_saturates_instead_of_overflowing() {
    let g = scenario();
    let mut state = FundingState::from_game(&g, goal(u32::MAX));
    state.mana = [u128::MAX; 6];
    assert_eq!(state.total_mana_upper(), u128::MAX);
    assert!(state.optimistic(goal(u32::MAX)));
}

#[test]
fn foreign_owned_sources_fund_only_their_controller_without_mutation() {
    for actor in [Player::P0, Player::P1] {
        let mut g = scenario();
        let forest = spawn(&mut g, actor.other(), Card::Forest, Zone::Battlefield);
        let archive = spawn(
            &mut g,
            actor.other(),
            Card::from_name("Hedron Archive").unwrap(),
            Zone::Battlefield,
        );
        g.state.objects[forest as usize].controller = actor;
        g.state.objects[archive as usize].controller = actor;
        let before = g.snapshot();
        let target = FundingGoal { actor, ..goal(3) };
        assert!(g.can_fund(target));
        assert!(!g.can_fund(FundingGoal {
            actor: actor.other(),
            ..goal(1)
        }));
        assert_eq!(g.snapshot(), before);
        assert_eq!(g.state.objects[forest as usize].owner, actor.other());
        assert_eq!(g.state.objects[archive as usize].owner, actor.other());
        let creature = spawn(
            &mut g,
            actor.other(),
            Card::from_name("Bear Cub").unwrap(),
            Zone::Battlefield,
        );
        g.state.objects[creature as usize].controller = actor;
        assert!(g.can_fund_convoke(FundingGoal { actor, ..goal(4) }));
        assert!(!g.can_fund_convoke(FundingGoal {
            actor: actor.other(),
            ..goal(1)
        }));
    }
}
