//! Native Windows launcher: preserve long compiler argument lists without cmd.exe.
use std::env;
use std::process::{exit, Command};

fn main() {
    let (Some(python), Some(guard)) = (
        env::var_os("RUSTMAGE_PYTHON"),
        env::var_os("RUSTMAGE_RUSTC_GUARD"),
    ) else {
        eprintln!("Rust compilation requires scripts/resource_guard.py");
        exit(1);
    };
    match Command::new(python)
        .arg(guard)
        .args(env::args_os().skip(1))
        .status()
    {
        Ok(status) => exit(status.code().unwrap_or(1)),
        Err(error) => {
            eprintln!("Could not start the Rust compiler guard: {error}");
            exit(1);
        }
    }
}
