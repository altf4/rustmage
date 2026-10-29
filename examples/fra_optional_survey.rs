//! Opt-in FRA legal survey: fixed seeds, both seats, owned branching and replay.
use rustmage::policy::smoke_index;
use rustmage::*;
#[allow(
    dead_code,
    reason = "The runner is shared with the standard FRA workload"
)]
#[path = "support/fra_games.rs"]
mod workload;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = std::env::var_os("RUSTMAGE_FRA_EVIDENCE_DIR").map(std::path::PathBuf::from);
    let report = workload::run_survey(dir.as_deref())?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
