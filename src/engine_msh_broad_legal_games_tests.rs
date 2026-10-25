//! Fixed whole-pool gameplay sampling; policy/engine inputs are never altered to make progress.
use super::msh_thirteenth_combined_tests::{
    hidden_information_contract, natural_event_label, owned_checked_step, views, write_cache_record,
};
use super::*;
use std::collections::{BTreeMap, BTreeSet};
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
#[test]
fn msh_broad_132_fixed_standard_set_only_games_owned_privacy_and_paid_identity_evidence() {
    let frozen: Pool =
        serde_json::from_str(include_str!("../data/assurance/msh-broad-legal-games.json")).unwrap();
    assert_eq!(frozen.schema, 1);
    assert_eq!(frozen.decision_cap, 30000);
    assert_eq!(frozen.decks.len(), 17);
    assert_eq!(frozen.games.len(), 132);
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
    let full_ledger = std::env::var_os("RUSTMAGE_RECORD_MSH_BROAD_GAMES").is_some();
    // Exact diagnostic selection retains the planned configuration and seed.
    let only = std::env::var("RUSTMAGE_MSH_BROAD_CONFIG")
        .ok()
        .map(|s| s.parse::<usize>().unwrap());
    assert!(only.is_none_or(|index| index < frozen.games.len()));
    let mut progress =
        if std::env::var_os("RUSTMAGE_MSH_BROAD_DIAGNOSTIC").is_some() {
            let path =
                std::path::PathBuf::from(std::env::var("RUSTMAGE_RECORD_MSH_BROAD_GAMES").unwrap())
                    .with_extension("progress.jsonl");
            assert!(path.starts_with(
                std::path::PathBuf::from(std::env::var("HOME").unwrap()).join(".cache")
            ));
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
        if only.is_some_and(|index| index != game_index) {
            continue;
        }
        let config = Config {
            decks: [decks[&game.decks[0]].clone(), decks[&game.decks[1]].clone()],
            seed: game.seed,
            starting_player: game.starting_player,
            deck_rules: DeckRules::Standard,
        };
        eprintln!(
            "MSH broad config={game_index} phase={} decks={:?} seed={} starter={:?}",
            game.phase, game.decks, game.seed, game.starting_player
        );
        let mut g = Game::new(config.clone()).unwrap();
        let mut ledger = Vec::new();
        let mut checkpoints = 0;
        let mut commits = Vec::new();
        let mut seen_casts = BTreeSet::new();
        let mut events = BTreeMap::<String, usize>::new();
        while g.outcome().is_none() && ledger.len() < frozen.decision_cap {
            let actor = g.actor().unwrap();
            progress_event(
                &mut progress,
                || serde_json::json!({"config":game_index,"decision":ledger.len(),"phase":"legal_actions","turn":g.state.turn,"pending":format!("{:?}",g.state.pending),"objects":g.state.objects.len(),"library_counts":[g.state.players[0].library.len(),g.state.players[1].library.len()]}),
            );
            let actions = g.legal_actions();
            progress_event(
                &mut progress,
                || serde_json::json!({"config":game_index,"decision":ledger.len(),"phase":"policy","legal_count":actions.len()}),
            );
            let index = crate::policy::smoke_index(&g, &actions);
            let action = actions[index].clone();
            assert!(!matches!(action, Action::Concede));
            let event = natural_event_label(&g, &action);
            ledger.push((actor, action.clone()));
            progress_event(
                &mut progress,
                || serde_json::json!({"config":game_index,"decision":ledger.len(),"phase":"before_apply","actor":actor,"action":action,"owned_privacy_checkpoint":ledger.len()%97==1}),
            );
            if ledger.len() % 97 == 1 {
                hidden_information_contract(&g, actor);
                g = owned_checked_step(g, actor, index, action);
                checkpoints += 1;
            } else {
                g.apply(actor, action).unwrap();
            }
            progress_event(
                &mut progress,
                || serde_json::json!({"config":game_index,"decision":ledger.len(),"phase":"after_apply"}),
            );
            // Read actual committed stack packets after successful application.
            // This covers ordinary announcements and legacy direct casts alike;
            // provisional or rolled-back payment proposals are never counted.
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
        let mut replay = Game::new(config).unwrap();
        for (actor, action) in &ledger {
            replay.apply(*actor, action.clone()).unwrap();
        }
        assert_eq!(g.snapshot(), replay.snapshot());
        assert_eq!(g.outcome(), replay.outcome());
        assert_eq!(views(&g), views(&replay));
        assert!(checkpoints > 0);
        if g.outcome().is_none() {
            nonterminal.push(game_index);
        }
        decisions += ledger.len();
        let public = g.observe(game.starting_player);
        records.push(serde_json::json!({"config_index":game_index,"phase":game.phase,"decks":game.decks,"deck_configuration_sha256":[hashes[&game.decks[0]],hashes[&game.decks[1]]],"seed":game.seed,"starting_player":game.starting_player,"decisions":ledger.len(),"turn":g.state.turn,"outcome":g.outcome(),"pending_at_end":public.decision,"owned_privacy_checkpoints":checkpoints,"successful_spell_cast_commits":commits,"observed_successful_actions":events,"ledger":if full_ledger{serde_json::to_value(&ledger).unwrap()}else{serde_json::Value::Null}}));
    }
    assert_eq!(records.len(), if only.is_some() { 1 } else { 132 });
    let deck_records:Vec<_>=frozen.decks.iter().map(|d|serde_json::json!({"id":d.id,"ordered_cards":d.cards,"configuration_sha256":d.configuration_sha256})).collect();
    write_cache_record(
        "RUSTMAGE_RECORD_MSH_BROAD_GAMES",
        serde_json::json!({"schema":1,"scope":"Fixed natural Standard60 MSH-only games from132 planned configurations across17 frozen decks; unchanged smoke_index; full exact replay and owned/privacy probes; bounded sampling, not full-set acceptance","planned_configurations":132,"selected_configuration":only,"diagnostic_progress_enabled":progress.is_some(),"design_sha256":frozen.design_sha256,"decklists":deck_records,"games":records,"decisions":decisions,"paid_spell_commits_by_physical_identity":paid,"positive_mana_paid_spell_commits_by_physical_identity":paid_positive,"nonterminal_configuration_indices":nonterminal,"decision_cap":frozen.decision_cap,"checked_seconds_including_replay_assertions_and_record_preparation":start.elapsed().as_secs_f64(),"limits":"A payment is not a resolved effect. Announced activations and target choices are separate successful-action metrics. Bounded nonterminal policy/workload gaps stay explicit; no forced progress, concession, seed substitution or claim that a cap alone is a rules defect. Uses only the132 preselected phase1/2 configurations; selected_configuration identifies a diagnostic subset. Optional24 late followups are not run."}),
    );
}

// Diagnostic writes observe the fixed workload and never feed the game or policy.
fn progress_event(file: &mut Option<std::fs::File>, make: impl FnOnce() -> serde_json::Value) {
    if let Some(file) = file {
        use std::io::Write;
        let value = make();
        writeln!(file, "{value}").unwrap();
        file.flush().unwrap();
    }
}
