//! Native Standard dynamic stats, independently checked public values and replay.
use rustmage::{Card, Config, DeckRules, Game, Player};
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
        &[("Plains", 10), ("Forest", 10), ("Temple Garden", 4)],
        &[
            "Burrowguard Mentor",
            "Regal Bunnicorn",
            "Crusader of Odric",
            "Mirkwood Pathmaker",
            "Harmonious Grovestrider",
            "Moon-Vigil Adherents",
            "Blanchwood Armor",
            "Ethereal Armor",
            "Adelbert Steiner",
        ],
    );
    let second = deck(
        &[
            ("Steam Vents", 4),
            ("Watery Grave", 4),
            ("Blood Crypt", 4),
            ("Overgrown Tomb", 4),
            ("Breeding Pool", 4),
            ("Island", 4),
        ],
        &[
            "Tarmogoyf",
            "Enigma Drake",
            "Tempest Djinn",
            "Xande, Dark Mage",
            "Boneclub Berserker",
            "Kraven, Proud Predator",
            "Cosmogoyf",
            "Shock",
            "Quick Study",
        ],
    );
    let (mut decisions, mut verified_stats, mut replays, mut max_bytes) = (0, 0, 0, 0);
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
            for object in &before.objects {
                if object.zone != rustmage::Zone::Battlefield
                    || object.modified
                    || before
                        .objects
                        .iter()
                        .any(|a| a.attached_to == Some(object.reference))
                {
                    continue;
                }
                let own = |o: &&rustmage::ObjectView| {
                    o.zone == rustmage::Zone::Battlefield && o.owner == object.owner
                };
                let expected =
                    match object.card.name() {
                        "Burrowguard Mentor" | "Crusader of Odric" => Some(
                            before
                                .objects
                                .iter()
                                .filter(own)
                                .filter(|o| o.card.is_creature())
                                .count() as i32,
                        ),
                        "Mirkwood Pathmaker" | "Harmonious Grovestrider" => Some(
                            before
                                .objects
                                .iter()
                                .filter(own)
                                .filter(|o| o.card.is_land())
                                .count() as i32,
                        ),
                        "Regal Bunnicorn" => Some(
                            before
                                .objects
                                .iter()
                                .filter(own)
                                .filter(|o| !o.card.is_land())
                                .count() as i32,
                        ),
                        "Tarmogoyf" => {
                            let mut types = std::collections::BTreeSet::new();
                            for o in before.objects.iter().filter(|o| {
                                o.zone == rustmage::Zone::Graveyard && !o.card.is_token()
                            }) {
                                for ty in o
                                    .card
                                    .info()
                                    .type_line
                                    .split(" — ")
                                    .next()
                                    .unwrap()
                                    .split_whitespace()
                                {
                                    if [
                                        "Artifact",
                                        "Battle",
                                        "Creature",
                                        "Enchantment",
                                        "Instant",
                                        "Kindred",
                                        "Land",
                                        "Planeswalker",
                                        "Sorcery",
                                    ]
                                    .contains(&ty)
                                    {
                                        types.insert(ty);
                                    }
                                }
                            }
                            Some(types.len() as i32)
                        }
                        _ => None,
                    };
                if let Some(expected) = expected {
                    assert_eq!(object.power, expected, "{}", object.card.name());
                    assert_eq!(
                        object.toughness,
                        expected + i32::from(object.card.name() == "Tarmogoyf")
                    );
                    verified_stats += 1;
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
        assert!(verified_stats > 0 && replays > 0);
    }
    println!(
        "Completed {games} supported Standard dynamic-stat games, {decisions} decisions, {verified_stats} independently checked public stat observations, {replays} cloned/replayed boundaries, {max_bytes} maximum initialized state bytes; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
    Ok(())
}
