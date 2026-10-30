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
    let args: Vec<_> = std::env::args().skip(1).collect();
    let single = if args.is_empty() {
        None
    } else if args.len() == 4 && args[0] == "--case" {
        let swapped = match args[3].as_str() {
            "0" => false,
            "1" => true,
            _ => return Err("swap must be 0 or 1".into()),
        };
        Some((args[1].parse()?, args[2].parse()?, swapped))
    } else {
        return Err(
            "usage: fra_optional_survey [--case PAIR(0..4) SEED_INDEX(0..1) SWAP(0|1)]".into(),
        );
    };
    let report = workload::run_survey(dir.as_deref(), single)?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
