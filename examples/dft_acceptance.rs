//! DFT-only legal natural games. Verification and benchmark are separate workloads.
//! `verify 4 1 --pairing 0` runs one of ten unchanged eight-case groups.
//! `aggregate 4 1 report0.json ... report9.json` applies the full-corpus gates.
use rustmage::policy::smoke_index;
use rustmage::*;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
#[path = "support/dft_games.rs"]
mod workload;

type Case = (usize, u64, Player, bool);
#[derive(Deserialize)]
struct GroupReport {
    workload: String,
    catalogue: String,
    mode: String,
    pool_members: usize,
    selected_pairing: Option<usize>,
    unique_cases: usize,
    decks: serde_json::Value,
    games: Vec<serde_json::Value>,
    witnesses: BTreeMap<String, u64>,
    actual_spell_colors: u8,
    seconds: f64,
    decisions: usize,
}
#[derive(Deserialize)]
enum NaturalOutcome {
    Winner(Player),
    Draw,
}
#[derive(Deserialize)]
struct GameReceipt {
    pair: usize,
    seed: u64,
    starting_player: Player,
    deck_swapped: bool,
    decisions: usize,
    outcome: NaturalOutcome,
    owned_branches: usize,
    replayed_actions: usize,
    trace_digest: Option<String>,
    cards_cast: BTreeSet<String>,
    witnesses: BTreeMap<String, u64>,
}
fn witness_gate(witnesses: &BTreeMap<String, u64>, colors: u8) -> Result<(), String> {
    let missing: Vec<_> = workload::REQUIRED_MECHANICS
        .into_iter()
        .filter(|key| witnesses.get(*key).copied().unwrap_or(0) == 0)
        .collect();
    if !missing.is_empty() {
        return Err(format!("actual mechanic witnesses missing: {missing:?}"));
    }
    if colors != 31 {
        return Err("actual cast colors do not cover all five".into());
    }
    Ok(())
}
fn aggregate(
    n: u64,
    repeats: usize,
    paths: &[String],
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    if paths.is_empty() {
        return Err("aggregate needs report files".into());
    }
    let expected: BTreeMap<Case, usize> = workload::cases(n)
        .into_iter()
        .map(|case| (case, repeats))
        .collect();
    let expected_decks = serde_json::to_value(workload::deck_reports()?)?;
    let mut observed = BTreeMap::<Case, usize>::new();
    let mut games = Vec::new();
    let mut witnesses = BTreeMap::<String, u64>::new();
    let mut colors = 0;
    let mut decisions = 0;
    let mut seconds = 0.0;
    for path in paths {
        let group: GroupReport = serde_json::from_slice(&std::fs::read(path)?)?;
        if group.workload != "DFT_ARCHETYPES_V1"
            || group.catalogue != CATALOGUE_VERSION
            || group.mode != "verification"
            || group.pool_members != 276
            || group.decks != expected_decks
            || !group.seconds.is_finite()
            || group.seconds <= 0.0
        {
            return Err(format!("incompatible verification report {path}").into());
        }
        let mut group_cases = BTreeSet::new();
        let mut group_witnesses = BTreeMap::<String, u64>::new();
        let mut group_colors = 0;
        let mut group_decisions = 0;
        for value in group.games {
            let game: GameReceipt = serde_json::from_value(value.clone())?;
            let case = (
                game.pair,
                game.seed,
                game.starting_player,
                game.deck_swapped,
            );
            if !expected.contains_key(&case)
                || group.selected_pairing.is_some_and(|pair| pair != game.pair)
                || game.decisions == 0
                || game.decisions > 8192
                || game.owned_branches == 0
                || game.replayed_actions != game.decisions
                || !game.trace_digest.as_ref().is_some_and(|digest| {
                    digest.len() == 16 && digest.bytes().all(|b| b.is_ascii_hexdigit())
                })
            {
                return Err(format!(
                    "invalid natural/replay/owned case receipt {case:?} in {path}"
                )
                .into());
            }
            // Deserialization permits only actual terminal Outcome shapes.
            match game.outcome {
                NaturalOutcome::Winner(Player::P0 | Player::P1) | NaturalOutcome::Draw => {}
            }
            *observed.entry(case).or_default() += 1;
            group_cases.insert(case);
            group_decisions += game.decisions;
            for card in game.cards_cast {
                group_colors |= Card::from_name(&card)
                    .ok_or("unknown actual cast card")?
                    .colors();
            }
            for (key, count) in game.witnesses {
                *group_witnesses.entry(key).or_default() += count;
            }
            games.push(value);
        }
        if group_cases.len() != group.unique_cases
            || group_witnesses != group.witnesses
            || group_colors != group.actual_spell_colors
            || group_decisions != group.decisions
        {
            return Err(format!("inconsistent per-game aggregate receipts in {path}").into());
        }
        for (key, count) in group_witnesses {
            *witnesses.entry(key).or_default() += count;
        }
        colors |= group_colors;
        decisions += group_decisions;
        seconds += group.seconds;
    }
    if observed != expected {
        return Err("aggregate case multiset differs from the exact requested full corpus".into());
    }
    witness_gate(&witnesses, colors)?;
    Ok(serde_json::json!({
        "workload":"DFT_ARCHETYPES_V1", "catalogue":CATALOGUE_VERSION, "mode":"verification",
        "corpus_gate":"passed", "scope":"Exact unchanged full DFT case multiset assembled from bounded verification groups",
        "pool_members":276, "unique_cases":expected.len(), "decks":expected_decks,
        "decisions":decisions, "games_per_second":games.len() as f64 / seconds,
        "decisions_per_second":decisions as f64 / seconds, "seconds":seconds,
        "games":games, "witnesses":witnesses, "actual_spell_colors":colors,
        "measurement":"Sum of group verification durations, including replay/owned checks and telemetry; excludes this aggregation I/O. Not a simulation benchmark."
    }))
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let mode = args
        .first()
        .map(String::as_str)
        .unwrap_or("verify")
        .to_string();
    if mode == "aggregate" {
        if args.len() < 4 {
            return Err("use aggregate seeds_per_pairing repetitions report.json ...".into());
        }
        let n: u64 = args[1].parse()?;
        let repeats: usize = args[2].parse()?;
        if n == 0 || repeats == 0 {
            return Err("seeds and repetitions must be positive".into());
        }
        println!(
            "{}",
            serde_json::to_string_pretty(&aggregate(n, repeats, &args[3..])?)?
        );
        return Ok(());
    }
    let mut pairing = None;
    if let Some(index) = args.iter().position(|arg| arg == "--pairing") {
        let pair: usize = args
            .get(index + 1)
            .ok_or("--pairing needs an index from 0 through 9")?
            .parse()?;
        if pair >= 10 {
            return Err("pairing must be from 0 through 9".into());
        }
        pairing = Some(pair);
        args.drain(index..=index + 1);
    }
    if args.len() > 3 || args.iter().any(|arg| arg.starts_with("--")) {
        return Err("use verify|benchmark seeds_per_pairing repetitions [--pairing 0..9]".into());
    }
    let n: u64 = args.get(1).map(String::as_str).unwrap_or("4").parse()?;
    let repeats: usize = args.get(2).map(String::as_str).unwrap_or("1").parse()?;
    if n == 0 || repeats == 0 {
        return Err("seeds per pairing and corpus repetitions must be positive".into());
    }
    let mode = match mode.as_str() {
        "verify" => workload::Mode::Verify,
        "benchmark" => workload::Mode::Benchmark,
        _ => return Err("use verify or benchmark, followed by seeds per pairing".into()),
    };
    let dir = std::env::var_os("RUSTMAGE_DFT_EVIDENCE_DIR").map(std::path::PathBuf::from);
    let cases: Vec<_> = workload::cases(n)
        .into_iter()
        .filter(|case| pairing.is_none_or(|pair| case.0 == pair))
        .collect();
    let report = workload::run(&cases.repeat(repeats), mode, dir.as_deref())?;
    let mut value = serde_json::to_value(&report)?;
    value["selected_pairing"] = serde_json::json!(pairing);
    if pairing.is_none() {
        let missing = report.missing_mechanics();
        if !missing.is_empty() {
            return Err(format!("actual mechanic witnesses missing: {missing:?}").into());
        }
        if report.actual_spell_colors != 31 {
            return Err("actual cast colors do not cover all five".into());
        }
        value["corpus_gate"] = serde_json::json!("passed");
    } else {
        value["corpus_gate"] = serde_json::json!("partial_pairing; aggregate full corpus required");
    }
    println!("{}", serde_json::to_string_pretty(&value)?);
    Ok(())
}
