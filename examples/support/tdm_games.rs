//! Frozen TDM-only games. Policy sees only its actor observation; private reads
//! below belong exclusively to outcome verification and labelled telemetry.
use rustmage::*;
use rustmage_core::cards::{ActivationOrigin, CastFilter, Instruction, TriggerCondition};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::time::Instant;

pub const WORKLOAD: &str = "TDM_FROZEN_TERMINAL_V1";
pub const DECKS_SHA256: &str = "2daf20c80bb409b7f241680b990b39a1177101dd45e35eaa48fffc913d5fb12e";
pub const REQUIRED: [&str; 8] = [
    "endure",
    "flurry_or_prowess",
    "mobilize",
    "harmonize_cast_and_exile",
    "harmonize_power_contribution",
    "renew",
    "omen_resolution_shuffle",
    "behold_payoff",
];
const LIMIT: usize = 4096;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Verify,
    Benchmark,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DeckSpec {
    pub name: String,
    pub cards: BTreeMap<String, usize>,
}
#[derive(Deserialize)]
struct Inputs {
    schema: u32,
    standard_decks: Vec<DeckSpec>,
    freeform_cutter_deck: DeckSpec,
    standard_pairings: Vec<[String; 2]>,
    standard_seeds: Vec<u64>,
    starting_players: Vec<Player>,
    deck_swaps: Vec<bool>,
    standard_games: usize,
    freeform_seed: u64,
    freeform_opponent: String,
    freeform_games: usize,
    max_actions_per_game: usize,
    required_natural_mechanics: Vec<String>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Case {
    pub pair: usize,
    pub seed: u64,
    pub starting_player: Player,
    pub deck_swapped: bool,
    pub freeform: bool,
}
impl Case {
    pub fn id(self) -> String {
        format!(
            "{}-p{}-seed{}-start{}-swap{}",
            if self.freeform {
                "freeform"
            } else {
                "standard"
            },
            self.pair,
            self.seed,
            self.starting_player.index(),
            u8::from(self.deck_swapped)
        )
    }
}
pub struct Workload {
    inputs: Inputs,
    cards: BTreeMap<String, Vec<Card>>,
    pub cases: Vec<Case>,
    proof_ledger_sha256: String,
}
impl Workload {
    /// This is a refusal gate, never a skip or an admission override.
    pub fn load() -> Result<Self, String> {
        let bytes = include_bytes!("tdm_game_decks.json");
        ensure(
            sha256(bytes) == DECKS_SHA256,
            "frozen TDM deck/seed file SHA256 changed",
        )?;
        let inputs: Inputs = serde_json::from_slice(bytes).map_err(err)?;
        ensure(
            inputs.schema == 1
                && inputs.standard_decks.len() == 6
                && inputs.standard_pairings.len() == 6
                && inputs.standard_seeds.len() == 12
                && inputs.starting_players == [Player::P0, Player::P1]
                && inputs.deck_swaps == [false, true]
                && inputs.standard_games == 48
                && inputs.freeform_games == 4
                && inputs.freeform_seed == 426900
                && inputs.max_actions_per_game == LIMIT,
            "frozen workload dimensions changed",
        )?;
        ensure(
            inputs.required_natural_mechanics == REQUIRED,
            "required natural mechanic matrix changed",
        )?;
        let manifest: serde_json::Value =
            serde_json::from_str(include_str!("../../data/sets/tdm.json")).map_err(err)?;
        let rows = manifest["cards"]
            .as_array()
            .ok_or("TDM physical inventory missing")?;
        ensure(
            rows.len() == 276 && manifest["card_count"] == 276,
            "physical TDM denominator must be276",
        )?;
        let mut members = BTreeMap::new();
        let mut ids = BTreeSet::new();
        let mut gated = Vec::new();
        for row in rows {
            let name = row["name"].as_str().ok_or("physical name missing")?;
            let oracle = row["oracle_id"]
                .as_str()
                .ok_or("physical Oracle ID missing")?;
            ensure(
                members.insert(name.to_owned(), oracle.to_owned()).is_none() && ids.insert(oracle),
                "duplicate physical name/Oracle identity",
            )?;
            match Card::from_name(name) {
                Some(c)
                    if c.identity() == name
                        && c.info().oracle_id == Some(oracle)
                        && c.is_supported() => {}
                _ => gated.push(name),
            }
        }
        let mut omen_faces = BTreeSet::new();
        for name in members.keys() {
            if let Some(parent) = Card::from_name(name)
                && let Some(face) = parent.omen_spell()
            {
                ensure(
                    face.is_supported() && face.info().oracle_id == parent.info().oracle_id,
                    "an associated Omen face is incomplete or has a mismatched Oracle identity",
                )?;
                ensure(
                    omen_faces.insert(face.identity()),
                    "duplicate physical Omen face association",
                )?;
            }
        }
        ensure(
            omen_faces.len() == 13,
            "TDM must retain exactly13 complete paired Omen faces",
        )?;
        if !gated.is_empty() {
            return Err(format!(
                "TDM full-pool gate REFUSED: {}/276 missing, mismatched or incomplete physical identities: {}",
                gated.len(),
                gated.join(", ")
            ));
        }
        // Candidate admission is insufficient. Runtime loading keeps compile-only
        // checks possible in partial worktrees; absent/partial outcome proof refuses.
        let proof_path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/tdm-card-integration-evidence.json");
        let proof_bytes = std::fs::read(&proof_path).map_err(|e| {
            format!(
                "TDM printed-outcome proof gate REFUSED: {}: {e}",
                proof_path.display()
            )
        })?;
        let proof: serde_json::Value = serde_json::from_slice(&proof_bytes).map_err(err)?;
        ensure(
            proof["locally_verified_cards"].as_u64() == Some(276)
                && proof["remaining_physical_cards"].as_u64() == Some(0),
            "TDM printed-outcome proof gate REFUSED:276 locally verified physical cards and0 remaining required",
        )?;
        let proof_ledger_sha256 = sha256(&proof_bytes);
        let cutter = Card::from_name("Cori-Steel Cutter").ok_or("Cutter missing")?;
        ensure(
            cutter.info().oracle_id == Some("46b1a169-bc32-47bf-b379-fa0c3d13878f")
                && !cutter.info().standard_legal,
            "Cutter must remain banned in Standard and present in whole-pool behavior",
        )?;
        let mut cards = BTreeMap::new();
        for spec in inputs
            .standard_decks
            .iter()
            .chain(std::iter::once(&inputs.freeform_cutter_deck))
        {
            ensure(
                spec.cards.values().sum::<usize>() == 60,
                "frozen deck must contain exactly60 cards",
            )?;
            let mut deck = Vec::new();
            for (name, &count) in &spec.cards {
                let oracle = members
                    .get(name)
                    .ok_or_else(|| format!("outside/face-only deck entry {name}"))?;
                let c =
                    Card::from_name(name).ok_or_else(|| format!("unknown deck entry {name}"))?;
                ensure(
                    count > 0
                        && c.identity() == name
                        && c.info().oracle_id == Some(oracle.as_str()),
                    "deck entry is not a canonical physical Oracle identity",
                )?;
                ensure(
                    c.info().type_line.starts_with("Basic Land") || count <= 4,
                    "more than four nonbasic copies",
                )?;
                if spec.name != inputs.freeform_cutter_deck.name {
                    ensure(
                        c.info().standard_legal && c != cutter,
                        "nonlegal card in frozen Standard deck",
                    )?;
                }
                deck.extend(std::iter::repeat_n(c, count));
            }
            ensure(
                cards.insert(spec.name.clone(), deck).is_none(),
                "duplicate deck name",
            )?;
        }
        let mut cases = Vec::new();
        for pair in 0..6 {
            for n in 0..2 {
                let seed = inputs.standard_seeds[pair * 2 + n];
                ensure(
                    seed == 426000 + pair as u64 * 100 + n as u64,
                    "frozen seeds changed",
                )?;
                for &starting_player in &inputs.starting_players {
                    for &deck_swapped in &inputs.deck_swaps {
                        cases.push(Case {
                            pair,
                            seed,
                            starting_player,
                            deck_swapped,
                            freeform: false,
                        });
                    }
                }
            }
        }
        for &starting_player in &inputs.starting_players {
            for &deck_swapped in &inputs.deck_swaps {
                cases.push(Case {
                    pair: 6,
                    seed: inputs.freeform_seed,
                    starting_player,
                    deck_swapped,
                    freeform: true,
                });
            }
        }
        ensure(
            cases.len() == 52 && cases.iter().map(|c| c.id()).collect::<BTreeSet<_>>().len() == 52,
            "not exactly52 distinct frozen cases",
        )?;
        let w = Self {
            inputs,
            cards,
            cases,
            proof_ledger_sha256,
        };
        // Real Standard validation supplies legality/deck-size/limit validation too.
        for case in &w.cases {
            Game::new(w.config(*case)?)
                .map_err(|e| format!("real deck validation {}: {e:?}", case.id()))?;
        }
        let mut banned = w.config(w.cases[48])?;
        banned.deck_rules = DeckRules::Standard;
        ensure(
            matches!(Game::new(banned),Err(Error::InvalidDeck(reason)) if reason.contains("Cori-Steel Cutter")),
            "Cutter60 variant must fail Standard for Cutter itself",
        )?;
        Ok(w)
    }
    pub fn config(&self, case: Case) -> Result<Config, String> {
        ensure(self.cases.contains(&case), "case outside frozen corpus")?;
        let (a, b) = if case.freeform {
            (
                &self.inputs.freeform_cutter_deck.name,
                &self.inputs.freeform_opponent,
            )
        } else {
            let p = &self.inputs.standard_pairings[case.pair];
            (&p[0], &p[1])
        };
        let (a, b) = if case.deck_swapped { (b, a) } else { (a, b) };
        Ok(Config {
            decks: [
                self.cards.get(a).ok_or("missing deck")?.clone(),
                self.cards.get(b).ok_or("missing deck")?.clone(),
            ],
            seed: case.seed,
            starting_player: case.starting_player,
            deck_rules: if case.freeform {
                DeckRules::Freeform
            } else {
                DeckRules::Standard
            },
        })
    }
    pub fn pair(&self, pair: usize) -> Vec<Case> {
        self.cases
            .iter()
            .copied()
            .filter(|c| c.pair == pair)
            .collect()
    }
    pub fn standard(&self) -> Vec<Case> {
        self.cases.iter().copied().filter(|c| !c.freeform).collect()
    }
    pub fn freeform(&self) -> Vec<Case> {
        self.cases.iter().copied().filter(|c| c.freeform).collect()
    }
}
fn ensure(ok: bool, message: &str) -> Result<(), String> {
    if ok { Ok(()) } else { Err(message.into()) }
}
fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}
pub fn evidence_dir() -> Result<PathBuf, String> {
    std::env::var_os("RUSTMAGE_TDM_EVIDENCE_DIR")
        .map(PathBuf::from)
        .ok_or_else(|| {
            "set RUSTMAGE_TDM_EVIDENCE_DIR to a fresh persistent directory under the cache".into()
        })
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Witness {
    pub mechanic: String,
    pub decision: usize,
    pub packet: Option<u64>,
    pub source: Option<String>,
    pub before: serde_json::Value,
    pub after: serde_json::Value,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Recipient {
    pub object: ObjectRef,
    pub colors: u8,
    pub before: u64,
    pub after: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TerminalCause {
    pub kind: String,
    pub losers: Vec<Player>,
    pub life: [i128; 2],
    pub poison: [u64; 2],
    pub attempted_empty_draw: [bool; 2],
    pub call_packet: Option<u64>,
    pub positive_distinct_dragons: Vec<Recipient>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameReport {
    pub case: Case,
    pub case_id: String,
    pub repetition: usize,
    pub decisions: usize,
    pub turns: u32,
    #[serde(deserialize_with = "deserialize_outcome")]
    pub outcome: Outcome,
    pub cause: TerminalCause,
    pub cards_cast: BTreeSet<String>,
    pub spell_colors: u8,
    pub cast_count: usize,
    pub meaningful_interactions: usize,
    pub owned_decision_roots: Vec<String>,
    pub replayed_actions: usize,
    pub replay_seconds: f64,
    pub witnesses: BTreeMap<String, u64>,
    pub raw_trace_sha256: Option<String>,
    pub trace_file: Option<String>,
    pub final_snapshot_debug_sha256: Option<String>,
    pub final_observations_sha256: Option<String>,
    pub rejected_attack_attempts: usize,
    pub measured_seconds: f64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Report {
    pub workload: String,
    pub catalogue: String,
    pub deck_source_sha256: String,
    pub harness_source_sha256: String,
    pub printed_outcome_ledger_sha256: String,
    pub physical_manifest_sha256: String,
    pub mode: String,
    pub scope: String,
    pub physical_members: usize,
    pub standard_cases: usize,
    pub freeform_cases: usize,
    pub repetitions: usize,
    pub decks: Vec<DeckSpec>,
    pub games: Vec<GameReport>,
    pub witnesses: BTreeMap<String, u64>,
    pub actual_spell_colors: u8,
    pub decisions: usize,
    pub measured_seconds: f64,
    pub games_per_second: f64,
    pub decisions_per_second: f64,
    pub measurement: String,
}
impl Report {
    pub fn check_complete_standard(&self, w: &Workload) -> Result<(), String> {
        ensure(
            self.mode == "verification"
                && self.workload == WORKLOAD
                && self.catalogue == CATALOGUE_VERSION
                && self.deck_source_sha256 == DECKS_SHA256
                && self.harness_source_sha256 == harness_sha256()
                && self.printed_outcome_ledger_sha256 == w.proof_ledger_sha256
                && self.physical_manifest_sha256 == physical_manifest_sha256(),
            "receipt belongs to different executable/workload or benchmark",
        )?;
        let actual: BTreeSet<_> = self
            .games
            .iter()
            .filter(|g| !g.case.freeform)
            .map(|g| g.case_id.clone())
            .collect();
        let expected: BTreeSet<_> = w.standard().iter().map(|c| c.id()).collect();
        ensure(
            actual == expected && self.games.iter().filter(|g| !g.case.freeform).count() == 48,
            "receipt must contain exactly48 unique Standard games",
        )?;
        let freeform_actual: BTreeSet<_> = self
            .games
            .iter()
            .filter(|g| g.case.freeform)
            .map(|g| g.case_id.clone())
            .collect();
        let freeform_expected: BTreeSet<_> = w.freeform().iter().map(|c| c.id()).collect();
        ensure(
            freeform_actual == freeform_expected
                && self.games.iter().filter(|g| g.case.freeform).count() == 4,
            "receipt must retain exactly4 separate unique Cutter Freeform games",
        )?;
        ensure(
            self.games.len() == 52
                && self.standard_cases == 48
                && self.freeform_cases == 4
                && self.repetitions == 1
                && self.physical_members == 276,
            "verification corpus dimensions must be exactly48Standard+4Freeform, one repetition,276members",
        )?;
        let standard_colors = self
            .games
            .iter()
            .filter(|g| !g.case.freeform)
            .fold(0, |colors, g| colors | g.spell_colors);
        ensure(
            standard_colors == 31,
            "actual Standard selected-face casts must cover all five colors",
        )?;
        for name in REQUIRED {
            ensure(
                self.games
                    .iter()
                    .filter(|g| !g.case.freeform)
                    .any(|g| g.witnesses.get(name).copied().unwrap_or(0) > 0),
                &format!(
                    "missing actual natural payoff: {name}; offered choices or queued triggers do not count"
                ),
            )?;
        }
        for game in &self.games {
            authenticate_game(w, game)?;
            ensure(
                game.replayed_actions == game.decisions
                    && game.cast_count > 0
                    && game.meaningful_interactions > 0
                    && game
                        .witnesses
                        .get("actual_rules_payoff")
                        .copied()
                        .unwrap_or(0)
                        > 0
                    && !game.owned_decision_roots.is_empty()
                    && game.owned_decision_roots.len() <= 5,
                "missing bounded owned/replay proof",
            )?;
        }
        Ok(())
    }
}

#[derive(Clone)]
struct Obj {
    object: ObjectRef,
    card: Card,
    zone: Zone,
    controller: Player,
    tapped: bool,
    token: bool,
    attacking: bool,
    power: i128,
    toughness: i128,
    damage: i128,
    plus: u64,
    keywords: Keywords,
}
#[derive(Clone)]
struct Frame {
    objects: Vec<Obj>,
    life: [i128; 2],
    draws: [u64; 2],
    packets: Vec<SpellView>,
    top: Option<u64>,
    resolving: Option<u64>,
    active: Player,
    step: Step,
}
impl Frame {
    fn read(g: &Game, view: &Observation) -> Self {
        let visible: BTreeMap<_, _> = view.objects.iter().map(|o| (o.id, o)).collect();
        let objects = g
            .state
            .objects
            .iter()
            .enumerate()
            .map(|(id, o)| {
                let id = id as CardId;
                let v = visible.get(&id);
                Obj {
                    object: g.object_ref(id),
                    card: o.card,
                    zone: o.zone,
                    controller: o.controller,
                    tapped: o.tapped,
                    token: o.is_token(),
                    attacking: v.is_some_and(|v| v.attacking),
                    power: v.map_or(0, |v| v.power),
                    toughness: v.map_or(0, |v| v.toughness),
                    damage: o.damage,
                    plus: v.map_or(0, |v| v.counters.get(CounterKind::PlusOne)),
                    keywords: v.map_or(Keywords::NONE, |v| v.keywords),
                }
            })
            .collect();
        let mut packets = g.state.stack.to_vec();
        if let Some(r) = &g.state.resolving {
            packets.push(r.item.clone());
        }
        Self {
            objects,
            life: std::array::from_fn(|i| g.state.players[i].life),
            draws: std::array::from_fn(|i| g.state.players[i].cards_drawn_this_turn),
            packets,
            top: g.state.stack.last().map(|s| s.id),
            resolving: g.state.resolving.as_ref().map(|r| r.item.id),
            active: g.state.active,
            step: g.state.step,
        }
    }
    fn exact(&self, r: ObjectRef) -> Option<&Obj> {
        self.objects.get(r.card as usize).filter(|o| o.object == r)
    }
    fn has_packet(&self, id: u64) -> bool {
        self.packets.iter().any(|p| p.id == id)
    }
}
#[derive(Default)]
struct Basic {
    seen: BTreeSet<u64>,
    cards: BTreeSet<String>,
    colors: u8,
    casts: usize,
}
impl Basic {
    fn inspect(&mut self, g: &Game) {
        for p in g
            .state
            .stack
            .iter()
            .chain(g.state.resolving.iter().map(|r| &r.item))
        {
            if p.kind == StackKind::Spell
                && !p.is_copy
                && p.cast.is_some()
                && self.seen.insert(p.id)
                && let Some(s) = p.source()
            {
                self.cards.insert(s.card.name().into());
                self.colors |= s.card.colors();
                self.casts += 1;
            }
        }
    }
}
struct ResolutionProof {
    item: SpellView,
    before: Frame,
    endure: Option<(ObjectRef, u128, bool)>,
    behold: bool,
}
struct Harmonizer {
    object: ObjectRef,
    power: i128,
    tapped: bool,
    discount: u64,
    total_generic: Option<u32>,
    without_helper_generic: Option<u32>,
}
#[derive(Default)]
struct CallProof {
    controller: Option<Player>,
    selected: BTreeMap<ObjectRef, (u8, u64)>,
    positive: Vec<Recipient>,
    terminal_decision: Option<usize>,
}
#[derive(Default)]
struct Tracker {
    resolving: BTreeMap<u64, ResolutionProof>,
    harmonizers: BTreeMap<u64, Harmonizer>,
    flash_needed: BTreeSet<u64>,
    calls: BTreeMap<u64, CallProof>,
    receipts: Vec<Witness>,
    counts: BTreeMap<String, u64>,
    meaningful: usize,
    payoffs: usize,
}
impl Tracker {
    fn hit(
        &mut self,
        key: &str,
        decision: usize,
        item: Option<&SpellView>,
        before: serde_json::Value,
        after: serde_json::Value,
    ) {
        *self.counts.entry(key.into()).or_default() += 1;
        self.receipts.push(Witness {
            mechanic: key.into(),
            decision,
            packet: item.map(|p| p.id),
            source: item.and_then(|p| p.source()).map(|s| s.card.name().into()),
            before,
            after,
        });
    }
    fn before(&mut self, g: &Game, view: &Observation, action: &Action, preview_harmonize: bool) {
        // Read receipts only; neither these reads nor any private field affects policy.
        if let (
            Some(a),
            Action::ChooseHarmonizer {
                object: Some(object),
            },
        ) = (&g.state.announcing, action)
        {
            // A read-only verification branch previews the legal no-helper route;
            // no private field is changed, and this cannot affect policy or play.
            let mut without_helper_generic = None;
            if preview_harmonize
                && g.legal_actions()
                    .contains(&Action::ChooseHarmonizer { object: None })
            {
                let mut comparison = g.clone();
                comparison
                    .apply(
                        g.actor().unwrap(),
                        Action::ChooseHarmonizer { object: None },
                    )
                    .expect("offered counterfactual choice");
                without_helper_generic = comparison
                    .state
                    .announcing
                    .as_ref()
                    .and_then(|a| a.total_cost)
                    .map(|c| c.generic);
            }
            self.harmonizers.insert(
                a.item.id,
                Harmonizer {
                    object: *object,
                    power: g.stats(object.card).power,
                    tapped: false,
                    discount: 0,
                    total_generic: None,
                    without_helper_generic,
                },
            );
        }
        if let Some(a) = &g.state.announcing
            && let Some(h) = self.harmonizers.get_mut(&a.item.id)
            && let Some(total) = a.total_cost
        {
            h.total_generic = Some(total.generic);
            h.discount = a.harmonize_discount;
        }
        if let Some(r) = &g.state.resolving {
            if let Some(proof) = self.resolving.get_mut(&r.item.id) {
                if let (Some(e), Action::ChooseEndure { put_counters }) =
                    (&view.endure_choice, action)
                {
                    proof.endure = Some((e.source, e.amount, *put_counters));
                }
                if (view
                    .effect_choice
                    .as_ref()
                    .is_some_and(|c| c.kind == EffectChoiceKind::Behold)
                    || view.hand_choice.as_ref().is_some_and(|c| {
                        matches!(
                            c.kind,
                            HandChoiceKind::Behold | HandChoiceKind::BeholdOptional { .. }
                        )
                    }))
                    && matches!(
                        action,
                        Action::Sacrifice { .. }
                            | Action::ChooseHandCard { object: Some(_) }
                            | Action::ChooseBeholdCost { .. }
                    )
                {
                    proof.behold = true;
                }
            }
            if let Some(choice) = &view.effect_choice
                && let EffectChoiceKind::ColorCounter { color, subtype } = choice.kind
            {
                let object = match action {
                    Action::ChooseColorCounter { object } | Action::Sacrifice { object } => {
                        Some(*object)
                    }
                    _ => None,
                };
                if let Some(object) = object {
                    ensure(
                        g.has_subtype(object.card, subtype)
                            && view.objects.iter().any(|o| {
                                o.reference == object
                                    && o.zone == Zone::Battlefield
                                    && o.controller == r.item.controller
                                    && o.colors & (1 << color.index()) != 0
                            }),
                        "Call actual recipient subtype/color/controller mismatch",
                    )
                    .expect("offered exact subtype");
                    let entry = self.calls.entry(r.item.id).or_default();
                    entry.controller = Some(r.item.controller);
                    entry
                        .selected
                        .entry(object)
                        .or_insert((
                            1 << color.index(),
                            g.counter_count(object.card, CounterKind::PlusOne),
                        ))
                        .0 |= 1 << color.index();
                }
            }
        }
    }
    fn inspect(
        &mut self,
        before: &Frame,
        after: &Frame,
        action: &Action,
        decision: usize,
        g: &Game,
    ) {
        for packet in &after.packets {
            if packet.kind == StackKind::Spell && !packet.is_copy && !before.has_packet(packet.id) {
                if packet.cast.is_some_and(|c| c.beheld == Some(true))
                    && (before.active != packet.controller
                        || !matches!(before.step, Step::Main1 | Step::Main2)
                        || before.top.is_some())
                {
                    self.flash_needed.insert(packet.id);
                }
                if let Some(h) = self.harmonizers.get_mut(&packet.id) {
                    h.tapped = after.exact(h.object).is_some_and(|o| o.tapped);
                }
            }
        }
        // A real top packet began resolving, possibly suspended on an owned choice.
        if matches!(action, Action::Pass)
            && let Some(id) = before.top
            && (after.resolving == Some(id) || !after.has_packet(id))
            && let Some(item) = before.packets.iter().find(|p| p.id == id)
        {
            self.resolving.entry(id).or_insert_with(|| ResolutionProof {
                item: item.clone(),
                before: before.clone(),
                endure: None,
                behold: false,
            });
        }
        let changed_zone = before.objects.iter().zip(&after.objects).any(|(b, a)| {
            b.zone != a.zone
                && ((b.zone == Zone::Battlefield && a.zone != Zone::Stack)
                    || (b.zone == Zone::Graveyard && a.zone != Zone::Stack))
        });
        let damage_changed = before
            .objects
            .iter()
            .any(|b| after.exact(b.object).is_some_and(|a| a.damage != b.damage));
        let declared_combat = matches!(action, Action::FinishAttackers)
            && after
                .objects
                .iter()
                .any(|o| o.zone == Zone::Battlefield && o.attacking);
        if before.life != after.life || changed_zone || damage_changed || declared_combat {
            self.meaningful += 1;
        }
        let combat_damage_window =
            matches!(before.step, Step::FirstStrikeDamage | Step::CombatDamage)
                || matches!(after.step, Step::FirstStrikeDamage | Step::CombatDamage);
        if combat_damage_window
            && before.top.is_none()
            && before.resolving.is_none()
            && (before.life != after.life || damage_changed || changed_zone)
        {
            // Real combat applies the creatures' printed characteristics. A
            // vanilla body needs actual combat, not ordinary entry, as payoff.
            self.payoffs += 1;
            self.hit("actual_rules_payoff",decision,None,
                serde_json::json!({"kind":"actual_combat_damage","life":before.life}),
                serde_json::json!({"life":after.life,"creature_damage_changed":damage_changed,"battlefield_or_graveyard_move":changed_zone}));
        }
        let finished: Vec<_> = self
            .resolving
            .keys()
            .copied()
            .filter(|id| !after.has_packet(*id))
            .collect();
        for id in finished {
            let proof = self.resolving.remove(&id).unwrap();
            self.resolved(proof, after, decision, g);
        }
        for (&packet, call) in &mut self.calls {
            let active_resolution = before.resolving == Some(packet)
                || after.resolving == Some(packet)
                || (before.top == Some(packet) && matches!(action, Action::Pass));
            // Later unrelated counter additions can never repair an earlier
            // unsuccessful Call or supply its alternative-win arithmetic.
            if !active_resolution {
                continue;
            }
            call.positive = call
                .selected
                .iter()
                .filter_map(|(&object, &(colors, count))| {
                    after
                        .exact(object)
                        .filter(|o| {
                            o.zone == Zone::Battlefield
                                && o.controller == call.controller.unwrap()
                                && g.has_subtype(object.card, "Dragon")
                                && o.plus > count
                        })
                        .map(|o| Recipient {
                            object,
                            colors,
                            before: count,
                            after: o.plus,
                        })
                })
                .collect();
            if g.outcome().is_some() && call.positive.len() >= 5 {
                call.terminal_decision = Some(decision);
            }
        }
    }
    fn resolved(&mut self, proof: ResolutionProof, after: &Frame, decision: usize, g: &Game) {
        let item = &proof.item;
        let Some(source) = item.source() else { return };
        let baseline = &proof.before;
        let program = item.program().map(|(_, p)| p).unwrap_or(&[]);
        let new_tokens: Vec<_> = after
            .objects
            .iter()
            .filter(|a| {
                a.zone == Zone::Battlefield
                    && a.token
                    && !baseline
                        .objects
                        .iter()
                        .any(|b| b.object == a.object && b.zone == Zone::Battlefield)
            })
            .collect();
        let stats_changed = baseline.objects.iter().any(|b| {
            b.zone == Zone::Battlefield
                && after.exact(b.object).is_some_and(|a| {
                    a.plus != b.plus
                        || a.power != b.power
                        || a.toughness != b.toughness
                        || a.keywords != b.keywords
                })
        });
        let other_moves = baseline
            .objects
            .iter()
            .zip(&after.objects)
            .any(|(b, a)| b.object.card != source.id && b.zone != a.zone);
        let source_entry = baseline
            .objects
            .get(source.id as usize)
            .is_some_and(|b| b.zone == Zone::Stack)
            && after
                .objects
                .get(source.id as usize)
                .is_some_and(|a| a.zone == Zone::Battlefield);
        let damage_deltas: Vec<_> = baseline
            .objects
            .iter()
            .filter_map(|b| {
                after
                    .exact(b.object)
                    .filter(|a| a.damage != b.damage)
                    .map(|a| (a.object, b.damage, a.damage))
            })
            .collect();
        let payoff = !damage_deltas.is_empty()
            || baseline.life != after.life
            || baseline.draws != after.draws
            || stats_changed
            || other_moves
            || !new_tokens.is_empty();
        if other_moves {
            // A resolved spell/ability's draw/search/return/removal is a real
            // interaction; an ordinary turn draw outside resolution is excluded.
            self.meaningful += 1;
        }
        if payoff {
            self.payoffs += 1;
            self.hit("actual_rules_payoff",decision,Some(item),serde_json::json!({"life":baseline.life,"draws":baseline.draws}),serde_json::json!({"life":after.life,"draws":after.draws,"new_tokens":new_tokens.len(),"stats_or_counters_changed":stats_changed,"other_zone_moves":other_moves,"source_entered":source_entry}));
        }
        if let Some((object, amount, counters)) = proof.endure {
            let old = baseline.exact(object).map_or(0, |o| o.plus);
            let now = after.exact(object).map_or(0, |o| o.plus);
            if amount > 0
                && ((counters && u128::from(now.saturating_sub(old)) >= amount)
                    || (!counters
                        && new_tokens.iter().any(|o| {
                            o.card.has_subtype("Spirit") && o.power > 0 && o.toughness > 0
                        })))
            {
                self.hit("endure",decision,Some(item),serde_json::json!({"recipient":object,"counters":old,"chosen_amount":amount,"counter_branch":counters}),serde_json::json!({"counters":now,"new_tokens":new_tokens.len()}));
            }
        }
        let cast_trigger = if let StackKind::TriggeredAbility { ability } = item.kind {
            source
                .card
                .triggers()
                .get(ability as usize)
                .map(|a| a.condition.event())
        } else {
            None
        };
        let flurry = matches!(
            cast_trigger,
            Some(TriggerCondition::SpellCast {
                ordinal: Some(2),
                ..
            })
        );
        let prowess = matches!(
            cast_trigger,
            Some(TriggerCondition::SpellCast {
                filter: CastFilter::Noncreature,
                ..
            })
        ) && program.iter().any(|i| {
            matches!(
                i,
                Instruction::Grow {
                    power: 1,
                    toughness: 1,
                    ..
                }
            )
        });
        if (flurry || prowess) && payoff {
            self.hit("flurry_or_prowess",decision,Some(item),serde_json::json!({"life":baseline.life,"source":baseline.exact(source.object).map(|o|(o.power,o.toughness,o.plus))}),serde_json::json!({"life":after.life,"source":after.exact(source.object).map(|o|(o.power,o.toughness,o.plus))}));
        }
        if program
            .iter()
            .any(|i| matches!(i, Instruction::Mobilize { .. }))
            && new_tokens.iter().any(|o| {
                o.attacking
                    && o.tapped
                    && o.card.has_subtype("Warrior")
                    && o.power > 0
                    && o.toughness > 0
            })
        {
            self.hit("mobilize",decision,Some(item),serde_json::json!({"new_attacking_warriors":0}),serde_json::json!({"new_attacking_warriors":new_tokens.iter().filter(|o|o.attacking&&o.tapped&&o.card.has_subtype("Warrior")).count()}));
        }
        if let Some(cast) = item.cast {
            if matches!(
                cast.method,
                CastMethod::Harmonize | CastMethod::GrantedHarmonize
            ) && after
                .objects
                .get(source.id as usize)
                .is_some_and(|o| o.zone == Zone::Exile)
            {
                self.hit(
                    "harmonize_cast_and_exile",
                    decision,
                    Some(item),
                    serde_json::json!({"origin":cast.origin,"payment":cast.payment}),
                    serde_json::json!({"source_zone":"Exile"}),
                );
                if let Some(h) = self.harmonizers.remove(&item.id)
                    && h.power > 0
                    && h.discount > 0
                    && h.tapped
                    && h.without_helper_generic
                        .zip(h.total_generic)
                        .is_some_and(|(unreduced, reduced)| unreduced > reduced)
                    && after
                        .objects
                        .get(h.object.card as usize)
                        .is_some_and(|o| o.object == h.object && o.tapped)
                {
                    self.hit("harmonize_power_contribution",decision,Some(item),serde_json::json!({"helper":h.object,"sampled_power":h.power}),serde_json::json!({"actual_discount":h.discount,"final_generic":h.total_generic,"without_helper_generic":h.without_helper_generic,"actual_cast_payment":cast.payment,"helper_tapped":true}));
                }
            }
            if cast.face == CastFace::Omen
                && after
                    .objects
                    .get(source.id as usize)
                    .is_some_and(|o| o.zone == Zone::Library)
            {
                let owner = g.state.objects[source.id as usize].owner;
                if g.state.players[owner.index()].library.contains(&source.id) {
                    self.hit("omen_resolution_shuffle",decision,Some(item),serde_json::json!({"selected_face":source.card.name(),"payment":cast.payment}),serde_json::json!({"physical_parent":g.state.objects[source.id as usize].card.identity(),"owner":owner,"destination":"Library","owner_library_contains_card":true}));
                }
            }
            if cast.beheld == Some(true)
                && payoff
                && source
                    .card
                    .behold_cast()
                    .is_some_and(|b| !b.grants_flash || self.flash_needed.contains(&item.id))
            {
                self.hit("behold_payoff",decision,Some(item),serde_json::json!({"committed_beheld":true,"payment":cast.payment}),serde_json::json!({"life":after.life,"stats_changed":stats_changed,"damage_deltas":damage_deltas,"zone_effect":other_moves}));
            }
        }
        if let StackKind::ActivatedAbility { ability } = item.kind
            && source
                .card
                .activations()
                .get(ability as usize)
                .is_some_and(|a| a.origin == ActivationOrigin::Graveyard && a.cost.exile_self)
            && after
                .objects
                .get(source.id as usize)
                .is_some_and(|o| o.zone == Zone::Exile)
            && payoff
        {
            self.hit(
                "renew",
                decision,
                Some(item),
                serde_json::json!({"activation_source":source.object,"printed_origin":"Graveyard"}),
                serde_json::json!({"source_zone":"Exile","recipient_payoff":true}),
            );
        }
        if proof.behold && payoff {
            self.hit(
                "behold_payoff",
                decision,
                Some(item),
                serde_json::json!({"actual_behold_choice_paid":true}),
                serde_json::json!({"new_tokens":new_tokens.len(),"stats_changed":stats_changed}),
            );
        }
    }
}

fn views(g: &Game) -> Result<serde_json::Value, String> {
    serde_json::to_value([g.observe(Player::P0), g.observe(Player::P1)]).map_err(err)
}
// Restore changes only the external revision epoch by contract. Compare both
// full views with that one epoch normalized; assert stale/revision behavior apart.
fn semantic_views(g: &Game) -> Result<serde_json::Value, String> {
    let mut views = [g.observe(Player::P0), g.observe(Player::P1)];
    for v in &mut views {
        v.revision = 0;
    }
    serde_json::to_value(views).map_err(err)
}
fn root_kind(action: &Action) -> Option<&'static str> {
    match action {
        Action::ChooseEndure { .. } => Some("private_selection"),
        Action::ChooseHarmonizer { object: Some(_) } => Some("harmonize_contribution"),
        Action::ChooseHarmonizer { object: None } => None,
        Action::SelectLibrary { .. }
        | Action::ArrangeLibrary { .. }
        | Action::ChooseSearch { .. }
        | Action::ChooseHandCard { .. } => Some("private_selection"),
        Action::ChooseTarget { .. } | Action::ChooseColorCounter { .. } => Some("recipient_choice"),
        Action::ChooseCastCost { .. }
        | Action::PayCosts { .. }
        | Action::ChooseBeholdCost { .. } => Some("cost_payment"),
        Action::FinishAttackers | Action::AssignDamage { .. } | Action::Block { .. } => {
            Some("combat_choice")
        }
        _ => None,
    }
}
fn owned_root(game: &Game, actor: Player, index: usize, action: &Action) -> Result<(), String> {
    let parent = game.clone();
    let saved = parent.snapshot();
    let revision = parent.revision();
    let before_views = views(&parent)?;
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    let mut restored = parent.clone();
    let mut rejected = parent.clone();
    ensure(
        rejected.apply(actor.other(), action.clone()) == Err(Error::WrongActor)
            && rejected.snapshot() == saved
            && rejected.revision() == revision
            && views(&rejected)? == before_views,
        "wrong-actor rollback failed",
    )?;
    let illegal = Action::ChooseTarget {
        target: Target::Stack(u64::MAX),
    };
    ensure(
        !parent.legal_actions().contains(&illegal),
        "illegal rollback probe accidentally legal",
    )?;
    ensure(
        rejected.apply(actor, illegal) == Err(Error::IllegalAction)
            && rejected.snapshot() == saved
            && rejected.revision() == revision
            && views(&rejected)? == before_views,
        "concrete illegal-action rollback failed",
    )?;
    ensure(
        parent.snapshot() == saved
            && parent.revision() == revision
            && views(&parent)? == before_views,
        "branching mutated parent",
    )?;
    drop(parent);
    // Both owned children continue after their parent has actually been dropped.
    raw.apply(actor, action.clone())
        .map_err(|e| format!("owned raw child {e:?}"))?;
    indexed
        .apply_index(actor, revision, index)
        .map_err(|e| format!("owned indexed child {e:?}"))?;
    ensure(
        raw.snapshot() == indexed.snapshot()
            && raw.outcome() == indexed.outcome()
            && views(&raw)? == views(&indexed)?,
        "owned raw/index children or paired views differ",
    )?;
    restored
        .apply(actor, action.clone())
        .map_err(|e| format!("restore precursor {e:?}"))?;
    restored
        .restore(&saved)
        .map_err(|e| format!("restore {e:?}"))?;
    let restored_revision = restored.revision();
    ensure(
        restored.apply_index(actor, revision, index) == Err(Error::StaleRevision)
            && restored.snapshot() == saved
            && restored.revision() == restored_revision
            && semantic_views(&restored)? == semantic_views(game)?,
        "stale-after-restore did not reject transactionally",
    )?;
    restored
        .apply(actor, action.clone())
        .map_err(|e| format!("restored raw replay {e:?}"))?;
    ensure(
        restored.snapshot() == raw.snapshot()
            && restored.outcome() == raw.outcome()
            && semantic_views(&restored)? == semantic_views(&raw)?,
        "restored raw replay or paired views differ",
    )?;
    ensure(
        game.snapshot() == saved && game.revision() == revision && views(game)? == before_views,
        "owned verification mutated live game",
    )
}
fn terminal(g: &Game, tracker: Option<&Tracker>, decision: usize) -> Result<TerminalCause, String> {
    let life = std::array::from_fn(|i| g.state.players[i].life);
    let poison = std::array::from_fn(|i| g.state.players[i].poison_counters);
    let attempted_empty_draw = std::array::from_fn(|i| g.state.players[i].attempted_empty_draw);
    let losers: Vec<_> = [Player::P0, Player::P1]
        .into_iter()
        .filter(|p| {
            life[p.index()] <= 0 || poison[p.index()] >= 10 || attempted_empty_draw[p.index()]
        })
        .collect();
    let normal = match (g.outcome(), losers.as_slice()) {
        (Some(Outcome::Winner(w)), [loser]) => w == loser.other(),
        (Some(Outcome::Draw), [_, _]) => true,
        _ => false,
    };
    if normal {
        let mut reasons = Vec::new();
        if life.iter().any(|&n| n <= 0) {
            reasons.push("life_at_or_below_zero");
        }
        if poison.iter().any(|&n| n >= 10) {
            reasons.push("ten_poison");
        }
        if attempted_empty_draw.iter().any(|&n| n) {
            reasons.push("attempted_empty_draw");
        }
        return Ok(TerminalCause {
            kind: reasons.join("+"),
            losers,
            life,
            poison,
            attempted_empty_draw,
            call_packet: None,
            positive_distinct_dragons: Vec::new(),
        });
    }
    if let (Some(Outcome::Winner(w)), Some(t)) = (g.outcome(), tracker) {
        for (&packet, call) in &t.calls {
            if call.controller == Some(w)
                && call.terminal_decision == Some(decision)
                && call.positive.len() == 5
                && call
                    .positive
                    .iter()
                    .map(|r| r.object)
                    .collect::<BTreeSet<_>>()
                    .len()
                    == 5
                && call.positive.iter().all(|r| r.after > r.before)
                && call.positive.iter().fold(0, |mask, r| mask | r.colors) == 31
            {
                return Ok(TerminalCause {
                    kind: "call_five_distinct_positive_dragon_recipients".into(),
                    losers,
                    life,
                    poison,
                    attempted_empty_draw,
                    call_packet: Some(packet),
                    positive_distinct_dragons: call.positive.clone(),
                });
            }
        }
    }
    Err(format!(
        "unreconciled natural terminal {:?};life{life:?};poison{poison:?};attempted_empty_draw{attempted_empty_draw:?}; no independent five-recipient Call arithmetic",
        g.outcome()
    ))
}
fn run_one(
    w: &Workload,
    case: Case,
    repetition: usize,
    mode: Mode,
    dir: Option<&Path>,
    verified: Option<&GameReport>,
) -> Result<GameReport, String> {
    let config = w.config(case)?;
    let started = Instant::now();
    let mut game = Game::new(config.clone()).map_err(|e| format!("game init {e:?}"))?;
    let mut actions = Vec::new();
    let mut trace = Vec::new();
    let mut tracker = (mode == Mode::Verify).then(Tracker::default);
    let mut basic = Basic::default();
    let mut rejected_attacks = BTreeSet::new();
    let mut pending_attack = None;
    let mut rejected_attack_attempts = 0;
    let mut roots = BTreeSet::new();
    let mut decisions = 0;
    let result = (|| -> Result<GameReport, String> {
        while game.outcome().is_none() {
            if decisions == LIMIT {
                if let Some(dir) = dir {
                    write_json(
                        &dir.join(format!("{}-failure-trace.json", case.id())),
                        &serde_json::json!({"workload":WORKLOAD,"catalogue":CATALOGUE_VERSION,"deck_source_sha256":DECKS_SHA256,"harness_source_sha256":harness_sha256(),"case":case,"ordered_config":config,"actions":trace,"failure":"4096 actions without natural terminal"}),
                    )?;
                }
                return Err(format!(
                    "{} exhausted4096 actions without a natural terminal; no concession or artificial outcome is permitted",
                    case.id()
                ));
            }
            game.legal_actions_into(&mut actions);
            let actor = game.actor().ok_or("nonterminal game has no actor")?;
            // Exactly the FRA/HOB generic rejected-attack memory. No card-aware filter.
            let eligible: Vec<_> = actions
                .iter()
                .enumerate()
                .filter(|(_, a)| match a {
                    Action::Attack { card } | Action::AttackPlaneswalker { card, .. } => {
                        !rejected_attacks.contains(card)
                    }
                    _ => true,
                })
                .map(|(i, a)| (i, a.clone()))
                .collect();
            ensure(
                !eligible.is_empty(),
                "generic rejection memory filtered every action",
            )?;
            let offered: Vec<_> = eligible.iter().map(|(_, a)| a.clone()).collect();
            let index = eligible[policy::smoke_index(&game, &offered)].0;
            let action = actions[index].clone();
            ensure(
                !matches!(action, Action::Concede),
                "policy selected concession; refusing artificial terminal",
            )?;
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
                _ => (),
            }
            // Four ordinary root categories plus one reserved Harmonize category
            // bound *all* verification branch points to five distinct roots.
            let sampled_root =
                root_kind(&action).filter(|kind| roots.len() < 5 && !roots.contains(*kind));
            let before = if let Some(t) = &mut tracker {
                let observation = game.observe(actor);
                t.before(
                    &game,
                    &observation,
                    &action,
                    sampled_root == Some("harmonize_contribution"),
                );
                Some(Frame::read(&game, &observation))
            } else {
                None
            };
            if mode == Mode::Verify {
                if let Some(kind) = sampled_root {
                    owned_root(&game, actor, index, &action)?;
                    roots.insert(kind.to_owned());
                }
                trace.push((actor, action.clone()));
            }
            game.apply(actor, action.clone())
                .map_err(|e| format!("{} decision{decisions} {action:?}: {e:?}", case.id()))?;
            basic.inspect(&game);
            if let (Some(t), Some(before)) = (&mut tracker, before) {
                let after = Frame::read(&game, &game.observe(actor));
                t.inspect(&before, &after, &action, decisions, &game);
            }
            decisions += 1;
        }
        let play_seconds = started.elapsed().as_secs_f64();
        let cause = if mode == Mode::Benchmark {
            // Verified corpus supplies alternative-win arithmetic for this exact fixed
            // deterministic case. Benchmark does not repeat expensive verification.
            let expected = verified.ok_or("benchmark requires prior exact corpus verification")?;
            ensure(
                game.outcome() == Some(expected.outcome)
                    && decisions == expected.decisions
                    && basic.cards == expected.cards_cast
                    && basic.colors == expected.spell_colors,
                "benchmark diverged from the verified deterministic case",
            )?;
            match terminal(&game, None, decisions - 1) {
                Ok(c) => c,
                Err(_)
                    if expected.cause.kind == "call_five_distinct_positive_dragon_recipients" =>
                {
                    expected.cause.clone()
                }
                Err(e) => return Err(e),
            }
        } else {
            terminal(&game, tracker.as_ref(), decisions - 1)?
        };
        let mut replay_seconds = 0.0;
        let mut trace_file = None;
        let mut raw_trace_sha256 = None;
        let mut final_snapshot_debug_sha256 = None;
        let mut final_observations_sha256 = None;
        if let Some(t) = &tracker {
            ensure(
                basic.casts > 0 && t.payoffs > 0 && t.meaningful > 0,
                "game lacks an actual cast, completed rules payoff or meaningful life/zone interaction",
            )?;
            ensure(
                !roots.is_empty() && roots.len() <= 5,
                "game lacks bounded distinct meaningful owned decision roots",
            )?;
            let replay_started = Instant::now();
            let mut replay = Game::new(config.clone()).map_err(|e| format!("replay init {e:?}"))?;
            for (actor, action) in &trace {
                replay
                    .apply(*actor, action.clone())
                    .map_err(|e| format!("raw full replay {}: {e:?}", case.id()))?;
            }
            ensure(
                replay.snapshot() == game.snapshot()
                    && replay.outcome() == game.outcome()
                    && views(&replay)? == views(&game)?,
                "complete raw replay Snapshot/Outcome/both observations mismatch",
            )?;
            replay_seconds = replay_started.elapsed().as_secs_f64();
            let snapshot_hash = sha256(format!("{:?}", game.snapshot()).as_bytes());
            let observations_hash = sha256(&serde_json::to_vec(&views(&game)?).map_err(err)?);
            let payload = serde_json::json!({"workload":WORKLOAD,"catalogue":CATALOGUE_VERSION,"deck_source_sha256":DECKS_SHA256,"harness_source_sha256":harness_sha256(),"case":case,"ordered_config":config,"actions":trace,"outcome":game.outcome(),"terminal_cause":cause,"witness_receipts":t.receipts,"owned_root_kinds":roots,"final_snapshot_debug_sha256":snapshot_hash,"final_both_observations_sha256":observations_hash});
            let bytes = serde_json::to_vec(&payload).map_err(err)?;
            let digest = sha256(&bytes);
            let path = dir
                .ok_or("verification requires persistent evidence directory")?
                .join(format!("{}-trace.json", case.id()));
            std::fs::create_dir_all(path.parent().unwrap()).map_err(err)?;
            write_bytes(&path, &bytes)?;
            trace_file = Some(path.display().to_string());
            raw_trace_sha256 = Some(digest);
            final_snapshot_debug_sha256 = Some(snapshot_hash);
            final_observations_sha256 = Some(observations_hash);
        }
        let elapsed = if mode == Mode::Verify {
            started.elapsed().as_secs_f64()
        } else {
            play_seconds
        };
        Ok(GameReport {
            case,
            case_id: case.id(),
            repetition,
            decisions,
            turns: game.observe(case.starting_player).turn,
            outcome: game.outcome().unwrap(),
            cause,
            cards_cast: basic.cards,
            spell_colors: basic.colors,
            cast_count: basic.casts,
            meaningful_interactions: tracker.as_ref().map_or(0, |t| t.meaningful),
            owned_decision_roots: roots.into_iter().collect(),
            replayed_actions: if mode == Mode::Verify { trace.len() } else { 0 },
            witnesses: tracker.map_or_else(BTreeMap::new, |t| t.counts),
            raw_trace_sha256,
            trace_file,
            final_snapshot_debug_sha256,
            final_observations_sha256,
            rejected_attack_attempts,
            replay_seconds,
            measured_seconds: elapsed,
        })
    })();
    if let Err(reason) = &result
        && let Some(dir) = dir
    {
        let failure = serde_json::json!({"workload":WORKLOAD,"catalogue":CATALOGUE_VERSION,"deck_source_sha256":DECKS_SHA256,"harness_source_sha256":harness_sha256(),"case":case,"ordered_config":config,"attempted_primary_actions":trace,"failure":reason,"outcome":game.outcome()});
        write_json(&dir.join(format!("{}-failure.json", case.id())), &failure)?;
    }
    result
}
fn write_json(path: &Path, value: &impl Serialize) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(err)?;
    }
    write_bytes(path, &serde_json::to_vec_pretty(value).map_err(err)?)
}
fn write_bytes(path: &Path, bytes: &[u8]) -> Result<(), String> {
    use std::io::Write;
    if path.exists() {
        return ensure(
            std::fs::read(path).map_err(err)? == bytes,
            "evidence path already contains a different receipt; use a fresh cache directory",
        );
    }
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(err)?;
    file.write_all(bytes).map_err(err)
}
pub fn run(
    w: &Workload,
    cases: &[Case],
    mode: Mode,
    dir: Option<&Path>,
    verified: Option<&Report>,
) -> Result<Report, String> {
    ensure(
        !cases.is_empty() && cases.iter().all(|c| w.cases.contains(c)),
        "empty or foreign workload case list",
    )?;
    let repetitions = if mode == Mode::Verify {
        ensure(
            cases.iter().map(|c| c.id()).collect::<BTreeSet<_>>().len() == cases.len(),
            "verification cases must not repeat",
        )?;
        1
    } else {
        let standard = w.standard();
        let freeform = w.freeform();
        ensure(
            cases == standard.repeat(3) || cases == freeform.repeat(3),
            "benchmark is exactly3 repetitions of48 Standard cases or4 separate Freeform cases",
        )?;
        verified
            .ok_or("benchmark requires a passing full verification corpus receipt")?
            .check_complete_standard(w)?;
        3
    };
    let mut games = Vec::new();
    let mut witnesses = BTreeMap::new();
    let mut colors = 0;
    let mut decisions = 0;
    let mut seconds = 0.0;
    for (i, &case) in cases.iter().enumerate() {
        let expected = verified.and_then(|r| r.games.iter().find(|g| g.case == case));
        let g = run_one(
            w,
            case,
            if repetitions == 1 {
                0
            } else {
                i / (cases.len() / 3)
            },
            mode,
            dir,
            expected,
        )?;
        colors |= g.spell_colors;
        for (k, v) in &g.witnesses {
            *witnesses.entry(k.clone()).or_default() += v;
        }
        decisions += g.decisions;
        seconds += g.measured_seconds;
        games.push(g);
    }
    Ok(Report{workload:WORKLOAD.into(),catalogue:CATALOGUE_VERSION.into(),deck_source_sha256:DECKS_SHA256.into(),harness_source_sha256:harness_sha256(),printed_outcome_ledger_sha256:w.proof_ledger_sha256.clone(),physical_manifest_sha256:physical_manifest_sha256(),mode:if mode==Mode::Verify{"verification"}else{"benchmark"}.into(),scope:"Exact frozen TDM-only Standard48 plus separate Cutter Freeform4;4096 actions per game; not general Standard coverage, gameplay parity, or exhaustive card proof".into(),physical_members:276,standard_cases:cases.iter().filter(|c|!c.freeform).map(|c|c.id()).collect::<BTreeSet<_>>().len(),freeform_cases:cases.iter().filter(|c|c.freeform).map(|c|c.id()).collect::<BTreeSet<_>>().len(),repetitions,decks:w.inputs.standard_decks.iter().chain(std::iter::once(&w.inputs.freeform_cutter_deck)).cloned().collect(),games_per_second:games.len()as f64/seconds,decisions_per_second:decisions as f64/seconds,decisions,measured_seconds:seconds,games,witnesses,actual_spell_colors:colors,measurement:if mode==Mode::Verify{"Sum per-game wall time includes construction, legal-action emission, shared smoke policy/actor observations, before/after verification telemetry, at most5 owned roots, complete raw replay, paired observation comparisons, SHA256 and persistent trace serialization. Not a simulation benchmark."}else{"Sum per-game construction+play time includes exact Config cloning, Game::new/shuffle, legal-action emission, generic rejected-attack memory, policy::smoke_index with its actor observation, rules apply, and lightweight committed-cast packet/card/color telemetry. Excludes workload/deck validation, replay/owned checks, detailed mechanic telemetry, trace/per-decision JSON, terminal reconciliation/report serialization, model inference/network/tensors. Same frozen cases as verified corpus, exactly3 repetitions; Freeform separately labelled."}.into()})
}
pub fn write_shard(report: &Report, pair: usize, dir: &Path) -> Result<(), String> {
    write_json(&dir.join(format!("pair-{pair}-report.json")), report)
}
pub fn aggregate(w: &Workload, dir: &Path) -> Result<Report, String> {
    let mut all: Option<Report> = None;
    for pair in 0..7 {
        let path = dir.join(format!("pair-{pair}-report.json"));
        let bytes = std::fs::read(&path)
            .map_err(|e| format!("missing actual shard {}: {e}", path.display()))?;
        let r: Report = serde_json::from_slice(&bytes).map_err(err)?;
        ensure(
            r.workload == WORKLOAD
                && r.catalogue == CATALOGUE_VERSION
                && r.deck_source_sha256 == DECKS_SHA256
                && r.harness_source_sha256 == harness_sha256()
                && r.printed_outcome_ledger_sha256 == w.proof_ledger_sha256
                && r.physical_manifest_sha256 == physical_manifest_sha256()
                && r.mode == "verification"
                && r.repetitions == 1,
            "stale/foreign/nonverification shard",
        )?;
        ensure(
            r.games.iter().map(|g| g.case).collect::<Vec<_>>() == w.pair(pair),
            "shard does not contain exact frozen pair cases",
        )?;
        ensure(
            r.physical_members == 276
                && r.standard_cases == if pair == 6 { 0 } else { 8 }
                && r.freeform_cases == if pair == 6 { 4 } else { 0 },
            "shard Standard/Freeform dimension metadata differs from frozen cases",
        )?;
        for g in &r.games {
            authenticate_game(w, g)?;
            let path = Path::new(g.trace_file.as_deref().ok_or("shard missing raw trace")?);
            ensure(
                sha256(&std::fs::read(path).map_err(err)?)
                    == g.raw_trace_sha256.as_deref().ok_or("trace digest absent")?,
                "raw trace artifact digest mismatch",
            )?;
        }
        if let Some(a) = &mut all {
            merge(a, r);
        } else {
            all = Some(r);
        }
    }
    let mut result: Report = all.unwrap();
    result.standard_cases = 48;
    result.freeform_cases = 4;
    result.games_per_second = result.games.len() as f64 / result.measured_seconds;
    result.decisions_per_second = result.decisions as f64 / result.measured_seconds;
    ensure(
        result.games.len() == 52,
        "aggregate must retain exactly52 real games",
    )?;
    result.check_complete_standard(w)?;
    write_json(&dir.join("corpus-verification.json"), &result)?;
    Ok(result)
}
fn merge(a: &mut Report, b: Report) {
    a.games.extend(b.games);
    a.actual_spell_colors |= b.actual_spell_colors;
    a.decisions += b.decisions;
    a.measured_seconds += b.measured_seconds;
    for (k, v) in b.witnesses {
        *a.witnesses.entry(k).or_default() += v;
    }
}

// Small dependency-free SHA256 for frozen input/receipt integrity, not gameplay.
fn sha256(input: &[u8]) -> String {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    let mut bytes = input.to_vec();
    let bits = (bytes.len() as u64).wrapping_mul(8);
    bytes.push(0x80);
    while bytes.len() % 64 != 56 {
        bytes.push(0);
    }
    bytes.extend_from_slice(&bits.to_be_bytes());
    for block in bytes.as_chunks::<64>().0 {
        let mut w = [0u32; 64];
        for (i, b) in block.as_chunks::<4>().0.iter().enumerate() {
            w[i] = u32::from_be_bytes(*b);
        }
        for i in 16..64 {
            let a = w[i - 15];
            let b = w[i - 2];
            let s0 = a.rotate_right(7) ^ a.rotate_right(18) ^ (a >> 3);
            let s1 = b.rotate_right(17) ^ b.rotate_right(19) ^ (b >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut z] = h;
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ (!e & g);
            let t1 = z
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            z = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        for (i, v) in [a, b, c, d, e, f, g, z].into_iter().enumerate() {
            h[i] = h[i].wrapping_add(v);
        }
    }
    h.iter().map(|v| format!("{v:08x}")).collect()
}

fn deserialize_outcome<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Outcome, D::Error> {
    #[derive(Deserialize)]
    enum Wire {
        Winner(Player),
        Draw,
    }
    Ok(match Wire::deserialize(d)? {
        Wire::Winner(p) => Outcome::Winner(p),
        Wire::Draw => Outcome::Draw,
    })
}

fn harness_sha256() -> String {
    sha256(include_bytes!("tdm_games.rs"))
}

/// Integrity regression only; it neither loads the partial pool nor runs games.
pub fn verify_hash_vectors() -> Result<(), String> {
    ensure(
        sha256(b"") == "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        "SHA256 empty known vector",
    )?;
    ensure(
        sha256(b"abc") == "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
        "SHA256 abc known vector",
    )?;
    ensure(
        sha256(include_bytes!("tdm_game_decks.json")) == DECKS_SHA256,
        "frozen deck SHA256 known digest",
    )
}

fn physical_manifest_sha256() -> String {
    sha256(include_bytes!("../../data/sets/tdm.json"))
}
fn authenticate_game(w: &Workload, g: &GameReport) -> Result<(), String> {
    ensure(
        w.cases.contains(&g.case) && g.case_id == g.case.id(),
        "game case_id/typed case does not match frozen workload",
    )?;
    ensure(
        g.repetition == 0
            && g.decisions > 0
            && g.decisions <= LIMIT
            && g.replayed_actions == g.decisions,
        "verification game action ceiling/replay/repetition mismatch",
    )?;
    ensure(
        !g.owned_decision_roots.is_empty()
            && g.owned_decision_roots.len() <= 5
            && g.owned_decision_roots.iter().collect::<BTreeSet<_>>().len()
                == g.owned_decision_roots.len(),
        "owned decision root set must have1..5 distinct meaningful roots",
    )
}
