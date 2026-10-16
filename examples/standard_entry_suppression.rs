//! Native Standard self-play with an independent event-to-trigger ledger.
use rustmage::{
    Action, Card, Config, DeckRules, Error, Game, Keywords, ObjectRef, ObjectView, Observation,
    Player, StackKind, Zone,
};
use std::collections::{BTreeMap, BTreeSet};
const NAMES: [&str; 3] = ["Torpor Orb", "Doorkeeper Thrull", "Karn, Argent Defender"];
fn deck(f: usize) -> Vec<Card> {
    let mut d = vec![Card::from_name("Plains").unwrap(); 10];
    d.extend([Card::from_name("Swamp").unwrap(); 10]);
    for n in [
        "Tranquil Cove",
        NAMES[f],
        "Helpful Hunter",
        "Mephitic Draught",
        "Pactdoll Terror",
        "Hopeful Vigil",
        "Callous Inspector",
        "Diregraf Ghoul",
        "Burdened Stoneback",
        "Fear of Lost Teeth",
    ] {
        let c = Card::from_name(n).unwrap();
        assert!(c.is_supported() && c.is_standard_legal());
        d.extend([c; 4]);
    }
    assert_eq!(d.len(), 60);
    d
}
fn identity(o: &Observation) -> impl Iterator<Item = &rustmage::SpellView> {
    o.stack
        .iter()
        .chain(&o.waiting_triggers)
        .chain(&o.announcing)
        .chain(&o.resolving)
}
fn check(o: &Observation) -> u64 {
    let mut pairs = 0;
    for v in o
        .objects
        .iter()
        .filter(|v| v.zone == Zone::Battlefield && v.card.is_creature())
    {
        let (p, t, k) = match v.card.name() {
            "Doorkeeper Thrull" => (1, 2, Keywords::FLASH.union(Keywords::FLYING)),
            "Karn, Argent Defender" => (1, 3, Keywords::NONE),
            "Helpful Hunter" => (1, 1, Keywords::NONE),
            "Pactdoll Terror" => (3, 4, Keywords::NONE),
            "Callous Inspector" => (1, 1, Keywords::MENACE),
            "Diregraf Ghoul" => (2, 2, Keywords::NONE),
            "Burdened Stoneback" => (4, 4, Keywords::NONE),
            "Fear of Lost Teeth" => (1, 1, Keywords::NONE),
            _ => {
                assert!(v.card.token_info().is_some_and(|t| t.subtypes == "Knight"));
                (2, 2, Keywords::VIGILANCE)
            }
        };
        let delta = i128::from(v.counters.plus_one) - i128::from(v.counters.minus_one);
        assert_eq!((v.power, v.toughness), (p + delta, t + delta));
        assert_eq!(v.keywords, k);
        pairs += 1;
    }
    pairs
}
fn entry_condition(source: &ObjectView, entered: &ObjectView) -> bool {
    match source.card.name() {
        "Helpful Hunter" | "Mephitic Draught" | "Hopeful Vigil" | "Tranquil Cove" => {
            source.reference == entered.reference
        }
        "Pactdoll Terror" => {
            source.owner == entered.owner
                && (source.reference == entered.reference || entered.card.is_artifact())
        }
        _ => false,
    }
}
fn death_ability(v: &ObjectView) -> Option<u16> {
    match v.card.name() {
        "Mephitic Draught" | "Callous Inspector" | "Fear of Lost Teeth" => Some(0),
        "Hopeful Vigil" => Some(1),
        _ => None,
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let games: usize = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "100".into())
        .parse()?;
    let (mut decisions, mut replays, mut pairs, mut max_bytes) = (0u64, 0u64, 0u64, 0usize);
    let (mut casts, mut blocked, mut allowed, mut deaths) = ([0u64; 3], [0u64; 3], [0u64; 3], 0u64);
    let (mut land_entries, mut artifact_entries, mut creature_entries) = (0u64, 0u64, 0u64);
    for seed in 0..games {
        let family = seed % 3;
        let mut g = Game::new(Config {
            decks: [deck(family), deck(family)],
            seed: seed as u64,
            starting_player: Player::P0,
            deck_rules: DeckRules::Standard,
        })?;
        let mut seen = BTreeSet::new();
        let mut debt = BTreeMap::<(ObjectRef, u16), u64>::new();
        for _ in 0..100_000 {
            if g.outcome().is_some() {
                break;
            }
            let actor = g.actor().unwrap();
            let before = g.observe(actor);
            let actions = g.legal_actions();
            let fallback = rustmage::policy::smoke_index(&g, &actions);
            let selected = actions
                .iter()
                .enumerate()
                .max_by_key(|(j, a)| match a {
                    Action::PayCosts { .. } => 500,
                    Action::TapMana { .. } => {
                        if before.decision == "pay_costs" {
                            150
                        } else {
                            -100
                        }
                    }
                    Action::Cast { card, .. } | Action::AnnounceCast { card } => {
                        let v = before.objects.iter().find(|v| v.id == *card).unwrap();
                        if NAMES.contains(&v.card.name()) {
                            if seed % 2 == 0 || before.turn >= 5 {
                                280
                            } else {
                                -100
                            }
                        } else {
                            250
                        }
                    }
                    // Keep entry counters and keyword witnesses immutable in this workload.
                    Action::Activate { .. }
                    | Action::BeginActivate { .. }
                    | Action::AnnounceActivate { .. }
                    | Action::ActivateMana { .. } => -100,
                    Action::ChooseTarget {
                        target: rustmage::Target::Player(p),
                    } => {
                        if *p != actor {
                            400
                        } else {
                            -100
                        }
                    }
                    Action::Concede => -1000,
                    _ => {
                        if *j == fallback {
                            100
                        } else {
                            0
                        }
                    }
                })
                .unwrap()
                .0;
            let action = actions[selected].clone();
            let boundary = before
                .objects
                .iter()
                .any(|v| v.zone == Zone::Battlefield && NAMES.contains(&v.card.name()))
                || before.announcing.is_some()
                || !before.stack.is_empty()
                || !before.waiting_triggers.is_empty();
            let saved = g.snapshot();
            let mut fork = boundary.then(|| g.clone());
            g.apply(actor, action.clone())?;
            if let Some(f) = &mut fork {
                f.apply(actor, action.clone())?;
            }
            let after = g.observe(actor);
            // Independently reconstruct the post-event prohibition from public card names.
            let creature_block = after
                .objects
                .iter()
                .any(|v| v.zone == Zone::Battlefield && NAMES.contains(&v.card.name()));
            let artifact_block = after
                .objects
                .iter()
                .any(|v| v.zone == Zone::Battlefield && NAMES[1..].contains(&v.card.name()));
            for v in after.objects.iter().filter(|v| {
                v.zone == Zone::Battlefield
                    && !before
                        .objects
                        .iter()
                        .any(|b| b.zone == Zone::Battlefield && b.reference == v.reference)
            }) {
                creature_entries += u64::from(v.card.is_creature());
                artifact_entries += u64::from(v.card.is_artifact());
                land_entries += u64::from(v.card.is_land());
                let suppress = v.card.is_creature() && creature_block
                    || v.card.is_artifact() && artifact_block;
                for source in after
                    .objects
                    .iter()
                    .filter(|s| s.zone == Zone::Battlefield && entry_condition(s, v))
                {
                    if suppress {
                        blocked[family] += 1;
                    } else {
                        allowed[family] += 1;
                        *debt.entry((source.reference, 0)).or_default() += 1;
                    }
                }
            }
            for v in before.objects.iter().filter(|v| {
                v.zone == Zone::Battlefield
                    && after
                        .objects
                        .iter()
                        .any(|a| a.id == v.id && matches!(a.zone, Zone::Graveyard | Zone::Outside))
            }) {
                if let Some(ability) = death_ability(v) {
                    *debt.entry((v.reference, ability)).or_default() += 1;
                    deaths += 1;
                }
            }
            for item in identity(&after) {
                if !seen.insert(item.id) {
                    continue;
                }
                if item.kind == StackKind::Spell {
                    if let Some(i) = NAMES.iter().position(|n| *n == item.source_card.name()) {
                        casts[i] += 1;
                    }
                    continue;
                }
                if let StackKind::TriggeredAbility { ability } = item.kind {
                    let count = debt.get_mut(&(item.object, ability)).unwrap_or_else(|| {
                        panic!(
                            "unexpected trigger {} ability {ability} game {seed}",
                            item.source_card.name()
                        )
                    });
                    assert!(
                        *count > 0,
                        "extra trigger {} game {seed}",
                        item.source_card.name()
                    );
                    *count -= 1;
                }
            }
            if after.outcome.is_none() && after.decision == "priority" {
                assert!(
                    debt.values().all(|n| *n == 0),
                    "missing trigger game {seed}: {debt:?}"
                );
            }
            pairs += check(&after);
            check(&g.observe(actor.other()));
            assert_eq!(after.schema, 51);
            if let Some(f) = fork {
                let expected = g.snapshot();
                assert_eq!(f.snapshot(), expected);
                g.restore(&saved)?;
                assert_eq!(
                    g.apply_index(actor, before.revision, 0),
                    Err(Error::StaleRevision)
                );
                g.apply(actor, action)?;
                assert_eq!(g.snapshot(), expected);
                replays += 1;
            }
            max_bytes = max_bytes.max(g.state_bytes());
            decisions += 1;
        }
        assert!(g.outcome().is_some(), "game {seed} unfinished");
    }
    if games >= 100 {
        assert!(casts.iter().all(|n| *n > 0));
        assert!(blocked.iter().all(|n| *n > 0));
        assert!(allowed.iter().all(|n| *n > 0));
        assert!(deaths > 0 && land_entries > 0 && creature_entries > 0 && artifact_entries > 0);
    }
    println!(
        "Completed {games} Standard entry-suppression games, {decisions} decisions, suppressor casts {casts:?}, blocked entry triggers {blocked:?}, allowed entry triggers {allowed:?}, {deaths} death triggers, {land_entries} land entries, {creature_entries} creature entries, {artifact_entries} artifact entries, {pairs} public stat/ability pairs, {replays} replay boundaries, {max_bytes} maximum initialized logical bytes; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
    Ok(())
}
