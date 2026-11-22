//! Complete self-play using only implemented, pinned Standard-legal cards.
//! This exercises a supported deck, not the complete Standard card pool.
#[path = "support/standard.rs"]
mod standard;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let games: usize = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "1".into())
        .parse()?;
    standard::run(games)
}
