//! Bounded natural Standard games using only physical Star Trek identities.
use rustmage::*;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::time::Instant;

#[derive(Deserialize)]
struct DeckCard {
    name: String,
    oracle_id: String,
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

fn workload() -> Workload {
    serde_json::from_str(include_str!("../data/decks/trk-set-only.json")).unwrap()
}

fn set_identities() -> BTreeMap<String, String> {
    let manifest: serde_json::Value =
        serde_json::from_str(include_str!("../data/sets/trk.json")).unwrap();
    let identities: BTreeMap<_, _> = manifest["cards"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            (
                row["name"].as_str().unwrap().to_owned(),
                row["oracle_id"].as_str().unwrap().to_owned(),
            )
        })
        .collect();
    assert_eq!(identities.len(), 311, "TRK physical identity manifest");
    identities
}

fn decks(workload: &Workload) -> BTreeMap<&str, (&str, Vec<Card>)> {
    let members = set_identities();
    assert_eq!(workload.decks.len(), 5);
    let fixture_names: BTreeSet<_> = workload
        .decks
        .iter()
        .flat_map(|deck| deck.cards.iter().map(|row| row.name.as_str()))
        .collect();
    for required in [
        "Boarding Action",         // Beam me up
        "D-7 Battle Cruiser",      // Station
        "Test of Priorities",      // Modal dilemma
        "Death Is Irrelevant",     // Assimilate
        "Astute Terraformer",      // Strange new worlds
        "Captain Jonathan Archer", // Federation
        "Support Mission",         // Quest counters
        "Planetary Patrol",        // Landfall
    ] {
        assert!(
            fixture_names.contains(required),
            "TRK workload omits {required}"
        );
    }
    let mut colors = 0u8;
    let decks: BTreeMap<_, _> = workload
        .decks
        .iter()
        .map(|deck| {
            let cards: Vec<_> = deck
                .cards
                .iter()
                .flat_map(|row| {
                    assert_eq!(members.get(&row.name), Some(&row.oracle_id));
                    let card = Card::from_name(&row.name)
                        .unwrap_or_else(|| panic!("missing TRK card {}", row.name));
                    assert!(
                        card.is_standard_legal() && card.is_supported(),
                        "gated {}",
                        row.name
                    );
                    assert_eq!(card.info().oracle_id, Some(row.oracle_id.as_str()));
                    assert!(
                        card.is_basic_land() || row.count <= 4,
                        "too many {}",
                        row.name
                    );
                    colors |= card.colors();
                    std::iter::repeat_n(card, row.count)
                })
                .collect();
            assert_eq!(cards.len(), 60, "{}", deck.name);
            (deck.id.as_str(), (deck.name.as_str(), cards))
        })
        .collect();
    assert_eq!(decks.len(), 5, "distinct TRK deck IDs");
    assert_eq!(colors, 31, "decks must cover all five spell colors");
    decks
}

fn views(game: &Game) -> serde_json::Value {
    let mut observations = [game.observe(Player::P0), game.observe(Player::P1)];
    for observation in &mut observations {
        observation.revision = 0;
    }
    serde_json::to_value(observations).unwrap()
}

fn private_choice(view: &Observation) -> bool {
    view.library_choice.is_some()
        || view.search_choice.is_some()
        || view.search_distribution.is_some()
        || view.search_destination_choice.is_some()
        || view.zone_search.is_some()
        || view.hand_library_search.is_some()
        || view.hand_choice.is_some()
        || view.library_cast_look.is_some()
        || view.target_library.is_some()
        || view
            .effect_choice
            .as_ref()
            .is_some_and(|choice| choice.inspected_cards.is_some())
        || view
            .library_selection
            .as_ref()
            .is_some_and(|choice| !choice.public)
        || view.mana_payment.is_some()
}

