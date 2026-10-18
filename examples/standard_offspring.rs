//! Public independent copy, event, effect, statistics and rollback witnesses.
use rustmage::{
    Action, Card, Config, DeckRules, Error, Game, Keywords, ObjectRef, ObjectView, Observation,
    Player, SpellView, StackKind, Target, Zone,
};
use std::collections::{BTreeMap, BTreeSet};
const NAMES: [&str; 10] = [
    "Coruscation Mage",
    "Finch Formation",
    "Flowerfoot Swordmaster",
    "Intrepid Rabbit",
    "Iridescent Vinelasher",
    "Rust-Shield Rampager",
    "Splash Lasher",
    "Starscape Cleric",
    "Steampath Charger",
    "Tender Wildguide",
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
fn deck() -> Vec<Card> {
    let mut d = Vec::new();
    for (n, c) in [
        ("Plains", 5),
        ("Island", 5),
        ("Swamp", 4),
        ("Mountain", 5),
        ("Forest", 5),
    ] {
        d.extend(vec![Card::from_name(n).unwrap(); c]);
    }
    for n in NAMES {
        d.extend([Card::from_name(n).unwrap(); 2]);
    }
    for n in ["Opt", "Giant Growth", "Bear Cub", "Healer's Hawk"] {
        d.extend([Card::from_name(n).unwrap(); 4]);
    }
    assert_eq!(d.len(), 60);
    d
}
struct Stats {
    turn: u32,
    p: i128,
    t: i128,
    k: Keywords,
    c: u64,
}
impl Default for Stats {
    fn default() -> Self {
        Self {
            turn: 0,
            p: 0,
            t: 0,
            k: Keywords::NONE,
            c: 0,
        }
    }
}
type Ledger = BTreeMap<ObjectRef, Stats>;
fn bump(l: &mut Ledger, r: ObjectRef, turn: u32, p: i128, t: i128, k: Keywords) {
    let e = l.entry(r).or_default();
    if e.turn != turn {
        let c = e.c;
        *e = Stats {
            c,
            turn,
            ..Stats::default()
        };
    }
    e.p += p;
    e.t += t;
    e.k = e.k.union(k);
}
fn base(v: &ObjectView) -> (i128, i128, Keywords) {
    if let Some(i) = index(v.card) {
        let k = match i {
            1 => Keywords::FLYING,
            5 => Keywords::NO_POWER_LE_TWO_BLOCKERS,
            7 => Keywords::FLYING.union(Keywords::CANT_BLOCK),
            _ => Keywords::NONE,
        };
        if v.token.is_some() {
            return (1, 1, k);
        }
        let (p, t) = [
            (2, 2),
            (2, 2),
            (1, 2),
            (3, 2),
            (1, 2),
            (4, 4),
            (3, 3),
            (2, 1),
            (2, 1),
            (2, 2),
        ][i];
        return (p, t, k);
    }
    match v.card.name() {
        "Bear Cub" => (2, 2, Keywords::NONE),
        "Healer's Hawk" => (1, 1, Keywords::FLYING.union(Keywords::LIFELINK)),
        n => panic!("unexpected creature{n}"),
    }
}
fn check(o: &Observation, l: &Ledger) -> u64 {
    let mut n = 0;
    for v in battlefield(o).filter(|v| v.card.is_creature()) {
        let (mut p, mut t, mut k) = base(v);
        if let Some(e) = l.get(&v.reference) {
            p += i128::from(e.c);
            t += i128::from(e.c);
        }
        if let Some(e) = l.get(&v.reference).filter(|e| e.turn == o.turn) {
            p += e.p;
            t += e.t;
            k = k.union(e.k);
        }
        assert_eq!(
            (v.power, v.toughness, v.keywords),
            (p, t, k),
            "{} {:?}",
            v.card.name(),
            v.reference
        );
        assert_eq!(v.counters.plus_one, l.get(&v.reference).map_or(0, |e| e.c));
        if let Some(token) = v.token {
            let i = index(v.card).unwrap();
            let parent = Card::from_name(NAMES[i]).unwrap();
            assert_eq!(v.card.info().mana_cost, parent.info().mana_cost);
            assert_eq!(v.card.info().type_line, parent.info().type_line);
            assert_eq!(v.card.colors(), parent.colors());
            assert_eq!(token.name, parent.name());
            assert!(token.factory_identity.starts_with("@token/"));
            assert_eq!(Card::from_name(token.factory_identity), Some(v.card));
        }
        n += 1;
    }
    n
}
fn verify(i: usize, ability: u16, s: &SpellView, b: &Observation, a: &Observation, l: &mut Ledger) {
    if ability == 0 {
        let new: Vec<_> = battlefield(a)
            .filter(|v| {
                v.token.is_some()
                    && v.card.name() == NAMES[i]
                    && !battlefield(b).any(|x| x.reference == v.reference)
            })
            .collect();
        assert_eq!(new.len(), 1);
        assert_eq!(base(new[0]).0, 1);
        assert_eq!((new[0].power, new[0].toughness), (1, 1));
        assert!(!new[0].tapped);
        return;
    }
    let target = s
        .targets
        .first()
        .and_then(|t| t.object)
        .filter(|r| exact(a, *r).is_some());
    match i {
        0 | 4 | 7 | 8 => {
            let p = if i == 4 || i == 8 {
                match s.targets[0].target {
                    Target::Player(p) => p,
                    _ => panic!("player target"),
                }
            } else {
                s.controller.other()
            };
            assert_eq!(a.players[p.index()].life, b.players[p.index()].life - 1);
        }
        1 => {
            if let Some(r) = target {
                bump(l, r, a.turn, 0, 0, Keywords::FLYING);
            }
        }
        2 => {
            for v in battlefield(b).filter(|v| v.owner == s.controller && v.card.name() == NAMES[2])
            {
                if exact(a, v.reference).is_some() {
                    bump(l, v.reference, a.turn, 1, 0, Keywords::NONE);
                }
            }
        }
        3 => {
            if let Some(r) = target {
                bump(l, r, a.turn, 1, 1, Keywords::NONE);
            }
        }
        6 => {
            if let Some(r) = target {
                let v = exact(a, r).unwrap();
                assert!(v.tapped);
                assert_eq!(v.counters.stun, exact(b, r).unwrap().counters.stun + 1);
            }
        }
        _ => panic!("unexpected residual{i}"),
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let games: usize = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "100".into())
        .parse()?;
    let (mut decisions, mut replays, mut pairs, mut private, mut witnessed, mut max_bytes) =
        (0u64, 0u64, 0u64, 0u64, 0u64, 0usize);
    let (mut casts, mut paid, mut unpaid, mut children, mut effects) =
        ([0u64; 10], [0u64; 10], [0u64; 10], [0u64; 10], [0u64; 10]);
    let mut residual = [0u64; 10];
    let (mut tender_mana, mut tender_growth, mut evasion) = ([0u64; 2], [0u64; 2], [0u64; 2]);
    for seed in 0..games {
        let mut g = Game::new(Config {
            decks: [deck(), deck()],
            seed: seed as u64,
            starting_player: Player::P0,
            deck_rules: DeckRules::Standard,
        })?;
        let (mut ledger, mut seen_cast, mut seen_trigger, mut seen_target, mut once, mut debt) = (
            Ledger::new(),
            BTreeSet::new(),
            BTreeSet::new(),
            BTreeSet::new(),
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
            let chosen = actions
                .iter()
                .enumerate()
                .max_by_key(|(j, a)| match a {
                    Action::PayCosts { .. } => 500,
                    Action::ChooseOffspring { pay } => {
                        let wanted =
                            (seed + b.announcing.as_ref().unwrap().card as usize).is_multiple_of(2);
                        if *pay == wanted { 450 } else { 100 }
                    }
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
                    Action::Activate {
                        card, ability: 1, ..
                    } if b
                        .objects
                        .iter()
                        .any(|v| v.id == *card && v.card.name() == NAMES[9])
                        && b.active == actor
                        && seed % 3 == 0 =>
                    {
                        180
                    }
                    Action::Activate { .. }
                    | Action::BeginActivate { .. }
                    | Action::AnnounceActivate { .. } => -100,
                    Action::ChooseTarget {
                        target: Target::Player(p),
                    } => {
                        if *p == actor.other() {
                            400
                        } else {
                            0
                        }
                    }
                    Action::ChooseTarget {
                        target: Target::Card(id),
                    } => {
                        let v = b.objects.iter().find(|v| v.id == *id).unwrap();
                        if v.owner == actor && v.card.name() == NAMES[2] {
                            400
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
            let action = actions[chosen].clone();
            if b.library_choice.is_some() {
                assert!(g.observe(actor.other()).library_choice.is_none());
                private += 1;
            }
            if b.decision == "blockers" {
                for attacker in battlefield(&b)
                    .filter(|v| v.card.name() == NAMES[5] && b.attackers.contains(&v.id))
                {
                    for blocker in battlefield(&b)
                        .filter(|v| v.owner == actor && v.card.is_creature() && v.power <= 2)
                    {
                        assert!(!actions.contains(&Action::Block {
                            blocker: blocker.id,
                            attacker: attacker.id
                        }));
                        evasion[usize::from(attacker.token.is_some())] += 1;
                    }
                }
            }
            let saved = g.snapshot();
            let mut fork = g.clone();
            g.apply(actor, action.clone())?;
            fork.apply(actor, action.clone())?;
            let a = g.observe(actor);
            if let Action::ActivateMana {
                card, production, ..
            } = action
            {
                let source = b.objects.iter().find(|v| v.id == card).unwrap();
                if source.card.name() == NAMES[9] {
                    assert!(!source.tapped);
                    assert!(exact(&a, source.reference).unwrap().tapped);
                    for (i, n) in production.iter().enumerate() {
                        assert_eq!(
                            a.players[actor.index()].mana[i],
                            b.players[actor.index()].mana[i] + n
                        );
                    }
                    tender_mana[usize::from(source.token.is_some())] += 1;
                }
            }
            // Complete casts are independent from declaration proposals and optional payments.
            for s in a.stack.iter().filter(|s| s.kind == StackKind::Spell) {
                if !seen_cast.insert(s.id) {
                    continue;
                }
                if let Some(i) = index(s.source_card) {
                    casts[i] += 1;
                    match s.cast.as_ref().unwrap().offspring {
                        Some(true) => paid[i] += 1,
                        Some(false) => unpaid[i] += 1,
                        None => panic!("uncommitted offspring"),
                    }
                }
                if !s.source_card.is_creature() {
                    for source in battlefield(&a)
                        .filter(|v| v.owner == s.controller && v.card.name() == NAMES[0])
                    {
                        *debt.entry((source.reference, 1)).or_default() += 1;
                    }
                }
            }
            // Entry history carries only the paid parent spell's exact incarnation.
            for v in
                battlefield(&a).filter(|v| !battlefield(&b).any(|old| old.reference == v.reference))
            {
                if let Some(i) = index(v.card) {
                    if v.token.is_none() {
                        if b.stack.iter().any(|s| {
                            s.kind == StackKind::Spell
                                && s.card == v.id
                                && s.cast.as_ref().is_some_and(|c| c.offspring == Some(true))
                        }) {
                            *debt.entry((v.reference, 0)).or_default() += 1;
                        }
                    } else {
                        children[i] += 1;
                    }
                    if matches!(i, 1 | 3 | 6) {
                        *debt.entry((v.reference, 1)).or_default() += 1;
                    }
                }
                if v.card.is_land() {
                    for source in
                        battlefield(&a).filter(|x| x.owner == v.owner && x.card.name() == NAMES[4])
                    {
                        *debt.entry((source.reference, 1)).or_default() += 1;
                    }
                }
            }
            // Every newly bound own spell/ability target is a separate Valiant event.
            for s in items(&a) {
                for target in &s.targets {
                    if let Some(r) = target.object
                        && seen_target.insert((s.id, r))
                        && let Some(source) = exact(&a, r)
                        && source.owner == s.controller
                        && source.card.name() == NAMES[2]
                        && once.insert((r, a.turn))
                    {
                        *debt.entry((r, 1)).or_default() += 1;
                    }
                }
            }
            if a.turn == b.turn {
                for p in [Player::P0, Player::P1] {
                    let delta = a.players[p.index()]
                        .life_gain_events_this_turn
                        .saturating_sub(b.players[p.index()].life_gain_events_this_turn);
                    if delta > 0 {
                        for source in
                            battlefield(&b).filter(|v| v.owner == p && v.card.name() == NAMES[7])
                        {
                            *debt.entry((source.reference, 1)).or_default() += delta;
                        }
                    }
                }
            }
            for v in battlefield(&b)
                .filter(|v| v.card.name() == NAMES[8] && exact(&a, v.reference).is_none())
            {
                *debt.entry((v.reference, 1)).or_default() += 1;
            }
            if matches!(action, Action::Pass)
                && let Some(s) = b.stack.last()
                && !a.stack.iter().any(|x| x.id == s.id)
                && a.outcome.is_none()
            {
                match s.kind {
                    StackKind::DelayedReturn
                    | StackKind::DelayedSacrifice
                    | StackKind::DelayedWarpExile => {
                        panic!("no delayed producer in this fixture")
                    }
                    StackKind::TriggeredAbility { ability } => {
                        let i = index(s.source_card).unwrap();
                        verify(i, ability, s, &b, &a, &mut ledger);
                        if ability == 0 {
                            effects[i] += 1
                        } else {
                            residual[i] += 1;
                        }
                    }
                    StackKind::Spell => {
                        if s.source_card.name() == "Giant Growth"
                            && let Some(r) = s.targets.first().and_then(|t| t.object)
                            && exact(&a, r).is_some()
                        {
                            bump(&mut ledger, r, a.turn, 3, 3, Keywords::NONE);
                        }
                    }
                    StackKind::ActivatedAbility { ability: 1 }
                        if s.source_card.name() == NAMES[9] =>
                    {
                        if let Some(v) = exact(&a, s.object) {
                            let e = ledger.entry(s.object).or_default();
                            e.c += 1;
                            assert_eq!(
                                v.counters.plus_one,
                                exact(&b, s.object).unwrap().counters.plus_one + 1
                            );
                            tender_growth[usize::from(v.token.is_some())] += 1;
                        }
                    }
                    _ => {}
                }
            }
            for s in items(&a) {
                if let StackKind::TriggeredAbility { ability } = s.kind
                    && seen_trigger.insert(s.id)
                {
                    assert_eq!(
                        s.entering_object.is_some(),
                        ability == 0 || matches!(index(s.source_card).unwrap(), 1 | 3 | 4 | 6)
                    );
                    let n = debt.get_mut(&(s.object, ability)).unwrap_or_else(|| {
                        panic!(
                            "unexpected {} ability{ability} seed{seed}",
                            s.source_card.name()
                        )
                    });
                    assert!(*n > 0);
                    *n -= 1;
                    witnessed += 1;
                }
            }
            if a.outcome.is_none() && a.decision == "priority" {
                assert!(
                    debt.values().all(|n| *n == 0),
                    "missing {debt:?} seed{seed}"
                );
            }
            pairs += check(&a, &ledger);
            check(&g.observe(actor.other()), &ledger);
            assert_eq!(a.schema, rustmage::OBSERVATION_SCHEMA);
            let expected = g.snapshot();
            assert_eq!(fork.snapshot(), expected);
            g.restore(&saved)?;
            assert_eq!(
                g.apply_index(actor, b.revision, 0),
                Err(Error::StaleRevision)
            );
            g.apply(actor, action)?;
            assert_eq!(g.snapshot(), expected);
            decisions += 1;
            replays += 1;
            max_bytes = max_bytes.max(g.state_bytes());
        }
        assert!(g.outcome().is_some(), "unfinished{seed}");
    }
    if games >= 100 {
        assert!(
            tender_mana.iter().all(|n| *n > 0),
            "Tender mana {tender_mana:?}"
        );
        assert!(
            tender_growth.iter().all(|n| *n > 0),
            "Tender growth {tender_growth:?}"
        );
        assert!(
            evasion.iter().all(|n| *n > 0),
            "Rampager evasion {evasion:?}"
        );
        assert!(casts.iter().all(|n| *n > 0));
        assert!(paid.iter().all(|n| *n > 0));
        assert!(unpaid.iter().all(|n| *n > 0));
        assert!(effects.iter().all(|n| *n > 0));
        for i in [0, 1, 2, 3, 4, 6, 7, 8] {
            assert!(residual[i] > 0, "missing residual{}", NAMES[i]);
        }
    }
    println!(
        "Completed {games} Standard offspring games, {decisions} decisions, casts{casts:?}, paid{paid:?}, unpaid{unpaid:?}, children{children:?}, verified offspring{effects:?}, residual effects{residual:?}, {witnessed} independently qualified triggers, {pairs} public stat/keyword pairs, {private} private choices, Tender parent/copy mana {tender_mana:?}, counters {tender_growth:?}, Rampager parent/copy forbidden-block checks {evasion:?}, {replays} replay boundaries, {max_bytes} maximum initialized logical bytes; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
    Ok(())
}
