//! Diagnostic JSON-lines runner. First line: SessionConfig, then Request lines.
use std::io::{self, BufRead, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();
    let config = lines
        .next()
        .ok_or("first line must be a session config")??;
    let mut session =
        rustmage::protocol::Session::from_config(serde_json::from_str(&config)?, None)?;
    let stdout = io::stdout();
    let mut out = stdout.lock();
    for line in lines {
        writeln!(out, "{}", session.json(&line?))?;
        out.flush()?;
    }
    Ok(())
}
