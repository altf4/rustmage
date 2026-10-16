//! Native Standard guarded mana, prerequisite changes, combat and owned replay.
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
    let first = deck(
        &[
            ("Blazemire Verge", 4),
            ("Bleachbone Verge", 4),
            ("Floodfarm Verge", 4),
            ("Gloomlake Verge", 4),
            ("Hushwood Verge", 4),
            ("Godless Shrine", 4),
        ],
        &[
            "Mox Jasper",
            "Three Tree Mascot",
            "Prideful Feastling",
            "Feisty Spikeling",
            "Mary Jane Watson",
            "Youthful Valkyrie",
            "Flock Impostor",
            "Rooftop Percher",
            "Regal Caracal",
        ],
    );
    let second = deck(
        &[
            ("Riverpyre Verge", 4),
            ("Sunbillow Verge", 4),
            ("Thornspire Verge", 4),
            ("Wastewood Verge", 4),
            ("Willowrush Verge", 4),
            ("Stomping Ground", 4),
        ],
        &[
            "Mox Jasper",
            "Three Tree Mascot",
            "Gangly Stompling",
            "Chitinous Graspling",
            "Samut, Hazoret's Champion",
            "Skilled Battlecarver",
            "Null Group Biological Assets",
            "Boldwyr Aggressor",
            "Lightning Strike",
        ],
    );
    let (mut decisions, mut verge_mana, mut jasper_mana, mut replays, mut max_bytes) =
        (0, 0, 0, 0, 0);
    for seed in 0..games {
        let mut g = Game::new(Config {
            decks: [first.clone(), second.clone()],
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
            let mut guarded = false;
            if let Action::ActivateMana { card, ability, .. } = &a {
                let name = before
                    .objects
                    .iter()
                    .find(|o| o.id == *card)
                    .unwrap()
                    .card
                    .name();
                if name == "Mox Jasper" {
                    jasper_mana += 1;
                    guarded = true;
                } else if name.ends_with(" Verge") && *ability == 1 {
                    verge_mana += 1;
                    guarded = true;
                }
            }
            let saved = g.snapshot();
            g.apply(actor, a.clone())?;
            let after = g.observe(g.actor().unwrap_or(actor));
            if guarded
                || before.entry_life.is_some()
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
        assert!(verge_mana > 0 && jasper_mana > 0 && replays > 0);
    }
    println!(
        "Completed {games} supported Standard guarded-mana games, {decisions} decisions, {verge_mana} secondary Verge mana activations, {jasper_mana} Jasper mana activations, {replays} cloned/replayed boundaries, {max_bytes} maximum initialized state bytes; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
    Ok(())
}
