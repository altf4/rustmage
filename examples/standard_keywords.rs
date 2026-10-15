//! Supported Standard self-play with effective temporary keyword gains.
use rustmage::{Action, Card, Config, DeckRules, Game, Keywords, Player, Zone};
fn deck(lands: &[(&str, usize)], spells: &[&str]) -> Vec<Card> {
    let mut cards = Vec::new();
    for &(name, n) in lands {
        cards.extend(vec![Card::from_name(name).unwrap(); n]);
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
    let white_green = deck(
        &[("Plains", 12), ("Forest", 12)],
        &[
            "Llanowar Elves",
            "Overprotect",
            "Horrid Vigor",
            "Make a Stand",
            "Healer's Hawk",
            "Anthem of Champions",
            "Leaping Ambush",
            "Wildheart Invoker",
            "Dawnwing Marshal",
        ],
    );
    let blue_red = deck(
        &[("Island", 12), ("Mountain", 12)],
        &[
            "Axgard Cavalry",
            "Ancestors' Aid",
            "Shore Up",
            "Impolite Entrance",
            "Kindled Fury",
            "Shore Lurker",
            "Unsummon",
            "Shivan Dragon",
            "Terror of Mount Velus",
        ],
    );
    let mut total = 0;
    let mut protected = 0;
    let mut activations = 0;
    for seed in 0..games {
        let mut game = Game::new(Config {
            decks: [white_green.clone(), blue_red.clone()],
            seed: seed as u64,
            starting_player: Player::P0,
            deck_rules: DeckRules::Standard,
        })?;
        let mut actions = Vec::new();
        for _ in 0..100_000 {
            if game.outcome().is_some() {
                break;
            }
            let actor = game.actor().unwrap();
            protected += usize::from(game.observe(actor).objects.iter().any(|o| {
                o.zone == Zone::Battlefield
                    && (o.keywords.contains(Keywords::HEXPROOF)
                        || o.keywords.contains(Keywords::INDESTRUCTIBLE))
            }));
            game.legal_actions_into(&mut actions);
            let action = actions[rustmage::policy::smoke_index(&game, &actions)].clone();
            activations += usize::from(matches!(
                action,
                Action::Activate { .. } | Action::BeginActivate { .. }
            ));
            game.apply(actor, action)?;
            total += 1;
        }
        if game.outcome().is_none() {
            return Err("self-play budget reached without terminal result".into());
        }
    }
    println!(
        "Completed {games} keyword games, {total} decisions, {activations} activations, {protected} decisions with hexproof/indestructible; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
    Ok(())
}
