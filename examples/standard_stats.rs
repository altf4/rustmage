//! Supported Standard self-play with derived anthems/lords and temporary stats.
use rustmage::{Action, Card, Config, DeckRules, Game, Player, Zone};
fn deck(lands: &[(&str, usize)], spells: &[&str]) -> Vec<Card> {
    let mut deck = Vec::new();
    for &(name, count) in lands {
        deck.extend(vec![Card::from_name(name).unwrap(); count]);
    }
    for &name in spells {
        deck.extend([Card::from_name(name).unwrap(); 4]);
    }
    deck
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let games: usize = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "1".into())
        .parse()?;
    let white_green = deck(
        &[("Plains", 12), ("Forest", 12)],
        &[
            "Anthem of Champions",
            "Imperious Perfect",
            "Llanowar Elves",
            "Air Nomad Legacy",
            "Healer's Hawk",
            "On the Job",
            "Dawnwing Marshal",
            "Warren Elder",
            "Intrepid Tenderfoot",
        ],
    );
    let red_black = deck(
        &[("Mountain", 12), ("Swamp", 12)],
        &[
            "Goblin Oriflamme",
            "Shivan Dragon",
            "Sugar Rush",
            "Diresight",
            "Noxious Newt",
            "Burrog Banemaker",
            "Desolation Prowler",
            "Reckless Ransacking",
            "Highborn Vampire",
        ],
    );
    let mut total = 0;
    let mut activations = 0;
    let mut static_positions = 0;
    for seed in 0..games {
        let mut game = Game::new(Config {
            decks: [white_green.clone(), red_black.clone()],
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
            static_positions += usize::from(game.observe(actor).objects.iter().any(|o| {
                o.zone == Zone::Battlefield
                    && matches!(
                        o.card.name(),
                        "Anthem of Champions"
                            | "Imperious Perfect"
                            | "Air Nomad Legacy"
                            | "Goblin Oriflamme"
                    )
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
        "Completed {games} stats games, {total} decisions, {activations} activations, {static_positions} decisions with live static sources; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
    Ok(())
}
