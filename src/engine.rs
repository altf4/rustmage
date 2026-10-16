use crate::cards::{
    ActivatedAbility, ActivationCost, ActivationGuard, ActivationOrigin, CastRelation, CounterKind,
    CounterSubject, Counters, CreatureFilter, DamageQualification, DamageSubject, Effect,
    EffectChoiceKind, EntryTapped, Instruction, ManaCost, ManaProduction, MillAmount, PlayerSet,
    RecoveryFilter, SacrificeFilter, SearchSpec, Stats, StepGuard, StepValue, Subject, TargetPlan,
    TargetRule, TargetSpec, TargetZone, TriggerCondition, TriggerStep, payments,
};
#[path = "engine_activations.rs"]
mod activations;
#[path = "engine_connive.rs"]
mod connive;
#[path = "engine_damage_triggers.rs"]
mod damage_triggers;
#[path = "engine_discard_events.rs"]
mod discard_events;
#[path = "engine_draw_events.rs"]
mod draw_events;
#[path = "engine_funding.rs"]
mod funding;
#[path = "engine_resolution_mana.rs"]
mod resolution_mana;
use funding::FundingGoal;
#[path = "engine_blocking.rs"]
mod blocking;
#[path = "engine_effect_choices.rs"]
mod effect_choices;
#[path = "engine_exile.rs"]
mod exile;
#[path = "engine_fight.rs"]
mod fight;
#[path = "engine_milling.rs"]
mod milling;
#[path = "engine_sba.rs"]
mod sba;
#[path = "engine_search.rs"]
mod search;
#[path = "engine_stats.rs"]
mod stats;
#[path = "engine_steps.rs"]
mod steps;
#[path = "engine_tokens.rs"]
mod tokens;
use crate::rng::Rng;
use crate::{Card, Keywords, Mana, TokenInfo};
use serde::{Deserialize, Serialize};

/// Stable physical-card slot; ceased token slots may be reused.
/// Use `ObjectRef` for a particular game-object incarnation.
pub type CardId = u32;

/// Rules identity within a game branch. Every zone change creates a new incarnation.
/// References are scoped to their game/snapshot; revision guards still protect actions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ObjectRef {
    pub card: CardId,
    pub generation: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Player {
    P0,
    P1,
}

impl Player {
    pub fn other(self) -> Self {
        match self {
            Self::P0 => Self::P1,
            Self::P1 => Self::P0,
        }
    }
    pub fn index(self) -> usize {
        self as usize
    }
}

