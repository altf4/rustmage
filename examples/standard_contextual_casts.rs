//! Independent public cast-target, whole-body effect, statistics and rollback witnesses.
use rustmage::{
    Action, Card, Config, DeckRules, Error, Game, Keywords, LibraryDestination, ObjectRef,
    ObjectView, Observation, Player, SpellView, StackKind, Target, Zone,
};
use std::collections::{BTreeMap, BTreeSet};
const NAMES: [&str; 17] = [
    "Colleen Wing, Street Samurai",
    "Danitha, Spear of Agony",
    "Danitha, Sword of Hope",
    "Firespitter Whelp",
    "Forum Necroscribe",
    "Gnarlback Rhino",
    "Graduation Day",
    "Informed Inkwright",
    "Inkling Mascot",
    "Inkshape Demonstrator",
    "Lecturing Scornmage",
    "Madame Hydra",
    "Melancholic Poet",
    "Mockingbird, Ace Agent",
    "Rehearsed Debater",
    "Rodeo Pyromancers",
    "Stirring Hopesinger",
];
fn index(c: Card) -> Option<usize> {
    NAMES.iter().position(|n| *n == c.name())
}
fn battlefield(o: &Observation) -> impl Iterator<Item = &ObjectView> {
    o.objects.iter().filter(|v| v.zone == Zone::Battlefield)
}
fn exact(o: &Observation, r: ObjectRef) -> Option<&ObjectView> {
    battlefield(o).find(|v| v.reference == r)
}
fn items(o: &Observation) -> impl Iterator<Item = &SpellView> {
    o.stack
        .iter()
        .chain(&o.waiting_triggers)
        .chain(&o.announcing)
        .chain(&o.resolving)
}
fn deck(seed: usize) -> Vec<Card> {
    let mut d = Vec::new();
    for (n, c) in [
        ("Plains", 6),
        ("Island", 4),
        ("Swamp", 4),
        ("Mountain", 4),
        ("Forest", 6),
    ] {
        d.extend(vec![Card::from_name(n).unwrap(); c]);
    }
    for n in [
        NAMES[seed % 17],
        NAMES[(seed + 1) % 17],
        NAMES[(seed + 2) % 17],
        "Giant Growth",
        "Mental Modulation",
        "Lightning Helix",
        "Goldvein Pick",
        "Three Tree Mascot",
        "Bear Cub",
    ] {
        let c = Card::from_name(n).unwrap();
        assert!(c.is_supported() && c.is_standard_legal(), "{n}");
        d.extend([c; 4]);
    }
    assert_eq!(d.len(), 60);
    d
}
struct Stats {
    counter: i128,
    turn: u32,
    p: i128,
    t: i128,
    k: Keywords,
}
impl Default for Stats {
    fn default() -> Self {
        Self {
            counter: 0,
            turn: 0,
            p: 0,
            t: 0,
            k: Keywords::NONE,
        }
    }
}
type Ledger = BTreeMap<ObjectRef, Stats>;
fn bump(l: &mut Ledger, r: ObjectRef, turn: u32, c: i128, p: i128, t: i128, k: Keywords) {
    let e = l.entry(r).or_default();
    e.counter += c;
    if e.turn != turn {
        e.turn = turn;
        e.p = 0;
        e.t = 0;
        e.k = Keywords::NONE;
    }
    e.p += p;
    e.t += t;
    e.k = e.k.union(k);
}
fn base(v: &ObjectView) -> (i128, i128, Keywords) {
    match v.card.name() {
        "Colleen Wing, Street Samurai" | "Inkling Mascot" | "Melancholic Poet" | "Bear Cub" => {
            (2, 2, Keywords::NONE)
        }
        "Danitha, Spear of Agony" | "Danitha, Sword of Hope" => (2, 2, Keywords::FIRST_STRIKE),
        "Firespitter Whelp" => (2, 2, Keywords::FLYING),
        "Forum Necroscribe" => (5, 4, Keywords::NONE),
        "Gnarlback Rhino" => (4, 4, Keywords::TRAMPLE),
        "Informed Inkwright" => (2, 2, Keywords::VIGILANCE),
        "Inkshape Demonstrator" => (3, 4, Keywords::NONE),
        "Lecturing Scornmage" => (1, 1, Keywords::NONE),
        "Madame Hydra" => (2, 3, Keywords::NONE),
        "Mockingbird, Ace Agent" => (2, 2, Keywords::DOUBLE_STRIKE),
        "Rehearsed Debater" => (3, 3, Keywords::VIGILANCE),
        "Rodeo Pyromancers" => (3, 4, Keywords::NONE),
        "Stirring Hopesinger" => (1, 3, Keywords::FLYING.union(Keywords::LIFELINK)),
        "Three Tree Mascot" => (2, 1, Keywords::CHANGELING),
        _ => match v.card.token_info().unwrap().subtypes {
            "Inkling" => (1, 1, Keywords::FLYING),
            "Villain" => (2, 1, Keywords::MENACE),
            x => panic!("unknown token{x}"),
        },
    }
}
fn check(o: &Observation, l: &Ledger) -> u64 {
    let mut n = 0;
    for v in battlefield(o).filter(|v| v.card.is_creature()) {
        let (p, t, mut k) = base(v);
        let e = l.get(&v.reference);
        let c = e.map_or(0, |e| e.counter);
        let (dp, dt) = e.filter(|e| e.turn == o.turn).map_or((0, 0), |e| {
            k = k.union(e.k);
            (e.p, e.t)
        });
        assert_eq!(
            (
                v.power,
                v.toughness,
                v.keywords,
                i128::from(v.counters.plus_one) - i128::from(v.counters.minus_one)
            ),
            (p + c + dp, t + c + dt, k, c),
            "{}",
            v.card.name()
        );
        n += 1;
    }
    n
}
fn qualifies(i: usize, source: &ObjectView, s: &SpellView, o: &Observation) -> bool {
    if source.owner != s.controller {
        return false;
    }
    let targeted = |relation: i8, self_only: bool| {
        s.targets.iter().any(|b| {
            b.object.and_then(|r| exact(o, r)).is_some_and(|v| {
                v.card.is_creature()
                    && (!self_only || v.reference == source.reference)
                    && (relation < 0 || (v.owner == source.owner) == (relation == 0))
            })
        })
    };
    match i {
        0 | 13 => targeted(0, false),
        1 => {
            s.targets
                .iter()
                .any(|b| matches!(b.target,Target::Player(p)if p!=source.owner))
                || targeted(1, false)
        }
        2 => {
            s.source()
                .is_some_and(|source| source.card.has_subtype("Equipment"))
                || targeted(0, false)
        }
        3 => {
            !s.source()
                .expect("example expects an object-backed packet")
                .card
                .is_creature()
                || s.source()
                    .is_some_and(|source| source.card.has_subtype("Dragon"))
        }
        5 => targeted(-1, true),
        11 => s
            .source()
            .is_some_and(|source| source.card.has_subtype("Villain")),
        15 => o.players[s.controller.index()].spells_cast_this_turn == 1,
        _ => {
            matches!(
                s.source()
                    .expect("example expects an object-backed packet")
                    .card
                    .info()
                    .type_line,
                "Instant" | "Sorcery"
            ) && targeted(-1, false)
        }
    }
}
fn verify(i: usize, s: &SpellView, b: &Observation, a: &Observation, l: &mut Ledger) {
    let target = s.targets.first().and_then(|t| t.object);
    match i {
        0 => assert_eq!(
            a.players[s.controller.index()].library_count,
            b.players[s.controller.index()].library_count
        ),
        1 | 10 | 13 => {
            if exact(
                a,
                s.source()
                    .expect("example expects an object-backed packet")
                    .object,
            )
            .is_some()
            {
                bump(
                    l,
                    s.source()
                        .expect("example expects an object-backed packet")
                        .object,
                    a.turn,
                    1,
                    0,
                    0,
                    Keywords::NONE,
                );
            }
        }
        2 | 5 => assert_eq!(
            a.players[s.controller.index()].library_count + 1,
            b.players[s.controller.index()].library_count
        ),
        3 => assert_eq!(
            a.players[s.controller.other().index()].life,
            b.players[s.controller.other().index()].life - 1
        ),
        4 => {
            if let Some(r) = target {
                let previous = b.objects.iter().find(|v| v.reference == r).unwrap();
                assert_eq!(previous.zone, Zone::Graveyard);
                assert!(battlefield(a).any(|v| v.id == r.card && v.reference != r));
            }
        }
        6 => {
            if let Some(r) = target
                && exact(a, r).is_some()
            {
                bump(l, r, a.turn, 1, 0, 0, Keywords::NONE);
            }
        }
        7 | 11 => {
            let subtype = if i == 7 { "Inkling" } else { "Villain" };
            assert_eq!(
                battlefield(a)
                    .filter(|v| v.card.is_token()
                        && v.card.has_subtype(subtype)
                        && !battlefield(b).any(|old| old.reference == v.reference))
                    .count(),
                1
            );
        }
        8 => {}
        9 => {
            if exact(
                a,
                s.source()
                    .expect("example expects an object-backed packet")
                    .object,
            )
            .is_some()
            {
                bump(
                    l,
                    s.source()
                        .expect("example expects an object-backed packet")
                        .object,
                    a.turn,
                    0,
                    1,
                    0,
                    Keywords::LIFELINK,
                );
            }
        }
        12 => {
            assert_eq!(
                a.players[s.controller.index()].life,
                b.players[s.controller.index()].life + 1
            );
            assert_eq!(
                a.players[s.controller.other().index()].life,
                b.players[s.controller.other().index()].life - 1
            );
        }
        14 => {
            if exact(
                a,
                s.source()
                    .expect("example expects an object-backed packet")
                    .object,
            )
            .is_some()
            {
                bump(
                    l,
                    s.source()
                        .expect("example expects an object-backed packet")
                        .object,
                    a.turn,
                    0,
                    1,
                    1,
                    Keywords::NONE,
                );
            }
        }
        15 => assert_eq!(
            a.players[s.controller.index()].mana[3],
            b.players[s.controller.index()].mana[3] + 2
        ),
        16 => {
            for v in battlefield(b).filter(|v| v.owner == s.controller && v.card.is_creature()) {
                if exact(a, v.reference).is_some() {
                    bump(l, v.reference, a.turn, 1, 0, 0, Keywords::NONE);
                }
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
    let (
        mut decisions,
        mut replays,
        mut pairs,
        mut private,
        mut checks,
        mut unannouncable,
        mut wards,
        mut max_bytes,
    ) = (0u64, 0u64, 0u64, 0u64, 0u64, 0u64, 0u64, 0usize);
    let (mut casts, mut triggers, mut effects) = ([0u64; 17], [0u64; 17], [0u64; 17]);
    for seed in 0..games {
        let mut g = Game::new(Config {
            decks: [deck(seed), deck(seed + 8)],
            seed: seed as u64,
            starting_player: Player::P0,
            deck_rules: DeckRules::Standard,
        })?;
        let (mut ledger, mut seen_cast, mut seen_trigger, mut used, mut debt, mut pending) = (
            Ledger::new(),
            BTreeSet::new(),
            BTreeSet::new(),
            BTreeSet::new(),
            BTreeMap::<(ObjectRef, u16), u64>::new(),
            BTreeMap::<u64, (usize, SpellView, Observation)>::new(),
        );
        for _ in 0..100000 {
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
                    Action::TapMana { .. }
                    | Action::ActivateMana { .. }
                    | Action::AnnounceMana { .. } => {
                        if b.decision == "pay_costs" {
                            150
                        } else {
                            -100
                        }
                    }
                    Action::Cast { .. } | Action::AnnounceCast { .. } => 250,
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
            if let Some(choice) = &b.library_choice {
                assert!(g.observe(actor.other()).library_choice.is_none());
                assert_eq!(choice.remaining.len(), 1);
                private += 1;
            }
            let boundary = b.announcing.is_some()
                || b.resolving.is_some()
                || !b.stack.is_empty()
                || battlefield(&b).any(|v| index(v.card).is_some());
            let saved = g.snapshot();
            let mut fork = boundary.then(|| g.clone());
            g.apply(actor, action.clone())?;
            if let Some(f) = &mut fork {
                f.apply(actor, action.clone())?;
            }
            let a = g.observe(actor);
            if let Action::ArrangeLibrary { card, destination } = action {
                assert_eq!(
                    a.players[actor.index()].library_count
                        + usize::from(destination == LibraryDestination::Graveyard),
                    b.players[actor.index()].library_count
                );
                if destination == LibraryDestination::Graveyard {
                    assert!(
                        a.objects
                            .iter()
                            .any(|v| v.id == card && v.zone == Zone::Graveyard)
                    );
                } else {
                    assert!(!a.objects.iter().any(|v| v.id == card));
                }
            }
            for s in a.stack.iter().filter(|s| s.kind == StackKind::Spell) {
                if !seen_cast.insert(s.id) {
                    continue;
                }
                if let Some(i) = s.source().and_then(|source| index(source.card)) {
                    casts[i] += 1;
                }
                assert!(s.entering_object.is_none());
                for source in battlefield(&a) {
                    if let Some(i) = index(source.card) {
                        checks += 1;
                        if qualifies(i, source, s, &a)
                            && (i != 2 || used.insert((source.reference, a.turn)))
                        {
                            *debt
                                .entry((source.reference, u16::from(i == 4 || i == 9)))
                                .or_default() += 1;
                        }
                    }
                }
            }
            if matches!(action, Action::Pass)
                && let Some(s) = b.stack.last()
                && !a.stack.iter().any(|x| x.id == s.id)
            {
                if s.kind == StackKind::Spell {
                    if s.source()
                        .is_some_and(|source| source.card.name() == "Giant Growth")
                        && let Some(r) = s.targets.first().and_then(|t| t.object)
                        && exact(&a, r).is_some()
                    {
                        bump(&mut ledger, r, a.turn, 0, 3, 3, Keywords::NONE);
                    }
                } else if let StackKind::TriggeredAbility { ability } = s.kind
                    && let Some(i) = s.source().and_then(|source| index(source.card))
                    && !(matches!(i, 4 | 9) && ability == 0)
                {
                    if exact(
                        &a,
                        s.source()
                            .expect("example expects an object-backed packet")
                            .object,
                    )
                    .is_some()
                    {
                        if i == 0 {
                            bump(
                                &mut ledger,
                                s.source()
                                    .expect("example expects an object-backed packet")
                                    .object,
                                a.turn,
                                1,
                                0,
                                0,
                                Keywords::NONE,
                            );
                        }
                        if i == 8 {
                            bump(
                                &mut ledger,
                                s.source()
                                    .expect("example expects an object-backed packet")
                                    .object,
                                a.turn,
                                0,
                                0,
                                0,
                                Keywords::FLYING,
                            );
                        }
                    }
                    pending.insert(s.id, (i, s.clone(), b.clone()));
                }
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
                let (i, s, before) = pending.remove(&id).unwrap();
                if a.outcome.is_none() {
                    verify(i, &s, &before, &a, &mut ledger);
                    effects[i] += 1;
                }
            }
            for s in items(&a) {
                if let StackKind::TriggeredAbility { ability } = s.kind {
                    if !seen_trigger.insert(s.id) {
                        continue;
                    }
                    let i = s.source().and_then(|source| index(source.card)).unwrap();
                    assert!(s.entering_object.is_none());
                    if matches!(i, 4 | 9) && ability == 0 {
                        assert!(s.related_stack.is_some());
                        wards += 1;
                        continue;
                    }
                    let n = debt
                        .get_mut(&(
                            s.source()
                                .expect("example expects an object-backed packet")
                                .object,
                            ability,
                        ))
                        .unwrap_or_else(|| {
                            panic!(
                                "unexpected {} seed{seed}",
                                s.source()
                                    .expect("example expects an object-backed packet")
                                    .card
                                    .name()
                            )
                        });
                    assert!(*n > 0);
                    *n -= 1;
                    triggers[i] += 1;
                }
            }
            if a.outcome.is_none() && a.decision == "priority" {
                for ((r, ability), n) in &mut debt {
                    if *n > 0 {
                        let source = a.objects.iter().find(|v| v.id == r.card).unwrap();
                        if source.card.name() == NAMES[4] {
                            assert_eq!(*ability, 1);
                            assert!(!a.objects.iter().any(|v| v.zone == Zone::Graveyard
                                && v.owner == source.owner
                                && v.card.is_creature()));
                        } else {
                            assert_eq!(source.card.name(), NAMES[6]);
                            assert_eq!(*ability, 0);
                            assert!(
                                !battlefield(&a)
                                    .any(|v| v.owner == source.owner && v.card.is_creature())
                            );
                        }
                        unannouncable += *n;
                        *n = 0;
                    }
                }
            }
            pairs += check(&a, &ledger);
            check(&g.observe(actor.other()), &ledger);
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
        assert!(g.outcome().is_some(), "unfinished game{seed}");
    }
    if games >= 100 {
        assert!(casts.iter().all(|n| *n > 0), "casts{casts:?}");
        assert!(triggers.iter().all(|n| *n > 0), "triggers{triggers:?}");
        assert!(effects.iter().all(|n| *n > 0), "effects{effects:?}");
        assert!(private > 0 && checks > 0 && replays > 0);
    }
    println!(
        "Completed {games} Standard contextual-cast games, {decisions} decisions, casts {casts:?}, triggers {triggers:?}, verified effects {effects:?}, {checks} cast/observer checks, {private} private choices, {wards} ward packets, {unannouncable} triggers without legal targets, {pairs} public stat/keyword/counter pairs, {replays} replay boundaries, {max_bytes} maximum initialized logical bytes; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
    Ok(())
}
