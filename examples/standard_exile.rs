//! Supported Standard self-play with face-up exile and linked returns.
use rustmage::{Action, Card, Config, DeckRules, Game, Player, Zone};
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
    let white_blue = deck(
        &[("Plains", 12), ("Island", 12)],
        &[
            "Banishing Light",
            "Glass Casket",
            "Stasis Snare",
            "White Auracite",
            "Soul-Guide Lantern",
            "Helpful Hunter",
            "Dazzling Angel",
            "Unsummon",
            "Disenchant",
        ],
    );
    let white_black = deck(
        &[("Plains", 12), ("Swamp", 12)],
        &[
            "Makeshift Binding",
            "Seam Rip",
            "Food Coma",
            "Angel of Finality",
            "Epic Downfall",
            "Magic Pot",
            "Agents of HYDRA",
            "Highborn Vampire",
            "Disenchant",
        ],
    );
    let mut total = 0;
    let mut exiled = 0;
    let mut linked = 0;
    let mut targets = 0;
    for seed in 0..games {
        let mut game = Game::new(Config {
            decks: [white_blue.clone(), white_black.clone()],
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
            exiled += usize::from(view.objects.iter().any(|o| o.zone == Zone::Exile));
            linked += usize::from(!view.exile_links.is_empty());
            game.legal_actions_into(&mut actions);
            let action = actions[rustmage::policy::smoke_index(&game, &actions)].clone();
            targets += usize::from(matches!(action, Action::ChooseTarget { .. }));
            game.apply(actor, action)?;
            total += 1;
        }
        if game.outcome().is_none() {
            return Err("self-play budget reached without terminal result".into());
        }
    }
    println!(
        "Completed {games} exile games, {total} decisions, {targets} target choices, {exiled} decisions with exiled objects, {linked} decisions with linked returns; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
    Ok(())
}
