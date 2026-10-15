//! Supported Standard self-play with dual lands and entry replacements.
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
    let white_blue = deck(
        &[
            ("Plains", 4),
            ("Azorius Guildgate", 4),
            ("Tranquil Cove", 4),
            ("Deserted Beach", 4),
            ("Temple of Enlightenment", 4),
            ("Meticulous Archive", 4),
        ],
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
        &[
            ("Plains", 4),
            ("Concealed Courtyard", 4),
            ("Scoured Barrens", 4),
            ("Shattered Sanctum", 4),
            ("Temple of Silence", 4),
            ("Shadowy Backstreet", 4),
        ],
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
    let mut entries = 0;
    let mut tapped = 0;
    let mut mana = 0;
    let mut choices = 0;
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
            choices += usize::from(view.library_choice.is_some());
            game.legal_actions_into(&mut actions);
            let action = actions[rustmage::policy::smoke_index(&game, &actions)].clone();
            mana += usize::from(matches!(
                action,
                Action::ActivateMana { .. } | Action::AnnounceMana { .. }
            ));
            let played = if let Action::PlayLand { card } = action {
                Some(card)
            } else {
                None
            };
            game.apply(actor, action)?;
            if let Some(id) = played {
                let view = game.observe(actor);
                let land = view.objects.iter().find(|o| o.id == id).unwrap();
                if !land.card.is_basic_land() {
                    entries += 1;
                    tapped += usize::from(land.tapped);
                }
            }
            total += 1;
        }
        if game.outcome().is_none() {
            return Err("self-play budget reached without terminal result".into());
        }
    }
    println!(
        "Completed {games} nonbasic-land games, {total} decisions, {entries} nonbasic land plays ({tapped} tapped), {mana} mana choices, {choices} library-choice decisions; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
    Ok(())
}
