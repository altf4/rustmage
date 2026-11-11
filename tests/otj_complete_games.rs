//! Exact OTJ-only Standard decks, replay, and owned branches.
use rustmage::{
    Action, Card, CastMethod, Config, DeckRules, Error, Game, ObjectView, Observation, Outcome,
    Player, StackKind, Step, Zone, policy,
};
use rustmage_core::cards::Instruction;
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
        Card::from_name(&row.name).unwrap_or_else(|| panic!("unknown OTJ card {}", row.name));
    assert_eq!(
        card.info().oracle_id,
        Some(row.oracle_id.as_str()),
        "{}",
        row.name
    );
    card
}

fn pool() -> BTreeSet<Card> {
    let set: FrozenSet = serde_json::from_str(include_str!("../data/sets/otj.json")).unwrap();
    assert_eq!(set.card_count, 276);
    assert!(set.unresolved.is_empty());
    let pool = set.cards.iter().map(identify).collect::<BTreeSet<_>>();
    assert_eq!(pool.len(), set.card_count);
    pool
}

#[test]
fn outlaws_frozen_pool_supports_every_member_with_its_exact_legality() {
    let banned = Card::from_name("This Town Ain't Big Enough").unwrap();
    for card in pool() {
        assert!(card.is_supported(), "{} remains gated", card.name());
        assert_eq!(card.is_standard_legal(), card != banned, "{}", card.name());
        if let Some(face) = card.back_face().or_else(|| card.adventure_spell()) {
            assert!(face.is_supported(), "{} has a gated face", card.name());
        }
    }
    let forest = Card::from_name("Forest").unwrap();
    let mut config = Config {
        decks: [vec![forest; 60], vec![forest; 60]],
        seed: 7,
        starting_player: Player::P0,
        deck_rules: DeckRules::Standard,
    };
    config.decks[0][0] = banned;
    assert!(matches!(
        Game::new(config.clone()),
        Err(Error::InvalidDeck(_))
    ));
    config.deck_rules = DeckRules::Freeform;
    assert!(Game::new(config).is_ok());
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

fn saddle_activation<'a>(view: &'a Observation, action: &Action) -> Option<&'a ObjectView> {
    let (card, ability) = match *action {
        Action::AnnounceActivate { card, ability, .. }
        | Action::BeginActivate { card, ability, .. }
        | Action::Activate { card, ability, .. } => (card, ability),
        _ => return None,
    };
    view.objects
        .iter()
        .find(|object| object.id == card)
        .filter(|object| {
            object
                .card
                .activations()
                .get(usize::from(ability))
                .is_some_and(|ability| {
                    ability
                        .program
                        .iter()
                        .any(|step| matches!(step, Instruction::BecomeSaddled { .. }))
                })
        })
}

fn choose(view: &Observation, plot_paid: bool, saddle_turn: Option<u32>) -> usize {
    let preferred = view
        .actions
        .iter()
        .position(|action| {
            matches!(
                action,
                Action::AnnouncePlotted { .. } | Action::AnnouncePlottedFace { .. }
            )
        })
        .or_else(|| {
            (!plot_paid)
                .then(|| {
                    view.actions
                        .iter()
                        .position(|action| matches!(action, Action::AnnouncePlot { .. }))
                })
                .flatten()
        })
        .or_else(|| {
            (view.step == Step::Main1 && saddle_turn != Some(view.turn))
                .then(|| {
                    view.actions.iter().position(|action| {
                        saddle_activation(view, action)
                            .is_some_and(|mount| !mount.saddled && !mount.tapped)
                    })
                })
                .flatten()
        })
        .or_else(|| {
            view.actions
                .iter()
                .rposition(|action| matches!(action, Action::ChooseMode { .. }))
        });
    if let Some(index) = preferred {
        return index;
    }
    let index = policy::smoke_index_from_observation(view, &view.actions);
    if saddle_activation(view, &view.actions[index]).is_none() {
        return index;
    }
    // Repeated Saddle can tap all available attackers before combat.
    let (indices, actions): (Vec<_>, Vec<_>) = view
        .actions
        .iter()
        .enumerate()
        .filter(|(_, action)| saddle_activation(view, action).is_none())
        .map(|(index, action)| (index, action.clone()))
        .unzip();
    indices[policy::smoke_index_from_observation(view, &actions)]
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

    fn transition(&mut self, before: &Observation, after: &Observation) {
        for index in 0..2 {
            if !before.players[index].committed_crime_this_turn
                && after.players[index].committed_crime_this_turn
            {
                self.note("crime");
            }
        }
        for plotted in &after.plotted_cards {
            if !before.plotted_cards.contains(plotted) {
                assert!(after.objects.iter().any(|object| {
                    object.reference == plotted.object && object.zone == Zone::Exile
                }));
                self.note("plot designation");
            }
        }
        for object in &after.objects {
            let prior = before
                .objects
                .iter()
                .find(|old| old.reference == object.reference);
            if object.saddled && !prior.is_some_and(|old| old.saddled) {
                self.note("saddle resolved");
            }
            if object.zone == Zone::Battlefield && prior.is_none() {
                if object.has_subtype("Desert") {
                    self.note("Desert entered");
                }
                if object.is_token() && object.has_subtype("Mercenary") {
                    self.note("Mercenary created");
                }
            }
        }
        for packet in &after.stack {
            if packet.kind != StackKind::Spell
                || packet.is_copy
                || before.stack.iter().any(|old| old.id == packet.id)
            {
                continue;
            }
            let Some((_, card)) = packet.source_identity() else {
                continue;
            };
            let Some(cast) = packet.cast else { continue };
            self.colors |= card.colors();
            self.casts += 1;
            self.names.insert(card.name());
            if cast.method == CastMethod::Plot {
                assert_eq!(cast.origin, Zone::Exile);
                self.note("plotted spell cast");
            }
            if card.info().oracle_keywords.contains(&"Spree") {
                assert!(packet.mode.is_some());
                self.note("Spree cast");
            }
        }
    }
}

#[test]
fn outlaws_standard_games_finish_and_replay_with_real_mechanic_events() {
    let workload: Workload =
        serde_json::from_str(include_str!("../data/decks/otj-set-only.json")).unwrap();
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
    assert_eq!(decks.len(), 5);
    let directory =
        std::env::var_os("RUSTMAGE_TDM_EVIDENCE_DIR").map(|dir| PathBuf::from(dir).join("otj"));
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
            let mut plot_paid = false;
            let mut saddle_turn = None;
            let mut branches = [false; 2];
            let started = Instant::now();
            while game.outcome().is_none() && journal.len() < 4096 {
                let actor = game.actor().unwrap();
                let view = game.observe(actor);
                check_privacy(&view, actor);
                let index = choose(&view, plot_paid, saddle_turn);
                let action = view.actions[index].clone();
                assert_ne!(action, Action::Concede);
                let private = private_choice(&view);
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
                plot_paid |= matches!(action, Action::PayPlot { .. });
                if action == Action::FinishSaddle {
                    saddle_turn = Some(view.turn);
                }
                coverage.transition(&view, &game.observe(actor));
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

    assert_eq!(games, 15);
    assert_eq!(coverage.colors, 31, "spells cast: {:?}", coverage.names);
    assert!(public_branches >= 5 && private_branches > 0);
    for required in [
        "crime",
        "plot designation",
        "plotted spell cast",
        "saddle resolved",
        "Spree cast",
        "Desert entered",
        "Mercenary created",
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
    println!("OTJ-only Standard: {report}");
    write_report(directory.as_deref(), "summary.json", report);
}
