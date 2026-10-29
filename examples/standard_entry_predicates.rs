//! Independent entry-condition, effect, continuous-stat and native replay witnesses.
use rustmage::{
    Action, Card, Config, DeckRules, Error, Game, Keywords, LibraryDestination, ObjectRef,
    ObjectView, Observation, Player, SpellView, StackKind, Zone,
};
use std::collections::{BTreeMap, BTreeSet};
const NAMES: [&str; 9] = [
    "HYDRA Assault Robot",
    "Impact Tremors",
    "Marketwatch Phantom",
    "Neighborhood Guardian",
    "Snarling Gorehound",
    "Valley Mightcaller",
    "Vial Smasher, Gleeful Grenadier",
    "Vicious Clown",
    "Warleader's Call",
];
type Ledger = BTreeMap<ObjectRef, (u32, i128, i128, Keywords, u64)>;
fn index(c: Card) -> Option<usize> {
    NAMES.iter().position(|n| *n == c.name())
}
fn deck(f: usize) -> Vec<Card> {
    let spells = match f {
        0 => vec![
            NAMES[0], NAMES[1], NAMES[2], NAMES[3], NAMES[4], NAMES[5], NAMES[6], NAMES[7],
            NAMES[8],
        ],
        1 => vec![
            NAMES[0],
            NAMES[1],
            NAMES[2],
            NAMES[4],
            NAMES[5],
            NAMES[6],
            NAMES[7],
            "Swab Goblin",
            "Gigantoad",
        ],
        _ => vec![
            NAMES[2],
            NAMES[3],
            NAMES[4],
            NAMES[5],
            NAMES[7],
            NAMES[8],
            "Swab Goblin",
            "Gigantoad",
            "Torpor Orb",
        ],
    };
    let mut d = Vec::new();
    for n in ["Plains", "Swamp", "Mountain", "Forest"] {
        d.extend([Card::from_name(n).unwrap(); 6]);
    }
    for n in spells {
        let c = Card::from_name(n).unwrap();
        assert!(c.is_supported() && c.is_standard_legal());
        d.extend([c; 4]);
    }
    assert_eq!(d.len(), 60);
    d
}
fn expected(v: &ObjectView, o: &Observation, l: &Ledger) -> (i128, i128, Keywords, u64) {
    let (mut p, mut t, mut k) = match v.card.name() {
        "HYDRA Assault Robot" => (1, 3, Keywords::NONE),
        "Marketwatch Phantom" | "Neighborhood Guardian" | "Swab Goblin" => (2, 2, Keywords::NONE),
        "Snarling Gorehound" => (1, 1, Keywords::MENACE),
        "Valley Mightcaller" => (1, 1, Keywords::TRAMPLE),
        "Vial Smasher, Gleeful Grenadier" => (3, 2, Keywords::NONE),
        "Vicious Clown" => (2, 3, Keywords::NONE),
        "Gigantoad" => {
            let boost = if o
                .objects
                .iter()
                .filter(|x| x.owner == v.owner && x.zone == Zone::Battlefield && x.card.is_land())
                .count()
                >= 7
            {
                2
            } else {
                0
            };
            (4 + boost, 4 + boost, Keywords::NONE)
        }
        _ => panic!("unexpected creature {}", v.card.name()),
    };
    let calls = o
        .objects
        .iter()
        .filter(|x| x.owner == v.owner && x.zone == Zone::Battlefield && x.card.name() == NAMES[8])
        .count() as i128;
    p += calls;
    t += calls;
    let mut counters = 0;
    if let Some((turn, gp, gt, gk, c)) = l.get(&v.reference) {
        counters = *c;
        p += i128::from(*c);
        t += i128::from(*c);
        if *turn == o.turn {
            p += gp;
            t += gt;
            k = k.union(*gk);
        }
    }
    (p, t, k, counters)
}
fn check(o: &Observation, l: &Ledger) -> u64 {
    let mut pairs = 0;
    for v in o
        .objects
        .iter()
        .filter(|v| v.zone == Zone::Battlefield && v.card.is_creature())
    {
        let (p, t, k, c) = expected(v, o, l);
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
fn matches(i: usize, s: &ObjectView, v: &ObjectView, o: &Observation, l: &Ledger) -> bool {
    if s.owner != v.owner {
        return false;
    }
    match i {
        0 => s.reference != v.reference && (v.card.is_artifact() || v.card.has_subtype("Villain")),
        1 | 8 => v.card.is_creature(),
        2 | 3 | 4 | 7 => {
            s.reference != v.reference && v.card.is_creature() && expected(v, o, l).0 <= 2
        }
        5 => {
            s.reference != v.reference
                && ["Frog", "Rabbit", "Raccoon", "Squirrel"]
                    .iter()
                    .any(|t| v.card.has_subtype(t))
        }
        6 => {
            s.reference != v.reference
                && ["Assassin", "Mercenary", "Pirate", "Rogue", "Warlock"]
                    .iter()
                    .any(|t| v.card.has_subtype(t))
        }
        _ => false,
    }
}
fn bump(l: &mut Ledger, r: ObjectRef, turn: u32, p: i128, t: i128, k: Keywords, c: u64) {
    let e = l.entry(r).or_insert((turn, 0, 0, Keywords::NONE, 0));
    if e.0 != turn {
        e.0 = turn;
        e.1 = 0;
        e.2 = 0;
        e.3 = Keywords::NONE;
    }
    e.1 += p;
    e.2 += t;
    e.3 = e.3.union(k);
    e.4 += c;
}
fn verify(i: usize, s: &SpellView, b: &Observation, a: &Observation, l: &mut Ledger) {
    let live = |r: ObjectRef| {
        a.objects
            .iter()
            .any(|v| v.reference == r && v.zone == Zone::Battlefield)
    };
    match i {
        0 | 1 | 6 | 8 => assert_eq!(
            a.players[s.controller.other().index()].life,
            b.players[s.controller.other().index()].life - 1
        ),
        2 => {
            if live(s.object) {
                bump(l, s.object, a.turn, 0, 0, Keywords::FLYING, 0);
            }
        }
        3 => {
            if let Some(r) = s.targets.first().and_then(|t| t.object)
                && live(r)
            {
                bump(l, r, a.turn, 1, 1, Keywords::NONE, 0);
            }
        }
        4 => {
            assert!(
                a.players[s.controller.index()].library_count
                    <= b.players[s.controller.index()].library_count
            );
            assert!(
                b.players[s.controller.index()].library_count
                    - a.players[s.controller.index()].library_count
                    <= 1
            );
        }
        5 => {
            if live(s.object) {
                bump(l, s.object, a.turn, 0, 0, Keywords::NONE, 1);
            }
        }
        7 => {
            if live(s.object) {
                bump(l, s.object, a.turn, 2, 0, Keywords::NONE, 0);
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
    let (mut decisions, mut replays, mut pairs, mut private, mut blocked, mut max_bytes) =
        (0u64, 0u64, 0u64, 0u64, 0u64, 0usize);
    let (mut casts, mut triggers, mut verified) = ([0u64; 9], [0u64; 9], [0u64; 9]);
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
            BTreeMap::<(ObjectRef, u16), u64>::new(),
            BTreeMap::<u64, (usize, SpellView, Observation)>::new(),
        );
        for _ in 0..100_000 {
            if g.outcome().is_some() {
                break;
            }
            let actor = g.actor().unwrap();
            let b = g.observe(actor);
            let actions = g.legal_actions();
            let fallback = rustmage::policy::smoke_index(&g, &actions);
            let selected = actions
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
                    Action::Cast { .. } | Action::AnnounceCast { .. } => 250,
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
            if b.library_choice.is_some() {
                assert!(g.observe(actor.other()).library_choice.is_none());
                private += 1;
            }
            let boundary = b.announcing.is_some()
                || b.resolving.is_some()
                || !b.stack.is_empty()
                || b.objects
                    .iter()
                    .any(|v| v.zone == Zone::Battlefield && index(v.card).is_some());
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
                let object = a.objects.iter().find(|v| v.id == *card);
                if bury {
                    assert_eq!(object.unwrap().zone, Zone::Graveyard);
                } else {
                    assert!(object.is_none(), "library choice must become private again");
                }
            }
            if matches!(action, Action::Pass)
                && let Some(root) = b.stack.last()
                && matches!(root.kind, StackKind::TriggeredAbility { .. })
                && !a.stack.iter().any(|s| s.id == root.id)
            {
                let i = root.source().and_then(|source| index(source.card)).unwrap();
                pending.insert(root.id, (i, root.clone(), b.clone()));
            }
            let finished: Vec<_> = pending
                .keys()
                .copied()
                .filter(|id| {
                    !a.stack.iter().any(|s| s.id == *id)
                        && !a.resolving.as_ref().is_some_and(|s| s.id == *id)
                })
                .collect();
            for id in finished {
                let (i, s, before) = pending.remove(&id).unwrap();
                if a.outcome.is_none() {
                    verify(i, &s, &before, &a, &mut l);
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
                    if let Some(i) = index(s.card)
                        && matches(i, s, v, &a, &l)
                    {
                        if prohibit && v.card.is_creature() {
                            blocked += 1;
                        } else {
                            *debt.entry((s.reference, 0)).or_default() += 1;
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
                    if let Some(i) = item.source().and_then(|source| index(source.card)) {
                        casts[i] += 1;
                    }
                } else if let StackKind::TriggeredAbility { ability } = item.kind {
                    let i = item.source().and_then(|source| index(source.card)).unwrap();
                    let count = debt.get_mut(&(item.object, ability)).unwrap_or_else(|| {
                        panic!("unexpected trigger {} seed{seed}", item.source().expect("example expects an object-backed packet").card.name())
                    });
                    assert!(*count > 0);
                    *count -= 1;
                    triggers[i] += 1;
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
        "Completed {games} Standard entry-predicate games, {decisions} decisions, casts {casts:?}, triggers {triggers:?}, verified effects {verified:?}, {blocked} suppressed would-be triggers, {pairs} public stat/ability pairs, {private} private choices, {replays} replay boundaries, {max_bytes} maximum initialized logical bytes; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
    Ok(())
}
