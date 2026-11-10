//! Natural terminal games with only admitted SPM cards and the shared smoke policy.
use rustmage::*;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::time::Instant;

#[derive(Deserialize)]
struct Deck {
    id: String,
    name: String,
    cards: Vec<DeckCard>,
}

#[derive(Deserialize)]
struct DeckCard {
    name: String,
    oracle_id: String,
    count: usize,
}

#[derive(Deserialize)]
struct Pairing {
    p0: String,
    p1: String,
    runs: Vec<Run>,
}

#[derive(Deserialize)]
struct Run {
    seed: u64,
    starting_player: usize,
}

#[derive(Deserialize)]
struct Workload {
    decks: Vec<Deck>,
    pairings: Vec<Pairing>,
}

fn views(game: &Game) -> serde_json::Value {
    let mut observations = [game.observe(Player::P0), game.observe(Player::P1)];
    for observation in &mut observations {
        observation.revision = 0;
    }
    serde_json::to_value(observations).unwrap()
}

fn prove_branch(game: &Game, actor: Player, actions: &[Action], index: usize) -> bool {
    let parent = game.clone();
    let saved = parent.snapshot();
    let revision = parent.revision();
    let before_views = views(&parent);
    let action = &actions[index];
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    let mut sibling = parent.clone();
    let mut restored = parent.clone();
    drop(parent);
    assert_eq!(
        raw.apply(actor.other(), action.clone()),
        Err(Error::WrongActor)
    );
    assert_eq!(raw.snapshot(), saved);
    raw.apply(actor, action.clone()).unwrap();
    indexed.apply_index(actor, revision, index).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(views(&raw), views(&indexed));
    assert_eq!(sibling.snapshot(), saved);
    assert_eq!(views(&sibling), before_views);
    let Some(alternative) = actions.iter().find(|candidate| {
        if *candidate == action || matches!(candidate, Action::Concede) {
            return false;
        }
        sibling.apply(actor, (*candidate).clone()).unwrap();
        if sibling.snapshot() != raw.snapshot() {
            return true;
        }
        sibling.restore(&saved).unwrap();
        false
    }) else {
        return false;
    };
    assert_eq!(game.snapshot(), saved);
    assert_ne!(sibling.snapshot(), raw.snapshot());
    restored.apply(actor, alternative.clone()).unwrap();
    assert_eq!(restored.snapshot(), sibling.snapshot());
    restored.restore(&saved).unwrap();
    assert_eq!(restored.snapshot(), saved);
    assert_eq!(views(&restored), before_views);
    assert_eq!(
        restored.apply_index(actor, revision, index),
        Err(Error::StaleRevision)
    );
    restored
        .apply_index(actor, restored.revision(), index)
        .unwrap();
    assert_eq!(restored.snapshot(), raw.snapshot());
    assert_eq!(views(&restored), views(&raw));
    true
}

fn decks(workload: &Workload) -> BTreeMap<&str, (&str, Vec<Card>)> {
    let manifest: serde_json::Value =
        serde_json::from_str(include_str!("../data/sets/spm.json")).unwrap();
    let pool: BTreeSet<_> = manifest["cards"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            let name = row["name"].as_str().unwrap();
            let card = Card::from_name(name).unwrap_or_else(|| panic!("missing {name}"));
            assert!(
                card.is_standard_legal() && card.is_supported(),
                "gated {name}"
            );
            card
        })
        .collect();
    assert_eq!(pool.len(), 193);
    workload
        .decks
        .iter()
        .map(|deck| {
            let cards: Vec<_> = deck
                .cards
                .iter()
                .flat_map(|row| {
                    let card = Card::from_name(&row.name)
                        .unwrap_or_else(|| panic!("missing {}", row.name));
                    assert!(pool.contains(&card), "{} is outside SPM", row.name);
                    assert_eq!(card.info().oracle_id, Some(row.oracle_id.as_str()));
                    std::iter::repeat_n(card, row.count)
                })
                .collect();
            assert_eq!(cards.len(), 60, "{}", deck.name);
            (deck.id.as_str(), (deck.name.as_str(), cards))
        })
        .collect()
}

