//! Natural terminal games with only admitted SPM cards and the shared smoke policy.
use rustmage::*;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;
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
    #[serde(default)]
    prefer_back_face: bool,
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
        // Each candidate starts from the same allocator state; restoring a
        // prior candidate would intentionally allocate a newer attempt ID.
        sibling = game.clone();
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
    let mut restored_raw = restored.clone();
    restored_raw.apply(actor, action.clone()).unwrap();
    restored
        .apply_index(actor, restored.revision(), index)
        .unwrap();
    assert_eq!(restored.snapshot(), restored_raw.snapshot());
    assert_eq!(views(&restored), views(&restored_raw));
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
            if let Some(back) = card.back_face() {
                assert!(back.is_supported(), "gated back face of {name}");
            }
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

// Workload preferences inspect only this actor's current public information.
// The estimate chooses which attempt to try; concrete payment remains an action.
fn current_mana_budget(view: &Observation) -> u64 {
    let actor = view.actor.unwrap();
    view.players[actor.index()]
        .mana
        .iter()
        .map(|&n| u64::from(n))
        .sum::<u64>()
        + view
            .objects
            .iter()
            .filter(|object| {
                object.controller == actor
                    && object.zone == Zone::Battlefield
                    && object.card.is_land()
                    && !object.tapped
            })
            .count() as u64
}

fn caller_allows(view: &Observation, action: &Action) -> bool {
    let actor = view.actor.unwrap();
    if view.decision == "priority" {
        let budget = current_mana_budget(view);
        if let Action::AnnounceCast { card } = action
            && view
                .objects
                .iter()
                .find(|object| object.id == *card)
                .is_some_and(|object| object.card.fixed_mana_value() > budget)
        {
            return false;
        }
        if matches!(
            action,
            Action::AnnounceActivate { .. }
                | Action::BeginActivate { .. }
                | Action::Activate { .. }
        ) && (view.active != actor || !matches!(view.step, Step::Main1 | Step::Main2))
        {
            return false;
        }
        if let Action::AnnounceWebSlinging { cost, .. } = action {
            let amount =
                u64::from(cost.generic) + cost.colored.iter().map(|&n| u64::from(n)).sum::<u64>();
            if amount > budget
                || !view.objects.iter().any(|object| {
                    object.controller == actor
                        && object.zone == Zone::Battlefield
                        && object.is_creature()
                        && object.tapped
                })
            {
                return false;
            }
        }
    }
    if let Some(item) = &view.announcing
        && let StackKind::ActivatedAbility { ability } = item.kind
        && let Some((source, definition)) = item.source_identity()
        && definition.activations()[usize::from(ability)].cost.tap
        && matches!(action,
            Action::TapMana { card } | Action::ActivateMana { card, .. }
                | Action::TapGrantedMana { card, .. } | Action::AnnounceMana { card, .. }
                if *card == source.card)
    {
        // Preserve the current activation's supplied tap cost. Tapping its
        // source for mana would make the final concrete payment illegal.
        return false;
    }
    true
}

fn decision(view: &Observation, actions: &[Action], seek_back_face: bool) -> usize {
    let smoke = policy::smoke_index_from_observation(view, actions);
    if matches!(actions[smoke], Action::PlayLand { .. })
        || (view.attempt.is_none()
            && view.decision == "priority"
            && view.last_attempt_outcome.is_some_and(|outcome| {
                outcome.turn == u64::from(view.turn) && outcome.outcome == AttemptOutcome::Failed
            }))
    {
        return smoke;
    }
    // Exercise an offered contribution after the current pool already covers
    // the mana cost; no additional producer or contributor is predicted.
    if view.mana_payment.is_some()
        && let Some(index) = actions
            .iter()
            .position(|action| matches!(action, Action::BeginConvoke))
    {
        return index;
    }
    if view.decision != "priority" {
        return smoke;
    }
    let budget = current_mana_budget(view);
    if let Some(index) = actions.iter().position(|action| match action {
        Action::AnnounceBack { card } if seek_back_face => view
            .objects
            .iter()
            .find(|object| object.id == *card)
            .and_then(|object| object.card.back_face())
            .is_some_and(|back| back.fixed_mana_value() <= budget),
        Action::AnnounceLibraryLife { .. } | Action::AnnounceWebSlinging { .. } => true,
        _ => false,
    }) {
        return index;
    }
    if matches!(
        actions[smoke],
        Action::AnnounceWebSlinging { .. }
            | Action::AnnounceMayhem { .. }
            | Action::AnnounceGrantedMayhem { .. }
            | Action::AnnounceLibraryLife { .. }
            | Action::AnnounceExile { .. }
    ) {
        return smoke;
    }
    actions
        .iter()
        .enumerate()
        .filter_map(|(index, action)| {
            let (Action::AnnounceCast { card } | Action::Cast { card, .. }) = action else {
                return None;
            };
            let object = view.objects.iter().find(|object| object.id == *card)?;
            let value = object.card.fixed_mana_value();
            (value <= budget).then_some((index, value))
        })
        .max_by_key(|&(_, value)| value)
        .map_or(smoke, |(index, _)| index)
}

