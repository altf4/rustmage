#[path = "support/standard.rs"]
mod standard;

use rustmage::{
    Action, AttemptStage, AutoTapOptions, AutoTapStatus, Card, Config, DeckRules, Game, Player,
};
use std::collections::HashSet;
use std::hint::black_box;
use std::time::Instant;

// Optional second argument loads the exact mainboards and checks every sideboard
// card too. The native smoke policy supplies manual gameplay decisions.
fn named_decks(path: &str) -> Vec<(String, Vec<Card>)> {
    let data: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).expect("decklist archive"))
            .expect("decklist JSON");
    data["decks"]
        .as_object()
        .expect("named decks")
        .iter()
        .map(|(name, list)| {
            let mut mainboard = Vec::new();
            for side in ["mainboard", "sideboard"] {
                let mut count = 0usize;
                for row in list[side].as_array().expect("deckboard") {
                    let name = row["name"].as_str().expect("card name");
                    let card = Card::from_name(name)
                        .unwrap_or_else(|| panic!("unknown pool card: {name}"));
                    assert!(card.is_supported(), "gated pool card: {name}");
                    let quantity = usize::try_from(row["quantity"].as_u64().expect("quantity"))
                        .expect("quantity range");
                    count += quantity;
                    if side == "mainboard" {
                        mainboard.extend(std::iter::repeat_n(card, quantity));
                    }
                }
                assert_eq!(count, if side == "mainboard" { 60 } else { 15 });
            }
            (name.clone(), mainboard)
        })
        .collect()
}

fn replay_indexed(branch: &mut Game, trace: &[Action]) {
    let mut actions = Vec::new();
    for action in trace {
        let actor = branch.actor().expect("auto-tap trace after game end");
        branch.legal_actions_into(&mut actions);
        if let Some(index) = actions.iter().position(|offered| offered == action) {
            branch.apply_index(actor, branch.revision(), index).unwrap();
        } else {
            // Large numeric domains can have legal raw allocations outside the indexed sample.
            branch.apply(actor, action.clone()).unwrap();
        }
    }
}

/// Check the original raw/index path, then replay from the restored state at its fresh revision.
fn verify_auto_tap(mut branch: Game, after: &Game, trace: &[Action]) {
    let saved = branch.snapshot();
    replay_indexed(&mut branch, trace);
    assert!(
        branch.snapshot() == after.snapshot(),
        "indexed auto-tap replay diverged"
    );
    assert_eq!(branch.revision(), after.revision());
    branch.restore(&saved).unwrap();
    assert!(branch.snapshot() == saved);
    let mut restored_peer = after.clone();
    restored_peer.restore(&saved).unwrap();
    replay_indexed(&mut restored_peer, trace);
    for action in trace {
        let actor = branch.actor().expect("auto-tap trace after game end");
        branch.apply(actor, action.clone()).unwrap();
    }
    assert!(
        branch.snapshot() == restored_peer.snapshot(),
        "restored auto-tap replay diverged"
    );
}

#[derive(Default)]
struct AutoTapMetrics {
    latency_ns: Vec<u64>,
    completed: u64,
    paid: u64,
    needs_decision: u64,
    insufficient: u64,
    primitives: u64,
}

impl AutoTapMetrics {
    fn pay(&mut self, game: &mut Game, actor: Player, attempt_id: u64, verify: bool) -> u64 {
        let revision = game.revision();
        let options = AutoTapOptions::default();
        let branch = verify.then(|| game.clone());
        let start = Instant::now();
        let result = game
            .auto_pay_mana(actor, revision, attempt_id, &options)
            .unwrap();
        self.latency_ns
            .push(u64::try_from(start.elapsed().as_nanos()).expect("request duration"));
        match result.status {
            AutoTapStatus::Completed => self.completed += 1,
            AutoTapStatus::Paid => self.paid += 1,
            AutoTapStatus::NeedsDecision => self.needs_decision += 1,
            AutoTapStatus::InsufficientSupportedMana => self.insufficient += 1,
        }
        let count = result.actions.len() as u64;
        assert_eq!(
            game.revision() - revision,
            count,
            "auto-tap primitive trace length"
        );
        assert_eq!(
            matches!(
                result.status,
                AutoTapStatus::Completed | AutoTapStatus::Paid
            ),
            count != 0
        );
        if let Some(branch) = branch {
            verify_auto_tap(branch, game, &result.actions);
        }
        self.primitives += count;
        count
    }

