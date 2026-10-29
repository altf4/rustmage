//! Legal Standard games with independent cast-event, effect, stat and replay ledgers.
use rustmage::{
    Action, Card, Config, DeckRules, Error, Game, Keywords, LibraryDestination, ObjectRef,
    ObjectView, Observation, Player, SpellView, StackKind, Zone,
};
use std::collections::{BTreeMap, BTreeSet};
const NAMES: [&str; 13] = [
    "Angry Rabble",
    "Brineborn Cutthroat",
    "Enraged Flamecaster",
    "Kulrath Mystic",
    "Lurking Lizards",
    "Mage Tower Referee",
    "Nightmare Sower",
    "Skybeast Tracker",
    "Solarium Sentry",
    "Spider Manifestation",
    "Stormkeld Prowler",
    "Tanufel Rimespeaker",
    "Unwelcome Sprite",
];
const BASE: [(i128, i128); 13] = [
    (2, 2),
    (2, 1),
    (3, 2),
    (2, 4),
    (1, 3),
    (2, 1),
    (2, 3),
    (2, 4),
    (3, 3),
    (2, 2),
    (2, 1),
    (2, 4),
    (2, 1),
];
fn index(c: Card) -> Option<usize> {
    NAMES.iter().position(|n| *n == c.name())
}
fn intrinsic(i: usize) -> Keywords {
    match i {
        0 | 4 => Keywords::TRAMPLE,
        1 => Keywords::FLASH,
        2 | 7 | 9 => Keywords::REACH,
        6 => Keywords::FLYING.union(Keywords::LIFELINK),
        12 => Keywords::FLYING,
        _ => Keywords::NONE,
    }
}
fn deck(f: usize) -> Vec<Card> {
    let mut d = Vec::new();
    for (n, count) in [
        ("Plains", 4),
        ("Island", 8),
        ("Swamp", 4),
        ("Mountain", 4),
        ("Forest", 4),
    ] {
        d.extend(vec![Card::from_name(n).unwrap(); count]);
    }
    let spells = if f == 0 {
        vec![
            NAMES[0],
            NAMES[2],
            NAMES[3],
            NAMES[4],
            NAMES[7],
            NAMES[9],
            NAMES[10],
            NAMES[11],
            "Defibrillating Current",
        ]
    } else {
        vec![
            NAMES[1],
            NAMES[5],
            NAMES[6],
            NAMES[8],
            NAMES[12],
            "Opt",
            "Lightning Helix",
            "Shivan Dragon",
            "Bear Cub",
        ]
    };
    for n in spells {
        let c = Card::from_name(n).unwrap();
        assert!(c.is_standard_legal() && c.is_supported(), "{n}");
        d.extend([c; 4]);
    }
    assert_eq!(d.len(), 60);
    d
}
// Independent printed-symbol interpretation; does not call the engine's MV/color filters.
fn printed(c: Card) -> (u64, usize) {
    let mut mv = 0;
    let mut colors = BTreeSet::new();
    for piece in c.info().mana_cost.split('{').skip(1) {
        let symbol = piece.split('}').next().unwrap();
        let parts: Vec<_> = symbol.split('/').collect();
        mv += parts
            .iter()
            .filter_map(|s| s.parse::<u64>().ok())
            .max()
            .unwrap_or(1);
        for ch in symbol.chars() {
            if "WUBRG".contains(ch) {
                colors.insert(ch);
            }
        }
    }
    (mv, colors.len())
}
fn qualifies(i: usize, s: &ObjectView, caster: Player, card: Card, active: Player) -> bool {
    let (mv, colors) = printed(card);
    match i {
        0 | 2 | 3 | 4 | 9 | 11 => s.owner == caster && mv >= 4,
        7 | 10 => s.owner == caster && mv >= 5,
        1 | 6 | 12 => s.owner == caster && active != s.owner,
        5 => s.owner == caster && colors >= 2,
        8 => s.owner != caster && mv <= 2,
        _ => false,
    }
}
struct Stats {
    counter: i128,
    turn: u32,
    power: i128,
    keywords: Keywords,
}
impl Default for Stats {
    fn default() -> Self {
        Self {
            counter: 0,
            turn: 0,
            power: 0,
            keywords: Keywords::NONE,
        }
    }
}
type Ledger = BTreeMap<ObjectRef, Stats>;
fn bump(l: &mut Ledger, r: ObjectRef, turn: u32, counter: i128, power: i128, k: Keywords) {
    let e = l.entry(r).or_default();
    e.counter += counter;
    if e.turn != turn {
        e.turn = turn;
        e.power = 0;
        e.keywords = Keywords::NONE;
    }
    e.power += power;
    e.keywords = e.keywords.union(k);
}
fn check(o: &Observation, l: &Ledger) -> u64 {
    let mut count = 0;
    for v in o
        .objects
        .iter()
        .filter(|v| v.zone == Zone::Battlefield && v.card.is_creature())
    {
        let (base, mut flags) = if let Some(i) = index(v.card) {
            (BASE[i], intrinsic(i))
        } else {
            match v.card.name() {
                "Shivan Dragon" => ((5, 5), Keywords::FLYING),
                "Bear Cub" => ((2, 2), Keywords::NONE),
                n => panic!("unexpected {n}"),
            }
        };
        let e = l.get(&v.reference);
        let counter = e.map_or(0, |e| e.counter);
        let power = e.filter(|e| e.turn == o.turn).map_or(0, |e| {
            flags = flags.union(e.keywords);
            e.power
        });
        assert_eq!(
            (v.power, v.toughness, v.keywords),
            (base.0 + counter + power, base.1 + counter, flags),
            "{}",
            v.card.name()
        );
        assert_eq!(
            i128::from(v.counters.plus_one) - i128::from(v.counters.minus_one),
            counter
        );
        count += 1;
    }
    count
}
fn exact(o: &Observation, r: ObjectRef) -> Option<&ObjectView> {
    o.objects
        .iter()
        .find(|v| v.reference == r && v.zone == Zone::Battlefield)
}
fn food(o: &Observation, p: Player) -> usize {
    o.objects
        .iter()
        .filter(|v| {
            v.owner == p
                && v.zone == Zone::Battlefield
                && v.token.is_some_and(|t| t.subtypes == "Food")
        })
        .count()
}
fn verify(i: usize, s: &SpellView, b: &Observation, a: &Observation, l: &mut Ledger) {
    match i {
        0 | 2 => assert_eq!(
            a.players[s.controller.other().index()].life,
            b.players[s.controller.other().index()].life - if i == 0 { 1 } else { 2 }
        ),
        1 | 4 | 5 | 10 => {
            if let Some(v) = exact(b, s.object) {
                let n = if i == 10 { 2 } else { 1 };
                if let Some(after) = exact(a, s.object) {
                    assert_eq!(after.counters.plus_one, v.counters.plus_one + n);
                }
                bump(l, s.object, a.turn, i128::from(n), 0, Keywords::NONE);
            }
        }
        3 => {
            if exact(a, s.object).is_some() {
                bump(l, s.object, a.turn, 0, 2, Keywords::VIGILANCE);
            }
        }
        6 => {
            if let Some(r) = s.targets.first().and_then(|t| t.object)
                && let Some(v) = exact(b, r)
            {
                bump(l, r, a.turn, -1, 0, Keywords::NONE);
                if v.toughness <= 1 {
                    assert!(exact(a, r).is_none());
                }
            }
        }
        7 => assert_eq!(food(a, s.controller), food(b, s.controller) + 1),
        8 => assert_eq!(
            a.players[s.controller.index()].life,
            b.players[s.controller.index()].life + 2
        ),
        9 => {
            if let Some(v) = exact(a, s.object) {
                assert!(!v.tapped);
            }
        }
        11 => assert_eq!(
            a.players[s.controller.index()].library_count,
            b.players[s.controller.index()]
                .library_count
                .saturating_sub(1)
        ),
        12 => assert!(
            b.players[s.controller.index()].library_count
                - a.players[s.controller.index()].library_count
                <= 2
        ),
        _ => unreachable!(),
    }
}
fn live(o: &Observation, id: u64) -> bool {
    o.stack.iter().any(|s| s.id == id) || o.resolving.as_ref().is_some_and(|s| s.id == id)
}
struct SurveilWitness {
    resolution: u64,
    owner: Player,
    initial_count: usize,
    looked: Vec<ObjectRef>,
    selected: Vec<(ObjectRef, LibraryDestination)>,
}
impl SurveilWitness {
    fn new(before: &Observation, owner: Player) -> Self {
        let choice = before.library_choice.as_ref().unwrap();
        assert!(choice.surveil);
        assert!(choice.top.is_empty() && choice.bottom.is_empty() && choice.graveyard.is_empty());
        assert_eq!(
            before.resolving.as_ref().unwrap().source().expect("example expects an object-backed packet").card.name(),
            NAMES[12]
        );
        Self {
            resolution: before.resolving.as_ref().unwrap().id,
            owner,
            initial_count: before.players[owner.index()].library_count,
            looked: choice.remaining.iter().map(|v| v.reference).collect(),
            selected: Vec::new(),
        }
    }
    fn choose(
        &mut self,
        card: u32,
        destination: LibraryDestination,
        before: &Observation,
        after: &Observation,
    ) -> bool {
        assert_eq!(before.resolving.as_ref().unwrap().id, self.resolution);
        let choice = before.library_choice.as_ref().unwrap();
        let reference = choice
            .remaining
            .iter()
            .find(|v| v.id == card)
            .unwrap()
            .reference;
        assert!(self.looked.contains(&reference));
        assert!(!self.selected.iter().any(|(r, _)| *r == reference));
        self.selected.push((reference, destination));
        if let Some(partial) = &after.library_choice {
            assert!(partial.surveil);
            assert_eq!(
                after.players[self.owner.index()].library_count,
                self.initial_count
            );
            assert_eq!(
                partial.remaining.len() + self.selected.len(),
                self.looked.len()
            );
            for (reference, destination) in &self.selected {
                let group = match destination {
                    LibraryDestination::Top => &partial.top,
                    LibraryDestination::Graveyard => &partial.graveyard,
                    LibraryDestination::Bottom
                    | LibraryDestination::Shuffle
                    | LibraryDestination::SecondFromTop => {
                        panic!("invalid surveil destination")
                    }
                };
                let v = group.iter().find(|v| v.reference == *reference).unwrap();
                assert_eq!(v.zone, Zone::Library);
                assert!(
                    !after
                        .objects
                        .iter()
                        .any(|v| v.id == reference.card && v.zone == Zone::Graveyard)
                );
            }
            false
        } else {
            assert_eq!(self.selected.len(), self.looked.len());
            let buried = self
                .selected
                .iter()
                .filter(|(_, d)| *d == LibraryDestination::Graveyard)
                .count();
            assert_eq!(
                after.players[self.owner.index()].library_count + buried,
                self.initial_count
            );
            for (reference, destination) in &self.selected {
                if *destination == LibraryDestination::Graveyard {
                    let v = after
                        .objects
                        .iter()
                        .find(|v| v.id == reference.card)
                        .unwrap();
                    assert_eq!((v.owner, v.zone), (self.owner, Zone::Graveyard));
                    assert_eq!(v.reference.generation, reference.generation + 1);
                } else {
                    assert_eq!(*destination, LibraryDestination::Top);
                    assert!(!after.objects.iter().any(|v| v.id == reference.card));
                }
            }
            true
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let games: usize = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "100".into())
        .parse()?;
    let (mut decisions, mut pairs, mut replay, mut private, mut checks, mut max_bytes) =
        (0u64, 0u64, 0u64, 0u64, 0u64, 0usize);
    let (mut buried, mut partial_burials, mut mixed, mut top_after_bury, mut bury_after_top) =
        (0u64, 0u64, 0u64, 0u64, 0u64);
    let mut double_burials = 0u64;
    let (mut casts, mut triggers, mut verified) = ([0u64; 13], [0u64; 13], [0u64; 13]);
    for seed in 0..games {
        let mut g = Game::new(Config {
            decks: [deck(seed % 2), deck(seed % 2)],
            seed: seed as u64,
            starting_player: Player::P0,
            deck_rules: DeckRules::Standard,
        })?;
        let (mut ledger, mut seen_cast, mut seen_trigger, mut debt, mut pending) = (
            Ledger::new(),
            BTreeSet::new(),
            BTreeSet::new(),
            BTreeMap::<(ObjectRef, u16), u64>::new(),
            BTreeMap::<u64, (usize, SpellView, Observation)>::new(),
        );
        let mut surveil_sequence: Option<SurveilWitness> = None;
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
                .max_by_key(|(j, action)| match action {
                    Action::ArrangeLibrary { destination, .. }
                        if b.library_choice
                            .as_ref()
                            .is_some_and(|choice| choice.surveil) =>
                    {
                        let selected = surveil_sequence
                            .as_ref()
                            .map_or(0, |sequence| sequence.selected.len());
                        let bury = match (seed / 2) % 4 {
                            0 => false,
                            1 => selected.is_multiple_of(2),
                            2 => !selected.is_multiple_of(2),
                            _ => true,
                        };
                        if *destination
                            == if bury {
                                LibraryDestination::Graveyard
                            } else {
                                LibraryDestination::Top
                            }
                        {
                            450
                        } else {
                            -100
                        }
                    }
                    Action::Concede => -1000,
                    Action::PlayLand { .. } => 300,
                    Action::PayCosts { .. } => 500,
                    Action::ChooseSpellCost { cost } => {
                        let c = b
                            .cost_choices
                            .iter()
                            .find(|c| c.index == *cost)
                            .unwrap()
                            .cost;
                        450 - i64::from(c.generic + c.colored.iter().sum::<u32>())
                    }
                    Action::TapMana { .. } | Action::ActivateMana { .. } => {
                        if b.decision == "pay_costs" { 150 } else { -100 }
                    }
                    Action::Activate { .. }
                    | Action::BeginActivate { .. }
                    | Action::AnnounceActivate { .. }
                    | Action::AnnounceMana { .. } => -100,
                    Action::Cast { card, .. } | Action::AnnounceCast { card } => {
                        let c = b.objects.iter().find(|v| v.id == *card).unwrap().card;
                        if c.name() == "Opt" && b.active == actor {
                            -100
                        } else {
                            250
                        }
                    }
                    Action::ChooseTarget {
                        target: rustmage::Target::Player(p),
                    } => {
                        if *p != actor {
                            400
                        } else {
                            -100
                        }
                    }
                    Action::ChooseTarget {
                        target: rustmage::Target::Card(id),
                    } => {
                        let v = b.objects.iter().find(|v| v.id == *id).unwrap();
                        if v.owner != actor { 400 } else { -100 }
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
            if b.library_choice.is_some() {
                let hidden = g.observe(actor.other());
                assert!(hidden.library_choice.is_none() && hidden.actions.is_empty());
                private += 1;
            }
            let saved = g.snapshot();
            let mut fork = g.clone();
            g.apply(actor, action.clone())?;
            fork.apply(actor, action.clone())?;
            let a = g.observe(actor);
            if let Action::ArrangeLibrary { card, destination } = &action
                && b.library_choice
                    .as_ref()
                    .is_some_and(|choice| choice.surveil)
            {
                let sequence =
                    surveil_sequence.get_or_insert_with(|| SurveilWitness::new(&b, actor));
                if sequence.choose(*card, *destination, &b, &a) {
                    let n = sequence
                        .selected
                        .iter()
                        .filter(|(_, d)| *d == LibraryDestination::Graveyard)
                        .count();
                    buried += n as u64;
                    double_burials += u64::from(n == 2);
                    if n > 0 && n < sequence.selected.len() {
                        mixed += 1;
                        top_after_bury += u64::from(*destination == LibraryDestination::Top);
                        bury_after_top += u64::from(*destination == LibraryDestination::Graveyard);
                    }
                    surveil_sequence = None;
                } else if *destination == LibraryDestination::Graveyard {
                    partial_burials += 1;
                }
            }
            // Only completed stack insertion is a cast, never an uncommitted announcement.
            for spell in a.stack.iter().filter(|s| s.kind == StackKind::Spell) {
                if !seen_cast.insert(spell.id) {
                    continue;
                }
                if let Some(i) = spell.source().and_then(|source| index(source.card)) {
                    casts[i] += 1;
                }
                for s in a.objects.iter().filter(|v| v.zone == Zone::Battlefield) {
                    if let Some(i) = index(s.card) {
                        checks += 1;
                        if qualifies(i, s, spell.controller, spell.source().expect("example expects an object-backed packet").card, a.active) {
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
                if !matches!(item.kind, StackKind::TriggeredAbility { .. })
                    || !seen_trigger.insert(item.id)
                {
                    continue;
                }
                let StackKind::TriggeredAbility { ability } = item.kind else {
                    unreachable!()
                };
                let i =
                    item.source().and_then(|source| index(source.card)).expect("only qualified cast triggers in these decks");
                let n = debt.get_mut(&(item.source().expect("example expects an object-backed packet").object, ability)).unwrap_or_else(|| {
                    panic!("unexpected {} trigger seed{seed}", item.source().expect("example expects an object-backed packet").card.name())
                });
                assert!(*n > 0);
                *n -= 1;
                triggers[i] += 1;
            }
            if matches!(action, Action::Pass)
                && let Some(root) = b.stack.last()
                && matches!(root.kind, StackKind::TriggeredAbility { .. })
                && !a.stack.iter().any(|s| s.id == root.id)
            {
                pending.insert(
                    root.id,
                    (root.source().and_then(|source| index(source.card)).unwrap(), root.clone(), b.clone()),
                );
            }
            let done: Vec<_> = pending
                .keys()
                .copied()
                .filter(|id| !live(&a, *id))
                .collect();
            for id in done {
                let (i, s, before) = pending.remove(&id).unwrap();
                if a.outcome.is_none() {
                    verify(i, &s, &before, &a, &mut ledger);
                    verified[i] += 1;
                }
            }
            if a.outcome.is_none() && a.decision == "priority" {
                assert!(
                    debt.values().all(|n| *n == 0),
                    "missing triggers {debt:?} seed{seed}"
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
            replay += 1;
            decisions += 1;
            max_bytes = max_bytes.max(g.state_bytes());
        }
        assert!(g.outcome().is_some(), "game{seed} unfinished");
    }
    if games >= 100 {
        assert!(casts.iter().all(|n| *n > 0), "casts {casts:?}");
        assert!(triggers.iter().all(|n| *n > 0), "triggers {triggers:?}");
        assert!(verified.iter().all(|n| *n > 0), "verified {verified:?}");
        assert!(
            private > 0
                && buried > 0
                && partial_burials > 0
                && mixed > 0
                && top_after_bury > 0
                && bury_after_top > 0
                && double_burials > 0
        );
    }
    println!(
        "Completed {games} Standard qualified-cast games, {decisions} decisions, casts {casts:?}, triggers {triggers:?}, verified effects {verified:?}, {checks} independent cast-source qualification checks, {pairs} public stat/ability pairs, {private} private choices, {buried} committed surveil burials, {partial_burials} deferred burial choices, {double_burials} double-burial sequences, {mixed} mixed-destination surveils ({top_after_bury} ending Top, {bury_after_top} ending Graveyard), {replay} replay boundaries, {max_bytes} maximum initialized logical bytes; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
    Ok(())
}
