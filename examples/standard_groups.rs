//! Supported Standard self-play through distinct target groups and noncombat damage.
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
        &[("Forest", 24)],
        &[
            "Bushwhack",
            "Kapow!",
            "Quarrel",
            "Huatli's Final Strike",
            "Troll Negotiations",
            "Attercop",
            "Elfsworn Giant",
            "Galactic Wayfarer",
            "Rampaging Baloths",
        ],
    );
    let blue_red = deck(
        &[("Island", 12), ("Mountain", 12)],
        &[
            "Giantfall",
            "Vibrant Outburst",
            "Rabid Gnaw",
            "Combat Tutorial",
            "Quaketusk Boar",
            "Nimble Thopterist",
            "Rook Turret",
            "Dragon Fodder",
            "S.H.I.E.L.D. Deployment Drone",
        ],
    );
    let mut total = 0;
    let mut groups = 0;
    let mut first = 0;
    let mut second = 0;
    let mut pairs = 0;
    let mut modes = 0;
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
            let view = game.observe(actor);
            pairs += usize::from(
                view.stack
                    .iter()
                    .any(|item| item.targets.iter().any(|bound| bound.group == 1)),
            );
            game.legal_actions_into(&mut actions);
            let action = actions[rustmage::policy::smoke_index(&game, &actions)].clone();
            modes += usize::from(matches!(action, Action::ChooseMode { .. }));
            if matches!(action, Action::ChooseTarget { .. }) {
                groups += 1;
                first += usize::from(view.targeting_group == Some(0));
                second += usize::from(view.targeting_group == Some(1));
            }
            game.apply(actor, action)?;
            total += 1;
        }
        if game.outcome().is_none() {
            return Err("self-play budget reached without terminal result".into());
        }
    }
    println!(
        "Completed {games} target-group games, {total} decisions, {groups} target choices ({first} first/{second} second), {modes} mode choices, {pairs} decisions with distinct groups on stack; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
    Ok(())
}
