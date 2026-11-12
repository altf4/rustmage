//! Karlov Manor-only Standard decks, observable events, and owned replay.
use rustmage::{
    Action, Card, CastFace, Config, DeckRules, EffectChoiceKind, Error, FaceDownCause, Game,
    Observation, Outcome, Player, StackKind, Zone, policy,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

#[allow(dead_code)]
#[path = "support/tmt_evidence.rs"]
mod tmt_evidence;
use tmt_evidence::{check_privacy, private_choice};

const CLUE: Card = Card::from_catalogue_id(5296);
const PLAYERS: [Player; 2] = [Player::P0, Player::P1];

#[derive(Deserialize)]
struct Identity {
    name: String,
    oracle_id: String,
}

impl Identity {
    fn card(&self) -> Card {
        let card = Card::from_name(&self.name).unwrap_or_else(|| panic!("unknown {}", self.name));
        assert_eq!(card.info().oracle_id, Some(self.oracle_id.as_str()));
        card
    }
}

#[derive(Deserialize)]
struct FrozenSet {
    card_count: usize,
    cards: Vec<Identity>,
    unresolved: Vec<String>,
}

fn pool() -> BTreeSet<Card> {
    let set: FrozenSet = serde_json::from_str(include_str!("../data/sets/mkm.json")).unwrap();
    assert_eq!(set.card_count, 279);
    assert!(set.unresolved.is_empty());
    let pool = set
        .cards
        .iter()
        .map(Identity::card)
        .collect::<BTreeSet<_>>();
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
fn karlov_manor_admits_every_physical_card_under_its_exact_format() {
    let pool = pool();
    let banned = Card::from_name("Proft's Eidetic Memory").unwrap();
    assert!(pool.contains(&banned));
    let mut spell_faces = BTreeSet::new();
    let mut split_cards = 0;
    let mut adventures = 0;
    for card in pool {
        assert!(card.is_supported(), "{} is gated", card.name());
        assert_eq!(card.is_standard_legal(), card != banned, "{}", card.name());
        let mut member = config(
            [vec![Card::Forest; 60], vec![Card::Forest; 60]],
            81,
            Player::P0,
        );
        member.decks[0][0] = card;
        if card == banned {
            assert!(matches!(
                Game::new(member.clone()),
                Err(Error::InvalidDeck(_))
            ));
            member.deck_rules = DeckRules::Freeform;
        }
        assert!(Game::new(member).is_ok(), "{}", card.name());
        if let Some(halves) = card.split_faces() {
            split_cards += 1;
            for half in halves {
                assert!(spell_faces.insert(half));
            }
        }
        if let Some(face) = card.adventure_spell() {
            adventures += 1;
            assert!(spell_faces.insert(face));
        }
    }
    assert_eq!((split_cards, adventures, spell_faces.len()), (5, 1, 11));

    let tokens = (5680..=5686).map(Card::from_catalogue_id).chain([CLUE]);
    for card in spell_faces.into_iter().chain(tokens) {
        assert!(
            card.is_supported() && !card.is_standard_legal(),
            "{}",
            card.name()
        );
        let mut member = config(
            [vec![Card::Forest; 60], vec![Card::Forest; 60]],
            81,
            Player::P0,
        );
        member.decks[0][0] = card;
        for rules in [DeckRules::Standard, DeckRules::Freeform] {
            member.deck_rules = rules;
            assert!(
                matches!(Game::new(member.clone()), Err(Error::InvalidDeck(_))),
                "{}",
                card.name()
            );
        }
    }
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

fn deck_cards(deck: &Deck, pool: &BTreeSet<Card>) -> Vec<Card> {
    let mut seen = BTreeSet::new();
    let mut cards = Vec::new();
    for row in &deck.cards {
        let card = row.identity.card();
        assert!(pool.contains(&card), "{} in {}", card.name(), deck.name);
        assert!(seen.insert(card) && row.count > 0);
        assert!(row.count <= 4 || card.is_basic_land());
        cards.extend(std::iter::repeat_n(card, row.count));
    }
    assert_eq!(cards.len(), 60, "{}", deck.name);
    assert_eq!(cards.iter().filter(|card| card.is_land()).count(), 24);
    cards
}

fn cloak_choice(view: &Observation) -> bool {
    view.effect_choice
        .as_ref()
        .is_some_and(|choice| choice.kind == EffectChoiceKind::CloakFromHand)
}

fn proves_owned_branch(game: &Game, view: &Observation, chosen: usize) -> bool {
    let Some(alternate) = view.actions.iter().enumerate().find_map(|(index, action)| {
        (index != chosen && *action != Action::Concede).then_some(index)
    }) else {
        return false;
    };
    let actor = view.viewer;
    let saved = game.snapshot();
    let mut raw = game.clone();
    let mut indexed = game.clone();
    raw.apply(actor, view.actions[chosen].clone()).unwrap();
    indexed.apply_index(actor, view.revision, chosen).unwrap();
    let successor = indexed.snapshot();
    assert_eq!(raw.snapshot(), successor);
    assert_eq!(
        indexed.apply_index(actor, view.revision, chosen),
        Err(Error::StaleRevision)
    );
    assert_eq!(indexed.snapshot(), successor);
    indexed.restore(&saved).unwrap();
    indexed
        .apply_index(actor, indexed.revision(), alternate)
        .unwrap();
    indexed.restore(&saved).unwrap();
    assert_eq!(indexed.snapshot(), saved);
    assert_eq!(game.snapshot(), saved);
    true
}

fn workload_index(view: &Observation) -> usize {
    let smoke = policy::smoke_index_from_observation(view, &view.actions);
    if matches!(view.actions[smoke], Action::PlayLand { .. }) {
        return smoke;
    }
    view.actions
        .iter()
        .position(|action| {
            matches!(
                action,
                Action::BeginFaceUp { .. }
                    | Action::BeginFaceUpX { .. }
                    | Action::AnnounceDisguise { .. }
                    | Action::AnnounceSplit { .. }
                    | Action::AnnounceAdventure { .. }
            )
        })
        .unwrap_or(smoke)
}

#[derive(Default)]
struct Coverage {
    colors: u8,
    casts: usize,
    names: BTreeSet<&'static str>,
    events: BTreeMap<&'static str, usize>,
}

impl Coverage {
    fn add(&mut self, name: &'static str, amount: usize) {
        if amount != 0 {
            *self.events.entry(name).or_default() += amount;
        }
    }

    fn transition(&mut self, before: &Observation, after: &Observation, action: &Action) {
        for face in &after.face_down_permanents {
            if before
                .face_down_permanents
                .iter()
                .all(|old| old.object != face.object)
            {
                match face.cause {
                    FaceDownCause::Disguise => self.add("disguised creature entered", 1),
                    FaceDownCause::Cloak => self.add("cloaked creature entered", 1),
                    _ => (),
                }
            }
        }
        if matches!(action, Action::PayFaceUp { .. })
            && let Some(payment) = before.face_up_payment
            && after
                .face_down_permanents
                .iter()
                .all(|row| row.object != payment.object)
        {
            self.add("paid turn face up", 1);
        }
        if let Some(evidence) = &after.last_evidence
            && before.last_evidence.as_ref().map(|old| old.sequence) != Some(evidence.sequence)
        {
            assert!(evidence.total_mana >= u128::from(evidence.threshold));
            self.add("evidence collected", 1);
        }
        self.add(
            "Case solved",
            after
                .solved_cases
                .iter()
                .filter(|row| !before.solved_cases.contains(row))
                .count(),
        );
        self.add(
            "creature suspected",
            after
                .suspected
                .iter()
                .filter(|row| !before.suspected.contains(row))
                .count(),
        );
        self.add(
            "combat damage to player",
            after
                .combat_damaged_players
                .iter()
                .filter(|row| !before.combat_damaged_players.contains(row))
                .count(),
        );
        self.add(
            "Clue token entered",
            after
                .objects
                .iter()
                .filter(|row| {
                    row.zone == Zone::Battlefield
                        && row.card == CLUE
                        && before
                            .objects
                            .iter()
                            .all(|old| old.reference != row.reference)
                })
                .count(),
        );

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
            let Some(cast) = packet.cast else {
                continue;
            };
            self.colors |= card.colors();
            self.casts += 1;
            self.names.insert(cast.physical_card.unwrap_or(card).name());
            if matches!(cast.face, CastFace::SplitLeft | CastFace::SplitRight) {
                self.add("split half cast", 1);
            }
            if cast.face == CastFace::Adventure {
                self.add("Adventure cast", 1);
            }
        }
    }
}

fn write_report(directory: Option<&Path>, filename: &str, report: Value) {
    if let Some(directory) = directory {
        std::fs::write(
            directory.join(filename),
            serde_json::to_vec_pretty(&report).unwrap(),
        )
        .unwrap();
    }
}

#[test]
fn karlov_manor_standard_games_finish_and_replay_with_set_mechanics() {
    let workload: Workload =
        serde_json::from_str(include_str!("../data/decks/mkm-set-only.json")).unwrap();
    let pool = pool();
    let decks = workload
        .decks
        .iter()
        .map(|deck| (deck.id.as_str(), deck_cards(deck, &pool)))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(decks.len(), 6);
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
    let directory = std::env::var_os("RUSTMAGE_MKM_EVIDENCE_DIR")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("RUSTMAGE_TDM_EVIDENCE_DIR").map(|dir| PathBuf::from(dir).join("mkm"))
        });
    if let Some(directory) = &directory {
        std::fs::create_dir_all(directory).unwrap();
    }

    let mut coverage = Coverage::default();
    let mut wins = [0; 2];
    let mut draws = 0;
    let mut games = 0;
    let mut decisions = 0;
    let mut branch_counts = [0; 2];
    let mut private_decisions = 0;
    let mut gameplay = Duration::ZERO;
    let mut replay = Duration::ZERO;
    for pair in &workload.pairings {
        assert_ne!(pair.p0, pair.p1);
        assert_eq!(pair.runs.len(), 2);
        assert_eq!(
            [pair.runs[0].starting_player, pair.runs[1].starting_player],
            [0, 1]
        );
        assert_ne!(pair.runs[0].seed, pair.runs[1].seed);
        for run in &pair.runs {
            let label = format!("{}-{}-{}", pair.p0, pair.p1, run.seed);
            let input = config(
                [
                    decks[pair.p0.as_str()].clone(),
                    decks[pair.p1.as_str()].clone(),
                ],
                run.seed,
                PLAYERS[run.starting_player],
            );
            let mut game = Game::new(input.clone()).unwrap();
            let mut journal = Vec::new();
            let mut branched = [false; 2];
            let started = Instant::now();
            while let Some(actor) = game.actor() {
                if journal.len() == 4096 {
                    break;
                }
                let view = game.observe(actor);
                check_privacy(&view, actor);
                let index = workload_index(&view);
                let action = view.actions[index].clone();
                assert_ne!(action, Action::Concede);
                let private = cloak_choice(&view)
                    || view.evidence.is_some()
                    || view.manifest_choice.is_some()
                    || (private_choice(&view)
                        && view
                            .library_selection
                            .as_ref()
                            .is_none_or(|choice| !choice.public));
                if private {
                    let other = game.observe(actor.other());
                    check_privacy(&other, actor);
                    assert!(!cloak_choice(&other));
                    assert!(other.evidence.is_none() && other.manifest_choice.is_none());
                    private_decisions += 1;
                }
                let branch_kind = usize::from(private);
                if !branched[branch_kind] && !matches!(action, Action::Keep | Action::Pass) {
                    branched[branch_kind] = proves_owned_branch(&game, &view, index);
                }

                if let Err(error) = game.apply_index(actor, view.revision, index) {
                    write_report(
                        directory.as_deref(),
                        &format!("{label}-failure.json"),
                        json!({
                            "config": input, "actions": journal, "actor": actor,
                            "failing_action": action, "error": error.to_string(),
                        }),
                    );
                    panic!("{label}, decision {} {action:?}: {error}", journal.len());
                }
                let after = game.observe(actor);
                let new_hidden: Vec<_> = after
                    .face_down_permanents
                    .iter()
                    .chain(&after.face_down_spells)
                    .filter(|row| {
                        row.underlying.is_some()
                            && view
                                .face_down_permanents
                                .iter()
                                .chain(&view.face_down_spells)
                                .all(|old| old.object != row.object)
                    })
                    .collect();
                if !new_hidden.is_empty() && game.outcome().is_none() {
                    let other = game.observe(actor.other());
                    assert!(new_hidden.iter().all(|new| {
                        other
                            .face_down_permanents
                            .iter()
                            .chain(&other.face_down_spells)
                            .any(|row| {
                                row.object == new.object
                                    && row.underlying.is_none()
                                    && row.cause == new.cause
                            })
                    }));
                }
                coverage.transition(&view, &after, &action);
                journal.push((actor, action));
            }
            gameplay += started.elapsed();
            write_report(
                directory.as_deref(),
                &format!("{label}.json"),
                json!({
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
            decisions += journal.len();
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
        "games": games, "decisions": decisions, "wins": wins, "draws": draws,
        "colors": coverage.colors, "actual_casts": coverage.casts, "cast_names": coverage.names,
        "events": coverage.events, "public_branches": branch_counts[0],
        "private_branches": branch_counts[1], "private_decisions": private_decisions,
        "gameplay_seconds": gameplay.as_secs_f64(), "replay_seconds": replay.as_secs_f64(),
        "decisions_per_second": decisions as f64 / gameplay.as_secs_f64(),
    });
    println!("Karlov Manor-only Standard {report}");
    write_report(directory.as_deref(), "summary.json", report);
    assert_eq!(games, 12);
    assert_eq!(coverage.colors, 31, "{:?}", coverage.names);
    assert_eq!(branch_counts[0], games);
    assert!(branch_counts[1] > 0);
    for required in [
        "disguised creature entered",
        "cloaked creature entered",
        "paid turn face up",
        "evidence collected",
        "Case solved",
        "creature suspected",
        "combat damage to player",
        "Clue token entered",
        "split half cast",
        "Adventure cast",
    ] {
        assert!(
            coverage.events.contains_key(required),
            "missing {required} in {:?}",
            coverage.events
        );
    }
}
