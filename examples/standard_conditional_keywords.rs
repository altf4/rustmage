//! Full native Standard games with independent conditional keyword/stat checks and replay.
use rustmage::{
    Action, Card, Config, DeckRules, Error, Game, Keywords, Observation, Player, StackKind, Zone,
};
use std::collections::BTreeMap;
const NAMES: [&str; 9] = [
    "Barrow Naughty",
    "Didact Echo",
    "Dreadwing Scavenger",
    "Echo of Dusk",
    "Goblin Tomb Raider",
    "Gravblade Heavy",
    "Kargan Dragonrider",
    "Spineseeker Centipede",
    "Wildfire Wickerfolk",
];
const BASE: [(i128, i128); 9] = [
    (1, 3),
    (3, 2),
    (2, 2),
    (2, 2),
    (1, 2),
    (3, 4),
    (2, 2),
    (2, 1),
    (3, 2),
];
const BONUS: [(i128, i128); 9] = [
    (0, 0),
    (0, 0),
    (1, 1),
    (1, 1),
    (1, 0),
    (1, 0),
    (0, 0),
    (1, 2),
    (1, 1),
];
const KEYWORDS: [Keywords; 9] = [
    Keywords::LIFELINK,
    Keywords::FLYING,
    Keywords::DEATHTOUCH,
    Keywords::LIFELINK,
    Keywords::HASTE,
    Keywords::DEATHTOUCH,
    Keywords::FLYING,
    Keywords::VIGILANCE,
    Keywords::TRAMPLE,
];
fn index(c: Card) -> Option<usize> {
    NAMES.iter().position(|n| *n == c.name())
}
fn deck(f: usize) -> Vec<Card> {
    let mut d = Vec::new();
    let spells = if f == 0 {
        vec![
            NAMES[0],
            NAMES[1],
            NAMES[2],
            NAMES[3],
            NAMES[5],
            NAMES[7],
            NAMES[8],
            "Three Tree Mascot",
            "Opt",
        ]
    } else {
        vec![
            NAMES[0],
            NAMES[3],
            NAMES[4],
            NAMES[5],
            NAMES[6],
            NAMES[7],
            NAMES[8],
            "Three Tree Mascot",
            "Opt",
        ]
    };
    for n in [
        "Dismal Backwater",
        "Jungle Hollow",
        "Bloodfell Caves",
        "Rugged Highlands",
        "Swiftwater Cliffs",
        "Forest",
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
fn choose(g: &Game, o: &Observation, actions: &[Action]) -> usize {
    let fallback = rustmage::policy::smoke_index(g, actions);
    actions
        .iter()
        .enumerate()
        .max_by_key(|(i, a)| match a {
            Action::Concede => -1000,
            Action::PlayLand { .. } => 300,
            Action::Cast { .. } | Action::AnnounceCast { .. } => 250,
            Action::AnnounceActivate { card, .. } => {
                if o.stack.is_empty()
                    && o.step == rustmage::Step::Attackers
                    && o.objects
                        .iter()
                        .any(|o| o.id == *card && o.card.name() == NAMES[0])
                {
                    240
                } else {
                    -100
                }
            }
            Action::Activate { .. }
            | Action::BeginActivate { .. }
            | Action::AnnounceMana { .. } => -100,
            Action::TapMana { .. } | Action::ActivateMana { .. } => {
                if o.decision == "pay_costs" {
                    180
                } else {
                    -100
                }
            }
            Action::ChooseSearch { .. } => 260,
            Action::Discard { card } => {
                let b = o.objects.iter().find(|b| b.id == *card).unwrap();
                if b.card.name() == "Opt" {
                    250
                } else if b.card.is_artifact() {
                    240
                } else if b.card.is_land() {
                    230
                } else {
                    200
                }
            }
            Action::ArrangeLibrary { destination, .. } => {
                if *destination == rustmage::LibraryDestination::Bottom {
                    240
                } else {
                    -100
                }
            }
            Action::Attack { .. } => 200,
            _ => {
                if *i == fallback {
                    150
                } else {
                    0
                }
            }
        })
        .unwrap()
        .0
}
fn types(c: Card) -> impl Iterator<Item = &'static str> {
    c.info()
        .type_line
        .split(" — ")
        .next()
        .unwrap()
        .split_whitespace()
}
fn subtype(c: Card, t: &str) -> bool {
    c.info()
        .type_line
        .split(" — ")
        .nth(1)
        .is_some_and(|s| s.split_whitespace().any(|v| v == t))
        || c.info().oracle_keywords.contains(&"Changeling")
}
fn check(
    o: &Observation,
    boosted: &mut [u64; 9],
    pumps: &BTreeMap<(u32, u64), (u32, i128)>,
) -> u64 {
    let mut checked = 0;
    for b in &o.objects {
        let Some(i) = index(b.card) else {
            continue;
        };
        if b.zone != Zone::Battlefield {
            continue;
        }
        let own: Vec<_> = o.objects.iter().filter(|c| c.owner == b.owner).collect();
        let grave: Vec<_> = own
            .iter()
            .filter(|c| c.zone == Zone::Graveyard && !c.card.is_token())
            .collect();
        let permanents = grave
            .iter()
            .filter(|c| {
                types(c.card).any(|t| {
                    [
                        "Artifact",
                        "Creature",
                        "Enchantment",
                        "Land",
                        "Planeswalker",
                        "Battle",
                    ]
                    .contains(&t)
                })
            })
            .count();
        let mut grave_types = std::collections::BTreeSet::new();
        for c in &grave {
            for t in types(c.card) {
                if [
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
                .contains(&t)
                {
                    grave_types.insert(t);
                }
            }
        }
        let on = match i {
            0 => own.iter().any(|c| {
                c.zone == Zone::Battlefield
                    && c.reference != b.reference
                    && subtype(c.card, "Faerie")
            }),
            1 | 3 => permanents >= 4,
            2 => grave.len() >= 7,
            4 | 5 => own
                .iter()
                .any(|c| c.zone == Zone::Battlefield && c.card.is_artifact()),
            6 => own
                .iter()
                .any(|c| c.zone == Zone::Battlefield && subtype(c.card, "Dragon")),
            7 | 8 => grave_types.len() >= 4,
            _ => unreachable!(),
        };
        let growth = pumps
            .get(&(b.id, b.reference.generation))
            .filter(|(turn, _)| *turn == o.turn)
            .map_or(0, |(_, n)| *n);
        let wanted = (
            BASE[i].0 + if on { BONUS[i].0 } else { 0 } + growth,
            BASE[i].1 + if on { BONUS[i].1 } else { 0 },
        );
        assert_eq!((b.power, b.toughness), wanted, "{}", b.card.name());
        let intrinsic = if [0, 2].contains(&i) {
            Keywords::FLYING
        } else if i == 8 {
            Keywords::HASTE
        } else {
            Keywords::NONE
        };
        assert_eq!(
            b.keywords,
            if on {
                intrinsic.union(KEYWORDS[i])
            } else {
                intrinsic
            },
            "{}",
            b.card.name()
        );
        if on {
            boosted[i] += 1;
        }
        checked += 1;
    }
    checked
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
    let decks = [deck(0), deck(1)];
    let (mut decisions, mut replays, mut private, mut history_checks, mut max_bytes) =
        (0, 0, 0, 0, 0);
    let (mut casts, mut triggers, mut resolutions, mut activations) =
        ([0; 9], [0; 9], [0; 9], [0; 9]);
    let mut boosted = [0u64; 9];
    let mut checked_stats = 0u64;
    for seed in 0..games {
        let mut g = Game::new(Config {
            decks: [decks[seed % 2].clone(), decks[seed % 2].clone()],
            seed: seed as u64,
            starting_player: Player::P0,
            deck_rules: DeckRules::Standard,
        })?;
        let mut tracked = BTreeMap::new();
        let mut pump_roots = BTreeMap::new();
        let mut pumps = BTreeMap::new();

        for _ in 0..100_000 {
            if g.outcome().is_some() {
                break;
            }
            let p = g.actor().unwrap();
            let before = g.observe(p);
            let actions = g.legal_actions();
            let action = actions[choose(&g, &before, &actions)].clone();
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
                || before.stack.iter().any(|s| index(s.source_card).is_some());
            let saved = g.snapshot();
            let mut fork = boundary.then(|| g.clone());
            g.apply(p, action.clone())?;
            if let Some(f) = &mut fork {
                f.apply(p, action.clone())?;
            }
            let after = g.observe(p);
            for s in &after.stack {
                if before.stack.iter().any(|old| old.id == s.id)
                    || before.resolving.as_ref().is_some_and(|old| old.id == s.id)
                {
                    continue;
                }
                let Some(i) = index(s.source_card) else {
                    continue;
                };
                if s.kind == StackKind::Spell {
                    casts[i] += 1;
                } else if matches!(s.kind, StackKind::ActivatedAbility { .. }) {
                    activations[i] += 1;
                    if i == 0 {
                        pump_roots.insert(s.id, (s.object, after.turn));
                    }
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
            let finished: Vec<_> = pump_roots
                .iter()
                .filter(|(id, _)| !live(&after, **id))
                .map(|(&id, &v)| (id, v))
                .collect();
            for (id, (object, turn)) in finished {
                pump_roots.remove(&id);
                if after
                    .objects
                    .iter()
                    .any(|o| o.reference == object && o.zone == Zone::Battlefield)
                {
                    let entry = pumps
                        .entry((object.card, object.generation))
                        .or_insert((turn, 0));
                    if entry.0 != turn {
                        *entry = (turn, 0);
                    }
                    entry.1 += 1;
                }
            }
            let public = g.observe(p.other());
            checked_stats += check(&after, &mut boosted, &pumps);
            let mut discarded = [0; 9];
            check(&public, &mut discarded, &pumps);
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
            assert_eq!(after.schema, 51);

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
        assert!(boosted.iter().all(|n| *n > 0), "boosts {boosted:?}");
        for i in [1, 2, 7] {
            assert!(triggers[i] > 0 && resolutions[i] > 0);
        }
        assert!(activations[0] > 0 && private > 0);
    }
    println!(
        "Completed {games} Standard conditional-keyword games, {decisions} decisions, casts {casts:?}, triggers {triggers:?}, finished triggers {resolutions:?}, activations {activations:?}, enabled observations {boosted:?}, {checked_stats} independently checked public keyword/stat pairs, {history_checks} creature-history checks, {private} private choices, {replays} replay boundaries, {max_bytes} maximum initialized logical bytes; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
    Ok(())
}
