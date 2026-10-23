//! Frozen TMT-only natural games. Inclusion, committed casts and executed rules are separate evidence.
//! The optimized workload uses the same policy/decks/seeds without privacy, branch or replay checks.
use rustmage::{
    Action, Card, CastMethod, Config, DeckRules, Error, Game, Observation, Outcome, Player,
    StackKind, Zone,
};
use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;

const POOL_SEEDS: &[u64] = &[0, 7];
const THEME_SEEDS: &[u64] = &[0, 7, 19, 31];
const BASICS: [&str; 5] = ["Plains", "Island", "Swamp", "Mountain", "Forest"];
fn members() -> BTreeSet<Card> {
    let manifest: serde_json::Value =
        serde_json::from_str(include_str!("../data/sets/tmt.json")).unwrap();
    let cards = manifest["cards"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| Card::from_name(row["name"].as_str().unwrap()).expect("frozen member identity"))
        .collect::<BTreeSet<_>>();
    assert_eq!(cards.len(), 195, "frozen physical membership changed");
    for card in &cards {
        assert!(
            card.is_supported(),
            "full-set acceptance still gates {}",
            card.identity()
        );
    }
    cards
}
fn pool_decks(members: &BTreeSet<Card>) -> Vec<Vec<Card>> {
    let mut cards = members.iter().copied().collect::<Vec<_>>();
    cards.sort_by_key(|card| (card.colors(), card.fixed_mana_value(), card.name()));
    let (lands, spells): (Vec<_>, Vec<_>) = cards.iter().copied().partition(|card| card.is_land());
    let groups = spells.chunks(9).collect::<Vec<_>>();
    let count = groups.len().max(lands.len().div_ceil(6));
    let mut decks = Vec::new();
    for index in 0..count {
        let group = groups[index % groups.len()];
        let colors = group.iter().fold(0u8, |mask, card| mask | card.colors());
        let mut basics = BASICS
            .iter()
            .enumerate()
            .filter(|(i, _)| colors & (1 << i) != 0)
            .map(|(_, name)| Card::from_name(name).unwrap())
            .collect::<Vec<_>>();
        if basics.is_empty() {
            basics.push(Card::from_name("Forest").unwrap());
        }
        let mut deck = (0..24)
            .map(|i| basics[i % basics.len()])
            .collect::<Vec<_>>();
        for &card in group {
            deck.extend([card; 4]);
        }
        while deck.len() < 60 {
            deck.push(basics[deck.len() % basics.len()]);
        }
        decks.push(deck);
    }
    // Distribute every set land into real spell decks, four copies each.
    for (index, land) in lands.into_iter().enumerate() {
        let first = 4 * (index / count);
        decks[index % count][first..first + 4].fill(land);
    }
    assert_eq!(
        decks.iter().flatten().copied().collect::<BTreeSet<_>>(),
        cards.into_iter().collect()
    );
    decks
}
fn theme_decks(members: &BTreeSet<Card>) -> Vec<Vec<Card>> {
    let themes: Vec<(String, Vec<String>)> =
        serde_json::from_str(include_str!("data/tmt_themes.json")).unwrap();
    themes
        .into_iter()
        .map(|(colors, names)| {
            assert_eq!(names.len(), 9);
            let cards = names
                .iter()
                .map(|name| Card::from_name(name).unwrap())
                .collect::<Vec<_>>();
            assert!(
                cards
                    .iter()
                    .all(|card| members.contains(card) && card.is_supported())
            );
            let lands = colors
                .chars()
                .map(|color| Card::from_name(BASICS["WUBRG".find(color).unwrap()]).unwrap())
                .collect::<Vec<_>>();
            let mut deck = (0..24).map(|i| lands[i % lands.len()]).collect::<Vec<_>>();
            for card in cards {
                deck.extend([card; 4]);
            }
            deck
        })
        .collect()
}
type Workload = (&'static str, Vec<Vec<Card>>, &'static [u64]);
fn workloads() -> Vec<Workload> {
    let members = members();
    let pool = pool_decks(&members);
    let themes = theme_decks(&members);
    for deck in pool.iter().chain(&themes) {
        assert_eq!(deck.len(), 60);
        assert!(deck.iter().all(|card| members.contains(card)));
    }
    vec![
        ("entire pool", pool, POOL_SEEDS),
        ("mechanic themes", themes, THEME_SEEDS),
    ]
}
fn sideboards(deck: &[Card]) -> Vec<Card> {
    // Real private tournament pools; no fifth copy of any main-deck card.
    members()
        .into_iter()
        .filter(|c| c.is_creature() && c.info().legendary && !deck.contains(c))
        .take(15)
        .collect()
}
#[derive(Default)]
struct Evidence {
    executed: BTreeMap<&'static str, u64>,
    casts: BTreeSet<&'static str>,
    triggers: BTreeSet<&'static str>,
    games: usize,
    decisions: usize,
    branches: usize,
    private: usize,
    replayed: usize,
    combat: usize,
    drawn_to_empty: usize,
    wins: [usize; 2],
    draws: usize,
}
impl Evidence {
    fn add(&mut self, name: &'static str, amount: u64) {
        *self.executed.entry(name).or_default() += amount;
    }
    fn transition(&mut self, before: &Observation, after: &Observation, action: &Action) {
        if matches!(action, Action::ChooseKicker { pay: true }) {
            self.add("kicker choices", 1);
        }
        if matches!(action, Action::ChooseAdditionalCost { .. }) {
            self.add("additional costs selected", 1);
        }
        if matches!(action, Action::ChooseMode { .. }) {
            self.add("modal choices", 1);
        }
        if matches!(action, Action::ChooseCounterRemoval { .. }) {
            self.add("counter removals selected", 1);
        }
        if matches!(action, Action::CastExiledDuringResolution { .. }) {
            self.add("resolution cast choices", 1);
        }
        for packet in &after.stack {
            if before.stack.iter().any(|p| p.id == packet.id) {
                continue;
            }
            if packet.kind == StackKind::Spell {
                self.casts.insert(packet.source_card.name());
                if let Some(cast) = packet.cast {
                    if cast.method == CastMethod::Sneak {
                        self.add("sneak casts committed", 1);
                    }
                    if cast.method == CastMethod::Free {
                        self.add("free casts committed", 1);
                    }
                    if cast.origin == Zone::Exile {
                        self.add("exile casts committed", 1);
                    }
                }
            } else {
                self.triggers.insert(packet.source_card.name());
            }
        }
        for row in &after.ability_resolutions {
            let previous = before
                .ability_resolutions
                .iter()
                .find(|old| {
                    old.source == row.source
                        && old.definition == row.definition
                        && old.kind == row.kind
                        && old.ability_origin == row.ability_origin
                })
                .map_or(0, |old| old.count);
            let count = row.count.saturating_sub(previous);
            if matches!(row.kind, StackKind::TriggeredAbility { .. }) {
                self.add("triggered abilities resolved", count);
            }
            if matches!(row.kind, StackKind::ActivatedAbility { .. }) {
                self.add("activated abilities resolved", count);
            }
        }
        for row in &after.class_levels {
            let old = before
                .class_levels
                .iter()
                .find(|o| o.source == row.source)
                .map_or(1, |o| o.level);
            if row.level > old {
                self.add("class levels gained", u64::from(row.level - old));
            }
        }
        for row in &after.delayed_steps {
            if !before.delayed_steps.iter().any(|o| o.id == row.id) {
                self.add("delayed step programs scheduled", 1);
            }
        }
        for object in &after.objects {
            let old = before
                .objects
                .iter()
                .find(|o| o.reference == object.reference);
            if object.zone == Zone::Battlefield && old.is_none_or(|o| o.zone != Zone::Battlefield) {
                if object.is_token() {
                    self.add("tokens entered", 1);
                }
                if before
                    .objects
                    .iter()
                    .any(|o| o.id == object.id && o.zone == Zone::Graveyard)
                {
                    self.add("graveyard returns", 1);
                }
                if before
                    .objects
                    .iter()
                    .any(|o| o.id == object.id && o.zone == Zone::Outside)
                {
                    self.add("outside cards entered", 1);
                }
            }
            if object.counters.plus_one > old.map_or(0, |o| o.counters.plus_one) {
                self.add("plus counters placed", 1);
            }
        }
        if !after.attackers.is_empty() {
            self.combat += 1;
        }
    }
}
fn semantic_observation(game: &Game, viewer: Player) -> serde_json::Value {
    let mut value = serde_json::to_value(game.observe(viewer)).unwrap();
    value.as_object_mut().unwrap().remove("revision");
    value
}
fn private_choice(observation: &Observation) -> bool {
    observation.library_choice.is_some()
        || observation.search_choice.is_some()
        || observation.connive_choice.is_some()
        || observation
            .entry_value
            .as_ref()
            .is_some_and(|v| v.opponent_hand.is_some())
        || observation.effect_choice.as_ref().is_some_and(|choice| {
            choice.kind == rustmage::EffectChoiceKind::Discard
                || choice
                    .hand_reveal
                    .as_ref()
                    .is_some_and(|view| !view.revealed)
        })
}
fn run_workloads(workloads: Vec<Workload>, verify: bool) -> Evidence {
    let trace_steps = verify && std::env::var_os("RUSTMAGE_TMT_TRACE").is_some();
    let start = Instant::now();
    let mut evidence = Evidence::default();
    for (scope, decks, seeds) in &workloads {
        for pair in 0..decks.len() {
            for &seed in *seeds {
                for starting_player in [Player::P0, Player::P1] {
                    let config = || Config {
                        decks: [decks[pair].clone(), decks[(pair + 1) % decks.len()].clone()],
                        seed,
                        starting_player,
                        deck_rules: DeckRules::Standard,
                    };
                    if verify {
                        eprintln!(
                            "TMT_GAME start scope={scope} pair={pair} seed={seed} starter={starting_player:?}"
                        );
                    }
                    let game_start = Instant::now();
                    let mut game = Game::new_with_sideboards(
                        config(),
                        [
                            sideboards(&decks[pair]),
                            sideboards(&decks[(pair + 1) % decks.len()]),
                        ],
                    )
                    .unwrap();
                    let mut journal = Vec::new();
                    let mut decisions = 0usize;
                    let mut last_counts = (0usize, 0usize, 0usize);
                    while game.outcome().is_none() {
                        assert!(
                            decisions < 25000,
                            "TMT stalled scope={scope} pair={pair} seed={seed} starter={starting_player:?}"
                        );
                        if verify && decisions.is_multiple_of(128) {
                            eprintln!(
                                "TMT_PROGRESS scope={scope} pair={pair} seed={seed} starter={starting_player:?} decision={decisions} seconds={:.3} visible_objects={} battlefield={} stack={}",
                                game_start.elapsed().as_secs_f64(),
                                last_counts.0,
                                last_counts.1,
                                last_counts.2
                            );
                        }
                        let trace = |stage: &str| {
                            if trace_steps {
                                eprintln!(
                                    "TMT_TRACE scope={scope} pair={pair} seed={seed} starter={starting_player:?} decision={decisions} stage={stage}"
                                );
                            }
                        };
                        let decision_start = verify.then(Instant::now);
                        let actor = game.actor().unwrap();
                        trace("legal_actions");
                        let actions = game.legal_actions();
                        assert!(!actions.is_empty());
                        trace("policy");
                        let index = rustmage::policy::smoke_index(&game, &actions);
                        let action = actions[index].clone();
                        assert!(
                            !matches!(action, Action::Concede),
                            "policy dead end scope={scope} pair={pair} seed={seed} starter={starting_player:?} decision={decisions} view={:?}",
                            game.observe(actor)
                        );
                        if trace_steps {
                            eprintln!(
                                "TMT_ACTION decision={decisions} actor={actor:?} action={action:?}"
                            );
                        }
                        if verify {
                            trace("observe_before");
                            let before_view = game.observe(actor);
                            let private = private_choice(&before_view);
                            for viewer in [Player::P0, Player::P1] {
                                let view = game.observe(viewer);
                                assert!(view.objects.iter().all(|o| o.zone != Zone::Hand
                                    || o.owner == viewer
                                    || view.known_hand.contains(&o.reference)));
                                assert!(view.outside_game_cards.iter().all(|o| o.owner == viewer));
                                assert!(
                                    view.objects
                                        .iter()
                                        .all(|o| o.zone != Zone::Outside || o.owner == viewer)
                                );
                                if viewer != actor {
                                    assert!(view.actions.is_empty());
                                    assert!(view.library_choice.is_none());
                                    assert!(view.search_choice.is_none());
                                    assert!(view.connive_choice.is_none());
                                }
                            }
                            if private {
                                evidence.private += 1;
                            }
                            if decisions.is_multiple_of(64) || private {
                                trace("branch_snapshot");
                                let snapshot = game.snapshot();
                                let revision = game.revision();
                                assert_eq!(
                                    game.apply(actor.other(), action.clone()),
                                    Err(Error::WrongActor)
                                );
                                assert_eq!(game.snapshot(), snapshot);
                                let invalid = Action::ChooseMode { mode: u16::MAX };
                                assert!(!actions.contains(&invalid));
                                assert_eq!(game.apply(actor, invalid), Err(Error::IllegalAction));
                                assert_eq!(game.snapshot(), snapshot);
                                assert_eq!(game.revision(), revision);
                                let mut child = {
                                    let parent = game.clone();
                                    parent.clone()
                                };
                                trace("branch_apply_index");
                                child.apply_index(actor, revision, index).unwrap();
                                trace("branch_apply_raw");
                                game.apply(actor, action.clone()).unwrap();
                                assert_eq!(child.snapshot(), game.snapshot());
                                for viewer in [Player::P0, Player::P1] {
                                    assert_eq!(
                                        serde_json::to_value(child.observe(viewer)).unwrap(),
                                        serde_json::to_value(game.observe(viewer)).unwrap()
                                    );
                                }
                                let after = game.snapshot();
                                trace("restore");
                                game.restore(&snapshot).unwrap();
                                assert_eq!(
                                    game.apply_index(actor, revision, index),
                                    Err(Error::StaleRevision)
                                );
                                game.apply(actor, action.clone()).unwrap();
                                assert_eq!(game.snapshot(), after);
                                evidence.branches += 1;
                            } else {
                                trace("apply_raw");
                                game.apply(actor, action.clone()).unwrap();
                            }
                            trace("observe_transition");
                            let after_view = game.observe(actor);
                            last_counts = (
                                after_view.objects.len(),
                                after_view
                                    .objects
                                    .iter()
                                    .filter(|o| o.zone == Zone::Battlefield)
                                    .count(),
                                after_view.stack.len(),
                            );
                            evidence.transition(&before_view, &after_view, &action);
                            journal.push((actor, action));
                        } else {
                            game.apply(actor, action).unwrap();
                        }
                        if let Some(decision_start) = decision_start
                            && decision_start.elapsed().as_secs() >= 1
                        {
                            eprintln!(
                                "TMT_SLOW scope={scope} pair={pair} seed={seed} starter={starting_player:?} decision={decisions} seconds={:.3}",
                                decision_start.elapsed().as_secs_f64()
                            );
                        }
                        decisions += 1;
                    }
                    if verify {
                        eprintln!(
                            "TMT_REPLAY start scope={scope} pair={pair} seed={seed} starter={starting_player:?} decisions={decisions}"
                        );
                        let mut replay = Game::new_with_sideboards(
                            config(),
                            [
                                sideboards(&decks[pair]),
                                sideboards(&decks[(pair + 1) % decks.len()]),
                            ],
                        )
                        .unwrap();
                        for (replayed, (actor, action)) in journal.into_iter().enumerate() {
                            if trace_steps {
                                eprintln!(
                                    "TMT_REPLAY_ACTION decision={replayed} actor={actor:?} action={action:?}"
                                );
                            }
                            replay.apply(actor, action).unwrap();
                            evidence.replayed += 1;
                        }
                        assert_eq!(replay.snapshot(), game.snapshot());
                        for viewer in [Player::P0, Player::P1] {
                            assert_eq!(
                                semantic_observation(&replay, viewer),
                                semantic_observation(&game, viewer)
                            );
                        }
                    }
                    match game.outcome().unwrap() {
                        Outcome::Winner(Player::P0) => evidence.wins[0] += 1,
                        Outcome::Winner(Player::P1) => evidence.wins[1] += 1,
                        Outcome::Draw => evidence.draws += 1,
                    }
                    evidence.drawn_to_empty += usize::from(
                        game.observe(Player::P0)
                            .players
                            .iter()
                            .any(|p| p.library_count == 0),
                    );
                    if verify {
                        eprintln!(
                            "TMT_GAME end scope={scope} pair={pair} seed={seed} starter={starting_player:?} decisions={decisions} seconds={:.3}",
                            game_start.elapsed().as_secs_f64()
                        );
                    }
                    evidence.games += 1;
                    evidence.decisions += decisions;
                }
            }
        }
    }
    let elapsed = start.elapsed().as_secs_f64();
    let result = serde_json::json!({
        "mode": if verify { "verified natural games" } else { "optimized policy simulation" },
        "physical_members": members().len(),
        "supported_members": members().iter().filter(|c| c.is_supported()).count(),
        "workloads": workloads.iter().map(|(scope, decks, seeds)| serde_json::json!({
            "scope": scope, "decks": decks.len(), "seeds": seeds, "starters": ["P0", "P1"]
        })).collect::<Vec<_>>(),
        "games": evidence.games,
        "decisions": evidence.decisions,
        "owned_branches": evidence.branches,
        "private_decisions": evidence.private,
        "replayed_actions": evidence.replayed,
        "combat_observations": evidence.combat,
        "wins": evidence.wins,
        "draws": evidence.draws,
        "end_states_with_empty_library": evidence.drawn_to_empty,
        "seconds": elapsed,
        "decisions_per_second": evidence.decisions as f64 / elapsed,
        "games_per_second": evidence.games as f64 / elapsed,
        "decision_definition": "one accepted policy-selected action",
        "timing_scope": if verify {
            "game initialization, legal actions, smoke policy, apply, rules evidence, privacy, owned branches, restore and complete journal replay; excludes compilation and workload construction"
        } else {
            "game initialization, legal actions, smoke policy, apply and terminal checks; excludes compilation, workload construction, rules evidence, privacy/branch/restore checks and replay"
        },
        "cast_card_names": evidence.casts,
        "triggered_card_names": evidence.triggers,
        "executed": evidence.executed
    });
    println!("TMT_ACCEPTANCE {result}");
    evidence
}
#[test]
fn tmt_entire_pool_and_theme_acceptance() {
    let e = run_workloads(workloads(), true);
    assert!(
        e.games > 100 && e.decisions > 10000 && e.branches > 100 && e.private > 0 && e.combat > 0
    );
    assert_eq!(e.replayed, e.decisions);
    for mechanic in [
        "sneak casts committed",
        "class levels gained",
        "tokens entered",
        "plus counters placed",
        "triggered abilities resolved",
        "activated abilities resolved",
    ] {
        assert!(
            e.executed.get(mechanic).is_some_and(|&n| n > 0),
            "mechanic not executed: {mechanic}"
        );
    }
}
#[test]
#[ignore = "explicit optimized workload measurement after strict full-set acceptance"]
#[allow(
    clippy::assertions_on_constants,
    reason = "reject debug measurement at runtime"
)]
fn tmt_optimized_throughput() {
    assert!(!cfg!(debug_assertions), "use an optimized profile");
    let e = run_workloads(workloads(), false);
    assert!(e.games > 100 && e.decisions > 10000);
}
