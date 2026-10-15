//! Supported Standard self-play exercising activated costs, counters and sacrificed-source abilities.
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
            "Brave-Kin Duo",
            "Cathar Commando",
            "Envoy of Okinec Ahau",
            "Moonlit Lamenter",
            "Intrepid Tenderfoot",
            "Treetop Snarespinner",
            "Guardian of the Halls",
            "Thrashing Brontodon",
            "Inspiring Overseer",
        ],
    );
    let blue_green = deck(
        &[("Swamp", 12), ("Mountain", 12)],
        &[
            "Fanatical Firebrand",
            "Deadeye Duelist",
            "Vampire Neonate",
            "Valgavoth's Faithful",
            "Gnarlbark Elm",
            "Shattered Acolyte",
            "Harried Spearguard",
            "Elder Auntie",
            "Blight Rot",
        ],
    );
    let mut total = 0;
    let mut targets = 0;
    let mut multi = 0;
    let mut activations = 0;

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
                Action::Activate { .. } | Action::BeginActivate { .. }
            ));
            game.apply(game.actor().unwrap(), action)?;
            total += 1;
        }
        if game.outcome().is_none() {
            return Err("self-play budget reached without terminal result".into());
        }
    }
    println!(
        "Completed {games} activation games, {total} decisions, {targets} target choices, {multi} multi-target casts, {activations} activations; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
    Ok(())
}
