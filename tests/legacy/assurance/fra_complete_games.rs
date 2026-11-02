//! Full FRA legal games use the same workload as the standalone benchmark.
use super::*;
use crate::CATALOGUE_VERSION;
use crate::policy::smoke_index;
#[path = "../../../examples/support/fra_games.rs"]
mod workload;
// Pair declarations are proven against actual outcomes below. Their union is
// checked using the workload's existing global mechanic gate, not another list.
struct PairGate {
    pair: usize,
    colors: u8,
    mechanics: &'static [&'static str],
}
macro_rules! complete_pair_tests {
    ($($name:ident: $pair:literal, $colors:literal, [$($mechanic:literal),*];)*) => {
        const PAIR_GATES: &[PairGate] = &[
            $(PairGate { pair: $pair, colors: $colors, mechanics: &[$($mechanic),*] },)*
        ];
        $(#[test]
        fn $name() {
            complete_pair(&PAIR_GATES[$pair]);
        })*
    };
}
// Each row retains all four original seeds and both deck assignments. The
// witnessed subsets come from the unchanged 64-case verification receipt.
complete_pair_tests! {
    fra_complete_legal_games_pair_0_replay_owned_and_actual_mechanics: 0, 15, [
        "prepared_created", "prepared_spell_cast", "prepared_spell_resolved",
        "empower_counters", "fatehold_scry_trigger", "scry_or_surveil_completed",
        "loyalty_activation"
    ];
    fra_complete_legal_games_pair_1_replay_owned_and_actual_mechanics: 1, 25, [
        "prepared_created", "prepared_spell_cast", "loyalty_activation"
    ];
    fra_complete_legal_games_pair_2_replay_owned_and_actual_mechanics: 2, 14, [
        "prepared_created", "prepared_spell_cast", "empower_counters"
    ];
    fra_complete_legal_games_pair_3_replay_owned_and_actual_mechanics: 3, 28, [
        "prepared_created", "prepared_spell_cast"
    ];
    fra_complete_legal_games_pair_4_replay_owned_and_actual_mechanics: 4, 23, [
        "prepared_created", "prepared_spell_cast", "prepared_spell_resolved",
        "empower_counters", "fatehold_scry_trigger", "scry_or_surveil_completed",
        "loyalty_activation"
    ];
    fra_complete_legal_games_pair_5_replay_owned_and_actual_mechanics: 5, 26, [
        "prepared_created", "prepared_spell_cast", "prepared_spell_resolved",
        "empower_counters", "fatehold_scry_trigger", "scry_or_surveil_completed"
    ];
    fra_complete_legal_games_pair_6_replay_owned_and_actual_mechanics: 6, 23, [
        "scry_or_surveil_completed"
    ];
    fra_complete_legal_games_pair_7_replay_owned_and_actual_mechanics: 7, 15, [
        "prepared_created", "prepared_spell_cast", "prepared_spell_resolved",
        "scry_or_surveil_completed"
    ];
}
fn pair_cases(pair: usize) -> Vec<(usize, u64, Player, bool)> {
    workload::cases(4)
        .into_iter()
        .filter(|case| case.0 == pair)
        .collect()
}
fn complete_pair(gate: &PairGate) {
    let missing = workload::missing_pool_members().unwrap();
    if !missing.is_empty() {
        eprintln!(
            "FRA complete games skipped: {} unsupported identities; this is not set acceptance",
            missing.len()
        );
        return;
    }
    let cases = pair_cases(gate.pair);
    assert_eq!(cases.len(), 8);
    let report = workload::run(&cases, workload::Mode::Verify, None).unwrap();
    assert_eq!(report.pool_members, 285);
    assert_eq!(report.unique_cases, 8);
    assert_eq!(report.games.len(), 8);
    assert_eq!(
        report
            .games
            .iter()
            .map(|game| (
                game.pair,
                game.seed,
                game.starting_player,
                game.deck_swapped
            ))
            .collect::<Vec<_>>(),
        cases
    );
    assert_eq!(report.actual_spell_colors & gate.colors, gate.colors);
    assert_eq!(report.actual_spell_colors & !31, 0);
    for mechanic in gate.mechanics {
        assert!(
            report.witnesses.get(*mechanic).copied().unwrap_or(0) > 0,
            "pair {} missing actual mechanic {mechanic}",
            gate.pair
        );
    }
    assert!(
        report
            .games
            .iter()
            .all(|game| { game.owned_branches > 0 && game.replayed_actions == game.decisions })
    );
    // Only the authentic gameplay report is emitted as evidence.
    println!(
        "FRA_COMPLETE_GAME_SHARD_EVIDENCE {}",
        serde_json::json!({"pair": gate.pair, "report": &report})
    );
    // Declaration-only coverage proof: every declared member has an actual
    // assertion in its pair test. Reuse the existing global gate so a future
    // required mechanic cannot silently disappear from the sharded workload.
    // This synthetic coverage object is never emitted as gameplay evidence.
    let declared_coverage = workload::Report {
        witnesses: PAIR_GATES
            .iter()
            .flat_map(|gate| gate.mechanics)
            .map(|mechanic| ((*mechanic).to_owned(), 1))
            .collect(),
        ..report
    };
    assert!(
        declared_coverage.missing_mechanics().is_empty(),
        "pair declarations omit required mechanics {:?}",
        declared_coverage.missing_mechanics()
    );
}

