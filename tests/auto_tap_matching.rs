//! Exercise the pure payment policy against independent exhaustive allocations.
use rustmage_core::cards::{Mana, ManaCost};
#[path = "../crates/engine/src/engine_auto_tap_solver.rs"]
mod auto_tap_solver;
use auto_tap_solver::{Mode, Pools, Solution, Source, SpendBonus, solve};

fn unit(color: usize, amount: u32) -> Mana {
    let mut mana = [0; 6];
    mana[color] = amount;
    mana
}

fn land(colors: &[usize]) -> Source {
    Source {
        modes: colors
            .iter()
            .map(|&c| Mode {
                mana: unit(c, 1),
                pain: false,
                sacrifice: false,
            })
            .collect(),
        creature: false,
        variable: false,
        treasure: false,
        followup_eligible: true,
    }
}

fn valid(cost: ManaCost, sources: &[Source], mut pool: Mana, plan: &Solution) {
    let mut used = vec![false; sources.len()];
    for &(source, mode) in &plan.taps {
        assert!(!used[source], "one permanent tapped twice");
        used[source] = true;
        for (n, delta) in pool.iter_mut().zip(sources[source].modes[mode].mana) {
            *n = n.checked_add(delta).unwrap();
        }
    }
    let mut generic = 0;
    for ((paid, available), required) in plan.payment.into_iter().zip(pool).zip(cost.colored) {
        assert!((required..=available).contains(&paid));
        generic += paid - required;
    }
    assert_eq!(generic, cost.generic);
}

fn ordinary(cost: ManaCost, sources: &[Source]) -> Option<Solution> {
    solve(
        cost,
        Pools::default(),
        sources,
        None,
        &[],
        SpendBonus::default(),
    )
}

