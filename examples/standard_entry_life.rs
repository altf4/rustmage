//! Native Standard shock-land entry choices, mana, combat and owned replay.
use rustmage::{Action, Card, Config, DeckRules, Game, Player};
fn deck(lands: &[(&str, usize)], spells: &[&str]) -> Vec<Card> {
    let mut cards = Vec::new();
    for &(name, n) in lands {
        cards.extend(vec![Card::from_name(name).unwrap(); n]);
    }
    for &name in spells {
        let c = Card::from_name(name).unwrap();
        assert!(c.is_supported() && c.is_standard_legal(), "{name}");
        cards.extend([c; 4]);
    }
    assert_eq!(cards.len(), 60);
    cards
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let games: usize = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "1".into())
        .parse()?;
    let selesnya = deck(
        &[
            ("Temple Garden", 4),
            ("Overgrown Tomb", 4),
            ("Godless Shrine", 4),
            ("Sacred Foundry", 4),
            ("Breeding Pool", 4),
            ("Vigorbloom Annex", 4),
        ],
        &[
            "Regal Caracal",
            "Bearer of Glory",
            "Okoye, Dora Milaje Leader",
            "Aggressive Mammoth",
            "Llanowar Elves",
            "Druid of the Cowl",
            "Three Tree Mascot",
            "Changeling Wayfinder",
            "Savannah Lions",
        ],
    );
    let izzet = deck(
        &[
            ("Blood Crypt", 4),
            ("Watery Grave", 4),
            ("Hallowed Fountain", 4),
            ("Steam Vents", 4),
            ("Stomping Ground", 4),
            ("Innovative Commons", 4),
        ],
        &[
            "Null Group Biological Assets",
            "Skilled Battlecarver",
            "Fire Elemental",
            "Shock",
            "Lightning Strike",
            "Opt",
            "Quick Study",
            "Mischievous Mystic",
            "Strix Lookout",
        ],
    );
    let (mut decisions, mut paid, mut declined, mut replays, mut max_bytes) = (0, 0, 0, 0, 0);
    for seed in 0..games {
        let mut g = Game::new(Config {
            decks: [selesnya.clone(), izzet.clone()],
            seed: seed as u64,
            starting_player: Player::P0,
            deck_rules: DeckRules::Standard,
        })?;
        let mut actions = Vec::new();
        for _ in 0..100_000 {
            if g.outcome().is_some() {
                break;
            }
            let actor = g.actor().unwrap();
            let before = g.observe(actor);
            g.legal_actions_into(&mut actions);
            let a = actions[rustmage::policy::smoke_index(&g, &actions)].clone();
            if let Action::ChooseEntryLife { pay } = &a {
                if *pay {
                    paid += 1;
                } else {
                    declined += 1;
                }
            }
            let saved = g.snapshot();
            g.apply(actor, a.clone())?;
            let after = g.observe(g.actor().unwrap_or(actor));
            if before.entry_life.is_some()
                || after.entry_life.is_some()
                || before.announcing.is_some()
                || before.search_choice.is_some()
                || after.search_choice.is_some()
                || before.decision == "optional_effect"
                || after.decision == "optional_effect"
            {
                let expected = g.snapshot();
                g.restore(&saved).unwrap();
                let mut fork = g.clone();
                g.apply(actor, a.clone())?;
                fork.apply(actor, a)?;
                assert_eq!(g.snapshot(), expected);
                assert_eq!(fork.snapshot(), expected);
                replays += 1;
            }
            decisions += 1;
            max_bytes = max_bytes.max(g.state_bytes());
        }
        assert!(g.outcome().is_some(), "game {seed} did not finish");
    }
    if games >= 20 {
        assert!(paid > 0 && declined > 0 && replays > 0);
    }
    println!(
        "Completed {games} supported Standard pre-entry-life games, {decisions} decisions, {paid} paid entries, {declined} declined payments, {replays} cloned/replayed boundaries, {max_bytes} maximum initialized state bytes; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
    Ok(())
}
