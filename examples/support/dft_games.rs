//! One legal DFT workload for native replay assurance and a separate plain benchmark.
//! Action selection delegates to the shared actor-observation smoke policy after
//! generic public-state filters avoid redundant preparation and rejected attacks.
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
    pub dft_membership: bool,
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
    pub deck_swapped: bool,
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
pub const REQUIRED_MECHANICS: [&str; 15] = [
    "speed_started",
    "speed_increased",
    "max_speed",
    "exhaust_paid",
    "crew_paid",
    "saddled_attack",
    "cycling",
    "exert_paid",
    "discard",
    "token_created",
    "graveyard_return",
    "embalm_paid",
    "embalmed_token",
    "as_entry_copy",
    "extra_combat",
];
impl Report {
    pub fn missing_mechanics(&self) -> Vec<&'static str> {
        REQUIRED_MECHANICS
            .into_iter()
            .filter(|m| self.witnesses.get(*m).copied().unwrap_or(0) == 0)
            .collect()
    }
}
pub fn cases(seeds_per_pair: u64) -> Vec<(usize, u64, Player, bool)> {
    (0..10)
        .flat_map(|pair| {
            (0..seeds_per_pair).flat_map(move |seed| {
                [false, true].into_iter().map(move |swapped| {
                    (
                        pair,
                        921400 + pair as u64 * 100 + seed,
                        if seed % 2 == 0 {
                            Player::P0
                        } else {
                            Player::P1
                        },
                        swapped,
                    )
                })
            })
        })
        .collect()
}
/// Public immutable deck evidence, also used to authenticate sharded reports.
pub fn deck_reports() -> Result<Vec<DeckReport>, String> {
    decks().map(|(decks, _)| decks.into_iter().map(|deck| deck.report).collect())
}
fn decks() -> Result<(Vec<Deck>, usize), String> {
    let source: serde_json::Value = serde_json::from_str(include_str!("../../data/sets/dft.json"))
        .map_err(|e| e.to_string())?;
    let members: BTreeSet<&str> = source["cards"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["oracle_id"].as_str().unwrap())
        .collect();
    if members.len() != 276 {
        return Err(format!("DFT denominator changed: {}", members.len()));
    }
    for row in source["cards"].as_array().unwrap() {
        let name = row["name"].as_str().unwrap();
        let c = Card::from_name(name).ok_or_else(|| format!("missing physical member {name}"))?;
        if !c.is_supported() || c.info().oracle_id != row["oracle_id"].as_str() {
            return Err(format!("gated or mismatched DFT member {name}"));
        }
    }
    let specifications = [
        (
            "White-blue artifacts",
            ["Plains", "Island"],
            [
                "Voyager Quickwelder",
                "Interface Ace",
                "Voyager Glidecar",
                "Spotcycle Scouter",
                "Nimble Thopterist",
                "Mu Yanling, Wind Rider",
                "Stock Up",
                "Roadside Assistance",
            ],
        ),
        (
            "Blue-black speed",
            ["Island", "Swamp"],
            [
                "Gas Guzzler",
                "Engine Rat",
                "Deathless Pilot",
                "Keen Buccaneer",
                "Scrounging Skyray",
                "Carrion Cruiser",
                "Midnight Mangler",
                "Stall Out",
            ],
        ),
        (
            "Black-red speed",
            ["Swamp", "Mountain"],
            [
                "Burnout Bashtronaut",
                "Endrider Catalyzer",
                "Endrider Spikespitter",
                "Dynamite Diver",
                "Engine Rat",
                "Gas Guzzler",
                "Lightning Strike",
                "Kickoff Celebrations",
            ],
        ),
        (
            "Red-green exhaust",
            ["Mountain", "Forest"],
            [
                "Stampeding Scurryfoot",
                "Afterburner Expert",
                "Elvish Refueler",
                "Draconautics Engineer",
                "Greasewrench Goblin",
                "Beastrider Vanguard",
                "Gilded Ghoda",
                "Lightning Strike",
            ],
        ),
        (
            "Green-white saddle",
            ["Forest", "Plains"],
            [
                "Alacrian Jaguar",
                "Brightfield Mustang",
                "Brightfield Glider",
                "Bulwark Ox",
                "Cloudspire Captain",
                "Canyon Vaulter",
                "District Mascot",
                "Defend the Rider",
            ],
        ),
        (
            "White-black recursion",
            ["Plains", "Swamp"],
            [
                "Basri, Tomorrow's Champion",
                "Nesting Bot",
                "Swiftwing Assailant",
                "Cursecloth Wrappings",
                "Deathless Pilot",
                "Cryptcaller Chariot",
                "Back on Track",
                "Grim Bauble",
            ],
        ),
        (
            "Blue-red exhaust",
            ["Island", "Mountain"],
            [
                "Keen Buccaneer",
                "Mindspring Merfolk",
                "Rangers' Refueler",
                "Greasewrench Goblin",
                "Draconautics Engineer",
                "Scrounging Skyray",
                "Lightning Strike",
                "Fuel the Flames",
            ],
        ),
        (
            "Black-green graveyards",
            ["Swamp", "Forest"],
            [
                "Afterburner Expert",
                "Molt Tender",
                "Ooze Patrol",
                "Pothole Mole",
                "Dredger's Insight",
                "Carrion Cruiser",
                "Back on Track",
                "Deathless Pilot",
            ],
        ),
        (
            "Red-white combat",
            ["Mountain", "Plains"],
            [
                "Basri, Tomorrow's Champion",
                "Cloudspire Captain",
                "Canyon Vaulter",
                "Brightfield Mustang",
                "Gilded Ghoda",
                "Dracosaur Auxiliary",
                "Full Throttle",
                "Roadside Assistance",
            ],
        ),
        (
            "Green-blue value",
            ["Forest", "Island"],
            [
                "Stampeding Scurryfoot",
                "Elvish Refueler",
                "Keen Buccaneer",
                "Rangers' Refueler",
                "Stock Up",
                "Oviya, Automech Artisan",
                "Waxen Shapethief",
                "Veloheart Bike",
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
                return Err(format!("non-DFT workload card {}", c.name()));
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
            dft_membership: true,
        };
        out.push(Deck { cards, report });
    }
    Ok((out, members.len()))
}
#[derive(Default)]
struct Tracker {
    seen_packets: BTreeSet<u64>,
    hand_activations: BTreeSet<u64>,
    embalm_announcements: BTreeSet<u64>,
    combat_starts: BTreeMap<u32, u32>,
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
    fn inspect(&mut self, before: &Observation, after: &Observation, action: &Action) {
        if matches!(action, Action::AnnounceEmbalm { .. })
            && let Some(item) = &after.announcing
        {
            self.embalm_announcements.insert(item.id);
        }
        if let Action::ChooseEntryCopy { object: Some(_) } = action {
            self.hit("as_entry_copy", format!("{}:{action:?}", after.turn));
        }
        if before.step != after.step && after.step == Step::BeginCombat {
            let starts = self.combat_starts.entry(after.turn).or_default();
            *starts += 1;
            let count = *starts;
            if count > 1 {
                self.hit("extra_combat", format!("{}:{count}", after.turn));
            }
        }
        for view in [before, after] {
            if let Some(item) = &view.announcing
                && matches!(item.kind, StackKind::ActivatedAbility { .. })
                && let Some(source) = item.source()
                && source.card.info().oracle_keywords.contains(&"Cycling")
                && view
                    .objects
                    .iter()
                    .any(|object| object.id == source.id && object.zone == Zone::Hand)
            {
                self.hand_activations.insert(item.id);
            }
        }
        for s in after.stack.iter().chain(after.resolving.iter()) {
            if !self.seen_packets.insert(s.id) {
                continue;
            }
            let Some(source) = s.source() else {
                continue;
            };
            if s.kind == StackKind::Spell && !s.is_copy && s.cast.is_some() {
                self.cards.insert(source.card.name().into());
                self.colors |= source.card.colors();
                self.hit("actual_cast", s.id.to_string());
            }
            if self.embalm_announcements.contains(&s.id) {
                self.hit("embalm_paid", s.id.to_string());
            }
            if matches!(s.kind, StackKind::ActivatedAbility { .. })
                && (self.hand_activations.contains(&s.id)
                    || (source.card.info().oracle_keywords.contains(&"Cycling")
                        && before
                            .objects
                            .iter()
                            .any(|object| object.id == source.id && object.zone == Zone::Hand)
                        && after.objects.iter().any(|object| {
                            object.id == source.id && object.zone == Zone::Graveyard
                        })))
            {
                self.hit("cycling", s.id.to_string());
            }
        }
        for player in [Player::P0, Player::P1] {
            let old = &before.players[player.index()];
            let new = &after.players[player.index()];
            if old.speed == 0 && new.speed == 1 {
                self.hit("speed_started", format!("{player:?}"));
            }
            if old.speed > 0 && new.speed > old.speed {
                self.hit("speed_increased", format!("{player:?}/{}", new.speed));
            }
            if new.speed == 4 {
                self.hit("max_speed", format!("{player:?}"));
            }
            if new.cards_discarded_this_turn > old.cards_discarded_this_turn {
                self.hit(
                    "discard",
                    format!(
                        "{player:?}/{}/{}",
                        after.turn, new.cards_discarded_this_turn
                    ),
                );
            }
        }
        for history in &after.crewed_vehicles {
            if !history.contributors.is_empty() {
                self.hit(
                    "crew_paid",
                    format!(
                        "{:?}/{}/{:?}",
                        history.vehicle, after.turn, history.contributors
                    ),
                );
            }
        }
        for row in after.exertions.iter().flatten() {
            self.hit(
                "exert_paid",
                format!("{:?}/{:?}/{}", row.object, row.player, after.turn),
            );
        }
        for object in &after.objects {
            if object.zone != Zone::Battlefield {
                continue;
            }
            let old_exhaust = before
                .objects
                .iter()
                .find(|old| old.reference == object.reference)
                .map_or(0, |old| old.exhausted_abilities);
            if object.exhausted_abilities & !old_exhaust != 0 {
                self.hit(
                    "exhaust_paid",
                    format!("{:?}/{}", object.reference, object.exhausted_abilities),
                );
            }
            if object.attacking && object.saddled {
                self.hit(
                    "saddled_attack",
                    format!("{:?}/{}", object.reference, after.turn),
                );
            }
            if object.is_token() {
                self.hit("token_created", format!("{:?}", object.reference));
                if object
                    .copy_recipe
                    .is_some_and(|recipe| recipe.exceptions.embalm)
                {
                    self.hit("embalmed_token", format!("{:?}", object.reference));
                }
            }
            if before.objects.iter().any(|old| {
                old.id == object.id
                    && old.zone == Zone::Graveyard
                    && old.reference != object.reference
            }) {
                self.hit("graveyard_return", format!("{:?}", object.reference));
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
/// This is a strategy filter, not a legality rule. Re-crewing can be legal
/// (including for tap triggers), but is unnecessary merely to prepare an already
/// animated Vehicle. Inspect only public effective views and immutable programs;
/// preserve activations that also have other effects, such as adding counters.
fn redundant_preparation(
    observation: &Observation,
    action: &Action,
    finish_crew: bool,
    finish_saddle: bool,
) -> bool {
    match action {
        Action::ChooseCrewer { .. } => finish_crew,
        Action::ChooseSaddler { .. } => finish_saddle,
        Action::AnnounceActivate { card, ability, .. }
        | Action::BeginActivate { card, ability, .. }
        | Action::Activate { card, ability, .. } => observation
            .objects
            .iter()
            .find(|object| object.id == *card && object.zone == Zone::Battlefield)
            .and_then(|object| {
                rustmage_core::cards::authored::rules(object.card)
                    .and_then(|rules| rules.activated.get(usize::from(*ability)))
                    .map(|ability| (object, ability))
            })
            .is_some_and(|(object, ability)| match ability.program {
                [rustmage_core::cards::Instruction::Crew { .. }] => object.kinds & 1 != 0,
                [rustmage_core::cards::Instruction::BecomeSaddled { .. }] => object.saddled,
                _ => false,
            }),
        _ => false,
    }
}
pub fn run(
    cases: &[(usize, u64, Player, bool)],
    mode: Mode,
    trace_dir: Option<&Path>,
) -> Result<Report, String> {
    let (decks, pool_members) = decks()?;
    let pairings = [
        (0, 1),
        (2, 3),
        (4, 5),
        (6, 7),
        (8, 9),
        (0, 7),
        (1, 4),
        (2, 9),
        (3, 5),
        (6, 8),
    ];
    let started = Instant::now();
    let mut games = Vec::new();
    let mut total_witnesses = BTreeMap::new();
    let mut colors = 0;
    let mut total_decisions = 0;
    for &(pair, seed, starting_player, deck_swapped) in cases {
        let (a, b) = pairings[pair];
        let (a, b) = if deck_swapped { (b, a) } else { (a, b) };
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
            let actor = game.actor().unwrap();
            let before = game.observe(actor);
            // Offered completion already includes special contribution rules
            // (toughness, as-though bonuses, etc.); never recompute those here.
            let finish_crew = actions.contains(&Action::FinishCrew);
            let finish_saddle = actions.contains(&Action::FinishSaddle);
            let eligible: Vec<(usize, Action)> = actions
                .iter()
                .cloned()
                .enumerate()
                .filter(|(_, action)| {
                    !redundant_preparation(&before, action, finish_crew, finish_saddle)
                })
                .filter(|(_, action)| match action {
                    Action::Attack { card } | Action::AttackPlaneswalker { card, .. } => {
                        !rejected_attacks.contains(card)
                    }
                    _ => true,
                })
                .collect();
            if eligible.is_empty() {
                return Err("workload strategy filtered every legal action".into());
            }
            let offered: Vec<Action> = eligible.iter().map(|(_, action)| action.clone()).collect();
            let index = eligible[crate_policy(&game, &offered)].0;
            let action = actions[index].clone();
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
                let path = dir.join(format!(
                    "pair-{pair}-seed-{seed}-swapped-{deck_swapped}.json"
                ));
                let payload = serde_json::json!({"catalogue":CATALOGUE_VERSION,"decks":[decks[a].report.cards,decks[b].report.cards],"ordered_decks":[decks[a].cards,decks[b].cards],"seed":seed,"starting_player":starting_player,"deck_swapped":deck_swapped,"actions":trace});
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
            deck_swapped,
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
        workload: "DFT_ARCHETYPES_V1",
        catalogue: CATALOGUE_VERSION,
        mode: if mode.verifies() {
            "verification"
        } else {
            "benchmark"
        },
        scope: "Ten exactly 60-card Standard decks; physical DFT members only; max four nonbasic copies, 28 basic lands; natural terminals; shared observation-only smoke policy with generic rejected-attack memory and redundant pure Crew/Saddle preparation filters",
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
            "Includes game construction, rules, action emission, shared smoke policy and observation telemetry. Excludes replay/owned checks, trace serialization, model inference, tensors and network. NamedDFTworkload only; not allStandard or XMage parity"
        },
    })
}
fn crate_policy(game: &Game, actions: &[Action]) -> usize {
    super::smoke_index(game, actions)
}
