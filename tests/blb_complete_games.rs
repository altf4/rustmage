//! Exact BLB-only Standard decks, replay, and owned branches.
use rustmage::{
    Action, Card, Config, DeckRules, Error, ForageRoute, Game, GiftDeclaration, Observation,
    Outcome, Player, StackKind, Zone, policy,
};
use rustmage_core::cards::TriggerCondition;
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

#[allow(dead_code)]
#[path = "support/tmt_evidence.rs"]
mod tmt_evidence;
use tmt_evidence::{check_privacy, private_choice};

#[derive(Deserialize)]
struct Identity {
    name: String,
    oracle_id: String,
}

#[derive(Deserialize)]
struct FrozenSet {
    card_count: usize,
    cards: Vec<Identity>,
    unresolved: Vec<String>,
}

#[derive(Deserialize)]
struct DeckCard {
    #[serde(flatten)]
    identity: Identity,
    count: usize,
}

#[derive(Deserialize)]
struct Deck {
    id: String,
    name: String,
    cards: Vec<DeckCard>,
}

#[derive(Deserialize)]
struct Run {
    seed: u64,
    starting_player: usize,
}

#[derive(Deserialize)]
struct Pairing {
    p0: String,
    p1: String,
    runs: Vec<Run>,
}

#[derive(Deserialize)]
struct Workload {
    decks: Vec<Deck>,
    pairings: Vec<Pairing>,
}

fn identify(row: &Identity) -> Card {
    let card =
        Card::from_name(&row.name).unwrap_or_else(|| panic!("unknown BLB card {}", row.name));
    assert_eq!(
        card.info().oracle_id,
        Some(row.oracle_id.as_str()),
        "{}",
        row.name
    );
    card
}

fn pool() -> BTreeSet<Card> {
    let set: FrozenSet = serde_json::from_str(include_str!("../data/sets/blb.json")).unwrap();
    assert_eq!(set.card_count, 279);
    assert!(set.unresolved.is_empty());
    let pool = set.cards.iter().map(identify).collect::<BTreeSet<_>>();
    assert_eq!(pool.len(), set.card_count);
    pool
}

#[test]
fn bloomburrow_frozen_pool_supports_every_member_with_its_exact_legality() {
    let banned = ["Heartfire Hero", "Stormchaser's Talent"]
        .map(|name| Card::from_name(name).expect("a frozen banned identity"));
    let pool = pool();
    for &card in &pool {
        assert!(card.is_supported(), "{} remains gated", card.name());
        assert_eq!(
            card.is_standard_legal(),
            !banned.contains(&card),
            "{}",
            card.name()
        );
        if let Some(face) = card.back_face().or_else(|| card.adventure_spell()) {
            assert!(face.is_supported(), "{} has a gated face", card.name());
        }
    }
    let forest = Card::from_name("Forest").unwrap();
    for card in banned {
        assert!(pool.contains(&card));
        let mut config = Config {
            decks: [vec![forest; 60], vec![forest; 60]],
            seed: 7,
            starting_player: Player::P0,
            deck_rules: DeckRules::Standard,
        };
        config.decks[0][0] = card;
        assert!(matches!(
            Game::new(config.clone()),
            Err(Error::InvalidDeck(_))
        ));
        config.deck_rules = DeckRules::Freeform;
        assert!(Game::new(config).is_ok());
    }
}

fn deck_cards(deck: &Deck, pool: &BTreeSet<Card>) -> Vec<Card> {
    let mut seen = BTreeSet::new();
    let mut cards = Vec::new();
    for row in &deck.cards {
        let card = identify(&row.identity);
        assert!(pool.contains(&card) && card.is_standard_legal() && card.is_supported());
        assert!(seen.insert(card) && row.count > 0);
        assert!(row.count <= 4 || card.is_basic_land());
        cards.extend(std::iter::repeat_n(card, row.count));
    }
    assert_eq!(cards.len(), 60, "{}", deck.id);
    assert_eq!(cards.iter().filter(|card| card.is_land()).count(), 24);
    cards
}

fn prove_branch(game: &Game, view: &Observation, index: usize) -> bool {
    let Some(other) =
        view.actions.iter().enumerate().find_map(|(at, action)| {
            (at != index && !matches!(action, Action::Concede)).then_some(at)
        })
    else {
        return false;
    };
    let actor = view.viewer;
    let saved = game.snapshot();
    let mut raw = game.clone();
    let mut indexed = game.clone();
    raw.apply(actor, view.actions[index].clone()).unwrap();
    indexed.apply_index(actor, view.revision, index).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    let next = indexed.snapshot();
    assert_eq!(
        indexed.apply_index(actor, view.revision, index),
        Err(Error::StaleRevision)
    );
    assert_eq!(next, indexed.snapshot());
    indexed.restore(&saved).unwrap();
    assert_eq!(saved, indexed.snapshot());
    indexed
        .apply_index(actor, indexed.revision(), other)
        .unwrap();
    indexed.restore(&saved).unwrap();
    assert_eq!(saved, indexed.snapshot());
    assert_eq!(saved, game.snapshot());
    true
}

