//! Complete self-play using only implemented, pinned Standard-legal cards.
//! This exercises a supported deck, not the complete Standard card pool.
use rustmage::{Card, Config, DeckRules, Game, Player};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let games: usize = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "1".into())
        .parse()?;
    let mut white = vec![Card::Plains; 28];
    for name in [
        "Savannah Lions",
        "Healer's Hawk",
        "Bishop's Soldier",
        "Brightblade Stoat",
        "Leonin Skyhunter",
        "Serra Angel",
        "Shrike Force",
        "Iron Giant",
    ] {
        white.extend([Card::from_name(name).unwrap(); 4]);
    }
    let mut green = vec![Card::Forest; 24];
    for name in [
        "Ankle Biter",
        "Dragon Sniper",
        "Magnigoth Sentry",
        "Colossadactyl",
        "Tajuru Pathwarden",
        "Galewind Moose",
        "Gigantosaurus",
        "Bear Cub",
        "Jibbirik Omnivore",
    ] {
        green.extend([Card::from_name(name).unwrap(); 4]);
    }
    let mut total_decisions = 0;
    for seed in 0..games {
        let mut game = Game::new(Config {
            decks: [white.clone(), green.clone()],
            seed: seed as u64,
            starting_player: Player::P0,
            deck_rules: DeckRules::Standard,
        })?;
        let mut actions = Vec::new();
        for decisions in 0..100_000 {
            if let Some(outcome) = game.outcome() {
                println!(
                    "{outcome:?} after {decisions} primitive decisions; catalogue {}",
                    rustmage::CATALOGUE_VERSION
                );
                total_decisions += decisions;
                break;
            }
            game.legal_actions_into(&mut actions);
            let index = rustmage::policy::smoke_index(&game, &actions);
            game.apply(game.actor().unwrap(), actions[index].clone())?;
        }
        if game.outcome().is_none() {
            return Err("self-play budget reached without a terminal result".into());
        }
    }
    println!("Completed {games} games and {total_decisions} decisions");
    Ok(())
}
