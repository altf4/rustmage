//! Diagnostic JSON-lines runner. First line: Config. Subsequent lines: Request.
use std::io::{self, BufRead, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();
    let config = lines.next().ok_or("first line must be a Config")??;
    let mut session = rustmage::protocol::Session::new(serde_json::from_str(&config)?)?;
    let stdout = io::stdout();
    let mut out = stdout.lock();
    for line in lines {
        writeln!(out, "{}", session.json(&line?))?;
        out.flush()?;
    }
    Ok(())
}