fn write_report(directory: Option<&Path>, name: &str, report: Value) {
    if let Some(directory) = directory {
        std::fs::write(
            directory.join(name),
            serde_json::to_vec_pretty(&report).unwrap(),
        )
        .unwrap();
    }
}

#[derive(Default)]
struct Coverage {
    colors: u8,
    casts: usize,
    names: BTreeSet<&'static str>,
    events: BTreeMap<&'static str, usize>,
}

impl Coverage {
    fn note(&mut self, event: &'static str) {
        *self.events.entry(event).or_default() += 1;
    }

    fn transition(&mut self, before: &Observation, action: &Action, after: &Observation) {
        if let Action::ChooseForage { object } = action
            && let Some(forage) = &before.forage
            && let Some(route) = forage.route
        {
            match route {
                ForageRoute::Food
                    if !after
                        .objects
                        .iter()
                        .any(|row| row.reference == *object && row.zone == Zone::Battlefield) =>
                {
                    self.note("Food foraged");
                }
                ForageRoute::ExileThree
                    if forage.selected.len() == 2
                        && forage.selected.iter().chain([object]).all(|selected| {
                            after.objects.iter().any(|row| {
                                row.id == selected.card
                                    && row.reference != *selected
                                    && row.zone == Zone::Exile
                            })
                        }) =>
                {
                    self.note("three cards foraged");
                }
                _ => (),
            }
        }
        for object in &after.objects {
            let prior = before
                .objects
                .iter()
                .find(|old| old.reference == object.reference);
            if object.zone == Zone::Battlefield && prior.is_none() && object.is_token() {
                if object.has_subtype("Food") {
                    self.note("Food created");
                }
                if object.has_subtype("Rabbit") {
                    self.note("Rabbit token created");
                }
            }
            if let Some(level) = object.class_level
                && level > 1
                && prior
                    .and_then(|old| old.class_level)
                    .is_some_and(|old| old < level)
            {
                self.note("Class level gained");
            }
        }
        for packet in &after.stack {
            if packet.is_copy || before.stack.iter().any(|old| old.id == packet.id) {
                continue;
            }
            let Some((_, card)) = packet.source_identity() else {
                continue;
            };
            if let StackKind::TriggeredAbility { ability } = packet.kind {
                let Some(trigger) = card.triggers().get(usize::from(ability)) else {
                    continue;
                };
                match trigger.condition.event() {
                    TriggerCondition::Expend(_) => self.note("expend triggered"),
                    TriggerCondition::FirstTargetedByController => self.note("valiant triggered"),
                    TriggerCondition::SelfOffspring => self.note("offspring triggered"),
                    TriggerCondition::Foraged => self.note("forage triggered"),
                    _ => (),
                }
                continue;
            }
            if packet.kind != StackKind::Spell {
                continue;
            }
            let Some(cast) = packet.cast else { continue };
            self.colors |= card.colors();
            self.casts += 1;
            self.names.insert(card.name());
            if matches!(cast.gift, Some(GiftDeclaration::Promised { .. })) {
                self.note("Gift promised");
            }
            if cast.offspring == Some(true) {
                self.note("offspring paid");
            }
            if card.name().starts_with("Season of ") {
                self.note("Season cast");
            }
        }
    }
}

