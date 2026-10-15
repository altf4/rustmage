//! Supported Standard self-play with menace and blocking restrictions.
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
    let red_black = deck(
        &[("Swamp", 12), ("Mountain", 12)],
        &[
            "Ravine Raider",
            "Edgewall Pack",
            "Dinotomaton",
            "Flame-Chain Mauler",
            "Agents of HYDRA",
            "Vampire Interloper",
            "Vampire Soulcaller",
            "Kindled Fury",
            "Fanatical Firebrand",
        ],
    );
    let blue_green = deck(
        &[("Island", 12), ("Forest", 12)],
        &[
            "Rampaging Ceratops",
            "Elvenking's Harper",
            "Enter the Enigma",
            "Shore Lurker",
            "Sterling Hound",
            "Llanowar Elves",
            "Bear Cub",
            "Unsummon",
            "Overrun",
        ],
    );
    let mut total = 0;
    let mut restricted = 0;
    let mut blocks = 0;
    let mut activations = 0;
    for seed in 0..games {
        let mut game = Game::new(Config {
            decks: [red_black.clone(), blue_green.clone()],
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
            restricted += usize::from(game.observe(actor).objects.iter().any(|o| {
                o.zone == Zone::Battlefield
                    && (o.keywords.contains(Keywords::MENACE)
                        || o.keywords.contains(Keywords::MIN_THREE_BLOCKERS))
            }));
            game.legal_actions_into(&mut actions);
            let action = actions[rustmage::policy::smoke_index(&game, &actions)].clone();
            blocks += usize::from(matches!(action, Action::Block { .. }));
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
        "Completed {games} blocking games, {total} decisions, {activations} activations, {restricted} decisions with minimum-blocker restrictions, {blocks} selected blockers; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
    Ok(())
}
