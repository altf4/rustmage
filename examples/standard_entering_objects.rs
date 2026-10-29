//! Independent non-target entry-reference, persistent-counter and temporary-haste witnesses.
use rustmage::{
    Action, Card, Config, DeckRules, Error, Game, Keywords, LibraryDestination, ObjectRef,
    ObjectView, Observation, Player, SpellView, StackKind, Zone,
};
use std::collections::{BTreeMap, BTreeSet};
const NAMES: [&str; 3] = [
    "Good-Fortune Unicorn",
    "Serra Redeemer",
    "Wartime Protestors",
];
type Ledger = BTreeMap<ObjectRef, (u32, Keywords, u64)>;
fn deck(f: usize) -> Vec<Card> {
    let spells = match f {
        0 => vec![
            NAMES[0],
            NAMES[1],
            NAMES[2],
            "First-Time Flyer",
            "Merchant of Many Hats",
            "Swab Goblin",
            "Bear Cub",
            "Snarling Gorehound",
            "Torpor Orb",
        ],
        1 => vec![
            NAMES[0],
            NAMES[1],
            NAMES[2],
            "First-Time Flyer",
            "Merchant of Many Hats",
            "Swab Goblin",
            "Bear Cub",
            "Snarling Gorehound",
            "Warleader's Call",
        ],
        _ => vec![
            NAMES[0],
            NAMES[1],
            NAMES[2],
            "First-Time Flyer",
            "Invasion Reinforcements",
            "Helpful Hunter",
            "Bear Cub",
            "Snarling Gorehound",
            "Torpor Orb",
        ],
    };
    let mut d = vec![Card::from_name("Plains").unwrap(); 4];
    for n in ["Plains", "Forest", "Mountain", "Island", "Swamp"] {
        d.extend([Card::from_name(n).unwrap(); 4]);
    }
    for n in spells {
        let c = Card::from_name(n).unwrap();
        assert!(c.is_supported() && c.is_standard_legal());
        d.extend([c; 4]);
    }
    assert_eq!(d.len(), 60);
    d
}
fn base(v: &ObjectView, o: &Observation, l: &Ledger) -> (i128, i128, Keywords, u64) {
    let (p, t, mut k) = match v.card.name() {
        "Good-Fortune Unicorn" => (2, 2, Keywords::NONE),
        "Serra Redeemer" => (2, 4, Keywords::FLYING),
        "Wartime Protestors" => (4, 4, Keywords::HASTE),
        "First-Time Flyer" => (1, 2, Keywords::FLYING),
        "Merchant of Many Hats" | "Swab Goblin" | "Bear Cub" => (2, 2, Keywords::NONE),
        "Snarling Gorehound" => (1, 1, Keywords::MENACE),
        "Helpful Hunter" => (1, 1, Keywords::NONE),
        "Invasion Reinforcements" => (1, 1, Keywords::FLASH),
        _ => {
            assert!(v.card.token_info().is_some_and(|t| t.subtypes == "Ally"));
            (1, 1, Keywords::NONE)
        }
    };
    let calls = o
        .objects
        .iter()
        .filter(|x| {
            x.zone == Zone::Battlefield && x.owner == v.owner && x.card.name() == "Warleader's Call"
        })
        .count() as i128;
    let mut counters = 0;
    if let Some((turn, granted, c)) = l.get(&v.reference) {
        counters = *c;
        if *turn == o.turn {
            k = k.union(*granted);
        }
    }
    let delta = calls + i128::from(counters);
    (p + delta, t + delta, k, counters)
}
fn check(o: &Observation, l: &Ledger) -> u64 {
    let mut pairs = 0;
    for v in o
        .objects
        .iter()
        .filter(|v| v.zone == Zone::Battlefield && v.card.is_creature())
    {
        let (p, t, k, c) = base(v, o, l);
        assert_eq!(
            (v.power, v.toughness, v.keywords, v.counters.plus_one),
            (p, t, k, c),
            "{}",
            v.card.name()
        );
        pairs += 1;
    }
    pairs
}
fn matches(s: &ObjectView, v: &ObjectView, o: &Observation, l: &Ledger) -> bool {
    if s.owner != v.owner {
        return false;
    }
    match s.card.name() {
        "Good-Fortune Unicorn" => s.reference != v.reference && v.card.is_creature(),
        "Serra Redeemer" | "Snarling Gorehound" => {
            s.reference != v.reference && v.card.is_creature() && base(v, o, l).0 <= 2
        }
        "Wartime Protestors" => s.reference != v.reference && v.card.has_subtype("Ally"),
        "Warleader's Call" => v.card.is_creature(),
        "Helpful Hunter" | "Invasion Reinforcements" => s.reference == v.reference,
        _ => false,
    }
}
fn bump(l: &mut Ledger, r: ObjectRef, turn: u32, c: u64, haste: bool) {
    let e = l.entry(r).or_insert((turn, Keywords::NONE, 0));
    if e.0 != turn {
        e.0 = turn;
        e.1 = Keywords::NONE;
    }
    e.2 += c;
    if haste {
        e.1 = e.1.union(Keywords::HASTE);
    }
}
fn verify(s: &SpellView, b: &Observation, a: &Observation, l: &mut Ledger) -> Option<usize> {
    let name = s.source()?.card.name();
    let r = s.entering_object.expect("entry trigger context");
    assert!(s.targets.is_empty());
    if let Some(i) = NAMES.iter().position(|n| *n == name) {
        if a.objects
            .iter()
            .any(|v| v.reference == r && v.zone == Zone::Battlefield)
        {
            bump(l, r, a.turn, if i == 1 { 2 } else { 1 }, i == 2);
        }
        return Some(i);
    }
    match name {
        "Warleader's Call" => assert_eq!(
            a.players[s.controller.other().index()].life,
            b.players[s.controller.other().index()].life - 1
        ),
        "Helpful Hunter" => assert_eq!(
            a.players[s.controller.index()].library_count,
            b.players[s.controller.index()].library_count - 1
        ),
        "Invasion Reinforcements" => assert_eq!(
            a.objects
                .iter()
                .filter(|v| v.zone == Zone::Battlefield
                    && v.card.is_token()
                    && !b.objects.iter().any(|old| old.reference == v.reference))
                .count(),
            1
        ),
        "Snarling Gorehound" => {}
        _ => panic!("unhandled effect {name}"),
    };
    None
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let games: usize = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "100".into())
        .parse()?;
    let (
        mut decisions,
        mut replays,
        mut pairs,
        mut private,
        mut blocked,
        mut contexts,
        mut max_bytes,
    ) = (0u64, 0u64, 0u64, 0u64, 0u64, 0u64, 0usize);
    let (mut casts, mut triggers, mut verified) = ([0u64; 3], [0u64; 3], [0u64; 3]);
    for seed in 0..games {
        let mut g = Game::new(Config {
            decks: [deck(seed % 3), deck(seed % 3)],
            seed: seed as u64,
            starting_player: Player::P0,
            deck_rules: DeckRules::Standard,
        })?;
        let (mut l, mut seen, mut debt, mut pending) = (
            Ledger::new(),
            BTreeSet::new(),
            BTreeMap::<(ObjectRef, ObjectRef, u16), u64>::new(),
            BTreeMap::<u64, (SpellView, Observation)>::new(),
        );
        for _ in 0..100_000 {
            if g.outcome().is_some() {
                break;
            }
            let actor = g.actor().unwrap();
            let b = g.observe(actor);
            let actions = g.legal_actions();
            let fallback = rustmage::policy::smoke_index(&g, &actions);
            let chosen = actions
                .iter()
                .enumerate()
                .max_by_key(|(j, a)| match a {
                    Action::PayCosts { .. } => 500,
                    Action::TapMana { .. } => {
                        if b.decision == "pay_costs" {
                            150
                        } else {
                            -100
                        }
                    }
                    Action::Cast { card, .. } | Action::AnnounceCast { card } => {
                        if b.objects
                            .iter()
                            .any(|v| v.id == *card && v.card.name() == "Torpor Orb")
                            && b.turn < 9
                        {
                            -100
                        } else {
                            250
                        }
                    }
                    Action::Activate { .. }
                    | Action::BeginActivate { .. }
                    | Action::AnnounceActivate { .. } => -100,
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
            let action = actions[chosen].clone();
            if b.library_choice.is_some() {
                assert!(g.observe(actor.other()).library_choice.is_none());
                private += 1;
            }
            let boundary = b.announcing.is_some()
                || b.resolving.is_some()
                || !b.stack.is_empty()
                || b.objects
                    .iter()
                    .any(|v| v.zone == Zone::Battlefield && NAMES.contains(&v.card.name()));
            let saved = g.snapshot();
            let mut fork = boundary.then(|| g.clone());
            g.apply(actor, action.clone())?;
            if let Some(f) = &mut fork {
                f.apply(actor, action.clone())?;
            }
            let a = g.observe(actor);
            if b.library_choice.is_some()
                && let Action::ArrangeLibrary { card, destination } = &action
            {
                let bury = *destination == LibraryDestination::Graveyard;
                assert_eq!(
                    b.players[actor.index()].library_count - a.players[actor.index()].library_count,
                    usize::from(bury)
                );
                let v = a.objects.iter().find(|v| v.id == *card);
                if bury {
                    assert_eq!(v.unwrap().zone, Zone::Graveyard);
                } else {
                    assert!(v.is_none());
                }
            }
            if matches!(action, Action::Pass)
                && let Some(root) = b.stack.last()
                && matches!(root.kind, StackKind::TriggeredAbility { .. })
                && !a.stack.iter().any(|s| s.id == root.id)
            {
                pending.insert(root.id, (root.clone(), b.clone()));
            }
            let done: Vec<_> = pending
                .keys()
                .copied()
                .filter(|id| {
                    !a.stack.iter().any(|s| s.id == *id)
                        && !a.resolving.as_ref().is_some_and(|s| s.id == *id)
                })
                .collect();
            for id in done {
                let (s, before) = pending.remove(&id).unwrap();
                if a.outcome.is_none()
                    && let Some(i) = verify(&s, &before, &a, &mut l)
                {
                    verified[i] += 1;
                }
            }
            let prohibit = a
                .objects
                .iter()
                .any(|v| v.zone == Zone::Battlefield && v.card.name() == "Torpor Orb");
            for v in a.objects.iter().filter(|v| {
                v.zone == Zone::Battlefield
                    && !b
                        .objects
                        .iter()
                        .any(|x| x.zone == Zone::Battlefield && x.reference == v.reference)
            }) {
                for s in a.objects.iter().filter(|s| s.zone == Zone::Battlefield) {
                    if matches(s, v, &a, &l) {
                        if prohibit && v.card.is_creature() {
                            blocked += 1;
                        } else {
                            *debt.entry((s.reference, v.reference, 0)).or_default() += 1;
                        }
                    }
                }
            }
            for item in a
                .stack
                .iter()
                .chain(&a.waiting_triggers)
                .chain(&a.announcing)
                .chain(&a.resolving)
            {
                if !seen.insert(item.id) {
                    continue;
                }
                if item.kind == StackKind::Spell {
                    assert!(item.entering_object.is_none());
                    if let Some(i) = NAMES.iter().position(|n| *n == item.source().expect("example expects an object-backed packet").card.name()) {
                        casts[i] += 1;
                    }
                } else if let StackKind::TriggeredAbility { ability } = item.kind {
                    let r = item
                        .entering_object
                        .expect("entry reference must be owned by the packet");
                    let count = debt.get_mut(&(item.source().expect("example expects an object-backed packet").object, r, ability)).unwrap_or_else(|| {
                        panic!(
                            "unexpected trigger {} context {r:?} seed{seed}",
                            item.source().expect("example expects an object-backed packet").card.name()
                        )
                    });
                    assert!(*count > 0);
                    *count -= 1;
                    contexts += 1;
                    assert!(item.targets.is_empty());
                    if let Some(i) = NAMES.iter().position(|n| *n == item.source().expect("example expects an object-backed packet").card.name()) {
                        triggers[i] += 1;
                    }
                }
            }
            if a.outcome.is_none() && a.decision == "priority" {
                assert!(
                    debt.values().all(|n| *n == 0),
                    "missing triggers seed{seed}: {debt:?}"
                );
            }
            pairs += check(&a, &l);
            check(&g.observe(actor.other()), &l);
            assert_eq!(a.schema, rustmage::OBSERVATION_SCHEMA);
            if let Some(f) = fork {
                let expected = g.snapshot();
                assert_eq!(f.snapshot(), expected);
                g.restore(&saved)?;
                assert_eq!(
                    g.apply_index(actor, b.revision, 0),
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
        assert!(triggers.iter().all(|n| *n > 0), "triggers {triggers:?}");
        assert!(verified.iter().all(|n| *n > 0), "effects {verified:?}");
        assert!(private > 0 && blocked > 0);
    }
    println!(
        "Completed {games} Standard entering-object games, {decisions} decisions, casts {casts:?}, triggers {triggers:?}, verified effects {verified:?}, {contexts} exact entry contexts, {blocked} suppressed would-be triggers, {pairs} public stat/ability pairs, {private} private choices, {replays} replay boundaries, {max_bytes} maximum initialized logical bytes; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
    Ok(())
}
