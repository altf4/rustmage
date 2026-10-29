//! Full native Standard games with independent conditional keyword/stat checks and replay.
use rustmage::{
    Action, Card, Config, DeckRules, Error, Game, Keywords, Observation, Player, StackKind, Zone,
};
use std::collections::BTreeMap;
const NAMES: [&str; 12] = [
    "Apothecary Stomper",
    "Coliseum Behemoth",
    "Cosmogrand Zenith",
    "Divining Duelist",
    "Downwind Ambusher",
    "Elven Raft-Steerer",
    "Ferocification",
    "Gaius van Baelsar",
    "Giant-Sized Flying Ant",
    "Glamermite",
    "Lord Skitter's Butcher",
    "Wardens of the Cycle",
];
const COUNTS: [usize; 12] = [2, 2, 2, 3, 2, 2, 2, 3, 2, 2, 3, 2];
const BASE: [(i128, i128); 12] = [
    (4, 4),
    (7, 7),
    (2, 4),
    (3, 2),
    (4, 2),
    (3, 2),
    (0, 0),
    (3, 2),
    (3, 2),
    (2, 2),
    (2, 3),
    (3, 4),
];
type Ledger = BTreeMap<rustmage::ObjectRef, (u32, i128, i128, Keywords)>;
fn index(c: Card) -> Option<usize> {
    NAMES.iter().position(|n| *n == c.name())
}
fn deck(f: usize) -> Vec<Card> {
    let mut d = Vec::new();
    let spells = match f {
        0 => vec![
            NAMES[0],
            NAMES[1],
            NAMES[2],
            NAMES[3],
            NAMES[4],
            NAMES[8],
            NAMES[10],
            "Goldvein Pick",
            "Shock",
        ],
        1 => vec![
            NAMES[3], NAMES[4], NAMES[5], NAMES[6], NAMES[7], NAMES[9], NAMES[10], NAMES[11],
            "Shock",
        ],
        _ => vec![
            NAMES[0], NAMES[1], NAMES[2], NAMES[5], NAMES[6], NAMES[7], NAMES[8], NAMES[9],
            NAMES[11],
        ],
    };
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
        assert!(c.is_supported() && c.is_standard_legal(), "{n}");
        d.extend([c; 4]);
    }
    assert_eq!(d.len(), 60);
    d
}
fn choose(g: &Game, o: &Observation, actions: &[Action], seed: usize, uses: &[usize; 12]) -> usize {
    let fallback = rustmage::policy::smoke_index(g, actions);
    let ambush =
        o.objects.iter().any(|v| {
            v.owner == o.actor.unwrap() && v.zone == Zone::Hand && v.card.name() == NAMES[4]
        }) && (seed + 4 + uses[4]) % 2 == 1;
    let damaged = o.objects.iter().any(|v| {
        v.owner != o.actor.unwrap()
            && v.zone == Zone::Battlefield
            && v.card.is_creature()
            && v.damaged_this_turn
    });
    let large = o.objects.iter().any(|v| {
        v.owner != o.actor.unwrap()
            && v.zone == Zone::Battlefield
            && v.card.is_creature()
            && v.toughness > 2
    });
    actions
        .iter()
        .enumerate()
        .max_by_key(|(n, a)| match a {
            Action::Concede => -1000,
            Action::PlayLand { .. } => 300,
            Action::Cast { card, .. } | Action::AnnounceCast { card } => {
                let c = o.objects.iter().find(|v| v.id == *card).unwrap().card;
                if c.name() == NAMES[4] && ambush && !damaged {
                    -100
                } else if c.name() == "Shock" && ambush && large {
                    280
                } else if ambush
                    && !damaged
                    && o.objects
                        .iter()
                        .filter(|v| {
                            v.owner == o.actor.unwrap()
                                && v.zone == Zone::Battlefield
                                && v.card.is_land()
                        })
                        .count()
                        >= 4
                {
                    -100
                } else {
                    250
                }
            }
            Action::ChooseMode { mode } => {
                let root = o.announcing.as_ref().unwrap();
                let i = root.source().and_then(|source| index(source.card)).unwrap();
                if usize::from(*mode) == (seed + i + uses[i]) % COUNTS[i] {
                    350
                } else {
                    200
                }
            }
            Action::ChooseTarget {
                target: rustmage::Target::Card(id),
            } => {
                let v = o.objects.iter().find(|v| v.id == *id).unwrap();
                if o.announcing.as_ref().is_some_and(|s| {
                    s.source()
                        .is_some_and(|source| source.card.name() == "Shock")
                }) && v.owner != o.actor.unwrap()
                    && v.toughness > 2
                {
                    340
                } else if *n == fallback {
                    310
                } else {
                    100
                }
            }
            Action::ChooseOptional { accept } => {
                if *accept {
                    230
                } else {
                    -100
                }
            }
            Action::AnnounceActivate { .. }
            | Action::Activate { .. }
            | Action::BeginActivate { .. }
            | Action::AnnounceMana { .. } => -100,
            Action::TapMana { .. } | Action::ActivateMana { .. } => {
                if matches!(o.decision, "pay_costs" | "resolution_mana") {
                    180
                } else {
                    -100
                }
            }
            Action::Discard { .. } | Action::Sacrifice { .. } | Action::ArrangeLibrary { .. } => {
                if *n == fallback {
                    220
                } else {
                    100
                }
            }
            Action::Attack { .. } => 200,
            _ => {
                if *n == fallback {
                    150
                } else {
                    0
                }
            }
        })
        .unwrap()
        .0
}
fn intrinsic(i: usize) -> Keywords {
    let mut k = Keywords::NONE;
    if i == 0 {
        k = k.union(Keywords::VIGILANCE);
    }
    if i == 1 {
        k = k.union(Keywords::TRAMPLE);
    }
    if [3, 4, 8, 9].contains(&i) {
        k = k.union(Keywords::FLASH);
    }
    if [8, 9].contains(&i) {
        k = k.union(Keywords::FLYING);
    }
    k
}
fn check(o: &Observation, ledger: &Ledger) -> u64 {
    let mut checked = 0;
    for v in &o.objects {
        if v.zone != Zone::Battlefield || !v.card.is_creature() {
            continue;
        }
        let (base, mut keywords) = if let Some(i) = index(v.card) {
            (BASE[i], intrinsic(i))
        } else {
            assert!(v.token.is_some());
            (
                (1, 1),
                if v.token.unwrap().subtypes == "Rat" {
                    Keywords::CANT_BLOCK
                } else {
                    Keywords::NONE
                },
            )
        };
        let (power, toughness) =
            ledger
                .get(&v.reference)
                .filter(|v| v.0 == o.turn)
                .map_or((0, 0), |v| {
                    keywords = keywords.union(v.3);
                    (v.1, v.2)
                });
        let counter = i128::from(v.counters.plus_one) - i128::from(v.counters.minus_one);
        assert_eq!(
            (v.power, v.toughness),
            (base.0 + power + counter, base.1 + toughness + counter),
            "{}",
            v.card.name()
        );
        assert_eq!(v.keywords, keywords, "{}", v.card.name());
        checked += 1;
    }
    checked
}
struct Expected {
    i: usize,
    mode: usize,
    root: rustmage::SpellView,
    before: Observation,
    targets: Vec<rustmage::ObjectView>,
}
fn expected(o: &Observation, s: &rustmage::SpellView, i: usize) -> Option<Expected> {
    if !matches!(s.kind, StackKind::TriggeredAbility { .. }) {
        return None;
    }
    let mode = usize::from(s.mode.unwrap());
    let targets: Vec<_> = s
        .targets
        .iter()
        .filter_map(|b| {
            o.objects
                .iter()
                .find(|v| Some(v.reference) == b.object && v.zone == Zone::Battlefield)
                .filter(|v| i != 4 || mode != 1 || v.damaged_this_turn)
                .cloned()
        })
        .collect();
    if !s.targets.is_empty() && targets.is_empty() {
        return None;
    }
    Some(Expected {
        i,
        mode,
        root: s.clone(),
        before: o.clone(),
        targets,
    })
}
fn ledger_add(
    ledger: &mut Ledger,
    reference: rustmage::ObjectRef,
    turn: u32,
    p: i128,
    t: i128,
    k: Keywords,
) {
    let e = ledger
        .entry(reference)
        .or_insert((turn, 0, 0, Keywords::NONE));
    if e.0 != turn {
        *e = (turn, 0, 0, Keywords::NONE);
    }
    e.1 += p;
    e.2 += t;
    e.3 = e.3.union(k);
}
fn verify(o: &Observation, e: &Expected, ledger: &mut Ledger) {
    let p = e.root.controller;
    let seat = if p == Player::P0 { 0 } else { 1 };
    let before = &e.before;
    let old = &before.players[seat];
    let now = &o.players[seat];
    let own: Vec<_> = before
        .objects
        .iter()
        .filter(|v| v.owner == p && v.zone == Zone::Battlefield)
        .collect();
    let find = |r| {
        o.objects
            .iter()
            .find(|v| v.reference == r && v.zone == Zone::Battlefield)
    };
    let removed = |v: &rustmage::ObjectView| {
        let after = o.objects.iter().find(|a| a.id == v.id);
        if v.token.is_some() {
            assert!(after.is_none());
        } else {
            let a = after.unwrap();
            assert_eq!(a.zone, Zone::Graveyard);
            assert_eq!(a.reference.generation, v.reference.generation + 1);
        }
    };
    let token_count = |obs: &Observation, sub: &str| {
        obs.objects
            .iter()
            .filter(|v| {
                v.owner == p
                    && v.zone == Zone::Battlefield
                    && v.token.is_some_and(|t| t.subtypes == sub)
            })
            .count()
    };
    match (e.i, e.mode) {
        (0, 0) => {
            for v in &e.targets {
                assert_eq!(
                    find(v.reference).unwrap().counters.plus_one,
                    v.counters.plus_one + 2
                );
            }
        }
        (0, 1) => assert_eq!(now.life, old.life + 4),
        (1, 0) | (4, 1) => {
            for v in &e.targets {
                removed(v);
            }
        }
        (1, 1) => assert_eq!(now.library_count, old.library_count.saturating_sub(1)),
        (2, 0) => assert_eq!(
            token_count(o, "Human Soldier"),
            token_count(before, "Human Soldier") + 2
        ),
        (2, 1) => {
            for v in &own {
                if v.card.is_creature() {
                    assert_eq!(
                        find(v.reference).unwrap().counters.plus_one,
                        v.counters.plus_one + 1
                    );
                }
            }
        }
        (3, 0) | (5, 0) | (8, 0) | (9, 0) => {
            for v in &e.targets {
                assert!(find(v.reference).unwrap().tapped);
            }
        }
        (3, 1) | (5, 1) | (8, 1) | (9, 1) => {
            for v in &e.targets {
                assert!(!find(v.reference).unwrap().tapped);
            }
        }
        (3, 2) => {
            assert_eq!(now.library_count, old.library_count.saturating_sub(1));
            assert_eq!(
                now.hand_count,
                (old.hand_count + usize::from(old.library_count > 0)).saturating_sub(1)
            );
        }
        (4, 0) => {
            for v in &e.targets {
                let tough = v.toughness - 1;
                if tough <= 0 || v.damage >= tough {
                    removed(v);
                } else {
                    let a = find(v.reference).unwrap();
                    assert_eq!((a.power, a.toughness), (v.power - 1, tough));
                }
                ledger_add(ledger, v.reference, o.turn, -1, -1, Keywords::NONE);
            }
        }
        (6, 0) => {
            for v in &e.targets {
                assert_eq!(find(v.reference).unwrap().power, v.power + 2);
                ledger_add(ledger, v.reference, o.turn, 2, 0, Keywords::NONE);
            }
        }
        (6, 1) => {
            for v in &e.targets {
                assert!(
                    find(v.reference)
                        .unwrap()
                        .keywords
                        .contains(Keywords::MENACE.union(Keywords::HASTE))
                );
                ledger_add(
                    ledger,
                    v.reference,
                    o.turn,
                    0,
                    0,
                    Keywords::MENACE.union(Keywords::HASTE),
                );
            }
        }
        (7, mode) => {
            for player in [Player::P0, Player::P1] {
                let eligible = |v: &&rustmage::ObjectView| {
                    v.owner == player
                        && v.zone == Zone::Battlefield
                        && match mode {
                            0 => v.card.is_creature() && v.token.is_some(),
                            1 => v.card.is_creature() && v.token.is_none(),
                            2 => v
                                .card
                                .info()
                                .type_line
                                .split(" — ")
                                .next()
                                .unwrap()
                                .split_whitespace()
                                .any(|t| t == "Enchantment"),
                            _ => unreachable!(),
                        }
                };
                let count = before.objects.iter().filter(eligible).count();
                let after = o.objects.iter().filter(eligible).count();
                assert_eq!(after, count.saturating_sub(1));
            }
        }
        (10, 0) => assert_eq!(token_count(o, "Rat"), token_count(before, "Rat") + 1),
        (10, 1) => {
            let can = own.iter().any(|v| {
                v.card.is_creature()
                    && v.reference
                        != e.root
                            .source()
                            .expect("example expects an object-backed packet")
                            .object
            });
            assert_eq!(
                now.library_count,
                old.library_count.saturating_sub(usize::from(can))
            );
        }
        (10, 2) => {
            for v in &own {
                if v.card.is_creature() {
                    assert!(
                        find(v.reference)
                            .unwrap()
                            .keywords
                            .contains(Keywords::MENACE)
                    );
                    ledger_add(ledger, v.reference, o.turn, 0, 0, Keywords::MENACE);
                }
            }
        }
        (11, 0) => assert_eq!(now.life, old.life + 2),
        (11, 1) => {
            assert_eq!(now.life, old.life - 1);
            assert_eq!(now.library_count, old.library_count.saturating_sub(1));
        }
        _ => unreachable!(),
    }
}
fn live(o: &Observation, id: u64) -> bool {
    o.stack.iter().any(|s| s.id == id)
        || o.resolving.as_ref().is_some_and(|s| s.id == id)
        || o.announcing.as_ref().is_some_and(|s| s.id == id)
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let games: usize = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "100".into())
        .parse()?;
    let decks = [deck(0), deck(1), deck(2)];
    let (mut decisions, mut replays, mut private, mut history_checks, mut max_bytes) =
        (0, 0, 0, 0, 0);
    let (mut casts, mut triggers, mut resolutions, mut activations) =
        ([0; 12], [0; 12], [0; 12], [0; 12]);
    let mut effect_checks = 0;
    let mut chosen = [[0u64; 3]; 12];
    let mut verified = [[0u64; 3]; 12];
    let mut checked_stats = 0u64;
    for seed in 0..games {
        let mut g = Game::new(Config {
            decks: [decks[seed % 3].clone(), decks[seed % 3].clone()],
            seed: seed as u64,
            starting_player: Player::P0,
            deck_rules: DeckRules::Standard,
        })?;
        let mut tracked = BTreeMap::new();
        let mut expectations = BTreeMap::<u64, Expected>::new();
        let mut ledger = Ledger::new();
        let mut uses = [0usize; 12];

        for _ in 0..100_000 {
            if g.outcome().is_some() {
                break;
            }
            let p = g.actor().unwrap();
            let before = g.observe(p);
            let actions = g.legal_actions();
            let action = actions[choose(&g, &before, &actions, seed, &uses)].clone();
            if before.library_choice.is_some()
                || before.search_choice.is_some()
                || before.decision == "effect_discard"
            {
                let other = g.observe(p.other());
                assert!(
                    other.library_choice.is_none()
                        && other.search_choice.is_none()
                        && other.effect_discards.is_empty()
                        && other.actions.is_empty()
                );
                private += 1;
            }
            let boundary = before.announcing.is_some()
                || before.resolving.is_some()
                || before
                    .stack
                    .iter()
                    .any(|s| s.source().and_then(|source| index(source.card)).is_some());
            if matches!(action, Action::ChooseMode { .. }) {
                let hidden = g.observe(p.other());
                assert!(hidden.mode_choices.is_empty() && hidden.actions.is_empty());
            }
            let saved = g.snapshot();
            let mut fork = boundary.then(|| g.clone());
            g.apply(p, action.clone())?;
            if let Some(f) = &mut fork {
                f.apply(p, action.clone())?;
            }
            let after = g.observe(p);
            if let Action::ChooseMode { mode } = action {
                let root = before.announcing.as_ref().unwrap();
                let i = root.source().and_then(|source| index(source.card)).unwrap();
                assert!(matches!(root.kind, StackKind::TriggeredAbility { .. }));
                assert!(root.mode.is_none());
                chosen[i][usize::from(mode)] += 1;
                uses[i] += 1;
            }
            if let Some(root) = before.stack.last()
                && let Some(i) = root.source().and_then(|source| index(source.card))
                && !after.stack.iter().any(|v| v.id == root.id)
                && matches!(action, Action::Pass)
                && let Some(e) = expected(&before, root, i)
            {
                expectations.insert(root.id, e);
            }
            for s in &after.stack {
                if before.stack.iter().any(|old| old.id == s.id)
                    || before.resolving.as_ref().is_some_and(|old| old.id == s.id)
                {
                    continue;
                }
                let Some(i) = s.source().and_then(|source| index(source.card)) else {
                    continue;
                };
                if s.kind == StackKind::Spell {
                    casts[i] += 1;
                } else if matches!(s.kind, StackKind::ActivatedAbility { .. }) {
                    activations[i] += 1;
                } else if matches!(s.kind, StackKind::TriggeredAbility { .. }) {
                    assert!(!tracked.contains_key(&s.id));
                    tracked.insert(s.id, i);
                    triggers[i] += 1;
                }
            }
            let completed: Vec<_> = tracked
                .iter()
                .filter(|(id, _)| !live(&after, **id))
                .map(|(&id, &i)| (id, i))
                .collect();
            for (id, i) in completed {
                tracked.remove(&id);
                resolutions[i] += 1;
            }
            let finished: Vec<_> = expectations
                .keys()
                .copied()
                .filter(|id| !live(&after, *id))
                .collect();
            for id in finished {
                let e = expectations.remove(&id).unwrap();
                if after.outcome.is_none() {
                    verify(&after, &e, &mut ledger);
                    verified[e.i][e.mode] += 1;
                    effect_checks += 1;
                }
            }
            let public = g.observe(p.other());
            checked_stats += check(&after, &ledger);
            check(&public, &ledger);
            for o in public
                .objects
                .iter()
                .filter(|o| o.zone == Zone::Hand && o.owner == p)
            {
                assert!(
                    public.known_hand.contains(&o.reference),
                    "private hand leaked"
                );
            }
            assert_eq!(after.schema, rustmage::OBSERVATION_SCHEMA);

            if before.turn == after.turn {
                // This deck has no exile/bounce or zero-toughness incoming
                // creatures. Departed creature tokens cease before observation.
                let died = before
                    .objects
                    .iter()
                    .filter(|b| b.zone == Zone::Battlefield && b.card.is_creature())
                    .filter(|b| match after.objects.iter().find(|o| o.id == b.id) {
                        Some(o) => o.zone == Zone::Graveyard && o.reference != b.reference,
                        None => b.token.is_some(),
                    })
                    .count() as u64;
                assert_eq!(
                    after.creatures_died_this_turn,
                    before.creatures_died_this_turn + died
                );
                history_checks += 1;
            }
            if let Some(f) = fork {
                let expected = g.snapshot();
                assert_eq!(f.snapshot(), expected);
                g.restore(&saved)?;
                assert_eq!(
                    g.apply_index(p, before.revision, 0),
                    Err(Error::StaleRevision)
                );
                g.apply(p, action)?;
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
        assert!(effect_checks > 0);
        for i in 0..12 {
            for mode in 0..COUNTS[i] {
                assert!(
                    chosen[i][mode] > 0 && verified[i][mode] > 0,
                    "missing mode {} {mode}: chosen {chosen:?}, verified {verified:?}",
                    NAMES[i]
                );
            }
        }
        assert!(private > 0);
    }
    println!(
        "Completed {games} Standard modal-trigger games, {decisions} decisions, casts {casts:?}, triggers {triggers:?}, finished triggers {resolutions:?}, activations {activations:?}, modes chosen {chosen:?}, verified modes {verified:?}, {effect_checks} independently verified resolved mode programs, {checked_stats} independently checked public keyword/stat pairs, {history_checks} creature-history checks, {private} private choices, {replays} replay boundaries, {max_bytes} maximum initialized logical bytes; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
    Ok(())
}
