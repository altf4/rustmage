//! Supported Standard self-play exercising entry/death/attack trigger programs.
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
    let white_blue = deck(
        &[("Plains", 12), ("Island", 12)],
        &[
            "Helpful Hunter",
            "Inspiring Overseer",
            "Bellowing Crier",
            "Buzz Bots",
            "Summit Sentinel",
            "Icewind Elemental",
            "Lifecreed Duo",
            "Rook Turret",
            "Cloudblazer",
        ],
    );
    let black_green = deck(
        &[("Swamp", 12), ("Forest", 12)],
        &[
            "Bogwater Lumaret",
            "Burglar Rat",
            "Hecteyes",
            "Vampire Spawn",
            "Glidedive Duo",
            "Virulent Emissary",
            "Eumidian Terrabotanist",
            "Pelakka Wurm",
            "Primordial Pachyderm",
        ],
    );
    let mut total = 0;
    let mut orders = 0;
    let mut optional = 0;
    let mut discards = 0;
    for seed in 0..games {
        let mut game = Game::new(Config {
            decks: [white_blue.clone(), black_green.clone()],
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
            orders += usize::from(matches!(action, Action::PutTrigger { .. }));
            optional += usize::from(matches!(action, Action::ChooseOptional { .. }));
            discards += usize::from(matches!(action, Action::Discard { .. }));
            game.apply(game.actor().unwrap(), action)?;
            total += 1;
        }
        if game.outcome().is_none() {
            return Err("self-play budget reached without terminal result".into());
        }
    }
    println!(
        "Completed {games} trigger games, {total} decisions, {orders} trigger-order choices, {optional} optional choices, {discards} discards; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
    Ok(())
}
