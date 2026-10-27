//! One legal SOS workload for native replay assurance and a separate plain benchmark.
//! Action selection always delegates to the shared actor-observation smoke policy.
use super::*;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::time::Instant;
#[derive(Clone, Copy)]
#[allow(
    dead_code,
    reason = "The same support module serves native verification and the benchmark example"
)]
pub enum Mode {
    Verify,
    Benchmark,
}
impl Mode {
    fn verifies(self) -> bool {
        matches!(self, Self::Verify)
    }
}
#[derive(Clone, Serialize)]
pub struct DeckReport {
    pub name: &'static str,
    pub cards: BTreeMap<String, usize>,
    pub size: usize,
    pub standard_validation: bool,
    pub hob_membership: bool,
}
struct Deck {
    cards: Vec<Card>,
    report: DeckReport,
}
#[derive(Serialize)]
pub struct GameReport {
    pub pair: usize,
    pub seed: u64,
    pub starting_player: Player,
    pub decisions: usize,
    pub turns: u32,
    pub outcome: Outcome,
    pub owned_branches: usize,
    pub replayed_actions: usize,
    pub rejected_attack_attempts: usize,
    pub trace_digest: Option<String>,
    pub trace_file: Option<String>,
    pub cards_cast: BTreeSet<String>,
    pub witnesses: BTreeMap<String, u64>,
    pub max_sampled_state_bytes: usize,
}
#[derive(Serialize)]
pub struct Report {
    pub workload: &'static str,
    pub catalogue: &'static str,
    pub mode: &'static str,
    pub scope: &'static str,
    pub pool_members: usize,
    pub unique_cases: usize,
    pub decks: Vec<DeckReport>,
    pub games: Vec<GameReport>,
    pub witnesses: BTreeMap<String, u64>,
    pub actual_spell_colors: u8,
    pub seconds: f64,
    pub games_per_second: f64,
    pub decisions_per_second: f64,
    pub decisions: usize,
    pub measurement: &'static str,
}
impl Report {
    pub fn missing_mechanics(&self) -> Vec<&'static str> {
        [
            "enduring_story",
            "hone_counter",
            "goblin_army_token",
            "human_soldier_token",
            "landfall_trigger",
        ]
        .into_iter()
        .filter(|m| self.witnesses.get(*m).copied().unwrap_or(0) == 0)
        .collect()
    }
}
pub fn cases(seeds_per_pair: u64) -> Vec<(usize, u64, Player)> {
    (0..6)
        .flat_map(|pair| {
            (0..seeds_per_pair).map(move |seed| {
                (
                    pair,
                    782300 + pair as u64 * 100 + seed,
                    if seed % 2 == 0 {
                        Player::P0
                    } else {
                        Player::P1
                    },
                )
            })
        })
        .collect()
}
fn decks() -> Result<(Vec<Deck>, usize), String> {
    let source: serde_json::Value = serde_json::from_str(include_str!("../../data/sets/hob.json"))
        .map_err(|e| e.to_string())?;
    let members: BTreeSet<&str> = source["cards"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["oracle_id"].as_str().unwrap())
        .collect();
    if members.len() != 193 {
        return Err(format!("HOB denominator changed: {}", members.len()));
    }
    for row in source["cards"].as_array().unwrap() {
        let name = row["name"].as_str().unwrap();
        let c = Card::from_name(name).ok_or_else(|| format!("missing physical member {name}"))?;
        if !c.is_supported() {
            return Err(format!("gated HOB member {name}"));
        }
    }
    let specifications = [
        (
            "Dwarves",
            ["Plains", "Mountain"],
            [
                "Dwalin, Weaponmaster",
                "Iron Hills Blacksmith",
                "Dwarven Mattock",
                "Óin the Brave",
                "Dáin, Lord of the Iron Hills",
                "Sting, Bilbo's Sword",
                "Well-Worn Spatula",
                "The Mountain-king's Return",
            ],
        ),
        (
            "Goblins",
            ["Swamp", "Mountain"],
            [
                "Goblin-town Flunkies",
                "Bothersome Noisemaker",
                "Tidings of War",
                "Rage into the Valley",
                "Along the Crooked Way",
                "Nighthowl Pursuer",
                "Ravening Warg",
                "Fearsome Goblin Pair",
            ],
        ),
        (
            "Elves",
            ["Forest", "Island"],
            [
                "Attercop",
                "Beorn's Hospitality",
                "Down in the Valley",
                "Elven Raft-Steerer",
                "Mirkwood Meditator",
                "Wood Elves",
                "Woodland Weavemaster",
                "Silvan Reveler",
            ],
        ),
        (
            "Lake-town",
            ["Plains", "Island"],
            [
                "Lake-town Lookout",
                "Patient Instructor",
                "Great Gilded Boat",
                "Long Lake Nuisance",
                "The Queen of Dale",
                "The Mountain-king's Return",
                "Esgaroth Garrison",
                "Bard's Company",
            ],
        ),
        (
            "Wilderland",
            ["Swamp", "Forest"],
            [
                "Wargling",
                "Attercop",
                "Nighthowl Pursuer",
                "Ravening Warg",
                "Along the Crooked Way",
                "Rage into the Valley",
                "Wood Elves",
                "Woodland Weavemaster",
            ],
        ),
        (
            "Bears",
            ["Forest", "Mountain"],
            [
                "Little Bear",
                "Ordinary Bear",
                "Attercop",
                "Beorn's Hospitality",
                "Snowslope Hunter",
                "Goblin-town Flunkies",
                "Tidings of War",
                "Bothersome Noisemaker",
            ],
        ),
    ];
    let mut out = Vec::new();
    for (name, lands, spells) in specifications {
        let mut cards = Vec::new();
        for land in lands {
            let card = Card::from_name(land).unwrap();
            cards.extend(std::iter::repeat_n(card, 14));
        }
        for spell in spells {
            let card =
                Card::from_name(spell).ok_or_else(|| format!("unknown workload card {spell}"))?;
            cards.extend(std::iter::repeat_n(card, 4));
        }
        let mut counts = BTreeMap::new();
        for &c in &cards {
            if !c.is_supported() || !c.is_standard_legal() {
                return Err(format!("illegal workload card {}", c.name()));
            }
            if !c.info().oracle_id.is_some_and(|oid| members.contains(oid)) {
                return Err(format!("non-HOB workload card {}", c.name()));
            }
            *counts.entry(c.name().to_string()).or_insert(0usize) += 1;
        }
        for &c in &cards {
            let n = counts[c.name()];
            if n > 4
                && !matches!(
                    c,
                    Card::Plains | Card::Island | Card::Swamp | Card::Mountain | Card::Forest
                )
            {
                return Err(format!("too many copies of {}: {n}", c.name()));
            }
        }
        if cards.len() != 60 {
            return Err("workload deck must contain sixty cards".into());
        }
        let report = DeckReport {
            name,
            cards: counts,
            size: cards.len(),
            standard_validation: true,
            hob_membership: true,
        };
        out.push(Deck { cards, report });
    }
    Ok((out, members.len()))
}
#[derive(Default)]
struct Tracker {
    seen_packets: BTreeSet<u64>,
    seen_events: BTreeSet<String>,
    cards: BTreeSet<String>,
    witnesses: BTreeMap<String, u64>,
    colors: u8,
}
impl Tracker {
    fn hit(&mut self, key: &str, event: String) {
        if self.seen_events.insert(format!("{key}/{event}")) {
            *self.witnesses.entry(key.to_string()).or_default() += 1;
        }
    }
    fn inspect(&mut self, _before: &Observation, after: &Observation, _action: &Action) {
        for s in after.stack.iter().chain(after.resolving.iter()) {
            if !self.seen_packets.insert(s.id) {
                continue;
            }
            if s.kind == StackKind::Spell && !s.is_copy && s.cast.is_some() {
                self.cards.insert(s.source_card.name().into());
                self.colors |= s.source_card.colors();
                self.hit("actual_cast", s.id.to_string());
            }
            let printed_landfall = match (s.source_card.name(), s.kind) {
                (
                    "Attercop"
                    | "Beorn's Hospitality"
                    | "Elven Raft-Steerer"
                    | "Mirkwood Meditator",
                    StackKind::TriggeredAbility { ability: 0 },
                ) => true,
                ("Silvan Reveler", StackKind::TriggeredAbility { ability: 1 }) => true,
                _ => false,
            };
            if printed_landfall
                && s.source_ability_origin.is_none()
                && s.attachment_trigger.is_none()
            {
                self.hit("landfall_trigger", s.id.to_string());
            }
        }
        for p in [Player::P0, Player::P1] {
            if after.players[p.index()].enduring_story {
                self.hit("enduring_story", format!("{p:?}"));
            }
        }
        for object in &after.objects {
            if object.zone != Zone::Battlefield {
                continue;
            }
            if object.counters.get(CounterKind::Hone) > 0 {
                self.hit("hone_counter", format!("{:?}", object.reference));
            }
            if let Some(token) = &object.token {
                let subtypes: Vec<_> = token.subtypes.split_whitespace().collect();
                if subtypes.contains(&"Goblin") && subtypes.contains(&"Army") {
                    self.hit("goblin_army_token", format!("{:?}", object.reference));
                }
                if subtypes.contains(&"Human") && subtypes.contains(&"Soldier") {
                    self.hit("human_soldier_token", format!("{:?}", object.reference));
                }
            }
        }
    }
}
fn digest(mut hash: u64, actor: Player, action: &Action) -> u64 {
    for byte in serde_json::to_vec(&(actor, action)).unwrap() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}
