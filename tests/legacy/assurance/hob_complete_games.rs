//! Full HOB legal games use the same workload as the standalone benchmark.
use super::*;
use crate::CATALOGUE_VERSION;
use crate::policy::smoke_index;
#[path = "../../../examples/support/hob_games.rs"]
mod workload;
#[test]
fn hob_complete_legal_games_replay_owned_and_actual_mechanics() {
    let report = workload::run(&workload::cases(4), workload::Mode::Verify, None).unwrap();
    println!(
        "HOB_COMPLETE_GAME_EVIDENCE {}",
        serde_json::to_string(&report).unwrap()
    );
    assert_eq!(report.pool_members, 193);
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