#[cfg(test)]
#[path = "engine_activation_guard_tests.rs"]
mod activation_guard_tests;
#[cfg(test)]
#[path = "engine_damage_tests.rs"]
mod damage_tests;
#[cfg(test)]
#[path = "engine_entry_life_tests.rs"]
mod entry_life_tests;
#[cfg(test)]
#[path = "engine_identity_tests.rs"]
mod identity_tests;
#[path = "engine_programs.rs"]
mod programs;
use programs::Resolution;
#[path = "engine_targeting.rs"]
mod targeting;
use targeting::Announcement;
#[cfg(test)]
#[path = "engine_activation_tests.rs"]
mod activation_tests;
#[cfg(test)]
#[path = "engine_counter_tests.rs"]
mod counter_tests;
#[cfg(test)]
#[path = "engine_funding_tests.rs"]
mod funding_tests;
#[cfg(test)]
#[path = "engine_mana_tests.rs"]
mod mana_tests;
#[cfg(test)]
#[path = "engine_program_tests.rs"]
mod program_tests;
#[cfg(test)]
#[path = "engine_target_tests.rs"]
mod target_tests;
#[cfg(test)]
#[path = "engine_tests.rs"]
mod tests;
#[cfg(test)]
#[path = "engine_token_tests.rs"]
mod token_tests;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Zone {
    Library,
    Hand,
    Battlefield,
    Stack,
    Graveyard,
    Exile,
    /// Vacant slot after a token ceases to exist; never a game zone.
    Outside,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Step {
    Pregame,
    Upkeep,
    Draw,
    Main1,
    BeginCombat,
    Attackers,
    Blockers,
    FirstStrikeDamage,
    CombatDamage,
    EndCombat,
    Main2,
    End,
    Cleanup,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Target {
    Player(Player),
    Card(CardId),
    Spell(CardId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct BoundTarget {
    pub group: u16,
    pub target: Target,
    pub object: Option<ObjectRef>,
}

/// Choices are incremental for declarations and multi-blocker damage; no combinatorial
/// enumeration of attacker subsets, blocking configurations, or damage partitions.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Action {
    ChooseSpellCost {
        cost: u16,
    },
    ChooseConniver {
        object: ObjectRef,
    },
    PayResolutionMana {
        payment: Mana,
    },
    DeclineResolutionMana,
    RecoverMilled {
        object: Option<ObjectRef>,
    },
    ChooseEntryLife {
        pay: bool,
    },
    ChooseAuraAttachment {
        object: ObjectRef,
    },
    Keep,
    Mulligan,
    Bottom {
        card: CardId,
    },
    Pass,
    PlayLand {
        card: CardId,
    },
    TapMana {
        card: CardId,
    },
    AnnounceCast {
        card: CardId,
    },
    AnnounceActivate {
        card: CardId,
        ability: u16,
        counter_payment: Counters,
    },
    AnnounceMana {
        card: CardId,
        ability: u16,
        counter_payment: Counters,
        production: Mana,
    },
    PayCosts {
        payment: Mana,
    },
    Cast {
        card: CardId,
        target: Option<Target>,
        payment: Mana,
    },
    BeginCast {
        card: CardId,
        payment: Mana,
    },
    Activate {
        card: CardId,
        ability: u16,
        target: Option<Target>,
        payment: Mana,
        counter_payment: Counters,
    },
    ActivateMana {
        card: CardId,
        ability: u16,
        payment: Mana,
        counter_payment: Counters,
        production: Mana,
    },
    ChooseMana {
        production: Mana,
    },
    BeginActivate {
        card: CardId,
        ability: u16,
        payment: Mana,
        counter_payment: Counters,
    },
    ChooseMode {
        mode: u16,
    },
    ChooseTarget {
        target: Target,
    },
    FinishTargets,
    Attack {
        card: CardId,
    },
    FinishAttackers,
    Block {
        blocker: CardId,
        attacker: CardId,
    },
    FinishBlockers,
    AssignDamage {
        target: Target,
        amount: i32,
    },
    Discard {
        card: CardId,
    },
    PutTrigger {
        id: u64,
    },
    ChooseSearch {
        card: CardId,
    },
    FinishSearch,
    ArrangeLibrary {
        card: CardId,
        destination: LibraryDestination,
    },
    ChooseOptional {
        accept: bool,
    },
    Sacrifice {
        object: ObjectRef,
    },
    DeclineEffectPayment,
    KeepLegend {
        object: ObjectRef,
    },
    Concede,
}

/// Within each destination, selected first is highest in that ordered group.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LibraryDestination {
    Top,
    Bottom,
    Graveyard,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Outcome {
    Winner(Player),
    Draw,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    IllegalAction,
    WrongActor,
    GameOver,
    InvalidDeck(String),
    StaleRevision,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for Error {}

/// Freeform is deliberately explicit: useful for training, not tournament deck legality.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DeckRules {
    #[default]
    Constructed,
    Freeform,
    /// Standard pool frozen at the compiled catalogue version, not host time.
    Standard,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub decks: [Vec<Card>; 2],
    pub seed: u64,
    #[serde(default = "first_player")]
    pub starting_player: Player,
    #[serde(default)]
    pub deck_rules: DeckRules,
}
fn first_player() -> Player {
    Player::P0
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Object {
    card: Card,
    owner: Player,
    zone: Zone,
    generation: u64,
    tapped: bool,
    // True until this creature's controller begins their next turn.
    summoning_sick: bool,
    damage: i32,
    deathtouch_since_sba: bool,
    growth: Stats,
    gained_keywords: Keywords,
    counters: Counters,
    used_abilities: u64,
    attached_to: Option<ObjectRef>,
}

impl Object {
    fn keywords(&self) -> Keywords {
        self.card.keywords().union(self.gained_keywords)
    }
    fn counter_delta(&self) -> i32 {
        i32::try_from(i64::from(self.counters.plus_one) - i64::from(self.counters.minus_one))
            .expect("counter modifier exhausted")
    }
    fn untap(&mut self) {
        if self.tapped {
            if self.counters.stun > 0 {
                self.counters.stun -= 1;
            } else {
                self.tapped = false;
            }
        }
    }

    fn local_power(&self) -> i32 {
        match self.card.effect() {
            Effect::Creature(p, _) => p + self.growth.power + self.counter_delta(),
            _ => 0,
        }
    }
    fn local_toughness(&self) -> i32 {
        match self.card.effect() {
            Effect::Creature(_, t) => t + self.growth.toughness + self.counter_delta(),
            _ => 0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PlayerState {
    spells_cast_this_turn: u64,
    cards_drawn_this_turn: u64,
    life_gain_events_this_turn: u64,
    attacked_this_turn: bool,
    life: i64,
    mana: Mana,
    library: Vec<CardId>, // top is last, bottom is first
    mulligans: u8,
    kept: bool,
    attempted_empty_draw: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SpellView {
    pub id: u64,
    pub kind: StackKind,
    pub source_card: Card,
    pub source_keywords: Keywords,
    pub source_tapped: bool,
    pub source_attachment: Option<ObjectRef>,
    pub damage_event: Option<DamageTriggerView>,
    pub discard_event: Option<DiscardTriggerView>,
    pub draw_event: Option<DrawTriggerView>,
    pub source_facts: DamageFacts,
    pub related_stack: Option<StackLink>,
    pub card: CardId,
    pub object: ObjectRef,
    pub targets: Vec<BoundTarget>,
    pub mode: Option<u16>,
    pub controller: Player,
}

/// Exact stack identity and its controller, retained after that item leaves.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct StackLink {
    pub id: u64,
    pub controller: Player,
}

/// Frozen public context for a player-damage trigger. Aggregated 'one or more'
/// events have no single dealer. These references never follow later incarnations.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct DamageTriggerView {
    pub dealer: Option<ObjectRef>,
    pub player: Player,
    pub amount: u64,
    pub combat: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum StackKind {
    Spell,
    TriggeredAbility { ability: u16 },
    ActivatedAbility { ability: u16 },
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Attacker {
    card: CardId,
    blocked: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct DamageChoice {
    source: DamageSource,
    blockers: Vec<CardId>,
    remaining: i32,
    trample: bool,
    prior_blockers_lethal: bool,
}

/// Damage-relevant characteristics captured while a source exists in its zone.
/// Battlefield status stays true for last-known battlefield sources, not cards
/// that deal damage from their hand or as spells.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct DamageFacts {
    pub card: Card,
    pub battlefield: bool,
    pub counter: bool,
    pub plus_one: bool,
    pub modified: bool,
    pub attacking: bool,
}
impl DamageFacts {
    fn from_view(view: &ObjectView) -> Self {
        Self {
            card: view.card,
            battlefield: view.zone == Zone::Battlefield,
            counter: view.counters.plus_one > 0
                || view.counters.minus_one > 0
                || view.counters.stun > 0,
            plus_one: view.counters.plus_one > 0,
            modified: view.modified,
            attacking: view.attacking,
        }
    }
}

/// Source characteristics captured when a damage event is constructed. Damage
/// need not originate from a permanent that survives the simultaneous batch.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct DamageSource {
    pub facts: DamageFacts,
    pub object: CardId,
    pub reference: ObjectRef,
    pub controller: Player,
    pub keywords: Keywords,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct DamageEvent {
    pub source: DamageSource,
    pub target: Target,
    pub amount: i32,
    pub combat: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct LifeGainEvent {
    source: DamageSource,
    amount: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Pending {
    SpellCost {
        player: Player,
    },
    ConniveOrder {
        player: Player,
    },
    ResolutionMana {
        player: Player,
    },
    RecoverMilled {
        player: Player,
        filter: RecoveryFilter,
    },
    EntryLife {
        player: Player,
    },
    AuraEntry {
        player: Player,
    },
    Mulligan(Player),
    Bottom {
        player: Player,
        remaining: u8,
    },
    Priority,
    Legend {
        player: Player,
    },
    EffectLibrary {
        player: Player,
    },
    EffectSearch {
        player: Player,
    },
    EffectMana {
        player: Player,
        production: ManaProduction,
    },
    Targets {
        player: Player,
    },
    Modes {
        player: Player,
    },
    PayCosts {
        player: Player,
    },
    OrderTriggers {
        player: Player,
    },
    EffectOptional {
        player: Player,
        skip_to: usize,
    },
    EffectChoice {
        player: Player,
    },
    EffectDiscard {
        player: Player,
        remaining: usize,
    },
    Attackers,
    Blockers,
    Damage,
    Discard,
    Terminal,
}

/// All mutable rules state, including every suspended choice and future randomness.
/// Card definitions are enum/static data, never cloned object graphs or trait objects.
#[derive(Clone, Debug, PartialEq, Eq)]
struct State {
    objects: Vec<Object>,
    free_tokens: Vec<CardId>,
    static_sources: Vec<CardId>,
    cast_sources: Vec<CardId>,
    damage_sources: Vec<CardId>,
    discard_sources: Vec<CardId>,
    draw_sources: Vec<CardId>,
    cleanup_discards: Vec<CardId>,
    triggered_this_turn: Vec<TriggerUse>,
    step_sources: Vec<CardId>,
    life_sources: Vec<CardId>,
    death_sources: Vec<CardId>,
    attachment_sources: Vec<CardId>,
    exile_links: Vec<ExileLink>,
    known_hand: Vec<ObjectRef>,
    players: [PlayerState; 2],
    rng: Rng,
    active: Player,
    priority: Player,
    starting: Player,
    turn: u32,
    step: Step,
    pending: Pending,
    sba_batch: Option<Box<sba::SbaBatch>>,
    entry_batch: Option<Box<entries::EntryBatch>>,
    mulligan_round: [bool; 2],
    stack: Vec<SpellView>,
    resolving: Option<Resolution>,
    announcing: Option<Announcement>,
    pending_triggers: Vec<SpellView>,
    ordering_triggers: Vec<SpellView>,
    next_stack_id: u64,
    passes: u8,
    land_played: bool,
    attack_declared: bool,
    attackers: Vec<Attacker>,
    blocking: Vec<CardId>, // Remains blocking even after its attacker leaves combat.
    blocks: Vec<(CardId, CardId)>, // blocker, attacker
    selected_attackers: Vec<CardId>,
    selected_blocks: Vec<(CardId, CardId)>,
    hits: Vec<DamageEvent>,
    damage_choices: Vec<DamageChoice>,
    first_strikers: Vec<CardId>,
    outcome: Option<Outcome>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Snapshot(State);

/// Cloning produces an independent search branch. Restore accepts a typed complete
/// snapshot and advances the external revision, so old action indices stay invalid.
#[derive(Clone, Debug)]
pub struct Game {
    state: State,
    revision: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ObjectView {
    pub id: CardId,
    pub reference: ObjectRef,
    pub card: Card,
    pub owner: Player,
    pub zone: Zone,
    pub tapped: bool,
    pub summoning_sick: bool,
    pub damage: i32,
    pub counters: Counters,
    pub used_abilities: u64,
    pub attached_to: Option<ObjectRef>,
    pub modified: bool,
    pub attacking: bool,
    pub power: i32,
    pub toughness: i32,
    pub keywords: Keywords,
    pub token: Option<&'static TokenInfo>,
}

/// A single zone transition with the departing object's last-known characteristics.
/// Engine internals consume this value; observations do not expose hidden transitions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ZoneChange {
    pub before: ObjectView,
    pub after: ObjectRef,
    pub destination: Zone,
    pub moved: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct PlayerView {
    pub spells_cast_this_turn: u64,
    pub cards_drawn_this_turn: u64,
    pub life_gain_events_this_turn: u64,
    pub attacked_this_turn: bool,
    pub life: i64,
    pub mana: Mana,
    pub library_count: usize,
    pub hand_count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct PaymentRequestView {
    pub required: Mana,
    pub generic: u32,
    pub counters: Counters,
    pub production: Option<Mana>,
}

/// The choosing player's uncommitted announcement costs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct PendingPaymentView {
    pub mana: Mana,
    pub counters: Counters,
}

/// Legal nonhybrid equivalents of a currently announced hybrid spell cost.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct SpellCostChoiceView {
    pub index: u16,
    pub cost: ManaCost,
}

/// Legal modes of the currently announced modal spell.
#[derive(Clone, Debug, Serialize)]
pub struct ModeChoiceView {
    pub mode: u16,
    pub text: &'static str,
}

/// Private cards and partial order of a resolving scry/surveil choice.
#[derive(Clone, Debug, Serialize)]
pub struct LibraryChoiceView {
    pub surveil: bool,
    pub remaining: Vec<ObjectView>,
    pub top: Vec<ObjectView>,
    pub bottom: Vec<ObjectView>,
    pub graveyard: Vec<ObjectView>,
}

/// Private library contents and still-uncommitted selections of a search.
#[derive(Clone, Debug, Serialize)]
pub struct SearchChoiceView {
    pub filter: crate::SearchFilter,
    pub maximum: u16,
    pub destination: crate::SearchDestination,
    pub tapped: bool,
    pub reveal: bool,
    pub cards: Vec<ObjectView>,
    pub selected: Vec<CardId>,
}

/// All milled identities and legal recovery candidates are public graveyard facts.
#[derive(Clone, Debug, Serialize)]
pub struct RecoveryChoiceView {
    pub player: Player,
    pub filter: RecoveryFilter,
    pub milled: Vec<ObjectRef>,
    pub candidates: Vec<ObjectRef>,
}

/// Private discard costs and public sacrifice choices suspended during resolution.
#[derive(Clone, Debug, Serialize)]
pub struct EffectChoiceView {
    pub kind: EffectChoiceKind,
    pub amount: u16,
    pub optional: bool,
    pub required: [u16; 2],
    pub selected: [Vec<ObjectRef>; 2],
    pub candidates: Vec<ObjectRef>,
}

/// Public state-based choices; earlier APNAP choices stay visible to later players.
#[derive(Clone, Debug, Serialize)]
pub struct LegendGroupView {
    pub controller: Player,
    pub name: &'static str,
    pub objects: Vec<ObjectRef>,
    pub keep: Option<ObjectRef>,
}

/// Public face-up exile relation, bound to both exact incarnations.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct ExileLink {
    pub source: ObjectRef,
    pub exiled: ObjectRef,
    pub return_zone: Zone,
}

/// A public triggered-ability limit consumed by an exact source incarnation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct TriggerUse {
    pub source: ObjectRef,
    pub ability: u16,
}

#[derive(Clone, Debug, Serialize)]
pub struct Observation {
    pub schema: u32,
    pub catalogue: &'static str,
    pub viewer: Player,
    pub revision: u64,
    pub actor: Option<Player>,
    pub active: Player,
    pub turn: u32,
    pub step: Step,
    pub decision: &'static str,
    pub players: [PlayerView; 2],
    pub objects: Vec<ObjectView>,
    pub exile_links: Vec<ExileLink>,
    pub triggered_this_turn: Vec<TriggerUse>,
    pub legend_choices: Vec<LegendGroupView>,
    pub stack: Vec<SpellView>,
    pub resolving: Option<SpellView>,
    pub announcing: Option<SpellView>,
    pub announcing_payment: Option<PendingPaymentView>,
    pub mana_choices: Vec<Mana>,
    pub mode_choices: Vec<ModeChoiceView>,
    pub cost_choices: Vec<SpellCostChoiceView>,
    pub announcing_cost: Option<ManaCost>,
    pub targeting_group: Option<u16>,
    pub library_choice: Option<LibraryChoiceView>,
    pub recovery_choice: Option<RecoveryChoiceView>,
    pub search_choice: Option<SearchChoiceView>,
    pub effect_choice: Option<EffectChoiceView>,
    pub aura_entry: Option<entries::AuraEntryView>,
    pub entry_life: Option<entries::EntryLifeView>,
    pub connive_choice: Option<connive::ConniveChoiceView>,
    pub known_hand: Vec<ObjectRef>,
    pub payment_request: Option<PaymentRequestView>,
    pub resolution_payment: Option<resolution_mana::ResolutionPaymentView>,
    pub waiting_triggers: Vec<SpellView>,
    pub effect_discards: Vec<CardId>,
    pub attackers: Vec<CardId>,
    pub blocking: Vec<CardId>,
    pub blocks: Vec<(CardId, CardId)>,
    pub damage_assigned: Vec<DamageEvent>,
    pub damage_source: Option<CardId>,
    pub outcome: Option<Outcome>,
    pub actions: Vec<Action>,
}

impl Game {
    pub fn new(config: Config) -> Result<Self, Error> {
        for deck in &config.decks {
            // Explicit representation bound, never silent truncation.
            if deck.len() < 7 || deck.len() > 65_535 {
                return Err(Error::InvalidDeck(
                    "supported deck size is 7..=65535".into(),
                ));
            }
            if config.deck_rules != DeckRules::Freeform {
                if deck.len() < 60 {
                    return Err(Error::InvalidDeck("constructed needs 60 cards".into()));
                }
                let mut counts = vec![0u8; Card::catalogue_len()];
                for &card in deck {
                    if let Some(limit) = card.info().deck_limit {
                        counts[card.index()] = counts[card.index()].saturating_add(1);
                        if counts[card.index()] > limit {
                            return Err(Error::InvalidDeck(format!(
                                "more than {limit} {}",
                                card.name()
                            )));
                        }
                    }
                }
            }
            for &card in deck {
                if card.is_token() {
                    return Err(Error::InvalidDeck(
                        "tokens cannot be submitted in decks".into(),
                    ));
                }
                if config.deck_rules == DeckRules::Standard && !card.is_standard_legal() {
                    return Err(Error::InvalidDeck(format!(
                        "not Standard legal in this catalogue: {}",
                        card.name()
                    )));
                }
                if !card.is_supported() {
                    return Err(Error::InvalidDeck(format!(
                        "rules not implemented: {}",
                        card.name()
                    )));
                }
            }
        }
        let mut objects = Vec::with_capacity(config.decks.iter().map(Vec::len).sum());
        let mut players = std::array::from_fn(|_| PlayerState {
            spells_cast_this_turn: 0,
            cards_drawn_this_turn: 0,
            life_gain_events_this_turn: 0,
            attacked_this_turn: false,
            life: 20,
            mana: [0; 6],
            library: Vec::new(),
            mulligans: 0,
            kept: false,
            attempted_empty_draw: false,
        });
        let mut rng = Rng(config.seed);
        for (p, deck) in [Player::P0, Player::P1].into_iter().zip(config.decks) {
            for card in deck {
                let id = objects.len() as CardId;
                objects.push(Object {
                    card,
                    owner: p,
                    zone: Zone::Library,
                    generation: 0,
                    tapped: false,
                    summoning_sick: false,
                    damage: 0,
                    deathtouch_since_sba: false,
                    growth: Stats::default(),
                    gained_keywords: Keywords::NONE,
                    counters: Counters::default(),
                    used_abilities: 0,
                    attached_to: None,
                });
                players[p.index()].library.push(id);
            }
            rng.shuffle(&mut players[p.index()].library);
        }
        let mut game = Self {
            revision: 0,
            state: State {
                objects,
                free_tokens: Vec::new(),
                static_sources: Vec::new(),
                cast_sources: Vec::new(),
                damage_sources: Vec::new(),
                discard_sources: Vec::new(),
                draw_sources: Vec::new(),
                cleanup_discards: Vec::new(),
                triggered_this_turn: Vec::new(),
                step_sources: Vec::new(),
                life_sources: Vec::new(),
                death_sources: Vec::new(),
                attachment_sources: Vec::new(),
                exile_links: Vec::new(),
                known_hand: Vec::new(),
                players,
                rng,
                active: config.starting_player,
                priority: config.starting_player,
                starting: config.starting_player,
                turn: 1,
                step: Step::Pregame,
                pending: Pending::Mulligan(config.starting_player),
                sba_batch: None,
                entry_batch: None,
                stack: Vec::new(),
                resolving: None,
                announcing: None,
                pending_triggers: Vec::new(),
                ordering_triggers: Vec::new(),
                next_stack_id: 0,
                passes: 0,
                mulligan_round: [false; 2],
                land_played: false,
                attack_declared: false,
                attackers: Vec::new(),
                blocking: Vec::new(),
                blocks: Vec::new(),
                selected_attackers: Vec::new(),
                selected_blocks: Vec::new(),
                hits: Vec::new(),
                damage_choices: Vec::new(),
                first_strikers: Vec::new(),
                outcome: None,
            },
        };
        for p in [Player::P0, Player::P1] {
            for _ in 0..7 {
                game.draw(p);
            }
        }
        Ok(game)
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }
    pub fn outcome(&self) -> Option<Outcome> {
        self.state.outcome
    }
    pub fn snapshot(&self) -> Snapshot {
        Snapshot(self.state.clone())
    }
    pub fn restore(&mut self, snapshot: &Snapshot) {
        self.state.clone_from(&snapshot.0);
        self.revision = self.revision.checked_add(1).expect("revision exhausted");
    }

    pub fn actor(&self) -> Option<Player> {
        match self.state.pending {
            Pending::Mulligan(p) | Pending::Bottom { player: p, .. } => Some(p),
            Pending::Priority => Some(self.state.priority),
            Pending::SpellCost { player }
            | Pending::ConniveOrder { player }
            | Pending::ResolutionMana { player }
            | Pending::EntryLife { player }
            | Pending::AuraEntry { player }
            | Pending::EffectChoice { player }
            | Pending::Legend { player }
            | Pending::RecoverMilled { player, .. }
            | Pending::EffectSearch { player }
            | Pending::EffectLibrary { player }
            | Pending::EffectMana { player, .. }
            | Pending::PayCosts { player }
            | Pending::Modes { player }
            | Pending::Targets { player }
            | Pending::OrderTriggers { player }
            | Pending::EffectOptional { player, .. }
            | Pending::EffectDiscard { player, .. } => Some(player),
            Pending::Attackers | Pending::Damage | Pending::Discard => Some(self.state.active),
            Pending::Blockers => Some(self.state.active.other()),
            Pending::Terminal => None,
        }
    }

    /// Reuse a caller-owned action buffer during native rollouts.
    pub fn legal_actions_into(&self, out: &mut Vec<Action>) {
        out.clear();
        if self.actor().is_none() {
            return;
        }
        self.visit_actions(|a| {
            out.push(a);
            false
        });
    }

    pub fn legal_actions(&self) -> Vec<Action> {
        let mut actions = Vec::new();
        self.legal_actions_into(&mut actions);
        actions
    }

    /// Mutation happens only after complete validation. Rejected actions leave state
    /// and revision unchanged. No user-supplied IDs are used to index before validation.
    pub fn apply(&mut self, actor: Player, action: Action) -> Result<(), Error> {
        if self.state.outcome.is_some() {
            return Err(Error::GameOver);
        }
        if action == Action::Concede {
            self.state.outcome = Some(Outcome::Winner(actor.other()));
            self.state.pending = Pending::Terminal;
        } else {
            if self.actor() != Some(actor) {
                return Err(Error::WrongActor);
            }
            if !self.visit_actions(|a| a == action) {
                return Err(Error::IllegalAction);
            }
            self.apply_legal(actor, action);
            // Resolution is atomic with respect to SBAs and priority, even when
            // a program is suspended for an optional effect or discard choice.
            loop {
                if self.advance_entries() {
                    break;
                }
                if self.state.resolving.is_none() && self.state.announcing.is_none() {
                    self.state_based_actions();
                    if self.state.entry_batch.is_some() {
                        continue;
                    }
                    self.prepare_triggers();
                }
                break;
            }
        }
        self.revision = self.revision.checked_add(1).expect("revision exhausted");
        Ok(())
    }

    pub fn apply_index(&mut self, actor: Player, revision: u64, index: usize) -> Result<(), Error> {
        if revision != self.revision {
            return Err(Error::StaleRevision);
        }
        if self.actor() != Some(actor) {
            return Err(Error::WrongActor);
        }
        let mut n = 0;
        let mut found = None;
        self.visit_actions(|a| {
            if n == index {
                found = Some(a);
                true
            } else {
                n += 1;
                false
            }
        });
        self.apply(actor, found.ok_or(Error::IllegalAction)?)
    }

    fn visit_actions(&self, mut emit: impl FnMut(Action) -> bool) -> bool {
        let s = &self.state;
        let Some(actor) = self.actor() else {
            return false;
        };
        if emit(Action::Concede) {
            return true;
        }
        // This query is read-only: identical fixed spell costs share a funding
        // result. Cache capacity affects work only, never the admitted action set.
        let mut spell_funding_cache: [Option<(Mana, u32, bool)>; 8] = [None; 8];
        match s.pending {
            Pending::ResolutionMana { .. } => {
                if self.visit_resolution_payment(&mut emit) {
                    return true;
                }
            }
            Pending::EntryLife { .. } => {
                if emit(Action::ChooseEntryLife { pay: false })
                    || self.can_pay_entry_life() && emit(Action::ChooseEntryLife { pay: true })
                {
                    return true;
                }
            }
            Pending::AuraEntry { .. } => {
                for (id, _) in s.objects.iter().enumerate() {
                    let object = self.object_ref(id as CardId);
                    if self.entry_candidate(object) && emit(Action::ChooseAuraAttachment { object })
                    {
                        return true;
                    }
                }
            }
            Pending::EffectChoice { player } => {
                let choice = s.resolving.as_ref().unwrap().choice.as_ref().unwrap();
                if choice.skip_to.is_some() && emit(Action::DeclineEffectPayment) {
                    return true;
                }
                for (index, _) in s.objects.iter().enumerate() {
                    let object = self.object_ref(index as CardId);
                    if self.effect_choice_candidate(choice, player, object) {
                        let action = match choice.kind {
                            EffectChoiceKind::Sacrifice => Action::Sacrifice { object },
                            EffectChoiceKind::Discard => Action::Discard { card: object.card },
                        };
                        if emit(action) {
                            return true;
                        }
                    }
                }
            }
            Pending::Legend { player } => {
                for group in &s.sba_batch.as_ref().unwrap().groups {
                    if group.controller == player && group.keep.is_none() {
                        for &object in &group.objects {
                            if emit(Action::KeepLegend { object }) {
                                return true;
                            }
                        }
                    }
                }
            }
            Pending::PayCosts { .. } => {
                if self.visit_cost_payment(&mut emit) {
                    return true;
                }
            }
            Pending::EffectMana { production, .. } => {
                if production.visit(|production| emit(Action::ChooseMana { production })) {
                    return true;
                }
            }
            Pending::Mulligan(p) => {
                if emit(Action::Keep) {
                    return true;
                }
                if s.players[p.index()].mulligans < 7 && emit(Action::Mulligan) {
                    return true;
                }
            }
            Pending::Bottom { .. } | Pending::Discard | Pending::EffectDiscard { .. } => {
                for (i, obj) in s.objects.iter().enumerate() {
                    if obj.owner == actor && obj.zone == Zone::Hand && !obj.card.is_token() {
                        if matches!(s.pending, Pending::EffectDiscard { .. })
                            && s.resolving.as_ref().unwrap().selected_discards[actor.index()]
                                .contains(&(i as CardId))
                        {
                            continue;
                        }
                        if s.pending == Pending::Discard
                            && s.cleanup_discards.contains(&(i as CardId))
                        {
                            continue;
                        }
                        let a = if matches!(s.pending, Pending::Bottom { .. }) {
                            Action::Bottom { card: i as CardId }
                        } else {
                            Action::Discard { card: i as CardId }
                        };
                        if emit(a) {
                            return true;
                        }
                    }
                }
            }
            Pending::Priority => {
                if emit(Action::Pass) {
                    return true;
                }
                let sorcery = actor == s.active
                    && matches!(s.step, Step::Main1 | Step::Main2)
                    && s.stack.is_empty();
                for (i, obj) in s.objects.iter().enumerate() {
                    if obj.owner != actor {
                        continue;
                    }
                    let card = i as CardId;
                    if obj.zone == Zone::Battlefield
                        && !obj.tapped
                        && obj.card.mana_color().is_some()
                        && (!obj.card.is_creature()
                            || !obj.summoning_sick
                            || self.keywords(card).contains(Keywords::HASTE))
                        && emit(Action::TapMana { card })
                    {
                        return true;
                    }
                    if matches!(obj.zone, Zone::Battlefield | Zone::Hand)
                        && self.visit_activations(card, actor, sorcery, &mut emit)
                    {
                        return true;
                    }
                    if obj.zone != Zone::Hand || obj.card.is_token() {
                        continue;
                    }
                    if obj.card.is_land() {
                        if sorcery && !s.land_played && emit(Action::PlayLand { card }) {
                            return true;
                        }
                        continue;
                    }
                    if !sorcery
                        && !obj.card.is_instant()
                        && !self.keywords(card).contains(Keywords::FLASH)
                    {
                        continue;
                    }
                    let completable = match obj.card.effect() {
                        Effect::Modal(modes) => modes.iter().any(|mode| {
                            self.targets_available(mode.targets, actor, self.object_ref(card))
                        }),
                        Effect::Spell(spec, _) | Effect::Aura(spec) => {
                            self.targets_available(spec, actor, self.object_ref(card))
                        }
                        _ => true,
                    };
                    if completable {
                        let can_fund = obj.card.cost_options().any(|cost| {
                            let colored = cost.colored;
                            let generic = cost.generic;
                            let cached = spell_funding_cache
                                .iter()
                                .flatten()
                                .find(|(required, amount, _)| {
                                    *required == colored && *amount == generic
                                })
                                .map(|(_, _, result)| *result);
                            if let Some(result) = cached {
                                result
                            } else {
                                let result = self.can_fund(FundingGoal {
                                    actor,
                                    required: colored,
                                    generic,
                                    source: None,
                                });
                                if let Some(slot) =
                                    spell_funding_cache.iter_mut().find(|slot| slot.is_none())
                                {
                                    *slot = Some((colored, generic, result));
                                }
                                result
                            }
                        });
                        if can_fund && emit(Action::AnnounceCast { card }) {
                            return true;
                        }
                    }
                    if matches!(obj.card.effect(), Effect::Modal(_)) {
                        // Modal spells use the staged path: mode, targets, payment.
                        continue;
                    }
                    let mut done = false;
                    obj.card
                        .spell_payments(s.players[actor.index()].mana, |payment| {
                            if done {
                                return;
                            }
                            let plan = match obj.card.effect() {
                                Effect::Spell(plan, _) | Effect::Aura(plan) => plan,
                                _ => TargetPlan::NONE,
                            };
                            let Some(spec) = plan.single() else {
                                if self.targets_available(plan, actor, self.object_ref(card)) {
                                    done = emit(Action::BeginCast { card, payment });
                                }
                                return;
                            };
                            let source = self.object_ref(card);
                            if spec.max > 1 {
                                let mut count = 0;
                                self.visit_targets(spec.rule, actor, source, |_| {
                                    count += 1;
                                    false
                                });
                                if count >= usize::from(spec.min) {
                                    done = emit(Action::BeginCast { card, payment });
                                }
                            } else {
                                if spec.min == 0
                                    && emit(Action::Cast {
                                        card,
                                        target: None,
                                        payment,
                                    })
                                {
                                    done = true;
                                }
                                if !done && spec.max > 0 {
                                    done = self.visit_targets(spec.rule, actor, source, |target| {
                                        emit(Action::Cast {
                                            card,
                                            target: Some(target),
                                            payment,
                                        })
                                    });
                                }
                            }
                        });
                    if done {
                        return true;
                    }
                }
            }
            Pending::Attackers => {
                if emit(Action::FinishAttackers) {
                    return true;
                }
                for (i, obj) in s.objects.iter().enumerate() {
                    let card = i as CardId;
                    if obj.owner == actor
                        && obj.zone == Zone::Battlefield
                        && obj.card.is_creature()
                        && !obj.tapped
                        && (!obj.summoning_sick || self.keywords(card).contains(Keywords::HASTE))
                        && !self.keywords(card).contains(Keywords::DEFENDER)
                        && !s.selected_attackers.contains(&card)
                        && emit(Action::Attack { card })
                    {
                        return true;
                    }
                }
            }
            Pending::Blockers => {
                if self.blocks_complete() && emit(Action::FinishBlockers) {
                    return true;
                }
                for (i, _) in s.objects.iter().enumerate() {
                    let blocker = i as CardId;
                    if !self.can_be_blocker(blocker)
                        || s.selected_blocks.iter().any(|&(b, _)| b == blocker)
                    {
                        continue;
                    }
                    for attacker in &s.attackers {
                        if self.can_block_pair(blocker, attacker.card)
                            && self.block_preserves_completion(blocker, attacker.card)
                            && emit(Action::Block {
                                blocker,
                                attacker: attacker.card,
                            })
                        {
                            return true;
                        }
                    }
                }
            }
            Pending::Damage => {
                let choice = s.damage_choices.last().expect("damage choice");
                let target = Target::Card(choice.blockers[0]);
                let minimum = if choice.blockers.len() == 1 && choice.trample {
                    if choice.prior_blockers_lethal {
                        self.lethal_assignment(choice.source, choice.blockers[0])
                            .min(choice.remaining)
                    } else {
                        choice.remaining
                    }
                } else {
                    0
                };
                for amount in minimum..=choice.remaining {
                    if emit(Action::AssignDamage { target, amount }) {
                        return true;
                    }
                }
            }
            Pending::SpellCost { player } => {
                let card = s.announcing.as_ref().unwrap().item.source_card;
                for (index, cost) in card.cost_options().enumerate() {
                    if self.can_fund(FundingGoal {
                        actor: player,
                        required: cost.colored,
                        generic: cost.generic,
                        source: None,
                    }) && emit(Action::ChooseSpellCost { cost: index as u16 })
                    {
                        return true;
                    }
                }
            }
            Pending::Modes { player } => {
                let item = &s.announcing.as_ref().unwrap().item;
                let Effect::Modal(modes) = item.source_card.effect() else {
                    unreachable!()
                };
                for (index, mode) in modes.iter().enumerate() {
                    if self.targets_available(mode.targets, player, item.object)
                        && emit(Action::ChooseMode { mode: index as u16 })
                    {
                        return true;
                    }
                }
            }
            Pending::Targets { .. } => {
                let announcement = s.announcing.as_ref().unwrap();
                let count = announcement
                    .item
                    .targets
                    .iter()
                    .filter(|bound| bound.group == announcement.group)
                    .count();
                if count >= usize::from(announcement.spec.min) && emit(Action::FinishTargets) {
                    return true;
                }
                if count < usize::from(announcement.spec.max) {
                    return self.visit_targets(
                        announcement.spec.rule,
                        actor,
                        announcement.item.object,
                        |target| {
                            !announcement.item.targets.iter().any(|bound| {
                                bound.group == announcement.group && bound.target == target
                            }) && emit(Action::ChooseTarget { target })
                        },
                    );
                }
            }
            Pending::ConniveOrder { player } => {
                for conniver in &s
                    .resolving
                    .as_ref()
                    .unwrap()
                    .connive
                    .as_ref()
                    .unwrap()
                    .remaining
                {
                    if conniver.controller == player
                        && emit(Action::ChooseConniver {
                            object: conniver.object,
                        })
                    {
                        return true;
                    }
                }
            }
            Pending::RecoverMilled { player, filter } => {
                if emit(Action::RecoverMilled { object: None }) {
                    return true;
                }
                for &object in &s.resolving.as_ref().unwrap().milled {
                    if self.can_recover_milled(player, filter, object)
                        && emit(Action::RecoverMilled {
                            object: Some(object),
                        })
                    {
                        return true;
                    }
                }
            }
            Pending::EffectSearch { player } => {
                let choice = s.resolving.as_ref().unwrap().search.as_ref().unwrap();
                if emit(Action::FinishSearch) {
                    return true;
                }
                if choice.selected.len() < usize::from(choice.spec.maximum) {
                    for &card in &s.players[player.index()].library {
                        if choice.spec.filter.matches(s.objects[card as usize].card)
                            && !choice.selected.contains(&card)
                            && emit(Action::ChooseSearch { card })
                        {
                            return true;
                        }
                    }
                }
            }
            Pending::EffectLibrary { .. } => {
                let choice = s.resolving.as_ref().unwrap().library.as_ref().unwrap();
                for &card in &choice.remaining {
                    for destination in [
                        LibraryDestination::Top,
                        if choice.surveil {
                            LibraryDestination::Graveyard
                        } else {
                            LibraryDestination::Bottom
                        },
                    ] {
                        if emit(Action::ArrangeLibrary { card, destination }) {
                            return true;
                        }
                    }
                }
            }
            Pending::OrderTriggers { player } => {
                for item in &s.ordering_triggers {
                    if item.controller == player && emit(Action::PutTrigger { id: item.id }) {
                        return true;
                    }
                }
            }
            Pending::EffectOptional { .. } => {
                if emit(Action::ChooseOptional { accept: true })
                    || emit(Action::ChooseOptional { accept: false })
                {
                    return true;
                }
            }
            Pending::Terminal => {}
        }
        false
    }

    fn apply_legal(&mut self, actor: Player, action: Action) {
        match action {
            Action::ChooseConniver { object } => {
                self.choose_conniver(object);
                self.run_program();
            }
            Action::PayResolutionMana { payment } => self.finish_resolution_payment(Some(payment)),
            Action::DeclineResolutionMana => self.finish_resolution_payment(None),
            Action::ChooseEntryLife { pay } => self.choose_entry_life(pay),
            Action::ChooseAuraAttachment { object } => self.choose_aura_attachment(object),
            Action::Keep => {
                self.state.players[actor.index()].kept = true;
                self.next_mulligan(actor);
            }
            Action::Mulligan => {
                // Mulligan decisions for both players precede the actual redraw.
                // Flag the choice using kept=false; next_mulligan executes the round.
                self.state.players[actor.index()].mulligans += 1;
                self.state.mulligan_round[actor.index()] = true;
                self.next_mulligan(actor);
            }
            Action::Bottom { card } => {
                self.move_object(card, Zone::Library);
                // move_object appends at the top; bottom ordering is this choice's rule.
                self.state.players[actor.index()].library.pop();
                // Chosen first is put lowest. Subsequent choices go above prior ones.
                let already_bottomed = match self.state.pending {
                    Pending::Bottom { remaining, .. } => {
                        self.state.players[actor.index()].mulligans - remaining
                    }
                    _ => unreachable!(),
                } as usize;
                self.state.players[actor.index()]
                    .library
                    .insert(already_bottomed, card);
                if let Pending::Bottom { remaining, .. } = &mut self.state.pending {
                    *remaining -= 1;
                    if *remaining == 0 {
                        self.next_bottom(actor);
                    }
                }
            }
            Action::Pass => {
                self.state.passes += 1;
                if self.state.passes == 1 {
                    self.state.priority = actor.other();
                } else {
                    self.state.passes = 0;
                    if self.state.stack.is_empty() {
                        self.advance_step();
                    } else {
                        self.resolve_top();
                    }
                }
            }
            Action::PlayLand { card } => {
                if matches!(
                    self.state.objects[card as usize].card.entry_tapped(),
                    EntryTapped::PayLife(_)
                ) {
                    // A land play is public even while its pre-entry choice is pending.
                    let reference = self.object_ref(card);
                    if !self.state.known_hand.contains(&reference) {
                        self.state.known_hand.push(reference);
                    }
                    let movement =
                        self.entry_move(card, Zone::Battlefield, self.enters_tapped(card));
                    self.stage_entries(vec![movement], None);
                } else {
                    self.move_object(card, Zone::Battlefield);
                }
                self.state.land_played = true;
                self.state.passes = 0;
            }
            Action::TapMana { card } => {
                let obj = &mut self.state.objects[card as usize];
                obj.tapped = true;
                self.state.players[actor.index()].mana[obj.card.mana_color().unwrap()] += 1;
                self.state.passes = 0;
            }
            Action::AnnounceCast { card } => {
                let item = self.cast_item(actor, card, [0; 6]);
                self.open_payment_announcement(item, Counters::default(), None);
            }
            Action::AnnounceActivate {
                card,
                ability,
                counter_payment,
            } => {
                let item = self.activation_item(actor, card, ability);
                self.open_payment_announcement(item, counter_payment, None);
            }
            Action::AnnounceMana {
                card,
                ability,
                counter_payment,
                production,
            } => {
                let item = self.activation_item(actor, card, ability);
                self.open_payment_announcement(item, counter_payment, Some(production));
            }
            Action::PayCosts { payment } => {
                self.state.announcing.as_mut().unwrap().payment = Some(payment);
                self.finish_announcement();
            }
            Action::Cast {
                card,
                target,
                payment,
            } => {
                let mut item = self.cast_item(actor, card, payment);
                if let Some(target) = target {
                    let bound = self.bind_target(target);
                    self.target_becomes_bound(&item, bound);
                    item.targets.push(bound);
                }
                self.push_stack_item(item);
                self.state.passes = 0;
            }
            Action::Activate {
                card,
                ability,
                target,
                payment,
                counter_payment,
            } => {
                let mut item = self.activation_item(actor, card, ability);
                if let Some(target) = target {
                    let bound = self.bind_target(target);
                    self.target_becomes_bound(&item, bound);
                    item.targets.push(bound);
                }
                self.pay_activation(&item, payment, counter_payment);
                self.push_stack_item(item);
                self.state.passes = 0;
            }
            Action::ActivateMana {
                card,
                ability,
                payment,
                counter_payment,
                production,
            } => {
                let item = self.activation_item(actor, card, ability);
                self.pay_activation(&item, payment, counter_payment);
                self.add_mana(actor, production);
            }
            Action::ChooseMana { production } => {
                self.add_mana(actor, production);
                self.run_program();
            }
            Action::BeginActivate {
                card,
                ability,
                payment,
                counter_payment,
            } => {
                let item = self.activation_item(actor, card, ability);
                if self.start_announcement(item, false) {
                    let announcement = self.state.announcing.as_mut().unwrap();
                    announcement.payment = Some(payment);
                    announcement.counter_payment = counter_payment;
                } else {
                    let item = self.state.stack.last().unwrap().clone();
                    self.pay_activation(&item, payment, counter_payment);
                }
            }
            Action::BeginCast { card, payment } => {
                let item = self.cast_item(actor, card, [0; 6]);
                if self.start_announcement(item, false) {
                    self.state.announcing.as_mut().unwrap().payment = Some(payment);
                } else {
                    for (pool, cost) in self.state.players[actor.index()]
                        .mana
                        .iter_mut()
                        .zip(payment)
                    {
                        *pool -= cost;
                    }
                    self.record_spell_cast(actor, self.state.objects[card as usize].card);
                }
            }
            Action::KeepLegend { object } => self.keep_legend(object),
            Action::ChooseSpellCost { cost } => self.choose_spell_cost(cost),
            Action::ChooseMode { mode } => self.choose_mode(mode),
            Action::ChooseTarget { target } => {
                let mut bound = self.bind_target(target);
                let item = self.state.announcing.as_ref().unwrap().item.clone();
                self.target_becomes_bound(&item, bound);
                let announcement = self.state.announcing.as_mut().unwrap();
                bound.group = announcement.group;
                announcement.item.targets.push(bound);
                if announcement
                    .item
                    .targets
                    .iter()
                    .filter(|bound| bound.group == announcement.group)
                    .count()
                    == usize::from(announcement.spec.max)
                {
                    self.finish_target_group();
                }
            }
            Action::FinishTargets => self.finish_target_group(),
            Action::Attack { card } => self.state.selected_attackers.push(card),
            Action::FinishAttackers => {
                self.state.attack_declared = !self.state.selected_attackers.is_empty();
                self.state.players[actor.index()].attacked_this_turn |= self.state.attack_declared;
                let declared = std::mem::take(&mut self.state.selected_attackers);
                for card in declared {
                    if !self.keywords(card).contains(Keywords::VIGILANCE) {
                        self.state.objects[card as usize].tapped = true;
                    }
                    self.state.attackers.push(Attacker {
                        card,
                        blocked: false,
                    });
                    self.attack_triggers(&self.object_view(card));
                }
                self.give_priority();
            }
            Action::Block { blocker, attacker } => {
                self.state.selected_blocks.push((blocker, attacker))
            }
            Action::FinishBlockers => {
                self.state
                    .blocking
                    .extend(self.state.selected_blocks.iter().map(|&(b, _)| b));
                self.state.blocks.append(&mut self.state.selected_blocks);
                for attacker in &mut self.state.attackers {
                    attacker.blocked = self.state.blocks.iter().any(|&(_, a)| a == attacker.card);
                }
                self.give_priority();
            }
            Action::AssignDamage { target, amount } => {
                let Target::Card(blocker) = target else {
                    unreachable!()
                };
                let source = self.state.damage_choices.last().unwrap().source;
                let lethal = amount >= self.lethal_assignment(source, blocker);
                self.queue_hit(source, target, amount, true);
                let choice = self.state.damage_choices.last_mut().unwrap();
                choice.remaining -= amount;
                choice.blockers.remove(0);
                choice.prior_blockers_lethal &= lethal;
                if choice.blockers.is_empty() {
                    let choice = self.state.damage_choices.pop().unwrap();
                    self.queue_hit(
                        source,
                        Target::Player(self.state.active.other()),
                        choice.remaining,
                        true,
                    );
                }
                self.advance_damage_choices();
            }
            Action::Sacrifice { object } => self.choose_effect_object(actor, object),
            Action::DeclineEffectPayment => self.decline_effect_payment(),
            Action::Discard { card } => {
                if matches!(self.state.pending, Pending::EffectChoice { .. }) {
                    self.choose_effect_object(actor, self.object_ref(card));
                } else if matches!(self.state.pending, Pending::EffectDiscard { .. }) {
                    let resolution = self.state.resolving.as_mut().unwrap();
                    resolution.discards[actor.index()] -= 1;
                    resolution.selected_discards[actor.index()].push(card);
                    self.run_program();
                } else {
                    self.state.cleanup_discards.push(card);
                    if self.hand_count(actor) - self.state.cleanup_discards.len() <= 7 {
                        let discarded = std::mem::take(&mut self.state.cleanup_discards);
                        self.discard_batch(&discarded, None);
                        self.finish_cleanup();
                    }
                }
            }
            Action::PutTrigger { id } => {
                let index = self
                    .state
                    .ordering_triggers
                    .iter()
                    .position(|item| item.id == id)
                    .unwrap();
                let item = self.state.ordering_triggers.remove(index);
                if !self.start_announcement(item, true) {
                    self.continue_trigger_order();
                }
            }
            Action::RecoverMilled { object } => self.recover_milled(object),
            Action::ChooseSearch { card } => self.choose_search(card),
            Action::FinishSearch => self.finish_search(actor),
            Action::ArrangeLibrary { card, destination } => {
                self.arrange_library(actor, card, destination);
            }
            Action::ChooseOptional { accept } => {
                let Pending::EffectOptional { skip_to, .. } = self.state.pending else {
                    unreachable!()
                };
                if !accept {
                    self.state.resolving.as_mut().unwrap().pc = skip_to;
                }
                self.run_program();
            }
            Action::Concede => unreachable!(),
        }
    }

    fn next_mulligan(&mut self, actor: Player) {
        let start = self.state.starting;
        if actor == start && !self.state.players[start.other().index()].kept {
            self.state.pending = Pending::Mulligan(start.other());
            return;
        }
        // Everyone who chose to mulligan shuffles and draws simultaneously.
        for p in [start, start.other()] {
            if self.state.mulligan_round[p.index()] {
                let hand: Vec<_> = self
                    .state
                    .objects
                    .iter()
                    .enumerate()
                    .filter(|(_, obj)| obj.owner == p && obj.zone == Zone::Hand)
                    .map(|(i, _)| i as CardId)
                    .collect();
                for id in hand {
                    self.move_object(id, Zone::Library);
                }
                self.state
                    .rng
                    .shuffle(&mut self.state.players[p.index()].library);
                for _ in 0..7 {
                    self.draw(p);
                }
            }
        }
        for p in [start, start.other()] {
            if self.state.mulligan_round[p.index()] {
                self.state.pending = Pending::Bottom {
                    player: p,
                    remaining: self.state.players[p.index()].mulligans,
                };
                return;
            }
        }
        self.begin_mulligan_round();
    }

    fn next_bottom(&mut self, actor: Player) {
        self.state.mulligan_round[actor.index()] = false;
        let other = self.state.starting.other();
        if actor == self.state.starting && self.state.mulligan_round[other.index()] {
            self.state.pending = Pending::Bottom {
                player: other,
                remaining: self.state.players[other.index()].mulligans,
            };
        } else {
            self.begin_mulligan_round();
        }
    }

    fn begin_mulligan_round(&mut self) {
        for p in [self.state.starting, self.state.starting.other()] {
            if self.state.players[p.index()].mulligans == 7 {
                self.state.players[p.index()].kept = true;
            }
            if !self.state.players[p.index()].kept {
                self.state.pending = Pending::Mulligan(p);
                return;
            }
        }
        self.begin_turn();
    }

    fn hand_count(&self, p: Player) -> usize {
        self.state
            .objects
            .iter()
            .filter(|o| o.owner == p && o.zone == Zone::Hand && !o.card.is_token())
            .count()
    }

    fn draw(&mut self, p: Player) {
        match self.state.players[p.index()].library.last().copied() {
            Some(id) => {
                self.move_object(id, Zone::Hand);
                self.card_drawn(p);
            }
            None => self.state.players[p.index()].attempted_empty_draw = true,
        }
    }

    fn on_battlefield(&self, id: CardId) -> bool {
        self.state
            .objects
            .get(id as usize)
            .is_some_and(|o| o.zone == Zone::Battlefield)
    }

    fn object_ref(&self, card: CardId) -> ObjectRef {
        ObjectRef {
            card,
            generation: self.state.objects[card as usize].generation,
        }
    }

    fn object_view(&self, id: CardId) -> ObjectView {
        let obj = &self.state.objects[id as usize];
        let stats = self.stats(id);
        ObjectView {
            id,
            reference: self.object_ref(id),
            card: obj.card,
            owner: obj.owner,
            zone: obj.zone,
            tapped: obj.tapped,
            summoning_sick: obj.summoning_sick,
            damage: obj.damage,
            counters: obj.counters,
            used_abilities: obj.used_abilities,
            attached_to: obj.attached_to,
            modified: self.is_modified(id),
            attacking: self.state.attackers.iter().any(|a| a.card == id),
            power: stats.power,
            toughness: stats.toughness,
            keywords: self.keywords(id),
            token: obj.card.token_info(),
        }
    }

    /// All real zone changes go through here. The library's default insertion is
    /// its top; effects that specify ordering reposition after the transition.
    /// The returned event is owned so later trigger processing can retain LKI.
    fn move_object(&mut self, id: CardId, destination: Zone) -> ZoneChange {
        let sources = if self.state.objects[id as usize].zone == Zone::Battlefield
            && destination == Zone::Graveyard
        {
            self.death_trigger_sources()
        } else {
            Vec::new()
        };
        let event = self.move_object_raw(id, destination);
        self.finish_zone_events(std::slice::from_ref(&event), &sources);
        event
    }

    /// Capture every pre-event characteristic before any simultaneous departure.
    fn move_objects_raw(&mut self, ids: &[CardId], destination: Zone) -> Vec<ZoneChange> {
        let before: Vec<_> = ids
            .iter()
            .map(|&id| {
                assert!(
                    destination != Zone::Battlefield
                        || !matches!(
                            self.state.objects[id as usize].card.entry_tapped(),
                            EntryTapped::PayLife(_)
                        ),
                    "choice-bearing entry must be staged"
                );
                (
                    self.object_view(id),
                    destination == Zone::Battlefield && self.enters_tapped(id),
                )
            })
            .collect();
        before
            .into_iter()
            .map(|(before, tapped)| self.transition_object(before, destination, tapped))
            .collect()
    }

    fn enters_tapped(&self, id: CardId) -> bool {
        let object = &self.state.objects[id as usize];
        match object.card.entry_tapped() {
            EntryTapped::Never | EntryTapped::PayLife(_) => false,
            EntryTapped::UnlessPlaneswalker => !self.state.objects.iter().any(|o| {
                o.zone == Zone::Battlefield
                    && o.owner == object.owner
                    && o.card.kind_mask() & 64 != 0
            }),
            EntryTapped::Always => true,
            EntryTapped::UnlessLowLife => self.state.players.iter().all(|p| p.life > 13),
            EntryTapped::UnlessBasic => !self.state.objects.iter().any(|o| {
                o.zone == Zone::Battlefield && o.owner == object.owner && o.card.is_basic_land()
            }),
            EntryTapped::UnlessAtMostTwo | EntryTapped::UnlessAtLeastTwo => {
                let lands = self
                    .state
                    .objects
                    .iter()
                    .enumerate()
                    .filter(|(other, o)| {
                        *other != id as usize
                            && o.zone == Zone::Battlefield
                            && o.owner == object.owner
                            && o.card.is_land()
                    })
                    .count();
                match object.card.entry_tapped() {
                    EntryTapped::UnlessAtMostTwo => lands > 2,
                    _ => lands < 2,
                }
            }
        }
    }

    fn move_object_raw(&mut self, id: CardId, destination: Zone) -> ZoneChange {
        let tapped = destination == Zone::Battlefield && self.enters_tapped(id);
        self.move_object_raw_with_entry(id, destination, tapped)
    }

    fn move_object_raw_with_entry(
        &mut self,
        id: CardId,
        destination: Zone,
        tapped: bool,
    ) -> ZoneChange {
        assert!(
            destination != Zone::Battlefield
                || !matches!(
                    self.state.objects[id as usize].card.entry_tapped(),
                    EntryTapped::PayLife(_)
                ),
            "choice-bearing entry must be staged"
        );
        let before = self.object_view(id);
        self.transition_object(before, destination, tapped)
    }

    fn transition_object(
        &mut self,
        before: ObjectView,
        destination: Zone,
        tapped: bool,
    ) -> ZoneChange {
        let id = before.id;
        // CR111.8: after leaving the battlefield, a token cannot move again.
        if before.card.is_token() && before.zone != Zone::Battlefield {
            return ZoneChange {
                after: before.reference,
                destination: before.zone,
                before,
                moved: false,
            };
        }
        assert_ne!(
            before.zone, destination,
            "same-zone moves are ordering operations"
        );
        let generation = before
            .reference
            .generation
            .checked_add(1)
            .expect("object generation exhausted");
        if before.zone == Zone::Hand {
            self.state
                .known_hand
                .retain(|&reference| reference != before.reference);
        }
        if before.zone == Zone::Library {
            let library = &mut self.state.players[before.owner.index()].library;
            let position = library
                .iter()
                .rposition(|&card| card == id)
                .expect("library membership");
            library.remove(position);
        }
        if before.zone == Zone::Exile {
            self.state
                .exile_links
                .retain(|link| link.exiled != before.reference);
        }
        if before.zone == Zone::Stack {
            self.state
                .stack
                .retain(|spell| spell.kind != StackKind::Spell || spell.object != before.reference);
        }
        if before.zone == Zone::Battlefield {
            // Abilities use the source's actual last-known keywords if it leaves,
            // including gains made after that ability was announced.
            for item in self
                .state
                .stack
                .iter_mut()
                .chain(&mut self.state.pending_triggers)
                .chain(&mut self.state.ordering_triggers)
                .chain(self.state.resolving.iter_mut().map(|r| &mut r.item))
                .chain(self.state.announcing.iter_mut().map(|a| &mut a.item))
            {
                if item.object == before.reference {
                    item.source_keywords = before.keywords;
                    item.source_tapped = before.tapped;
                    item.source_attachment = before.attached_to;
                    item.source_facts = DamageFacts::from_view(&before);
                }
            }
            self.state.static_sources.retain(|&source| source != id);
            self.state.cast_sources.retain(|&source| source != id);
            self.state.damage_sources.retain(|&source| source != id);
            self.state.discard_sources.retain(|&source| source != id);
            self.state.draw_sources.retain(|&source| source != id);
            self.state.step_sources.retain(|&source| source != id);
            self.state.life_sources.retain(|&source| source != id);
            self.state.death_sources.retain(|&source| source != id);
            self.state.attachment_sources.retain(|&source| source != id);
            self.state.attackers.retain(|attacker| attacker.card != id);
            self.state.blocking.retain(|&blocker| blocker != id);
            self.state
                .blocks
                .retain(|&(blocker, attacker)| blocker != id && attacker != id);
            self.state.selected_attackers.retain(|&card| card != id);
            self.state
                .selected_blocks
                .retain(|&(blocker, attacker)| blocker != id && attacker != id);
            self.state.first_strikers.retain(|&card| card != id);
            // A surviving attacker remains blocked after its last blocker leaves.
        }
        let obj = &mut self.state.objects[id as usize];
        obj.zone = destination;
        obj.generation = generation;
        obj.tapped = tapped;
        obj.summoning_sick = destination == Zone::Battlefield;
        obj.damage = 0;
        obj.deathtouch_since_sba = false;
        obj.growth = Stats::default();
        obj.gained_keywords = Keywords::NONE;
        obj.counters = if destination == Zone::Battlefield {
            obj.card.enters_counters()
        } else {
            Counters::default()
        };
        obj.used_abilities = 0;
        obj.attached_to = None;
        if destination == Zone::Library {
            self.state.players[obj.owner.index()].library.push(id);
        }
        if destination == Zone::Battlefield {
            self.register_battlefield_sources(id);
        }
        ZoneChange {
            before,
            after: self.object_ref(id),
            destination,
            moved: true,
        }
    }

    fn give_priority(&mut self) {
        self.state.pending = Pending::Priority;
        self.state.priority = self.state.active;
        self.state.passes = 0;
    }

    fn begin_turn(&mut self) {
        self.state.triggered_this_turn.clear();
        for player in &mut self.state.players {
            player.spells_cast_this_turn = 0;
            player.cards_drawn_this_turn = 0;
            player.life_gain_events_this_turn = 0;
            player.attacked_this_turn = false;
        }
        self.state.land_played = false;
        self.state.attack_declared = false;
        self.state.attackers.clear();
        self.state.blocking.clear();
        self.state.blocks.clear();
        self.state.first_strikers.clear();
        for obj in &mut self.state.objects {
            obj.used_abilities = 0;
            if obj.owner == self.state.active && obj.zone == Zone::Battlefield {
                obj.untap();
                obj.summoning_sick = false;
            }
        }
        self.begin_step(Step::Upkeep);
        self.give_priority();
    }

    fn advance_step(&mut self) {
        for player in &mut self.state.players {
            player.mana = [0; 6];
        }
        match self.state.step {
            Step::Upkeep => {
                if self.state.turn == 1 {
                    self.begin_step(Step::Main1);
                } else {
                    self.begin_step(Step::Draw);
                    self.draw(self.state.active);
                }
                self.give_priority();
            }
            Step::Draw => {
                self.begin_step(Step::Main1);
                self.give_priority();
            }
            Step::Main1 => {
                self.begin_step(Step::BeginCombat);
                self.give_priority();
            }
            Step::BeginCombat => {
                self.state.step = Step::Attackers;
                self.state.pending = Pending::Attackers;
            }
            Step::Attackers => {
                if !self.state.attack_declared {
                    self.begin_step(Step::EndCombat);
                    self.give_priority();
                } else {
                    self.state.step = Step::Blockers;
                    self.state.pending = Pending::Blockers;
                }
            }
            Step::Blockers => {
                self.state.first_strikers = self
                    .combatants()
                    .into_iter()
                    .filter(|&id| {
                        let keywords = self.keywords(id);
                        keywords.contains(Keywords::FIRST_STRIKE)
                            || keywords.contains(Keywords::DOUBLE_STRIKE)
                    })
                    .collect();
                self.state.step = if self.state.first_strikers.is_empty() {
                    Step::CombatDamage
                } else {
                    Step::FirstStrikeDamage
                };
                self.prepare_damage();
            }
            Step::FirstStrikeDamage => {
                self.state.step = Step::CombatDamage;
                self.prepare_damage();
            }
            Step::CombatDamage => {
                self.begin_step(Step::EndCombat);
                self.give_priority();
            }
            Step::EndCombat => {
                self.state.attackers.clear();
                self.state.blocking.clear();
                self.state.attack_declared = false;
                self.state.blocks.clear();
                self.state.first_strikers.clear();
                self.begin_step(Step::Main2);
                self.give_priority();
            }
            Step::Main2 => {
                self.begin_step(Step::End);
                self.give_priority();
            }
            Step::End | Step::Cleanup => {
                self.state.step = Step::Cleanup;
                if self.hand_count(self.state.active) > 7 {
                    self.state.pending = Pending::Discard;
                } else {
                    self.finish_cleanup();
                }
            }
            Step::Pregame => unreachable!(),
        }
    }

    fn finish_cleanup(&mut self) {
        // Damage removal and expiration of until-end-of-turn effects are simultaneous.
        for obj in &mut self.state.objects {
            obj.damage = 0;
            obj.deathtouch_since_sba = false;
            obj.growth = Stats::default();
            obj.gained_keywords = Keywords::NONE;
        }
        // Losing temporary growth can expose lethal negative counters. Apply
        // SBAs before advancing the turn; CR514.3 gives an extra priority cycle.
        if !self.state.pending_triggers.is_empty()
            || self.state.objects.iter().enumerate().any(|(id, obj)| {
                obj.zone == Zone::Battlefield
                    && obj.card.is_creature()
                    && self.stats(id as CardId).toughness <= 0
            })
        {
            self.give_priority();
            self.state_based_actions();
            return;
        }
        self.state.active = self.state.active.other();
        self.state.turn += 1;
        self.begin_turn();
    }

    fn damage_source(&self, object: CardId) -> DamageSource {
        let obj = &self.state.objects[object as usize];
        // Control-changing cards are not admitted yet. This is the single place
        // where current controller must be used once control layers are added.
        DamageSource {
            facts: self.damage_facts(object),
            object,
            reference: self.object_ref(object),
            controller: obj.owner,
            keywords: self.keywords(object),
        }
    }

    fn queue_hit(&mut self, source: DamageSource, target: Target, amount: i32, combat: bool) {
        if amount > 0 {
            self.state.hits.push(DamageEvent {
                source,
                target,
                amount,
                combat,
            });
        }
    }

    fn apply_damage_batch(&mut self, events: &[DamageEvent]) -> Vec<LifeGainEvent> {
        let mut gains: Vec<LifeGainEvent> = Vec::new();
        let mut gain_indices: std::collections::BTreeMap<DamageSource, usize> =
            std::collections::BTreeMap::new();
        for event in events {
            if event.amount <= 0 {
                continue;
            }
            match event.target {
                Target::Player(p) => self.state.players[p.index()].life -= i64::from(event.amount),
                Target::Card(id) => {
                    let obj = &mut self.state.objects[id as usize];
                    obj.damage += event.amount;
                    obj.deathtouch_since_sba |=
                        event.source.keywords.contains(Keywords::DEATHTOUCH);
                }
                Target::Spell(_) => unreachable!("damage cannot target stack objects"),
            }
            if event.source.keywords.contains(Keywords::LIFELINK) {
                // One source damaging multiple objects simultaneously causes a
                // single life-gain event; distinct sources remain distinct.
                if let Some(&index) = gain_indices.get(&event.source) {
                    gains[index].amount += event.amount;
                } else {
                    gain_indices.insert(event.source, gains.len());
                    gains.push(LifeGainEvent {
                        source: event.source,
                        amount: event.amount,
                    });
                }
            }
        }
        for gain in &gains {
            self.gain_life(gain.source.controller, i64::from(gain.amount));
        }
        self.player_damage_triggers(events);
        // Trigger events are recorded before SBAs, after the whole damage batch.
        gains
    }

    fn combatants(&self) -> Vec<CardId> {
        self.state
            .attackers
            .iter()
            .map(|attacker| attacker.card)
            .chain(self.state.blocking.iter().copied())
            .filter(|&id| self.on_battlefield(id))
            .collect()
    }

    fn assigns_this_step(&self, id: CardId) -> bool {
        if self.state.step == Step::FirstStrikeDamage {
            self.state.first_strikers.contains(&id)
        } else {
            !self.state.first_strikers.contains(&id)
                || self.keywords(id).contains(Keywords::DOUBLE_STRIKE)
        }
    }

    fn lethal_assignment(&self, source: DamageSource, blocker: CardId) -> i32 {
        let obj = &self.state.objects[blocker as usize];
        let remaining = (self.stats(blocker).toughness - obj.damage).max(0);
        if source.keywords.contains(Keywords::DEATHTOUCH) {
            remaining.min(1)
        } else {
            remaining
        }
    }

    fn prepare_damage(&mut self) {
        self.state.hits.clear();
        self.state.damage_choices.clear();
        // Clone only the declaration list so construction may append events.
        for attacker in self.state.attackers.clone() {
            if !self.on_battlefield(attacker.card) {
                continue;
            }
            let blockers: Vec<_> = self
                .state
                .blocks
                .iter()
                .filter(|&&(b, a)| a == attacker.card && self.on_battlefield(b))
                .map(|&(b, _)| b)
                .collect();
            for &blocker in &blockers {
                if self.assigns_this_step(blocker) {
                    let source = self.damage_source(blocker);
                    let amount = self.stats(blocker).power.max(0);
                    self.queue_hit(source, Target::Card(attacker.card), amount, true);
                }
            }
            if !self.assigns_this_step(attacker.card) {
                continue;
            }
            let source = self.damage_source(attacker.card);
            let power = self.stats(attacker.card).power.max(0);
            let trample = source.keywords.contains(Keywords::TRAMPLE);
            if !attacker.blocked || (trample && blockers.is_empty()) {
                self.queue_hit(
                    source,
                    Target::Player(self.state.active.other()),
                    power,
                    true,
                );
            } else if !blockers.is_empty() && power > 0 {
                self.state.damage_choices.push(DamageChoice {
                    source,
                    blockers,
                    remaining: power,
                    trample,
                    prior_blockers_lethal: true,
                });
            }
        }
        self.advance_damage_choices();
    }

    fn advance_damage_choices(&mut self) {
        while let Some(choice) = self.state.damage_choices.last() {
            let forced = choice.remaining == 0
                || (choice.blockers.len() == 1
                    && (!choice.trample
                        || !choice.prior_blockers_lethal
                        || choice.remaining
                            <= self.lethal_assignment(choice.source, choice.blockers[0])));
            if !forced {
                self.state.pending = Pending::Damage;
                return;
            }
            let choice = self.state.damage_choices.pop().unwrap();
            self.queue_hit(
                choice.source,
                Target::Card(choice.blockers[0]),
                choice.remaining,
                true,
            );
        }
        self.deal_combat_damage();
    }

    fn deal_combat_damage(&mut self) {
        let hits = std::mem::take(&mut self.state.hits);
        self.apply_damage_batch(&hits);
        self.give_priority();
    }

    /// Player-facing information only. Unrevealed opponent hands and RNG state
    /// are absent. Explicit library looks/searches and partial declarations are
    /// projected only to their actor; known revealed hand incarnations are public.
    pub fn observe(&self, viewer: Player) -> Observation {
        let s = &self.state;
        let owns_choice = self.actor() == Some(viewer);
        let objects = s
            .objects
            .iter()
            .enumerate()
            .filter(|(i, o)| {
                o.zone != Zone::Outside
                    && o.zone != Zone::Library
                    && (o.zone != Zone::Hand
                        || o.owner == viewer
                        || s.known_hand.contains(&self.object_ref(*i as CardId)))
            })
            .map(|(i, _)| self.object_view(i as CardId))
            .collect();
        Observation {
            schema: 33,
            catalogue: crate::CATALOGUE_VERSION,
            viewer,
            revision: self.revision,
            actor: self.actor(),
            active: s.active,
            turn: s.turn,
            step: s.step,
            decision: match s.pending {
                Pending::ResolutionMana { .. } => "resolution_mana",
                Pending::ConniveOrder { .. } => "connive_order",
                Pending::Mulligan(_) => "mulligan",
                Pending::Bottom { .. } => "bottom",
                Pending::Priority => "priority",
                Pending::Legend { .. } => "legend_rule",
                Pending::Targets { .. } => "targets",
                Pending::SpellCost { .. } => "spell_cost",
                Pending::Modes { .. } => "spell_mode",
                Pending::PayCosts { .. } => "pay_costs",
                Pending::OrderTriggers { .. } => "order_triggers",
                Pending::EffectOptional { .. } => "optional_effect",
                Pending::EffectMana { .. } => "mana_choice",
                Pending::EffectLibrary { .. } => "library_choice",
                Pending::RecoverMilled { .. } => "recover_milled",
                Pending::EffectSearch { .. } => "library_search",
                Pending::EffectDiscard { .. } => "effect_discard",
                Pending::EffectChoice { .. } => "effect_choice",
                Pending::EntryLife { .. } => "entry_life",
                Pending::AuraEntry { .. } => "aura_entry",
                Pending::Attackers => "attackers",
                Pending::Blockers => "blockers",
                Pending::Damage => "damage",
                Pending::Discard => "discard",
                Pending::Terminal => "terminal",
            },
            players: std::array::from_fn(|i| PlayerView {
                spells_cast_this_turn: s.players[i].spells_cast_this_turn,
                cards_drawn_this_turn: s.players[i].cards_drawn_this_turn,
                life_gain_events_this_turn: s.players[i].life_gain_events_this_turn,
                attacked_this_turn: s.players[i].attacked_this_turn,
                life: s.players[i].life,
                mana: s.players[i].mana,
                library_count: s.players[i].library.len(),
                hand_count: self.hand_count(if i == 0 { Player::P0 } else { Player::P1 }),
            }),
            recovery_choice: match s.pending {
                Pending::RecoverMilled { player, filter } => Some(RecoveryChoiceView {
                    player,
                    filter,
                    milled: s.resolving.as_ref().unwrap().milled.clone(),
                    candidates: s
                        .resolving
                        .as_ref()
                        .unwrap()
                        .milled
                        .iter()
                        .copied()
                        .filter(|&object| self.can_recover_milled(player, filter, object))
                        .collect(),
                }),
                _ => None,
            },
            effect_choice: self.effect_choice_view(viewer),
            aura_entry: self.aura_entry_view(viewer),
            entry_life: self.entry_life_view(viewer),
            objects,
            exile_links: s.exile_links.clone(),
            triggered_this_turn: s.triggered_this_turn.clone(),
            legend_choices: s
                .sba_batch
                .as_ref()
                .map(|batch| {
                    batch
                        .groups
                        .iter()
                        .map(|group| LegendGroupView {
                            controller: group.controller,
                            name: group.name,
                            objects: group.objects.clone(),
                            keep: group.keep,
                        })
                        .collect()
                })
                .unwrap_or_default(),
            stack: s.stack.clone(),
            resolving: s
                .resolving
                .as_ref()
                .map(|resolution| resolution.item.clone()),
            announcing: s.announcing.as_ref().map(|announcement| {
                let mut item = announcement.item.clone();
                if !owns_choice {
                    item.targets.clear();
                }
                item
            }),
            announcing_payment: if owns_choice {
                s.announcing.as_ref().and_then(|announcement| {
                    announcement.payment.map(|mana| PendingPaymentView {
                        mana,
                        counters: announcement.counter_payment,
                    })
                })
            } else {
                None
            },
            resolution_payment: s
                .resolving
                .as_ref()
                .and_then(|r| r.mana_payment.as_deref())
                .map(|p| resolution_mana::ResolutionPaymentView {
                    payer: p.payer,
                    cost: p.cost,
                    victim: p.victim,
                    exile: p.exile,
                }),
            payment_request: self.announcement_payment_request(viewer),
            targeting_group: if owns_choice && matches!(s.pending, Pending::Targets { .. }) {
                s.announcing.as_ref().map(|announcement| announcement.group)
            } else {
                None
            },
            announcing_cost: s.announcing.as_ref().and_then(|a| a.cost),
            cost_choices: if owns_choice && matches!(s.pending, Pending::SpellCost { .. }) {
                let card = s.announcing.as_ref().unwrap().item.source_card;
                card.cost_options()
                    .enumerate()
                    .filter(|(_, cost)| {
                        self.can_fund(FundingGoal {
                            actor: viewer,
                            required: cost.colored,
                            generic: cost.generic,
                            source: None,
                        })
                    })
                    .map(|(index, cost)| SpellCostChoiceView {
                        index: index as u16,
                        cost,
                    })
                    .collect()
            } else {
                Vec::new()
            },
            mode_choices: if owns_choice && matches!(s.pending, Pending::Modes { .. }) {
                let item = &s.announcing.as_ref().unwrap().item;
                let Effect::Modal(modes) = item.source_card.effect() else {
                    unreachable!()
                };
                modes
                    .iter()
                    .enumerate()
                    .filter(|(_, mode)| {
                        self.targets_available(mode.targets, item.controller, item.object)
                    })
                    .map(|(index, mode)| ModeChoiceView {
                        mode: index as u16,
                        text: mode.text,
                    })
                    .collect()
            } else {
                Vec::new()
            },
            mana_choices: {
                let mut options = Vec::new();
                if owns_choice && let Pending::EffectMana { production, .. } = s.pending {
                    production.visit(|mana| {
                        options.push(mana);
                        false
                    });
                }
                options
            },
            library_choice: if owns_choice {
                s.resolving
                    .as_ref()
                    .and_then(|r| r.library.as_ref())
                    .map(|choice| LibraryChoiceView {
                        surveil: choice.surveil,
                        remaining: choice
                            .remaining
                            .iter()
                            .map(|&id| self.object_view(id))
                            .collect(),
                        top: choice.top.iter().map(|&id| self.object_view(id)).collect(),
                        bottom: choice
                            .bottom
                            .iter()
                            .map(|&id| self.object_view(id))
                            .collect(),
                        graveyard: choice
                            .graveyard
                            .iter()
                            .map(|&id| self.object_view(id))
                            .collect(),
                    })
            } else {
                None
            },
            search_choice: if owns_choice {
                s.resolving
                    .as_ref()
                    .and_then(|r| r.search.as_ref())
                    .map(|choice| SearchChoiceView {
                        filter: choice.spec.filter,
                        maximum: choice.spec.maximum,
                        destination: choice.spec.destination,
                        tapped: choice.spec.tapped,
                        reveal: choice.spec.reveal,
                        cards: s.players[viewer.index()]
                            .library
                            .iter()
                            .map(|&id| self.object_view(id))
                            .collect(),
                        selected: choice.selected.clone(),
                    })
            } else {
                None
            },
            connive_choice: s
                .resolving
                .as_ref()
                .and_then(|r| r.connive.as_ref())
                .map(|c| connive::ConniveChoiceView {
                    amount: c.amount,
                    remaining: c.remaining.clone(),
                    current: c.current,
                }),
            known_hand: s.known_hand.clone(),
            waiting_triggers: s.ordering_triggers.clone(),
            effect_discards: s
                .resolving
                .as_ref()
                .map(|resolution| resolution.selected_discards[viewer.index()].clone())
                .unwrap_or_default(),
            attackers: if owns_choice && s.pending == Pending::Attackers {
                s.selected_attackers.clone()
            } else {
                s.attackers.iter().map(|a| a.card).collect()
            },
            blocking: if owns_choice && s.pending == Pending::Blockers {
                s.selected_blocks.iter().map(|&(b, _)| b).collect()
            } else {
                s.blocking.clone()
            },
            blocks: if owns_choice && s.pending == Pending::Blockers {
                s.selected_blocks.clone()
            } else {
                s.blocks.clone()
            },
            damage_assigned: if owns_choice {
                s.hits.clone()
            } else {
                Vec::new()
            },
            damage_source: if owns_choice {
                s.damage_choices.last().map(|choice| choice.source.object)
            } else {
                None
            },
            outcome: s.outcome,
            actions: if owns_choice {
                self.legal_actions()
            } else {
                Vec::new()
            },
        }
    }

    /// Bytes occupied by state structs plus initialized vector elements, excluding
    /// allocator overhead, spare capacity, snapshots, and static card definitions.
    pub fn state_bytes(&self) -> usize {
        let s = &self.state;
        std::mem::size_of::<State>()
            + s.objects.len() * std::mem::size_of::<Object>()
            + s.exile_links.len() * std::mem::size_of::<ExileLink>()
            + s.triggered_this_turn.len() * std::mem::size_of::<TriggerUse>()
            + s.sba_batch.as_ref().map(|batch| batch.bytes()).unwrap_or(0)
            + s.entry_batch
                .as_ref()
                .map(|batch| batch.bytes())
                .unwrap_or(0)
            + s.known_hand.len() * std::mem::size_of::<ObjectRef>()
            + (s.free_tokens.len()
                + s.static_sources.len()
                + s.cast_sources.len()
                + s.damage_sources.len()
                + s.discard_sources.len()
                + s.draw_sources.len()
                + s.cleanup_discards.len()
                + s.step_sources.len()
                + s.life_sources.len()
                + s.death_sources.len()
                + s.attachment_sources.len())
                * std::mem::size_of::<CardId>()
            + s.players
                .iter()
                .map(|p| p.library.len() * std::mem::size_of::<CardId>())
                .sum::<usize>()
            + (s.stack.len() + s.pending_triggers.len() + s.ordering_triggers.len())
                * std::mem::size_of::<SpellView>()
            + s.resolving
                .as_ref()
                .map(|resolution| {
                    resolution.connive.as_ref().map_or(0, |c| c.bytes())
                        + resolution.mana_payment.as_ref().map_or(0, |_| {
                            std::mem::size_of::<resolution_mana::ResolutionPayment>()
                        })
                        + resolution.milled.len() * std::mem::size_of::<ObjectRef>()
                        + resolution
                            .choice
                            .as_ref()
                            .map(|choice| choice.bytes())
                            .unwrap_or(0)
                        + resolution
                            .search
                            .as_ref()
                            .map(|choice| choice.selected.len() * std::mem::size_of::<CardId>())
                            .unwrap_or(0)
                        + resolution
                            .library
                            .as_ref()
                            .map(|choice| {
                                (choice.remaining.len()
                                    + choice.top.len()
                                    + choice.bottom.len()
                                    + choice.graveyard.len())
                                    * std::mem::size_of::<CardId>()
                            })
                            .unwrap_or(0)
                        + resolution
                            .selected_discards
                            .iter()
                            .map(Vec::len)
                            .sum::<usize>()
                            * std::mem::size_of::<CardId>()
                })
                .unwrap_or(0)
            + s.stack
                .iter()
                .chain(&s.pending_triggers)
                .chain(&s.ordering_triggers)
                .chain(s.resolving.iter().map(|resolution| &resolution.item))
                .chain(s.announcing.iter().map(|announcement| &announcement.item))
                .map(|item| item.targets.len() * std::mem::size_of::<BoundTarget>())
                .sum::<usize>()
            + s.attackers.len() * std::mem::size_of::<Attacker>()
            + s.blocking.len() * std::mem::size_of::<CardId>()
            + s.blocks.len() * std::mem::size_of::<(CardId, CardId)>()
            + s.selected_attackers.len() * std::mem::size_of::<CardId>()
            + s.selected_blocks.len() * std::mem::size_of::<(CardId, CardId)>()
            + s.hits.len() * std::mem::size_of::<DamageEvent>()
            + s.first_strikers.len() * std::mem::size_of::<CardId>()
            + s.damage_choices.len() * std::mem::size_of::<DamageChoice>()
            + s.damage_choices
                .iter()
                .map(|d| d.blockers.len() * std::mem::size_of::<CardId>())
                .sum::<usize>()
    }
}

#[cfg(test)]
#[path = "engine_library_tests.rs"]
mod library_tests;

#[cfg(test)]
#[path = "engine_stats_tests.rs"]
mod stats_tests;

#[cfg(test)]
#[path = "engine_keyword_tests.rs"]
mod keyword_tests;

#[cfg(test)]
#[path = "engine_blocking_tests.rs"]
mod blocking_tests;

#[cfg(test)]
#[path = "engine_exile_tests.rs"]
mod exile_tests;

#[cfg(test)]
#[path = "engine_land_tests.rs"]
mod land_tests;

#[cfg(test)]
#[path = "engine_search_tests.rs"]
mod search_tests;

#[cfg(test)]
#[path = "engine_mode_tests.rs"]
mod mode_tests;

#[cfg(test)]
#[path = "engine_group_tests.rs"]
mod group_tests;

#[cfg(test)]
#[path = "engine_sba_tests.rs"]
mod sba_tests;

#[cfg(test)]
#[path = "engine_cast_tests.rs"]
mod cast_tests;

#[cfg(test)]
#[path = "engine_step_tests.rs"]
mod step_tests;

#[cfg(test)]
#[path = "engine_effect_choice_tests.rs"]
mod effect_choice_tests;

#[cfg(test)]
#[path = "engine_vital_tests.rs"]
mod vital_tests;

#[path = "engine_equipment.rs"]
mod equipment;

#[cfg(test)]
#[path = "engine_equipment_tests.rs"]
mod equipment_tests;

#[path = "engine_entries.rs"]
mod entries;
pub use entries::{AuraAttachmentView, AuraEntryView, EntryLifeSelection, EntryLifeView};

#[cfg(test)]
#[path = "engine_aura_tests.rs"]
mod aura_tests;

#[cfg(test)]
#[path = "engine_milling_tests.rs"]
mod milling_tests;

#[cfg(test)]
#[path = "engine_damage_trigger_tests.rs"]
mod damage_trigger_tests;

#[cfg(test)]
#[path = "engine_ward_tests.rs"]
mod ward_tests;
pub use resolution_mana::ResolutionPaymentView;

#[cfg(test)]
#[path = "engine_cycling_tests.rs"]
mod cycling_tests;

pub use discard_events::DiscardTriggerView;

#[cfg(test)]
#[path = "engine_discard_tests.rs"]
mod discard_tests;

pub use draw_events::DrawTriggerView;

#[cfg(test)]
#[path = "engine_draw_tests.rs"]
mod draw_tests;

pub use connive::{ConniveChoiceView, ConniverView};

#[cfg(test)]
#[path = "engine_connive_tests.rs"]
mod connive_tests;

#[cfg(test)]
#[path = "engine_limited_trigger_tests.rs"]
mod limited_trigger_tests;

#[cfg(test)]
#[path = "engine_hybrid_tests.rs"]
mod hybrid_tests;

#[cfg(test)]
#[path = "engine_changeling_tests.rs"]
mod changeling_tests;