pub fn run(
    cases: &[(usize, u64, Player)],
    mode: Mode,
    trace_dir: Option<&Path>,
) -> Result<Report, String> {
    let (decks, pool_members) = decks()?;
    let pairings = [(0, 1), (2, 3), (4, 1), (1, 2), (3, 4), (5, 2)];
    let started = Instant::now();
    let mut games = Vec::new();
    let mut total_witnesses = BTreeMap::new();
    let mut colors = 0;
    let mut total_decisions = 0;
    for &(pair, seed, starting_player) in cases {
        let (a, b) = pairings[pair];
        let config = Config {
            decks: [decks[a].cards.clone(), decks[b].cards.clone()],
            seed,
            starting_player,
            deck_rules: DeckRules::Standard,
        };
        let mut game =
            Game::new(config.clone()).map_err(|e| format!("legal deck rejected {e:?}"))?;
        let mut actions = Vec::new();
        let mut trace = Vec::new();
        let mut tracker = Tracker::default();
        let mut decisions = 0usize;
        let mut rejected_attacks = BTreeSet::new();
        let mut pending_attack = None;
        let mut rejected_attack_attempts = 0;
        let mut owned_branches = 0;
        let mut hash = 0xcbf29ce484222325u64;
        let mut max_bytes = game.state_bytes();
        while game.outcome().is_none() {
            if decisions >= 8192 {
                return Err(format!(
                    "nonterminal budget pair{pair}/seed{seed},turn{} phase{} last{:?}",
                    game.observe(game.actor().unwrap()).turn,
                    game.observe(game.actor().unwrap()).decision,
                    &trace[trace.len().saturating_sub(12)..]
                ));
            }
            game.legal_actions_into(&mut actions);
            let eligible: Vec<(usize, Action)> = actions
                .iter()
                .cloned()
                .enumerate()
                .filter(|(_, action)| match action {
                    Action::Attack { card } | Action::AttackPlaneswalker { card, .. } => {
                        !rejected_attacks.contains(card)
                    }
                    _ => true,
                })
                .collect();
            let offered: Vec<Action> = eligible.iter().map(|(_, action)| action.clone()).collect();
            let index = eligible[crate_policy(&game, &offered)].0;
            let action = actions[index].clone();
            let actor = game.actor().unwrap();
            match action {
                Action::Attack { card } | Action::AttackPlaneswalker { card, .. } => {
                    pending_attack = Some(card)
                }
                Action::CancelAttackDeclaration => {
                    if let Some(card) = pending_attack.take() {
                        rejected_attacks.insert(card);
                        rejected_attack_attempts += 1;
                    }
                }
                Action::FinishAttackers => {
                    rejected_attacks.clear();
                    pending_attack = None;
                }
                _ => {}
            }
            if matches!(action, Action::Concede) {
                return Err("smoke policy selected concession".into());
            }
            let before = game.observe(actor);
            if mode.verifies() {
                hash = digest(hash, actor, &action);
                trace.push((actor, action.clone()));
            }
            if mode.verifies()
                && (decisions.is_multiple_of(37)
                    || (before.decision != "priority" && decisions.is_multiple_of(13)))
            {
                let saved = game.snapshot();
                let revision = game.revision();
                let mut owned = {
                    let parent = game.clone();
                    parent.clone()
                };
                if game.apply(actor.other(), action.clone()) != Err(Error::WrongActor)
                    || game.snapshot() != saved
                {
                    return Err("wrong-actor mutation".into());
                }
                owned
                    .apply_index(actor, revision, index)
                    .map_err(|e| format!("owned index {e:?}"))?;
                game.apply(actor, action.clone())
                    .map_err(|e| format!("raw action {e:?}"))?;
                if game.snapshot() != owned.snapshot() {
                    return Err("owned/raw/index state mismatch".into());
                }
                for viewer in [Player::P0, Player::P1] {
                    if serde_json::to_value(game.observe(viewer)).unwrap()
                        != serde_json::to_value(owned.observe(viewer)).unwrap()
                    {
                        return Err("owned observation mismatch".into());
                    }
                }
                game.restore(&saved).map_err(|e| format!("restore {e:?}"))?;
                if game.apply_index(actor, revision, index) != Err(Error::StaleRevision) {
                    return Err("restore accepted stale index".into());
                }
                game.apply(actor, action.clone())
                    .map_err(|e| format!("restored raw action {e:?}"))?;
                if game.snapshot() != owned.snapshot() {
                    return Err("restored replay mismatch".into());
                }
                owned_branches += 1;
            } else {
                game.apply(actor, action.clone()).map_err(|e| {
                    format!("pair{pair}/seed{seed} action{decisions} {action:?}: {e:?}")
                })?;
            }
            let after = game.observe(actor);
            tracker.inspect(&before, &after, &action);
            if decisions.is_multiple_of(64) {
                max_bytes = max_bytes.max(game.state_bytes());
            }
            decisions += 1;
        }
        if mode.verifies() {
            let mut replay = Game::new(config.clone()).map_err(|e| format!("replay init {e:?}"))?;
            for (actor, action) in &trace {
                replay
                    .apply(*actor, action.clone())
                    .map_err(|e| format!("complete replay {e:?}"))?;
            }
            if replay.snapshot() != game.snapshot() {
                return Err(format!(
                    "complete deterministic replay mismatch pair{pair}/seed{seed}"
                ));
            }
        }
        let trace_file = if mode.verifies() {
            if let Some(dir) = trace_dir {
                std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
                let path = dir.join(format!("pair-{pair}-seed-{seed}.json"));
                let payload = serde_json::json!({"catalogue":CATALOGUE_VERSION,"decks":[decks[a].report.cards,decks[b].report.cards],"ordered_decks":[decks[a].cards,decks[b].cards],"seed":seed,"starting_player":starting_player,"actions":trace});
                std::fs::write(&path, serde_json::to_vec(&payload).unwrap())
                    .map_err(|e| e.to_string())?;
                Some(path.display().to_string())
            } else {
                None
            }
        } else {
            None
        };
        colors |= tracker.colors;
        for (key, count) in &tracker.witnesses {
            *total_witnesses.entry(key.clone()).or_insert(0u64) += count;
        }
        total_decisions += decisions;
        let outcome = game.outcome().unwrap();
        let turns = game.observe(starting_player).turn;
        games.push(GameReport {
            pair,
            seed,
            starting_player,
            decisions,
            turns,
            outcome,
            owned_branches,
            replayed_actions: if mode.verifies() { trace.len() } else { 0 },
            rejected_attack_attempts,
            trace_digest: mode.verifies().then(|| format!("{hash:016x}")),
            trace_file,
            cards_cast: tracker.cards,
            witnesses: tracker.witnesses,
            max_sampled_state_bytes: max_bytes,
        });
    }
    let seconds = started.elapsed().as_secs_f64();
    Ok(Report {
        workload: "HOB_ARCHETYPES_V1",
        catalogue: CATALOGUE_VERSION,
        mode: if mode.verifies() {
            "verification"
        } else {
            "benchmark"
        },
        scope: "Six exactly 60-card Standard decks; physical HOB members only; max four nonbasic copies, 28 basic lands; natural terminals; shared observation-only smoke policy with generic rejected-attack memory",
        pool_members,
        unique_cases: cases.iter().copied().collect::<BTreeSet<_>>().len(),
        decks: decks.into_iter().map(|d| d.report).collect(),
        games_per_second: games.len() as f64 / seconds,
        decisions_per_second: total_decisions as f64 / seconds,
        decisions: total_decisions,
        games,
        witnesses: total_witnesses,
        actual_spell_colors: colors,
        seconds,
        measurement: if mode.verifies() {
            "Includes full raw trace replay, sampled wrong-actor/owned/raw/index/restore/observation contracts and telemetry; not a simulation benchmark"
        } else {
            "Includes game construction, rules, action emission, shared smoke policy and observation telemetry. Excludes replay/owned checks, trace serialization, model inference, tensors and network. NamedHOBworkload only; not allStandard or XMage parity"
        },
    })
}
fn crate_policy(game: &Game, actions: &[Action]) -> usize {
    super::smoke_index(game, actions)
}
