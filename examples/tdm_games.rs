//! Exact TDM terminal corpus, persisted replay verification and separate benchmark.
#[path = "support/tdm_games.rs"]
mod workload;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let command = std::env::args().nth(1).unwrap_or_else(|| "verify".into());
    workload::verify_hash_vectors()?;
    let w = workload::Workload::load()?;
    let dir = workload::evidence_dir()?;
    match command.as_str(){
        "verify"=>{for pair in 0..7{let r=workload::run(&w,&w.pair(pair),workload::Mode::Verify,Some(&dir),None)?;workload::write_shard(&r,pair,&dir)?;}let r=workload::aggregate(&w,&dir)?;println!("{}",serde_json::to_string_pretty(&r)?);}
        "aggregate"=>{let r=workload::aggregate(&w,&dir)?;println!("{}",serde_json::to_string_pretty(&r)?);}
        "benchmark"|"benchmark-freeform"=>{let verified=workload::aggregate(&w,&dir)?;let cases=if command=="benchmark"{w.standard()}else{w.freeform()};let r=workload::run(&w,&cases.repeat(3),workload::Mode::Benchmark,None,Some(&verified))?;println!("{}",serde_json::to_string_pretty(&r)?);}
        _=>return Err("use verify, aggregate, benchmark, or benchmark-freeform; fixed decks/seeds/repetitions are immutable".into()),
    }
    Ok(())
}