#[test]
fn bloomburrow_standard_games_finish_and_replay_with_real_mechanic_events() {
    let workload: Workload =
        serde_json::from_str(include_str!("../data/decks/blb-set-only.json")).unwrap();
    let pool = pool();
    let decks: BTreeMap<_, _> = workload
        .decks
        .iter()
        .map(|deck| {
            (
                deck.id.as_str(),
                (deck.name.as_str(), deck_cards(deck, &pool)),
            )
        })
        .collect();
    assert_eq!(decks.len(), 10);
    assert_eq!(workload.decks.len(), decks.len());
    assert_eq!(workload.pairings.len(), 10);
    for deck in decks.keys() {
        assert_eq!(
            workload
                .pairings
                .iter()
                .filter(|pair| pair.p0 == *deck)
                .count(),
            1
        );
        assert_eq!(
            workload
                .pairings
                .iter()
                .filter(|pair| pair.p1 == *deck)
                .count(),
            1
        );
    }
    let directory =
        std::env::var_os("RUSTMAGE_TDM_EVIDENCE_DIR").map(|dir| PathBuf::from(dir).join("blb"));
    if let Some(directory) = &directory {
        std::fs::create_dir_all(directory).unwrap();
    }
    let mut coverage = Coverage::default();
    let mut wins = [0; 2];
    let mut draws = 0;
    let mut games = 0;
    let mut actions = 0;
    let mut public_branches = 0;
    let mut private_branches = 0;
    let mut gameplay = Duration::ZERO;
    let mut replay = Duration::ZERO;

    for (pair_index, pair) in workload.pairings.iter().enumerate() {
        assert_ne!(pair.p0, pair.p1);
        assert_eq!(pair.runs.len(), 2);
        assert_ne!(pair.runs[0].seed, pair.runs[1].seed);
        assert_eq!(pair.runs[0].starting_player, 0);
        assert_eq!(pair.runs[1].starting_player, 1);
        let (left_name, left) = &decks[pair.p0.as_str()];
        let (right_name, right) = &decks[pair.p1.as_str()];
        for run in &pair.runs {
            let label = format!("{pair_index}-{}-{}", run.seed, run.starting_player);
            let config = Config {
                decks: [left.clone(), right.clone()],
                seed: run.seed,
                starting_player: [Player::P0, Player::P1][run.starting_player],
                deck_rules: DeckRules::Standard,
            };
            let mut game = Game::new(config.clone()).unwrap();
            let mut journal = Vec::new();
            let mut branches = [false; 2];
            let started = Instant::now();
            while game.outcome().is_none() && journal.len() < 4096 {
                let actor = game.actor().unwrap();
                let view = game.observe(actor);
                check_privacy(&view, actor);
                let index = policy::smoke_index_from_observation(&view, &view.actions);
                let action = view.actions[index].clone();
                assert_ne!(action, Action::Concede);
                let private = private_choice(&view)
                    && view
                        .library_selection
                        .as_ref()
                        .is_none_or(|selection| !selection.public);
                if private {
                    check_privacy(&game.observe(actor.other()), actor);
                }
                let kind = usize::from(private);
                if !branches[kind]
                    && !matches!(action, Action::Pass | Action::Keep)
                    && prove_branch(&game, &view, index)
                {
                    branches[kind] = true;
                }
                if let Err(error) = game.apply(actor, action.clone()) {
                    write_report(
                        directory.as_deref(),
                        &format!("{label}-failure.json"),
                        json!({
                            "config": config, "actions": journal, "actor": actor,
                            "failing_action": action, "error": error.to_string(),
                        }),
                    );
                    panic!("{label}, decision {} {action:?}: {error}", journal.len());
                }
                coverage.transition(&view, &action, &game.observe(actor));
                journal.push((actor, action));
            }
            gameplay += started.elapsed();
            write_report(
                directory.as_deref(),
                &format!("{label}.json"),
                json!({
                    "catalogue": rustmage::CATALOGUE_VERSION, "config": config,
                    "decks": [left_name, right_name], "actions": journal, "outcome": game.outcome(),
                    "public_branch": branches[0], "private_branch": branches[1],
                }),
            );
            match game
                .outcome()
                .unwrap_or_else(|| panic!("{label} exceeded the decision limit"))
            {
                Outcome::Winner(player) => wins[player.index()] += 1,
                Outcome::Draw => draws += 1,
            }
            actions += journal.len();
            games += 1;
            public_branches += usize::from(branches[0]);
            private_branches += usize::from(branches[1]);

            let started = Instant::now();
            let mut rerun = Game::new(config).unwrap();
            for (actor, action) in journal {
                rerun.apply(actor, action).unwrap();
            }
            assert_eq!(rerun.snapshot(), game.snapshot(), "{label}");
            replay += started.elapsed();
        }
    }

    assert_eq!(games, 20);
    assert_eq!(coverage.colors, 31, "spells cast: {:?}", coverage.names);
    assert!(public_branches >= 5 && private_branches > 0);
    assert!(
        coverage.events.contains_key("Food foraged")
            || coverage.events.contains_key("three cards foraged")
            || coverage.events.contains_key("forage triggered"),
        "{:?}",
        coverage.events
    );
    for required in [
        "expend triggered",
        "valiant triggered",
        "Gift promised",
        "offspring paid",
        "offspring triggered",
        "Food created",
        "Rabbit token created",
        "Season cast",
    ] {
        assert!(
            coverage.events.contains_key(required),
            "missing {required} in {:?}",
            coverage.events
        );
    }
    let report = json!({
        "games": games, "decisions": actions, "wins": wins, "draws": draws,
        "colors": coverage.colors, "actual_casts": coverage.casts, "cast_names": coverage.names,
        "events": coverage.events, "public_branches": public_branches,
        "private_branches": private_branches, "gameplay_seconds": gameplay.as_secs_f64(),
        "replay_seconds": replay.as_secs_f64(), "decisions_per_second": actions as f64 / gameplay.as_secs_f64(),
    });
    println!("BLB-only Standard: {report}");
    write_report(directory.as_deref(), "summary.json", report);
}