#[test]
fn all_three_source_boards_match_exhaustive_manual_funding() {
    let types = [0, 2, 5]; // two colors and true colorless
    for first in 1..8 {
        for second in first..8 {
            for third in second..8 {
                let sources = [first, second, third].map(|mask| {
                    land(
                        &types
                            .into_iter()
                            .enumerate()
                            .filter(|(i, _)| mask & (1 << i) != 0)
                            .map(|(_, color)| color)
                            .collect::<Vec<_>>(),
                    )
                });
                let mut reachable = Vec::new();
                for a in &sources[0].modes {
                    for b in &sources[1].modes {
                        for c in &sources[2].modes {
                            let mana: Mana =
                                std::array::from_fn(|i| a.mana[i] + b.mana[i] + c.mana[i]);
                            reachable.push(mana);
                        }
                    }
                }
                for w in 0..=3 {
                    for b in 0..=3 - w {
                        for c in 0..=3 - w - b {
                            for generic in [0, 1] {
                                let cost = ManaCost {
                                    colored: [w, 0, b, 0, 0, c],
                                    generic,
                                };
                                let expected = w + b + c + generic <= 3
                                    && reachable
                                        .iter()
                                        .any(|p| (0..6).all(|i| p[i] >= cost.colored[i]));
                                let payment = ordinary(cost, &sources);
                                assert_eq!(
                                    expected,
                                    payment.is_some(),
                                    "{first}/{second}/{third} {cost:?}"
                                );
                                if let Some(plan) = payment {
                                    valid(cost, &sources, [0; 6], &plan);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn overlapping_blue_and_red_are_reserved_even_when_a_land_is_cheaper() {
    let mut sources = [land(&[0, 1, 3]), land(&[1, 3]), land(&[0])];
    sources[2].creature = true;
    let cost = ManaCost {
        colored: [1, 1, 0, 1, 0, 0],
        generic: 0,
    };
    let p = ordinary(cost, &sources).unwrap();
    valid(cost, &sources, [0; 6], &p);
    assert!(p.taps.contains(&(2, 0)));
    assert!(!p.taps.contains(&(0, 0)));
}

#[test]
fn hold_up_double_blue_and_an_agents_specified_followup() {
    let sources = [land(&[1]), land(&[1]), land(&[0]), land(&[3])];
    let cost = ManaCost {
        colored: [0; 6],
        generic: 2,
    };
    let uu = ManaCost {
        colored: unit(1, 2),
        generic: 0,
    };
    let rw = ManaCost {
        colored: [1, 0, 0, 1, 0, 0],
        generic: 0,
    };
    for (preferred, options) in [(None, vec![uu]), (Some(uu), vec![rw, rw, rw, rw])] {
        let p = solve(
            cost,
            Pools::default(),
            &sources,
            preferred,
            &options,
            SpendBonus::default(),
        )
        .unwrap();
        assert_eq!(p.payment, [1, 0, 0, 1, 0, 0]);
    }
}

#[test]
fn generic_spends_painless_colorless_and_preserves_the_archdruid() {
    let mut painland = land(&[0, 1, 5]);
    painland.modes[0].pain = true;
    painland.modes[1].pain = true;
    let cost = ManaCost {
        colored: [0; 6],
        generic: 1,
    };
    let p = ordinary(cost, &[land(&[0]), painland]).unwrap();
    assert_eq!(p.taps, vec![(1, 2)]);
    assert_eq!(p.payment, unit(5, 1));

    let mut archdruid = land(&[4]);
    archdruid.variable = true;
    archdruid.creature = true;
    archdruid.modes[0].mana[4] = 3;
    let mut elves = land(&[4]);
    elves.creature = true;
    let p = ordinary(
        ManaCost {
            colored: unit(4, 1),
            generic: 0,
        },
        &[archdruid.clone(), elves],
    )
    .unwrap();
    assert_eq!(p.taps, vec![(1, 0)]);
    let p = ordinary(
        ManaCost {
            colored: unit(4, 2),
            generic: 1,
        },
        &[archdruid],
    )
    .unwrap();
    assert_eq!(p.taps, vec![(0, 0)]);
    assert_eq!(p.payment, unit(4, 3));
}

#[test]
fn spend_three_actual_colors_for_bonus_and_save_treasure_if_possible() {
    let mut treasure = land(&[0, 1, 2, 3, 4]);
    treasure.treasure = true;
    for m in &mut treasure.modes {
        m.sacrifice = true;
    }
    let cost = ManaCost {
        colored: [0; 6],
        generic: 3,
    };
    let bonus = SpendBonus {
        distinct_colors: 3,
        treasure: false,
    };
    let sources = [treasure.clone(), land(&[1])];
    let pool = [2, 0, 0, 0, 0, 1];
    let p = solve(
        cost,
        Pools {
            eligible: pool,
            followup: pool,
            ..Default::default()
        },
        &sources,
        None,
        &[],
        bonus,
    )
    .unwrap();
    valid(cost, &sources, pool, &p);
    assert_eq!(p.payment[..5].iter().filter(|&&n| n > 0).count(), 3);
    assert_eq!(p.taps.len(), 2);
    assert_eq!(p.payment[5], 0);
    let sources = [treasure, land(&[1]), land(&[4])];
    let p = solve(
        cost,
        Pools {
            eligible: pool,
            followup: pool,
            ..Default::default()
        },
        &sources,
        None,
        &[],
        bonus,
    )
    .unwrap();
    valid(cost, &sources, pool, &p);
    assert!(p.taps.iter().all(|&(s, _)| s != 0));
    assert_eq!(p.payment, [1, 1, 0, 0, 1, 0]);
}

#[test]
fn treasure_provenance_can_improve_an_already_funded_payment() {
    let mut treasure = land(&[0, 1, 2, 3, 4]);
    treasure.treasure = true;
    for m in &mut treasure.modes {
        m.sacrifice = true;
    }
    let cost = ManaCost {
        colored: unit(0, 1),
        generic: 0,
    };
    let bonus = SpendBonus {
        treasure: true,
        ..Default::default()
    };
    for already_floating in [false, true] {
        let tagged = unit(0, u32::from(already_floating));
        let p = solve(
            cost,
            Pools {
                eligible: unit(0, 1),
                treasure: tagged,
                followup: unit(0, 1),
            },
            std::slice::from_ref(&treasure),
            None,
            &[],
            bonus,
        )
        .unwrap();
        assert_eq!(p.payment, unit(0, 1));
        assert_eq!(p.taps.len(), usize::from(!already_floating));
        assert!(p.prefer_treasure);
    }
}

#[test]
fn huge_pool_and_locked_cost_work_by_quantities() {
    let cost = ManaCost {
        colored: unit(1, u32::MAX),
        generic: u32::MAX,
    };
    let pool = [u32::MAX, u32::MAX, 0, 0, 0, 0];
    let p = solve(
        cost,
        Pools {
            eligible: pool,
            followup: pool,
            ..Default::default()
        },
        &[],
        None,
        &[],
        SpendBonus::default(),
    )
    .unwrap();
    assert!(p.taps.is_empty());
    assert_eq!(p.payment, pool);
    assert!(ordinary(cost, &[land(&[0, 1])]).is_none());
}
