//! Full native Standard games with independent conditional keyword/stat checks and replay.
use rustmage::{
    Action, Card, Config, DeckRules, Error, Game, Keywords, Observation, Player, StackKind, Zone,
};
use std::collections::BTreeMap;
const NAMES: [&str; 13] = [
    "Akawalli, the Seething Tower",
    "Cephalid Inkmage",
    "Druid of the Spade",
    "First-Time Flyer",
    "Foggy Swamp Hunters",
    "Ghitu Lavarunner",
    "June, Bounty Hunter",
    "Kitesail Corsair",
    "Messenger Hawk",
    "Nightwhorl Hermit",
    "Skyward Spider",
    "Stoic Sphinx",
    "Twinblade Paladin",
];
const BASE: [(i128, i128); 13] = [
    (3, 3),
    (2, 2),
    (2, 3),
    (1, 2),
    (3, 4),
    (1, 2),
    (2, 2),
    (2, 1),
    (1, 2),
    (1, 4),
    (2, 2),
    (5, 3),
    (3, 3),
];
const BONUS: [(i128, i128); 13] = [
    (2, 2),
    (0, 0),
    (2, 0),
    (1, 1),
    (0, 0),
    (1, 0),
    (0, 0),
    (0, 0),
    (2, 0),
    (1, 0),
    (0, 0),
    (0, 0),
    (0, 0),
];
const KEYWORDS: [Keywords; 13] = [
    Keywords::TRAMPLE,
    Keywords::CANT_BE_BLOCKED,
    Keywords::TRAMPLE,
    Keywords::NONE,
    Keywords::LIFELINK.union(Keywords::MENACE),
    Keywords::HASTE,
    Keywords::CANT_BE_BLOCKED,
    Keywords::FLYING,
    Keywords::NONE,
    Keywords::CANT_BE_BLOCKED,
    Keywords::FLYING,
    Keywords::HEXPROOF,
    Keywords::DOUBLE_STRIKE,
];
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
            NAMES[5],
            NAMES[9],
            NAMES[8],
            "Opt",
            "Abandon Attachments",
        ],
        1 => vec![
            NAMES[4],
            NAMES[6],
            NAMES[7],
            NAMES[8],
            NAMES[10],
            NAMES[11],
            NAMES[12],
            "Goldvein Pick",
            "Opt",
        ],
        _ => vec![
            NAMES[0],
            NAMES[1],
            NAMES[3],
            NAMES[5],
            NAMES[9],
            NAMES[10],
            NAMES[12],
            "Mind Drill Assailant",
            "Abandon Attachments",
        ],
    };
    for n in [
        "Tranquil Cove",
        "Blossoming Sands",
        "Scoured Barrens",
        "Dismal Backwater",
        "Bloodfell Caves",
        "Rugged Highlands",
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
                let b = o.objects.iter().find(|b| b.id == *card).unwrap();
                if o.stack.is_empty()
                    && o.active == o.actor.unwrap()
                    && matches!(o.step, rustmage::Step::Main1 | rustmage::Step::Main2)
                    && ((b.card.name() == "Goldvein Pick" && b.attached_to.is_none())
                        || b.card.name() == NAMES[6]
                        || b.card.name() == "Mind Drill Assailant"
                        || b.card.token_info().is_some_and(|t| t.subtypes == "Clue"))
                {
                    240
                } else {
                    -100
                }
            }
            Action::ChooseTarget {
                target: rustmage::Target::Card(card),
            } => {
                if o.objects
                    .iter()
                    .any(|b| b.id == *card && b.card.name() == NAMES[10])
                {
                    270
                } else {
                    100
                }
            }
            Action::Activate { .. }
            | Action::BeginActivate { .. }
            | Action::AnnounceMana { .. } => -100,
            Action::TapMana { .. } | Action::ActivateMana { .. } => {
                if matches!(o.decision, "pay_costs" | "resolution_mana") {
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
                if *destination == rustmage::LibraryDestination::Graveyard {
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
fn literal_subtype(c: Card, t: &str) -> bool {
    c.info()
        .type_line
        .split(" — ")
        .nth(1)
        .is_some_and(|s| s.split_whitespace().any(|v| v == t))
}
fn check(o: &Observation, boosted: &mut [u64; 13]) -> u64 {
    let mut checked = 0;
    for b in &o.objects {
        let Some(i) = index(b.card) else { continue };
        if b.zone != Zone::Battlefield {
            continue;
        }
        let own: Vec<_> = o.objects.iter().filter(|v| v.owner == b.owner).collect();
        let grave: Vec<_> = own
            .iter()
            .filter(|v| v.zone == Zone::Graveyard && v.token.is_none())
            .collect();
        let permanent = grave
            .iter()
            .filter(|v| {
                types(v.card).any(|t| {
                    [
                        "Creature",
                        "Artifact",
                        "Enchantment",
                        "Land",
                        "Battle",
                        "Planeswalker",
                    ]
                    .contains(&t)
                })
            })
            .count();
        let player = &o.players[if b.owner == Player::P0 { 0 } else { 1 }];
        let modified = b.counters.plus_one > 0
            || b.counters.minus_one > 0
            || b.counters.stun > 0
            || b.counters.loyalty > 0
            || o.objects.iter().any(|v| {
                v.zone == Zone::Battlefield
                    && v.attached_to == Some(b.reference)
                    && (literal_subtype(v.card, "Equipment")
                        || (literal_subtype(v.card, "Aura") && v.owner == b.owner))
            });
        assert_eq!(b.modified, modified);
        let on = match i {
            0 => permanent >= 4,
            1 | 9 => grave.len() >= 7,
            2 => own
                .iter()
                .any(|v| v.zone == Zone::Battlefield && v.token.is_some()),
            3 => grave.iter().any(|v| literal_subtype(v.card, "Lesson")),
            4 | 6 | 8 => player.cards_drawn_this_turn >= 2,
            5 => {
                grave
                    .iter()
                    .filter(|v| types(v.card).any(|t| ["Instant", "Sorcery"].contains(&t)))
                    .count()
                    >= 2
            }
            7 => b.attacking && o.decision != "attackers",
            10 => modified,
            11 => player.spells_cast_this_turn == 0,
            12 => player.life >= 25,
            _ => unreachable!(),
        };
        let extra = if i == 0 && permanent >= 8 { 2 } else { 0 };
        let equipment = o
            .objects
            .iter()
            .filter(|v| {
                v.zone == Zone::Battlefield
                    && v.attached_to == Some(b.reference)
                    && v.card.name() == "Goldvein Pick"
            })
            .count() as i128;
        let counter = i128::from(b.counters.plus_one) - i128::from(b.counters.minus_one);
        assert_eq!(
            (b.power, b.toughness),
            (
                BASE[i].0 + if on { BONUS[i].0 } else { 0 } + extra + counter + equipment,
                BASE[i].1 + if on { BONUS[i].1 } else { 0 } + extra + counter + equipment
            ),
            "{}",
            b.card.name()
        );
        let intrinsic = if [3, 8].contains(&i) {
            Keywords::FLYING
        } else if i == 11 {
            Keywords::FLYING.union(Keywords::FLASH)
        } else if i == 9 {
            Keywords::VIGILANCE
        } else {
            Keywords::NONE
        };
        let expected = if on {
            intrinsic.union(KEYWORDS[i])
        } else {
            intrinsic
        };
        let expected = if extra > 0 {
            expected.union(Keywords::MAX_ONE_BLOCKER)
        } else {
            expected
        };
        assert_eq!(b.keywords, expected, "{}", b.card.name());
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
    let decks = [deck(0), deck(1), deck(2)];
    let (mut decisions, mut replays, mut private, mut history_checks, mut max_bytes) =
        (0, 0, 0, 0, 0);
    let (mut casts, mut triggers, mut resolutions, mut activations) =
        ([0; 13], [0; 13], [0; 13], [0; 13]);
    let mut boosted = [0u64; 13];
    let mut checked_stats = 0u64;
    for seed in 0..games {
        let mut g = Game::new(Config {
            decks: [decks[seed % 3].clone(), decks[seed % 3].clone()],
            seed: seed as u64,
            starting_player: Player::P0,
            deck_rules: DeckRules::Standard,
        })?;
        let mut tracked = BTreeMap::new();

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
            let public = g.observe(p.other());
            checked_stats += check(&after, &mut boosted);
            let mut discarded = [0; 13];
            check(&public, &mut discarded);
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
        assert!(boosted.iter().all(|n| *n > 0), "boosts {boosted:?}");
        for i in [1, 8, 12] {
            assert!(triggers[i] > 0 && resolutions[i] > 0);
        }
        assert!(activations[6] > 0 && private > 0);
    }
    println!(
        "Completed {games} Standard source-predicate games, {decisions} decisions, casts {casts:?}, triggers {triggers:?}, finished triggers {resolutions:?}, activations {activations:?}, enabled observations {boosted:?}, {checked_stats} independently checked public keyword/stat pairs, {history_checks} creature-history checks, {private} private choices, {replays} replay boundaries, {max_bytes} maximum initialized logical bytes; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
    Ok(())
}
