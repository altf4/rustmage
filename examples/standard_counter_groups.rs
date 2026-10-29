//! Independent public counter checks and complete native Standard branch replay.
use rustmage::{
    Action, Card, Config, Counters, DeckRules, Game, Observation, Player, SpellView, StackKind,
    Zone,
};
fn deck(lands: &[(&str, usize)], spells: &[&str]) -> Vec<Card> {
    let mut cards = Vec::new();
    for &(name, n) in lands {
        cards.extend(vec![Card::from_name(name).unwrap(); n]);
    }
    for &name in spells {
        let card = Card::from_name(name).unwrap();
        assert!(card.is_supported() && card.is_standard_legal(), "{name}");
        cards.extend([card; 4]);
    }
    assert_eq!(cards.len(), 60);
    cards
}
fn check(before: &Observation, after: &Observation, item: &SpellView) -> usize {
    let Some(source) = item.source() else { return 0; };
    let name = source.card.name();
    let known = match item.kind {
        StackKind::CasualtyCopy
        | StackKind::CopyEndStepSacrifice
        | StackKind::DelayedReturn
        | StackKind::DelayedTokenCreation { .. }
        | StackKind::DelayedSacrifice
        | StackKind::DelayedWarpExile
        | StackKind::DelayedManifestDread
        | StackKind::SiegeDefeat => false,
        StackKind::Spell => matches!(
            name,
            "Darkness Descends" | "The Crystal's Chosen" | "Duty Beyond Death"
        ),
        StackKind::TriggeredAbility { .. } => matches!(
            name,
            "Web-Warriors"
                | "Lyra, Archangel of Dawn"
                | "Minwu, White Mage"
                | "Agent Bishop, Man in Black"
                | "Earth Kingdom Soldier"
                | "Zimone, Paradox Sculptor"
        ),
        StackKind::ActivatedAbility { .. } => matches!(
            name,
            "Abandoned Air Temple" | "Agent Phil Coulson" | "Zimone, Paradox Sculptor"
        ),
    };
    if !known {
        return 0;
    }
    let mut checked = 0;
    for old in &before.objects {
        if old.zone != Zone::Battlefield {
            continue;
        }
        let Some(new) = after
            .objects
            .iter()
            .find(|o| o.reference == old.reference && o.zone == Zone::Battlefield)
        else {
            continue;
        };
        let own = old.owner == item.controller;
        let targeted = item.targets.iter().any(|t| t.object == Some(old.reference));
        let other = old.reference != item.source().expect("example expects an object-backed packet").object;
        let mode = match name {
            "Darkness Descends" if old.card.is_creature() => Some(1),
            "The Crystal's Chosen" | "Duty Beyond Death" if own && old.card.is_creature() => {
                Some(0)
            }
            "Abandoned Air Temple"
                if matches!(item.kind, StackKind::ActivatedAbility { .. })
                    && own
                    && old.card.is_creature() =>
            {
                Some(0)
            }
            "Web-Warriors"
                if matches!(item.kind, StackKind::TriggeredAbility { .. })
                    && own
                    && other
                    && old.card.is_creature() =>
            {
                Some(0)
            }
            "Agent Phil Coulson"
                if matches!(item.kind, StackKind::ActivatedAbility { .. })
                    && own
                    && other
                    && old.card.is_creature()
                    && old.card.has_subtype("Hero") =>
            {
                Some(0)
            }
            "Lyra, Archangel of Dawn"
                if matches!(item.kind, StackKind::TriggeredAbility { .. })
                    && own
                    && old.card.is_creature()
                    && old.card.has_subtype("Angel") =>
            {
                Some(0)
            }
            "Minwu, White Mage"
                if matches!(item.kind, StackKind::TriggeredAbility { .. })
                    && own
                    && old.card.is_creature()
                    && old.card.has_subtype("Cleric") =>
            {
                Some(0)
            }
            "Agent Bishop, Man in Black" | "Earth Kingdom Soldier"
                if matches!(item.kind, StackKind::TriggeredAbility { .. }) && targeted =>
            {
                Some(0)
            }
            "Zimone, Paradox Sculptor" if targeted => match item.kind {
                StackKind::TriggeredAbility { .. } => Some(0),
                StackKind::ActivatedAbility { .. } => Some(2),
                _ => None,
            },
            _ => None,
        };
        let mut expected = old.counters.clone();
        match mode {
            Some(0) => expected.plus_one += 1,
            Some(1) => expected.minus_one += 2,
            Some(2) => {
                expected.plus_one *= 2;
                expected.minus_one *= 2;
                expected.stun *= 2;
            }
            None => {}
            _ => unreachable!(),
        }
        let cancel = expected.plus_one.min(expected.minus_one);
        expected.plus_one -= cancel;
        expected.minus_one -= cancel;
        assert_eq!(new.counters, expected, "{name} on {}", old.card.name());
        checked += 1;
    }
    if name == "The Crystal's Chosen" {
        let created: Vec<_> = after
            .objects
            .iter()
            .filter(|o| {
                o.zone == Zone::Battlefield
                    && o.card.is_token()
                    && !before
                        .objects
                        .iter()
                        .any(|b| b.reference == o.reference && b.zone == Zone::Battlefield)
            })
            .collect();
        assert_eq!(created.len(), 4);
        assert!(created.iter().all(|o| o.owner == item.controller
            && o.card.has_subtype("Hero")
            && o.counters == Counters::of(rustmage::CounterKind::PlusOne, 1)));
        checked += 4;
    }
    checked
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let games: usize = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "1".into())
        .parse()?;
    let first = deck(
        &[
            ("Plains", 8),
            ("Abandoned Air Temple", 4),
            ("Temple Garden", 4),
            ("Hallowed Fountain", 4),
            ("Breeding Pool", 4),
        ],
        &[
            "Agent Bishop, Man in Black",
            "Agent Phil Coulson",
            "Duty Beyond Death",
            "Earth Kingdom Soldier",
            "Lyra Dawnbringer",
            "Lyra, Archangel of Dawn",
            "Minwu, White Mage",
            "Web-Warriors",
            "Zimone, Paradox Sculptor",
        ],
    );
    let second = deck(
        &[("Plains", 12), ("Swamp", 8), ("Godless Shrine", 4)],
        &[
            "Agent Phil Coulson",
            "Darkness Descends",
            "The Crystal's Chosen",
            "Earth Kingdom Soldier",
            "Web-Warriors",
            "Corrupted Conviction",
            "Final Vengeance",
            "Minwu, White Mage",
            "Lyra, Archangel of Dawn",
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
                && after.resolving.is_none()
            {
                let n = check(&before, &after, item);
                checks += n;
                if n > 0 {
                    resolved.insert((item.source().expect("example expects an object-backed packet").card.name(), format!("{:?}", item.kind)));
                }
            }
            if before.announcing.is_some()
                || before.entry_life.is_some()
                || before.decision == "targets"
                || matches!(action, Action::Pass) && before.stack.last().is_some()
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
        assert!(checks > 0 && replays > 0);
        assert!(resolved.iter().any(|(n,k)|*n=="Zimone, Paradox Sculptor"&&k.starts_with("ActivatedAbility")),"doubling was not exercised");
    }
    println!(
        "Completed {games} supported Standard counter-group games, {decisions} decisions, {checks} independently checked public counter values, {replays} cloned/replayed boundaries, {max_bytes} maximum initialized state bytes; resolved counter effects {resolved:?}; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
    Ok(())
}