fn spent_station_action(view: &Observation, action: &Action) -> bool {
    let (card, ability) = match action {
        Action::AnnounceActivate { card, ability, .. }
        | Action::BeginActivate { card, ability, .. }
        | Action::Activate { card, ability, .. } => (*card, *ability),
        _ => return false,
    };
    if ability != 0 {
        return false;
    }
    let Some(source) = view.objects.iter().find(|object| object.id == card) else {
        return false;
    };
    let threshold = match source.name {
        "Oberth-Class Starship" => 6,
        "Suurok-Class Science Vessel" => 5,
        "Romulan Bird-of-Prey" | "D-7 Battle Cruiser" => 8,
        _ => return false,
    };
    source.counters.get(CounterKind::Charge) >= threshold
}

fn check_privacy(game: &Game, actor: Player) {
    let acting = game.observe(actor);
    let opponent = game.observe(actor.other());
    assert!(
        opponent.actions.is_empty(),
        "nonactor received legal actions"
    );
    let exposed = [
        ("library_choice", opponent.library_choice.is_some()),
        ("search_choice", opponent.search_choice.is_some()),
        (
            "search_distribution",
            opponent.search_distribution.is_some(),
        ),
        (
            "search_destination_choice",
            opponent.search_destination_choice.is_some(),
        ),
        ("zone_search", opponent.zone_search.is_some()),
        (
            "hand_library_search",
            opponent.hand_library_search.is_some(),
        ),
        ("hand_choice", opponent.hand_choice.is_some()),
        ("library_cast_look", opponent.library_cast_look.is_some()),
        ("target_library", opponent.target_library.is_some()),
        ("mana_payment", opponent.mana_payment.is_some()),
    ]
    .into_iter()
    .filter_map(|(name, present)| present.then_some(name))
    .collect::<Vec<_>>();
    assert!(
        exposed.is_empty(),
        "nonactor panels {exposed:?} during {}",
        opponent.decision
    );
    let visible: BTreeSet<_> = opponent
        .objects
        .iter()
        .map(|object| object.reference)
        .collect();
    let hidden_hand: BTreeSet<_> = acting
        .objects
        .iter()
        .filter(|object| object.owner == actor && object.zone == Zone::Hand)
        .map(|object| object.reference)
        .filter(|reference| !visible.contains(reference))
        .collect();
    if let Some(choice) = &opponent.effect_choice {
        assert!(
            choice.inspected_cards.is_none(),
            "nonactor saw inspected cards"
        );
        assert!(
            choice
                .candidates
                .iter()
                .chain(choice.selected.iter().flatten())
                .all(|reference| !hidden_hand.contains(reference)),
            "public effect choice named a hidden hand card"
        );
    }
    if let Some(choice) = &opponent.library_selection {
        if !choice.public {
            assert!(
                choice.remaining.is_empty()
                    && choice.candidates.is_empty()
                    && choice.bottom.is_empty(),
                "nonactor saw private library selection contents"
            );
        }
    }
    if let Some(choice) = &opponent.entry_exile {
        assert!(
            visible.contains(&choice.entrant),
            "nonactor saw hidden entrant"
        );
        assert!(
            choice
                .candidates
                .iter()
                .chain(&choice.selected)
                .all(|reference| visible.contains(reference)),
            "entry exile exposed a hidden candidate"
        );
    }
}

