//! Supported Standard self-play through explicit mode/target/payment decisions.
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
    let red_green = deck(
        &[("Mountain", 12), ("Forest", 12)],
        &[
            "Abrade",
            "Goblin Surprise",
            "Mouser Attack!",
            "Attercop",
            "Rampaging Baloths",
            "Elfsworn Giant",
            "Eumidian Terrabotanist",
            "Galactic Wayfarer",
            "Reclamation Sage",
        ],
    );
    let white_blue = deck(
        &[("Plains", 12), ("Island", 12)],
        &[
            "School Daze",
            "Spellgyre",
            "Family Reunion",
            "Thorin's Last Stand",
            "Helpful Hunter",
            "Dazzling Angel",
            "Inspiring Overseer",
            "Nimble Thopterist",
            "Rook Turret",
        ],
    );
    let mut total = 0;
    let mut modes = 0;
    let mut first = 0;
    let mut second = 0;
    let mut targets = 0;
    let mut payments = 0;
    let mut library_choices = 0;
    for seed in 0..games {
        let mut game = Game::new(Config {
            decks: [red_green.clone(), white_blue.clone()],
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
            library_choices += usize::from(view.library_choice.is_some());
            game.legal_actions_into(&mut actions);
            let action = actions[rustmage::policy::smoke_index(&game, &actions)].clone();
            if let Action::ChooseMode { mode } = action {
                modes += 1;
                first += usize::from(mode == 0);
                second += usize::from(mode == 1);
            }
            targets += usize::from(matches!(action, Action::ChooseTarget { .. }));
            payments += usize::from(matches!(action, Action::PayCosts { .. }));
            game.apply(actor, action)?;
            total += 1;
        }
        if game.outcome().is_none() {
            return Err("self-play budget reached without terminal result".into());
        }
    }
    println!(
        "Completed {games} modal games, {total} decisions, {modes} mode choices ({first} first/{second} second), {targets} target choices, {payments} staged payments, {library_choices} library-choice decisions; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
    Ok(())
}
