//! Each pairing is a separate bounded native case; missing cards fail closed.
#[path = "../examples/support/tdm_games.rs"]
mod workload;
fn pair(pair: usize) {
    let w = workload::Workload::load().expect(
        "TDM terminal games require all276 complete physical identities; no skip/PASS while gated",
    );
    let dir =
        workload::evidence_dir().expect("persistent unique RUSTMAGE_TDM_EVIDENCE_DIR is required");
    let cases = w.pair(pair);
    assert_eq!(cases.len(), if pair == 6 { 4 } else { 8 });
    let report = workload::run(&w, &cases, workload::Mode::Verify, Some(&dir), None).unwrap();
    assert!(
        report
            .games
            .iter()
            .all(|g| g.replayed_actions == g.decisions
                && g.cast_count > 0
                && g.decisions <= 4096
                && !g.owned_decision_roots.is_empty())
    );
    workload::write_shard(&report, pair, &dir).unwrap();
    println!(
        "TDM_COMPLETE_GAME_SHARD_EVIDENCE {}",
        serde_json::to_string(&report).unwrap()
    );
}
#[test]
fn tdm_complete_games_pair_0_abzan_jeskai() {
    pair(0)
}
#[test]
fn tdm_complete_games_pair_1_jeskai_sultai() {
    pair(1)
}
#[test]
fn tdm_complete_games_pair_2_sultai_mardu() {
    pair(2)
}
#[test]
fn tdm_complete_games_pair_3_mardu_temur() {
    pair(3)
}
#[test]
fn tdm_complete_games_pair_4_temur_abzan() {
    pair(4)
}
#[test]
fn tdm_complete_games_pair_5_dragons_jeskai() {
    pair(5)
}
#[test]
fn tdm_complete_games_pair_6_cutter_freeform() {
    pair(6)
}
// Test-thread count is1 under the native runner. Alphabetic zz runs after shards;
// aggregate can also run separately against the same authenticated shard files.
#[test]
fn zz_tdm_complete_games_corpus_actual_outcome_gate() {
    let w = workload::Workload::load().expect("all276 complete physical identities required");
    let dir = workload::evidence_dir().unwrap();
    let report = workload::aggregate(&w, &dir).unwrap();
    assert_eq!(report.games.len(), 52);
    assert_eq!(report.standard_cases, 48);
    assert_eq!(report.freeform_cases, 4);
    report.check_complete_standard(&w).unwrap();
    println!(
        "TDM_COMPLETE_GAME_CORPUS_EVIDENCE {}",
        serde_json::to_string(&report).unwrap()
    );
}

/// Pure receipt/input integrity. A pass is not terminal-game acceptance.
#[test]
fn tdm_game_receipt_hash_vectors_and_frozen_input_digest() {
    workload::verify_hash_vectors().unwrap();
}