fn prove_owned_branch(game: &Game, actor: Player, actions: &[Action], selected: usize) -> bool {
    let saved = game.snapshot();
    let revision = game.revision();
    let before_views = views(game);
    let action = actions[selected].clone();
    let mut raw = game.clone();
    let mut indexed = game.clone();
    let mut sibling = game.clone();
    assert_eq!(
        raw.apply(actor.other(), action.clone()),
        Err(Error::WrongActor)
    );
    assert_eq!(raw.snapshot(), saved);
    raw.apply(actor, action.clone()).unwrap();
    indexed.apply_index(actor, revision, selected).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(views(&raw), views(&indexed));
    let mut different = false;
    for alternative in actions {
        if *alternative == action || matches!(alternative, Action::Concede) {
            continue;
        }
        sibling.apply(actor, alternative.clone()).unwrap();
        if sibling.snapshot() != raw.snapshot() {
            different = true;
            break;
        }
        sibling.restore(&saved).unwrap();
    }
    if !different {
        return false;
    }
    assert_eq!(game.snapshot(), saved, "child changed parent");
    assert_eq!(views(game), before_views);
    assert_ne!(
        sibling.snapshot(),
        raw.snapshot(),
        "children did not diverge"
    );
    sibling.restore(&saved).unwrap();
    assert_eq!(sibling.snapshot(), saved);
    assert_eq!(views(&sibling), before_views);
    assert_eq!(
        sibling.apply_index(actor, revision, selected),
        Err(Error::StaleRevision)
    );
    sibling
        .apply_index(actor, sibling.revision(), selected)
        .unwrap();
    assert_eq!(sibling.snapshot(), raw.snapshot());
    assert_eq!(views(&sibling), views(&raw));
    true
}

#[derive(Default)]
struct Evidence {
    games: usize,
    decisions: usize,
    casts: usize,
    private_choices: usize,
    owned_branches: usize,
    spell_colors: u8,
    mechanics: BTreeSet<&'static str>,
}

fn record_mechanics(
    before: &Observation,
    after: &Observation,
    action: &Action,
    seen_spells: &mut BTreeSet<u64>,
    evidence: &mut Evidence,
) {
    match action {
        Action::ChooseMode { .. } => {
            evidence.mechanics.insert("modal choice / dilemma");
        }
        Action::AnnounceBeamMeUp { .. } | Action::PayBeamMeUpReturn { .. } => {
            evidence.mechanics.insert("beam me up");
        }
        _ => {}
    }
    for item in &after.stack {
        if item.kind != StackKind::Spell || item.is_copy || !seen_spells.insert(item.id) {
            continue;
        }
        evidence.casts += 1;
        if let Some((_, card)) = item.source_identity() {
            evidence.spell_colors |= card.colors();
        }
        if item
            .cast
            .is_some_and(|cast| cast.method == CastMethod::BeamMeUp)
        {
            evidence.mechanics.insert("beam me up");
        }
    }
    for object in &after.objects {
        if object.zone != Zone::Battlefield {
            continue;
        }
        let previous = before
            .objects
            .iter()
            .find(|prior| prior.reference == object.reference);
        if object.counters.get(CounterKind::Charge)
            > previous.map_or(0, |prior| prior.counters.get(CounterKind::Charge))
        {
            evidence.mechanics.insert("station / charge counters");
        }
        if object.counters.get(CounterKind::Quest)
            > previous.map_or(0, |prior| prior.counters.get(CounterKind::Quest))
        {
            evidence.mechanics.insert("quest counters");
        }
        if object.token.is_some() && previous.is_none() {
            evidence.mechanics.insert("token creation");
        }
    }
}

