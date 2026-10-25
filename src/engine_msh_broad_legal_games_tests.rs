//! Fixed whole-pool game groups; policy, seeds and decision/time caps stay unchanged.
use super::msh_thirteenth_combined_tests::{
    hidden_information_contract, natural_event_label, owned_checked_step, views,
};
use super::*;
use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;
#[derive(Deserialize)]
struct Pool {
    schema: u32,
    design_sha256: String,
    decision_cap: usize,
    decks: Vec<Deck>,
    games: Vec<Match>,
}
#[derive(Deserialize)]
struct Deck {
    id: String,
    cards: Vec<(String, usize)>,
    configuration_sha256: String,
}
#[derive(Deserialize)]
struct Match {
    phase: String,
    decks: [String; 2],
    seed: u64,
    starting_player: Player,
}
const BROAD: &str = include_str!("../data/assurance/msh-broad-legal-games.json");
const FINAL: &str = include_str!("../data/assurance/msh-final281-legal-games.json");
macro_rules! fixed_group {
    ($name:ident, $pool:expr, $decks:expr, $games:expr, $start:expr, $variable:literal) => {
        #[test]
        fn $name() {
            run_pool(
                serde_json::from_str($pool).unwrap(),
                $decks,
                $games,
                $start..$start + 4,
                Some($start / 4),
                $variable,
            );
        }
    };
}
fixed_group!(
    msh_broad_132_group_00,
    BROAD,
    17,
    132,
    0,
    "RUSTMAGE_RECORD_MSH_BROAD_GAMES"
);
fixed_group!(
    msh_broad_132_group_01,
    BROAD,
    17,
    132,
    4,
    "RUSTMAGE_RECORD_MSH_BROAD_GAMES"
);
fixed_group!(
    msh_broad_132_group_02,
    BROAD,
    17,
    132,
    8,
    "RUSTMAGE_RECORD_MSH_BROAD_GAMES"
);
fixed_group!(
    msh_broad_132_group_03,
    BROAD,
    17,
    132,
    12,
    "RUSTMAGE_RECORD_MSH_BROAD_GAMES"
);
fixed_group!(
    msh_broad_132_group_04,
    BROAD,
    17,
    132,
    16,
    "RUSTMAGE_RECORD_MSH_BROAD_GAMES"
);
fixed_group!(
    msh_broad_132_group_05,
    BROAD,
    17,
    132,
    20,
    "RUSTMAGE_RECORD_MSH_BROAD_GAMES"
);
fixed_group!(
    msh_broad_132_group_06,
    BROAD,
    17,
    132,
    24,
    "RUSTMAGE_RECORD_MSH_BROAD_GAMES"
);
fixed_group!(
    msh_broad_132_group_07,
    BROAD,
    17,
    132,
    28,
    "RUSTMAGE_RECORD_MSH_BROAD_GAMES"
);
fixed_group!(
    msh_broad_132_group_08,
    BROAD,
    17,
    132,
    32,
    "RUSTMAGE_RECORD_MSH_BROAD_GAMES"
);
fixed_group!(
    msh_broad_132_group_09,
    BROAD,
    17,
    132,
    36,
    "RUSTMAGE_RECORD_MSH_BROAD_GAMES"
);
fixed_group!(
    msh_broad_132_group_10,
    BROAD,
    17,
    132,
    40,
    "RUSTMAGE_RECORD_MSH_BROAD_GAMES"
);
fixed_group!(
    msh_broad_132_group_11,
    BROAD,
    17,
    132,
    44,
    "RUSTMAGE_RECORD_MSH_BROAD_GAMES"
);
fixed_group!(
    msh_broad_132_group_12,
    BROAD,
    17,
    132,
    48,
    "RUSTMAGE_RECORD_MSH_BROAD_GAMES"
);
fixed_group!(
    msh_broad_132_group_13,
    BROAD,
    17,
    132,
    52,
    "RUSTMAGE_RECORD_MSH_BROAD_GAMES"
);
fixed_group!(
    msh_broad_132_group_14,
    BROAD,
    17,
    132,
    56,
    "RUSTMAGE_RECORD_MSH_BROAD_GAMES"
);
fixed_group!(
    msh_broad_132_group_15,
    BROAD,
    17,
    132,
    60,
    "RUSTMAGE_RECORD_MSH_BROAD_GAMES"
);
fixed_group!(
    msh_broad_132_group_16,
    BROAD,
    17,
    132,
    64,
    "RUSTMAGE_RECORD_MSH_BROAD_GAMES"
);
fixed_group!(
    msh_broad_132_group_17,
    BROAD,
    17,
    132,
    68,
    "RUSTMAGE_RECORD_MSH_BROAD_GAMES"
);
fixed_group!(
    msh_broad_132_group_18,
    BROAD,
    17,
    132,
    72,
    "RUSTMAGE_RECORD_MSH_BROAD_GAMES"
);
fixed_group!(
    msh_broad_132_group_19,
    BROAD,
    17,
    132,
    76,
    "RUSTMAGE_RECORD_MSH_BROAD_GAMES"
);
fixed_group!(
    msh_broad_132_group_20,
    BROAD,
    17,
    132,
    80,
    "RUSTMAGE_RECORD_MSH_BROAD_GAMES"
);
fixed_group!(
    msh_broad_132_group_21,
    BROAD,
    17,
    132,
    84,
    "RUSTMAGE_RECORD_MSH_BROAD_GAMES"
);
fixed_group!(
    msh_broad_132_group_22,
    BROAD,
    17,
    132,
    88,
    "RUSTMAGE_RECORD_MSH_BROAD_GAMES"
);
fixed_group!(
    msh_broad_132_group_23,
    BROAD,
    17,
    132,
    92,
    "RUSTMAGE_RECORD_MSH_BROAD_GAMES"
);
fixed_group!(
    msh_broad_132_group_24,
    BROAD,
    17,
    132,
    96,
    "RUSTMAGE_RECORD_MSH_BROAD_GAMES"
);
fixed_group!(
    msh_broad_132_group_25,
    BROAD,
    17,
    132,
    100,
    "RUSTMAGE_RECORD_MSH_BROAD_GAMES"
);
fixed_group!(
    msh_broad_132_group_26,
    BROAD,
    17,
    132,
    104,
    "RUSTMAGE_RECORD_MSH_BROAD_GAMES"
);
fixed_group!(
    msh_broad_132_group_27,
    BROAD,
    17,
    132,
    108,
    "RUSTMAGE_RECORD_MSH_BROAD_GAMES"
);
fixed_group!(
    msh_broad_132_group_28,
    BROAD,
    17,
    132,
    112,
    "RUSTMAGE_RECORD_MSH_BROAD_GAMES"
);
fixed_group!(
    msh_broad_132_group_29,
    BROAD,
    17,
    132,
    116,
    "RUSTMAGE_RECORD_MSH_BROAD_GAMES"
);
fixed_group!(
    msh_broad_132_group_30,
    BROAD,
    17,
    132,
    120,
    "RUSTMAGE_RECORD_MSH_BROAD_GAMES"
);
fixed_group!(
    msh_broad_132_group_31,
    BROAD,
    17,
    132,
    124,
    "RUSTMAGE_RECORD_MSH_BROAD_GAMES"
);
fixed_group!(
    msh_broad_132_group_32,
    BROAD,
    17,
    132,
    128,
    "RUSTMAGE_RECORD_MSH_BROAD_GAMES"
);
fixed_group!(
    msh_final281_group_00,
    FINAL,
    20,
    120,
    0,
    "RUSTMAGE_RECORD_MSH_FINAL281_GAMES"
);
fixed_group!(
    msh_final281_group_01,
    FINAL,
    20,
    120,
    4,
    "RUSTMAGE_RECORD_MSH_FINAL281_GAMES"
);
fixed_group!(
    msh_final281_group_02,
    FINAL,
    20,
    120,
    8,
    "RUSTMAGE_RECORD_MSH_FINAL281_GAMES"
);
fixed_group!(
    msh_final281_group_03,
    FINAL,
    20,
    120,
    12,
    "RUSTMAGE_RECORD_MSH_FINAL281_GAMES"
);
fixed_group!(
    msh_final281_group_04,
    FINAL,
    20,
    120,
    16,
    "RUSTMAGE_RECORD_MSH_FINAL281_GAMES"
);
fixed_group!(
    msh_final281_group_05,
    FINAL,
    20,
    120,
    20,
    "RUSTMAGE_RECORD_MSH_FINAL281_GAMES"
);
fixed_group!(
    msh_final281_group_06,
    FINAL,
    20,
    120,
    24,
    "RUSTMAGE_RECORD_MSH_FINAL281_GAMES"
);
fixed_group!(
    msh_final281_group_07,
    FINAL,
    20,
    120,
    28,
    "RUSTMAGE_RECORD_MSH_FINAL281_GAMES"
);
fixed_group!(
    msh_final281_group_08,
    FINAL,
    20,
    120,
    32,
    "RUSTMAGE_RECORD_MSH_FINAL281_GAMES"
);
fixed_group!(
    msh_final281_group_09,
    FINAL,
    20,
    120,
    36,
    "RUSTMAGE_RECORD_MSH_FINAL281_GAMES"
);
fixed_group!(
    msh_final281_group_10,
    FINAL,
    20,
    120,
    40,
    "RUSTMAGE_RECORD_MSH_FINAL281_GAMES"
);
fixed_group!(
    msh_final281_group_11,
    FINAL,
    20,
    120,
    44,
    "RUSTMAGE_RECORD_MSH_FINAL281_GAMES"
);
fixed_group!(
    msh_final281_group_12,
    FINAL,
    20,
    120,
    48,
    "RUSTMAGE_RECORD_MSH_FINAL281_GAMES"
);
fixed_group!(
    msh_final281_group_13,
    FINAL,
    20,
    120,
    52,
    "RUSTMAGE_RECORD_MSH_FINAL281_GAMES"
);
fixed_group!(
    msh_final281_group_14,
    FINAL,
    20,
    120,
    56,
    "RUSTMAGE_RECORD_MSH_FINAL281_GAMES"
);
fixed_group!(
    msh_final281_group_15,
    FINAL,
    20,
    120,
    60,
    "RUSTMAGE_RECORD_MSH_FINAL281_GAMES"
);
fixed_group!(
    msh_final281_group_16,
    FINAL,
    20,
    120,
    64,
    "RUSTMAGE_RECORD_MSH_FINAL281_GAMES"
);
fixed_group!(
    msh_final281_group_17,
    FINAL,
    20,
    120,
    68,
    "RUSTMAGE_RECORD_MSH_FINAL281_GAMES"
);
fixed_group!(
    msh_final281_group_18,
    FINAL,
    20,
    120,
    72,
    "RUSTMAGE_RECORD_MSH_FINAL281_GAMES"
);
fixed_group!(
    msh_final281_group_19,
    FINAL,
    20,
    120,
    76,
    "RUSTMAGE_RECORD_MSH_FINAL281_GAMES"
);
fixed_group!(
    msh_final281_group_20,
    FINAL,
    20,
    120,
    80,
    "RUSTMAGE_RECORD_MSH_FINAL281_GAMES"
);
fixed_group!(
    msh_final281_group_21,
    FINAL,
    20,
    120,
    84,
    "RUSTMAGE_RECORD_MSH_FINAL281_GAMES"
);
fixed_group!(
    msh_final281_group_22,
    FINAL,
    20,
    120,
    88,
    "RUSTMAGE_RECORD_MSH_FINAL281_GAMES"
);
fixed_group!(
    msh_final281_group_23,
    FINAL,
    20,
    120,
    92,
    "RUSTMAGE_RECORD_MSH_FINAL281_GAMES"
);
fixed_group!(
    msh_final281_group_24,
    FINAL,
    20,
    120,
    96,
    "RUSTMAGE_RECORD_MSH_FINAL281_GAMES"
);
fixed_group!(
    msh_final281_group_25,
    FINAL,
    20,
    120,
    100,
    "RUSTMAGE_RECORD_MSH_FINAL281_GAMES"
);
fixed_group!(
    msh_final281_group_26,
    FINAL,
    20,
    120,
    104,
    "RUSTMAGE_RECORD_MSH_FINAL281_GAMES"
);
fixed_group!(
    msh_final281_group_27,
    FINAL,
    20,
    120,
    108,
    "RUSTMAGE_RECORD_MSH_FINAL281_GAMES"
);
fixed_group!(
    msh_final281_group_28,
    FINAL,
    20,
    120,
    112,
    "RUSTMAGE_RECORD_MSH_FINAL281_GAMES"
);
fixed_group!(
    msh_final281_group_29,
    FINAL,
    20,
    120,
    116,
    "RUSTMAGE_RECORD_MSH_FINAL281_GAMES"
);
#[test]
fn msh_fourteenth_32_fixed_natural_standard_set_only_games_owned_privacy() {
    let frozen: Pool = serde_json::from_str(include_str!(
        "../data/assurance/msh-fourteenth-legal-games.json"
    ))
    .unwrap();
    run_pool(
        frozen,
        1,
        32,
        0..32,
        None,
        "RUSTMAGE_RECORD_MSH_FOURTEENTH_GAMES",
    );
}
#[test]
fn msh_final281_deck_union_is_exact_frozen_membership_and_standard60() {
    let membership: serde_json::Value =
        serde_json::from_str(include_str!("../data/sets/msh.json")).unwrap();
    let members: BTreeSet<_> = membership["cards"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["name"].as_str().unwrap())
        .collect();
    assert_eq!(members.len(), 281);
    let mut names = BTreeSet::new();
    for source in [BROAD, FINAL] {
        let pool: Pool = serde_json::from_str(source).unwrap();
        for deck in pool.decks {
            let mut cards = Vec::new();
            for (name, count) in deck.cards {
                assert!(members.contains(name.as_str()));
                let card = Card::from_name(&name).unwrap();
                assert!(card.is_supported() && card.is_standard_legal(), "{name}");
                names.insert(name);
                cards.extend(std::iter::repeat_n(card, count));
            }
            assert_eq!(cards.len(), 60);
            Game::new(Config {
                decks: [cards.clone(), cards],
                seed: 20261025,
                starting_player: Player::P0,
                deck_rules: DeckRules::Standard,
            })
            .unwrap();
        }
    }
    assert_eq!(
        names.iter().map(String::as_str).collect::<BTreeSet<_>>(),
        members
    );
}
fn output_path(variable: &str, group: Option<usize>) -> Option<std::path::PathBuf> {
    let base = std::path::PathBuf::from(std::env::var_os(variable)?);
    assert!(
        base.starts_with(std::path::PathBuf::from(std::env::var("HOME").unwrap()).join(".cache"))
    );
    Some(group.map_or(base.clone(), |index| {
        base.with_extension(format!("group{index:02}.json"))
    }))
}
fn write_record(path: &std::path::Path, value: &serde_json::Value) {
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .unwrap();
    serde_json::to_writer_pretty(&mut file, value).unwrap();
    file.write_all(b"\n").unwrap();
}
fn run_pool(
    frozen: Pool,
    expected_decks: usize,
    expected_games: usize,
    range: Range<usize>,
    group: Option<usize>,
    record_variable: &str,
) {
    assert_eq!(frozen.schema, 1);
    assert_eq!(frozen.decision_cap, 30000);
    assert_eq!(frozen.decks.len(), expected_decks);
    assert_eq!(frozen.games.len(), expected_games);
    assert!(range.start < range.end && range.end <= expected_games);
    let membership: serde_json::Value =
        serde_json::from_str(include_str!("../data/sets/msh.json")).unwrap();
    let members: BTreeSet<_> = membership["cards"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["name"].as_str().unwrap())
        .collect();
    let mut decks = BTreeMap::new();
    let mut hashes = BTreeMap::new();
    for deck in &frozen.decks {
        let mut cards = Vec::new();
        for (name, count) in &deck.cards {
            assert!(
                members.contains(name.as_str()),
                "outside frozen MSH membership:{name}"
            );
            let card = Card::from_name(name).unwrap();
            assert!(card.is_supported() && card.is_standard_legal(), "{name}");
            cards.extend(std::iter::repeat_n(card, *count));
        }
        assert_eq!(cards.len(), 60);
        assert!(decks.insert(deck.id.clone(), cards).is_none());
        hashes.insert(deck.id.clone(), deck.configuration_sha256.clone());
    }
    let start = std::time::Instant::now();
    let mut records = Vec::new();
    let mut decisions = 0usize;
    let mut paid = BTreeMap::<String, usize>::new();
    let mut paid_positive = BTreeMap::<String, usize>::new();
    let mut nonterminal = Vec::new();
    let output = output_path(record_variable, group);
    let full_ledger = output.is_some();
    let only = std::env::var("RUSTMAGE_MSH_BROAD_CONFIG")
        .ok()
        .map(|s| s.parse::<usize>().unwrap());
    assert!(
        only.is_none_or(|i| range.contains(&i)),
        "diagnostic index must belong to the exact selected test group"
    );
    let mut progress = if std::env::var_os("RUSTMAGE_MSH_BROAD_DIAGNOSTIC").is_some() {
        let path = output
            .as_ref()
            .expect("diagnostic requires a fresh cache record path")
            .with_extension("progress.jsonl");
        Some(
            std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(path)
                .unwrap(),
        )
    } else {
        None
    };
    for (game_index, game) in frozen.games.iter().enumerate() {
        if !range.contains(&game_index) || only.is_some_and(|i| i != game_index) {
            continue;
        }
        let config = Config {
            decks: [decks[&game.decks[0]].clone(), decks[&game.decks[1]].clone()],
            seed: game.seed,
            starting_player: game.starting_player,
            deck_rules: DeckRules::Standard,
        };
        eprintln!(
            "MSH fixed config={game_index} group={group:?} phase={} decks={:?} seed={} starter={:?}",
            game.phase, game.decks, game.seed, game.starting_player
        );
        let mut g = Game::new(config.clone()).unwrap();
        let mut ledger = Vec::new();
        let mut checkpoints = 0usize;
        let mut commits = Vec::new();
        let mut seen_casts = BTreeSet::new();
        let mut events = BTreeMap::<String, usize>::new();
        let mut action_kinds = BTreeMap::<String, usize>::new();
        let (mut hand_swaps, mut library_positions) = (0usize, 0usize);
        let (mut legal_seconds, mut policy_seconds, mut apply_seconds) = (0.0f64, 0.0f64, 0.0f64);
        let (mut max_legal_seconds, mut max_legal_decision) = (0.0f64, 0usize);
        while g.outcome().is_none() && ledger.len() < frozen.decision_cap {
            let actor = g.actor().unwrap();
            progress_event(
                &mut progress,
                || serde_json::json!({"config":game_index,"decision":ledger.len(),"phase":"legal_actions","turn":g.state.turn,"pending":format!("{:?}",g.state.pending),"objects":g.state.objects.len(),"library_counts":[g.state.players[0].library.len(),g.state.players[1].library.len()]}),
            );
            let clock = std::time::Instant::now();
            let actions = g.legal_actions();
            let elapsed = clock.elapsed().as_secs_f64();
            legal_seconds += elapsed;
            if elapsed > max_legal_seconds {
                max_legal_seconds = elapsed;
                max_legal_decision = ledger.len();
            }
            progress_event(
                &mut progress,
                || serde_json::json!({"config":game_index,"decision":ledger.len(),"phase":"policy","legal_count":actions.len()}),
            );
            let clock = std::time::Instant::now();
            let index = crate::policy::smoke_index(&g, &actions);
            policy_seconds += clock.elapsed().as_secs_f64();
            let action = actions[index].clone();
            assert!(!matches!(action, Action::Concede));
            let event = natural_event_label(&g, &action);
            let action_json = serde_json::to_value(&action).unwrap();
            *action_kinds
                .entry(action_json["kind"].as_str().unwrap().to_owned())
                .or_default() += 1;
            ledger.push((actor, action.clone()));
            progress_event(
                &mut progress,
                || serde_json::json!({"config":game_index,"decision":ledger.len(),"phase":"before_apply","actor":actor,"semantic_actor":g.semantic_actor(),"action":action,"owned_privacy_checkpoint":ledger.len()%97==1}),
            );
            let clock = std::time::Instant::now();
            if ledger.len() % 97 == 1 {
                let counts = hidden_information_contract(&g, actor);
                hand_swaps += counts.hand_identity_swaps;
                library_positions += counts.library_positions_changed;
                g = owned_checked_step(g, actor, index, action);
                checkpoints += 1;
            } else {
                g.apply(actor, action).unwrap();
            }
            apply_seconds += clock.elapsed().as_secs_f64();
            progress_event(
                &mut progress,
                || serde_json::json!({"config":game_index,"decision":ledger.len(),"phase":"after_apply"}),
            );
            let newly_cast: Vec<_> = g
                .state
                .stack
                .iter()
                .filter_map(|item| {
                    if item.kind != StackKind::Spell {
                        return None;
                    }
                    let event = item.actual_cast_event()?;
                    if !seen_casts.insert(event.stack_id) {
                        return None;
                    }
                    let physical = item
                        .cast
                        .and_then(|cast| cast.physical_card)
                        .unwrap_or_else(|| g.physical_definition(item.card))
                        .front_face();
                    Some((physical, event.payment, event.controller, event.stack_id))
                })
                .collect();
            for (physical, payment, controller, stack_id) in newly_cast {
                *paid.entry(physical.identity().to_owned()).or_default() += 1;
                if payment.iter().any(|n| *n > 0) {
                    *paid_positive
                        .entry(physical.identity().to_owned())
                        .or_default() += 1;
                }
                commits.push(serde_json::json!({"decision":ledger.len(),"physical_identity":physical.identity(),"oracle_id":physical.info().oracle_id,"controller":controller,"stack_id":stack_id,"mana_payment":payment,"positive_mana_paid":payment.iter().any(|n|*n>0)}));
            }
            if let Some(event) = event {
                *events.entry(event).or_default() += 1;
            }
        }
        let replay_clock = std::time::Instant::now();
        let mut replay = Game::new(config).unwrap();
        for (actor, action) in &ledger {
            replay.apply(*actor, action.clone()).unwrap();
        }
        assert_eq!(g.snapshot(), replay.snapshot());
        assert_eq!(g.outcome(), replay.outcome());
        assert_eq!(views(&g), views(&replay));
        let replay_seconds = replay_clock.elapsed().as_secs_f64();
        assert!(checkpoints > 0);
        if g.outcome().is_none() {
            nonterminal.push(game_index);
        }
        decisions += ledger.len();
        let public = g.observe(game.starting_player);
        records.push(serde_json::json!({"config_index":game_index,"phase":game.phase,"decks":game.decks,"deck_configuration_sha256":[hashes[&game.decks[0]],hashes[&game.decks[1]]],"seed":game.seed,"starting_player":game.starting_player,"decisions":ledger.len(),"turn":g.state.turn,"outcome":g.outcome(),"pending_at_end":public.decision,"owned_privacy_checkpoints":checkpoints,"privacy_hand_identity_swaps":hand_swaps,"privacy_library_positions_changed":library_positions,"successful_spell_cast_commits":commits,"observed_successful_actions":events,"successful_action_kinds":action_kinds,"replay_snapshot_and_both_observations_verified":true,"owned_raw_index_restore_verified":true,"timing_seconds":{"legal_actions":legal_seconds,"policy_observation_and_choice":policy_seconds,"apply_including_owned_privacy_checks":apply_seconds,"replay_and_final_equalities":replay_seconds,"maximum_legal_actions":max_legal_seconds,"maximum_legal_actions_decision":max_legal_decision},"ledger":if full_ledger{serde_json::to_value(&ledger).unwrap()}else{serde_json::Value::Null}}));
    }
    assert_eq!(records.len(), if only.is_some() { 1 } else { range.len() });
    let deck_records:Vec<_>=frozen.decks.iter().map(|d|serde_json::json!({"id":d.id,"ordered_cards":d.cards,"configuration_sha256":d.configuration_sha256})).collect();
    if let Some(path) = output {
        write_record(
            &path,
            &serde_json::json!({"schema":1,"scope":"Fixed natural Standard60 MSH-only configurations; unchanged smoke_index, full exact replay, owned/privacy probes; each group is a subset, union verification required","planned_configurations":expected_games,"configuration_indices":range.collect::<Vec<_>>(),"group_index":group,"selected_configuration":only,"diagnostic_progress_enabled":progress.is_some(),"design_sha256":frozen.design_sha256,"decklists":deck_records,"games":records,"decisions":decisions,"paid_spell_commits_by_physical_identity":paid,"positive_mana_paid_spell_commits_by_physical_identity":paid_positive,"nonterminal_configuration_indices":nonterminal,"decision_cap":frozen.decision_cap,"checked_seconds_including_replay_assertions_and_record_preparation":start.elapsed().as_secs_f64(),"limits":"A committed cast is not a resolved effect. Announced activations and selected helpers are separate action metrics. No concession, seed substitution, progress injection or cap increase. Zero eligible privacy perturbations are counted honestly; full snapshots/raw-index/restore still checked."}),
        );
    }
    assert!(
        nonterminal.is_empty(),
        "fixed games reached decision cap without natural outcomes:{nonterminal:?}"
    );
}
fn progress_event(file: &mut Option<std::fs::File>, make: impl FnOnce() -> serde_json::Value) {
    if let Some(file) = file {
        use std::io::Write;
        writeln!(file, "{}", make()).unwrap();
        file.flush().unwrap();
    }
}
#[test]
fn msh_final281_privacy_probe_preserves_controlled_hands_and_visible_references() {
    use super::assurance::big_helpers::{named, setup};
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p.other());
        let first = named(&mut g, p.other(), "Island", Zone::Hand);
        let second = named(&mut g, p.other(), "Mountain", Zone::Hand);
        named(&mut g, p.other(), "Swamp", Zone::Library);
        named(&mut g, p.other(), "Island", Zone::Library);
        let top = named(&mut g, p.other(), "Plains", Zone::Library);
        let counts = hidden_information_contract(&g, p);
        assert_eq!(counts.hand_identity_swaps, 1);
        assert!(counts.library_positions_changed > 0);
        g.state.known_hand.push(g.object_ref(first));
        assert_eq!(hidden_information_contract(&g, p).hand_identity_swaps, 0);
        // Bounded entitlement seam, not a substitute for the paid Cube card tests.
        g.schedule_player_control(p.other(), p);
        g.begin_player_control_turn();
        assert_eq!(g.actor(), Some(p));
        assert_eq!(g.semantic_actor(), Some(p.other()));
        g.remember_library_top(p.other(), g.object_ref(top));
        let visible = g.observe(p);
        assert!(visible.objects.iter().any(|o| o.reference.card == first));
        assert!(visible.objects.iter().any(|o| o.reference.card == second));
        let counts = hidden_information_contract(&g, p);
        assert_eq!(counts.hand_identity_swaps, 0);
        assert!(counts.library_positions_changed > 0);
        let opponent_counts = hidden_information_contract(&g, p.other());
        assert_eq!(opponent_counts.hand_identity_swaps, 0);
    }
}
#[test]
fn msh_final281_privacy_probe_preserves_controlled_nick_private_top_seven() {
    use super::assurance::big_helpers::{cast, checked, named, settle, setup};
    for controller in [Player::P0, Player::P1] {
        let victim = controller.other();
        let mut g = setup(victim);
        let nick = named(
            &mut g,
            victim,
            "Nick Fury, Agent of S.H.I.E.L.D.",
            Zone::Hand,
        );
        g.state.players[victim.index()].mana = [1, 0, 0, 0, 0, 0];
        cast(&mut g, nick, None, &[]);
        settle(&mut g);
        for name in [
            "Island",
            "Swamp",
            "Mountain",
            "Agent Maria Hill",
            "Super Suit",
            "Dependable Quinjet",
            "Forest",
        ] {
            named(&mut g, victim, name, Zone::Library);
        }
        // The control setup isolates this test-harness entitlement boundary. The
        // Nick spell and discounted power-up are genuinely announced and paid.
        g.schedule_player_control(victim, controller);
        g.begin_player_control_turn();
        g.state.players[victim.index()].mana = [0, 1, 1, 1, 1, 0];
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: nick,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        for _ in 0..20 {
            if g.observe(controller).library_selection.is_some() {
                break;
            }
            let action = g
                .legal_actions()
                .into_iter()
                .find(|a| {
                    matches!(
                        a,
                        Action::Pass | Action::PayCosts { .. } | Action::FinishTargets
                    )
                })
                .unwrap();
            checked(&mut g, action);
        }
        assert_eq!(g.state.players[victim.index()].mana, [0; 6]);
        assert_eq!(
            g.observe(controller)
                .library_selection
                .as_ref()
                .unwrap()
                .remaining
                .len(),
            7
        );
        assert!(g.observe(victim).library_selection.is_some());
        assert_eq!(g.actor(), Some(controller));
        assert_eq!(g.semantic_actor(), Some(victim));
        let counts = hidden_information_contract(&g, controller);
        assert!(counts.library_positions_changed > 0);
        let actions = g.legal_actions();
        let index = actions
            .iter()
            .position(|a| *a == Action::FinishLibrarySelection)
            .unwrap();
        g = owned_checked_step(g, controller, index, Action::FinishLibrarySelection);
        settle(&mut g);
        assert!(g.on_battlefield(nick));
    }
}
