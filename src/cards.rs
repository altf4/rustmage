use crate::capacity::CapacityExt;
use crate::engine::GrantedSubtypes;
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
    Emblem,
    Aura(TargetPlan),
    Unsupported,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SpellMode {
    pub text: &'static str,
    pub targets: TargetPlan,
    pub program: &'static [Instruction],
}

/// The four complete Gift payloads present in the pinned Standard catalogue.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum GiftKind {
    Card,
    Food,
    Treasure,
    TappedFish,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GiftSpec {
    pub kind: GiftKind,
    pub token: Option<Card>,
    pub promised_targets: TargetPlan,
}

/// A relationship between distinct bindings within one occurrence of target.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TargetGroupConstraint {
    SameGraveyard,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TargetRule {
    Grouped {
        base: &'static TargetRule,
        constraint: TargetGroupConstraint,
    },
    None,
    Union(&'static [TargetRule]),
    Filtered {
        base: &'static TargetRule,
        predicate: &'static TargetPredicate,
    },
    Creature,
    /// Bound to the captured attachment attack defender before target selection.
    DefendingCreature,
    CreatureOrVehicle {
        relation: i8,
    },
    Any,
    Spell,
    SpellKinds {
        kinds: u8,
        excluded: u8,
    },
    SpellCharacteristics {
        colors: u8,
        min_mana: u64,
        max_mana: u64,
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

/// Immutable live constraints; indirection keeps target plans compact in checkpoints.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TargetPredicate {
    Blocking,
    Nontoken,
    Permanent,
    Attacking,
    AttackingOrBlocking,
    Damaged,
    DealtDamage,
    PowerAtLeast(i32),
    PowerAtMost(i32),
    ToughnessAtLeast(i32),
    PowerOrToughnessAtLeast(i32),
    Colors(u8),
    Keyword(Keywords),
    WithoutKeyword(Keywords),
    WithoutKinds(u16),
    WithoutSubtype(&'static str),
    Token,
    Subtype(&'static str),
    Nonbasic,
    Legendary,
    NonOutlaw,
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
    /// usize::MAX denotes any number; only arena/address-space limits apply.
    pub max: usize,
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
    CastingPlayer,
    DrawingPlayer,
    DamagedPlayer,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LibraryRemainder {
    Graveyard,
    BottomAny,
    BottomRandom,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LibrarySelectionFilter {
    Any,
    Kinds(u16),
    ExcludingKinds(u16),
    Subtype(&'static str),
    AnyOf(&'static [LibrarySelectionFilter]),
    AllOf(&'static [LibrarySelectionFilter]),
    CreaturePowerAtMost(i32),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LibrarySelectionCount {
    Fixed(u16),
    Lessons,
    InstantAndSorcery,
    GainedLife,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct LibrarySelectionSpec {
    pub look: u16,
    pub count: LibrarySelectionCount,
    pub filter: LibrarySelectionFilter,
    pub optional: bool,
    pub reveal: bool,
    pub remainder: LibraryRemainder,
}

/// Fixed qualities admitted by library-search instructions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum SearchFilter {
    AnyOf(&'static [SearchFilter]),
    AllOf(&'static [SearchFilter]),
    Name(&'static str),
    Kinds(u16),
    Keyword(Keywords),
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
            Self::AnyOf(filters) => filters.iter().any(|filter| filter.matches(card)),
            Self::AllOf(filters) => filters.iter().all(|filter| filter.matches(card)),
            Self::Name(name) => card.token_info().map_or(card.name(), |info| info.name) == name,
            Self::Kinds(kinds) => card.kind_mask() & kinds != 0,
            // Card characteristics in the library; flashback is not flash.
            // Dynamic outside-battlefield ability grants remain gated.
            Self::Keyword(keyword) => card.keywords().contains(keyword),
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
    BattlefieldThenHand { battlefield: u16 },
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
    HandCreature,
    HandLand { tapped: bool },
    CollectEvidence(u16),
    Forage,
    ForageOrBlack,
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
    /// Bargain disjunction; unlike ordinary kind/token constraints these are ORed.
    pub artifact_enchantment_or_token: bool,
    pub subtype: Option<&'static str>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct AdditionalCost {
    pub kind: EffectChoiceKind,
    pub filter: SacrificeFilter,
}

/// Public qualities for a non-targeted choice from one exact milling event.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum MilledFilter {
    Kinds(u8),
    KindsOrSubtype { kinds: u8, subtype: &'static str },
    Adventure,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum MilledDestination {
    Hand,
    Battlefield,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MilledCount {
    Range { minimum: u16, maximum: u16 },
    All,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct MilledSelectionSpec {
    pub filter: MilledFilter,
    pub count: MilledCount,
    pub destination: MilledDestination,
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
pub(crate) struct TargetSpellReduction {
    pub rule: TargetRule,
    pub discount: ManaCost,
}

pub(crate) type ManaCostOptions = std::iter::Chain<
    std::iter::Once<ManaCost>,
    std::iter::Copied<std::slice::Iter<'static, ManaCost>>,
>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CounterSubject {
    Targets,
    Related,
}

/// Immutable rule bytecode. Only the program counter and choices live in state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct BattlefieldFilter {
    pub kinds: u16,
    pub relation: i8,
    pub nonland: bool,
    pub tapped: Option<bool>,
    pub without: Keywords,
    pub subtype: Option<&'static str>,
    pub max_mana: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Instruction {
    PutPowerExcessCounters {
        filter: CreatureFilter,
    },
    Suspect {
        subject: Subject,
    },
    UnsuspectCreatures {
        filter: CreatureFilter,
    },
    ReturnGrantedSourceWithSuspect {
        tapped: bool,
    },
    ReturnToBattlefieldSuspected,
    GiveGift,
    ReturnSpellToHand,
    ProhibitCastingThisTurn,
    PlayerHexproofThisTurn,
    GrantPermanentsThisTurn(Keywords),
    DamageWithController {
        damage: i32,
        controller_damage: i32,
    },
    IfGift {
        promised: bool,
        skip: usize,
    },
    /// Register an exact current target for a direct exile replacement until cleanup.
    ExileIfDiesThisTurn,
    /// Queue a reflexive ability only when this exact Role actually enters attached.
    CreateRoleReflexive {
        token: Card,
        ability: u16,
    },
    /// The first fighter is the captured non-targeted successful-action subject.
    ReflexiveFight,
    CreateRolesMatching {
        token: Card,
        filter: CreatureFilter,
    },
    ReturnToBattlefieldWithRole {
        token: Card,
    },
    CreateRole {
        token: Card,
        subject: Subject,
    },
    IfManaSpent {
        minimum: Option<u64>,
        maximum: Option<u64>,
        skip: usize,
    },
    CounterThenDrawIfDiscounted,
    DiscardScaled(PlayerSet, AmountValue),
    DiscardManaValueLifeLoss,
    PutHandCreature,
    PutHandLand {
        tapped: bool,
    },
    JobSelect {
        token: Card,
    },
    MayPayMana {
        cost: ManaCost,
        skip: usize,
    },
    Endure {
        amount: u16,
        token: Card,
    },

    Mobilize {
        token: Card,
        amount: AmountValue,
    },
    SacrificeDelayedTokens,
    ExileWarpedPermanent,
    Crew {
        minimum: u16,
    },
    BecomeSaddled {
        minimum: u16,
    },
    ExileTopPlay {
        amount: u16,
        next_turn: bool,
    },
    ExileTargetLibraryPlay {
        amount: u16,
    },
    Forage {
        skip: usize,
    },
    CollectEvidence {
        threshold: u16,
        skip: usize,
    },
    EvidenceWard(u16),
    IfEvidence {
        skip: usize,
    },
    SacrificeGreatestPower,
    Reflexive {
        ability: u16,
    },
    /// Definition-only modal trigger header; selected before targets, never executed.
    Modal(&'static [SpellMode]),
    /// Skip the following block when its live condition is false.
    If {
        guard: StepGuard,
        skip: usize,
    },
    /// Historical paid qualification of the exact cast.
    Explore(Subject),
    DestroyAndCreateForController {
        token: Card,
        amount: u16,
    },
    IfKicked {
        skip: usize,
    },
    IfBargained {
        skip: usize,
    },
    /// Live predicate on an exact resolving object, never on a reincarnation.
    IfSubject {
        subject: Subject,
        filter: CreatureFilter,
        skip: usize,
    },
    Skip(usize),
    MoveBattlefield {
        filters: &'static [BattlefieldFilter],
        exile: bool,
    },
    /// Freeze a non-targeted battlefield group, then return it simultaneously.
    ReturnBattlefield {
        filters: &'static [BattlefieldFilter],
        attacking: bool,
        players: Option<PlayerSet>,
    },
    DamageBattlefield {
        filters: &'static [BattlefieldFilter],
        players: Option<PlayerSet>,
        amount: i32,
    },
    DrawMoved,
    GainLifeMoved,
    CreateEmblem(Card),
    LoseGainedLife(PlayerSet),
    Connive {
        subject: Subject,
        amount: u16,
    },
    CounterUnlessPayScaled {
        amount: AmountValue,
        exile: bool,
    },
    PutCountersScaled {
        subject: Subject,
        kind: CounterKind,
        amount: AmountValue,
    },
    CounterUnlessPay {
        cost: ManaCost,
        subject: CounterSubject,
        exile: bool,
        unpaid_skip: usize,
    },
    ReturnSourceToHand(TargetZone),
    ReturnSourceToBattlefield {
        tapped: bool,
    },
    CounterRelated,
    AddPoison(PlayerSet, u16),
    MayOpponent {
        skip: usize,
    },
    MayPayOpponentLife {
        amount: u16,
        skip: usize,
    },
    DrawDamageAmount,
    PutDiscardCounters(CounterKind),
    DamageDiscardAmount(PlayerSet),
    DiscardHand(PlayerSet),
    TapCreatures(CreatureFilter),
    Mill(PlayerSet, MillAmount),
    MayMill(u16),
    RecoverMilled(RecoveryFilter),
    SelectMilled(MilledSelectionSpec),
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
    AddCombatRed(u16),
    AddCombatRedScaled(AmountValue),
    Scry(u16),
    Surveil(u16),
    Search(SearchSpec),
    SelectLibrary(LibrarySelectionSpec),
    Damage(i32),
    DamageScaled(AmountValue),
    DamagePlayers(PlayerSet, i32),
    UntapSource,
    TapSource,
    UntapCreatures(CreatureFilter),
    SetBaseStats {
        subject: Subject,
        power: Option<i128>,
        toughness: Option<i128>,
    },
    SetResolvedBaseStats {
        subject: Subject,
        power: Option<ResolvedStatValue>,
        toughness: Option<ResolvedStatValue>,
    },
    Grow {
        subject: Subject,
        power: i32,
        toughness: i32,
    },
    GrowScaled {
        subject: Subject,
        amount: AmountValue,
        power: i32,
        toughness: i32,
    },
    GrantUntilEndTurn {
        ability: u16,
    },
    ReturnGrantedSource {
        counter: Option<(CounterKind, u64)>,
        role: Option<Card>,
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
    GrowCreaturesScaled {
        filter: CreatureFilter,
        excluding_subtype: Option<&'static str>,
        amount: AmountValue,
        power: i32,
        toughness: i32,
    },
    Counter,
    Exile,
    ExileUntilSourceLeaves,
    ExileGraveyard(PlayerSet),
    ReturnToHand,
    BottomLibrary,
    TopLibrary,
    AdditionalLandPlays(u64),
    OwnerLibrary,
    ShuffleTargetIntoLibrary,
    MultiplyTargetStats {
        factor: u8,
    },
    ReturnToBattlefield,
    ReturnToBattlefieldTapped,
    Destroy,
    Tap,
    Untap,
    PutCreatureCounters {
        filter: CreatureFilter,
        kind: CounterKind,
        amount: u32,
    },
    DoubleCounters,
    RemoveAllCounters,
    SnapshotDoubleCreatures(CreatureFilter),
    PutCounters {
        subject: Subject,
        kind: CounterKind,
        amount: u32,
    },
    CreateTokensScaled {
        token: Card,
        amount: AmountValue,
        tapped: bool,
    },
    CreateTokens {
        token: Card,
        amount: u16,
        tapped: bool,
    },
    Draw(PlayerSet, u16),
    DrawScaled(PlayerSet, AmountValue),
    Discard(PlayerSet, u16),
    GainLife(PlayerSet, u16),
    GainLifeScaled(PlayerSet, AmountValue),
    LoseLife(PlayerSet, u16),
    /// Each opponent loses the frozen amount; the controller gains it once.
    DrainLifeScaled(AmountValue),
    /// Optional block containing the following `skip` instructions.
    May {
        skip: usize,
    },
}

/// Live static recipients use only earlier-layer facts; no derived-stat predicates.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum LiveStatRecipients {
    Creatures {
        relation: i8,
        other: bool,
        enchanted_by_controller: bool,
    },
    Attached,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum LiveStatValue {
    Fixed(i128),
    ControllerLife,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct LiveStatCondition {
    pub subtype: &'static str,
    pub negated: bool,
}
/// Immutable complete continuous-effect clause. Values and membership remain live.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct LiveStaticEffect {
    pub recipients: LiveStatRecipients,
    pub condition: Option<LiveStatCondition>,
    pub power: Option<LiveStatValue>,
    pub toughness: Option<LiveStatValue>,
    pub bonus_power: i128,
    pub bonus_toughness: i128,
    pub keywords: Keywords,
    pub cannot_block_subtype: Option<&'static str>,
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
            .capacity("power representation exhausted");
        self.toughness = self
            .toughness
            .checked_add(other.toughness)
            .capacity("toughness representation exhausted");
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
pub(crate) enum AmountValue {
    ManaSpent,
    ColorsSpent,
    SourcePower,
    Fixed(u16),
    /// Explicit immutable parameter of the resolving spell or its exact ETB packet.
    ChosenX,
    Stat(StatValue),
    LifeGainedThisTurn,
    CardsDiscardedThisTurn,
    Multiple {
        value: &'static AmountValue,
        factor: u16,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ResolvedStatValue {
    /// Capture a signed scalar once, when the instruction resolves.
    Stat(StatValue),
    /// Evaluate before creating the fixed, until-cleanup layer-7b effect.
    Threshold {
        value: StatValue,
        minimum: i128,
        yes: i128,
        no: i128,
    },
    /// Current exact event object, or its signed departure LKI.
    EnteringPower,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum StatValue {
    Fixed(u16),
    TappedCount(CountFilter),
    Not(&'static StatValue),
    CardsDrawnThisTurn,
    ControllerHandSize,
    NonlandEntriesThisTurn,
    SourceAttacking,
    AttackingCreatures,
    /// Unqualified all-player count; distinct from controller-owned attackers.
    AllAttackingCreatures,
    SourceModified,
    Tokens(CountFilter),
    AtLeast {
        value: &'static StatValue,
        minimum: u16,
    },
    SpellsCastThisTurn,
    LifeGainOccurredThisTurn,
    ControllerLife,
    Count(CountFilter),
    GreatestManaValue(CountFilter),
    DistinctNames(CountFilter),
    CountUnion(&'static [CountFilter]),
    Colors(CountFilter),
    CardTypes(CountFilter),
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
pub(crate) struct BaseStatSetter {
    pub power: Option<i128>,
    pub toughness: Option<i128>,
    pub own_turn: bool,
    pub condition: Option<StatValue>,
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
    pub(crate) fn matches_characteristics(
        self,
        card: Card,
        kinds: u16,
        added: GrantedSubtypes,
        yours: bool,
        same: bool,
        attacking: bool,
    ) -> bool {
        (match self.relation {
            0 => yours,
            1 => !yours,
            _ => true,
        }) && (!self.other || !same)
            && self
                .subtype
                .is_none_or(|subtype| card.has_subtype(subtype) || added.contains(subtype))
            && (!self.attacking || attacking)
            && self.tokens.is_none_or(|tokens| card.is_token() == tokens)
            && (!self.artifact || kinds & 16 != 0)
    }
}

/// Commuting positive layer-6 grants. Keyword-qualified recipients and ability
/// removal/prohibition remain unsupported; no recursive dependency solver is implied.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct KeywordGrant {
    pub condition: Option<StatValue>,
    pub requires_life_gain: bool,
    pub filter: CreatureFilter,
    pub source_only: bool,
    pub own_turn: bool,
    pub keywords: Keywords,
}
impl KeywordGrant {
    // Borrowed actual/virtual characteristics are passed without allocating a view.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn matches_characteristics(
        self,
        card: Card,
        kinds: u16,
        added: GrantedSubtypes,
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
            && (self.filter.subtype.is_some() || kinds & 1 != 0)
            && self.filter.matches_characteristics(card, kinds, added, yours, same, attacking)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct StaticModifier {
    pub own_turn: bool,
    pub requires_life_gain: bool,
    pub filter: CreatureFilter,
    pub stats: Stats,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct EnchantedModifier {
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
    /// Enchant object-kind union, evaluated using live characteristics.
    pub kinds: u8,
    pub relation: i8,
    pub bonus: EquipmentSpec,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Subject {
    Entering,
    Source,
    Targets,
    Attached,
}

/// Immutable +1/+1 counter replacements evaluated against the pre-entry world.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EntryCounterAmount {
    ColorsSpent,
    IfNotCastOrZero(u16),
    Fixed(u16),
    Stat(StatValue),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct EntryCounterRule {
    pub guard: StepGuard,
    pub amount: EntryCounterAmount,
}

/// Immutable replacement programs; active games retain only exact event/source identities.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ReplacementFilter {
    Permanent,
    Creature,
    CreatureOrVehicle,
    CreatureSpacecraftPlanet,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ReplacementSpec {
    Counters {
        filter: ReplacementFilter,
        kind: Option<CounterKind>,
        factor: u32,
        add: u32,
        effects_only: bool,
    },
    DoubleTokens {
        effects_only: bool,
    },
    AddToken(Card),
    ReplaceArtifactTokens(Card),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum CounterKind {
    Lore,
    PlusOne,
    MinusOne,
    Stun,
    Loyalty,
}
impl CounterKind {
    pub const ALL: [Self; 5] = [
        Self::PlusOne,
        Self::MinusOne,
        Self::Stun,
        Self::Loyalty,
        Self::Lore,
    ];
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Counters {
    #[serde(default, skip_serializing_if = "is_zero_counter")]
    pub lore: u64,
    pub plus_one: u64,
    pub minus_one: u64,
    pub stun: u64,
    #[serde(default, skip_serializing_if = "is_zero_counter")]
    pub loyalty: u64,
}
fn is_zero_counter(value: &u64) -> bool {
    *value == 0
}
impl Counters {
    pub fn get(self, kind: CounterKind) -> u64 {
        match kind {
            CounterKind::PlusOne => self.plus_one,
            CounterKind::MinusOne => self.minus_one,
            CounterKind::Stun => self.stun,
            CounterKind::Loyalty => self.loyalty,
            CounterKind::Lore => self.lore,
        }
    }
    pub fn total(self) -> u128 {
        CounterKind::ALL
            .into_iter()
            .map(|kind| u128::from(self.get(kind)))
            .sum()
    }
    pub(crate) fn available(self, kind: Option<CounterKind>) -> u128 {
        kind.map_or_else(|| self.total(), |kind| u128::from(self.get(kind)))
    }
    pub(crate) fn add(&mut self, kind: CounterKind, amount: u64) {
        let counter = match kind {
            CounterKind::PlusOne => &mut self.plus_one,
            CounterKind::MinusOne => &mut self.minus_one,
            CounterKind::Stun => &mut self.stun,
            CounterKind::Loyalty => &mut self.loyalty,
            CounterKind::Lore => &mut self.lore,
        };
        *counter = counter
            .checked_add(amount)
            .capacity("counter representation exhausted");
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CastTarget {
    Creature(CastRelation),
    OpponentOrCreature,
    Source,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CastFilter {
    PermanentAdventure,
    Any,
    Creature,
    Noncreature,
    /// Historical qualifying cast ordinal, evaluated only with the actual caster.
    InstantSorceryOrdinal(u64),
    InstantSorcery,
    Sorcery,
    Artifact,
    // Compiler vocabulary; no complete admitted card currently uses this filter.
    #[allow(dead_code)]
    Enchantment,
    Subtype(&'static str),
    AnyOf(&'static [CastFilter]),
    Multicolored,
    Targeted {
        filter: &'static CastFilter,
        target: CastTarget,
    },
    ManaValueMaximum {
        filter: &'static CastFilter,
        maximum: u64,
    },
    ManaValueMinimum {
        filter: &'static CastFilter,
        minimum: u64,
    },
}
impl CastFilter {
    pub fn matches(self, card: Card) -> bool {
        self.matches_mana_value(card, card.fixed_mana_value())
    }
    pub(crate) fn matches_mana_value(self, card: Card, mana_value: u64) -> bool {
        match self {
            Self::Any => true,
            Self::PermanentAdventure => {
                card.adventure_spell().is_some()
                    && card.kind_mask() & (1 | 2 | 16 | 32 | 64 | 256) != 0
            }
            // Contextual cast observers use Game::cast_filter_matches instead.
            Self::Targeted { .. } | Self::InstantSorceryOrdinal(_) => false,
            Self::Multicolored => card.colors().count_ones() >= 2,
            Self::ManaValueMaximum { filter, maximum } => {
                filter.matches_mana_value(card, mana_value) && mana_value <= maximum
            }
            Self::Creature => card.is_creature(),
            Self::Noncreature => !card.is_creature(),
            Self::InstantSorcery => card.kind_mask() & (4 | 8) != 0,
            Self::Sorcery => card.kind_mask() & 4 != 0,
            Self::Artifact => card.is_artifact(),
            Self::Enchantment => card.kind_mask() & 32 != 0,
            Self::Subtype(subtype) => card.has_subtype(subtype),
            Self::AnyOf(filters) => filters
                .iter()
                .any(|filter| filter.matches_mana_value(card, mana_value)),
            Self::ManaValueMinimum { filter, minimum } => {
                filter.matches_mana_value(card, mana_value) && mana_value >= minimum
            }
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CostTurn {
    Any,
    // Parsed vocabulary; all complete own-turn bodies still have other dependencies.
    #[allow(dead_code)]
    Own,
    Other,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SpellReductionAmount {
    CreaturePowerCount {
        minimum: u32,
    },
    Fixed(u32),
    Attackers {
        factor: u16,
    },
    Stat {
        value: StatValue,
        factor: u16,
    },
    CreaturePower {
        filter: CreatureFilter,
        greatest: bool,
    },
    SourcePower,
    EquippedPower,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SpellReduction {
    pub source_only: bool,
    pub filter: CastFilter,
    pub amount: SpellReductionAmount,
    pub turn: CostTurn,
    pub guard: StepGuard,
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
    EnchantedCreatures,
    Life,
    Hand,
    Creatures,
    TappedCreatures,
    Lands,
    GreatestCreaturePower,
    GreatestOtherCreaturePower,
    TotalCreaturePower,
    TotalCreatureToughness,
    SpellsCast,
}
/// Continuous restrictions are not abilities granted to their subject.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CombatRestriction {
    pub subject: Subject,
    pub attack: bool,
    pub block: bool,
    pub unless: Option<StepGuard>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum StepGuard {
    SourcePowerAtLeast(u16),
    SourceToughnessAtMost(u16),
    Outlaw,
    Always,
    Attacked,
    CreatureDied,
    OpponentLostLife,
    GainedLife,
    SourceTapped,
    Expression {
        value: StatValue,
        minimum: Option<i128>,
        maximum: Option<i128>,
    },
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
    PowerAboveBase,
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

/// Complete entry-time predicates over the post-event permanent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EntryPredicate {
    CreaturePowerAtMost(i32),
    SubtypeAny(&'static [&'static str]),
    KindOrSubtype { kinds: u16, subtype: &'static str },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TriggerCondition {
    /// Historical lore threshold crossing; never a live resolution recheck.
    Chapter {
        number: u16,
    },
    ManaSpent {
        minimum: u64,
        increment: bool,
        event: &'static TriggerCondition,
    },
    /// Immutable Equipment program, with the exact recipient as ability source.
    Granted(&'static TriggerCondition),
    SelfAttacksWhileSaddled,
    SelfEntersOrAttacksWhileSaddled,
    SelfSaddledFirst,
    /// A paid promise belongs to this actual cast-to-battlefield entry event.
    SelfGiftEnters,
    Foraged,
    FoodsSacrificed,
    Reflexive,
    /// A cast-event qualification, not an intervening-if resolution guard.
    DuringOpponentsTurn(&'static TriggerCondition),
    OncePerTurn(&'static TriggerCondition),
    InterveningIf {
        guard: StepGuard,
        event: &'static TriggerCondition,
    },
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
    SelfTargeted {
        relation: CastRelation,
        spells_only: bool,
    },
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
    Sacrificed {
        filter: SacrificeFilter,
        own_turn: bool,
    },
    CreatureDiesFiltered {
        filter: CreatureFilter,
    },
    PermanentToGraveyard {
        kinds: u16,
        relation: CastRelation,
        exclude_source_kinds: u16,
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
    SelfOffspring,
    SelfKicked,
    CreatureExploresYou,
    CreatureExploresCardYou {
        land: bool,
    },
    SelfBargained,
    SelfEvidence,
    EvidenceCollected,
    SelfDies,
    SelfEntersOrDies,
    SelfAttacks,
    /// Declaration-time condition; not a resolution-time intervening if.
    SelfAttacksAlone,
    EquippedAttacks,
    GrantedAttacks,
    SelfBecomesTapped,
    AttackDeclared {
        minimum: u16,
        power_min: Option<i32>,
        guard: StepGuard,
    },
    SelfEntersOrAttacks,
    /// Explicit self entry OR another owned permanent satisfying all filters.
    /// The self arm bypasses the other-permanent filters; it is never counted twice.
    PermanentEntersYou {
        kinds: u16,
        nontoken: bool,
        subtype: Option<&'static str>,
        include_source: bool,
    },
    OtherFilteredEntersYou(EntryPredicate),
    OtherCreatureEntersYou,
    CreatureEnters {
        relation: CastRelation,
        other: bool,
    },
    OtherSubtypeEntersYou(&'static str),
    SubtypeEntersYou(&'static str),
    ArtifactEntersYou,
    EnchantmentEntersYou,
    CreatureEntersYou,
    OtherArtifactEntersYou,
    LandEntersYou,
}

impl TriggerCondition {
    pub(crate) fn mana_spent_rule(self) -> Option<(u64, bool)> {
        match self {
            Self::ManaSpent {
                minimum, increment, ..
            } => Some((minimum, increment)),
            Self::Granted(e)
            | Self::OncePerTurn(e)
            | Self::DuringOpponentsTurn(e)
            | Self::InterveningIf { event: e, .. } => e.mana_spent_rule(),
            _ => None,
        }
    }

    pub(crate) fn is_granted(self) -> bool {
        match self {
            Self::Granted(_) | Self::GrantedAttacks => true,
            Self::OncePerTurn(e)
            | Self::DuringOpponentsTurn(e)
            | Self::InterveningIf { event: e, .. }
            | Self::ManaSpent { event: e, .. } => e.is_granted(),
            _ => false,
        }
    }
    pub(crate) fn requires_opponents_turn(self) -> bool {
        match self {
            Self::DuringOpponentsTurn(_) => true,
            Self::Granted(event)
            | Self::OncePerTurn(event)
            | Self::ManaSpent { event, .. }
            | Self::InterveningIf { event, .. } => event.requires_opponents_turn(),
            _ => false,
        }
    }
    pub(crate) fn event(self) -> Self {
        match self {
            Self::Granted(event)
            | Self::OncePerTurn(event)
            | Self::DuringOpponentsTurn(event)
            | Self::InterveningIf { event, .. }
            | Self::ManaSpent { event, .. } => event.event(),
            event => event,
        }
    }
    pub(crate) fn once_per_turn(self) -> bool {
        match self {
            Self::OncePerTurn(_) => true,
            Self::Granted(event)
            | Self::InterveningIf { event, .. }
            | Self::ManaSpent { event, .. }
            | Self::DuringOpponentsTurn(event) => event.once_per_turn(),
            _ => false,
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
    pub alternatives: &'static [ManaCost],
    pub additional: Option<AdditionalCost>,
    pub mana_component: bool,
    pub tap: bool,
    pub sacrifice: bool,
    pub exile_self: bool,
    pub discard: bool,
    pub life: u16,
    pub remove: Option<(Option<CounterKind>, u32)>,
    pub loyalty: Option<i32>,
}
impl ActivationCost {
    pub(crate) fn mana_options(self) -> ManaCostOptions {
        std::iter::once(ManaCost {
            colored: self.colored,
            generic: self.generic,
        })
        .chain(self.alternatives.iter().copied())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ActivationOrigin {
    Battlefield,
    Hand,
    Graveyard,
}
/// Starting restrictions, not costs that must still hold during payment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ActivationGuard {
    Always,
    OwnTurn,
    ControlSubtypeAny(&'static [&'static str]),
}
impl ActivationGuard {
    pub(crate) fn subtypes(self) -> &'static [&'static str] {
        match self {
            Self::Always | Self::OwnTurn => &[],
            Self::ControlSubtypeAny(types) => types,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ActivatedAbility {
    pub exhaust: bool,
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
        if self.cost.loyalty.is_some() {
            return None;
        }
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
    pub const SKULK: Self = Self(1048576);
    pub const NO_POWER_LE_TWO_BLOCKERS: Self = Self(2097152);
    pub const NO_POWER_GE_THREE_BLOCKERS: Self = Self(4194304);
    /// Equivalent evasion text does not itself grant the skulk keyword.
    pub const NO_GREATER_POWER_BLOCKERS: Self = Self(8388608);
    pub fn contains(self, keyword: Self) -> bool {
        self.0 & keyword.0 == keyword.0
    }
    pub const fn intersects(self, other: Self) -> bool {
        self.0 & other.0 != 0
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
    /// Catalogue-scoped factory lookup identity, distinct from the copied name.
    pub factory_identity: &'static str,
    pub name: &'static str,
    pub subtypes: &'static str,
    pub colors: u8,
}

/// Immutable abilities of a generated command-zone emblem.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct EmblemInfo {
    pub oracle_text: &'static str,
}

/// Self replacement evaluated against the battlefield immediately before entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EntryTapped {
    UnlessPlainsOrIsland,
    Never,
    Always,
    OpponentsTurn,
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
    /// Intrinsic colors (W=1, U=2, B=4, R=8, G=16), including both hybrid halves
    /// and token colors. Unsupported multiple-face identities describe the front.
    pub(crate) fn combat_restriction(self) -> Option<CombatRestriction> {
        generated::combat_restriction(self)
    }
    pub(crate) fn attached_turn_keywords(self) -> Keywords {
        generated::attached_turn_keywords(self)
    }
    pub(crate) fn attached_subtype(self) -> Option<&'static str> {
        generated::attached_subtype(self)
    }
    pub(crate) fn replacement_effects(self) -> &'static [ReplacementSpec] {
        generated::replacement_effects(self)
    }
    pub fn colors(self) -> u8 {
        generated::COLORS[self.index()]
    }
    /// Intrinsic subtype membership in every zone. Dynamic type/ability-changing
    /// effects use Game/ObjectView membership; changeling adds creature types, never other subtypes.
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
    /// Stable catalogue lookup/serialization identity, distinct from a token copy's printed name.
    pub fn identity(self) -> &'static str {
        generated::adventure_identity(self).unwrap_or_else(|| {
            self.token_info()
                .map_or(self.name(), |token| token.factory_identity)
        })
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
    pub fn emblem_info(self) -> Option<&'static EmblemInfo> {
        self.index()
            .checked_sub(generated::EMBLEM_BASE)
            .and_then(|offset| generated::EMBLEMS.get(offset))
    }
    pub fn is_emblem(self) -> bool {
        self.emblem_info().is_some()
    }
    /// Closed battlefield casting restrictions; unsupported bodies emit no rule.
    pub(crate) fn static_cast_restriction(self) -> u8 {
        generated::static_cast_restriction(self)
    }
    /// Executable prohibition for this card's own spell while on the stack.
    /// Unsupported bodies do not emit this metadata.
    pub fn cannot_be_countered(self) -> bool {
        generated::uncounterable(self)
    }
    pub(crate) fn exhaust_mask(self) -> u64 {
        generated::exhaust_mask(self)
    }
    pub(crate) fn loyalty_mask(self) -> u64 {
        self.activations()
            .iter()
            .enumerate()
            .fold(0, |mask, (i, a)| {
                mask | if a.cost.loyalty.is_some() {
                    1u64 << i
                } else {
                    0
                }
            })
    }
    pub fn is_token(self) -> bool {
        self.token_info().is_some()
    }
    pub fn vehicle_stats(self) -> Option<(i128, i128)> {
        generated::vehicle_stats(self).map(|s| (s.power, s.toughness))
    }
    /// Immutable inset spell definition; never a second physical deck card.
    pub fn adventure_spell(self) -> Option<Card> {
        generated::adventure_spell(self)
    }
    /// Physical parent of an internal Adventure spell definition.
    pub fn adventure_parent(self) -> Option<Card> {
        generated::adventure_parent(self)
    }
    /// Immutable Omen inset spell; never a separate physical card.
    pub fn omen_spell(self) -> Option<Card> {
        generated::omen_spell(self)
    }
    pub fn omen_parent(self) -> Option<Card> {
        generated::omen_parent(self)
    }
    pub fn inset_parent(self) -> Option<Card> {
        self.adventure_parent().or_else(|| self.omen_parent())
    }
    pub fn warp_cost(self) -> Option<ManaCost> {
        generated::warp_cost(self)
    }
    pub fn plot_cost(self) -> Option<ManaCost> {
        generated::plot_cost(self)
    }
    pub(crate) fn additional_cost(self) -> Option<AdditionalCost> {
        generated::additional_cost(self)
    }
    pub(crate) fn live_static_effects(self) -> &'static [LiveStaticEffect] {
        generated::live_static_effects(self)
    }
    pub(crate) fn tiered_costs(self) -> Option<&'static [ManaCost]> {
        generated::tiered_costs(self)
    }
    pub(crate) fn base_stat_setter(self) -> Option<BaseStatSetter> {
        generated::base_stat_setter(self)
    }
    pub(crate) fn dynamic_stats(self) -> Option<DynamicStats> {
        generated::dynamic_stats(self)
    }
    pub(crate) fn scaling_modifiers(self) -> &'static [ScalingModifier] {
        generated::scaling_modifiers(self)
    }
    /// Immutable entering-kind mask; valid while this source is on the battlefield.
    pub(crate) fn entry_trigger_suppression(self) -> u16 {
        generated::entry_trigger_suppression(self)
    }
    /// Opposing entering-kind mask while this source is on the battlefield.
    pub(crate) fn opponent_entry_tapped(self) -> u16 {
        generated::opponent_entry_tapped(self)
    }

    pub(crate) fn static_modifiers(self) -> &'static [StaticModifier] {
        generated::static_modifiers(self)
    }
    pub(crate) fn enchanted_modifiers(self) -> &'static [EnchantedModifier] {
        generated::enchanted_modifiers(self)
    }
    pub(crate) fn flash_surcharge(self) -> Option<ManaCost> {
        generated::flash_surcharge(self)
    }
    pub(crate) fn equipment(self) -> Option<EquipmentSpec> {
        generated::equipment(self)
    }
    pub(crate) fn role(self) -> Option<RoleKind> {
        generated::role(self)
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
    pub fn saga_final_chapter(self) -> Option<u16> {
        self.triggers()
            .iter()
            .filter_map(|a| match a.condition {
                TriggerCondition::Chapter { number } => Some(number),
                _ => None,
            })
            .max()
    }
    pub(crate) fn enters_counters(self) -> Counters {
        let mut counters = generated::DEFINITIONS[self.0 as usize].enters;
        counters.loyalty = self.starting_loyalty();
        counters.lore = u64::from(self.saga_final_chapter().is_some());
        counters
    }
    pub(crate) fn entry_counter_rules(self) -> &'static [EntryCounterRule] {
        generated::entry_counter_rules(self)
    }
    pub fn is_planeswalker(self) -> bool {
        self.kind_mask() & 64 != 0
    }
    /// Printed starting loyalty of the represented face; zero for non-walkers.
    pub fn starting_loyalty(self) -> u64 {
        generated::starting_loyalty(self)
    }
    pub(crate) fn mana_color(self) -> Option<usize> {
        generated::DEFINITIONS[self.0 as usize].mana_color
    }
    pub fn x_symbols(self) -> u8 {
        generated::x_symbols(self)
    }
    pub(crate) fn x_entry_counters(self) -> bool {
        generated::x_entry_counters(self)
    }
    pub(crate) fn fixed_mana_value(self) -> u64 {
        // The canonical equivalent takes every two-brid symbol's generic side;
        // a cheaper chosen payment never changes the printed mana value.
        let (colored, generic) = self.cost();
        u64::from(generic) + colored.into_iter().map(u64::from).sum::<u64>()
    }
    pub(crate) fn cost(self) -> (Mana, u32) {
        let definition = &generated::DEFINITIONS[self.0 as usize];
        (definition.colored, definition.generic)
    }
    pub(crate) fn target_cost_reduction(self) -> Option<&'static TargetSpellReduction> {
        generated::target_cost_reduction(self)
    }
    /// Timing permission is an as-though effect, not a granted Flash keyword.
    pub(crate) fn cast_timing_grants(self) -> &'static [CastFilter] {
        generated::cast_timing_grants(self)
    }
    pub(crate) fn cost_reductions(self) -> &'static [SpellReduction] {
        generated::cost_reductions(self)
    }
    pub(crate) fn has_hybrid_cost(self) -> bool {
        !generated::cost_alternatives(self).is_empty()
    }
    pub fn collect_evidence_cost(self) -> Option<u16> {
        generated::collect_evidence(self).map(|v| v.0)
    }
    pub(crate) fn evidence_reduction(self) -> u32 {
        generated::collect_evidence(self).map_or(0, |v| v.1)
    }
    pub fn has_convoke(self) -> bool {
        generated::has_convoke(self)
    }
    pub fn has_bargain(self) -> bool {
        generated::bargain_reduction(self).is_some()
    }
    pub(crate) fn bargain_reduction(self) -> u32 {
        generated::bargain_reduction(self).unwrap_or(0)
    }
    /// Fixed optional kicker surcharge for this immutable whole-card definition.
    pub fn kicker_cost(self) -> Option<ManaCost> {
        generated::kicker_cost(self)
    }
    pub(crate) fn gift(self) -> Option<GiftSpec> {
        generated::gift(self)
    }
    pub(crate) fn kicked_entry_counters(self) -> u16 {
        generated::kicked_entry_counters(self)
    }
    pub(crate) fn counter_keyword_grants(self) -> &'static [(CounterKind, Keywords)] {
        generated::counter_keyword_grants(self)
    }
    /// Optional additional mana cost for the immutable offspring recipe.
    pub fn offspring_cost(self) -> Option<ManaCost> {
        generated::offspring_cost(self)
    }
    /// Immutable factory with the source copiable values and a 1/1 base.
    pub fn offspring_copy(self) -> Option<Card> {
        generated::offspring_copy(self)
    }
    /// The fixed alternative mana cost for casting this complete card from its graveyard.
    pub fn flashback_cost(self) -> Option<ManaCost> {
        generated::flashback_cost(self)
    }
    pub(crate) fn cost_options(self) -> ManaCostOptions {
        let definition = &generated::DEFINITIONS[self.0 as usize];
        std::iter::once(ManaCost {
            colored: definition.colored,
            generic: definition.generic,
        })
        .chain(generated::cost_alternatives(self).iter().copied())
    }
    pub(crate) fn spell_payments(self, pool: Mana, mut emit: impl FnMut(Mana)) {
        self.spell_payments_for_reduction(pool, 0, &mut emit);
    }
    pub(crate) fn reduced_spell_payments(
        self,
        pool: Mana,
        reduction: u32,
        mut emit: impl FnMut(Mana),
    ) {
        if reduction == 0 {
            self.spell_payments(pool, &mut emit);
            return;
        }
        self.spell_payments_for_reduction(pool, reduction, &mut emit);
    }
    fn spell_payments_for_reduction(self, pool: Mana, reduction: u32, mut emit: impl FnMut(Mana)) {
        if !self.has_hybrid_cost() {
            let (colored, generic) = self.cost();
            payments(pool, colored, generic.saturating_sub(reduction), emit);
            return;
        }
        // Different hybrid assignments can consume the same pool allocation.
        // Emit each action once, without charging fixed-cost casts an allocation.
        let mut seen = Vec::new();
        for cost in self.cost_options() {
            payments(
                pool,
                cost.colored,
                cost.generic.saturating_sub(reduction),
                |payment| {
                    if !seen.contains(&payment) {
                        seen.push(payment);
                        emit(payment);
                    }
                },
            );
        }
    }
    pub(crate) fn index(self) -> usize {
        self.0 as usize
    }
}

impl Serialize for Card {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.identity())
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

/// Predefined Role characteristics are rules data, independent of printed card names.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RoleKind {
    Cursed,
    Monster,
    Royal,
    Sorcerer,
    Wicked,
    YoungHero,
}
