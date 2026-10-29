//! Full FRA legal games use the same workload as the standalone benchmark.
use super::*;
use crate::CATALOGUE_VERSION;
use crate::policy::smoke_index;
#[path = "../../examples/support/fra_games.rs"]
mod workload;
#[test]
#[ignore = "full FRA set and complete-game acceptance are not yet verified; run explicitly when all 285 identities are supported"]
fn fra_complete_legal_games_replay_owned_and_actual_mechanics() {
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
    assert_eq!(cases.len(), 24);
    for pair in 0..6 {
        assert!(cases.iter().any(|&(p, _, seat)| p == pair && seat == Player::P0));
        assert!(cases.iter().any(|&(p, _, seat)| p == pair && seat == Player::P1));
    }
    if !missing.is_empty() {
        let error = match workload::run(&cases, workload::Mode::Verify, None) {
            Ok(_) => panic!("full workload accepted incomplete FRA pool"),
            Err(error) => error,
        };
        assert!(error.contains("FRA full-pool gate"), "{error}");
    }
}
