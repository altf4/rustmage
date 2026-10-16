use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Compact catalogue identity. The immutable definitions are shared across all
/// games; no card strings, JSON, or rule objects are cloned into game states.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Card(u16);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Effect {
    Creature(i32, i32),
    Spell(TargetPlan, &'static [Instruction]),
    Modal(&'static [SpellMode]),
    Land,
    Permanent,
    Aura(TargetPlan),
    Unsupported,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SpellMode {
    pub text: &'static str,
    pub targets: TargetPlan,
    pub program: &'static [Instruction],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TargetRule {
    None,
    Creature,
    CreatureOrVehicle {
        relation: i8,
    },
    Any,
    Spell,
    SpellKinds {
        kinds: u8,
        excluded: u8,
    },
    Player {
        opponent_only: bool,
    },
    Object {
        zone: TargetZone,
        kinds: u8,
        relation: i8,
        other: bool,
        nonland: bool,
        tapped: bool,
        min_mana: u32,
        max_mana: u32,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TargetZone {
    Battlefield,
    Graveyard,
}

/// One homogeneous occurrence of "target"; distinct choices, incremental bounds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TargetSpec {
    pub rule: TargetRule,
    pub min: u16,
    pub max: u16,
}
impl TargetSpec {
    pub const fn one(rule: TargetRule) -> Self {
        Self {
            rule,
            min: 1,
            max: 1,
        }
    }
    pub const NONE: Self = Self {
        rule: TargetRule::None,
        min: 0,
        max: 0,
    };
}

/// Independent occurrences of target. Cross-occurrence relationships are gated.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TargetPlan {
    Single(TargetSpec),
    Groups(&'static [TargetSpec]),
}
impl TargetPlan {
    pub const NONE: Self = Self::Single(TargetSpec::NONE);
    pub fn group(self, index: u16) -> Option<TargetSpec> {
        match self {
            Self::Single(spec) => (index == 0).then_some(spec),
            Self::Groups(groups) => groups.get(usize::from(index)).copied(),
        }
    }
    pub fn single(self) -> Option<TargetSpec> {
        match self {
            Self::Single(spec) => Some(spec),
            _ => None,
        }
    }
    pub fn has_targets(self) -> bool {
        match self {
            Self::Single(spec) => spec.max > 0,
            Self::Groups(groups) => groups.iter().any(|spec| spec.max > 0),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PlayerSet {
    DrawingPlayer,
    Controller,
    Opponent,
    Each,
    Targeted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ManaProduction {
    Fixed(Mana),
    Choice { mask: u8, amount: u32 },
}
impl ManaProduction {
    pub(crate) fn visit(self, mut emit: impl FnMut(Mana) -> bool) -> bool {
        match self {
            Self::Fixed(mana) => emit(mana),
            Self::Choice { mask, amount } => {
                for index in 0..6 {
                    if mask & (1 << index) != 0 {
                        let mut mana = [0; 6];
                        mana[index] = amount;
                        if emit(mana) {
                            return true;
                        }
                    }
                }
                false
            }
        }
    }
}

/// Fixed qualities admitted by library-search instructions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum SearchFilter {
    BasicLand,
    Land,
    Creature,
    Artifact,
    Enchantment,
    Subtype(&'static str),
}
impl SearchFilter {
    pub(crate) fn matches(self, card: Card) -> bool {
        match self {
            Self::BasicLand => card.is_land() && card.is_basic_land(),
            Self::Land => card.is_land(),
            Self::Creature => card.is_creature(),
            Self::Artifact => card.is_artifact(),
            Self::Enchantment => card.kind_mask() & 32 != 0,
            Self::Subtype(subtype) => card.has_subtype(subtype),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum SearchDestination {
    Hand,
    Battlefield,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct SearchSpec {
    pub filter: SearchFilter,
    pub maximum: u16,
    pub destination: SearchDestination,
    pub tapped: bool,
    pub reveal: bool,
}

/// A resolution choice; discard here is a cost, not a forced discard effect.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum EffectChoiceKind {
    Sacrifice,
    Discard,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SacrificeFilter {
    pub keywords: Keywords,
    pub source_only: bool,
    pub kinds: u8,
    pub nonland: bool,
    pub other: bool,
    pub tokens: Option<bool>,
    pub subtype: Option<&'static str>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct AdditionalCost {
    pub kind: EffectChoiceKind,
    pub filter: SacrificeFilter,
}

/// Card qualities for a non-targeted choice from an exact milled batch.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum RecoveryFilter {
    Permanent,
    Land,
}
impl RecoveryFilter {
    pub(crate) fn matches(self, card: Card) -> bool {
        match self {
            Self::Permanent => {
                card.is_land() || card.is_creature() || card.kind_mask() & (16 | 32 | 64) != 0
            }
            Self::Land => card.is_land(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MillAmount {
    Fixed(u16),
    ControllerLife,
}

/// A fixed mana payment, independent of casting or activation costs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct ManaCost {
    pub colored: Mana,
    pub generic: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CounterSubject {
    Targets,
    Related,
}

/// Immutable rule bytecode. Only the program counter and choices live in state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Instruction {
    Connive {
        subject: Subject,
        amount: u16,
    },
    CounterUnlessPay {
        cost: ManaCost,
        subject: CounterSubject,
        exile: bool,
        unpaid_skip: usize,
    },
    DrawDamageAmount,
    PutDiscardCounters(CounterKind),
    DamageDiscardAmount(PlayerSet),
    DiscardHand(PlayerSet),
    TapCreatures(CreatureFilter),
    Mill(PlayerSet, MillAmount),
    MayMill(u16),
    RecoverMilled(RecoveryFilter),
    AttachSource,
    TapAttached,
    ChooseObjects {
        kind: EffectChoiceKind,
        players: PlayerSet,
        filter: SacrificeFilter,
        amount: u16,
        optional_skip: Option<usize>,
    },
    SelectGroup(u16),
    Fight {
        source: u16,
        recipient: u16,
    },
    PowerDamage {
        source: u16,
        recipient: u16,
    },
    AddMana(ManaProduction),
    Scry(u16),
    Surveil(u16),
    Search(SearchSpec),
    Damage(i32),
    DamagePlayers(PlayerSet, i32),
    UntapSource,
    TapSource,
    UntapCreatures(CreatureFilter),
    Grow {
        subject: Subject,
        power: i32,
        toughness: i32,
    },
    GrantKeywords {
        subject: Subject,
        keywords: Keywords,
    },
    GrantCreatures {
        filter: CreatureFilter,
        keywords: Keywords,
    },
    GrowCreatures {
        filter: CreatureFilter,
        power: i32,
        toughness: i32,
    },
    Counter,
    Exile,
    ExileUntilSourceLeaves,
    ExileGraveyard(PlayerSet),
    ReturnToHand,
    ReturnToBattlefield,
    Destroy,
    Tap,
    Untap,
    PutCreatureCounters {
        filter: CreatureFilter,
        kind: CounterKind,
        amount: u32,
    },
    DoubleCounters,
    PutCounters {
        subject: Subject,
        kind: CounterKind,
        amount: u32,
    },
    CreateTokens {
        token: Card,
        amount: u16,
        tapped: bool,
    },
    Draw(PlayerSet, u16),
    Discard(PlayerSet, u16),
    GainLife(PlayerSet, u16),
    LoseLife(PlayerSet, u16),
    /// Optional block containing the following `skip` instructions.
    May {
        skip: usize,
    },
}

/// Fixed additive layer-7c values, including asymmetric temporary modifiers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Stats {
    pub power: i128,
    pub toughness: i128,
}
impl From<i128> for Stats {
    fn from(value: i128) -> Self {
        Self {
            power: value,
            toughness: value,
        }
    }
}
impl std::ops::AddAssign for Stats {
    fn add_assign(&mut self, other: Self) {
        self.power = self
            .power
            .checked_add(other.power)
            .expect("power representation exhausted");
        self.toughness = self
            .toughness
            .checked_add(other.toughness)
            .expect("toughness representation exhausted");
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CountZone {
    Battlefield,
    Graveyard,
    Exile,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CountFilter {
    pub zone: CountZone,
    pub relation: i8,
    pub kind: u16,
    pub exclude: u16,
    pub subtype: Option<&'static str>,
    pub basic: bool,
    pub other: bool,
    pub cards_only: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum StatValue {
    Count(CountFilter),
    GreatestManaValue(CountFilter),
    GraveyardCardTypes,
    Sum(&'static [StatValue]),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct StatExpr {
    pub value: StatValue,
    pub multiplier: i128,
    pub offset: i128,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct DynamicStats {
    pub power: Option<StatExpr>,
    pub toughness: Option<StatExpr>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ScalingModifier {
    pub subject: Subject,
    pub value: StatValue,
    pub stats: Stats,
}

/// Predicates depend only on earlier-layer characteristics and combat membership.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CreatureFilter {
    pub relation: i8,
    pub other: bool,
    pub subtype: Option<&'static str>,
    pub keyword: Keywords,
    pub attacking: bool,
    pub tokens: Option<bool>,
    pub artifact: bool,
}

impl CreatureFilter {
    /// Earlier-layer facts only. Effective keywords are checked separately.
    pub(crate) fn matches_intrinsic(
        self,
        card: Card,
        yours: bool,
        same: bool,
        attacking: bool,
    ) -> bool {
        (match self.relation {
            0 => yours,
            1 => !yours,
            _ => true,
        }) && (!self.other || !same)
            && self.subtype.is_none_or(|subtype| card.has_subtype(subtype))
            && (!self.attacking || attacking)
            && self.tokens.is_none_or(|tokens| card.is_token() == tokens)
            && (!self.artifact || card.is_artifact())
    }
}

/// Commuting positive layer-6 grants. Keyword-qualified recipients and ability
/// removal/prohibition remain unsupported; no recursive dependency solver is implied.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct KeywordGrant {
    pub filter: CreatureFilter,
    pub source_only: bool,
    pub own_turn: bool,
    pub keywords: Keywords,
}
impl KeywordGrant {
    pub(crate) fn matches(
        self,
        card: Card,
        yours: bool,
        same: bool,
        attacking: bool,
        own_turn: bool,
    ) -> bool {
        assert_eq!(
            self.filter.keyword,
            Keywords::NONE,
            "keyword-dependent layer-6 filter"
        );
        (!self.own_turn || own_turn)
            && (!self.source_only || same)
            // A tribal noun may also describe a noncreature kindred permanent.
            && (self.filter.subtype.is_some() || card.is_creature())
            && self.filter.matches_intrinsic(card, yours, same, attacking)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct StaticModifier {
    pub filter: CreatureFilter,
    pub stats: Stats,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct EquipmentSpec {
    pub stats: Stats,
    pub keywords: Keywords,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct AuraSpec {
    pub relation: i8,
    pub bonus: EquipmentSpec,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Subject {
    Source,
    Targets,
    Attached,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum CounterKind {
    PlusOne,
    MinusOne,
    Stun,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Counters {
    pub plus_one: u64,
    pub minus_one: u64,
    pub stun: u64,
}
impl Counters {
    pub(crate) fn add(&mut self, kind: CounterKind, amount: u64) {
        let counter = match kind {
            CounterKind::PlusOne => &mut self.plus_one,
            CounterKind::MinusOne => &mut self.minus_one,
            CounterKind::Stun => &mut self.stun,
        };
        *counter = counter
            .checked_add(amount)
            .expect("counter representation exhausted");
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CastFilter {
    Any,
    Creature,
    Noncreature,
    InstantSorcery,
    Artifact,
    // Compiler vocabulary; no complete admitted card currently uses this filter.
    #[allow(dead_code)]
    Enchantment,
}
impl CastFilter {
    pub fn matches(self, card: Card) -> bool {
        match self {
            Self::Any => true,
            Self::Creature => card.is_creature(),
            Self::Noncreature => !card.is_creature(),
            Self::InstantSorcery => card.kind_mask() & (4 | 8) != 0,
            Self::Artifact => card.is_artifact(),
            Self::Enchantment => card.kind_mask() & 32 != 0,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CastRelation {
    You,
    // Complete-text gating currently excludes the corresponding printed cards.
    #[allow(dead_code)]
    Opponent,
    #[allow(dead_code)]
    Any,
}

// The compiler has exact spellings for this condition vocabulary. Some values
// remain unused until the rest of the corresponding card text is implemented.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum StepValue {
    Life,
    Hand,
    Creatures,
    TappedCreatures,
    Lands,
    GreatestCreaturePower,
    TotalCreaturePower,
    TotalCreatureToughness,
    SpellsCast,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum StepGuard {
    Always,
    Attacked,
    SourceTapped,
    Threshold {
        value: StepValue,
        minimum: Option<i128>,
        maximum: Option<i128>,
    },
}
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TriggerStep {
    Upkeep,
    Draw,
    Main1,
    Main2,
    MainPhases,
    BeginCombat,
    EndCombat,
    End,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DamageQualification {
    Any,
    Modified,
    Counter,
    PlusOne,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DamageSubject {
    Source,
    Attached,
    Creatures {
        filter: CreatureFilter,
        qualification: DamageQualification,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TriggerCondition {
    OncePerTurn(&'static TriggerCondition),
    CardDrawn {
        relation: CastRelation,
        ordinal: Option<u64>,
    },
    SelfDiscarded,
    SelfCycled,
    Discarded {
        relation: CastRelation,
        one_or_more: bool,
    },
    Ward,
    PlayerDamaged {
        subject: DamageSubject,
        combat_only: bool,
        opponent_only: bool,
        one_or_more: bool,
    },
    LifeGained {
        first_only: bool,
        own_turn: bool,
    },
    CreatureDies {
        relation: CastRelation,
        other: bool,
        nontoken: bool,
        subtype: Option<&'static str>,
    },
    AtStep {
        step: TriggerStep,
        relation: CastRelation,
        guard: StepGuard,
    },
    SpellCast {
        relation: CastRelation,
        filter: CastFilter,
        ordinal: Option<u64>,
    },
    SelfEnters,
    SelfDies,
    SelfEntersOrDies,
    SelfAttacks,
    SelfEntersOrAttacks,
    OtherCreatureEntersYou,
    OtherSubtypeEntersYou(&'static str),
    SubtypeEntersYou(&'static str),
    ArtifactEntersYou,
    EnchantmentEntersYou,
    CreatureEntersYou,
    OtherArtifactEntersYou,
    LandEntersYou,
}

impl TriggerCondition {
    pub(crate) fn event(self) -> Self {
        match self {
            Self::OncePerTurn(event) => event.event(),
            event => event,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TriggeredAbility {
    pub condition: TriggerCondition,
    pub targets: TargetPlan,
    pub program: &'static [Instruction],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ActivationCost {
    pub colored: Mana,
    pub generic: u32,
    pub mana_component: bool,
    pub tap: bool,
    pub sacrifice: bool,
    pub discard: bool,
    pub life: u16,
    pub remove: Option<(Option<CounterKind>, u32)>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ActivationOrigin {
    Battlefield,
    Hand,
}
/// Starting restrictions, not costs that must still hold during payment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ActivationGuard {
    Always,
    ControlSubtypeAny(&'static [&'static str]),
}
impl ActivationGuard {
    pub(crate) fn subtypes(self) -> &'static [&'static str] {
        match self {
            Self::Always => &[],
            Self::ControlSubtypeAny(types) => types,
        }
    }
    pub(crate) fn matches_card(self, card: Card) -> bool {
        self.subtypes()
            .iter()
            .any(|subtype| card.has_subtype(subtype))
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ActivatedAbility {
    pub guard: ActivationGuard,
    pub origin: ActivationOrigin,
    pub cycling: bool,
    pub cost: ActivationCost,
    pub targets: TargetPlan,
    pub program: &'static [Instruction],
    pub sorcery: bool,
    pub once_per_turn: bool,
}

impl ActivatedAbility {
    pub(crate) fn mana_production(&self) -> Option<ManaProduction> {
        match self.program {
            [Instruction::AddMana(production)] if !self.targets.has_targets() => Some(*production),
            _ => None,
        }
    }
}

/// Implemented static keyword flags. Other keyword cards remain unsupported.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct Keywords(u32);

impl Keywords {
    pub const FLYING: Self = Self(1);
    pub const REACH: Self = Self(2);
    pub const HASTE: Self = Self(4);
    pub const VIGILANCE: Self = Self(8);
    pub const DEFENDER: Self = Self(16);
    pub const FLASH: Self = Self(32);
    pub const LIFELINK: Self = Self(64);
    pub const DEATHTOUCH: Self = Self(128);
    pub const TRAMPLE: Self = Self(256);
    pub const FIRST_STRIKE: Self = Self(512);
    pub const DOUBLE_STRIKE: Self = Self(1024);
    pub const CANT_BLOCK: Self = Self(2048);
    pub const NONE: Self = Self(0);
    pub const HEXPROOF: Self = Self(4096);
    pub const INDESTRUCTIBLE: Self = Self(8192);
    pub const CANT_BE_BLOCKED: Self = Self(16384);
    pub const MENACE: Self = Self(32768);
    pub const MAX_ONE_BLOCKER: Self = Self(65536);
    pub const MIN_THREE_BLOCKERS: Self = Self(131072);
    pub const NO_TOKEN_BLOCKERS: Self = Self(262144);
    pub const CHANGELING: Self = Self(524288);
    pub fn contains(self, keyword: Self) -> bool {
        self.0 & keyword.0 == keyword.0
    }
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct CardInfo {
    pub name: &'static str,
    pub oracle_id: Option<&'static str>,
    pub mana_cost: &'static str,
    pub type_line: &'static str,
    pub oracle_keywords: &'static [&'static str],
    pub standard_legal: bool,
    pub legendary: bool,
    pub implemented: bool,
    pub keywords: Keywords,
    pub deck_limit: Option<u8>,
}

/// Immutable characteristics of a fixed creature token. Colors use W/U/B/R/G bits.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct TokenInfo {
    pub name: &'static str,
    pub subtypes: &'static str,
    pub colors: u8,
}

/// Self replacement evaluated against the battlefield immediately before entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EntryTapped {
    Never,
    Always,
    UnlessBasic,
    UnlessAtMostTwo,
    UnlessAtLeastTwo,
    UnlessLowLife,
    UnlessPlaneswalker,
    PayLife(u32),
}

struct Definition {
    info: CardInfo,
    effect: Effect,
    triggers: &'static [TriggeredAbility],
    activated: &'static [ActivatedAbility],
    enters: Counters,
    entry: EntryTapped,
    kind: u16,
    basic: bool,
    mana_color: Option<usize>,
    colored: Mana,
    generic: u32,
}

mod generated {
    use super::*;
    include!("generated/catalogue.rs");
}
pub use generated::{CATALOGUE_VERSION, CREATURE_TYPES};

/// Creature types from the pinned Comprehensive Rules, including Time Lord.
pub fn is_creature_type(subtype: &str) -> bool {
    CREATURE_TYPES.binary_search(&subtype).is_ok()
}

fn printed_subtype(type_line: &str, subtype: &str) -> bool {
    let Some((_, types)) = type_line.split_once(" — ") else {
        return false;
    };
    if subtype == "Time Lord" {
        let mut words = types.split_whitespace();
        while let Some(word) = words.next() {
            if word == "Time" && words.next() == Some("Lord") {
                return true;
            }
        }
        false
    } else {
        // CR205.3m defines one two-word creature type, not separate Time/Lord types.
        subtype != "Time" && subtype != "Lord" && types.split_whitespace().any(|ty| ty == subtype)
    }
}

/// Mana is W U B R G C; C is colorless. Generic costs deliberately have no colour slot.
pub type Mana = [u32; 6];

impl Card {
    /// Intrinsic subtype membership in every zone. Dynamic type/ability-changing
    /// effects remain gated; changeling adds creature types, never other subtypes.
    pub fn has_subtype(self, subtype: &str) -> bool {
        self.keywords().contains(Keywords::CHANGELING) && is_creature_type(subtype)
            || printed_subtype(self.info().type_line, subtype)
    }
    pub fn from_name(name: &str) -> Option<Self> {
        // Accept the foundation's JSON tokens as aliases for canonical names.
        let canonical = match name {
            "GrizzlyBears" => "Grizzly Bears",
            "GoblinPiker" => "Goblin Piker",
            "HillGiant" => "Hill Giant",
            "LlanowarElves" => "Llanowar Elves",
            "LightningBolt" => "Lightning Bolt",
            "GiantGrowth" => "Giant Growth",
            _ => name,
        };
        generated::NAME_INDEX
            .binary_search_by_key(&canonical, |&(name, _)| name)
            .ok()
            .map(|index| Self(generated::NAME_INDEX[index].1))
    }
    pub fn catalogue() -> impl ExactSizeIterator<Item = Card> {
        (0..generated::DEFINITIONS.len()).map(|index| Card(index as u16))
    }
    pub fn catalogue_len() -> usize {
        generated::DEFINITIONS.len()
    }
    pub fn info(self) -> &'static CardInfo {
        &generated::DEFINITIONS[self.0 as usize].info
    }
    pub fn name(self) -> &'static str {
        self.info().name
    }
    pub fn is_legendary(self) -> bool {
        self.info().legendary
    }
    pub fn is_supported(self) -> bool {
        self.info().implemented
    }
    pub fn is_standard_legal(self) -> bool {
        self.info().standard_legal
    }
    pub fn token_info(self) -> Option<&'static TokenInfo> {
        self.index()
            .checked_sub(generated::TOKEN_BASE)
            .and_then(|offset| generated::TOKENS.get(offset))
    }
    pub fn is_token(self) -> bool {
        self.token_info().is_some()
    }
    pub(crate) fn additional_cost(self) -> Option<AdditionalCost> {
        generated::additional_cost(self)
    }
    pub(crate) fn dynamic_stats(self) -> Option<DynamicStats> {
        generated::dynamic_stats(self)
    }
    pub(crate) fn scaling_modifiers(self) -> &'static [ScalingModifier] {
        generated::scaling_modifiers(self)
    }
    pub(crate) fn static_modifiers(self) -> &'static [StaticModifier] {
        generated::static_modifiers(self)
    }
    pub(crate) fn equipment(self) -> Option<EquipmentSpec> {
        generated::equipment(self)
    }
    pub(crate) fn aura(self) -> Option<AuraSpec> {
        generated::aura(self)
    }
    pub(crate) fn attachment_bonus(self) -> Option<EquipmentSpec> {
        self.equipment().or_else(|| self.aura().map(|a| a.bonus))
    }
    pub fn keywords(self) -> Keywords {
        self.info().keywords
    }
    pub fn is_creature(self) -> bool {
        generated::DEFINITIONS[self.0 as usize].kind & 1 != 0
    }
    pub fn is_land(self) -> bool {
        generated::DEFINITIONS[self.0 as usize].kind & 2 != 0
    }
    pub fn is_artifact(self) -> bool {
        generated::DEFINITIONS[self.0 as usize].kind & 16 != 0
    }
    pub(crate) fn kind_mask(self) -> u16 {
        generated::DEFINITIONS[self.0 as usize].kind
    }
    pub fn is_basic_land(self) -> bool {
        generated::DEFINITIONS[self.0 as usize].basic
    }
    pub fn is_instant(self) -> bool {
        generated::DEFINITIONS[self.0 as usize].kind & 8 != 0
    }
    pub(crate) fn effect(self) -> Effect {
        generated::DEFINITIONS[self.0 as usize].effect
    }
    pub(crate) fn spell_program(
        self,
        mode: Option<u16>,
    ) -> Option<(TargetPlan, &'static [Instruction])> {
        match self.effect() {
            Effect::Spell(spec, program) => Some((spec, program)),
            Effect::Aura(spec) => Some((spec, &[])),
            Effect::Modal(modes) => mode
                .and_then(|id| modes.get(usize::from(id)))
                .map(|mode| (mode.targets, mode.program)),
            _ => None,
        }
    }
    pub(crate) fn triggers(self) -> &'static [TriggeredAbility] {
        generated::DEFINITIONS[self.0 as usize].triggers
    }
    pub(crate) fn keyword_grants(self) -> &'static [KeywordGrant] {
        generated::keyword_grants(self)
    }

    pub(crate) fn activations(self) -> &'static [ActivatedAbility] {
        generated::DEFINITIONS[self.0 as usize].activated
    }
    pub(crate) fn entry_tapped(self) -> EntryTapped {
        generated::DEFINITIONS[self.0 as usize].entry
    }
    pub(crate) fn enters_counters(self) -> Counters {
        generated::DEFINITIONS[self.0 as usize].enters
    }
    pub(crate) fn mana_color(self) -> Option<usize> {
        generated::DEFINITIONS[self.0 as usize].mana_color
    }
    pub(crate) fn fixed_mana_value(self) -> u64 {
        let (colored, generic) = self.cost();
        u64::from(generic) + colored.into_iter().map(u64::from).sum::<u64>()
    }
    pub(crate) fn cost(self) -> (Mana, u32) {
        let definition = &generated::DEFINITIONS[self.0 as usize];
        (definition.colored, definition.generic)
    }
    pub(crate) fn has_hybrid_cost(self) -> bool {
        !generated::cost_alternatives(self).is_empty()
    }
    pub(crate) fn cost_options(self) -> impl Iterator<Item = ManaCost> {
        let definition = &generated::DEFINITIONS[self.0 as usize];
        std::iter::once(definition.colored)
            .chain(generated::cost_alternatives(self).iter().copied())
            .map(|colored| ManaCost {
                colored,
                generic: definition.generic,
            })
    }
    pub(crate) fn spell_payments(self, pool: Mana, mut emit: impl FnMut(Mana)) {
        if !self.has_hybrid_cost() {
            let (colored, generic) = self.cost();
            payments(pool, colored, generic, emit);
            return;
        }
        // Different hybrid assignments can consume the same pool allocation.
        // Emit each action once, without charging fixed-cost casts an allocation.
        let mut seen = Vec::new();
        for cost in self.cost_options() {
            payments(pool, cost.colored, cost.generic, |payment| {
                if !seen.contains(&payment) {
                    seen.push(payment);
                    emit(payment);
                }
            });
        }
    }
    pub(crate) fn index(self) -> usize {
        self.0 as usize
    }
}

impl Serialize for Card {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.name())
    }
}
impl<'de> Deserialize<'de> for Card {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let name = String::deserialize(deserializer)?;
        Self::from_name(&name)
            .ok_or_else(|| serde::de::Error::custom(format!("unknown card: {name}")))
    }
}

pub(crate) fn payments(pool: Mana, colored: Mana, generic: u32, mut emit: impl FnMut(Mana)) {
    if (0..6).any(|i| pool[i] < colored[i]) {
        return;
    }
    fn visit(i: usize, left: u32, pool: Mana, mut paid: Mana, emit: &mut impl FnMut(Mana)) {
        if i == 5 {
            if pool[i] >= left {
                paid[i] += left;
                emit(paid);
            }
            return;
        }
        for n in 0..=pool[i].min(left) {
            let old = paid[i];
            paid[i] = old + n;
            visit(i + 1, left - n, pool, paid, emit);
            paid[i] = old;
        }
    }
    let remaining = std::array::from_fn(|i| pool[i] - colored[i]);
    visit(0, generic, remaining, colored, &mut emit);
}

#[cfg(test)]
mod subtype_tests {
    use super::*;
    #[test]
    fn two_word_creature_type_is_one_subtype_and_other_subtypes_stay_separate() {
        let line = "Legendary Creature — Time Lord Doctor";
        assert!(printed_subtype(line, "Time Lord"));
        assert!(printed_subtype(line, "Doctor"));
        for typ in [
            "Time",
            "Lord",
            "Time Lord Doctor",
            "Lord Doctor",
            "Human",
            "",
        ] {
            assert!(!printed_subtype(line, typ), "{typ}");
        }
        assert!(printed_subtype("Artifact Creature — Food Golem", "Food"));
        assert!(printed_subtype("Artifact Creature — Food Golem", "Golem"));
        assert!(!printed_subtype(
            "Creature — Human Soldier",
            "Human Soldier"
        ));
    }
    #[test]
    fn compiled_creature_types_match_pinned_taxonomy_exactly() {
        let doc: serde_json::Value =
            serde_json::from_str(include_str!("../data/creature-types.json")).unwrap();
        let expected = doc["types"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| s.as_str().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(CREATURE_TYPES, expected);
        assert!(is_creature_type("C'tan"));
        assert!(is_creature_type("Shi'ar"));
        assert!(is_creature_type("Time Lord"));
        for typ in [
            "Hunter", "Klingon", "Xindi", "Forest", "Aura", "Time", "Lord", "elf",
        ] {
            assert!(!is_creature_type(typ));
        }
    }
}
