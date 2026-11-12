//! Exact Duskmourn-only Standard games, observable rules events, and owned replay.
use rustmage::{
    Action, Card, CastFace, Config, CounterKind, DeckRules, Error, Game, Observation, Outcome,
    Player, RoomDoor, StackKind, Zone, policy,
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
    let card = Card::from_name(&row.name)
        .unwrap_or_else(|| panic!("unknown Duskmourn identity {}", row.name));
    assert_eq!(
        card.info().oracle_id,
        Some(row.oracle_id.as_str()),
        "{}",
        row.name
    );
    card
}

fn pool() -> BTreeSet<Card> {
    let set: FrozenSet = serde_json::from_str(include_str!("../data/sets/dsk.json")).unwrap();
    assert_eq!(set.card_count, 276);
    assert!(set.unresolved.is_empty());
    let pool = set.cards.iter().map(identify).collect::<BTreeSet<_>>();
    assert_eq!(pool.len(), set.card_count);
    pool
}

fn config(decks: [Vec<Card>; 2], seed: u64, starting_player: Player) -> Config {
    Config {
        decks,
        seed,
        starting_player,
        deck_rules: DeckRules::Standard,
    }
}

#[test]
fn duskmourn_frozen_physical_members_and_non_deck_room_faces_have_exact_admission() {
    let pool = pool();
    let forest = Card::from_name("Forest").unwrap();
    let banned = Card::from_name("Screaming Nemesis").unwrap();
    assert!(pool.contains(&banned));
    let mut room_faces = BTreeSet::new();
    for card in pool {
        assert!(card.is_supported(), "{} is gated", card.name());
        assert_eq!(card.is_standard_legal(), card != banned, "{}", card.name());
        let mut member = config([vec![forest; 60], vec![forest; 60]], 81, Player::P0);
        member.decks[0][0] = card;
        if card == banned {
            assert!(matches!(
                Game::new(member.clone()),
                Err(Error::InvalidDeck(_))
            ));
            member.deck_rules = DeckRules::Freeform;
        }
        assert!(Game::new(member.clone()).is_ok(), "{}", card.name());
        if let Some(room) = card.room() {
            for half in room.halves {
                assert!(room_faces.insert(half));
                assert!(half.is_supported(), "{} is gated", half.name());
                assert!(!half.is_standard_legal());
                member.decks[0][0] = half;
                member.deck_rules = DeckRules::Freeform;
                assert!(matches!(
                    Game::new(member.clone()),
                    Err(Error::InvalidDeck(_))
                ));
            }
        }
    }
    assert_eq!(room_faces.len(), 46);
}

fn deck_cards(deck: &Deck, pool: &BTreeSet<Card>) -> Vec<Card> {
    let mut seen = BTreeSet::new();
    let mut cards = Vec::new();
    for row in &deck.cards {
        let card = identify(&row.identity);
        assert!(
            pool.contains(&card) && card.is_standard_legal() && card.is_supported(),
            "{} is unavailable in {}",
            card.name(),
            deck.name
        );
        assert!(seen.insert(card) && row.count > 0);
        assert!(row.count <= 4 || card.is_basic_land());
        cards.extend(std::iter::repeat_n(card, row.count));
    }
    assert_eq!(cards.len(), 60, "{}", deck.id);
    assert_eq!(cards.iter().filter(|card| card.is_land()).count(), 24);
    cards
}

fn prove_branch(game: &Game, view: &Observation, index: usize) -> bool {
    let Some(other) = view
        .actions
        .iter()
        .enumerate()
        .find_map(|(at, action)| (at != index && *action != Action::Concede).then_some(at))
    else {
        return false;
    };
    let actor = view.viewer;
    let saved = game.snapshot();
    let mut raw = game.clone();
    let mut indexed = game.clone();
    raw.apply(actor, view.actions[index].clone()).unwrap();
    indexed.apply_index(actor, view.revision, index).unwrap();
    let next = indexed.snapshot();
    assert_eq!(raw.snapshot(), next);
    assert_eq!(
        indexed.apply_index(actor, view.revision, index),
        Err(Error::StaleRevision)
    );
    assert_eq!(indexed.snapshot(), next);
    indexed.restore(&saved).unwrap();
    assert_eq!(indexed.snapshot(), saved);
    indexed
        .apply_index(actor, indexed.revision(), other)
        .unwrap();
    indexed.restore(&saved).unwrap();
    assert_eq!(indexed.snapshot(), saved);
    assert_eq!(game.snapshot(), saved);
    true
}

