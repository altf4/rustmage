use rustmage::{Card, Config, DeckRules, Game, Player};
use std::hint::black_box;
use std::time::Instant;

// Optional second argument loads the exact mainboards and checks every sideboard
// card too. The existing native smoke policy supplies all gameplay decisions.
fn named_decks(path: &str) -> Vec<(String, Vec<Card>)> {
    let data: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).expect("decklist archive"))
            .expect("decklist JSON");
    data["decks"]
        .as_object()
        .expect("named decks")
        .iter()
        .map(|(name, list)| {
            let mut mainboard = Vec::new();
            for side in ["mainboard", "sideboard"] {
                let mut count = 0usize;
                for row in list[side].as_array().expect("deckboard") {
                    let name = row["name"].as_str().expect("card name");
                    let card = Card::from_name(name)
                        .unwrap_or_else(|| panic!("unknown pool card: {name}"));
                    assert!(card.is_supported(), "gated pool card: {name}");
                    let quantity = usize::try_from(row["quantity"].as_u64().expect("quantity"))
                        .expect("quantity range");
                    count += quantity;
                    if side == "mainboard" {
                        mainboard.extend(std::iter::repeat_n(card, quantity));
                    }
                }
                assert_eq!(count, if side == "mainboard" { 60 } else { 15 });
            }
            (name.clone(), mainboard)
        })
        .collect()
}

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
    let archive = std::env::args().nth(2);
    let named = archive
        .as_deref()
        .map(named_decks)
        .unwrap_or_else(|| vec![("native-smoke".into(), deck)]);
    assert!(!named.is_empty());
    let config = Config {
        decks: [named[0].1.clone(), named[1 % named.len()].1.clone()],
        seed: 42,
        starting_player: Player::P0,
        deck_rules: if archive.is_some() {
            DeckRules::Standard
        } else {
            DeckRules::Freeform
        },
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
        config.decks = [
            named[seed % named.len()].1.clone(),
            named[(seed + 1) % named.len()].1.clone(),
        ];
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
            "decklist_archive": archive.as_deref(),
            "deck_names": named.iter().map(|(name, _)| name).collect::<Vec<_>>(),
            "deck_rules": if archive.is_some() { "Standard" } else { "Freeform" },
            "games_per_second": games as f64 / elapsed,
            "decisions_per_second": decisions as f64 / elapsed,
            "max_decisions_per_game": max_game,
            "initial_state_bytes_excluding_capacity_and_allocator": root.state_bytes(),
            "clone_ns": clone_ns, "restore_ns": restore_ns,
        "note": "native smoke policy and observation construction included; optional named decks rotate adjacent matchups through both seats; setup and gameplay timed, archive loading excluded; no model, tensor encoding, or per-decision JSON; sampled gameplay is not complete card-interaction coverage or XMage parity"
        })
    );
}
