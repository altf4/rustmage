//! Supported Standard self-play exercising private scry/surveil continuations.
use rustmage::{Action, Card, Config, DeckRules, Game, LibraryDestination, Player};

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
    let blue = deck(
        &[("Island", 16), ("Plains", 8)],
        &[
            "Opt",
            "Sterling Hound",
            "Sanitation Automaton",
            "Shore Lurker",
            "Unauthorized Exit",
            "Wakandan Drone Flock",
            "Candy Trail",
            "Rune-Sealed Wall",
            "Il Mheg Pixie",
        ],
    );
    let black = deck(
        &[("Swamp", 24)],
        &[
            "Diresight",
            "Wretched Doll",
            "Wreckage Wickerfolk",
            "Imperious Inkmage",
            "Hour of Defeat",
            "Sterling Hound",
            "Candy Trail",
            "Sanitation Automaton",
            "Highborn Vampire",
        ],
    );
    let mut total = 0;
    let mut looks = 0;
    let mut bottomed = 0;
    let mut buried = 0;
    for seed in 0..games {
        let mut game = Game::new(Config {
            decks: [blue.clone(), black.clone()],
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
            let default = rustmage::policy::smoke_index(&game, &actions);
            // Alternate destinations using only the acting player's legal choices.
            let index = if looks % 2 == 0 {
                actions
                    .iter()
                    .position(|a| {
                        matches!(
                            a,
                            Action::ArrangeLibrary {
                                destination: LibraryDestination::Bottom
                                    | LibraryDestination::Graveyard,
                                ..
                            }
                        )
                    })
                    .unwrap_or(default)
            } else {
                default
            };
            let action = actions[index].clone();
            looks += usize::from(matches!(action, Action::ArrangeLibrary { .. }));
            bottomed += usize::from(matches!(
                action,
                Action::ArrangeLibrary {
                    destination: LibraryDestination::Bottom,
                    ..
                }
            ));
            buried += usize::from(matches!(
                action,
                Action::ArrangeLibrary {
                    destination: LibraryDestination::Graveyard,
                    ..
                }
            ));
            game.apply(game.actor().unwrap(), action)?;
            total += 1;
        }
        if game.outcome().is_none() {
            return Err("self-play budget reached without terminal result".into());
        }
    }
    println!(
        "Completed {games} library-choice games, {total} decisions, {looks} library selections, {bottomed} bottomed, {buried} buried; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
    Ok(())
}
