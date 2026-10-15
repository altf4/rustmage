//! Complete self-play using only implemented, pinned Standard-legal cards.
//! This exercises a supported deck, not the complete Standard card pool.
use rustmage::{Card, Config, DeckRules, Game, Player};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut deck = vec![Card::Plains; 18];
    deck.extend([Card::Forest; 18]);
    for name in [
        "Savannah Lions",
        "Leonin Skyhunter",
        "Serra Angel",
        "Bear Cub",
        "Jibbirik Omnivore",
        "Gigantosaurus",
    ] {
        deck.extend([Card::from_name(name).unwrap(); 4]);
    }
    let mut game = Game::new(Config {
        decks: [deck.clone(), deck],
        seed: 2026,
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
            return Ok(());
        }
        game.legal_actions_into(&mut actions);
        let index = rustmage::policy::smoke_index(&game, &actions);
        game.apply(game.actor().unwrap(), actions[index].clone())?;
    }
    Err("self-play budget reached without a terminal result".into())
}
