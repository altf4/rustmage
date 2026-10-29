//! Full FRA legal games use the same workload as the standalone benchmark.
use super::*;
use crate::CATALOGUE_VERSION;
use crate::policy::smoke_index;
#[path = "../../examples/support/fra_games.rs"]
mod workload;
#[test]
fn fra_complete_legal_games_replay_owned_and_actual_mechanics() {
    let missing = workload::missing_pool_members().unwrap();
    if !missing.is_empty() {
        eprintln!(
            "FRA complete games skipped: {} unsupported identities; this is not set acceptance",
            missing.len()
        );
        return;
    }
    let report = workload::run(&workload::cases(4), workload::Mode::Verify, None).unwrap();
    println!(
        "FRA_COMPLETE_GAME_EVIDENCE {}",
        serde_json::to_string(&report).unwrap()
    );
    assert_eq!(report.pool_members, 285);
    assert_eq!(report.actual_spell_colors, 31);
    assert!(
        report.missing_mechanics().is_empty(),
        "Missing actual mechanics {:?}",
        report.missing_mechanics()
    );
    assert!(
        report
            .games
            .iter()
            .all(|g| g.owned_branches > 0 && g.replayed_actions == g.decisions)
    );
}

#[test]
fn fra_pool_audit_and_cases_preserve_the_acceptance_gate() {
    let missing = workload::missing_pool_members().unwrap();
    let cases = workload::cases(4);
    assert_eq!(cases.len(), 48);
    for pair in 0..6 {
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
    let cases: Vec<_> = workload::cases(2)
        .into_iter()
        .filter(|(pair, _, _, _)| *pair == 0)
        .collect();
    assert_eq!(cases.len(), 4);
    let report = workload::run_admitted_smoke(&cases).unwrap();
    assert_eq!(report.workload, "FRA_ADMITTED_SMOKE_ONLY");
    assert_eq!(report.games.len(), 4);
    assert!(
        report
            .games
            .iter()
            .all(|game| game.owned_branches > 0 && game.replayed_actions == game.decisions)
    );
    for swapped in [false, true] {
        for player in [Player::P0, Player::P1] {
            assert!(
                cases
                    .iter()
                    .any(|&(_, _, start, swap)| start == player && swap == swapped)
            );
        }
    }
}
