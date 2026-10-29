//! One legal FRA workload for native replay assurance and a separate plain benchmark.
//! Action selection always delegates to the shared actor-observation smoke policy.
use super::*;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::time::Instant;
#[path = "fra_survey_specs.rs"]
mod survey_specs;
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
    pub fra_membership: bool,
}
struct Deck {
    cards: Vec<Card>,
    report: DeckReport,
}
#[derive(Serialize)]
pub struct GameReport {
    pub pair: usize,
    pub seed: u64,
    pub deck_swapped: bool,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub physical_cards_cast: Option<BTreeSet<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub other_spell_faces_cast: Option<BTreeSet<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lands_played: Option<BTreeSet<String>>,
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
    /// Physical identities present in decks actually used by these cases.
    pub played_deck_identities: usize,
    /// Distinct observed spell-face names, including prepared faces.
    pub observed_spell_faces: usize,
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
            "prepared_spell_resolved",
            "empower_counters",
            "fatehold_scry_trigger",
            "scry_or_surveil_completed",
            "loyalty_activation",
        ]
        .into_iter()
        .filter(|m| self.witnesses.get(*m).copied().unwrap_or(0) == 0)
        .collect()
    }
}
pub fn cases(seeds_per_pair: u64) -> Vec<(usize, u64, Player, bool)> {
    (0..8)
        .flat_map(|pair| {
            (0..seeds_per_pair).flat_map(move |seed| {
                [false, true].into_iter().map(move |swapped| {
                    (
                        pair,
                        897500 + pair as u64 * 100 + seed,
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
/// Audit every physical identity; an unknown card is as incomplete as a gated one.
pub fn missing_pool_members() -> Result<Vec<String>, String> {
    let source: serde_json::Value = serde_json::from_str(include_str!("../../data/sets/fra.json"))
        .map_err(|e| e.to_string())?;
    let rows = source["cards"].as_array().ok_or("missing FRA card list")?;
    let identities: BTreeSet<_> = rows
        .iter()
        .filter_map(|r| r["oracle_id"].as_str())
        .collect();
    if rows.len() != 285 || identities.len() != 285 {
        return Err(format!(
            "FRA denominator changed: {} rows, {} identities",
            rows.len(),
            identities.len()
        ));
    }
    let mut missing = Vec::new();
    for row in rows {
        let name = row["name"].as_str().ok_or("FRA identity has no name")?;
        if !Card::from_name(name)
            .is_some_and(|c| c.is_supported() && c.info().oracle_id == row["oracle_id"].as_str())
        {
            missing.push(name.to_string());
        }
    }
    Ok(missing)
}
fn decks(require_complete: bool) -> Result<(Vec<Deck>, usize), String> {
    let source: serde_json::Value = serde_json::from_str(include_str!("../../data/sets/fra.json"))
        .map_err(|e| e.to_string())?;
    let members: BTreeSet<&str> = source["cards"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["oracle_id"].as_str().unwrap())
        .collect();
    if members.len() != 285 {
        return Err(format!("FRA denominator changed: {}", members.len()));
    }
    let missing = missing_pool_members()?;
    if require_complete && !missing.is_empty() {
        return Err(format!(
            "FRA full-pool gate: {} unsupported or missing identities: {}",
            missing.len(),
            missing.join(", ")
        ));
    }
    // Each deck has 28 in-set basics and four copies of eight FRA spells.
    // Chosen to exercise prepared faces, scry/surveil, combat and loyalty.
    let specifications = [
        (
            "Fatehold",
            ["Plains", "Island"],
            [
                "Campus Crier",
                "Repurposed Enforcer",
                "Fatehold Chronologist // Peer Review",
                "Diviner of Victory // Unwind History",
                "Surveillance Phantasm",
                "Proctor of Potential",
                "Fatehold Charm",
                "The Theorist, Jace Beleren",
            ],
        ),
        (
            "Stingerquill",
            ["Swamp", "Mountain"],
            [
                "Dark Matter Manipulator",
                "Rank Rat",
                "Eardrum Rattler",
                "Stingerquill Voxmancer // Vicious Verse",
                "Hallway Heckler // Vicious Verse",
                "Grim Repriser",
                "Stinging Vitriol",
                "Stingerquill Charm",
            ],
        ),
        (
            "Konstrari",
            ["Forest", "Mountain"],
            [
                "Heartwood Crafter // Soul Tether",
                "Konstrari Improviser // Soul Tether",
                "Woodwork Prodigy // Soul Tether",
                "Sureshot Sower",
                "Budding Insurgent",
                "Marwyn, the Clearcutter",
                "Konstrari Charm",
                "Blazing Crescendo",
            ],
        ),
        (
            "Vigorbloom",
            ["Forest", "Plains"],
            [
                "Emergency Phytomedic // Seed Suture",
                "Vigorbloom Vanguard // Seed Suture",
                "Blossom-Blessed Angel // Seed Suture",
                "Solarium Sentry",
                "Unflinching Hortimancer",
                "Sureshot Sower",
                "Vigorbloom Charm",
                "Ajani Resolute",
            ],
        ),
        (
            "Theorix",
            ["Island", "Swamp"],
            [
                "Diviner of Victory // Unwind History",
                "Theorist's Proxy",
                "Paradox Shaper // Omit Variables",
                "Theorix Metamage // Omit Variables",
                "Void Extrapolator // Omit Variables",
                "Dark Matter Manipulator",
                "Theorix Charm",
                "Last Gasp",
            ],
        ),
        (
            "Echoes",
            ["Forest", "Island"],
            [
                "Carnivorous Cultivator // Enroot",
                "Sureshot Sower",
                "Arcane Amphisbaena",
                "Avatar of Burgeoning Echoes",
                "Theorist's Proxy",
                "Diviner of Victory // Unwind History",
                "Kiora of Salt and Sand",
                "Unsummon",
            ],
        ),
    ];
    // Singleton spells broaden the exercised identities without changing the
    // existing archetype decks or their mechanic witnesses.
    let diverse = [
        (
            "Fatehold Survey",
            ["Plains", "Island"],
            [
                "Academic Ascent",
                "Afterthought Sentry",
                "Arni, Humble Scribe",
                "Cryotheory Adept",
                "Danitha, Sword of Hope",
                "Desperate Futurescribe",
                "Divining Duelist",
                "Enlightened Confidant",
                "Fblthp, Impossibly Lost",
                "Flickering Hound",
                "Geist of Saint Thalia",
                "Graft Surgeon",
                "Icy Reception",
                "Jace's Machinations",
                "Karn, Argent Defender",
                "Koth of the Homestead",
                "Liliana the Faultless",
                "Living Library",
                "Loyal Tutor",
                "Lyra, Archangel of Dawn",
                "Lyra, Tolarian Archangel",
                "Mindseeker Oculus",
                "Perfected Theory",
                "Precise Redaction",
                "Predictive Preparations",
                "Proft, Consulting Detective",
                "Prophesied End",
                "Refute Destiny",
                "Rescue Girl, First Responder",
                "Samut, Tyrant of Naktamun",
                "Surgical Precision",
                "Yuriko, Hope from the Shadows",
            ],
        ),
        (
            "Vigorbloom Survey",
            ["Forest", "Swamp"],
            [
                "Bestial Incursion",
                "Bloodline Recollector // Ancestral Craving",
                "Break Under Pressure",
                "Cast Away Doubt",
                "Compel Brutality",
                "Danitha, Spear of Agony",
                "Darklight Phoenix",
                "Extended Absence",
                "Ferocity of the Hunt",
                "Flourishing Grapple",
                "Gallia, Tragic Host",
                "Gideon the Oathless",
                "Greenhouse Propagator",
                "Hunter's Axe",
                "Inspired Tethermage",
                "Jiang Yanggu, Never Alone",
                "Lich's Relic",
                "Liliana the Repentant",
                "Loot, the Anomaly",
                "Loot, the Nexus",
                "Mabel, Bitter Recluse",
                "Marwyn, the Preserver",
                "Multiply by Zero",
                "Pia, Aether Ascetic",
                "Primal Witchstalker",
                "Proft, Sinister Mastermind",
                "Rampart Hunter",
                "Sanctum Lurker",
                "Screeching Soulbreaker",
                "Simulacrum Shaper",
                "Theoretical Necromancer",
                "Yoshimaru, Scrappy Stray",
            ],
        ),
        (
            "Fatehold Second Survey",
            ["Plains", "Island"],
            [
                "Chandra, Chill of Compliance",
                "Countersculpt",
                "Cruel Calculations",
                "Denzilore Fatehold",
                "Fateshaper Aspirant",
                "Generous Revival",
                "Germinate Recruits",
                "Gideon's Memorial",
                "Guiding Hydra",
                "Hapatra, the Desert Frost",
                "Hexhaven Battalion",
                "Infinite Coursework",
                "Jace, Reality Sculptor",
                "Memory Trap",
                "Plan for All Outcomes",
                "Protege's Awakening",
                "Prudent Fateseer // Peer Review",
                "Ruric Thar, Biomagus",
                "Saheeli, Consul of Oversight",
                "Seasoned Cryomancer",
                "Semester Foreseer // Peer Review",
                "Shatterwing Pegasus",
                "Sphinx of False Conclusions",
                "Sphinx's Approach",
                "Tetsuko Umezawa, Fugitive",
                "Teyo, Lightshield Expert",
                "Thalia, the Survivor",
                "Tomik, Orzhov Lawmage",
                "Traxos, Academy Guardian",
                "Undulating Witness",
                "Variable Chaser // Arc of Fortune",
                "Way of the Cryomancer",
            ],
        ),
        (
            "Stingerquill Second Survey",
            ["Swamp", "Mountain"],
            [
                "Ajani Unrelenting",
                "Ajani's Anguish",
                "Apex Witchstalker",
                "Arni, Renowned Champion",
                "Artifist Acumen",
                "Awaken the Inferno",
                "Chandra's Emberling",
                "Chandra, Torch of Defiance",
                "Command the Stage",
                "Curse-Marred Demon",
                "Draconic Visitor",
                "Essence Burn",
                "Extrapolate the Impossible",
                "Fulminous Forte",
                "Gallia, the Merrymaker",
                "Garruk, Veiled Butcher",
                "Heartstring Puller",
                "Identity Echo",
                "Ingris Stingerquill",
                "Jiang Yanggu, Alone",
                "Kiora of Fire and Ashes",
                "Koth, the Geomancer",
                "Massacre Girl, Most Wanted",
                "Master of Barbs",
                "No Admittance",
                "Overwrite the Multiverse",
                "Pia, Determined Rebuilder",
                "Pompous Battlemage // Improvised Act",
                "Pyre Rhymer // Molten Tide",
                "Rewrite Regrets",
                "Rise of the Deathbringer",
                "Samut, Hazoret's Champion",
            ],
        ),
    ];
    let all_specs = specifications
        .iter()
        .map(|(name, lands, spells)| (*name, *lands, spells.as_slice(), 4))
        .chain(
            diverse
                .iter()
                .map(|(name, lands, spells)| (*name, *lands, spells.as_slice(), 1)),
        );
    let mut out = Vec::new();
    for (name, lands, spells, copies) in all_specs {
        let mut cards = Vec::new();
        for land in lands {
            let card = Card::from_name(land).unwrap();
            cards.extend(std::iter::repeat_n(card, 14));
        }
        for spell in spells {
            let card =
                Card::from_name(spell).ok_or_else(|| format!("unknown workload card {spell}"))?;
            cards.extend(std::iter::repeat_n(card, copies));
        }
        let mut counts = BTreeMap::new();
        for &c in &cards {
            if !c.is_supported() || !c.is_standard_legal() {
                return Err(format!("illegal workload card {}", c.name()));
            }
            if !c.info().oracle_id.is_some_and(|oid| members.contains(oid)) {
                return Err(format!("non-FRA workload card {}", c.name()));
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
            fra_membership: true,
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
    prepared_spell_packets: BTreeSet<u64>,
    cards: BTreeSet<String>,
    witnesses: BTreeMap<String, u64>,
    colors: u8,
    pending_land: Option<(CardId, String)>,
    lands_played: BTreeSet<String>,
}
impl Tracker {
    fn hit(&mut self, key: &str, event: String) {
        if self.seen_events.insert(format!("{key}/{event}")) {
            *self.witnesses.entry(key.to_string()).or_default() += 1;
        }
    }
    fn inspect(
        &mut self,
        _before: &Observation,
        after: &Observation,
        _action: &Action,
        survey: bool,
    ) {
        if survey {
            if let Action::PlayLand { card } = _action
                && let Some(object) = _before.objects.iter().find(|object| object.id == *card)
            {
                self.pending_land = Some((*card, object.card.name().to_owned()));
            }
            if let Some((id, name)) = &self.pending_land
                && after
                    .objects
                    .iter()
                    .any(|o| o.id == *id && o.zone == Zone::Battlefield)
            {
                self.lands_played.insert(name.clone());
                self.pending_land = None;
            } else if after.decision == "priority" {
                self.pending_land = None;
            }
        }
        for row in &_before.preparations {
            self.prepared_copies.insert(row.spell_copy);
        }
        for row in &after.preparations {
            self.prepared_copies.insert(row.spell_copy);
            self.hit("prepared_created", format!("{:?}", row.spell_copy));
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
                if s.cast.is_some_and(|cast| cast.origin == Zone::Exile)
                    && self.prepared_copies.iter().any(|r| {
                        r.card == source.object.card
                            && r.generation.checked_add(1) == Some(source.object.generation)
                    })
                {
                    self.prepared_spell_packets.insert(s.id);
                    self.hit("prepared_spell_cast", s.id.to_string());
                }
            }
            if source.card.name() == "Diviner of Victory"
                && s.kind == (StackKind::TriggeredAbility { ability: 0 })
                && s.source_ability_origin.is_none()
                && s.attachment_trigger.is_none()
            {
                self.hit("fatehold_scry_trigger", s.id.to_string());
            }
            if matches!(s.kind, StackKind::ActivatedAbility { .. })
                && matches!(
                    source.card.name(),
                    "Ajani Resolute" | "The Theorist, Jace Beleren"
                )
            {
                self.hit("loyalty_activation", s.id.to_string());
            }
        }
        // Only count a directly observed resolving packet. An instantaneous
        // resolution may go unseen; it is intentionally not inferred from exit.
        if let Some(packet) = &after.resolving
            && self.prepared_spell_packets.contains(&packet.id)
        {
            self.hit("prepared_spell_resolving", packet.id.to_string());
        }
        if let Some(packet) = &_before.resolving
            && self.prepared_spell_packets.contains(&packet.id)
            && !after
                .resolving
                .as_ref()
                .is_some_and(|next| next.id == packet.id)
            && !after.stack.iter().any(|next| next.id == packet.id)
        {
            self.hit("prepared_spell_resolved", packet.id.to_string());
        }
        // Witness a real loyalty increase on the Jace token while a known
        // empower instruction is resolving. This excludes merely casting or
        // putting the corresponding trigger on the stack.
        if let Some(packet) = _before.resolving.as_ref().or_else(|| {
            matches!(_action, Action::Pass)
                .then(|| _before.stack.last())
                .flatten()
        }) && packet.source().is_some_and(|source| {
            matches!(
                source.card.name(),
                "Campus Crier"
                    | "Repurposed Enforcer"
                    | "Theorist's Proxy"
                    | "Avatar of Burgeoning Echoes"
                    | "Arcane Amphisbaena"
                    | "Fatehold Charm"
            )
        }) {
            for object in &after.objects {
                if object.zone != Zone::Battlefield
                    || object.controller != packet.controller
                    || object.card.identity() != "@token/jace-planeswalker"
                {
                    continue;
                }
                let prior = _before
                    .objects
                    .iter()
                    .find(|o| o.reference == object.reference)
                    .map_or(0, |o| o.counters.get(CounterKind::Loyalty));
                if object.counters.get(CounterKind::Loyalty) > prior {
                    self.hit(
                        "empower_counters",
                        format!("{}/{:?}", packet.id, object.reference),
                    );
                }
            }
        }
        if _before.library_choice.is_some()
            && after.library_choice.is_none()
            && matches!(_action, Action::ArrangeLibrary { .. })
        {
            self.hit(
                "scry_or_surveil_completed",
                format!("{}/{:?}/{}", _before.turn, _before.viewer, _before.decision),
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
    cases: &[(usize, u64, Player, bool)],
    mode: Mode,
    trace_dir: Option<&Path>,
) -> Result<Report, String> {
    run_inner(cases, mode, trace_dir, true, false)
}

// Bounded in-tree assurance only; it is never full-set acceptance.
#[cfg(test)]
pub fn run_admitted_smoke(cases: &[(usize, u64, Player, bool)]) -> Result<Report, String> {
    run_inner(cases, Mode::Verify, None, false, false)
}

/// Optional 20-game survey; never invoked by the routine acceptance tests.
#[allow(dead_code, reason = "The optional survey is a separate example")]
pub fn run_survey(
    trace_dir: Option<&Path>,
    single: Option<(usize, u64, bool)>,
) -> Result<Report, String> {
    let mut cases: Vec<_> = (0..5)
        .flat_map(|pair| {
            (0..2).flat_map(move |seed| {
                [false, true].into_iter().map(move |swap| {
                    (
                        pair,
                        946000 + pair as u64 * 100 + seed,
                        if seed == 0 { Player::P0 } else { Player::P1 },
                        swap,
                    )
                })
            })
        })
        .collect();
    if let Some((pair, seed, swapped)) = single {
        if pair >= 5 || seed >= 2 {
            return Err("case requires pair 0..4 and seed index 0..1".into());
        }
        cases.retain(|&(p, s, _, swap)| {
            p == pair && s == 946000 + pair as u64 * 100 + seed && swap == swapped
        });
    }
    run_inner(&cases, Mode::Verify, trace_dir, true, true)
}

fn survey_decks() -> Result<(Vec<Deck>, usize), String> {
    let (_, pool_members) = decks(true)?;
    let members: BTreeSet<_> =
        serde_json::from_str::<serde_json::Value>(include_str!("../../data/sets/fra.json"))
            .map_err(|e| e.to_string())?["cards"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|r| r["oracle_id"].as_str().map(str::to_owned))
            .collect();
    let mut result = Vec::new();
    for &(name, basics, spells, lands) in survey_specs::SPECS {
        let mut cards = Vec::new();
        for basic in basics {
            cards.extend(std::iter::repeat_n(
                Card::from_name(basic).ok_or(basic)?,
                10,
            ));
        }
        for card in spells.iter().chain(lands) {
            cards.push(Card::from_name(card).ok_or(*card)?);
        }
        let mut counts = BTreeMap::new();
        for &card in &cards {
            if !card.is_supported()
                || !card.is_standard_legal()
                || !card.info().oracle_id.is_some_and(|id| members.contains(id))
            {
                return Err(format!("invalid survey card {}", card.name()));
            }
            *counts.entry(card.name().to_string()).or_insert(0usize) += 1;
        }
        if cards.len() != 60
            || cards.iter().any(|card| {
                counts[card.name()] > 4
                    && !matches!(
                        *card,
                        Card::Plains | Card::Island | Card::Swamp | Card::Mountain | Card::Forest
                    )
            })
        {
            return Err(format!("invalid survey deck {name}"));
        }
        result.push(Deck {
            cards,
            report: DeckReport {
                name,
                cards: counts,
                size: 60,
                standard_validation: true,
                fra_membership: true,
            },
        });
    }
    Ok((result, pool_members))
}
fn run_inner(
    cases: &[(usize, u64, Player, bool)],
    mode: Mode,
    trace_dir: Option<&Path>,
    require_complete: bool,
    survey: bool,
) -> Result<Report, String> {
    let (decks, pool_members) = if survey {
        survey_decks()?
    } else {
        decks(require_complete)?
    };
    let pairings = [
        (0, 1),
        (2, 3),
        (4, 1),
        (1, 2),
        (3, 4),
        (5, 2),
        (6, 7),
        (8, 9),
    ];
    let pairings = if survey {
        [
            (0, 1),
            (2, 3),
            (4, 5),
            (6, 7),
            (8, 9),
            (0, 1),
            (0, 1),
            (0, 1),
        ]
    } else {
        pairings
    };
    let physical_names: BTreeSet<String> = if survey {
        serde_json::from_str::<serde_json::Value>(include_str!("../../data/sets/fra.json"))
            .map_err(|e| e.to_string())?["cards"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|r| {
                r["name"]
                    .as_str()
                    .map(|n| n.split(" // ").next().unwrap().to_owned())
            })
            .collect()
    } else {
        BTreeSet::new()
    };
    let trace_decisions =
        survey && std::env::var_os("RUSTMAGE_FRA_SURVEY_TRACE_DECISIONS").is_some();
    let started = Instant::now();
    let mut games = Vec::new();
    let mut total_witnesses = BTreeMap::new();
    let mut colors = 0;
    let mut total_decisions = 0;
    for &(pair, seed, starting_player, swapped) in cases {
        if survey {
            eprintln!(
                "FRA_SURVEY start pair={pair} seed={seed} swapped={swapped} starting={starting_player:?}"
            );
        }
        let (left, right) = pairings[pair];
        let (a, b) = if swapped {
            (right, left)
        } else {
            (left, right)
        };
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
            if trace_decisions {
                eprintln!("FRA_SURVEY decision={decisions} stage=legal_actions");
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
            if trace_decisions {
                eprintln!(
                    "FRA_SURVEY decision={decisions} stage=apply actor={actor:?} action={action:?}"
                );
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
            tracker.inspect(&before, &after, &action, survey);
            if decisions.is_multiple_of(64) {
                max_bytes = max_bytes.max(game.state_bytes());
            }
            decisions += 1;
        }
        if survey {
            eprintln!(
                "FRA_SURVEY replay pair={pair} seed={seed} swapped={swapped} decisions={decisions}"
            );
        }
        if mode.verifies() {
            let mut replay = Game::new(config.clone()).map_err(|e| format!("replay init {e:?}"))?;
            for (index, (actor, action)) in trace.iter().enumerate() {
                if trace_decisions {
                    eprintln!("FRA_SURVEY replay_action={index} actor={actor:?} action={action:?}");
                }
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
        if survey {
            eprintln!(
                "FRA_SURVEY done pair={pair} seed={seed} swapped={swapped} decisions={decisions} survey_elapsed={:.3}s",
                started.elapsed().as_secs_f64()
            );
        }
        let trace_file = if mode.verifies() {
            if let Some(dir) = trace_dir {
                std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
                let path = dir.join(format!("pair-{pair}-seed-{seed}-swap-{swapped}.json"));
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
            deck_swapped: swapped,
            starting_player,
            decisions,
            turns,
            outcome,
            owned_branches,
            replayed_actions: if mode.verifies() { trace.len() } else { 0 },
            rejected_attack_attempts,
            trace_digest: mode.verifies().then(|| format!("{hash:016x}")),
            trace_file,
            physical_cards_cast: survey.then(|| {
                tracker
                    .cards
                    .intersection(&physical_names)
                    .cloned()
                    .collect()
            }),
            other_spell_faces_cast: survey
                .then(|| tracker.cards.difference(&physical_names).cloned().collect()),
            lands_played: survey.then_some(tracker.lands_played),
            cards_cast: tracker.cards,
            witnesses: tracker.witnesses,
            max_sampled_state_bytes: max_bytes,
        });
    }
    let seconds = started.elapsed().as_secs_f64();
    let played_deck_identities = cases
        .iter()
        .flat_map(|(pair, _, _, _)| {
            let (left, right) = pairings[*pair];
            decks[left].cards.iter().chain(&decks[right].cards)
        })
        .filter_map(|card| card.info().oracle_id)
        .collect::<BTreeSet<_>>()
        .len();
    let observed_spell_faces = games
        .iter()
        .flat_map(|game| game.cards_cast.iter())
        .collect::<BTreeSet<_>>()
        .len();
    Ok(Report {
        workload: if survey {
            "FRA_OPTIONAL_UNCAST_SURVEY"
        } else if require_complete {
            "FRA_ARCHETYPES_V1"
        } else {
            "FRA_ADMITTED_SMOKE_ONLY"
        },
        catalogue: CATALOGUE_VERSION,
        mode: if mode.verifies() {
            "verification"
        } else {
            "benchmark"
        },
        scope: if survey {
            "Optional selected games from a 20-case corpus across ten FRA-only Standard decks, 20 basics and eight additional lands per deck; physical cast, other face and actual land play ledgers; natural terminals; not exhaustive gameplay coverage"
        } else {
            "Selected cases from ten exactly 60-card Standard decks; physical FRA members only; max four nonbasic copies, 28 basic lands; natural terminals; shared observation-only smoke policy with generic rejected-attack memory; not exhaustive gameplay coverage"
        },
        pool_members,
        played_deck_identities,
        observed_spell_faces,
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
            "Includes game construction, rules, action emission, shared smoke policy and observation telemetry. Excludes replay/owned checks, trace serialization, model inference, tensors and network. Named FRA workload only; not all Standard or XMage parity"
        },
    })
}
fn crate_policy(game: &Game, actions: &[Action]) -> usize {
    super::smoke_index(game, actions)
}
