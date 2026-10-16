//! Supported Standard damage/card-body rollouts. Walker branches use the explicitly
//! synthetic future-object Rust/C ABI fixtures, not unsupported submitted decks.
use rustmage::{
    Action, Card, Config, DeckRules, Game, Keywords, ObjectView, Observation, Player, SpellView,
    StackKind, Zone,
};
fn deck(lands: &[(&str, usize)], spells: &[&str]) -> Vec<Card> {
    let mut out = Vec::new();
    for &(name, n) in lands {
        out.extend(vec![Card::from_name(name).unwrap(); n]);
    }
    for &name in spells {
        let c = Card::from_name(name).unwrap();
        assert!(c.is_supported() && c.is_standard_legal(), "{name}");
        out.extend([c; 4]);
    }
    assert_eq!(out.len(), 60);
    out
}
fn verify_damage(old: &ObjectView, after: &Observation, amount: i128) {
    let same = after
        .objects
        .iter()
        .find(|o| o.reference == old.reference && o.zone == Zone::Battlefield);
    if let Some(now) = same {
        if old.card.is_planeswalker() {
            assert_eq!(
                now.counters.loyalty,
                old.counters.loyalty - (amount as u64).min(old.counters.loyalty)
            );
            assert_eq!(now.damage, old.damage);
        } else {
            assert_eq!(now.damage, old.damage + amount);
        }
        assert!(now.damaged_this_turn);
    } else if old.token.is_some() {
        assert!(after.objects.iter().all(|o| o.id != old.id));
    } else {
        let now = after.objects.iter().find(|o| o.id == old.id).unwrap();
        assert_eq!(now.zone, Zone::Graveyard);
        assert_ne!(now.reference, old.reference);
    }
}
fn check(before: &Observation, after: &Observation, item: &SpellView) -> usize {
    let name = item.source_card.name();
    let mode = item.mode;
    let mut checks = 0;
    if name == "Dragonback Assault"
        && matches!(item.kind, StackKind::TriggeredAbility { ability: 1 })
    {
        let count = |obs: &Observation| {
            obs.objects
                .iter()
                .filter(|o| {
                    o.zone == Zone::Battlefield
                        && o.owner == item.controller
                        && o.token.is_some_and(|t| t.subtypes == "Dragon")
                })
                .count()
        };
        assert_eq!(count(after), count(before) + 1);
        checks += 1;
        let new = after
            .objects
            .iter()
            .find(|o| {
                o.zone == Zone::Battlefield
                    && o.owner == item.controller
                    && o.token.is_some_and(|t| t.subtypes == "Dragon")
                    && !before
                        .objects
                        .iter()
                        .any(|old| old.reference == o.reference)
            })
            .unwrap();
        assert_eq!((new.power, new.toughness), (4, 4));
        assert!(new.keywords.contains(Keywords::FLYING));
        return checks + 2;
    }
    let group = match (name, item.kind, mode) {
        ("Dragonback Assault", StackKind::TriggeredAbility { ability: 0 }, _) => Some((3, false)),
        ("Fulminous Forte", StackKind::Spell, Some(0)) => Some((1, true)),
        ("Splatter Technique", StackKind::Spell, Some(1)) => Some((4, false)),
        _ => None,
    };
    if let Some((amount, opponent)) = group {
        for old in before.objects.iter().filter(|o| {
            o.zone == Zone::Battlefield
                && (o.card.is_creature() || o.card.is_planeswalker())
                && (!opponent || o.owner != item.controller)
        }) {
            verify_damage(old, after, amount);
            checks += 1;
        }
    }
    let targeted = match (name, item.kind, mode) {
        ("Sear", StackKind::Spell, _) => Some(4),
        ("Broadside Barrage", StackKind::Spell, _)
        | ("Fulminous Forte", StackKind::Spell, Some(1)) => Some(5),
        _ => None,
    };
    if let Some(amount) = targeted {
        let mut valid = false;
        for bound in &item.targets {
            let Some(old) = before.objects.iter().find(|o| {
                Some(o.reference) == bound.object
                    && o.zone == Zone::Battlefield
                    && (o.card.is_creature() || o.card.is_planeswalker())
                    && !(o.owner != item.controller && o.keywords.contains(Keywords::HEXPROOF))
            }) else {
                continue;
            };
            valid = true;
            verify_damage(old, after, amount);
            checks += 1;
        }
        if name == "Broadside Barrage" && valid {
            let p = item.controller.index();
            assert_eq!(
                after.players[p].cards_drawn_this_turn,
                before.players[p].cards_drawn_this_turn
                    + u64::from(before.players[p].library_count > 0)
            );
            checks += 1;
        }
    }
    if name == "Splatter Technique" && item.kind == StackKind::Spell && mode == Some(0) {
        let p = item.controller.index();
        assert_eq!(
            after.players[p].cards_drawn_this_turn,
            before.players[p].cards_drawn_this_turn + 4.min(before.players[p].library_count as u64)
        );
        checks += 1;
    }
    checks
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let games = std::env::args()
        .nth(1)
        .map(|s| s.parse())
        .transpose()?
        .unwrap_or(100);
    let first = deck(
        &[("Mountain", 8), ("Island", 8), ("Forest", 8)],
        &[
            "Bear Cub",
            "Cat-Owl",
            "Grasping Longneck",
            "Dragonback Assault",
            "Sear",
            "Fulminous Forte",
            "Broadside Barrage",
            "Splatter Technique",
            "Seismic Rupture",
        ],
    );
    let second = deck(
        &[("Mountain", 8), ("Island", 8), ("Swamp", 8)],
        &[
            "Rooftop Assassin",
            "Midnight Reaper",
            "Cat-Owl",
            "Sear",
            "Fulminous Forte",
            "Broadside Barrage",
            "Splatter Technique",
            "Pyroclasm",
            "Gravkill",
        ],
    );
    let (mut decisions, mut checks, mut replays, mut max_bytes) = (0, 0, 0, 0);
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
            let action = actions[rustmage::policy::smoke_index(&g, &actions)].clone();
            let saved = g.snapshot();
            g.apply(actor, action.clone())?;
            let after = g.observe(actor);
            if matches!(action, Action::Pass)
                && let Some(item) = before.stack.last()
                && !after.stack.iter().any(|s| s.id == item.id)
            {
                let n = check(&before, &after, item);
                checks += n;
                if n > 0 {
                    resolved.insert((
                        item.source_card.name(),
                        format!("{:?}", item.kind),
                        item.mode,
                    ));
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
        assert!(g.outcome().is_some(), "game {seed} didn't finish");
    }
    if games >= 20 {
        assert!(checks > 0 && replays > 0);
        assert!(
            resolved
                .iter()
                .any(|(name, _, _)| *name == "Broadside Barrage")
        );
    }
    println!(
        "Completed {games} supported Standard damage-body games, {decisions} decisions, {checks} independently checked public facts, {replays} cloned/replayed boundaries, {max_bytes} maximum initialized state bytes; resolved {resolved:?}; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
    Ok(())
}
