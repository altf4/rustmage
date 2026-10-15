//! Supported Standard self-play exercising mana production and predefined token abilities.
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
            "Novice Inspector",
            "Oasis Gardener",
            "Great Forest Druid",
            "Crustacean Commando",
            "Noxious Newt",
            "Magnifying Glass",
            "Hedron Archive",
            "Inspiring Overseer",
            "Intrepid Tenderfoot",
        ],
    );
    let blue_green = deck(
        &[("Swamp", 12), ("Mountain", 12)],
        &[
            "Plundering Pirate",
            "Rapacious Dragon",
            "Treasure Dredger",
            "Mintstrosity",
            "Savor",
            "Bake into a Pie",
            "Piggy Bank",
            "Collector's Vault",
            "Fanatical Firebrand",
        ],
    );
    let mut total = 0;
    let mut targets = 0;
    let mut multi = 0;
    let mut activations = 0;
    let mut payments = 0;

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
            activations += usize::from(matches!(
                action,
                Action::Activate { .. }
                    | Action::BeginActivate { .. }
                    | Action::ActivateMana { .. }
            ));
            payments += usize::from(matches!(action, Action::PayCosts { .. }));
            game.apply(game.actor().unwrap(), action)?;
            total += 1;
        }
        if game.outcome().is_none() {
            return Err("self-play budget reached without terminal result".into());
        }
    }
    println!(
        "Completed {games} mana/token games, {total} decisions, {targets} target choices, {multi} multi-target casts, {activations} activations, {payments} payments; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
    Ok(())
}
