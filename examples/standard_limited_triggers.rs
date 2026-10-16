//! Supported Standard trigger limits and typed entry observers with owned replay.
use rustmage::{Card, Config, DeckRules, Game, Player};
fn deck(lands: &[(&str, usize)], spells: &[&str]) -> Vec<Card> {
    let mut result = Vec::new();
    for &(name, n) in lands {
        result.extend(vec![Card::from_name(name).unwrap(); n]);
    }
    for &name in spells {
        let c = Card::from_name(name).unwrap();
        assert!(c.is_supported(), "{name}");
        result.extend([c; 4]);
    }
    assert_eq!(result.len(), 60);
    result
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let games: usize = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "1".into())
        .parse()?;
    let dimir = deck(
        &[("Island", 12), ("Swamp", 12)],
        &[
            "Flying Octobot",
            "Common Crook",
            "Mechan Assembler",
            "Hostile Investigator",
            "Homicide Investigator",
            "Friendly Teddy",
            "Donatello, Way with Machines",
            "Fateful Discovery",
            "Scrawling Crawler",
        ],
    );
    let selesnya = deck(
        &[("Plains", 12), ("Forest", 12)],
        &[
            "Bloombrute",
            "Elvish Archivist",
            "Youthful Valkyrie",
            "Avatar Enthusiasts",
            "Grasping Longneck",
            "Fear of Surveillance",
            "Serra Angel",
            "Glider Kids",
            "Reclamation Sage",
        ],
    );
    let (mut decisions, mut consumed, mut replays, mut resets, mut max_bytes) = (0, 0, 0, 0, 0);
    for seed in 0..games {
        let mut g = Game::new(Config {
            decks: [dimir.clone(), selesnya.clone()],
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
            let action = actions[rustmage::policy::smoke_index(&g, &actions)].clone();
            let saved = g.snapshot();
            g.apply(actor, action.clone())?;
            let after = g.observe(g.actor().unwrap_or(actor));
            if before.turn != after.turn {
                resets += 1;
                consumed += after.triggered_this_turn.len();
            } else {
                consumed += after
                    .triggered_this_turn
                    .iter()
                    .filter(|u| !before.triggered_this_turn.contains(u))
                    .count();
            }
            if before.turn != after.turn
                || before.triggered_this_turn != after.triggered_this_turn
                || before.announcing.is_some()
            {
                let expected = g.snapshot();
                g.restore(&saved).unwrap();
                let mut fork = g.clone();
                g.apply(actor, action.clone())?;
                fork.apply(actor, action)?;
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
        assert!(consumed > 0 && replays > 0 && resets > 0);
    }
    println!(
        "Completed {games} supported Standard games, {decisions} decisions, {consumed} consumed trigger limits, {resets} turn transitions, {replays} cloned/replayed boundaries, {max_bytes} maximum initialized state bytes; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
    Ok(())
}