    fn report(mut self, decisions: u64) -> serde_json::Value {
        self.latency_ns.sort_unstable();
        let requests = self.latency_ns.len();
        let percentile = |p: usize| {
            let rank = ((requests as u128 * p as u128).div_ceil(100)) as usize;
            rank.checked_sub(1).map(|index| self.latency_ns[index])
        };
        let manual = decisions - self.primitives;
        serde_json::json!({
            "protocol": "native_auto_tap_v1",
            "requests": requests,
            "completed": self.completed,
            "paid": self.paid,
            "declined": self.needs_decision + self.insufficient,
            "needs_decision": self.needs_decision,
            "insufficient_supported_mana": self.insufficient,
            "assisted_primitive_actions": self.primitives,
            "manual_primitive_actions": manual,
            "caller_decisions": manual + requests as u64,
            "auto_pay_p50_ns": percentile(50),
            "auto_pay_p99_ns": percentile(99),
        })
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args: Vec<_> = std::env::args().skip(1).collect();
    if args.first().is_some_and(|arg| arg == "--standard-smoke") {
        let games = args.get(1).map_or("1", String::as_str).parse()?;
        return standard::run(games);
    }
    let auto_tap = args.iter().any(|arg| arg == "--auto-tap");
    args.retain(|arg| arg != "--auto-tap");
    let verify = args.len() >= 2 && args.last().is_some_and(|arg| arg == "verify");
    if verify {
        args.pop();
    }
    assert!(
        args.len() <= 2,
        "usage: benchmark [games [archive]] [verify] [--auto-tap]"
    );
    let games: usize = args
        .first()
        .map_or("1000", String::as_str)
        .parse()
        .expect("game count");
    assert!(games > 0);
    let deck: Vec<_> = [
        vec![Card::Mountain; 24],
        vec![Card::LightningBolt; 20],
        vec![Card::GoblinPiker; 12],
        vec![Card::HillGiant; 4],
    ]
    .concat();
    let archive = args.get(1);
    let named = archive
        .map(|path| named_decks(path))
        .unwrap_or_else(|| vec![("native-smoke".into(), deck)]);
    assert!(!named.is_empty());
    let config = Config {
        decks: [named[0].1.clone(), named[1 % named.len()].1.clone()],
        seed: 42,
        starting_player: Player::P0,
        deck_rules: if archive.is_some() {
            DeckRules::Standard
        } else {
            DeckRules::Freeform
        },
    };
    let root = Game::new(config.clone()).unwrap();
    let saved = root.snapshot();
    let clone_iterations = 100_000;
    let start = Instant::now();
    for _ in 0..clone_iterations {
        black_box(black_box(&root).clone());
    }
    let clone_ns = start.elapsed().as_nanos() as f64 / clone_iterations as f64;
    let mut branch = root.clone();
    let start = Instant::now();
    for _ in 0..clone_iterations {
        branch.restore(black_box(&saved)).unwrap();
        black_box(&branch);
    }
    let restore_ns = start.elapsed().as_nanos() as f64 / clone_iterations as f64;
    let mut actions = Vec::new();
    let mut decisions = 0u64;
    let mut max_game = 0;
    let mut auto_tap_metrics = AutoTapMetrics::default();
    let start = Instant::now();
    for seed in 0..games {
        let mut config = config.clone();
        config.seed = seed as u64;
        config.decks = [
            named[seed % named.len()].1.clone(),
            named[(seed + 1) % named.len()].1.clone(),
        ];
        let mut game = Game::new(config).unwrap();
        let mut count = 0;
        let mut attempted = auto_tap.then(HashSet::new);
        while game.outcome().is_none() {
            let actor = game.actor().unwrap();
            if let Some(attempted) = &mut attempted
                && let Some(attempt) = game.attempt_view(actor)
                && attempt.context.is_some()
                && attempt.locked_cost.is_some()
                && matches!(
                    attempt.stage,
                    AttemptStage::GenerateMana | AttemptStage::PayCosts
                )
                && attempted.insert(attempt.attempt_id)
            {
                let advanced = auto_tap_metrics.pay(&mut game, actor, attempt.attempt_id, verify);
                if advanced > 0 {
                    count += advanced;
                    assert!(count < 100_000, "benchmark did not terminate naturally");
                    continue;
                }
            }
            game.legal_actions_into(&mut actions);
            let index = rustmage::policy::smoke_index(&game, &actions);
            let revision = game.revision();
            let action = actions[index].clone();
            let owned = verify.then(|| {
                let snapshot = game.snapshot();
                let branch = {
                    let parent = game.clone();
                    parent.clone()
                };
                (snapshot, branch)
            });
            game.apply(actor, action.clone()).unwrap_or_else(|error| {
                panic!("seed {seed}, decision {count}, {action:?}: {error:?}")
            });
            if let Some((snapshot, mut branch)) = owned {
                branch.apply_index(actor, revision, index).unwrap();
                assert!(
                    branch.snapshot() == game.snapshot(),
                    "indexed branch diverged at seed {seed}, decision {count}"
                );
                branch.restore(&snapshot).unwrap();
                assert!(branch.snapshot() == snapshot);
                // Attempt IDs incorporate the revision at announcement. Compare the replay
                // with an indexed peer at the same fresh revision after restoring.
                let mut restored_peer = game.clone();
                restored_peer.restore(&snapshot).unwrap();
                restored_peer
                    .apply_index(actor, restored_peer.revision(), index)
                    .unwrap();
                branch.apply(actor, action).unwrap();
                assert!(
                    branch.snapshot() == restored_peer.snapshot(),
                    "restored branch diverged at seed {seed}, decision {count}"
                );
            }
            count += 1;
            assert!(count < 100_000, "benchmark did not terminate naturally");
        }
        max_game = max_game.max(count);
        decisions += count;
        black_box(game.outcome());
    }
    let elapsed = start.elapsed().as_secs_f64();
    let mut report = serde_json::json!({
        "games": games, "decisions": decisions, "seconds": elapsed,
        "decklist_archive": archive,
        "owned_raw_index_restore_checks": verify,
        "deck_names": named.iter().map(|(name, _)| name).collect::<Vec<_>>(),
        "deck_rules": if archive.is_some() { "Standard" } else { "Freeform" },
        "games_per_second": games as f64 / elapsed,
        "decisions_per_second": decisions as f64 / elapsed,
        "max_decisions_per_game": max_game,
        "initial_state_bytes_excluding_capacity_and_allocator": root.state_bytes(),
        "clone_ns": clone_ns, "restore_ns": restore_ns,
        "note": "native smoke policy and observation construction included; optional named decks rotate adjacent matchups through both seats; setup and gameplay timed, archive loading excluded; no model, tensor encoding, or per-decision JSON; verify mode also times owned raw/index branch and restore checks; sampled gameplay is not complete card-interaction coverage or XMage parity"
    });
    if auto_tap {
        report["auto_tap"] = auto_tap_metrics.report(decisions);
    }
    println!("{report}");
    Ok(())
}