fn run_pair(pair_index: usize) {
    let workload = workload();
    let decks = decks(&workload);
    assert_eq!(workload.pairings.len(), 5);
    let pair = &workload.pairings[pair_index];
    assert_eq!(
        pair.runs.len(),
        4,
        "two fixed seeds and both starting seats"
    );
    for chunk in pair.runs.chunks_exact(2) {
        assert_eq!(chunk[0].seed, chunk[1].seed);
        assert_eq!((chunk[0].starting_player, chunk[1].starting_player), (0, 1));
    }
    let (left_name, left) = &decks[pair.p0.as_str()];
    let (right_name, right) = &decks[pair.p1.as_str()];
    let evidence_dir =
        PathBuf::from(std::env::var_os("RUSTMAGE_TDM_EVIDENCE_DIR").unwrap()).join("trk");
    std::fs::create_dir_all(&evidence_dir).unwrap();
    let started = Instant::now();
    let mut evidence = Evidence::default();
    for run in &pair.runs {
        let config = Config {
            decks: [left.clone(), right.clone()],
            seed: run.seed,
            starting_player: [Player::P0, Player::P1][run.starting_player],
            deck_rules: DeckRules::Standard,
        };
        let label = format!("{pair_index}-{}-{}", run.seed, run.starting_player);
        let mut game = Game::new(config.clone()).unwrap();
        for viewer in [Player::P0, Player::P1] {
            let opening = game.observe(viewer);
            assert!(
                opening
                    .objects
                    .iter()
                    .all(|object| { object.owner == viewer || object.zone != Zone::Hand }),
                "opponent opening hand leaked to {viewer:?}"
            );
        }
        let mut journal = Vec::new();
        let mut actions = Vec::new();
        let mut seen_spells = BTreeSet::new();
        let mut rejected_attacks = BTreeSet::new();
        let mut pending_attack = None;
        let mut proved = false;
        while game.outcome().is_none() && journal.len() < 4096 {
            let actor = game.actor().unwrap();
            let before = game.observe(actor);
            let sensitive = private_choice(&before);
            check_privacy(&game, actor);
            game.legal_actions_into(&mut actions);
            let eligible: Vec<_> = actions
                .iter()
                .enumerate()
                .filter(|(_, action)| match action {
                    Action::Attack { card } | Action::AttackPlaneswalker { card, .. } => {
                        !rejected_attacks.contains(card)
                    }
                    _ => !spent_station_action(&before, action),
                })
                .map(|(index, action)| (index, action.clone()))
                .collect();
            let offered: Vec<_> = eligible.iter().map(|(_, action)| action.clone()).collect();
            let chosen = policy::smoke_index_from_observation(&before, &offered);
            let (index, action) = &eligible[chosen];
            assert!(!matches!(action, Action::Concede), "{label} conceded");
            match action {
                Action::Attack { card } | Action::AttackPlaneswalker { card, .. } => {
                    pending_attack = Some(*card);
                }
                Action::CancelAttackDeclaration => {
                    rejected_attacks.extend(pending_attack.take());
                }
                Action::FinishAttackers => {
                    rejected_attacks.clear();
                    pending_attack = None;
                }
                _ => {}
            }
            if sensitive || journal.len().is_multiple_of(64) {
                let saved = game.snapshot();
                let revision = game.revision();
                assert_eq!(
                    game.apply(actor, Action::ChooseMode { mode: u16::MAX }),
                    Err(Error::IllegalAction)
                );
                assert_eq!(game.snapshot(), saved, "illegal action partially committed");
                assert_eq!(game.revision(), revision);
                evidence.private_choices += usize::from(sensitive);
            }
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                if !proved && !matches!(action, Action::Pass | Action::Keep) && actions.len() > 1 {
                    proved = prove_owned_branch(&game, actor, &actions, *index);
                }
                game.apply(actor, action.clone()).unwrap();
            }));
            if let Err(panic) = result {
                let failure = serde_json::json!({
                    "config": config,
                    "actions": journal,
                    "actor": actor,
                    "failing_action": action,
                });
                let path = evidence_dir.join(format!("{label}-failure.json"));
                std::fs::write(&path, serde_json::to_vec_pretty(&failure).unwrap()).unwrap();
                eprintln!(
                    "{label} decision {}: {action:?}; trace {}",
                    journal.len(),
                    path.display()
                );
                std::panic::resume_unwind(panic);
            }
            let after = game.observe(game.actor().unwrap_or(actor));
            record_mechanics(&before, &after, action, &mut seen_spells, &mut evidence);
            journal.push((actor, action.clone()));
        }
        if game.outcome().is_none() {
            let path = evidence_dir.join(format!("{label}-incomplete.json"));
            std::fs::write(
                &path,
                serde_json::to_vec_pretty(&serde_json::json!({
                    "config": config,
                    "actions": journal,
                    "observation": game.observe(game.actor().unwrap()),
                }))
                .unwrap(),
            )
            .unwrap();
            panic!("{label} exceeded 4096 decisions; trace {}", path.display());
        }
        assert!(proved, "{label} never exercised an owned branch");
        assert!(!seen_spells.is_empty(), "{label} cast no spells");
        evidence.owned_branches += 1;
        evidence.games += 1;
        evidence.decisions += journal.len();
        let mut replay = Game::new(config.clone()).unwrap();
        for (actor, action) in &journal {
            replay.apply(*actor, action.clone()).unwrap();
        }
        assert_eq!(
            replay.snapshot(),
            game.snapshot(),
            "{label} deterministic snapshot"
        );
        assert_eq!(
            replay.outcome(),
            game.outcome(),
            "{label} deterministic outcome"
        );
        assert_eq!(
            views(&replay),
            views(&game),
            "{label} deterministic observations"
        );
        std::fs::write(
            evidence_dir.join(format!("{label}.json")),
            serde_json::to_vec_pretty(&serde_json::json!({
                "catalogue": CATALOGUE_VERSION,
                "config": config,
                "decks": [left_name, right_name],
                "actions": journal,
                "outcome": game.outcome(),
                "owned_branch": proved,
            }))
            .unwrap(),
        )
        .unwrap();
    }
    assert_eq!(evidence.games, 4);
    assert!(evidence.casts > 0);
    assert!(
        evidence.private_choices > 0,
        "{pair_index} missed private choices"
    );
    let required_events: &[&str] = match pair_index {
        0 => &[
            "beam me up",
            "modal choice / dilemma",
            "station / charge counters",
        ],
        4 => &["quest counters", "token creation"],
        _ => &[],
    };
    for event in required_events {
        assert!(
            evidence.mechanics.contains(event),
            "{pair_index} did not exercise {event}"
        );
    }
    let seconds = started.elapsed().as_secs_f64();
    let summary = serde_json::json!({
        "catalogue": CATALOGUE_VERSION,
        "pairing": [pair.p0, pair.p1],
        "games": evidence.games,
        "decisions": evidence.decisions,
        "casts": evidence.casts,
        "private_choices": evidence.private_choices,
        "owned_branches": evidence.owned_branches,
        "spell_colors": evidence.spell_colors,
        "mechanics": evidence.mechanics,
        "acceptance_seconds": seconds,
        "acceptance_games_per_second": evidence.games as f64 / seconds,
        "acceptance_decisions_per_second": evidence.decisions as f64 / seconds,
        "throughput_scope": "includes game simulation, privacy checks, owned branches, replay, and evidence writes",
    });
    std::fs::write(
        evidence_dir.join(format!("summary-{pair_index}.json")),
        serde_json::to_vec_pretty(&summary).unwrap(),
    )
    .unwrap();
    println!("TRK_COMPLETE_GAME_PAIR {}", summary);
}

#[test]
fn trk_complete_games_pair_0_federation_science() {
    run_pair(0);
}
#[test]
fn trk_complete_games_pair_1_science_collective() {
    run_pair(1);
}
#[test]
fn trk_complete_games_pair_2_collective_klingon() {
    run_pair(2);
}
#[test]
fn trk_complete_games_pair_3_klingon_exploration() {
    run_pair(3);
}
#[test]
fn trk_complete_games_pair_4_exploration_federation() {
    run_pair(4);
}

// This full-set admission gate runs on the assembled 311-card catalogue.
#[test]
fn zz_trk_all_311_physical_identities_are_supported() {
    for (name, oracle_id) in set_identities() {
        let card = Card::from_name(&name).unwrap_or_else(|| panic!("missing {name}"));
        assert_eq!(card.info().oracle_id, Some(oracle_id.as_str()));
        assert!(
            card.is_standard_legal() && card.is_supported(),
            "gated {name}"
        );
    }
}
