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
    pub sos_membership: bool,
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
            "prepared_created",
            "prepared_spell_cast",
            "increment_trigger",
            "repartee_trigger",
            "opus_trigger",
            "infusion_copy",
            "paradigm_enrolled",
            "paradigm_copy_cast",
            "cascade_cast",
            "miracle_cast",
            "storm_copy",
            "casualty_paid_surviving_grantor",
            "casualty_copy",
            "flashback_cast",
            "converge_entry",
            "grandeur_paid",
            "crew_animated",
            "landfall_trigger",
            "affinity_reduced_cast",
            "surveil_completed",
            "mill_resolved",
            "treasure_created",
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
                    // Retain naturally reached Infusion and paid Grandeur witnesses
                    // found by the unchanged policy, while keeping seeds unique.
                    676200
                        + pair as u64 * 100
                        + match (pair, seed) {
                            (4, 3) => 15,
                            (4, 15) => 3,
                            (5, 3) => 11,
                            (5, 11) => 3,
                            _ => seed,
                        },
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
    let source: serde_json::Value = serde_json::from_str(include_str!("../../data/sets/sos.json"))
        .map_err(|e| e.to_string())?;
    let members: BTreeSet<&str> = source["cards"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["oracle_id"].as_str().unwrap())
        .collect();
    if members.len() != 271 {
        return Err(format!("SOS denominator changed: {}", members.len()));
    }
    for row in source["cards"].as_array().unwrap() {
        let name = row["name"].as_str().unwrap();
        let c = Card::from_name(name).ok_or_else(|| format!("missing physical member {name}"))?;
        if !c.is_supported() {
            return Err(format!("gated SOS member {name}"));
        }
    }
    let specifications = [
        (
            "Silverquill",
            ["Plains", "Swamp"],
            [
                "Informed Inkwright",
                "Adventurous Eater",
                "Inkling Mascot",
                "Imperious Inkmage",
                "Silverquill, the Disputant",
                "Last Gasp",
                "Dig Site Inventory",
                "Interjection",
            ],
        ),
        (
            "Prismari",
            ["Island", "Mountain"],
            [
                "Deluge Virtuoso",
                "Muse Seeker",
                "Exhibition Tidecaller",
                "Spectacular Skywhale",
                "Prismari, the Inspiration",
                "Rapturous Moment",
                "Goblin Glasswright",
                "Echocasting Symposium",
            ],
        ),
        (
            "Quandrix",
            ["Forest", "Island"],
            [
                "Cuboid Colony",
                "Textbook Tabulator",
                "Pterafractyl",
                "Fractalize",
                "Quandrix, the Proof",
                "Tam, Observant Sequencer",
                "Rancorous Archaic",
                "Germination Practicum",
            ],
        ),
        (
            "Witherbloom",
            ["Swamp", "Forest"],
            [
                "Moseo, Vein's New Dean",
                "Thornfist Striker",
                "Old-Growth Educator",
                "Efflorescence",
                "Lumaret's Favor",
                "Oracle's Restoration",
                "Decorum Dissertation",
                "Adventurous Eater",
            ],
        ),
        (
            "Lorehold",
            ["Mountain", "Plains"],
            [
                "Lorehold, the Historian",
                "Startled Relic Sloth",
                "Stirring Hopesinger",
                "Interjection",
                "Dig Site Inventory",
                "Daydream",
                "Restoration Seminar",
                "Shattered Acolyte",
            ],
        ),
        (
            "Campus",
            ["Swamp", "Forest"],
            [
                "Page, Loose Leaf",
                "Strixhaven Skycoach",
                "Rancorous Archaic",
                "Wildgrowth Archaic",
                "Witherbloom, the Balancer",
                "Snarl Song",
                "Together as One",
                "Moseo, Vein's New Dean",
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
                return Err(format!("non-SOS workload card {}", c.name()));
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
            sos_membership: true,
        };
        out.push(Deck { cards, report });
    }
    Ok((out, members.len()))
}
#[derive(Default)]
struct Tracker {
    seen_packets: BTreeSet<u64>,
    seen_events: BTreeSet<String>,
    prepared_copies: BTreeSet<ObjectRef>,
    copy_causes: BTreeMap<u64, &'static str>,
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
    fn cause(s: &SpellView) -> Option<&'static str> {
        if s.kind == StackKind::CasualtyCopy {
            Some("casualty_copy")
        } else if s
            .attachment_trigger
            .as_ref()
            .is_some_and(|g| g.granting_card.name() == "Prismari, the Inspiration")
        {
            Some("storm_copy")
        } else {
            None
        }
    }
    fn inspect(&mut self, before: &Observation, after: &Observation, action: &Action) {
        for row in &after.preparations {
            self.prepared_copies.insert(row.spell_copy);
            self.hit("prepared_created", format!("{:?}", row.spell_copy));
        }
        for row in &after.paradigms {
            self.hit(
                "paradigm_enrolled",
                format!("{:?}/{}", row.controller, row.name),
            );
        }
        if let Some(copy) = &after.retarget {
            let cause = after
                .resolving
                .as_ref()
                .or(before.resolving.as_ref())
                .or_else(|| before.stack.last())
                .and_then(Self::cause);
            if let Some(cause) = cause {
                self.copy_causes.insert(copy.item.id, cause);
            }
        }
        let immediate_cause = before
            .resolving
            .as_ref()
            .or_else(|| before.stack.last())
            .and_then(Self::cause);
        for s in after.stack.iter().chain(after.resolving.iter()) {
            if !self.seen_packets.insert(s.id) {
                continue;
            }
            if s.kind == StackKind::Spell {
                if let Some(cast) = s.cast.filter(|_| !s.is_copy) {
                    self.cards.insert(s.source_card.name().into());
                    self.colors |= s.source_card.colors();
                    self.hit("actual_cast", s.id.to_string());
                    if s.source_card.name() == "Witherbloom, the Balancer"
                        && cast.method == CastMethod::Normal
                        && cast.mana_spent() < 8
                        && before.objects.iter().any(|o| {
                            o.zone == Zone::Battlefield
                                && o.controller == cast.caster
                                && o.kinds & 1 != 0
                        })
                    {
                        self.hit("affinity_reduced_cast", s.id.to_string());
                    }
                    if matches!(
                        cast.method,
                        CastMethod::Flashback | CastMethod::GrantedFlashback
                    ) {
                        self.hit("flashback_cast", s.id.to_string());
                    }
                    if cast.method == CastMethod::Miracle {
                        self.hit("miracle_cast", s.id.to_string());
                        if cast.caster != after.active {
                            self.hit("miracle_opponent_turn", s.id.to_string());
                        }
                    }
                    if cast.origin == Zone::Exile
                        && self.prepared_copies.iter().any(|copy| {
                            copy.card == s.object.card
                                && copy.generation.checked_add(1) == Some(s.object.generation)
                        })
                    {
                        self.hit("prepared_spell_cast", s.id.to_string());
                    }
                    if cast.origin == Zone::Exile && before.cascade.is_some() {
                        self.hit("cascade_cast", s.id.to_string());
                    }
                    if cast.method == CastMethod::Free
                        && cast.origin == Zone::Exile
                        && before.resolving.as_ref().is_some_and(|packet| {
                            matches!(packet.kind, StackKind::TriggeredAbility { .. })
                                && packet.source_card == s.source_card
                        })
                        && before.cascade.is_none()
                        && [
                            "Germination Practicum",
                            "Echocasting Symposium",
                            "Decorum Dissertation",
                            "Restoration Seminar",
                            "Improvisation Capstone",
                        ]
                        .contains(&s.source_card.name())
                    {
                        self.hit("paradigm_copy_cast", s.id.to_string());
                    }
                } else if s.is_copy {
                    self.hit("uncast_copy", s.id.to_string());
                    if s.source_card.name() == "Lumaret's Favor" {
                        self.hit("infusion_copy", s.id.to_string());
                    }
                    if let Some(cause) = self.copy_causes.get(&s.id).copied().or(immediate_cause) {
                        self.hit(cause, s.id.to_string());
                    }
                }
            }
            if s.kind == (StackKind::ActivatedAbility { ability: 1 })
                && s.source_card.name() == "Page, Loose Leaf"
            {
                self.hit("grandeur_paid", s.id.to_string());
            }
            if let StackKind::TriggeredAbility { ability } = s.kind {
                let key = match (s.source_card.name(), ability) {
                    ("Cuboid Colony", 0) | ("Textbook Tabulator", 1) => Some("increment_trigger"),
                    ("Informed Inkwright", 0)
                    | ("Inkling Mascot", 0)
                    | ("Stirring Hopesinger", 0) => Some("repartee_trigger"),
                    ("Deluge Virtuoso", 1)
                    | ("Muse Seeker", 0)
                    | ("Elemental Mascot", 0)
                    | ("Spectacular Skywhale", 0) => Some("opus_trigger"),
                    (name, 0) if name.starts_with("Tam, Observant Sequencer") => {
                        Some("landfall_trigger")
                    }
                    _ => None,
                };
                if let Some(key) = key {
                    self.hit(key, s.id.to_string());
                }
            }
        }
        if let Action::PayCasualty { object } = action {
            for grant in &before.casualty_declarations {
                if grant.pay
                    && !grant.paid
                    && grant.grant.source != *object
                    && after
                        .objects
                        .iter()
                        .any(|o| o.reference == grant.grant.source && o.zone == Zone::Battlefield)
                    && !after
                        .objects
                        .iter()
                        .any(|o| o.reference == *object && o.zone == Zone::Battlefield)
                {
                    self.hit(
                        "casualty_paid_surviving_grantor",
                        format!("{:?}/{:?}", object, grant.grant.source),
                    );
                }
            }
        }
        for player in [Player::P0, Player::P1] {
            let i = player.index();
            if after.players[i].life_gain_events_this_turn
                > before.players[i].life_gain_events_this_turn
            {
                self.hit(
                    "life_gain",
                    format!(
                        "{}/{i}/{}",
                        after.turn, after.players[i].life_gain_events_this_turn
                    ),
                );
            }
            if after.players[i].card_left_graveyard_this_turn {
                self.hit("graveyard_departure", format!("{}/{i}", after.turn));
            }
        }
        if before
            .library_choice
            .as_ref()
            .is_some_and(|choice| choice.surveil)
            && after.library_choice.is_none()
            && matches!(action, Action::ArrangeLibrary { .. })
        {
            self.hit(
                "surveil_completed",
                format!("{}/{:?}/{}", before.turn, before.viewer, before.decision),
            );
        }
        let mill_packet = before.resolving.as_ref().or_else(|| before.stack.last());
        if mill_packet.is_some_and(|packet| {
            packet.source_card.name() == "Exhibition Tidecaller"
                && matches!(packet.kind, StackKind::TriggeredAbility { .. })
        }) && [Player::P0, Player::P1].into_iter().any(|player| {
            after.players[player.index()].library_count
                < before.players[player.index()].library_count
        }) && after.graveyard_entries_this_turn.len() > before.graveyard_entries_this_turn.len()
        {
            self.hit("mill_resolved", mill_packet.unwrap().id.to_string());
        }
        for object in &after.objects {
            if object.token.as_ref().is_some_and(|token| {
                token
                    .subtypes
                    .split_whitespace()
                    .any(|subtype| subtype == "Treasure")
            }) && object.zone == Zone::Battlefield
            {
                self.hit("treasure_created", format!("{:?}", object.reference));
            }
            if object.name == "Strixhaven Skycoach"
                && object.zone == Zone::Battlefield
                && object.kinds & 1 != 0
            {
                self.hit(
                    "crew_animated",
                    format!("{}/{:?}", after.turn, object.reference),
                );
            }
            if object.name == "Rancorous Archaic" && object.zone == Zone::Battlefield {
                for spell in &before.stack {
                    if spell.card == object.id
                        && let Some(cast) = spell.cast
                        && cast.colors_spent() >= 2
                        && object.counters.plus_one >= u64::from(cast.colors_spent())
                    {
                        self.hit("converge_entry", format!("{:?}", object.reference));
                    }
                }
            }
            if object.name == "Thornfist Striker"
                && object.zone == Zone::Battlefield
                && object.keywords.contains(Keywords::TRAMPLE)
                && after.players[object.controller.index()].life_gained_this_turn > 0
            {
                self.hit(
                    "infusion_static",
                    format!("{}/{:?}", after.turn, object.reference),
                );
            }
        }
        if after.library_choice.is_some() {
            self.hit(
                "private_library_choice",
                format!("{}/{:?}/{}", after.turn, after.viewer, after.decision),
            );
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
        let mut owned_branches = 0;
        let mut hash = 0xcbf29ce484222325u64;
        let mut max_bytes = game.state_bytes();
        while game.outcome().is_none() {
            if decisions >= 8192 {
                return Err(format!(
                    "nonterminal budget pair{pair}/seed{seed},turn{} phase{}",
                    game.observe(game.actor().unwrap()).turn,
                    game.observe(game.actor().unwrap()).decision
                ));
            }
            game.legal_actions_into(&mut actions);
            let index = crate_policy(&game, &actions);
            let action = actions[index].clone();
            let actor = game.actor().unwrap();
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
            trace_digest: mode.verifies().then(|| format!("{hash:016x}")),
            trace_file,
            cards_cast: tracker.cards,
            witnesses: tracker.witnesses,
            max_sampled_state_bytes: max_bytes,
        });
    }
    let seconds = started.elapsed().as_secs_f64();
    Ok(Report {
        workload: "SOS_COLLEGES_V1",
        catalogue: CATALOGUE_VERSION,
        mode: if mode.verifies() {
            "verification"
        } else {
            "benchmark"
        },
        scope: "Six exactly 60-card Standard decks; physical SOS members only; max four nonbasic copies, 28 basic lands; natural terminals; existing observation-only smoke policy",
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
            "Includes game construction, rules, action emission, shared smoke policy and observation telemetry. Excludes replay/owned checks, trace serialization, model inference, tensors and network. NamedSOSworkload only; not allStandard or XMage parity"
        },
    })
}
fn crate_policy(game: &Game, actions: &[Action]) -> usize {
    super::smoke_index(game, actions)
}
