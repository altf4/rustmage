//! Supported Standard self-play with cloned and replayed legend choices.
use rustmage::{Action, Card, Config, DeckRules, Game, Player};
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
    let green = deck(
        &[("Forest", 12), ("Island", 12)],
        &[
            "Tatyova, Benthic Druid",
            "Caelorna, Coral Tyrant",
            "Llanowar Elves",
            "Druid of the Cowl",
            "Shared Roots",
            "Rampaging Baloths",
            "Bushwhack",
            "Sami's Curiosity",
            "Giant Growth",
        ],
    );
    let blue_red = green.clone();
    let mut total = 0;
    let mut legends = 0;
    for seed in 0..games {
        let mut game = Game::new(Config {
            decks: [green.clone(), blue_red.clone()],
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
            game.legal_actions_into(&mut actions);
            let action = actions[rustmage::policy::smoke_index(&game, &actions)].clone();
            if matches!(action, Action::KeepLegend { .. }) {
                legends += 1;
                let saved = game.snapshot();
                let mut fork = game.clone();
                fork.apply(actor, action.clone())?;
                game.apply(actor, action.clone())?;
                assert_eq!(game.snapshot(), fork.snapshot());
                let expected = game.snapshot();
                game.restore(&saved);
                game.apply(actor, action)?;
                assert_eq!(game.snapshot(), expected);
                total += 1;
                continue;
            }
            game.apply(actor, action)?;
            total += 1;
        }
        if game.outcome().is_none() {
            return Err("self-play budget reached without terminal result".into());
        }
    }
    assert!(legends > 0, "no legend choices exercised");
    println!(
        "Completed {games} legend games, {total} decisions, {legends} cloned and replayed legend choices; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
    Ok(())
}
