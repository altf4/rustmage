//! Native Standard games with independently checked live threshold stats and replay.
use rustmage::{
    Action, Card, Config, DeckRules, Error, Game, Observation, Player, StackKind, Zone,
};
use std::collections::BTreeMap;
const NAMES: [&str; 9] = [
    "Basking Capybara",
    "Billowing Shriekmass",
    "Brightspear Zealot",
    "Cloudsculpt Technician",
    "Frilled Cave-Wurm",
    "Gaelicat",
    "Gigantoad",
    "Mind Drill Assailant",
    "Scorpion Sentinel",
];
const BASE: [(i128, i128); 9] = [
    (1, 3),
    (2, 3),
    (2, 4),
    (1, 4),
    (2, 5),
    (1, 3),
    (4, 4),
    (2, 5),
    (1, 4),
];
const BONUS: [(i128, i128); 9] = [
    (3, 0),
    (2, 1),
    (2, 0),
    (1, 0),
    (2, 0),
    (2, 0),
    (2, 2),
    (3, 0),
    (3, 0),
];
fn index(c: Card) -> Option<usize> {
    NAMES.iter().position(|n| *n == c.name())
}
fn deck() -> Vec<Card> {
    let mut d = Vec::new();
    for n in [
        "Tranquil Cove",
        "Dismal Backwater",
        "Thornwood Falls",
        "Jungle Hollow",
        "Blossoming Sands",
        "Forest",
    ]
    .into_iter()
    .chain(NAMES)
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
                    && o.objects
                        .iter()
                        .any(|o| o.id == *card && o.card.name() == NAMES[7])
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
fn check_stats(o: &Observation, boosted: &mut [u64; 9]) -> u64 {
    let mut checked = 0;
    for b in &o.objects {
        let Some(i) = index(b.card) else {
            continue;
        };
        if b.zone != Zone::Battlefield {
            continue;
        }
        let owned: Vec<_> = o.objects.iter().filter(|c| c.owner == b.owner).collect();
        let grave: Vec<_> = owned
            .iter()
            .filter(|c| c.zone == Zone::Graveyard && !c.card.is_token())
            .collect();
        let permanents = grave
            .iter()
            .filter(|c| {
                c.card
                    .info()
                    .type_line
                    .split(" — ")
                    .next()
                    .unwrap()
                    .split_whitespace()
                    .any(|t| {
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
        let artifacts = owned
            .iter()
            .filter(|c| c.zone == Zone::Battlefield && c.card.is_artifact())
            .count();
        let lands = owned
            .iter()
            .filter(|c| c.zone == Zone::Battlefield && c.card.is_land())
            .count();
        let on = match i {
            0 | 4 => permanents >= 4,
            1 | 7 => grave.len() >= 7,
            2 => o.players[b.owner.index()].spells_cast_this_turn >= 2,
            3 => artifacts >= 1,
            5 => artifacts >= 2,
            6 | 8 => lands >= 7,
            _ => unreachable!(),
        };
        let wanted = (
            BASE[i].0 + if on { BONUS[i].0 } else { 0 },
            BASE[i].1 + if on { BONUS[i].1 } else { 0 },
        );
        assert_eq!((b.power, b.toughness), wanted, "{}", b.card.name());
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
    let deck = deck();
    let (mut decisions, mut replays, mut private, mut history_checks, mut max_bytes) =
        (0, 0, 0, 0, 0);
    let (mut casts, mut triggers, mut resolutions, mut activations) =
        ([0; 9], [0; 9], [0; 9], [0; 9]);
    let mut boosted = [0u64; 9];
    let mut checked_stats = 0u64;
    for seed in 0..games {
        let mut g = Game::new(Config {
            decks: [deck.clone(), deck.clone()],
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
            if before.library_choice.is_some() || before.search_choice.is_some() {
                let other = g.observe(p.other());
                assert!(
                    other.library_choice.is_none()
                        && other.search_choice.is_none()
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
            checked_stats += check_stats(&after, &mut boosted);
            let mut discarded = [0; 9];
            check_stats(&public, &mut discarded);
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
        assert!(triggers[1] > 0 && resolutions[1] > 0 && activations[7] > 0 && private > 0);
    }
    println!(
        "Completed {games} Standard count-threshold games, {decisions} decisions, casts {casts:?}, triggers {triggers:?}, finished triggers {resolutions:?}, activations {activations:?}, boosted observations {boosted:?}, {checked_stats} independently checked public stats, {history_checks} creature-history checks, {private} private choices, {replays} replay boundaries, {max_bytes} maximum initialized logical bytes; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
    Ok(())
}
