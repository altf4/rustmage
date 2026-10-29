//! Independent public eligibility, cost, event and private-commit witnesses.
use rustmage::{
    Action, Card, Config, DeckRules, Error, Game, Keywords, LibrarySelectionView, ObjectRef,
    ObjectView, Observation, Player, SpellView, StackKind, Zone,
};
use std::collections::{BTreeMap, BTreeSet};
const NAMES: [&str; 12] = [
    "Ashe, Princess of Dalmasca",
    "Casey Jones, Jury-Rig Justiciar",
    "Cowabunga!",
    "Eclipsed Boggart",
    "Eclipsed Elf",
    "Eclipsed Flamekin",
    "Eclipsed Kithkin",
    "Eclipsed Merrow",
    "Frontier Seeker",
    "Master Piandao",
    "Staunch Crewmate",
    "Thundertrap Trainer",
];
fn index(c: Card) -> Option<usize> {
    NAMES.iter().position(|n| *n == c.name())
}
fn types(c: Card) -> BTreeSet<&'static str> {
    c.info()
        .type_line
        .split(" — ")
        .next()
        .unwrap()
        .split_whitespace()
        .collect()
}
fn subtype(c: Card, t: &str) -> bool {
    let printed = c
        .info()
        .type_line
        .split(" — ")
        .nth(1)
        .unwrap_or("")
        .split_whitespace()
        .any(|s| s == t);
    printed
        || c.info().keywords.contains(Keywords::CHANGELING)
            && [
                "Mutant",
                "Ninja",
                "Turtle",
                "Goblin",
                "Elf",
                "Elemental",
                "Kithkin",
                "Merfolk",
                "Mount",
                "Ally",
                "Pirate",
            ]
            .contains(&t)
}
fn eligible(i: usize, c: Card) -> bool {
    let ts = types(c);
    match i {
        0 | 1 => ts.contains("Artifact"),
        2 => ts.contains("Land") || ["Mutant", "Ninja", "Turtle"].iter().any(|t| subtype(c, t)),
        3 => ["Goblin", "Swamp", "Mountain"]
            .iter()
            .any(|t| subtype(c, t)),
        4 => ["Elf", "Swamp", "Forest"].iter().any(|t| subtype(c, t)),
        5 => ["Elemental", "Island", "Mountain"]
            .iter()
            .any(|t| subtype(c, t)),
        6 => ["Kithkin", "Forest", "Plains"]
            .iter()
            .any(|t| subtype(c, t)),
        7 => ["Merfolk", "Plains", "Island"]
            .iter()
            .any(|t| subtype(c, t)),
        8 => ts.contains("Creature") && subtype(c, "Mount") || subtype(c, "Plains"),
        9 => ["Ally", "Equipment", "Lesson"]
            .iter()
            .any(|t| subtype(c, t)),
        10 => ts.contains("Artifact") || subtype(c, "Pirate"),
        11 => !ts.contains("Creature") && !ts.contains("Land"),
        _ => unreachable!(),
    }
}
fn alternatives(i: usize) -> usize {
    match i {
        3 | 4 | 7 => 4,
        5 | 6 => 3,
        _ => 1,
    }
}
fn cost(i: usize, k: usize) -> ([u32; 6], u32) {
    let mut c = [0; 6];
    let generic = match i {
        3 => {
            c[2] = k as u32;
            c[3] = 3 - k as u32;
            0
        }
        4 => {
            c[2] = k as u32;
            c[4] = 3 - k as u32;
            0
        }
        5 => {
            c[1] = k as u32;
            c[3] = 2 - k as u32;
            1
        }
        6 => {
            c[0] = k as u32;
            c[4] = 2 - k as u32;
            0
        }
        7 => {
            c[0] = k as u32;
            c[1] = 3 - k as u32;
            0
        }
        _ => unreachable!(),
    };
    (c, generic)
}
fn deck(i: usize) -> Vec<Card> {
    let mut d = Vec::new();
    let lands = match i {
        3 => vec![("Swamp", 13), ("Mountain", 13), ("Forest", 4)],
        4 => vec![("Swamp", 13), ("Forest", 13), ("Island", 4)],
        5 => vec![("Island", 13), ("Mountain", 13), ("Plains", 4)],
        6 => vec![("Forest", 13), ("Plains", 13), ("Swamp", 4)],
        7 => vec![("Plains", 13), ("Island", 13), ("Mountain", 4)],
        _ => ["Plains", "Island", "Swamp", "Mountain", "Forest"]
            .map(|n| (n, 6))
            .to_vec(),
    };
    for (n, count) in lands {
        d.extend(vec![Card::from_name(n).unwrap(); count]);
    }
    for n in [
        NAMES[i],
        "Three Tree Mascot",
        "Mischievous Sneakling",
        "Goldvein Pick",
        "Abandon Attachments",
        "Prophetic Prism",
        "Bear Cub",
    ] {
        let c = Card::from_name(n).unwrap();
        assert!(c.is_supported() && c.is_standard_legal(), "{n}");
        d.extend([c; 4]);
    }
    d.extend([Card::from_name("Corrupted Conviction").unwrap(); 2]);
    assert_eq!(d.len(), 60);
    d
}
fn object(o: &Observation, id: u32) -> Option<&ObjectView> {
    o.objects.iter().find(|v| v.id == id)
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
fn selection(s: &SpellView) -> Option<usize> {
    let i = s.source().and_then(|source| index(source.card))?;
    match (i, s.kind) {
        (2, StackKind::Spell) => Some(i),
        (11, StackKind::TriggeredAbility { ability: 1 }) => Some(i),
        (0 | 1 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10, StackKind::TriggeredAbility { ability: 0 }) => {
            Some(i)
        }
        _ => None,
    }
}
fn all(f: &LibrarySelectionView) -> Vec<&ObjectView> {
    f.remaining
        .iter()
        .chain(&f.selected)
        .chain(&f.bottom)
        .collect()
}
#[derive(Default)]
struct Metrics {
    frames: [u64; NAMES.len()],
    commits: [u64; NAMES.len()],
    entries: [u64; NAMES.len()],
    hybrid: [[u64; 4]; 5],
    owners: [u64; 2],
    trainer_casts: [u64; 2],
    trainer_entries: [u64; 2],
    trainer_frames: [u64; 2],
    declarations: [u64; 2],
    departed: [u64; 2],
    zero: u64,
    selected: u64,
    partials: u64,
    changeling_yes: u64,
    changeling_no: u64,
    noncreature_yes: u64,
    lands_yes: u64,
    lands_no: u64,
    equipment_no: u64,
    reveals: u64,
    decisions: u64,
    replays: u64,
    max_bytes: usize,
}
fn verify_frame(i: usize, o: &Observation, f: &LibrarySelectionView, m: &mut Metrics) {
    let window = all(f);
    let looked = if matches!(i, 0 | 8) { 5 } else { 4 };
    assert_eq!(
        window.len(),
        looked.min(o.players[o.actor.unwrap().index()].library_count)
    );
    let candidates: BTreeSet<_> = f
        .remaining
        .iter()
        .filter(|v| eligible(i, v.card))
        .map(|v| v.reference)
        .collect();
    assert_eq!(
        f.candidates.iter().copied().collect::<BTreeSet<_>>(),
        candidates
    );
    assert_eq!(
        (f.minimum, f.maximum, f.reveal, f.remainder, f.ordering),
        (
            0,
            usize::from(!candidates.is_empty()),
            true,
            "bottom_random",
            false
        )
    );
    for v in window {
        let yes = eligible(i, v.card);
        let ts = types(v.card);
        if v.card.info().keywords.contains(Keywords::CHANGELING) {
            if yes {
                m.changeling_yes += 1;
            } else {
                m.changeling_no += 1;
            }
            for t in [
                "Plains",
                "Swamp",
                "Mountain",
                "Island",
                "Forest",
                "Equipment",
                "Lesson",
            ] {
                assert!(!subtype(v.card, t));
            }
        }
        m.noncreature_yes += u64::from(yes && !ts.contains("Creature") && !ts.contains("Land"));
        m.lands_yes += u64::from(yes && ts.contains("Land"));
        m.lands_no += u64::from(!yes && ts.contains("Land") && (3..=7).contains(&i));
        m.equipment_no += u64::from(i == 9 && ts.contains("Artifact") && !yes);
    }
    m.frames[i] += 1;
    m.owners[o.actor.unwrap().index()] += 1;
    m.zero += u64::from(candidates.is_empty());
}
fn verify_choice(
    i: usize,
    b: &Observation,
    a: &Observation,
    public: &Observation,
    f: &LibrarySelectionView,
    m: &mut Metrics,
) {
    let p = b.actor.unwrap().index();
    if let Some(next) = &a.library_selection {
        assert_eq!(
            (
                a.players[p].hand_count,
                a.players[p].library_count,
                a.players[p].cards_drawn_this_turn
            ),
            (
                b.players[p].hand_count,
                b.players[p].library_count,
                b.players[p].cards_drawn_this_turn
            )
        );
        assert_eq!(a.known_hand, b.known_hand);
        assert_eq!(
            (next.minimum, next.maximum, next.reveal, next.remainder),
            (f.minimum, f.maximum, f.reveal, f.remainder)
        );
        for old in all(f) {
            let now = all(next).into_iter().find(|v| v.id == old.id).unwrap();
            assert_eq!(
                (now.reference, now.zone, now.card),
                (old.reference, Zone::Library, old.card)
            );
        }
        m.partials += 1;
    } else {
        assert!(f.selected.len() <= 1);
        assert_eq!(
            a.players[p].hand_count,
            b.players[p].hand_count + f.selected.len()
        );
        assert_eq!(
            a.players[p].library_count,
            b.players[p].library_count - f.selected.len()
        );
        assert_eq!(
            a.players[p].cards_drawn_this_turn,
            b.players[p].cards_drawn_this_turn
        );
        for old in &f.selected {
            assert!(eligible(i, old.card));
            let now = object(a, old.id).unwrap();
            assert_eq!(
                (now.reference.generation, now.zone, now.card),
                (old.reference.generation + 1, Zone::Hand, old.card)
            );
            assert!(public.known_hand.contains(&now.reference));
            assert!(object(public, now.id).is_some_and(|v| v.reference == now.reference));
            m.reveals += 1;
        }
        let grave = |o: &Observation| {
            o.objects
                .iter()
                .filter(|v| v.zone == Zone::Graveyard)
                .map(|v| v.reference)
                .collect::<BTreeSet<_>>()
        };
        let before = grave(b);
        let after = grave(a);
        let expected = if b.resolving.as_ref().unwrap().kind == StackKind::Spell {
            let s = b.resolving.as_ref().unwrap();
            BTreeSet::from([ObjectRef {
                card: s.card,
                generation: s.object.generation + 1,
            }])
        } else {
            BTreeSet::new()
        };
        assert_eq!(
            after.difference(&before).copied().collect::<BTreeSet<_>>(),
            expected
        );
        assert!(before.is_subset(&after));
        m.selected += f.selected.len() as u64;
        m.commits[i] += 1;
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let games = std::env::args()
        .nth(1)
        .map_or(Ok(100), |v| v.parse::<u64>())?;
    let mut m = Metrics::default();
    for seed in 0..games {
        let families = [
            seed as usize % NAMES.len(),
            (seed as usize + 5) % NAMES.len(),
        ];
        let mut g = Game::new(Config {
            decks: [deck(families[0]), deck(families[1])],
            seed: seed + 3000,
            starting_player: Player::P0,
            deck_rules: DeckRules::Standard,
        })?;
        let mut seen = BTreeSet::new();
        let mut frames = BTreeSet::new();
        let mut debt: BTreeMap<(ObjectRef, u16), u64> = BTreeMap::new();
        let mut paid = BTreeSet::new();
        let mut costs = [0usize; 2];
        let mut steps = 0;
        while g.outcome().is_none() {
            steps += 1;
            assert!(steps < 30000, "unfinished seed{seed}");
            let actor = g.actor().unwrap();
            let b = g.observe(actor);
            let actions = g.legal_actions();
            let fallback = rustmage::policy::smoke_index(&g, &actions);
            let family = families[actor.index()];
            let focus = Card::from_name(NAMES[family]).unwrap();
            let pay = (seed as usize / NAMES.len() + actor.index()).is_multiple_of(2);
            let available = b.players[actor.index()].mana.iter().sum::<u32>() as usize
                + battlefield(&b)
                    .filter(|v| v.owner == actor && v.card.is_land() && !v.tapped)
                    .count();
            let attack_source = b.stack.iter().rev().find(|s| {
                s.controller == actor
                    && selection(s).is_some_and(|i| i == 0 || i == 9)
                    && object(&b, s.card)
                        .is_some_and(|v| v.reference == s.object && v.zone == Zone::Battlefield)
            });
            let preferred = (seed as usize / NAMES.len() + actor.index() + costs[actor.index()])
                % alternatives(family);
            let mut colored = b.players[actor.index()].mana;
            for v in battlefield(&b).filter(|v| v.owner == actor && !v.tapped) {
                if let Some(c) = ["Plains", "Island", "Swamp", "Mountain", "Forest"]
                    .iter()
                    .position(|n| *n == v.card.name())
                {
                    colored[c] += 1;
                }
            }
            let desired_fundable = if (3..=7).contains(&family) {
                let (need, generic) = cost(family, preferred);
                colored.iter().zip(need).all(|(have, need)| *have >= need)
                    && colored.iter().sum::<u32>() >= need.iter().sum::<u32>() + generic
            } else {
                true
            };
            let selected = actions
                .iter()
                .enumerate()
                .max_by_key(|(j, a)| match a {
                    Action::AnnounceCast { card } | Action::Cast { card, .. } => {
                        let v = object(&b, *card).unwrap();
                        if matches!(a, Action::Cast { .. }) && index(v.card).is_some() {
                            -100
                        } else if v.card == focus {
                            if !desired_fundable
                                || family == 11 && pay && available < 6
                                || v.card.info().legendary
                                    && battlefield(&b).any(|x| x.owner == actor && x.card == focus)
                            {
                                -100
                            } else {
                                350
                            }
                        } else if v.card.name() == "Corrupted Conviction" {
                            if attack_source.is_some() { 500 } else { -100 }
                        } else if *j == fallback {
                            100
                        } else {
                            0
                        }
                    }
                    Action::ChooseAdditionalCost { object: r } => {
                        if attack_source.is_some_and(|s| s.object == *r) {
                            600
                        } else {
                            -100
                        }
                    }
                    Action::PlayLand { card } if (3..=7).contains(&family) => {
                        let (need, _) = cost(family, preferred);
                        let c = ["Plains", "Island", "Swamp", "Mountain", "Forest"]
                            .iter()
                            .position(|n| *n == object(&b, *card).unwrap().card.name())
                            .unwrap();
                        if colored[c] < need[c] { 450 } else { 300 }
                    }
                    Action::ChooseSpellCost { cost } => {
                        500 - i64::from(usize::from(*cost) != preferred)
                    }
                    Action::ChooseOffspring { pay: p } => {
                        if *p == pay {
                            600
                        } else {
                            0
                        }
                    }
                    Action::SelectLibrary { .. } => {
                        if seed % 4 == 0 {
                            -10
                        } else {
                            500
                        }
                    }
                    Action::FinishLibrarySelection => 400,
                    Action::PayCosts { .. } => 700,
                    Action::TapMana { .. } => {
                        if b.decision == "pay_costs" {
                            250
                        } else {
                            -100
                        }
                    }
                    Action::Activate { .. }
                    | Action::BeginActivate { .. }
                    | Action::AnnounceActivate { .. } => -100,
                    Action::ChooseOptional { accept } => {
                        if *accept {
                            300
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
            if let Action::ChooseSpellCost { cost: k } = action {
                let root = b.announcing.as_ref().unwrap();
                if let Some(i) = root.source().and_then(|source| index(source.card)) {
                    assert!((3..=7).contains(&i));
                    for option in &b.cost_choices {
                        let expected = cost(i, usize::from(option.index));
                        assert_eq!((option.cost.colored, option.cost.generic), expected);
                    }
                    m.hybrid[i - 3][usize::from(k)] += 1;
                    costs[actor.index()] += 1;
                }
            }
            if let Some(f) = &b.library_selection {
                let root = b.resolving.as_ref().unwrap();
                let i = selection(root).unwrap();
                let other = g.observe(actor.other());
                assert!(other.library_selection.is_none() && other.actions.is_empty());
                if frames.insert(root.id) {
                    verify_frame(i, &b, f, &mut m);
                    if i == 11 {
                        let copy = root.source_card.is_token();
                        m.trainer_frames[usize::from(copy)] += 1;
                        assert_eq!(root.entering_object, Some(root.object));
                    }
                    if (i == 0 || i == 9)
                        && object(&b, root.card).is_none_or(|v| {
                            v.reference != root.object || v.zone != Zone::Battlefield
                        })
                    {
                        m.departed[usize::from(i == 9)] += 1;
                    }
                }
            }
            let saved = g.snapshot();
            let mut fork = g.clone();
            g.apply(actor, action.clone())?;
            fork.apply(actor, action.clone())?;
            let a = g.observe(actor);
            let other = g.observe(actor.other());
            if matches!(action, Action::Pass)
                && let Some(root) = b.stack.last()
                && selection(root).is_some()
                && !a.stack.iter().any(|s| s.id == root.id)
                && a.outcome.is_none()
                && b.players[root.controller.index()].library_count > 0
            {
                let owner = g.observe(root.controller);
                assert!(
                    owner.library_selection.is_some(),
                    "every nonempty optional window must be observable, including zero eligible cards"
                );
                assert_eq!(owner.resolving.as_ref().unwrap().id, root.id);
            }
            for v in battlefield(&a).filter(|v| {
                object(&b, v.id)
                    .is_none_or(|old| old.reference != v.reference || old.zone != Zone::Battlefield)
            }) {
                if let Some(i) = index(v.card) {
                    let expected = if v.card.is_token() {
                        assert_eq!(i, 11);
                        (1, 1, Keywords::NONE)
                    } else {
                        let (p, t) = [
                            (3, 2),
                            (2, 1),
                            (0, 0),
                            (2, 3),
                            (3, 2),
                            (1, 4),
                            (2, 1),
                            (2, 3),
                            (2, 1),
                            (4, 4),
                            (2, 1),
                            (1, 2),
                        ][i];
                        (
                            p,
                            t,
                            if i == 1 {
                                Keywords::HASTE
                            } else if i == 9 {
                                Keywords::FIRST_STRIKE
                            } else {
                                Keywords::NONE
                            },
                        )
                    };
                    assert_eq!((v.power, v.toughness, v.keywords), expected);
                    m.entries[i] += 1;
                    if i != 0 && i != 9 {
                        let ability = u16::from(i == 11);
                        *debt.entry((v.reference, ability)).or_default() += 1;
                    }
                    if i == 11 {
                        m.trainer_entries[usize::from(v.card.is_token())] += 1;
                        if paid.remove(&v.reference) {
                            *debt.entry((v.reference, 0)).or_default() += 1;
                        }
                    }
                }
            }
            if matches!(action, Action::FinishAttackers) {
                for v in battlefield(&a).filter(|v| v.owner == actor && a.attackers.contains(&v.id))
                {
                    if let Some(i) = index(v.card)
                        && matches!(i, 0 | 9)
                    {
                        *debt.entry((v.reference, 0)).or_default() += 1;
                        m.declarations[usize::from(i == 9)] += 1;
                    }
                }
            }
            for root in items(&a) {
                if root.kind == StackKind::Spell && !a.stack.iter().any(|s| s.id == root.id) {
                    continue;
                }
                if !seen.insert(root.id) {
                    continue;
                }
                match root.kind {
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
                    StackKind::Spell => {
                        if root.source().and_then(|source| index(source.card)) == Some(11) {
                            let p = root.cast.unwrap().offspring.unwrap();
                            m.trainer_casts[usize::from(p)] += 1;
                            if p {
                                paid.insert(ObjectRef {
                                    card: root.card,
                                    generation: root.object.generation + 1,
                                });
                            }
                        }
                    }
                    StackKind::TriggeredAbility { ability } => {
                        if root.source().and_then(|source| index(source.card)).is_some() {
                            let remaining =
                                debt.get_mut(&(root.object, ability)).unwrap_or_else(|| {
                                    panic!(
                                        "unexpected trigger {} {ability}",
                                        root.source_card.name()
                                    )
                                });
                            assert!(*remaining > 0);
                            *remaining -= 1;
                        }
                    }
                    StackKind::ActivatedAbility { .. } => {}
                }
            }
            if a.outcome.is_none() && a.decision == "priority" {
                assert!(
                    debt.values().all(|remaining| *remaining == 0),
                    "a qualified entry or attack trigger was omitted"
                );
            }
            if let Some(f) = &b.library_selection {
                verify_choice(
                    selection(b.resolving.as_ref().unwrap()).unwrap(),
                    &b,
                    &a,
                    &other,
                    f,
                    &mut m,
                );
            }
            assert_eq!(g.snapshot(), fork.snapshot());
            let expected = g.snapshot();
            g.restore(&saved)?;
            assert_eq!(
                g.apply_index(actor, b.revision, 0),
                Err(Error::StaleRevision)
            );
            g.apply(actor, action)?;
            assert_eq!(g.snapshot(), expected);
            assert_eq!(a.schema, rustmage::OBSERVATION_SCHEMA);
            m.replays += 1;
            m.decisions += 1;
            m.max_bytes = m.max_bytes.max(g.state_bytes());
        }
    }
    if games >= 100 {
        assert!(m.frames.iter().all(|n| *n > 0), "frames{:?}", m.frames);
        assert!(m.commits.iter().all(|n| *n > 0));
        for i in 3..=7 {
            assert!(
                m.hybrid[i - 3][..alternatives(i)].iter().all(|n| *n > 0),
                "hybrid{:?}",
                m.hybrid
            );
        }
        assert!(
            m.trainer_casts
                .iter()
                .chain(&m.trainer_entries)
                .chain(&m.trainer_frames)
                .chain(&m.declarations)
                .chain(&m.departed)
                .chain(&m.owners)
                .all(|n| *n > 0)
        );
        assert_eq!(m.selected, m.reveals);
        assert!(m.lands_no > 0 && m.equipment_no > 0);
        assert!(
            m.zero > 0
                && m.changeling_yes > 0
                && m.changeling_no > 0
                && m.noncreature_yes > 0
                && m.lands_yes > 0
        );
    }
    println!(
        "Completed {games} filtered-library Standard games, {} decisions/replays, frames {:?}, commits {:?}, entries {:?}, hybrid equivalents {:?}, owners {:?}, Trainer unpaid/paid casts {:?}, parent/copy entries {:?}/frames {:?}, Ashe/Piandao attacks {:?}/departed {:?}, {} zero-eligible windows, {} selected/revealed, {} partial commits deferred, {} eligible/{} rejected Changeling checks, {} noncreature/{} land eligibility checks, {} rejected off-color lands/{} rejected plain artifacts, {} maximum logical bytes; catalogue {}",
        m.decisions,
        m.frames,
        m.commits,
        m.entries,
        m.hybrid,
        m.owners,
        m.trainer_casts,
        m.trainer_entries,
        m.trainer_frames,
        m.declarations,
        m.departed,
        m.zero,
        m.reveals,
        m.partials,
        m.changeling_yes,
        m.changeling_no,
        m.noncreature_yes,
        m.lands_yes,
        m.lands_no,
        m.equipment_no,
        m.max_bytes,
        rustmage::CATALOGUE_VERSION
    );
    Ok(())
}
