//! FRA-only legal natural games. Verification and benchmark are separate workloads.
use rustmage::policy::smoke_index;
use rustmage::*;
#[path = "support/fra_games.rs"]
mod workload;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mode = std::env::args().nth(1).unwrap_or_else(|| "verify".into());
    let n: u64 = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "4".into())
        .parse()?;
    let repeats: usize = std::env::args()
        .nth(3)
        .unwrap_or_else(|| "1".into())
        .parse()?;
    if n == 0 || repeats == 0 {
        return Err("seeds per pairing and corpus repetitions must be positive".into());
    }
    let mode = match mode.as_str() {
        "verify" => workload::Mode::Verify,
        "benchmark" => workload::Mode::Benchmark,
        _ => return Err("use verify or benchmark, followed by seeds per pairing".into()),
    };
    let dir = std::env::var_os("RUSTMAGE_FRA_EVIDENCE_DIR").map(std::path::PathBuf::from);
    let cases = workload::cases(n).repeat(repeats);
    let report = workload::run(&cases, mode, dir.as_deref())?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    let missing = report.missing_mechanics();
    if !missing.is_empty() {
        return Err(format!("actual mechanic witnesses missing: {missing:?}").into());
    }
    if report.actual_spell_colors != 31 {
        return Err("actual cast colors do not cover all five".into());
    }
    Ok(())
}
