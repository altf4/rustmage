//! Public target legality checks, removal outcomes, and branch replay on supported Standard decks.
use rustmage::{
    Action, Card, Config, DeckRules, Game, Keywords, ObjectView, Observation, Player, SpellView,
    StackKind, Target, Zone,
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
// Independent printed predicates over public facts, rather than calling engine target matching.
fn legal(
    name: &str,
    mode: Option<u16>,
    kind: StackKind,
    o: &ObjectView,
    obs: &Observation,
    actor: Player,
) -> Option<bool> {
    let creature = o.card.is_creature();
    let typ = o.card.info().type_line.split(" — ").next().unwrap();
    let artifact = typ.split_whitespace().any(|t| t == "Artifact");
    let enchantment = typ.split_whitespace().any(|t| t == "Enchantment");
    let walker = typ.split_whitespace().any(|t| t == "Planeswalker");
    let stat4 = o.power >= 4 || o.toughness >= 4;
    let flying = o.keywords.contains(Keywords::FLYING);
    let attacked = o.attacking;
    let blocks = obs.blocking.contains(&o.id);
    let colors = o.card.colors();
    let mana = o
        .card
        .info()
        .mana_cost
        .split('{')
        .skip(1)
        .map(|s| s.trim_end_matches('}'))
        .map(|s| s.parse::<u64>().unwrap_or(1))
        .sum::<u64>();
    let rule = match (name, mode) {
        ("Not on My Watch", _) => creature && attacked,
        ("Sudden Strike", _) => creature && (attacked || blocks),
        ("Deathmark", _) => creature && colors & 17 != 0,
        ("Shoot the Sheriff", _) => {
            creature
                && !["Assassin", "Mercenary", "Pirate", "Rogue", "Warlock"]
                    .iter()
                    .any(|t| o.card.has_subtype(t))
        }
        ("Broken Wings" | "Airship Crash" | "Shattered Wings", _) => {
            artifact || enchantment || creature && flying
        }
        ("Make Your Move" | "Exorcise", _) => artifact || enchantment || creature && o.power >= 4,
        ("Radiant Strike", _) => artifact || creature && o.tapped,
        ("Stand Up for Yourself", _) => creature && o.power >= 3,
        ("Repel Calamity", _) => creature && stat4,
        ("Gallant Strike", _) => creature && o.toughness >= 4,
        ("Eriette's Lullaby", _) => creature && o.tapped,
        ("Death in the Family", _) => creature && mana <= 3,
        ("Spin Out", _) => creature || o.card.has_subtype("Vehicle"),
        ("Gravkill" | "Embrace Oblivion", _) => creature || o.card.has_subtype("Spacecraft"),
        ("Mortify" | "Withering Torment", _) => creature || enchantment,
        ("Hero's Downfall" | "Worthy Cost", _) => creature || walker,
        ("Grapple with Death", _) => artifact || creature,
        ("Devout Decree", _) => (creature || walker) && colors & 12 != 0,
        ("Refute Destiny", _) => (creature || walker) && colors & 18 != 0,
        ("Terminal Criticism", _) => (creature || walker) && colors & 10 != 0,
        ("Your Fate Ends Here", _) => (creature || walker) && mana >= 3,
        ("Ray of Ruin", _) => {
            creature || o.card.has_subtype("Vehicle") || o.card.is_land() && !o.card.is_basic_land()
        }
        ("Kraul Whipcracker", _) => o.token.is_some() && o.owner != actor,
        ("Meteor Golem", _) => !o.card.is_land() && o.owner != actor,
        ("Rooftop Assassin" | "Stingblade Assassin", _) => {
            creature && o.owner != actor && o.damaged_this_turn
        }
        ("Red Guardian, Super-Soldier", _) => {
            creature && o.owner != actor && o.dealt_damage_this_turn
        }
        ("Disruptor Wanderglyph", _) => {
            return Some(o.zone == Zone::Graveyard && o.owner != actor && o.token.is_none());
        }
        ("Goblin Firebomb", _) if matches!(kind, StackKind::ActivatedAbility { .. }) => true,
        ("Meteor Sword", _) if matches!(kind, StackKind::TriggeredAbility { .. }) => true,
        ("Bumbleflower's Sharepot", _) if matches!(kind, StackKind::ActivatedAbility { .. }) => {
            !o.card.is_land()
        }
        ("Giant's Boulder", _) if matches!(kind, StackKind::ActivatedAbility { ability: 1 }) => {
            true
        }
        ("Scrap Compactor", _) if matches!(kind, StackKind::ActivatedAbility { ability: 1 }) => {
            creature || o.card.has_subtype("Vehicle")
        }
        ("Battle Menu", Some(2)) => creature && o.power >= 4,
        ("Battle Menu", Some(1))
        | ("Sarkhan's Resolve", Some(0))
        | ("Valorous Stance", Some(0)) => return None,
        ("Sarkhan's Resolve" | "Scout the City", Some(1))
        | ("Pawpatch Formation" | "Unforgiving Aim", Some(0)) => creature && flying,
        ("Pawpatch Formation" | "Unforgiving Aim", Some(1)) => enchantment,
        ("Valorous Stance", Some(1)) => creature && o.toughness >= 4,
        _ => return None,
    };
    Some(
        o.zone == Zone::Battlefield
            && rule
            && !(o.owner != actor && o.keywords.contains(Keywords::HEXPROOF)),
    )
}
fn exile(name: &str) -> bool {
    matches!(
        name,
        "Not on My Watch"
            | "Gravkill"
            | "Death in the Family"
            | "Devout Decree"
            | "Exorcise"
            | "Ray of Ruin"
            | "Refute Destiny"
            | "Worthy Cost"
            | "Disruptor Wanderglyph"
    )
}
fn check_item(before: &Observation, after: &Observation, item: &SpellView) -> usize {
    let mut count = 0;
    for t in &item.targets {
        let Some(reference) = t.object else { continue };
        let Some(old) = before.objects.iter().find(|o| o.reference == reference) else {
            continue;
        };
        let Some(valid) = legal(
            item.source().expect("example expects an object-backed packet").card.name(),
            item.mode,
            item.kind,
            old,
            before,
            item.controller,
        ) else {
            continue;
        };
        let now = after.objects.iter().find(|o| o.id == old.id);
        if valid
            && (exile(item.source().expect("example expects an object-backed packet").card.name()) || !old.keywords.contains(Keywords::INDESTRUCTIBLE))
        {
            if old.token.is_some() {
                assert!(
                    now.is_none(),
                    "ceased tokens are absent from public observations"
                );
                count += 1;
                continue;
            }
            let now = now.unwrap();
            assert_ne!(
                now.reference,
                old.reference,
                "{} failed removal",
                item.source().expect("example expects an object-backed packet").card.name()
            );
            assert_eq!(
                now.zone,
                if old.token.is_some() {
                    Zone::Outside
                } else if exile(item.source().expect("example expects an object-backed packet").card.name()) {
                    Zone::Exile
                } else {
                    Zone::Graveyard
                }
            );
        } else {
            let now = now.unwrap();
            assert_eq!(now.reference, old.reference);
            assert_eq!(now.zone, old.zone);
        }
        count += 1;
    }
    count
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let games = std::env::args()
        .nth(1)
        .map(|s| s.parse())
        .transpose()?
        .unwrap_or(100);
    let first = deck(
        &[("Plains", 12), ("Forest", 12)],
        &[
            "Bear Cub",
            "Cat-Owl",
            "Magnigoth Sentry",
            "Battle Menu",
            "Valorous Stance",
            "Broken Wings",
            "Repel Calamity",
            "Not on My Watch",
            "Pawpatch Formation",
        ],
    );
    let second = deck(
        &[("Swamp", 12), ("Forest", 12)],
        &[
            "Rooftop Assassin",
            "Stingblade Assassin",
            "Kraul Whipcracker",
            "Disruptor Wanderglyph",
            "Meteor Golem",
            "Deathmark",
            "Shoot the Sheriff",
            "Withering Torment",
            "Spin Out",
        ],
    );
    let (mut decisions, mut target_checks, mut removals, mut replays, mut max_bytes) =
        (0, 0, 0, 0, 0);
    let mut resolved = std::collections::BTreeSet::new();
    for seed in 0..games {
        let mut g = Game::new(Config {
            decks: [first.clone(), second.clone()],
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
            for a in &actions {
                let (item, target) = match a {
                    Action::Cast {
                        card,
                        target: Some(target),
                        ..
                    } => (
                        Some((
                            before.objects.iter().find(|o| o.id == *card).unwrap().card,
                            None,
                            StackKind::Spell,
                        )),
                        Some(*target),
                    ),
                    Action::ChooseTarget { target } => (
                        before
                            .announcing
                            .as_ref()
                            .map(|s| (s.source().expect("example expects an object-backed packet").card, s.mode, s.kind)),
                        Some(*target),
                    ),
                    _ => (None, None),
                };
                if let (Some((card, mode, kind)), Some(Target::Card(id))) = (item, target) {
                    let obj = before.objects.iter().find(|o| o.id == id).unwrap();
                    if let Some(valid) = legal(card.name(), mode, kind, obj, &before, actor) {
                        assert!(valid, "illegal offered target {}", card.name());
                        target_checks += 1;
                    }
                }
            }
            let action = actions[rustmage::policy::smoke_index(&g, &actions)].clone();
            let saved = g.snapshot();
            g.apply(actor, action.clone())?;
            let after = g.observe(actor);
            if matches!(action, Action::Pass)
                && let Some(item) = before.stack.last()
                && !after.stack.iter().any(|s| s.id == item.id)
            {
                let n = check_item(&before, &after, item);
                removals += n;
                if n > 0 {
                    resolved.insert(item.source().expect("example expects an object-backed packet").card.name());
                }
            }
            if before.announcing.is_some()
                || before.resolving.is_some()
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
        assert!(g.outcome().is_some(), "game{seed} didn't finish");
    }
    if games >= 20 {
        assert!(target_checks > 0 && removals > 0 && replays > 0);
        assert!(resolved.contains("Deathmark") && resolved.contains("Repel Calamity"));
    }
    println!(
        "Completed {games} supported Standard removal games, {decisions} decisions, {target_checks} independently checked offered targets, {removals} checked removal outcomes, {replays} cloned/replayed boundaries, {max_bytes} maximum initialized state bytes; resolved {resolved:?}; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
    Ok(())
}
