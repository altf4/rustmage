//! Independent public declaration/effect ledger for fourteen complete Standard bodies.
use rustmage::{
    Action, Card, Config, DeckRules, Error, Game, Keywords, ObjectRef, ObjectView, Observation,
    Player, SpellView, StackKind, Zone,
};
use std::collections::{BTreeMap, BTreeSet};
const NAMES: [&str; 14] = [
    "Ancestor Dragon",
    "Armasaur Guide",
    "Battlesong Berserker",
    "Boggart Prankster",
    "Jolene, Plundering Pugilist",
    "Lionheart Glimmer",
    "Malamet Brawler",
    "Marshal of the Lost",
    "Meddling Youths",
    "Most Valuable Slayer",
    "Nori, Teller of Tales",
    "Razorkin Hordecaller",
    "Seasoned Consultant",
    "The Chief Warg",
];
type Ledger = BTreeMap<ObjectRef, (u32, i128, i128, Keywords, u64)>;
fn index(c: Card) -> Option<usize> {
    NAMES.iter().position(|n| *n == c.name())
}
fn battlefield(o: &Observation) -> impl Iterator<Item = &ObjectView> {
    o.objects.iter().filter(|v| v.zone == Zone::Battlefield)
}
fn items(o: &Observation) -> impl Iterator<Item = &SpellView> {
    o.stack
        .iter()
        .chain(&o.waiting_triggers)
        .chain(&o.announcing)
        .chain(&o.resolving)
}
fn deck() -> Vec<Card> {
    let mut d = Vec::new();
    for (n, c) in [
        ("Plains", 8),
        ("Mountain", 6),
        ("Forest", 6),
        ("Swamp", 4),
        ("Dragonlord's Servant", 4),
        ("Bear Cub", 4),
    ] {
        d.extend(vec![Card::from_name(n).unwrap(); c]);
    }
    for n in NAMES {
        let c = Card::from_name(n).unwrap();
        assert!(c.is_supported() && c.is_standard_legal());
        d.extend([c; 2]);
    }
    assert_eq!(d.len(), 60);
    d
}
fn base(v: &ObjectView) -> (i128, i128, Keywords) {
    match v.card.name() {
        "Ancestor Dragon" => (5, 6, Keywords::FLYING),
        "Armasaur Guide" => (4, 4, Keywords::VIGILANCE),
        "Battlesong Berserker" => (3, 4, Keywords::NONE),
        "Boggart Prankster" | "Dragonlord's Servant" => (1, 3, Keywords::NONE),
        "Jolene, Plundering Pugilist" => (4, 2, Keywords::NONE),
        "Lionheart Glimmer" => (2, 5, Keywords::NONE),
        "Malamet Brawler" | "Nori, Teller of Tales" | "Bear Cub" => (2, 2, Keywords::NONE),
        "Marshal of the Lost" => (3, 3, Keywords::DEATHTOUCH),
        "Meddling Youths" => (4, 5, Keywords::HASTE),
        "Most Valuable Slayer" => (2, 4, Keywords::NONE),
        "Razorkin Hordecaller" => (4, 4, Keywords::HASTE),
        "Seasoned Consultant" => (1, 3, Keywords::NONE),
        "The Chief Warg" => (3, 3, Keywords::MENACE),
        _ => {
            assert_eq!(v.card.token_info().unwrap().subtypes, "Gremlin");
            (1, 1, Keywords::NONE)
        }
    }
}
fn expected(v: &ObjectView, o: &Observation, l: &Ledger) -> (i128, i128, Keywords, u64) {
    let (mut p, mut t, mut k) = base(v);
    let mut c = 0;
    if let Some((turn, dp, dt, dk, dc)) = l.get(&v.reference) {
        c = *dc;
        p += i128::from(c);
        t += i128::from(c);
        if *turn == o.turn {
            p += dp;
            t += dt;
            k = k.union(*dk);
        }
    }
    (p, t, k, c)
}
fn check(o: &Observation, l: &Ledger) -> u64 {
    let mut n = 0;
    for v in battlefield(o).filter(|v| v.card.is_creature()) {
        assert_eq!(
            (v.power, v.toughness, v.keywords, v.counters.plus_one),
            expected(v, o, l),
            "{}",
            v.card.name()
        );
        n += 1;
    }
    n
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
fn qualifies(i: usize, s: &ObjectView, o: &Observation, l: &Ledger) -> bool {
    let attacking: Vec<_> = battlefield(o)
        .filter(|v| o.attackers.contains(&v.id) && v.owner == s.owner)
        .collect();
    match i {
        1 | 8 | 12 => attacking.len() >= 3,
        4 => attacking.iter().any(|v| expected(v, o, l).0 >= 4),
        6 | 10 => attacking.iter().any(|v| v.reference == s.reference),
        13 => {
            !attacking.is_empty()
                && battlefield(o)
                    .any(|v| v.owner == s.owner && v.card.is_creature() && expected(v, o, l).0 >= 4)
        }
        _ => !attacking.is_empty(),
    }
}
fn verify(i: usize, s: &SpellView, b: &Observation, a: &Observation, l: &mut Ledger) {
    let live = |r| battlefield(a).any(|v| v.reference == r);
    let target = s
        .targets
        .first()
        .and_then(|t| t.object)
        .filter(|&r| live(r));
    let attackers = battlefield(b)
        .filter(|v| b.attackers.contains(&v.id) && v.card.is_creature())
        .count() as i128;
    match i {
        0 => assert_eq!(
            a.players[s.controller.index()].life,
            b.players[s.controller.index()].life + attackers
        ),
        1 => {
            if let Some(r) = target {
                bump(l, r, a.turn, 0, 0, Keywords::NONE, 1);
            }
        }
        2 | 3 | 6 | 7 | 9 | 10 => {
            if let Some(r) = target {
                let (p, t, k) = match i {
                    2 => (1, 0, Keywords::MENACE),
                    3 => (1, 0, Keywords::NONE),
                    6 => (0, 0, Keywords::TRAMPLE),
                    7 => (attackers, attackers, Keywords::NONE),
                    9 => (1, 0, Keywords::FIRST_STRIKE),
                    10 => (0, 0, Keywords::FIRST_STRIKE),
                    _ => unreachable!(),
                };
                bump(l, r, a.turn, p, t, k, 0);
            }
        }
        4 | 8 | 11 => {
            let subtype = match i {
                4 => "Treasure",
                8 => "Clue",
                _ => "Gremlin",
            };
            assert_eq!(
                battlefield(a)
                    .filter(|v| v.card.is_token()
                        && v.card.has_subtype(subtype)
                        && !battlefield(b).any(|old| old.reference == v.reference))
                    .count(),
                1
            );
        }
        5 => {
            for v in battlefield(b).filter(|v| v.owner == s.controller && v.card.is_creature()) {
                if live(v.reference) {
                    bump(l, v.reference, a.turn, 1, 1, Keywords::NONE, 0);
                }
            }
        }
        12 => {
            if live(s.object) {
                bump(l, s.object, a.turn, 2, 0, Keywords::NONE, 0);
            }
        }
        13 => {
            assert_eq!(
                a.players[s.controller.index()].library_count + 1,
                b.players[s.controller.index()].library_count
            );
            assert_eq!(
                a.players[s.controller.index()].life,
                b.players[s.controller.index()].life - 1
            );
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
        mut declarations,
        mut idle,
        mut rejected_targets,
        mut max_bytes,
    ) = (0u64, 0u64, 0u64, 0u64, 0u64, 0u64, 0usize);
    let (mut casts, mut triggers, mut effects) = ([0u64; 14], [0u64; 14], [0u64; 14]);
    let mut hybrid = [0u64; 2];
    for seed in 0..games {
        let mut g = Game::new(Config {
            decks: [deck(), deck()],
            seed: seed as u64,
            starting_player: Player::P0,
            deck_rules: DeckRules::Standard,
        })?;
        let (mut ledger, mut seen, mut debt) = (
            Ledger::new(),
            BTreeSet::new(),
            BTreeMap::<(ObjectRef, u16), u64>::new(),
        );
        for _ in 0..100000 {
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
                    Action::TapMana { .. }
                    | Action::AnnounceMana { .. }
                    | Action::ActivateMana { .. } => {
                        if b.decision == "pay_costs" {
                            150
                        } else {
                            -100
                        }
                    }
                    Action::Cast { .. } | Action::AnnounceCast { .. } => 250,
                    Action::ChooseSpellCost { cost } => 300 - i32::from(*cost != (seed % 2) as u16),
                    Action::Activate { .. }
                    | Action::BeginActivate { .. }
                    | Action::AnnounceActivate { .. } => -100,
                    Action::Attack { card } | Action::AttackPlaneswalker { card, .. } => {
                        let source = b.objects.iter().find(|v| v.id == *card).unwrap();
                        if seed % 3 == 0 && index(source.card).is_some_and(|i| i != 6 && i != 10) {
                            -100
                        } else if *j == fallback {
                            100
                        } else {
                            0
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
            if let Action::ChooseSpellCost { cost } = &action
                && b.announcing
                    .as_ref()
                    .is_some_and(|s| s.source_card.name() == NAMES[10])
            {
                hybrid[usize::from(*cost)] += 1;
            }
            let boundary = b.announcing.is_some()
                || !b.stack.is_empty()
                || battlefield(&b).any(|v| index(v.card).is_some());
            let saved = g.snapshot();
            let mut fork = boundary.then(|| g.clone());
            g.apply(actor, action.clone())?;
            if let Some(f) = &mut fork {
                f.apply(actor, action.clone())?;
            }
            let a = g.observe(actor);
            if matches!(action, Action::FinishAttackers) && !a.attackers.is_empty() {
                declarations += 1;
                for s in battlefield(&a).filter(|s| s.owner == actor) {
                    if let Some(i) = index(s.card)
                        && qualifies(i, s, &a, &ledger)
                    {
                        *debt.entry((s.reference, u16::from(i == 5))).or_default() += 1;
                        idle += u64::from(!a.attackers.contains(&s.id));
                    }
                }
            }
            if matches!(action, Action::Pass)
                && let Some(s) = b.stack.last()
                && matches!(s.kind, StackKind::TriggeredAbility { .. })
                && !a.stack.iter().any(|x| x.id == s.id)
                && a.outcome.is_none()
            {
                let i = index(s.source_card).unwrap();
                verify(i, s, &b, &a, &mut ledger);
                effects[i] += 1;
            }
            for s in items(&a) {
                if !seen.insert(s.id) {
                    continue;
                }
                assert!(s.entering_object.is_none());
                match s.kind {
                    StackKind::Spell => {
                        if let Some(i) = index(s.source_card) {
                            casts[i] += 1;
                        }
                    }
                    StackKind::TriggeredAbility { ability } => {
                        let i = index(s.source_card).unwrap();
                        let n = debt.get_mut(&(s.object, ability)).unwrap_or_else(|| {
                            panic!("unexpected {} seed{seed}", s.source_card.name())
                        });
                        assert!(*n > 0);
                        *n -= 1;
                        triggers[i] += 1;
                    }
                    StackKind::ActivatedAbility { .. } => assert!(s.source_card.is_token()),
                    StackKind::DelayedReturn
                    | StackKind::DelayedSacrifice
                    | StackKind::DelayedWarpExile => {
                        panic!("no delayed producer in this fixture")
                    }
                }
            }
            // A mandatory targeted attack trigger with zero legal targets is removed during preparation.
            if a.outcome.is_none() && a.decision == "priority" {
                for ((r, _), n) in &mut debt {
                    if *n > 0 {
                        let source = battlefield(&a).find(|v| v.reference == *r).unwrap();
                        assert_eq!(source.card.name(), NAMES[3]);
                        assert!(!battlefield(&a).any(|v| v.owner == source.owner
                            && v.card.has_subtype("Goblin")
                            && a.attackers.contains(&v.id)));
                        rejected_targets += *n;
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
        assert!(hybrid.iter().all(|n| *n > 0));
        assert!(idle > 0 && declarations > 0 && replays > 0);
    }
    println!(
        "Completed {games} Standard attack-declaration games, {decisions} decisions, casts {casts:?}, triggers {triggers:?}, verified effects {effects:?}, Nori hybrid choices {hybrid:?}, {declarations} declarations, {idle} nonattacking observers, {rejected_targets} triggers without legal targets, {pairs} public stat/keyword/counter pairs, {replays} replay boundaries, {max_bytes} maximum initialized logical bytes; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
    Ok(())
}