fn verify_pairings(
    batch: &str,
    pairings: Range<usize>,
    required_colors: u8,
    required_mechanics: &[&str],
) {
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
    assert!(pairings.end <= workload.pairings.len());
    for pair_index in pairings {
        let pair = &workload.pairings[pair_index];
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
            let mut seek_back_face = run.prefer_back_face;
            while game.outcome().is_none() && trace.len() < 4096 {
                let actor = game.actor().unwrap();
                let observation = game.observe(actor);
                for object in &observation.objects {
                    if object.zone != Zone::Battlefield {
                        continue;
                    }
                    if object.modified {
                        mechanics.insert("modified creature");
                    }
                    if object.face.as_ref().is_some_and(|face| face.transforms > 0) {
                        mechanics.insert("transform");
                    }
                }
                for item in &observation.stack {
                    if item.kind == StackKind::Spell && item.is_copy {
                        mechanics.insert("spell copy");
                    }
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
                        seek_back_face = false;
                    }
                    if cast.origin == Zone::Exile {
                        mechanics.insert("cast from exile");
                    }
                }
                game.legal_actions_into(&mut actions);
                let eligible: Vec<_> = actions
                    .iter()
                    .enumerate()
                    .filter(|(_, action)| caller_allows(&observation, action))
                    .filter(|(_, action)| match action {
                        Action::Attack { card } | Action::AttackPlaneswalker { card, .. } => {
                            !rejected.contains(card)
                        }
                        // In the designated modal-face replay, hold the front
                        // until its alternative can be announced with normal mana.
                        Action::AnnounceCast { card } | Action::Cast { card, .. }
                            if seek_back_face =>
                        {
                            !observation.objects.iter().any(|object| {
                                object.id == *card && object.card.back_face().is_some()
                            })
                        }
                        _ => true,
                    })
                    .map(|(index, action)| (index, action.clone()))
                    .collect();
                let offered: Vec<_> = eligible.iter().map(|(_, action)| action.clone()).collect();
                let selected = decision(&observation, &offered, seek_back_face);
                let (index, action) = &eligible[selected];
                if matches!(action, Action::Concede) {
                    std::fs::write(
                        evidence.join(format!("{label}-stalled.json")),
                        serde_json::to_vec_pretty(&serde_json::json!({
                            "action_contract": ACTION_CONTRACT,
                            "observation_schema": OBSERVATION_SCHEMA,
                            "primitive_decision_limit": 4096,
                            "config": config, "actions": trace,
                            "observation": observation,
                        }))
                        .unwrap(),
                    )
                    .unwrap();
                    panic!("{label} attempted concession");
                }
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
                    Action::FinishCrew => {
                        mechanics.insert("crew");
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
                        "observation": observation,
                        "action_contract": ACTION_CONTRACT,
                        "observation_schema": OBSERVATION_SCHEMA,
                        "primitive_decision_limit": 4096,
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
            if game.outcome().is_none() {
                let actor = game.actor().unwrap();
                std::fs::write(
                    evidence.join(format!("{label}-limit.json")),
                    serde_json::to_vec_pretty(&serde_json::json!({
                        "action_contract": ACTION_CONTRACT,
                        "observation_schema": OBSERVATION_SCHEMA,
                        "primitive_decision_limit": 4096,
                        "config": config, "actions": trace,
                        "observation": game.observe(actor),
                    }))
                    .unwrap(),
                )
                .unwrap();
            }
            std::fs::write(
                evidence.join(format!("{label}.json")),
                serde_json::to_vec_pretty(&serde_json::json!({
                    "catalogue": CATALOGUE_VERSION,
                    "action_contract": ACTION_CONTRACT,
                    "observation_schema": OBSERVATION_SCHEMA,
                    "primitive_decision_limit": 4096,
                    "config": config,
                    "decks": [left_name, right_name],
                    "actions": trace,
                    "outcome": game.outcome(),
                    "actual_casts": casts,
                    "mechanics": mechanics,
                    "owned_branch": proved,
                    "prefer_back_face": run.prefer_back_face,
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
    assert_eq!(colors & required_colors, required_colors, "{batch} colors");
    for mechanic in required_mechanics {
        assert!(
            all_mechanics.contains(mechanic),
            "{batch} missed {mechanic}"
        );
    }
    let seconds = started.elapsed().as_secs_f64();
    let summary = serde_json::json!({
        "catalogue": CATALOGUE_VERSION,
        "completed_games": completed,
        "decisions": decisions,
        "spell_colors": colors,
        "observed_mechanics": all_mechanics,
        "verification_seconds": seconds,
        "scope": "Natural set-only Standard games with deterministic replay, owned branches, and evidence writes",
    });
    std::fs::write(
        evidence.join(format!("summary-{batch}.json")),
        serde_json::to_vec_pretty(&summary).unwrap(),
    )
    .unwrap();
}

#[test]
fn spm_core_matchups_cover_all_colors_and_replay() {
    verify_pairings(
        "core",
        0..4,
        31,
        &[
            "web-slinging",
            "mayhem",
            "modified creature",
            "riot entry choice",
            "transform",
            "cast from exile",
            "spell copy",
            "crew",
        ],
    );
}

#[test]
fn spm_library_and_convoke_matchups_replay() {
    verify_pairings(
        "library-convoke",
        4..5,
        0,
        &["convoke contribution", "library life casting"],
    );
}

#[test]
fn spm_library_and_web_slinging_matchups_replay() {
    verify_pairings("library-web", 5..6, 0, &[]);
}

#[test]
fn spm_modal_back_face_cast_replays() {
    verify_pairings("modal-back", 6..7, 0, &["back-face cast"]);
}
