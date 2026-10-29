//! Public Standard play with independent, commit-aware library-selection ledgers.
use rustmage::{
    Action, Card, Config, DeckRules, Error, Game, Keywords, LibrarySelectionView, ObjectRef,
    ObjectView, Observation, Player, SpellView, StackKind, Zone,
};
use std::collections::BTreeSet;
const NAMES: [&str; 15] = [
    "Accumulate Wisdom",
    "Beastrider Vanguard",
    "Commune with Beavers",
    "Commune with Evil",
    "Commune with Nature",
    "Confounding Riddle",
    "Flow State",
    "Follow the Lumarets",
    "Morbius the Living Vampire",
    "Rakshasa's Bargain",
    "Resentful Revelation",
    "Squad Rallier",
    "Stock Up",
    "Boughside Wanderers",
    "Living Phone",
];
fn index(c: Card) -> Option<usize> {
    NAMES.iter().position(|n| *n == c.name())
}
fn deck(f: usize) -> Vec<Card> {
    let mut d = Vec::new();
    for n in ["Plains", "Island", "Swamp", "Mountain", "Forest"] {
        d.extend([Card::from_name(n).unwrap(); 6]);
    }
    for mut n in [
        NAMES[f],
        "Bear Cub",
        "Llanowar Elves",
        "Corrupted Conviction",
        "Abandon Attachments",
        "Fancy Footwork",
        "Quick Study",
    ] {
        if f == 7 && n == "Abandon Attachments" {
            n = "Eumidian Terrabotanist";
        }
        let c = Card::from_name(n).unwrap();
        assert!(c.is_supported() && c.is_standard_legal(), "{n}");
        d.extend([c; 4]);
    }
    d.extend([Card::from_name("Sami's Curiosity").unwrap(); 2]);
    assert_eq!(d.len(), 60);
    d
}
fn kinds(c: Card) -> BTreeSet<&'static str> {
    c.info()
        .type_line
        .split(" — ")
        .next()
        .unwrap()
        .split_whitespace()
        .filter(|t| {
            [
                "Artifact",
                "Battle",
                "Creature",
                "Enchantment",
                "Instant",
                "Kindred",
                "Land",
                "Planeswalker",
                "Sorcery",
            ]
            .contains(t)
        })
        .collect()
}
fn power(c: Card) -> i128 {
    match c.name() {
        "Bear Cub" | "Beastrider Vanguard" | "Living Phone" => 2,
        "Llanowar Elves" => 1,
        "Squad Rallier" | "Morbius the Living Vampire" => 3,
        "Boughside Wanderers" => 4,
        _ => panic!("unreviewed library power {}", c.name()),
    }
}
fn eligible(i: usize, c: Card) -> bool {
    let k = kinds(c);
    match i {
        1 | 13 => k.iter().any(|t| {
            [
                "Artifact",
                "Creature",
                "Enchantment",
                "Land",
                "Planeswalker",
                "Battle",
            ]
            .contains(t)
        }),
        2 => k
            .iter()
            .any(|t| ["Artifact", "Creature", "Land"].contains(t)),
        4 => k.contains("Creature"),
        7 => k.contains("Creature") || k.contains("Land"),
        11 | 14 => k.contains("Creature") && power(c) <= 2,
        _ => true,
    }
}
fn rule(
    i: usize,
    o: &Observation,
    p: Player,
    window: usize,
) -> (usize, usize, bool, bool, &'static str) {
    let grave: Vec<_> = o
        .objects
        .iter()
        .filter(|v| v.zone == Zone::Graveyard && v.owner == p)
        .collect();
    match i {
        0 => (
            3,
            if grave
                .iter()
                .filter(|v| v.card.has_subtype("Lesson"))
                .count()
                >= 3
            {
                window
            } else {
                1
            },
            false,
            false,
            "bottom_any",
        ),
        1 => (3, 1, true, true, "bottom_any"),
        2 => (3, 1, true, true, "bottom_any"),
        3 => (4, 1, false, false, "graveyard"),
        4 => (5, 1, true, true, "bottom_any"),
        5 => (4, 1, false, false, "graveyard"),
        6 => (
            3,
            if grave.iter().any(|v| kinds(v.card).contains("Instant"))
                && grave.iter().any(|v| kinds(v.card).contains("Sorcery"))
            {
                2
            } else {
                1
            },
            false,
            false,
            "bottom_any",
        ),
        7 => (
            4,
            if o.players[p.index()].life_gained_this_turn > 0 {
                2
            } else {
                1
            },
            true,
            true,
            "bottom_random",
        ),
        8 => (3, 1, false, false, "bottom_any"),
        9 => (4, 2, false, false, "graveyard"),
        10 => (3, 1, false, false, "graveyard"),
        11 => (4, 1, true, true, "bottom_random"),
        12 => (5, 2, false, false, "bottom_any"),
        13 => (4, 1, true, true, "bottom_random"),
        14 => (5, 1, true, true, "bottom_random"),
        _ => unreachable!(),
    }
}
fn selection(s: &SpellView) -> Option<usize> {
    let i = s.source().and_then(|source| index(source.card))?;
    match (i, s.kind) {
        (1 | 8 | 11, StackKind::ActivatedAbility { ability: 0 })
        | (13 | 14, StackKind::TriggeredAbility { ability: 0 }) => Some(i),
        (0 | 2 | 3 | 4 | 6 | 7 | 9 | 10 | 12, StackKind::Spell) => Some(i),
        (5, StackKind::Spell) if s.mode == Some(0) => Some(i),
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
fn object(o: &Observation, id: u32) -> Option<&ObjectView> {
    o.objects.iter().find(|v| v.id == id)
}
#[derive(Default)]
struct Metrics {
    frames: [u64; NAMES.len()],
    commits: [u64; NAMES.len()],
    selected: u64,
    grave: u64,
    revealed: u64,
    unrevealed: u64,
    partials: u64,
    ordered: u64,
    random: u64,
    known_top: u64,
    zero: u64,
    short: u64,
    lesson_all: u64,
    flow_two: u64,
    infusion_two: u64,
    morbius_exiled: u64,
    phone_dead: u64,
    flashback: u64,
    landfall: u64,
    counter_modes: [u64; 2],
    counter_outcomes: [u64; 2],
    replays: u64,
    decisions: u64,
    max_bytes: usize,
}
// Known positions are learned only from public private-window observations. No
// engine internals, seed reconstruction, or definition program reads are used.
type Model = Vec<Option<(ObjectRef, Card)>>;
fn frame(i: usize, b: &Observation, f: &LibrarySelectionView, model: &mut Model, m: &mut Metrics) {
    let p = b.actor.unwrap();
    let views = all(f);
    let expected = rule(i, b, p, views.len());
    assert_eq!(
        views.len(),
        expected.0.min(b.players[p.index()].library_count)
    );
    let max = expected
        .1
        .min(views.iter().filter(|v| eligible(i, v.card)).count());
    assert_eq!(
        (f.minimum, f.maximum, f.reveal, f.remainder),
        (
            if expected.2 { 0 } else { max },
            max,
            expected.3,
            expected.4
        )
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
    assert_eq!(model.len(), b.players[p.index()].library_count);
    for (j, v) in f.remaining.iter().enumerate() {
        if let Some((r, c)) = model[j] {
            assert_eq!((v.reference, v.card), (r, c));
            m.known_top += 1;
        }
        model[j] = Some((v.reference, v.card));
    }
    m.frames[i] += 1;
    m.short += u64::from(views.len() < expected.0);
    m.flow_two += u64::from(i == 6 && max == 2);
    m.infusion_two += u64::from(i == 7 && max == 2);
}
fn partial(b: &Observation, a: &Observation, f: &LibrarySelectionView) {
    let p = b.actor.unwrap().index();
    assert_eq!(
        (
            a.players[p].library_count,
            a.players[p].hand_count,
            a.players[p].cards_drawn_this_turn
        ),
        (
            b.players[p].library_count,
            b.players[p].hand_count,
            b.players[p].cards_drawn_this_turn
        )
    );
    assert_eq!(a.known_hand, b.known_hand);
    let after = a.library_selection.as_ref().unwrap();
    assert_eq!(
        (after.minimum, after.maximum, after.reveal, after.remainder),
        (f.minimum, f.maximum, f.reveal, f.remainder)
    );
    for v in all(f) {
        let now = all(after).into_iter().find(|x| x.id == v.id).unwrap();
        assert_eq!(
            (now.reference, now.zone, now.card),
            (v.reference, Zone::Library, v.card)
        );
    }
    let grave = |o: &Observation| {
        o.objects
            .iter()
            .filter(|v| v.zone == Zone::Graveyard)
            .map(|v| v.reference)
            .collect::<BTreeSet<_>>()
    };
    assert_eq!(grave(a), grave(b));
}
fn commit(
    i: usize,
    observations: (&Observation, &Observation, &Observation),
    f: &LibrarySelectionView,
    action: &Action,
    model: &mut Model,
    m: &mut Metrics,
) {
    let (b, a, public) = observations;
    let p = b.actor.unwrap();
    let pi = p.index();
    let n = all(f).len();
    assert_eq!(model.len(), b.players[pi].library_count);
    if i == 3 {
        assert_eq!(a.players[pi].life, b.players[pi].life + 3);
    }
    let removed = f.selected.len()
        + if f.remainder == "graveyard" {
            f.remaining.len()
        } else {
            0
        };
    assert_eq!(
        a.players[pi].library_count,
        b.players[pi].library_count - removed
    );
    assert_eq!(
        a.players[pi].hand_count,
        b.players[pi].hand_count + f.selected.len()
    );
    assert_eq!(
        a.players[pi].cards_drawn_this_turn,
        b.players[pi].cards_drawn_this_turn
    );
    for v in &f.selected {
        let new = object(a, v.id).unwrap();
        assert_eq!(
            (new.zone, new.reference.generation, new.card),
            (Zone::Hand, v.reference.generation + 1, v.card)
        );
        if f.reveal {
            assert!(public.known_hand.contains(&new.reference));
            assert!(object(public, v.id).is_some_and(|v| v.reference == new.reference));
            m.revealed += 1;
        } else {
            assert!(!public.known_hand.contains(&new.reference));
            assert!(object(public, v.id).is_none());
            m.unrevealed += 1;
        }
    }
    for v in &f.remaining {
        if f.remainder == "graveyard" {
            let new = object(a, v.id).unwrap();
            assert_eq!(
                (new.zone, new.reference.generation, new.card),
                (Zone::Graveyard, v.reference.generation + 1, v.card)
            );
            m.grave += 1;
        }
    }
    model.drain(..n);
    match f.remainder {
        "bottom_any" => {
            let mut ordered: Vec<_> = f.bottom.iter().map(|v| (v.reference, v.card)).collect();
            if let Action::OrderLibraryBottom { card } = action {
                let v = f.remaining.iter().find(|v| v.id == *card).unwrap();
                ordered.push((v.reference, v.card));
            }
            model.extend(ordered.into_iter().map(Some));
            m.ordered += 1;
        }
        "bottom_random" => {
            model.extend(std::iter::repeat_n(None, f.remaining.len()));
            m.random += 1;
        }
        "graveyard" => {}
        _ => unreachable!(),
    };
    assert_eq!(model.len(), a.players[pi].library_count);
    m.selected += f.selected.len() as u64;
    m.zero += u64::from(f.selected.is_empty());
    m.commits[i] += 1;
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let games = std::env::args()
        .nth(1)
        .map_or(Ok(100), |s| s.parse::<u64>())?;
    let mut m = Metrics::default();
    let mut owners = [0u64; 2];
    for seed in 0..games {
        let families = [
            seed as usize % NAMES.len(),
            (seed as usize + 7) % NAMES.len(),
        ];
        let mut g = Game::new(Config {
            decks: [deck(families[0]), deck(families[1])],
            seed: seed + 2000,
            starting_player: Player::P0,
            deck_rules: DeckRules::Standard,
        })?;
        let initial = g.observe(Player::P0);
        let mut models: [Model; 2] =
            std::array::from_fn(|p| vec![None; initial.players[p].library_count]);
        let mut frame_ids = BTreeSet::new();
        let mut local = 0;
        while g.outcome().is_none() {
            local += 1;
            assert!(local < 30000, "unfinished seed{seed}");
            let actor = g.actor().unwrap();
            let b = g.observe(actor);
            let actions = g.legal_actions();
            let fallback = rustmage::policy::smoke_index(&g, &actions);
            let family = families[actor.index()];
            let focus = Card::from_name(NAMES[family]).unwrap();
            let focus_live = b
                .objects
                .iter()
                .any(|v| v.owner == actor && v.card == focus && v.zone == Zone::Battlefield);
            let victim = b.objects.iter().any(|v| {
                v.owner == actor
                    && v.zone == Zone::Battlefield
                    && (matches!(v.card.name(), "Bear Cub" | "Llanowar Elves")
                        || matches!(family, 8 | 14) && v.card == focus)
            });
            let counter = b
                .stack
                .iter()
                .any(|s| s.kind == StackKind::Spell && s.controller != actor);
            let wants_counter = family == 5 && seed % 2 == 0;
            let opponent_funding = b.players[actor.other().index()]
                .mana
                .iter()
                .map(|n| u64::from(*n))
                .sum::<u64>()
                + b.objects
                    .iter()
                    .filter(|v| {
                        v.owner == actor.other()
                            && v.zone == Zone::Battlefield
                            && v.card.is_land()
                            && !v.tapped
                    })
                    .count() as u64;
            let selected = actions
                .iter()
                .enumerate()
                .max_by_key(|(j, a)| match a {
                    Action::AnnounceCast { card }
                    | Action::AnnounceFlashback { card }
                    | Action::Cast { card, .. } => {
                        let v = object(&b, *card).unwrap();
                        if matches!(a, Action::Cast { .. }) && index(v.card).is_some() {
                            -100
                        } else if v.card == focus {
                            if wants_counter && (!counter || seed % 4 == 0 && opponent_funding < 4)
                                || focus.is_creature() && focus_live
                            {
                                -100
                            } else {
                                300
                            }
                        } else if v.card.name() == "Corrupted Conviction" {
                            if victim { 220 } else { -100 }
                        } else if v.card.name() == "Eumidian Terrabotanist" && family == 7 {
                            360
                        } else if v.card.name() == "Sami's Curiosity" && family == 7 {
                            340
                        } else if *j == fallback {
                            100
                        } else {
                            0
                        }
                    }
                    Action::ChooseMode { mode }
                        if b.announcing
                            .as_ref()
                            .is_some_and(|s| s.source().is_some_and(|source| source.card == focus) && family == 5) =>
                    {
                        if *mode == u16::from(wants_counter) {
                            400
                        } else {
                            -100
                        }
                    }
                    Action::ChooseAdditionalCost { object: r } => {
                        let v = object(&b, r.card).unwrap();
                        if matches!(family, 8 | 14) && v.card == focus {
                            500
                        } else if matches!(v.card.name(), "Bear Cub" | "Llanowar Elves") {
                            400
                        } else {
                            -100
                        }
                    }
                    Action::SelectLibrary { .. } => {
                        if seed % 4 == 0
                            && b.library_selection.as_ref().is_some_and(|f| f.minimum == 0)
                        {
                            -10
                        } else {
                            500
                        }
                    }
                    Action::FinishLibrarySelection => 400,
                    Action::OrderLibraryBottom { card } => 500 - i64::from(*card),
                    Action::AnnounceActivate { card, .. } | Action::Activate { card, .. } => {
                        if object(&b, *card).is_some_and(|v| v.card == focus)
                            && matches!(family, 1 | 8 | 11)
                        {
                            250
                        } else {
                            -100
                        }
                    }
                    Action::PayCosts { .. } => 600,
                    Action::TapMana { .. } => {
                        if b.decision == "pay_costs" {
                            200
                        } else {
                            -100
                        }
                    }
                    Action::ChooseOptional { accept } => {
                        if *accept {
                            300
                        } else {
                            0
                        }
                    }
                    Action::PayResolutionMana { .. } => {
                        if seed % 4 == 0 {
                            300
                        } else {
                            -100
                        }
                    }
                    Action::DeclineResolutionMana => {
                        if seed % 4 == 0 {
                            -100
                        } else {
                            300
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
            if let Some(f) = &b.library_selection {
                let root = b.resolving.as_ref().unwrap();
                let i = selection(root).unwrap();
                let other = g.observe(actor.other());
                assert!(other.library_selection.is_none() && other.actions.is_empty());
                assert_eq!(root.cast.is_some(), root.kind == StackKind::Spell);
                if frame_ids.insert(root.id) {
                    frame(i, &b, f, &mut models[actor.index()], &mut m);
                    owners[actor.index()] += 1;
                    if i == 8 {
                        let source = object(&b, root.card).unwrap();
                        assert_eq!(source.zone, Zone::Exile);
                        assert_eq!(source.reference.generation, root.object.generation + 1);
                        m.morbius_exiled += 1;
                    }
                    if i == 14 {
                        let source = object(&b, root.card).unwrap();
                        assert_eq!(source.zone, Zone::Graveyard);
                        assert_eq!(source.reference.generation, root.object.generation + 1);
                        m.phone_dead += 1;
                    }
                    if i == 10
                        && root
                            .cast
                            .is_some_and(|c| c.method == rustmage::CastMethod::Flashback)
                    {
                        m.flashback += 1;
                    }
                }
            }
            let boundary = b.library_selection.is_some()
                || b.resolving.is_some()
                || b.stack.iter().any(|s| s.source().and_then(|source| index(source.card)).is_some())
                || b.announcing
                    .as_ref()
                    .is_some_and(|s| s.source().and_then(|source| index(source.card)).is_some());
            let saved = g.snapshot();
            let mut fork = boundary.then(|| g.clone());
            g.apply(actor, action.clone())?;
            if let Some(f) = &mut fork {
                f.apply(actor, action.clone())?;
            }
            let a = g.observe(actor);
            let public = g.observe(actor.other());
            if let Some(f) = &b.library_selection {
                let i = selection(b.resolving.as_ref().unwrap()).unwrap();
                if a.library_selection.is_some() {
                    partial(&b, &a, f);
                    m.partials += 1;
                } else {
                    commit(
                        i,
                        (&b, &a, &public),
                        f,
                        &action,
                        &mut models[actor.index()],
                        &mut m,
                    );
                }
            } else {
                // Ordinary draws consume the known top prefix. Selection's automatic all-card
                // path is checked separately below because putting into hand is not drawing.
                for p in [Player::P0, Player::P1] {
                    let pi = p.index();
                    let before = b.players[pi].library_count;
                    let after = a.players[pi].library_count;
                    if after < before {
                        let moved = before - after;
                        assert_eq!(models[pi].len(), before);
                        for entry in models[pi].drain(..moved).flatten() {
                            let v = object(if p == actor { &a } else { &public }, entry.0.card)
                                .unwrap();
                            assert_eq!(
                                (v.card, v.zone, v.reference.generation),
                                (entry.1, Zone::Hand, entry.0.generation + 1)
                            );
                            m.known_top += 1;
                        }
                    }
                }
            }
            if let Action::ChooseMode { mode } = action
                && b.announcing
                    .as_ref()
                    .is_some_and(|s| s.source().is_some_and(|source| source.card.name() == NAMES[5]))
            {
                m.counter_modes[usize::from(mode)] += 1;
            }
            if matches!(action, Action::Pass)
                && let Some(root) = b.stack.last()
                && root.source().and_then(|source| index(source.card)) == Some(13)
                && matches!(root.kind, StackKind::TriggeredAbility { ability: 1 })
                && !a.stack.iter().any(|s| s.id == root.id)
                && let (Some(old), Some(now)) = (object(&b, root.card), object(&a, root.card))
                && old.reference == now.reference
            {
                assert_eq!(
                    (now.power, now.toughness),
                    (old.power + 2, old.toughness + 2)
                );
                m.landfall += 1;
            }
            if matches!(
                action,
                Action::PayResolutionMana { .. } | Action::DeclineResolutionMana
            ) && let Some(root) = &b.resolving
                && root.source().is_some_and(|source| source.card.name() == NAMES[5])
                && root.mode == Some(1)
            {
                let victim = root.targets[0].object.unwrap();
                let before = b.stack.iter().find(|s| s.object == victim).unwrap();
                let now = object(&a, victim.card).unwrap();
                if matches!(action, Action::PayResolutionMana { .. }) {
                    assert_eq!((now.zone, now.reference), (Zone::Stack, victim));
                    m.counter_outcomes[0] += 1;
                } else {
                    assert_eq!(
                        now.zone,
                        if before
                            .cast
                            .is_some_and(|c| c.method == rustmage::CastMethod::Flashback)
                        {
                            Zone::Exile
                        } else {
                            Zone::Graveyard
                        }
                    );
                    assert_eq!(now.reference.generation, victim.generation + 1);
                    m.counter_outcomes[1] += 1;
                }
            }
            for v in a.objects.iter().filter(|v| {
                v.zone == Zone::Battlefield
                    && [NAMES[1], NAMES[8], NAMES[11], NAMES[13], NAMES[14]]
                        .contains(&v.card.name())
                    && object(&b, v.id).is_none_or(|old| old.zone != Zone::Battlefield)
            }) {
                let (p, t, k) = match index(v.card).unwrap() {
                    1 => (2, 2, Keywords::NONE),
                    8 => (
                        3,
                        1,
                        Keywords::FLYING
                            .union(Keywords::VIGILANCE)
                            .union(Keywords::LIFELINK),
                    ),
                    11 => (3, 4, Keywords::NONE),
                    13 => (4, 4, Keywords::NONE),
                    14 => (2, 1, Keywords::NONE),
                    _ => unreachable!(),
                };
                assert_eq!((v.power, v.toughness, v.keywords), (p, t, k));
            }
            // Auto-selection only occurs when every remaining looked-at card is mandatory.
            if b.library_selection.is_none()
                && a.library_selection.is_none()
                && matches!(action, Action::Pass)
                && let Some(root) = b.stack.last()
                && let Some(i) = selection(root)
                && !a.stack.iter().any(|s| s.id == root.id)
                && a.outcome.is_none()
            {
                let p = root.controller.index();
                let n = rule(i, &b, root.controller, b.players[p].library_count)
                    .0
                    .min(b.players[p].library_count);
                let r = rule(i, &b, root.controller, n);
                if !r.2 && r.1 >= n {
                    assert_eq!(a.players[p].hand_count, b.players[p].hand_count + n);
                    assert_eq!(
                        a.players[p].cards_drawn_this_turn,
                        b.players[p].cards_drawn_this_turn
                    );
                    m.commits[i] += 1;
                    m.lesson_all += u64::from(i == 0 && n == 3);
                } else if r.2 && r.4 == "bottom_random" {
                    assert_eq!(
                        (a.players[p].hand_count, a.players[p].library_count),
                        (b.players[p].hand_count, b.players[p].library_count)
                    );
                    models[p].drain(..n);
                    models[p].extend(std::iter::repeat_n(None, n));
                    m.commits[i] += 1;
                    m.random += 1;
                    m.zero += 1;
                }
            }
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
                m.replays += 1;
            }
            assert_eq!(a.schema, rustmage::OBSERVATION_SCHEMA);
            m.decisions += 1;
            m.max_bytes = m.max_bytes.max(g.state_bytes());
        }
    }
    if games >= 100 {
        assert!(m.frames.iter().all(|n| *n > 0), "frames{:?}", m.frames);
        assert!(m.commits.iter().all(|n| *n > 0), "commits{:?}", m.commits);
        assert!(owners.iter().all(|n| *n > 0));
        assert!(
            m.random > 0
                && m.ordered > 0
                && m.known_top > 0
                && m.revealed > 0
                && m.unrevealed > 0
                && m.morbius_exiled > 0
                && m.phone_dead > 0
                && m.flashback > 0
                && m.landfall > 0
        );
        assert!(m.counter_modes.iter().all(|n| *n > 0));
        assert!(m.counter_outcomes.iter().all(|n| *n > 0));
        assert!(m.lesson_all > 0 && m.flow_two > 0 && m.infusion_two > 0);
    }
    println!(
        "Completed {games} Standard library-selection games, {} decisions, frames {:?}, commits {:?}, owners {owners:?}, {} selected, {} graveyard, {} revealed, {} unrevealed, {} partial commits deferred, {} ordered/{} random remainders, {} known-top comparisons, {} zero selections, {} short windows, {} all-Lesson/{} two-Flow/{} two-infusion branches, {} exiled Morbius/{} dead Phone/{} flashback/{} landfall witnesses, Riddle modes {:?}/counter outcomes {:?}, {} replays, {} maximum logical bytes; catalogue {}",
        m.decisions,
        m.frames,
        m.commits,
        m.selected,
        m.grave,
        m.revealed,
        m.unrevealed,
        m.partials,
        m.ordered,
        m.random,
        m.known_top,
        m.zero,
        m.short,
        m.lesson_all,
        m.flow_two,
        m.infusion_two,
        m.morbius_exiled,
        m.phone_dead,
        m.flashback,
        m.landfall,
        m.counter_modes,
        m.counter_outcomes,
        m.replays,
        m.max_bytes,
        rustmage::CATALOGUE_VERSION
    );
    Ok(())
}
