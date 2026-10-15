//! Supported Standard self-play exercising counters, death triggers and stun replacements.
use rustmage::{Action, Card, Config, DeckRules, Game, Player};

fn deck(lands: &[(&str, usize)], spells: &[&str]) -> Vec<Card> {
    let mut cards = Vec::new();
    for &(name, count) in lands {
        cards.extend(vec![Card::from_name(name).unwrap(); count]);
    }
    for &name in spells {
        cards.extend([Card::from_name(name).unwrap(); 4]);
    }
    cards
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let games: usize = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "1".into())
        .parse()?;
    let white_black = deck(
        &[("Plains", 12), ("Forest", 12)],
        &[
            "Ironpaw Aspirant",
            "Sterling Supplier",
            "Sazh's Chocobo",
            "Machinesmith Automaton",
            "Iron Giant",
            "Inspiring Overseer",
            "Thistledown Players",
            "Helpful Hunter",
            "Tajuru Pathwarden",
        ],
    );
    let blue_green = deck(
        &[("Island", 12), ("Swamp", 12)],
        &[
            "Blight Rot",
            "Bile-Vial Boggart",
            "Utrom Scientists",
            "Quick Study",
            "Refute",
            "Rook Turret",
            "Aegis Turtle",
            "Skeleton Archer",
            "Zombify",
        ],
    );
    let mut total = 0;
    let mut targets = 0;
    let mut multi = 0;

    for seed in 0..games {
        let mut game = Game::new(Config {
            decks: [white_black.clone(), blue_green.clone()],
            seed: seed as u64,
            starting_player: Player::P0,
            deck_rules: DeckRules::Standard,
        })?;
        let mut actions = Vec::new();
        for _ in 0..100_000 {
            if game.outcome().is_some() {
                break;
            }
            game.legal_actions_into(&mut actions);
            let action = actions[rustmage::policy::smoke_index(&game, &actions)].clone();
            targets += usize::from(matches!(action, Action::ChooseTarget { .. }));
            multi += usize::from(matches!(action, Action::BeginCast { .. }));
            game.apply(game.actor().unwrap(), action)?;
            total += 1;
        }
        if game.outcome().is_none() {
            return Err("self-play budget reached without terminal result".into());
        }
    }
    println!(
        "Completed {games} counter games, {total} decisions, {targets} target choices, {multi} multi-target casts; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
    Ok(())
}
