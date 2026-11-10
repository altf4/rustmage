//! Wilds-only Standard games and the frozen 281-card acceptance gate.
//! Individual printed rules, including the three bans' Freeform effects, are
//! exercised by the WOE card scenarios in the native suite.
use rustmage::{
    policy, Action, Card, CastFace, CastMethod, Config, DeckRules, Error, Game, Keywords,
    Observation, Outcome, Player, StackKind, Step, Zone, CATALOGUE_VERSION,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

#[allow(dead_code)]
#[path = "support/tmt_evidence.rs"]
mod tmt_evidence;
use tmt_evidence::{check_privacy, private_choice};

const BANNED: [&str; 3] = ["Hopeless Nightmare", "Monstrous Rage", "Up the Beanstalk"];
const MAX_DECISIONS: usize = 4096;

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
    #[serde(default)]
    prefer_first_adventure: bool,
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

fn identified_card(identity: &Identity) -> Card {
    let card = Card::from_name(&identity.name)
        .unwrap_or_else(|| panic!("unknown WOE identity {}", identity.name));
    assert_eq!(
        card.info().oracle_id,
        Some(identity.oracle_id.as_str()),
        "{}",
        identity.name,
    );
    card
}

fn frozen_pool() -> BTreeMap<Card, Identity> {
    let set: FrozenSet = serde_json::from_str(include_str!("../data/sets/woe.json")).unwrap();
    assert_eq!(set.card_count, 281);
    assert!(set.unresolved.is_empty());
    let pool = set
        .cards
        .into_iter()
        .map(|row| (identified_card(&row), row))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(pool.len(), set.card_count);
    pool
}

#[test]
fn woe_frozen_pool_requires_all_physical_cards_and_adventure_faces() {
    let pool = frozen_pool();
    let gated = pool
        .iter()
        .filter(|(card, _)| !card.is_supported())
        .map(|(_, identity)| identity.name.as_str())
        .collect::<Vec<_>>();
    assert!(gated.is_empty(), "gated Wilds identities {gated:?}");
    let mut adventure_count = 0;
    let mut banned_count = 0;
    for (&card, identity) in &pool {
        if let Some((_, spell_name)) = identity.name.split_once(" // ") {
            let inset = card
                .adventure_spell()
                .unwrap_or_else(|| panic!("missing Adventure of {}", identity.name));
            assert!(inset.is_supported(), "gated Adventure {spell_name}");
            assert_eq!(inset.name(), spell_name);
            assert_eq!(inset.adventure_parent(), Some(card));
            adventure_count += 1;
        }
        let banned = BANNED.contains(&identity.name.as_str());
        assert_eq!(card.is_standard_legal(), !banned, "{}", identity.name);
        if banned {
            banned_count += 1;
            let forest = Card::from_name("Forest").unwrap();
            assert!(pool.contains_key(&forest));
            let mut config = Config {
                decks: [vec![forest; 60], vec![forest; 60]],
                seed: 0,
                starting_player: Player::P0,
                deck_rules: DeckRules::Standard,
            };
            config.decks[0][0] = card;
            assert!(matches!(
                Game::new(config.clone()),
                Err(Error::InvalidDeck(reason))
                    if reason == format!("not Standard legal in this catalogue: {}", card.name())
            ));
            config.deck_rules = DeckRules::Freeform;
            Game::new(config).unwrap();
        }
    }
    assert_eq!(adventure_count, 51);
    assert_eq!(banned_count, BANNED.len());
}

fn deck_cards(deck: &Deck, pool: &BTreeMap<Card, Identity>) -> Vec<Card> {
    let mut seen = BTreeSet::new();
    let mut cards = Vec::new();
    for row in &deck.cards {
        let card = identified_card(&row.identity);
        assert_eq!(
            pool.get(&card).map(|identity| &identity.name),
            Some(&row.identity.name),
            "{} outside WOE",
            row.identity.name,
        );
        assert!(card.is_standard_legal() && card.is_supported());
        assert!(seen.insert(card) && row.count > 0, "{}", deck.id);
        cards.extend(std::iter::repeat_n(card, row.count));
    }
    assert_eq!(cards.len(), 60, "{}", deck.id);
    cards
}

fn player_views(game: &Game) -> Value {
    let mut observations = [game.observe(Player::P0), game.observe(Player::P1)];
    for observation in &mut observations {
        observation.revision = 0;
    }
    serde_json::to_value(observations).unwrap()
}

fn prove_branch(game: &Game, actor: Player, actions: &[Action], index: usize) -> bool {
    let chosen = &actions[index];
    let Some(alternative) = actions
        .iter()
        .find(|action| *action != chosen && !matches!(action, Action::Concede))
    else {
        return false;
    };
    let parent = game.clone();
    let saved = parent.snapshot();
    let revision = parent.revision();
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    let mut sibling = parent.clone();
    drop(parent);

    assert_eq!(
        raw.apply(actor.other(), chosen.clone()),
        Err(Error::WrongActor)
    );
    assert_eq!(
        raw.apply(actor, Action::ChooseMode { mode: u16::MAX }),
        Err(Error::IllegalAction),
    );
    assert_eq!(raw.snapshot(), saved);
    assert_eq!(raw.revision(), revision);
    raw.apply(actor, chosen.clone()).unwrap();
    indexed.apply_index(actor, revision, index).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(player_views(&raw), player_views(&indexed));
    assert_eq!(sibling.snapshot(), saved);
    sibling.apply(actor, alternative.clone()).unwrap();
    assert_eq!(game.snapshot(), saved);
    if sibling.snapshot() == raw.snapshot() {
        return false;
    }
    sibling.restore(&saved).unwrap();
    assert_eq!(sibling.snapshot(), saved);
    assert_eq!(player_views(&sibling), player_views(game));
    assert_eq!(
        sibling.apply_index(actor, revision, index),
        Err(Error::StaleRevision),
    );
    sibling
        .apply_index(actor, sibling.revision(), index)
        .unwrap();
    assert_eq!(sibling.snapshot(), raw.snapshot());
    assert_eq!(player_views(&sibling), player_views(&raw));
    true
}

#[derive(Default)]
struct Coverage {
    mechanics: BTreeSet<&'static str>,
    cast_names: BTreeSet<&'static str>,
    casts: usize,
    colors: u8,
    branches: usize,
    private_branches: usize,
    private_choices: usize,
}

impl Coverage {
    fn transition(&mut self, before: &Observation, after: &Observation, action: &Action) {
        for packet in &after.stack {
            if before.stack.iter().any(|old| old.id == packet.id) {
                continue;
            }
            let Some((_, card)) = packet.source_identity() else {
                continue;
            };
            if matches!(packet.kind, StackKind::TriggeredAbility { .. }) && card.has_subtype("Saga")
            {
                self.mechanics.insert("Saga chapter");
            }
            let Some(cast) = packet.cast.filter(|_| !packet.is_copy) else {
                continue;
            };
            if packet.kind != StackKind::Spell {
                continue;
            }
            self.casts += 1;
            self.cast_names.insert(card.name());
            self.colors |= card.colors();
            if cast.face == CastFace::Adventure {
                self.mechanics.insert("Adventure cast");
            }
            if cast.origin == Zone::Exile
                && cast.method != CastMethod::Free
                && cast.face == CastFace::Normal
                && card.adventure_spell().is_some()
            {
                self.mechanics.insert("adventurer cast from exile");
            }
            if cast.bargain == Some(true) {
                self.mechanics.insert("Bargain paid");
            }
        }

        if after
            .adventures
            .iter()
            .any(|right| !before.adventures.contains(right))
        {
            self.mechanics.insert("Adventure resolved to exile");
        }
        for object in &after.objects {
            if object.zone != Zone::Battlefield {
                continue;
            }
            let old = before
                .objects
                .iter()
                .find(|old| old.reference == object.reference);
            if object.is_token() && old.is_none_or(|old| old.zone != Zone::Battlefield) {
                for (subtype, event) in [
                    ("Role", "Role attached"),
                    ("Food", "Food created"),
                    ("Treasure", "Treasure created"),
                    ("Rat", "Rat created"),
                ] {
                    if object.has_subtype(subtype)
                        && (subtype != "Role" || object.attached_to.is_some())
                    {
                        self.mechanics.insert(event);
                    }
                }
            }
            if after.players[object.controller.index()].nonland_permanents_entered_this_turn >= 2
                && match object.name {
                    "Armory Mice" => {
                        object.toughness >= 3
                            && old.is_some_and(|old| old.toughness + 2 == object.toughness)
                    }
                    "Grand Ball Guest" => {
                        object.keywords.contains(Keywords::TRAMPLE)
                            && old.is_none_or(|old| !old.keywords.contains(Keywords::TRAMPLE))
                    }
                    _ => false,
                }
            {
                self.mechanics.insert("Celebration payoff");
            }
        }
        if matches!(action, Action::Pass)
            && matches!(
                before.step,
                Step::Blockers | Step::FirstStrikeDamage | Step::CombatDamage
            )
            && before.stack.is_empty()
            && before
                .players
                .iter()
                .zip(&after.players)
                .any(|(old, new)| new.life < old.life)
        {
            self.mechanics.insert("combat damage");
        }
    }
}

fn evidence(directory: Option<&Path>, name: &str, data: Value) {
    if let Some(directory) = directory {
        std::fs::write(
            directory.join(name),
            serde_json::to_vec_pretty(&data).unwrap(),
        )
        .unwrap();
    }
}

fn private_decision(game: &Game, view: &Observation, actor: Player) -> bool {
    check_privacy(view, actor);
    let beseech = view.resolution_free_casts.as_ref().is_some_and(|offer| {
        !offer.remaining.is_empty()
            && view
                .resolving
                .as_ref()
                .and_then(|item| item.source_identity())
                .is_some_and(|(_, card)| card.name() == "Beseech the Mirror")
    });
    let private = private_choice(view) || beseech;
    if private {
        let other = game.observe(actor.other());
        check_privacy(&other, actor);
        if beseech {
            let redacted = other.resolution_free_casts.as_ref().unwrap();
            assert!(redacted.remaining.is_empty() && redacted.paid.is_none());
            for exact in &view.resolution_free_casts.as_ref().unwrap().remaining {
                assert!(!other
                    .objects
                    .iter()
                    .any(|object| object.reference == *exact));
            }
        }
    }
    private
}

#[test]
fn woe_standard_terminal_games_replay_and_exercise_mechanics() {
    let pool = frozen_pool();
    let workload: Workload =
        serde_json::from_str(include_str!("../data/decks/woe-set-only.json")).unwrap();
    let decks = workload
        .decks
        .iter()
        .map(|deck| {
            (
                deck.id.as_str(),
                (deck.name.as_str(), deck_cards(deck, &pool)),
            )
        })
        .collect::<BTreeMap<_, _>>();
    assert_eq!(decks.len(), 5);
    let directory =
        std::env::var_os("RUSTMAGE_TDM_EVIDENCE_DIR").map(|path| PathBuf::from(path).join("woe"));
    if let Some(directory) = &directory {
        std::fs::create_dir_all(directory).unwrap();
    }
    let started = Instant::now();
    let mut gameplay = Duration::ZERO;
    let mut replay_time = Duration::ZERO;
    let mut total = Coverage::default();
    let mut decisions = 0usize;
    let mut completed = 0usize;
    let mut winners = [0usize; 2];
    let mut draws = 0usize;
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
            let mut cover = Coverage::default();
            let mut seek_adventure = [run.prefer_first_adventure; 2];
            let play_started = Instant::now();
            while game.outcome().is_none() && journal.len() < MAX_DECISIONS {
                let actor = game.actor().unwrap();
                let before = game.observe(actor);
                let actions = &before.actions;
                let preferred = seek_adventure[actor.index()]
                    .then(|| {
                        actions
                            .iter()
                            .position(|action| matches!(action, Action::AnnounceAdventure { .. }))
                    })
                    .flatten();
                let index = preferred
                    .unwrap_or_else(|| policy::smoke_index_from_observation(&before, actions));
                let action = actions[index].clone();
                assert_ne!(action, Action::Concede, "{label}");
                if matches!(action, Action::AnnounceAdventure { .. }) {
                    seek_adventure[actor.index()] = false;
                }
                let private = private_decision(&game, &before, actor);
                cover.private_choices += usize::from(private);
                if (cover.branches == 0 || private && cover.private_branches == 0)
                    && !matches!(action, Action::Keep | Action::Pass)
                    && prove_branch(&game, actor, actions, index)
                {
                    cover.branches += 1;
                    cover.private_branches += usize::from(private);
                }
                if let Err(error) = game.apply(actor, action.clone()) {
                    evidence(
                        directory.as_deref(),
                        &format!("{label}-failure.json"),
                        json!({
                            "config": config, "actions": journal, "actor": actor, "failing_action": action,
                            "error": format!("{error:?}"),
                        }),
                    );
                    panic!("{label} decision {} {action:?}: {error:?}", journal.len());
                }
                cover.transition(&before, &game.observe(actor), &action);
                journal.push((actor, action));
            }
            gameplay += play_started.elapsed();
            evidence(
                directory.as_deref(),
                &format!("{label}.json"),
                json!({
                    "catalogue": CATALOGUE_VERSION,
                    "config": config,
                    "decks": [left_name, right_name],
                    "actions": journal,
                    "outcome": game.outcome(),
                    "spell_colors": cover.colors,
                    "actual_casts": cover.casts,
                    "cast_names": cover.cast_names,
                    "mechanics": cover.mechanics,
                    "owned_branches": cover.branches,
                    "owned_private_branches": cover.private_branches,
                }),
            );
            assert!(
                game.outcome().is_some() && cover.casts > 0 && cover.branches > 0,
                "{label} incomplete after {} decisions",
                journal.len(),
            );

            let replay_started = Instant::now();
            let mut replay = Game::new(config).unwrap();
            for (actor, action) in &journal {
                replay.apply(*actor, action.clone()).unwrap();
            }
            assert_eq!(game.snapshot(), replay.snapshot(), "{label}");
            assert_eq!(game.outcome(), replay.outcome(), "{label}");
            assert_eq!(player_views(&game), player_views(&replay), "{label}");
            replay_time += replay_started.elapsed();

            match game.outcome().unwrap() {
                Outcome::Winner(player) => winners[player.index()] += 1,
                Outcome::Draw => draws += 1,
            }
            completed += 1;
            decisions += journal.len();
            total.colors |= cover.colors;
            total.casts += cover.casts;
            total.cast_names.extend(cover.cast_names);
            total.mechanics.extend(cover.mechanics);
            total.branches += cover.branches;
            total.private_branches += cover.private_branches;
            total.private_choices += cover.private_choices;
        }
    }
    evidence(
        directory.as_deref(),
        "summary.json",
        json!({
            "catalogue": CATALOGUE_VERSION,
            "scope": "Natural WOE-only Standard games with transition checks and deterministic replay",
            "completed_games": completed,
            "decisions": decisions,
            "winners": winners,
            "draws": draws,
            "spell_colors": total.colors,
            "actual_casts": total.casts,
            "cast_names": total.cast_names,
            "mechanics": total.mechanics,
            "owned_branches": total.branches,
            "owned_private_branches": total.private_branches,
            "private_choices": total.private_choices,
            "gameplay_with_checks_seconds": gameplay.as_secs_f64(),
            "replay_with_checks_seconds": replay_time.as_secs_f64(),
            "gameplay_checked_decisions_per_second": decisions as f64 / gameplay.as_secs_f64(),
            "replay_checked_decisions_per_second": decisions as f64 / replay_time.as_secs_f64(),
            "verification_seconds": started.elapsed().as_secs_f64(),
        }),
    );
    assert_eq!(completed, 15);
    assert!(winners.iter().sum::<usize>() > 0);
    assert_eq!(total.colors & 31, 31);
    assert!(total.private_branches > 0);
    for mechanic in [
        "Adventure cast",
        "Adventure resolved to exile",
        "adventurer cast from exile",
        "Bargain paid",
        "Role attached",
        "Celebration payoff",
        "Food created",
        "Treasure created",
        "Rat created",
        "Saga chapter",
        "combat damage",
    ] {
        assert!(
            total.mechanics.contains(mechanic),
            "games missed {mechanic}"
        );
    }
}