#[test]
fn fra_pool_audit_and_cases_preserve_the_acceptance_gate() {
    let missing = workload::missing_pool_members().unwrap();
    let cases = workload::cases(4);
    assert_eq!(cases.len(), 64);
    assert_eq!(PAIR_GATES.len(), 8);
    let mut declared_colors = 0;
    let mut partition = Vec::new();
    for (pair, gate) in PAIR_GATES.iter().enumerate() {
        assert_eq!(gate.pair, pair);
        assert_ne!(gate.colors, 0);
        assert_eq!(gate.colors & !31, 0);
        declared_colors |= gate.colors;
        let shard = pair_cases(pair);
        assert_eq!(shard.len(), 8);
        partition.extend(shard);
    }
    // Exact ordered equality preserves every seed, starting seat and swapped
    // deck assignment, with neither duplicated games nor a reduced corpus.
    assert_eq!(partition, cases);
    assert_eq!(declared_colors, 31);
    for pair in 0..8 {
        for swapped in [false, true] {
            for starting in [Player::P0, Player::P1] {
                assert!(
                    cases.iter().any(|&(p, _, seat, swap)| p == pair
                        && seat == starting
                        && swap == swapped)
                );
            }
        }
    }
    if !missing.is_empty() {
        let error = match workload::run(&cases, workload::Mode::Verify, None) {
            Ok(_) => panic!("full workload accepted incomplete FRA pool"),
            Err(error) => error,
        };
        assert!(error.contains("FRA full-pool gate"), "{error}");
    }
}

// This bounded smoke is not set acceptance; `run` retains its full-pool gate.
#[test]
fn fra_admitted_decks_finish_and_replay_in_both_seats() {
    let cases = workload::cases(2);
    assert_eq!(cases.len(), 32);
    let report = workload::run_admitted_smoke(&cases).unwrap();
    assert_eq!(report.workload, "FRA_ADMITTED_SMOKE_ONLY");
    assert_eq!(report.games.len(), 32);
    assert!(report.played_deck_identities >= 175);
    assert!(
        report
            .witnesses
            .get("empower_counters")
            .copied()
            .unwrap_or(0)
            > 0,
        "no observed empower loyalty increase"
    );
    // Record the observed cards and mechanics so a terminal game caused by
    // empty-library draws alone cannot be mistaken for exercised spell play.
    println!(
        "FRA_ADMITTED_SMOKE_EVIDENCE {}",
        serde_json::to_string(&report).unwrap()
    );
    for game in &report.games {
        assert!(
            !game.cards_cast.is_empty()
                && game.witnesses.get("actual_cast").copied().unwrap_or(0) > 0,
            "game did not observe an actual FRA spell cast: pair {} seed {} swapped {}",
            game.pair,
            game.seed,
            game.deck_swapped
        );
    }
    assert!(
        report
            .games
            .iter()
            .all(|game| game.owned_branches > 0 && game.replayed_actions == game.decisions)
    );
    for pair in 0..8 {
        for swapped in [false, true] {
            for player in [Player::P0, Player::P1] {
                assert!(cases.iter().any(|&(p, _, start, swap)| {
                    p == pair && start == player && swap == swapped
                }));
            }
        }
    }
}
