//! Supported Standard self-play with private searches, fetches and Lander tokens.
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
        &[
            ("Forest", 16),
            ("Evolving Wilds", 4),
            ("Terramorphic Expanse", 4),
        ],
        &[
            "Galactic Wayfarer",
            "Burnished Hart",
            "Shared Roots",
            "They Went This Way",
            "Dauntless Scrapbot",
            "Sami's Curiosity",
            "World Map",
            "Attercop",
            "Rampaging Baloths",
        ],
    );
    let black_green = deck(
        &[("Forest", 10), ("Swamp", 10), ("Vibrant Cityscape", 4)],
        &[
            "Beamsaw Prospector",
            "Galactic Wayfarer",
            "Burnished Hart",
            "Shared Roots",
            "They Went This Way",
            "World Map",
            "Highborn Vampire",
            "Agents of HYDRA",
            "Eumidian Terrabotanist",
        ],
    );
    let mut total = 0;
    let mut searches = 0;
    let mut selections = 0;
    let mut landers = 0;
    let mut known = 0;
    for seed in 0..games {
        let mut game = Game::new(Config {
            decks: [green.clone(), black_green.clone()],
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
            let view = game.observe(actor);
            landers += usize::from(view.objects.iter().any(|o| {
                o.card
                    .token_info()
                    .is_some_and(|t| t.name == "Lander Token")
            }));
            known += usize::from(!view.known_hand.is_empty());
            game.legal_actions_into(&mut actions);
            let action = actions[rustmage::policy::smoke_index(&game, &actions)].clone();
            searches += usize::from(matches!(action, Action::FinishSearch));
            selections += usize::from(matches!(action, Action::ChooseSearch { .. }));
            game.apply(actor, action)?;
            total += 1;
        }
        if game.outcome().is_none() {
            return Err("self-play budget reached without terminal result".into());
        }
    }
    println!(
        "Completed {games} search games, {total} decisions, {searches} searches, {selections} selections, {landers} decisions with Lander tokens, {known} decisions with revealed hand knowledge; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
    Ok(())
}