#[test]
fn spm_complete_games_exercise_actual_rules_and_replay() {
    let workload: Workload =
        serde_json::from_str(include_str!("../data/decks/spm-set-only.json")).unwrap();
    let decks = decks(&workload);
    let evidence =
        PathBuf::from(std::env::var_os("RUSTMAGE_TDM_EVIDENCE_DIR").unwrap()).join("spm");
    std::fs::create_dir_all(&evidence).unwrap();
    let started = Instant::now();
    let mut completed = 0usize;
    let mut decisions = 0usize;
    let mut colors = 0u8;
    let mut all_mechanics = BTreeSet::new();
    for (pair_index, pair) in workload.pairings.iter().enumerate() {
        let (left_name, left) = &decks[pair.p0.as_str()];
        let (right_name, right) = &decks[pair.p1.as_str()];
        for run in &pair.runs {
            let starting_player = [Player::P0, Player::P1][run.starting_player];
            let config = Config {
                decks: [left.clone(), right.clone()],
                seed: run.seed,
                starting_player,
                deck_rules: DeckRules::Standard,
            };
            let label = format!("{pair_index}-{}-{}", run.seed, run.starting_player);
            let mut game = Game::new(config.clone()).unwrap();
            let mut actions = Vec::new();
            let mut trace = Vec::new();
            let mut seen_stack = BTreeSet::new();
            let mut mechanics = BTreeSet::new();
            let mut rejected = BTreeSet::new();
            let mut pending_attack = None;
            let mut proved = false;
            let mut casts = 0usize;
            while game.outcome().is_none() && trace.len() < 4096 {
                let actor = game.actor().unwrap();
                let observation = game.observe(actor);
                for item in &observation.stack {
                    let Some(cast) = item.cast.filter(|_| !item.is_copy) else {
                        continue;
                    };
                    if item.kind != StackKind::Spell || !seen_stack.insert(item.id) {
                        continue;
                    }
                    casts += 1;
                    if let Some((_, card)) = item.source_identity() {
                        colors |= card.colors();
                    }
                    match cast.method {
                        CastMethod::WebSlinging => mechanics.insert("web-slinging"),
                        CastMethod::Mayhem | CastMethod::GrantedMayhem => {
                            mechanics.insert("mayhem")
                        }
                        CastMethod::LibraryLife => mechanics.insert("library life casting"),
                        _ => false,
                    };
                    if cast.face == CastFace::Back {
                        mechanics.insert("back-face cast");
                    }
                    if cast.origin == Zone::Exile {
                        mechanics.insert("cast from exile");
                    }
                }
                game.legal_actions_into(&mut actions);
                let eligible: Vec<_> = actions
                    .iter()
                    .enumerate()
                    .filter(|(_, action)| match action {
                        Action::Attack { card } | Action::AttackPlaneswalker { card, .. } => {
                            !rejected.contains(card)
                        }
                        _ => true,
                    })
                    .map(|(index, action)| (index, action.clone()))
                    .collect();
                let offered: Vec<_> = eligible.iter().map(|(_, action)| action.clone()).collect();
                let selected = policy::smoke_index_from_observation(&observation, &offered);
                let (index, action) = &eligible[selected];
                assert!(
                    !matches!(action, Action::Concede),
                    "{label} attempted concession"
                );
                match action {
                    Action::Attack { card } | Action::AttackPlaneswalker { card, .. } => {
                        pending_attack = Some(*card);
                    }
                    Action::CancelAttackDeclaration => {
                        rejected.extend(pending_attack.take());
                    }
                    Action::FinishAttackers => {
                        rejected.clear();
                        pending_attack = None;
                    }
                    Action::ChooseEntryRiot { .. } => {
                        mechanics.insert("riot entry choice");
                    }
                    Action::ChooseConvoker { .. } => {
                        mechanics.insert("convoke contribution");
                    }
                    _ => {}
                }
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    if !proved
                        && !matches!(action, Action::Pass | Action::Keep)
                        && actions.len() > 1
                    {
                        proved = prove_branch(&game, actor, &actions, *index);
                    }
                    game.apply(actor, action.clone()).unwrap();
                }));
                if let Err(panic) = result {
                    let failure = serde_json::json!({
                        "config": config,
                        "actions": trace,
                        "actor": actor,
                        "failing_action": action,
                    });
                    let path = evidence.join(format!("{label}-failure.json"));
                    std::fs::write(&path, serde_json::to_vec_pretty(&failure).unwrap()).unwrap();
                    eprintln!(
                        "{label} decision {}: {action:?}, trace {}",
                        trace.len(),
                        path.display()
                    );
                    std::panic::resume_unwind(panic);
                }
                trace.push((actor, action.clone()));
            }
            std::fs::write(
                evidence.join(format!("{label}.json")),
                serde_json::to_vec_pretty(&serde_json::json!({
                    "catalogue": CATALOGUE_VERSION,
                    "config": config,
                    "decks": [left_name, right_name],
                    "actions": trace,
                    "outcome": game.outcome(),
                    "actual_casts": casts,
                    "mechanics": mechanics,
                    "owned_branch": proved,
                }))
                .unwrap(),
            )
            .unwrap();
            assert!(
                game.outcome().is_some() && casts > 0 && proved,
                "{label} incomplete"
            );
            let mut replay = Game::new(config).unwrap();
            for (actor, action) in &trace {
                replay.apply(*actor, action.clone()).unwrap();
            }
            assert_eq!(replay.snapshot(), game.snapshot(), "{label}");
            assert_eq!(replay.outcome(), game.outcome(), "{label}");
            assert_eq!(views(&replay), views(&game), "{label}");
            all_mechanics.extend(mechanics);
            completed += 1;
            decisions += trace.len();
        }
    }
    assert_eq!(colors & 31, 31, "all five colors must be cast");
    println!(
        "{completed} natural Standard games and {decisions} decisions verified in {:.3}s with {all_mechanics:?}",
        started.elapsed().as_secs_f64(),
    );
}