fn write_report(directory: Option<&Path>, filename: &str, report: &Value) {
    if let Some(directory) = directory {
        std::fs::write(
            directory.join(filename),
            serde_json::to_vec_pretty(report).unwrap(),
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
    fn note(&mut self, name: &'static str) {
        *self.events.entry(name).or_default() += 1;
    }

    fn transition(&mut self, before: &Observation, after: &Observation, action: &Action) {
        if after.last_manifest_dread != before.last_manifest_dread
            && after
                .last_manifest_dread
                .as_ref()
                .is_some_and(|event| event.manifested.is_some())
        {
            self.note("manifested");
        }
        if matches!(action, Action::PayFaceUp { .. })
            && let Some(payment) = before.face_up_payment
            && after
                .face_down_permanents
                .iter()
                .all(|row| row.object != payment.object)
        {
            self.note("turned face up");
        }
        if matches!(action, Action::PayRoomUnlock { .. })
            && let Some(payment) = before.room_unlock_payment
            && after.objects.iter().any(|object| {
                object.reference == payment.object
                    && object.room_doors.is_some_and(|doors| match payment.door {
                        RoomDoor::Left => doors.left,
                        RoomDoor::Right => doors.right,
                    })
            })
        {
            self.note("paid Room door unlocked");
        }
        for object in &after.objects {
            if object.zone == Zone::Battlefield
                && object.cast_for_impending
                && !object.is_creature()
                && object.counters.get(CounterKind::Time) > 0
                && !before
                    .objects
                    .iter()
                    .any(|old| old.reference == object.reference)
            {
                self.note("Impending entered with time counters");
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
                if let Some(trigger) = card.triggers().get(usize::from(ability)) {
                    if trigger.condition.event() == TriggerCondition::Eerie {
                        self.note("Eerie triggered");
                    }
                    if trigger.condition == TriggerCondition::SURVIVAL {
                        self.note("Survival triggered");
                    }
                    if matches!(card.name(), "Hand That Feeds" | "Osseous Sticktwister") {
                        self.note("Delirium triggered");
                    }
                }
            }
            if packet.kind == StackKind::Spell
                && let Some(cast) = packet.cast
            {
                self.colors |= card.colors();
                self.casts += 1;
                self.names.insert(cast.physical_card.unwrap_or(card).name());
                if matches!(cast.face, CastFace::RoomLeft | CastFace::RoomRight) {
                    self.note("Room half cast");
                }
                if cast.impending {
                    self.note("Impending cast");
                }
                if card.name() == "Drag to the Roots"
                    && cast.payment.into_iter().map(u64::from).sum::<u64>() == 2
                {
                    self.note("Delirium cost paid");
                }
            }
        }
    }
}

fn workload_index(view: &Observation) -> usize {
    let smoke = policy::smoke_index_from_observation(view, &view.actions);
    if matches!(view.actions[smoke], Action::PlayLand { .. }) {
        return smoke;
    }
    // Exercise each paid alternate route when the stock policy would spend
    // mana on a different available action. Only the acting seat's view is used.
    view.actions
        .iter()
        .position(|action| match action {
            Action::ChooseSpellCost { cost } => view
                .cost_choices
                .iter()
                .any(|row| row.index == *cost && row.impending),
            Action::ChooseManifest { object } => {
                view.manifest_choice.as_ref().is_some_and(|choice| {
                    choice
                        .candidates
                        .iter()
                        .any(|(exact, card)| exact == object && card.is_creature())
                })
            }
            Action::BeginRoomUnlock { .. } | Action::BeginFaceUp { .. } => true,
            _ => false,
        })
        .unwrap_or(smoke)
}

#[test]
fn duskmourn_standard_games_finish_and_replay_with_real_mechanic_events() {
    let workload: Workload =
        serde_json::from_str(include_str!("../data/decks/dsk-set-only.json")).unwrap();
    let pool = pool();
    let decks = workload
        .decks
        .iter()
        .map(|deck| (deck.id.as_str(), deck_cards(deck, &pool)))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(decks.len(), 5);
    assert_eq!(workload.decks.len(), decks.len());
    assert_eq!(workload.pairings.len(), decks.len());
    for id in decks.keys() {
        assert_eq!(
            workload
                .pairings
                .iter()
                .filter(|pair| pair.p0 == *id)
                .count(),
            1
        );
        assert_eq!(
            workload
                .pairings
                .iter()
                .filter(|pair| pair.p1 == *id)
                .count(),
            1
        );
    }
    let directory = std::env::var_os("RUSTMAGE_DSK_EVIDENCE_DIR").map(PathBuf::from);
    if let Some(directory) = &directory {
        std::fs::create_dir_all(directory).unwrap();
    }

    let mut coverage = Coverage::default();
    let mut wins = [0; 2];
    let mut draws = 0;
    let mut games = 0;
    let mut actions = 0;
    let mut branch_counts = [0; 2];
    let mut gameplay = Duration::ZERO;
    let mut replay = Duration::ZERO;
    for pair in &workload.pairings {
        assert_ne!(pair.p0, pair.p1);
        assert_eq!(pair.runs.len(), 2);
        assert_ne!(pair.runs[0].seed, pair.runs[1].seed);
        assert_eq!(pair.runs[0].starting_player, 0);
        assert_eq!(pair.runs[1].starting_player, 1);
        for run in &pair.runs {
            let label = format!("{}-{}-{}", pair.p0, pair.p1, run.seed);
            let input = config(
                [
                    decks[pair.p0.as_str()].clone(),
                    decks[pair.p1.as_str()].clone(),
                ],
                run.seed,
                [Player::P0, Player::P1][run.starting_player],
            );
            let mut game = Game::new(input.clone()).unwrap();
            let mut journal = Vec::new();
            let mut branched = [false; 2];
            let started = Instant::now();
            while game.outcome().is_none() && journal.len() < 4096 {
                let actor = game.actor().unwrap();
                let view = game.observe(actor);
                check_privacy(&view, actor);
                let index = workload_index(&view);
                let action = view.actions[index].clone();
                assert_ne!(action, Action::Concede);
                let private = view.manifest_choice.is_some()
                    || (private_choice(&view)
                        && view
                            .library_selection
                            .as_ref()
                            .is_none_or(|selection| !selection.public));
                if private {
                    let other = game.observe(actor.other());
                    check_privacy(&other, actor);
                    assert!(other.manifest_choice.is_none());
                }
                let branch_kind = usize::from(private);
                if !branched[branch_kind] && !matches!(action, Action::Keep | Action::Pass) {
                    branched[branch_kind] = prove_branch(&game, &view, index);
                }
                if let Err(error) = game.apply(actor, action.clone()) {
                    write_report(
                        directory.as_deref(),
                        &format!("{label}-failure.json"),
                        &json!({
                            "config": input, "actions": journal, "actor": actor, "failing_action": action,
                            "error": error.to_string(),
                        }),
                    );
                    panic!("{label}, decision {} {action:?}: {error}", journal.len());
                }
                let after = game.observe(actor);
                coverage.transition(&view, &after, &action);
                journal.push((actor, action));
            }
            gameplay += started.elapsed();
            write_report(
                directory.as_deref(),
                &format!("{label}.json"),
                &json!({
                    "catalogue": rustmage::CATALOGUE_VERSION, "config": input, "actions": journal,
                    "outcome": game.outcome(), "public_branch": branched[0], "private_branch": branched[1],
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
            for (count, checked) in branch_counts.iter_mut().zip(branched) {
                *count += usize::from(checked);
            }

            let started = Instant::now();
            let mut rerun = Game::new(input).unwrap();
            for (actor, action) in journal {
                rerun.apply(actor, action).unwrap();
            }
            assert_eq!(rerun.snapshot(), game.snapshot(), "{label}");
            replay += started.elapsed();
        }
    }

    let report = json!({
        "games": games, "decisions": actions, "wins": wins, "draws": draws,
        "colors": coverage.colors, "actual_casts": coverage.casts, "cast_names": coverage.names,
        "events": coverage.events, "public_branches": branch_counts[0],
        "private_branches": branch_counts[1], "gameplay_seconds": gameplay.as_secs_f64(),
        "replay_seconds": replay.as_secs_f64(), "decisions_per_second": actions as f64 / gameplay.as_secs_f64(),
    });
    println!("Duskmourn-only Standard {report}");
    write_report(directory.as_deref(), "summary.json", &report);
    assert_eq!(games, 10);
    assert_eq!(coverage.colors, 31, "{:?}", coverage.names);
    assert!(branch_counts[0] >= 5 && branch_counts[1] > 0);
    assert!(
        coverage.events.contains_key("Delirium triggered")
            || coverage.events.contains_key("Delirium cost paid"),
        "{:?}",
        coverage.events
    );
    for required in [
        "Room half cast",
        "paid Room door unlocked",
        "Eerie triggered",
        "Impending cast",
        "Impending entered with time counters",
        "Survival triggered",
        "manifested",
        "turned face up",
    ] {
        assert!(
            coverage.events.contains_key(required),
            "missing {required} in {:?}",
            coverage.events
        );
    }
}
