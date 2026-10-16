//! Full native Standard games with independent conditional keyword/stat checks and replay.
use rustmage::{
    Action, Card, Config, DeckRules, Error, Game, Keywords, Observation, Player, StackKind, Zone,
};
use std::collections::BTreeMap;
const NAMES: [&str; 18] = [
    "Bigfin Bouncer",
    "Bounce Off",
    "Constrictor Sage",
    "Exclusion Mage",
    "Exosuit Savior",
    "Freeze in Place",
    "Grappling Kraken",
    "Iceridge Serpent",
    "Mental Modulation",
    "Mischievous Pup",
    "Peerless Ropemaster",
    "Proctor's Gaze",
    "Rimekin Recluse",
    "Ring of the Lucii",
    "Rowdy Snowballers",
    "Spider-Byte, Web Warden",
    "Stickytongue Sentinel",
    "Tidecaller Mentor",
];
const BASE: [(i128, i128); 18] = [
    (3, 2),
    (0, 0),
    (4, 4),
    (2, 2),
    (2, 2),
    (0, 0),
    (5, 6),
    (3, 3),
    (0, 0),
    (3, 1),
    (4, 4),
    (0, 0),
    (3, 2),
    (0, 0),
    (2, 2),
    (2, 2),
    (3, 3),
    (3, 3),
];
fn index(c: Card) -> Option<usize> {
    NAMES.iter().position(|n| *n == c.name())
}
fn deck(f: usize) -> Vec<Card> {
    let mut d = Vec::new();
    let spells = match f {
        0 => vec![
            NAMES[0], NAMES[1], NAMES[2], NAMES[3], NAMES[4], NAMES[5], NAMES[6], "Bear Cub", "Opt",
        ],
        1 => vec![
            NAMES[7], NAMES[8], NAMES[9], NAMES[10], NAMES[11], NAMES[12], NAMES[13], NAMES[14],
            "Bear Cub",
        ],
        _ => vec![
            NAMES[15],
            NAMES[16],
            NAMES[17],
            NAMES[2],
            NAMES[5],
            NAMES[11],
            NAMES[6],
            "Bear Cub",
            "Mind Drill Assailant",
        ],
    };
    for n in [
        "Tranquil Cove",
        "Thornwood Falls",
        "Dismal Backwater",
        "Blossoming Sands",
        "Island",
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
            Action::AnnounceActivate { card, ability, .. } => {
                let b = o.objects.iter().find(|b| b.id == *card).unwrap();
                if o.stack.is_empty()
                    && o.active == o.actor.unwrap()
                    && matches!(o.step, rustmage::Step::Main1 | rustmage::Step::Main2)
                    && ((b.card.name() == NAMES[2] && b.zone == Zone::Graveyard)
                        || (b.card.name() == NAMES[13]
                            && *ability == 1
                            && o.objects.iter().any(|v| {
                                v.zone == Zone::Battlefield && v.owner != b.owner && !v.tapped
                            }))
                        || b.card.name() == "Mind Drill Assailant")
                {
                    240
                } else {
                    -100
                }
            }
            Action::ChooseTarget {
                target: rustmage::Target::Card(card),
            } => {
                let v = o.objects.iter().find(|v| v.id == *card).unwrap();
                if v.owner != o.actor.unwrap() {
                    if !v.tapped { 290 } else { 280 }
                } else if v.card.is_creature() {
                    270
                } else {
                    260
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
fn check(o: &Observation) -> u64 {
    let mut checked = 0;
    for v in &o.objects {
        let Some(i) = index(v.card) else { continue };
        if v.zone != Zone::Battlefield || !v.card.is_creature() {
            continue;
        }
        let counters = i128::from(v.counters.plus_one) - i128::from(v.counters.minus_one);
        assert_eq!(
            (v.power, v.toughness),
            (BASE[i].0 + counters, BASE[i].1 + counters)
        );
        let expected = if i == 4 {
            Keywords::FLYING
        } else if i == 9 {
            Keywords::FLASH
        } else if i == 16 {
            Keywords::REACH
        } else if i == 17 {
            Keywords::MENACE
        } else {
            Keywords::NONE
        };
        assert_eq!(v.keywords, expected);
        checked += 1;
    }
    checked
}
#[derive(Clone)]
struct Expected {
    reference: rustmage::ObjectRef,
    zone: Zone,
    stun: u64,
    tapped: bool,
}
fn expected(o: &Observation, s: &rustmage::SpellView, i: usize) -> Vec<Expected> {
    if s.kind == StackKind::Spell && ![1, 5, 8, 11].contains(&i) {
        return vec![];
    }
    if i == 17
        && o.objects
            .iter()
            .filter(|v| v.owner == s.controller && v.zone == Zone::Graveyard && v.token.is_none())
            .count()
            < 7
    {
        return vec![];
    }
    s.targets
        .iter()
        .filter_map(|bound| {
            let reference = bound.object?;
            let v = o
                .objects
                .iter()
                .find(|v| v.reference == reference && v.zone == Zone::Battlefield)?;
            if i == 10 && !v.tapped {
                return None;
            }
            let bounce = [0, 1, 3, 4, 7, 9, 10, 11, 12, 15, 16, 17].contains(&i);
            Some(Expected {
                reference,
                zone: if bounce {
                    Zone::Hand
                } else {
                    Zone::Battlefield
                },
                stun: v.counters.stun
                    + if [2, 6, 14].contains(&i) {
                        1
                    } else if i == 5 {
                        3
                    } else {
                        0
                    },
                tapped: !bounce,
            })
        })
        .collect()
}
fn verify(o: &Observation, checks: &[Expected]) -> u64 {
    for e in checks {
        let v = o.objects.iter().find(|v| v.id == e.reference.card).unwrap();
        assert_eq!(v.zone, e.zone);
        if e.zone == Zone::Hand {
            assert_eq!(v.reference.generation, e.reference.generation + 1);
            assert!(o.known_hand.contains(&v.reference));
        } else {
            assert_eq!(v.reference, e.reference);
            assert_eq!(v.counters.stun, e.stun);
            assert_eq!(v.tapped, e.tapped);
        }
    }
    checks.len() as u64
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
        ([0; 18], [0; 18], [0; 18], [0; 18]);
    let mut effect_checks = 0;
    let mut checked_stats = 0u64;
    for seed in 0..games {
        let mut g = Game::new(Config {
            decks: [decks[seed % 3].clone(), decks[seed % 3].clone()],
            seed: seed as u64,
            starting_player: Player::P0,
            deck_rules: DeckRules::Standard,
        })?;
        let mut tracked = BTreeMap::new();
        let mut expectations = BTreeMap::<u64, Vec<Expected>>::new();

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
            if let Some(root) = before.stack.last()
                && let Some(i) = index(root.source_card)
                && !after.stack.iter().any(|v| v.id == root.id)
                && matches!(action, Action::Pass)
            {
                expectations.insert(root.id, expected(&before, root, i));
            }
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
            let finished: Vec<_> = expectations
                .keys()
                .copied()
                .filter(|id| !live(&after, *id))
                .collect();
            for id in finished {
                effect_checks += verify(&after, &expectations.remove(&id).unwrap());
            }
            let public = g.observe(p.other());
            checked_stats += check(&after);
            check(&public);
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
        assert!(effect_checks > 0);
        for i in [0, 2, 3, 4, 6, 7, 9, 10, 12, 14, 15, 16, 17] {
            assert!(triggers[i] > 0 && resolutions[i] > 0);
        }
        assert!(activations[2] > 0 && activations[13] > 0 && private > 0);
    }
    println!(
        "Completed {games} Standard targeted-bounce-tap games, {decisions} decisions, casts {casts:?}, triggers {triggers:?}, finished triggers {resolutions:?}, activations {activations:?}, {effect_checks} independently verified resolved target effects, {checked_stats} independently checked public keyword/stat pairs, {history_checks} creature-history checks, {private} private choices, {replays} replay boundaries, {max_bytes} maximum initialized logical bytes; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
    Ok(())
}
