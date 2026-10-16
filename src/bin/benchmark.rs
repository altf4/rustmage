use rustmage::{Card, Config, DeckRules, Game, Player};
use std::hint::black_box;
use std::time::Instant;

fn main() {
    let games: usize = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "1000".into())
        .parse()
        .expect("game count");
    assert!(games > 0);
    let deck: Vec<_> = [
        vec![Card::Mountain; 24],
        vec![Card::LightningBolt; 20],
        vec![Card::GoblinPiker; 12],
        vec![Card::HillGiant; 4],
    ]
    .concat();
    let config = Config {
        decks: [deck.clone(), deck],
        seed: 42,
        starting_player: Player::P0,
        deck_rules: DeckRules::Freeform,
    };
    let root = Game::new(config.clone()).unwrap();
    let saved = root.snapshot();
    let clone_iterations = 100_000;
    let start = Instant::now();
    for _ in 0..clone_iterations {
        black_box(black_box(&root).clone());
    }
    let clone_ns = start.elapsed().as_nanos() as f64 / clone_iterations as f64;
    let mut branch = root.clone();
    let start = Instant::now();
    for _ in 0..clone_iterations {
        branch.restore(black_box(&saved)).unwrap();
        black_box(&branch);
    }
    let restore_ns = start.elapsed().as_nanos() as f64 / clone_iterations as f64;
    let mut actions = Vec::new();
    let mut decisions = 0u64;
    let mut max_game = 0;
    let start = Instant::now();
    for seed in 0..games {
        let mut config = config.clone();
        config.seed = seed as u64;
        let mut game = Game::new(config).unwrap();
        let mut count = 0;
        while game.outcome().is_none() {
            game.legal_actions_into(&mut actions);
            let index = rustmage::policy::smoke_index(&game, &actions);
            game.apply(game.actor().unwrap(), actions[index].clone())
                .unwrap();
            count += 1;
            assert!(count < 100_000, "benchmark did not terminate naturally");
        }
        max_game = max_game.max(count);
        decisions += count;
        black_box(game.outcome());
    }
    let elapsed = start.elapsed().as_secs_f64();
    println!(
        "{}",
        serde_json::json!({
            "games": games, "decisions": decisions, "seconds": elapsed,
            "games_per_second": games as f64 / elapsed,
            "decisions_per_second": decisions as f64 / elapsed,
            "max_decisions_per_game": max_game,
            "initial_state_bytes_excluding_capacity_and_allocator": root.state_bytes(),
            "clone_ns": clone_ns, "restore_ns": restore_ns,
        "note": "limited catalogue; native smoke policy and observation construction included; no model, tensor encoding, or JSON; not an XMage parity comparison"
        })
    );
}
