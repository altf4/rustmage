//! Native Standard continuous grants, mana planning, combat and replay.
use rustmage::{Action, Card, Config, DeckRules, Game, Keywords, Player};
fn deck(lands: &[(&str, usize)], spells: &[&str]) -> Vec<Card> {
    let mut cards = Vec::new();
    for &(name, n) in lands {
        cards.extend(vec![Card::from_name(name).unwrap(); n]);
    }
    for &name in spells {
        let c = Card::from_name(name).unwrap();
        assert!(c.is_supported(), "{name}");
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
        &[("Plains", 12), ("Forest", 12)],
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
    let gruul = deck(
        &[("Mountain", 12), ("Forest", 12)],
        &[
            "Samut, Hazoret's Champion",
            "Feisty Spikeling",
            "Boldwyr Aggressor",
            "Skilled Battlecarver",
            "Null Group Biological Assets",
            "Aggressive Mammoth",
            "Three Tree Mascot",
            "Llanowar Elves",
            "Fire Elemental",
        ],
    );
    let (mut decisions, mut entries, mut haste_mana, mut replays, mut max_bytes) = (0, 0, 0, 0, 0);
    for seed in 0..games {
        let mut g = Game::new(Config {
            decks: [selesnya.clone(), gruul.clone()],
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
            if matches!(&a,Action::TapMana{card} | Action::ActivateMana{card,..} if before.objects.iter().any(|o|o.id==*card && o.summoning_sick && o.keywords.contains(Keywords::HASTE)))
            {
                haste_mana += 1;
            }
            let saved = g.snapshot();
            g.apply(actor, a.clone())?;
            let after = g.observe(g.actor().unwrap_or(actor));
            entries += after
                .objects
                .iter()
                .filter(|o| {
                    o.zone == rustmage::Zone::Battlefield
                        && !o.card.keywords().contains(Keywords::HASTE)
                        && o.keywords.contains(Keywords::HASTE)
                        && !before.objects.iter().any(|b| {
                            b.zone == rustmage::Zone::Battlefield && b.reference == o.reference
                        })
                })
                .count();
            if before.announcing.is_some()
                || before.search_choice.is_some()
                || after.search_choice.is_some()
                || before.decision == "optional_effect"
                || after.decision == "optional_effect"
            {
                let expected = g.snapshot();
                g.restore(&saved);
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
        assert!(entries > 0 && haste_mana > 0 && replays > 0);
    }
    println!(
        "Completed {games} supported Standard continuous-keyword games, {decisions} decisions, {entries} observed entries with continuously granted haste, {haste_mana} sick mana activations, {replays} cloned/replayed boundaries, {max_bytes} maximum initialized state bytes; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
    Ok(())
}
