//! Independent public batch checks on complete supported Standard decks.
use rustmage::{
    Action, Card, Config, DeckRules, Game, Keywords, ObjectView, Observation, Player, SpellView,
    StackKind, Zone,
};
fn deck(lands: &[(&str, usize)], spells: &[&str]) -> Vec<Card> {
    let mut d = Vec::new();
    for &(name, n) in lands {
        d.extend(vec![Card::from_name(name).unwrap(); n]);
    }
    for &name in spells {
        let c = Card::from_name(name).unwrap();
        assert!(c.is_supported() && c.is_standard_legal(), "{name}");
        d.extend([c; 4]);
    }
    assert_eq!(d.len(), 60);
    d
}
fn artifact(o: &ObjectView) -> bool {
    o.card
        .info()
        .type_line
        .split(" — ")
        .next()
        .unwrap()
        .split_whitespace()
        .any(|s| s == "Artifact")
}
fn enchantment(o: &ObjectView) -> bool {
    o.card
        .info()
        .type_line
        .split(" — ")
        .next()
        .unwrap()
        .split_whitespace()
        .any(|s| s == "Enchantment")
}
fn mana(o: &ObjectView) -> u64 {
    o.card
        .info()
        .mana_cost
        .split('{')
        .skip(1)
        .map(|s| s.trim_end_matches('}').parse::<u64>().unwrap_or(1))
        .sum()
}
fn check(before: &Observation, after: &Observation, item: &SpellView) -> usize {
    if item.kind != StackKind::Spell {
        return 0;
    }
    let name = item.source_card.name();
    let mode = item.mode;
    let moved = matches!(
        name,
        "Day of Judgment"
            | "Fumigate"
            | "Death Begets Life"
            | "Beyond the Quiet"
            | "Pest Control"
            | "Split Up"
    ) || name == "Ruinous Rampage" && mode == Some(1);
    let damage = match (name, mode) {
        ("Pyroclasm" | "Fuel the Flames" | "Seismic Rupture", _) => Some(2),
        ("Slagstorm", Some(0)) => Some(3),
        ("Tectonic Hazard", _) => Some(1),
        _ => None,
    };
    let mut checks = 0;
    let mut count = 0;
    for old in before
        .objects
        .iter()
        .filter(|o| o.zone == Zone::Battlefield)
    {
        let affected = match (name, mode) {
            (
                "Day of Judgment" | "Fumigate" | "Beyond the Quiet" | "Pyroclasm"
                | "Fuel the Flames",
                _,
            ) => old.card.is_creature(),
            ("Death Begets Life", _) => old.card.is_creature() || enchantment(old),
            ("Pest Control", _) => !old.card.is_land() && mana(old) <= 1,
            ("Split Up", Some(m)) => old.card.is_creature() && old.tapped == (m == 0),
            ("Ruinous Rampage", Some(1)) => artifact(old) && mana(old) <= 3,
            ("Seismic Rupture", _) => {
                old.card.is_creature() && !old.keywords.contains(Keywords::FLYING)
            }
            ("Tectonic Hazard", _) => old.card.is_creature() && old.owner != item.controller,
            ("Slagstorm", Some(0)) => old.card.is_creature(),
            _ => false,
        };
        if !affected {
            continue;
        }
        if moved {
            let exile = name == "Beyond the Quiet" || name == "Ruinous Rampage";
            if !exile && old.keywords.contains(Keywords::INDESTRUCTIBLE) {
                continue;
            }
            count += 1;
            let current = after.objects.iter().find(|o| o.id == old.id);
            if old.token.is_some() {
                assert!(current.is_none());
            } else {
                let current = current.unwrap();
                assert_ne!(current.reference, old.reference);
                assert_eq!(
                    current.zone,
                    if exile { Zone::Exile } else { Zone::Graveyard },
                    "{name}"
                );
            }
            checks += 1;
        } else if let Some(amount) = damage {
            if old.damage + amount >= old.toughness
                && !old.keywords.contains(Keywords::INDESTRUCTIBLE)
            {
                let current = after.objects.iter().find(|o| o.id == old.id);
                if old.token.is_some() {
                    assert!(current.is_none());
                } else {
                    let current = current.unwrap();
                    assert_ne!(current.reference, old.reference);
                    assert_eq!(current.zone, Zone::Graveyard, "{name}");
                }
                checks += 1;
                continue;
            }
            let current = after.objects.iter().find(|o| {
                o.id == old.id && o.reference == old.reference && o.zone == Zone::Battlefield
            });
            if let Some(current) = current {
                assert_eq!(current.damage, old.damage + amount, "{name}");
                assert!(current.damaged_this_turn);
                checks += 1;
            }
        }
    }
    let p = item.controller.index();
    if name == "Fumigate" {
        assert_eq!(after.players[p].life, before.players[p].life + count);
        assert_eq!(
            after.players[p].life_gain_events_this_turn,
            before.players[p].life_gain_events_this_turn + u64::from(count > 0)
        );
        checks += 2;
    }
    if name == "Death Begets Life" {
        assert_eq!(
            after.players[p].cards_drawn_this_turn,
            before.players[p].cards_drawn_this_turn
                + (count as u64).min(before.players[p].library_count as u64)
        );
        checks += 1;
    }
    let all_players = name == "Cast Away Doubt" || name == "Slagstorm" && mode == Some(1);
    if all_players || name == "Tectonic Hazard" || name == "Ruinous Rampage" && mode == Some(0) {
        let amount = if name == "Tectonic Hazard" {
            1
        } else if name == "Cast Away Doubt" {
            2
        } else {
            3
        };
        for i in 0..2 {
            assert_eq!(
                after.players[i].life,
                before.players[i].life - if all_players || i != p { amount } else { 0 }
            );
            checks += 1;
        }
    }
    checks
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let games = std::env::args()
        .nth(1)
        .map(|s| s.parse())
        .transpose()?
        .unwrap_or(100);
    let white = deck(
        &[("Plains", 12), ("Forest", 12)],
        &[
            "Bear Cub",
            "Cat-Owl",
            "Grasping Longneck",
            "Common Crook",
            "Zetalpa, Primal Dawn",
            "Fumigate",
            "Day of Judgment",
            "Split Up",
            "Beyond the Quiet",
        ],
    );
    let black_red = deck(
        &[("Mountain", 12), ("Swamp", 12)],
        &[
            "Gleaming Barrier",
            "Midnight Reaper",
            "Rooftop Assassin",
            "Harried Spearguard",
            "Pyroclasm",
            "Fuel the Flames",
            "Tectonic Hazard",
            "Slagstorm",
            "Ruinous Rampage",
        ],
    );
    let four_color = deck(
        &[("Plains", 6), ("Island", 6), ("Swamp", 6), ("Forest", 6)],
        &[
            "Bear Cub",
            "Grasping Longneck",
            "Midnight Reaper",
            "Phyrexian Arena",
            "Mox Jasper",
            "Death Begets Life",
            "Pest Control",
            "Extended Absence",
            "Cast Away Doubt",
        ],
    );
    let red = deck(
        &[("Mountain", 24)],
        &[
            "Harried Spearguard",
            "Fanatical Firebrand",
            "Swab Goblin",
            "Seismic Rupture",
            "Ruinous Rampage",
            "Slagstorm",
            "Fuel the Flames",
            "Tectonic Hazard",
            "Pyroclasm",
        ],
    );
    let (mut decisions, mut checks, mut replays, mut max_bytes) = (0, 0, 0, 0);
    let mut resolved = std::collections::BTreeSet::new();
    for seed in 0..games {
        let decks = if seed % 2 == 0 {
            [white.clone(), black_red.clone()]
        } else {
            [four_color.clone(), red.clone()]
        };
        let mut g = Game::new(Config {
            decks,
            seed: seed as u64,
            starting_player: Player::P0,
            deck_rules: DeckRules::Standard,
        })?;
        for _ in 0..100_000 {
            if g.outcome().is_some() {
                break;
            }
            let actor = g.actor().unwrap();
            let before = g.observe(actor);
            let actions = g.legal_actions();
            let action = actions[rustmage::policy::smoke_index(&g, &actions)].clone();
            let saved = g.snapshot();
            g.apply(actor, action.clone())?;
            let after = g.observe(actor);
            if matches!(action, Action::Pass)
                && let Some(item) = before.stack.last()
                && !after.stack.iter().any(|s| s.id == item.id)
                && after.resolving.is_none()
            {
                let n = check(&before, &after, item);
                checks += n;
                if n > 0 {
                    resolved.insert((item.source_card.name(), item.mode));
                }
            }
            if before.announcing.is_some()
                || before.resolving.is_some()
                || before.entry_life.is_some()
                || matches!(action, Action::Pass) && !before.stack.is_empty()
            {
                let expected = g.snapshot();
                g.restore(&saved)?;
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
        assert!(g.outcome().is_some(), "game {seed} didn't finish");
    }
    if games >= 20 {
        assert!(checks > 0 && replays > 0);
        assert!(resolved.iter().any(|(name, _)| *name == "Fumigate"));
    }
    println!(
        "Completed {games} supported Standard group-effect games, {decisions} decisions, {checks} independently checked public batch facts, {replays} cloned/replayed boundaries, {max_bytes} maximum initialized state bytes; resolved {resolved:?}; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
    Ok(())
}
