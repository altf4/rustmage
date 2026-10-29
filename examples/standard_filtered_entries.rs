//! Legal Standard self-play with public effect witnesses, full bodies and native replay.
use rustmage::{
    Action, Card, Config, DeckRules, Error, Game, Keywords, ObjectRef, ObjectView, Observation,
    Player, SpellView, StackKind, Zone,
};
use std::collections::{BTreeMap, BTreeSet};
const NAMES: [&str; 8] = [
    "Arahbo, the First Fang",
    "Arni, Humble Scribe",
    "Black Panther, Vanguard",
    "Genghis Frog",
    "Knightfisher",
    "Lathliss, Dragon Queen",
    "Pactdoll Terror",
    "Thoughtweft Lieutenant",
];
const BASE: [(i128, i128); 8] = [
    (2, 2),
    (3, 2),
    (4, 4),
    (1, 3),
    (4, 5),
    (6, 6),
    (3, 4),
    (2, 2),
];
type Ledger = BTreeMap<ObjectRef, (u32, i128, i128, Keywords)>;
fn index(c: Card) -> Option<usize> {
    NAMES.iter().position(|n| *n == c.name())
}
fn deck(f: usize) -> Vec<Card> {
    let spells = if f == 0 {
        vec![
            NAMES[0],
            NAMES[1],
            NAMES[2],
            NAMES[3],
            NAMES[4],
            NAMES[7],
            "Three Tree Mascot",
            "Helpful Hunter",
            "Shock",
        ]
    } else if f == 1 {
        NAMES.into_iter().chain(["Three Tree Mascot"]).collect()
    } else {
        vec![
            NAMES[1],
            NAMES[2],
            NAMES[3],
            NAMES[4],
            NAMES[5],
            NAMES[6],
            "Three Tree Mascot",
            "Shivan Dragon",
            "Rapacious Dragon",
        ]
    };
    let mut d = Vec::new();
    for n in [
        "Tranquil Cove",
        "Jungle Hollow",
        "Thornwood Falls",
        "Bloodfell Caves",
        "Rugged Highlands",
        "Blossoming Sands",
    ]
    .into_iter()
    .chain(spells)
    {
        let c = Card::from_name(n).unwrap();
        assert!(c.is_supported() && c.is_standard_legal());
        d.extend([c; 4]);
    }
    assert_eq!(d.len(), 60);
    d
}
fn intrinsic(i: usize) -> Keywords {
    match i {
        3 => Keywords::TRAMPLE,
        4 | 5 => Keywords::FLYING,
        _ => Keywords::NONE,
    }
}
fn add(l: &mut Ledger, r: ObjectRef, turn: u32, p: i128, t: i128, k: Keywords) {
    let e = l.entry(r).or_insert((turn, 0, 0, Keywords::NONE));
    if e.0 != turn {
        *e = (turn, 0, 0, Keywords::NONE);
    }
    e.1 += p;
    e.2 += t;
    e.3 = e.3.union(k);
}
fn cat(v: &ObjectView) -> bool {
    matches!(
        v.card.name(),
        "Arahbo, the First Fang" | "Three Tree Mascot" | "Helpful Hunter"
    ) || v.token.is_some_and(|t| t.subtypes == "Cat")
}
fn dragon(v: &ObjectView) -> bool {
    matches!(
        v.card.name(),
        "Lathliss, Dragon Queen" | "Three Tree Mascot" | "Shivan Dragon" | "Rapacious Dragon"
    ) || v.token.is_some_and(|t| t.subtypes == "Dragon")
}
fn check(o: &Observation, l: &Ledger) -> u64 {
    let mut checks = 0;
    for v in o
        .objects
        .iter()
        .filter(|v| v.zone == Zone::Battlefield && v.card.is_creature())
    {
        let (base, mut k) = if let Some(i) = index(v.card) {
            (BASE[i], intrinsic(i))
        } else if v.card.name() == "Three Tree Mascot" {
            ((2, 1), Keywords::CHANGELING)
        } else if v.card.name() == "Shivan Dragon" {
            ((5, 5), Keywords::FLYING)
        } else if v.card.name() == "Rapacious Dragon" {
            ((3, 3), Keywords::FLYING)
        } else if v.card.name() == "Helpful Hunter" {
            ((1, 1), Keywords::NONE)
        } else {
            match v.token.unwrap().subtypes {
                "Cat" | "Fish" | "Soldier" => ((1, 1), Keywords::NONE),
                "Dragon" => ((5, 5), Keywords::FLYING),
                s => panic!("unexpected token {s}"),
            }
        };
        let a = if cat(v) {
            o.objects
                .iter()
                .filter(|s| {
                    s.zone == Zone::Battlefield
                        && s.owner == v.owner
                        && s.reference != v.reference
                        && s.card.name() == NAMES[0]
                })
                .count() as i128
        } else {
            0
        };
        let growth = l
            .get(&v.reference)
            .filter(|e| e.0 == o.turn)
            .map_or((0, 0), |e| {
                k = k.union(e.3);
                (e.1, e.2)
            });
        let counters = i128::from(v.counters.plus_one) - i128::from(v.counters.minus_one);
        assert_eq!(
            (v.power, v.toughness),
            (
                base.0 + a + growth.0 + counters,
                base.1 + a + growth.1 + counters
            ),
            "{}",
            v.card.name()
        );
        assert_eq!(v.keywords, k, "{}", v.card.name());
        checks += 1;
    }
    checks
}
fn classify(s: &SpellView) -> Option<usize> {
    match s.kind {
        StackKind::CasualtyCopy
        | StackKind::CopyEndStepSacrifice
        | StackKind::DelayedReturn
        | StackKind::DelayedTokenCreation { .. }
        | StackKind::DelayedSacrifice
        | StackKind::DelayedWarpExile
        | StackKind::DelayedManifestDread
        | StackKind::SiegeDefeat => {
            panic!("no delayed producer in this fixture")
        }
        StackKind::TriggeredAbility { .. } => s.source().and_then(|source| index(source.card)),
        StackKind::ActivatedAbility { .. } => match s.source().and_then(|source| index(source.card)) {
            Some(1) => Some(8),
            Some(5) => Some(9),
            _ => s
                .source().expect("example expects an object-backed packet").card
                .token_info()
                .filter(|t| t.subtypes == "Mutagen")
                .map(|_| 10),
        },
        _ => None,
    }
}
fn exact(o: &Observation, r: ObjectRef) -> Option<&ObjectView> {
    o.objects
        .iter()
        .find(|v| v.reference == r && v.zone == Zone::Battlefield)
}
fn target<'a>(o: &'a Observation, s: &SpellView) -> Option<&'a ObjectView> {
    s.targets
        .first()
        .and_then(|b| b.object)
        .and_then(|r| exact(o, r))
}
fn verify(i: usize, root: &SpellView, b: &Observation, a: &Observation, l: &mut Ledger) {
    match i {
        0 | 3 | 4 | 5 => {
            let subtype = match i {
                0 => "Cat",
                3 => "Mutagen",
                4 => "Fish",
                _ => "Dragon",
            };
            let count = |o: &Observation| {
                o.objects
                    .iter()
                    .filter(|v| {
                        v.owner == root.controller
                            && v.zone == Zone::Battlefield
                            && v.token.is_some_and(|t| t.subtypes == subtype)
                    })
                    .count()
            };
            assert_eq!(count(a), count(b) + 1);
        }
        1 => {
            if let Some(old) = exact(b, root.object) {
                assert!(!exact(a, old.reference).unwrap().tapped);
            }
        }
        2 => {
            if root.mode == Some(0) {
                let count = |o: &Observation| {
                    o.objects
                        .iter()
                        .filter(|v| {
                            v.owner == root.controller
                                && v.zone == Zone::Battlefield
                                && v.token.is_some_and(|t| t.subtypes == "Soldier")
                        })
                        .count()
                };
                assert_eq!(count(a), count(b) + 1);
            } else {
                assert_eq!(root.mode, Some(1));
                for v in b.objects.iter().filter(|v| {
                    v.owner == root.controller
                        && v.zone == Zone::Battlefield
                        && v.card.is_creature()
                }) {
                    add(l, v.reference, a.turn, 1, 1, Keywords::NONE);
                    let after = exact(a, v.reference).unwrap();
                    assert_eq!(
                        (after.power, after.toughness),
                        (v.power + 1, v.toughness + 1)
                    );
                }
            }
        }
        6 => {
            assert_eq!(
                a.players[root.controller.index()].life,
                b.players[root.controller.index()].life + 1
            );
            assert_eq!(
                a.players[root.controller.other().index()].life,
                b.players[root.controller.other().index()].life - 1
            );
        }
        7 => {
            if let Some(v) = target(b, root) {
                add(l, v.reference, a.turn, 1, 1, Keywords::TRAMPLE);
                let after = exact(a, v.reference).unwrap();
                assert_eq!(
                    (after.power, after.toughness),
                    (v.power + 1, v.toughness + 1)
                );
                assert!(after.keywords.contains(Keywords::TRAMPLE));
            }
        }
        8 => {
            let p = root.controller.index();
            assert_eq!(a.players[p].library_count, b.players[p].library_count - 1);
            assert_eq!(a.players[p].hand_count, b.players[p].hand_count);
        }
        9 => {
            for v in b.objects.iter().filter(|v| {
                v.zone == Zone::Battlefield
                    && v.owner == root.controller
                    && v.card.is_creature()
                    && dragon(v)
            }) {
                add(l, v.reference, a.turn, 1, 0, Keywords::NONE);
                let after = exact(a, v.reference).unwrap();
                assert_eq!((after.power, after.toughness), (v.power + 1, v.toughness));
            }
        }
        10 => {
            if let Some(v) = target(b, root) {
                assert_eq!(
                    exact(a, v.reference).unwrap().counters.plus_one,
                    v.counters.plus_one + 1
                );
            }
        }
        _ => unreachable!(),
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let games: usize = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "100".into())
        .parse()?;
    let (mut decisions, mut replays, mut pairs, mut private, mut max_bytes) = (0, 0, 0, 0, 0);
    let (mut casts, mut triggers, mut activations, mut verified, mut modes) =
        ([0u64; 8], [0u64; 8], [0u64; 3], [0u64; 11], [0u64; 2]);
    for seed in 0..games {
        let mut g = Game::new(Config {
            decks: [deck(seed % 3), deck(seed % 3)],
            seed: seed as u64,
            starting_player: Player::P0,
            deck_rules: DeckRules::Standard,
        })?;
        let mut ledger = Ledger::new();
        let mut pending = BTreeMap::<u64, (usize, SpellView, Observation)>::new();
        let mut used = BTreeSet::new();
        for _ in 0..100_000 {
            if g.outcome().is_some() {
                break;
            }
            let actor = g.actor().unwrap();
            let before = g.observe(actor);
            let actions = g.legal_actions();
            let fallback = rustmage::policy::smoke_index(&g, &actions);
            let chosen = actions
                .iter()
                .enumerate()
                .max_by_key(|(j, a)| match a {
                    Action::ChooseMode { mode } => {
                        if usize::from(*mode) == (seed + verified[2] as usize) % 2 {
                            500
                        } else {
                            -100
                        }
                    }
                    Action::AnnounceActivate { card, .. } => {
                        let v = before.objects.iter().find(|v| v.id == *card).unwrap();
                        if !used.contains(&(v.reference, before.turn))
                            && matches!(
                                v.card.name(),
                                "Arni, Humble Scribe" | "Lathliss, Dragon Queen"
                            )
                            || v.token.is_some_and(|t| t.subtypes == "Mutagen")
                        {
                            if v.card.name() == NAMES[5]
                                && !before.objects.iter().any(|o| {
                                    o.owner == actor
                                        && o.zone == Zone::Battlefield
                                        && o.reference != v.reference
                                        && o.card.has_subtype("Dragon")
                                })
                            {
                                -100
                            } else {
                                180
                            }
                        } else {
                            -100
                        }
                    }
                    Action::Activate { .. } | Action::BeginActivate { .. } => -100,
                    Action::Cast { card, .. } | Action::AnnounceCast { card } => {
                        let v = before.objects.iter().find(|v| v.id == *card).unwrap();
                        let lathliss_waiting = before.objects.iter().any(|o| {
                            o.owner == actor && o.zone == Zone::Hand && o.card.name() == NAMES[5]
                        }) && !before.objects.iter().any(|o| {
                            o.owner == actor
                                && o.zone == Zone::Battlefield
                                && o.card.name() == NAMES[5]
                        });
                        if v.card.name() == "Three Tree Mascot" && lathliss_waiting {
                            -100
                        } else {
                            250
                        }
                    }
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
            let action = actions[chosen].clone();
            if before.library_choice.is_some()
                || before.search_choice.is_some()
                || before.decision == "effect_discard"
            {
                let other = g.observe(actor.other());
                assert!(
                    other.library_choice.is_none()
                        && other.search_choice.is_none()
                        && other.effect_discards.is_empty()
                        && other.actions.is_empty()
                );
                private += 1;
            }
            if let Action::ChooseMode { mode } = action {
                let root = before.announcing.as_ref().unwrap();
                assert_eq!(root.source().expect("example expects an object-backed packet").card.name(), NAMES[2]);
                assert!(g.observe(actor.other()).mode_choices.is_empty());
                modes[mode as usize] += 1;
            }
            if let Action::AnnounceActivate { card, .. } = action {
                let v = before.objects.iter().find(|v| v.id == card).unwrap();
                used.insert((v.reference, before.turn));
            }
            let boundary = before.announcing.is_some()
                || before.resolving.is_some()
                || before.stack.iter().any(|s| classify(s).is_some());
            let saved = g.snapshot();
            let mut fork = boundary.then(|| g.clone());
            g.apply(actor, action.clone())?;
            if let Some(f) = &mut fork {
                f.apply(actor, action.clone())?;
            }
            let after = g.observe(actor);
            for s in &after.stack {
                if before.stack.iter().any(|b| b.id == s.id)
                    || before.resolving.as_ref().is_some_and(|b| b.id == s.id)
                {
                    continue;
                }
                if s.kind == StackKind::Spell {
                    if let Some(i) = s.source().and_then(|source| index(source.card)) {
                        casts[i] += 1;
                    }
                } else if let Some(i) = classify(s) {
                    if i < 8 {
                        triggers[i] += 1;
                    } else {
                        activations[i - 8] += 1;
                    }
                }
            }
            if matches!(action, Action::Pass)
                && let Some(root) = before.stack.last()
                && let Some(i) = classify(root)
                && !after.stack.iter().any(|s| s.id == root.id)
            {
                pending.insert(root.id, (i, root.clone(), before.clone()));
            }
            let finished: Vec<_> = pending
                .keys()
                .copied()
                .filter(|id| {
                    !after.stack.iter().any(|s| s.id == *id)
                        && !after.resolving.as_ref().is_some_and(|s| s.id == *id)
                })
                .collect();
            for id in finished {
                let (i, root, b) = pending.remove(&id).unwrap();
                if after.outcome.is_none() {
                    verify(i, &root, &b, &after, &mut ledger);
                    verified[i] += 1;
                }
            }
            pairs += check(&after, &ledger);
            check(&g.observe(actor.other()), &ledger);
            assert_eq!(after.schema, rustmage::OBSERVATION_SCHEMA);
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
            decisions += 1;
            max_bytes = max_bytes.max(g.state_bytes());
        }
        assert!(g.outcome().is_some(), "game {seed} unfinished");
    }
    if games >= 100 {
        assert!(casts.iter().all(|n| *n > 0), "casts {casts:?}");
        assert!(verified.iter().all(|n| *n > 0), "verified {verified:?}");
        assert!(modes.iter().all(|n| *n > 0));
        assert!(activations.iter().all(|n| *n > 0));
    }
    println!(
        "Completed {games} Standard filtered-entry games, {decisions} decisions, casts {casts:?}, triggers {triggers:?}, activations Arni/Lathliss/Mutagen {activations:?}, verified trigger/activation effects {verified:?}, Black Panther modes {modes:?}, {pairs} independently checked public stat/ability pairs, {private} private choices, {replays} replay boundaries, {max_bytes} maximum initialized logical bytes; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
    Ok(())
}
