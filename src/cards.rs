#[path = "ability_layers.rs"]
mod ability_layers;
pub(crate) use ability_layers::{
    AbilityLayer, AnimationCreatureTypes, AnimationDuration, AnimationSpec,
    AttachedCharacteristics, LandManaRecipients, PermanentForm, StaticAbilityFragment,
    StaticAbilityGroup, UntapRestriction,
};

#[path = "card_mana_symbols.rs"]
mod mana_symbols;

use crate::capacity::CapacityExt;
use crate::engine::{GrantedSubtypes, Zone};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Immutable additions made as part of a token-copy effect (CR707.9).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub struct CopyExceptions {
    pub sacrifice_abilities: u16,
    pub nightmare: bool,
    pub artifact: bool,
    pub golem: bool,
    pub fractal: bool,
    pub base: Option<(i16, i16)>,
}
/// A layer-one copy between independently selected exact objects. Group absence
/// at resolution is intentional: losing either required group prevents the copy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct InPlaceCopySpec {
    /// None copies the ability source; Some selects this target group.
    pub recipient_group: Option<u16>,
    pub blueprint_group: u16,
    pub overrides: crate::engine::CopyOverrides,
    pub add_kinds: u16,
    pub base: Option<(i16, i16)>,
    pub next_turn: bool,
}
impl CopyExceptions {
    pub const NONE: Self = Self {
        sacrifice_abilities: 0,
        nightmare: false,
        artifact: false,
        golem: false,
        fractal: false,
        base: None,
    };
    pub const HASTE_AND_SACRIFICE: Self = Self {
        sacrifice_abilities: 1,
        nightmare: false,
        artifact: false,
        golem: false,
        fractal: false,
        base: None,
    };
    pub const NIGHTMARE: Self = Self {
        sacrifice_abilities: 0,
        nightmare: true,
        artifact: false,
        golem: false,
        fractal: false,
        base: None,
    };
    pub const ARTIFACT: Self = Self {
        artifact: true,
        ..Self::NONE
    };
    pub const GOLEM: Self = Self {
        artifact: true,
        golem: true,
        base: Some((3, 3)),
        ..Self::NONE
    };
    pub const FRACTAL: Self = Self {
        fractal: true,
        base: Some((0, 0)),
        ..Self::NONE
    };
    pub fn union(self, other: Self) -> Self {
        Self {
            sacrifice_abilities: self
                .sacrifice_abilities
                .checked_add(other.sacrifice_abilities)
                .capacity("copy sacrifice abilities"),
            nightmare: self.nightmare || other.nightmare,
            artifact: self.artifact || other.artifact,
            golem: self.golem || other.golem,
            fractal: self.fractal || other.fractal,
            base: other.base.or(self.base),
        }
    }
}

/// A temporary permission on one graveyard incarnation, independent of its grantor.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum GraveyardCastKind {
    /// Play this exact card normally, including a land, through the deadline.
    Play,
    Cast,
    Flashback,
}

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
    /// Attached to the exact object selected in another target group.
    AttachedToGroup(u16),
    DifferentControllers,
    OtherThanGroup(u16),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TargetRule {
    ManaValueAtMost {
        base: &'static TargetRule,
        amount: AmountValue,
        maximum: Option<u64>,
    },
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
    GraveCreatureEnteredThisTurn,
    GraveCreatureSourcePower {
        maximum: Option<i128>,
    },
    GraveCreaturePower {
        excluded_subtype: &'static str,
        maximum: Option<i128>,
    },
    Creature,
    /// Bound to the captured attachment attack defender before target selection.
    DefendingCreature,
    DamagedPlayerPermanent {
        kinds: u8,
    },
    CreatureOrVehicle {
        relation: i8,
    },
    Any,
    Spell,
    SingleTargetStackObject,
    YourSpellKinds {
        kinds: u8,
    },
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

impl TargetRule {
    pub const fn creature_you_control() -> Self {
        Self::Object {
            zone: TargetZone::Battlefield,
            kinds: 1,
            relation: 0,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        }
    }
    pub const fn another_nonland_permanent_you_control() -> Self {
        Self::Object {
            zone: TargetZone::Battlefield,
            kinds: 0,
            relation: 0,
            other: true,
            nonland: true,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        }
    }
}

/// Immutable live constraints; indirection keeps target plans compact in checkpoints.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TargetPredicate {
    OwnedByController,
    EnteredGraveyardThisTurn,
    ManaValueAtMostLifeGained,
    OtherThanSource,
    Blocking,
    Nontoken,
    Permanent,
    Attacking,
    Nonattacking,
    /// The sole creature currently attacking, irrespective of its defender.
    AttackingAlone,
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
    Active,
    DamagedPlayer,
    Controller,
    Opponent,
    Each,
    Targeted,
}

/// A color choice excludes colorless (CR105.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChoiceColor {
    White,
    Blue,
    Black,
    Red,
    Green,
}
impl ChoiceColor {
    pub const ALL: [Self; 5] = [Self::White, Self::Blue, Self::Black, Self::Red, Self::Green];
    pub(crate) const fn index(self) -> usize {
        self as usize
    }
}
/// Validated index into the pinned complete creature-type taxonomy; wire values
/// remain readable names and no owned String is stored in a game branch.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CreatureType(u16);
impl CreatureType {
    pub fn from_name(name: &str) -> Option<Self> {
        CREATURE_TYPES
            .binary_search(&name)
            .ok()
            .map(|i| Self(u16::try_from(i).expect("creature taxonomy fits u16")))
    }
    pub fn name(self) -> &'static str {
        CREATURE_TYPES[usize::from(self.0)]
    }
}
impl Serialize for CreatureType {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(self.name())
    }
}
impl<'de> Deserialize<'de> for CreatureType {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let name = String::deserialize(d)?;
        Self::from_name(&name).ok_or_else(|| serde::de::Error::custom("not a legal creature type"))
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EntryChoiceValue {
    CardName(crate::CardName),
    Color(ChoiceColor),
    CreatureType(CreatureType),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum EntryChoiceKind {
    CardNameAfterOpponentHand,
    LandCardName,
    Color,
    CreatureType,
}
impl EntryChoiceKind {
    pub(crate) fn matches(self, value: EntryChoiceValue) -> bool {
        if self == Self::LandCardName {
            return matches!(value, EntryChoiceValue::CardName(name) if name.is_land());
        }
        matches!(
            (self, value),
            (
                Self::CardNameAfterOpponentHand,
                EntryChoiceValue::CardName(_)
            ) | (Self::Color, EntryChoiceValue::Color(_))
                | (Self::CreatureType, EntryChoiceValue::CreatureType(_))
        )
    }
}
/// Layer-seven own-creature bonus using one source's entry choice. The optional
/// counter factor borrows the same sparse counter world as mana planning.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ChosenModifier {
    pub kind: EntryChoiceKind,
    pub other: bool,
    pub stats: Stats,
    pub source_counter: Option<CounterKind>,
}
/// Rules attached to produced mana, captured independently of its producer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ManaRestriction {
    /// Only the costs of casting a spell with the artifact card type.
    ArtifactSpell,
    /// Any cost except casting a spell without the artifact card type.
    ExceptNonArtifactSpells,
    /// Spells or activated abilities of any source with this subtype.
    SubtypeSpellOrActivation(&'static str),
    InstantSorcery,
    Unrestricted,
    SpellSubtype(&'static str),
    ChosenCreatureSpellOrActivation,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ManaSpellBonus {
    None,
    DragonCreatureHaste,
    CopyRedInstantSorcery,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize)]
pub enum ManaRetention {
    #[default]
    Step,
    Combat,
    Turn,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ManaQuality {
    pub restriction: ManaRestriction,
    pub spell_bonus: ManaSpellBonus,
    pub retention: ManaRetention,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ManaProduction {
    Tagged {
        production: &'static ManaProduction,
        quality: &'static ManaQuality,
    },
    ChosenColor,
    SourcePower {
        color: u8,
    },
    /// All source-power mana has one selected color, never a split.
    TapSourcePowerChoice {
        mask: u8,
    },
    MonocoloredPermanentColors,
    Fixed(Mana),
    CountSubtype {
        color: u8,
        subtype: &'static str,
    },
    /// Split an exact small amount among allowed mana colors, including repetitions.
    Combination {
        mask: u8,
        amount: u32,
    },
    Choice {
        mask: u8,
        amount: u32,
    },
}
impl ManaProduction {
    pub(crate) const fn any_color(amount: u32) -> Self {
        Self::Choice {
            mask: (1 << ChoiceColor::White.index())
                | (1 << ChoiceColor::Blue.index())
                | (1 << ChoiceColor::Black.index())
                | (1 << ChoiceColor::Red.index())
                | (1 << ChoiceColor::Green.index()),
            amount,
        }
    }
    pub(crate) fn base(self) -> Self {
        match self {
            Self::Tagged { production, .. } => production.base(),
            value => value,
        }
    }
    pub(crate) fn visit(self, mut emit: impl FnMut(Mana) -> bool) -> bool {
        match self {
            Self::Tagged { production, .. } => production.visit(emit),
            Self::ChosenColor
            | Self::SourcePower { .. }
            | Self::TapSourcePowerChoice { .. }
            | Self::MonocoloredPermanentColors
            | Self::CountSubtype { .. } => {
                panic!("dynamic mana requires a characteristic world")
            }
            Self::Fixed(mana) => emit(mana),
            Self::Combination { mask, amount } => {
                fn combinations(
                    mask: u8,
                    left: u32,
                    mana: &mut Mana,
                    emit: &mut impl FnMut(Mana) -> bool,
                ) -> bool {
                    let index = mask.trailing_zeros() as usize;
                    if index >= 6 {
                        return left == 0 && emit(*mana);
                    }
                    let remaining = mask & !(1 << index);
                    if remaining == 0 {
                        mana[index] = left;
                        let done = emit(*mana);
                        mana[index] = 0;
                        return done;
                    }
                    for count in 0..=left {
                        mana[index] = count;
                        if combinations(remaining, left - count, mana, emit) {
                            return true;
                        }
                    }
                    mana[index] = 0;
                    false
                }
                combinations(mask & 63, amount, &mut [0; 6], &mut emit)
            }
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
    HasX,
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

/// Explicit destinations for cards looked at without changing zones first.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LibraryPickDestination {
    LandsTappedRestHand,
    Hideaway,
    Hand,
    Battlefield,
    Top,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LibraryLook {
    Amount(AmountValue),
    Fixed(u16),
    ChosenX,
    UntilKinds(u16),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct LibrarySelectionEffect {
    pub spec: LibrarySelectionSpec,
    pub look: LibraryLook,
    pub destination: LibraryPickDestination,
    pub public: bool,
    pub maximum_mana_value: Option<AmountValue>,
}

/// Fixed qualities admitted by library-search instructions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum SearchFilter {
    Any,
    ManaValue { minimum: u64, maximum: u64 },
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
            Self::Any => true,
            Self::ManaValue { minimum, maximum } => {
                (minimum..=maximum).contains(&card.fixed_mana_value())
            }
            Self::AnyOf(filters) => filters.iter().any(|filter| filter.matches(card)),
            Self::AllOf(filters) => filters.iter().all(|filter| filter.matches(card)),
            Self::Name(name) => crate::card_names::same_name(card.name(), name),
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SearchDestination {
    /// Reveal the selected cards, then choose one destination for the selection.
    HandOrGraveyard,
    /// Face-up exile linked to this exact source incarnation, without an automatic return.
    LinkedExile,
    Graveyard,
    /// Shuffle, then put the selected cards on top (last selected uppermost).
    Top,
    Hand,
    Battlefield,
    BattlefieldThenHand {
        battlefield: u16,
    },
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
    ExileGreatestManaValue,
    ExileGraveyard,
    SacrificeDifferentNames(u16),
    /// Choose distinct untapped permanents as a non-targeting activation cost.
    TapPermanents(u16),
    ExileGraveyardCards(u16),
    HandToTop,
    RemoveCreatureCounters(u16),
    UntapLands {
        maximum: u16,
    },
    /// Public exact targeted graveyard return; only the independent defender is chosen.
    TargetAttackingReturn {
        counter: CounterKind,
    },
    HandCreature,
    HandPermanent {
        maximum: u32,
        tapped: bool,
    },
    HandExileCopy,
    HandLand {
        tapped: bool,
    },
    /// Exact public cards from the current simultaneous discard batch.
    DiscardedLand {
        tapped: bool,
    },
    LandCards {
        maximum: u32,
        tapped: bool,
    },
    CollectEvidence(u16),
    Forage,
    ForageOrBlack,
    Sacrifice,
    ReturnToHand,
    Discard,
    DiscardNamed(&'static str),
    /// One controlled permanent matching the sacrifice filter, or one private hand card.
    /// The flag restricts the discard route, not the permanent, for resolution or activation costs.
    SacrificeOrDiscard {
        nonland: bool,
    },
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

impl AdditionalCost {
    /// A union cost selects exactly one route using the selected object's zone.
    /// The hand route has its own printed restriction; the permanent filter
    /// applies only to the sacrifice route.
    pub(crate) fn in_zone(mut self, zone: Zone) -> Self {
        if let EffectChoiceKind::SacrificeOrDiscard { nonland } = self.kind {
            match zone {
                Zone::Hand => {
                    self.kind = EffectChoiceKind::Discard;
                    self.filter = SacrificeFilter {
                        keywords: Keywords::NONE,
                        source_only: false,
                        kinds: 0,
                        nonland,
                        other: false,
                        tokens: None,
                        artifact_enchantment_or_token: false,
                        subtype: None,
                    };
                }
                Zone::Battlefield => self.kind = EffectChoiceKind::Sacrifice,
                _ => {}
            }
        }
        self
    }
}

/// Public qualities for a non-targeted choice from one exact milling event.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum MilledFilter {
    Any,
    Kinds(u8),
    KindsOrSubtype { kinds: u8, subtype: &'static str },
    Adventure,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum MilledDestination {
    PlayPermission { next_turn: bool },
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
    ChosenX,
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
pub(crate) struct TargetActivationReduction {
    pub rule: TargetRule,
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
    /// Real nonmana variable activation cost; amount is declared before payment.
    SourceCounterCost {
        kind: CounterKind,
    },
    /// Mandatory untargeted resolution payment, then draw only if paid.
    RemoveCreatureCounterThenDraw,
    /// One simultaneous creation event, with all original recipes frozen first.
    CopyControlledNontokenCreaturesNonlegendary,
    RevealTopDrawIfArtifact,
    ExileTopPlayBoostSourceIfSubtype {
        subtype: &'static str,
        power: i32,
        toughness: i32,
    },
    /// First program header: a homogeneous optional target bound from the packet.
    UpToTargets(AmountValue),
    OfferMiracleCast(ManaCost),
    Cascade,
    CreateCopyTokenForPlayer {
        subject: Subject,
        amount: u16,
        player: PlayerSet,
    },
    PutPermanentCountersScaled {
        filters: &'static [BattlefieldFilter],
        kind: CounterKind,
        amount: AmountValue,
    },
    Paradigm {
        ability: u16,
    },
    CastParadigmCopy,
    /// Lose the frozen mana value of the originally legal target card, after movement.
    LoseLifeTargetCardManaValue(PlayerSet),
    LoseLifeTargetController(u16),
    DamageTargetController(u16),
    ExileUntilManaValueCastFree(u16),
    PrepareSource,
    ChooseLandCardName,
    Prepare {
        subject: Subject,
    },
    Unprepare {
        subject: Subject,
    },
    CreateTokenChoice(&'static [Card]),
    PlayHideawayIfCoven,
    ExileHandCardCopy,
    /// Exile the current hand and draw its frozen size, retaining exact exile results.
    ExileHandDraw,
    /// Grant ordinary paid plays for the preceding exact exile results after draws finish.
    GrantExiledResultPlay {
        next_turn: bool,
    },
    ExileTopPlayScaled {
        amount: AmountValue,
        next_turn: bool,
    },
    SearchDifferentNames(SearchSpec),
    RandomLinkedExileToBattlefield,
    PartitionLibraryPiles(u16),
    KeepOneOfEachPermanentType,
    RememberOwnSacrifice,
    RecoverAnotherPermanent,
    ReturnLinkedExileToHand,
    BlinkImmediately {
        counter: Option<(CounterKind, u64)>,
        tapped: bool,
    },
    IfCastFromNonHand {
        skip: usize,
    },
    BlinkUntilEndStep {
        under_owner: bool,
    },
    ReturnDeathCard(ReturnKind),
    ResolveDelayedReturn,
    CopyNextInstantSorcery {
        ability: u16,
    },
    ScheduleSacrificedArtifactCopy {
        ability: u16,
    },
    CreateCapturedArtifactCopy,
    CopyCastSpell {
        prior_spells: bool,
    },
    CopyStormSpell,
    ChangeSingleStackTarget,
    GrantGraveyardCast(GraveyardCastKind),
    /// Permission for the exact public card(s) just milled by this resolution.
    GrantMilledPlay,
    ExileDiscardedWithStash,
    /// Optional exact discarded-card exile; only actual success grants paid plays.
    MayExileDiscardedPlay {
        next_turn: bool,
    },
    ExileEachLibraryTopCastFree,
    CastTargetedGraveyardFree,
    CounterRelatedUnlessPayLife(u32),
    /// One captured ward payer may discard a card or pay this mana cost.
    CounterRelatedUnlessDiscardOrPay(ManaCost),
    /// One combined ward cost, paid atomically or the related item is countered.
    CounterRelatedUnlessPayManaLife {
        cost: ManaCost,
        life: u32,
    },
    CreateTemporaryArtifactCopy,
    BecomeCopy(InPlaceCopySpec),
    CreateCopyToken {
        subject: Subject,
        amount: u16,
    },
    /// Skip instructions unless this is the specified successful resolution this turn.
    IfThisAbilityResolutionNumber {
        number: u64,
        skip: usize,
    },
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
    CreateTokensReflexive {
        token: Card,
        amount: u16,
        ability: u16,
    },
    SacrificeSourceIfAttachedCounters {
        minimum: u64,
    },
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
    /// Excludes this cast, but includes an original spell when resolving an uncast copy.
    IfAnotherInstantSorceryCast {
        skip: usize,
    },
    IfManaSpent {
        minimum: Option<u64>,
        maximum: Option<u64>,
        skip: usize,
    },
    CounterThenDrawIfDiscounted,
    CounterAndDelayManaIfSubtype {
        subtype: &'static str,
        ability: u16,
    },
    /// Countering may fail; the spell's captured current controller still creates tokens.
    CounterAndCreateForController {
        token: Card,
        amount: u16,
    },
    /// Resolution-time life condition on the targeted player.
    IfTargetPlayerLife {
        life: i128,
        skip: usize,
    },
    /// The exact source permanent fights its targeted creature without targeting itself.
    FightSource,
    DiscardScaled(PlayerSet, AmountValue),
    DiscardManaValueLifeLoss,
    PutLandCards {
        maximum: AmountValue,
        tapped: bool,
    },
    /// Optional resolution payment: selecting a legal filtered hand permanent pays
    /// before any replacement changes the resulting move. Decline skips a block.
    PutHandFiltered {
        filter: SearchFilter,
        skip: usize,
    },
    PutHandCreature,
    PutHandPermanent {
        maximum: AmountValue,
        tapped: bool,
    },
    PutHandLand {
        tapped: bool,
    },
    PutHandLandWithLife {
        tapped: bool,
        subtype: &'static str,
        life: u16,
    },
    JobSelect {
        token: Card,
    },
    /// Choose and pay X during resolution, then create a separate reflexive trigger.
    PayXReflexive {
        ability: u16,
    },
    MayPayMana {
        cost: ManaCost,
        skip: usize,
    },
    MayPayManaAlternatives {
        costs: &'static [ManaCost],
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
    SacrificeSource,
    /// Queue a separate trigger only after successfully sacrificing this exact source.
    SacrificeSourceReflexive {
        ability: u16,
    },
    /// Offer a tap of the exact untapped source; queue only after the actual tap.
    PutSourceCounterReflexive {
        kind: CounterKind,
        amount: u64,
        ability: u16,
    },
    MayTapSourceReflexive {
        ability: u16,
    },
    ExileAndCopyNightmare,
    ExileOtherNightmares,
    DamageDivided {
        total: u16,
    },
    DistributeCounters {
        total: u16,
        kind: CounterKind,
    },
    DamageByGroup {
        first: i32,
        other: i32,
    },
    DamageForExcess(AmountValue),
    CreateExcessTokens(Card),
    ExileExcessPlay {
        next_turn: bool,
    },
    DamageCannotBePreventedThisTurn,
    LoseHalfLife(PlayerSet),
    CopyTargetSpell {
        group: u16,
        retarget: bool,
        exceptions: CopyExceptions,
    },
    FlipCoinsSkipTurns {
        coins: u16,
    },
    ChooseHalf {
        kind: EffectChoiceKind,
    },
    DiscardAnyDraw {
        players: PlayerSet,
        bonus: u16,
        required_all: bool,
    },
    AdditionalCombat,
    TransformSource,
    RequireBlocks(CreatureFilter),
    /// Definition header: announce X as a generic activation mana cost.
    ActivationX,
    DestroyCombatDamagedPlayersPermanents,
    CreateCopyTokenWithExceptions {
        subject: Subject,
        amount: u16,
        exceptions: CopyExceptions,
    },
    CreateCounteredCopyToken {
        subject: Subject,
        exceptions: CopyExceptions,
        kind: CounterKind,
        amount: u64,
    },
    ExileWarpedPermanent,
    Crew {
        minimum: u16,
    },
    BecomeSaddled {
        minimum: u16,
    },
    ExileTopChoosePlay {
        amount: u16,
        next_turn: bool,
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
    /// Definition-only triggered header whose modes may each be chosen once per source incarnation.
    ModalOnce(&'static [SpellMode]),
    /// Triggered modes each available once during the current actual turn.
    ModalOncePerTurn(&'static [SpellMode]),
    /// Immediately lose the game, independently of life total or state-based actions.
    LoseGame(PlayerSet),
    WinIfDistinctSubtypeNames {
        subtype: &'static str,
        minimum: u16,
    },
    EndTurn,
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
    IfTeamwork {
        skip: usize,
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
    GainControlUntilEndTurn,
    /// A fixed controller for this exact permanent, with no expiry or source dependence.
    GainControl {
        subject: Subject,
        controller: PlayerSet,
    },
    ReturnGraveyardManaValue {
        kinds: u16,
        value: AmountValue,
    },
    DestroyManaValue {
        kinds: u16,
        maximum: AmountValue,
    },
    ReturnGraveyardCreatures {
        players: PlayerSet,
        maximum_mana_value: u64,
    },
    DestroyTargetNameGroup,
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
    /// One simultaneous damage batch, excluding creatures with a live subtype.
    DamageBattlefieldExceptSubtype {
        filters: &'static [BattlefieldFilter],
        excluded_subtype: &'static str,
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
    ReturnSourceFromFirstPublicZoneToHand,
    /// Return the exact resulting graveyard card captured from this death event.
    ReturnEventCardToHand,
    PutSourceOnLibraryTop {
        origin: TargetZone,
    },
    ReturnSourceToBattlefield {
        tapped: bool,
    },
    CounterRelated,
    AddPoison(PlayerSet, u16),
    MayOpponent {
        skip: usize,
    },
    /// Offer the following payment block only when the controller can afford it.
    MayPayControllerLife {
        amount: u16,
        skip: usize,
    },
    MayPayOpponentLife {
        amount: u16,
        skip: usize,
    },
    DrawDamageAmount,
    PutDiscardCounters(CounterKind),
    DamageDiscardAmount(PlayerSet),
    DiscardThenPutLand {
        players: PlayerSet,
        amount: u16,
        tapped: bool,
    },
    DiscardHand(PlayerSet),
    TapCreatures(CreatureFilter),
    Mill(PlayerSet, MillAmount),
    MayMill(u16),
    RecoverMilled(RecoveryFilter),
    CollectGraveyard,
    RevealHandDiscard {
        excluded_kinds: u16,
    },
    DestroyLoseManaValue,
    SelectMilled(MilledSelectionSpec),
    AttachSource,
    AttachEquipmentByGroups {
        equipment: u16,
        recipient: u16,
    },
    TapAttached,
    PutGrantorCounter(CounterKind),
    PutCombatDealerCounters,
    RemoveSourceCounterIfAny {
        kind: CounterKind,
        skip: usize,
    },
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
    ScaledPowerDamage {
        source: u16,
        recipient: u16,
        factor: u16,
    },
    AddMana(ManaProduction),
    AddColorlessScaled(AmountValue),
    AddManaScaled {
        color: ChoiceColor,
        amount: AmountValue,
    },
    AddCombatRed(u16),
    AddCombatRedScaled(AmountValue),
    Scry(u16),
    Surveil(u16),
    AnimateSource(AnimationSpec),
    AnimateTargets(AnimationSpec),
    /// Change creature subtypes, base stats and keywords without adding a card type.
    ChangeSourceCreatureForm(AnimationSpec),
    SetCreatureForm {
        types: &'static [&'static str],
        colors: u8,
        base: AmountValue,
    },
    MoveCountersFromSource {
        kind: CounterKind,
        amount: AmountValue,
    },
    AnimateSourceIfNoncreature {
        spec: AnimationSpec,
        ability: u16,
    },
    Search(SearchSpec),
    SearchTargetController(SearchSpec),
    /// The legal player target in the currently selected target group searches.
    SearchTargetPlayer(SearchSpec),
    MayTargetController {
        skip: usize,
    },
    ExileAndGainLifeEqualPower,
    /// Capture the destroyed land's controller, then offer both optional basic searches.
    DestroyLandAndSearch,
    /// Unqualified search: find the maximum, or every remaining card if fewer.
    /// Existing qualified searches retain their legal failure-to-find behavior.
    SearchRequired(SearchSpec),
    SelectLibrary(LibrarySelectionSpec),
    SelectLibraryTo(LibrarySelectionEffect),
    /// Enter first; optionally transform the exact actual entrant before the remainder.
    SelectLibraryThenMayTransform(LibrarySelectionEffect),
    NoMaximumHandSize,
    ExileResolvingSpell,
    UntapLands(u16),
    IfAmountAtLeast {
        value: AmountValue,
        minimum: u128,
        skip: usize,
    },
    RevealUntilLandToGraveyard(PlayerSet),
    Damage(i32),
    DamageScaled(AmountValue),
    DamagePlayers(PlayerSet, i32),
    /// A bound untargeted event object, independently of the ability source.
    EventObjectDamagePlayers(PlayerSet, i32),
    DamagePlayersScaled(PlayerSet, AmountValue),
    CreateTokensFor {
        players: PlayerSet,
        token: Card,
        amount: u16,
        tapped: bool,
    },
    UntapSource,
    /// Untap the exact non-targeted object captured by an event packet.
    UntapEventObject,
    TapSource,
    UntapCreatures(CreatureFilter),
    UntapPlayerCreatures(PlayerSet),
    PutPlayerCreatureCounters {
        players: PlayerSet,
        kind: CounterKind,
        amount: u32,
    },
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
    /// Replace creature types until the affected permanent changes zones.
    #[allow(
        dead_code,
        reason = "Core instruction is scenario-tested while complete-card consumers remain gated"
    )]
    /// A permanent, exact-incarnation evolution, conditional at resolution.
    EvolveSource {
        required: &'static str,
        types: &'static [&'static str],
        stats: Option<Stats>,
        keywords: Keywords,
        ability: Option<u16>,
    },
    SetCreatureTypes {
        subject: Subject,
        types: &'static [&'static str],
    },
    SacrificeAbilitySource,
    ExileSourceGainLifeIfMoved(u16),
    GrantCreatureAbility {
        filter: CreatureFilter,
        ability: u16,
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
    /// Freeze a non-targeted permanent set and one scalar at resolution.
    GrowPermanentGroupScaled {
        filters: &'static [BattlefieldFilter],
        exclude_source: bool,
        amount: AmountValue,
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
    ExileTargetsForModifiedCast,
    ExileTargetsPlay {
        next_turn: bool,
    },
    ExileTargetsAndTopPlayForOwners,
    ExileUntilSourceLeaves,
    ExileGraveyard(PlayerSet),
    ReturnToHand,
    /// Move all legal target groups together, deduplicating exact incarnations.
    ReturnAllTargetsToHand,
    /// Capture a legal graveyard target's subtype, return it, then draw for a match.
    ReturnToHandDrawForSubtype(&'static str),
    BottomLibrary,
    TopLibrary,
    AdditionalLandPlays(u64),
    OwnerLibrary,
    OwnerLibrarySecondFromTop,
    ChooseManaValueParityDestroyOthers,
    ShuffleTargetIntoLibrary,
    ShuffleGraveyardIntoLibrary,
    DoubleTargetPower,
    DrawAndProtectCounterCreatures,
    MultiplyTargetStats {
        factor: u8,
    },
    ReturnToBattlefield,
    ReturnToBattlefieldUnderControl,
    ReturnToBattlefieldTapped,
    ReturnTargetAttackingWithCounter(CounterKind),
    Destroy,
    Tap,
    Untap,
    /// One simultaneous placement on battlefield permanents matching the count predicate.
    PutFilteredCounters {
        filter: CountFilter,
        kind: CounterKind,
        amount: u16,
    },
    PutCreatureCounters {
        filter: CreatureFilter,
        kind: CounterKind,
        amount: u32,
    },
    DoubleCounters,
    DoubleCounterKind {
        subject: Subject,
        kind: CounterKind,
    },
    ExileGraveTargets,
    IfExiledCreature {
        skip: usize,
    },
    RemoveAllCounters,
    RemoveAllCountersFrom(Subject),
    SnapshotDoubleCreatures(CreatureFilter),
    PutCounters {
        subject: Subject,
        kind: CounterKind,
        amount: u32,
    },
    /// One simultaneous placement may contain multiple counter kinds.
    PutCountersMixed {
        subject: Subject,
        amounts: &'static [(CounterKind, u32)],
    },
    CreateTokensWithTemporaryKeywords {
        token: Card,
        amount: AmountValue,
        keywords: Keywords,
    },
    CreateTokensScaled {
        token: Card,
        amount: AmountValue,
        tapped: bool,
    },
    /// Create a fixed token batch, then put scaled counters on those exact tokens.
    /// Copy every supported counter kind from this exact source/LKI to new tokens.
    CreateTokensWithSourceCounters {
        token: Card,
        amount: u16,
    },
    PutSourceCountersOnTargets,
    CreateTokensWithCounters {
        token: Card,
        amount: u16,
        kind: CounterKind,
        counters: AmountValue,
    },
    /// Create first, then attach every exact resulting Equipment to this source.
    CreateEquipmentForSource {
        token: Card,
        amount: u16,
    },
    CreateTokens {
        token: Card,
        amount: u16,
        tapped: bool,
    },
    DrawPowerOfTwoChosenX(PlayerSet),
    Draw(PlayerSet, u16),
    DrawScaled(PlayerSet, AmountValue),
    AddCastEntryCounters {
        kind: CounterKind,
        amount: AmountValue,
    },
    DrawThenDiscardIfDrew {
        amount: AmountValue,
        discard: u16,
    },
    Discard(PlayerSet, u16),
    /// Optional resolution choice; its accepted branch records an exact ability use.
    MayOncePerTurn {
        skip: usize,
    },
    RecordOncePerTurn,
    MemoryVessel,
    ExileAndBorrowActivatedAbilities,
    GainLife(PlayerSet, u16),
    GainLifeScaled(PlayerSet, AmountValue),
    LoseLife(PlayerSet, u16),
    LoseLifeScaled(PlayerSet, AmountValue),
    /// Each opponent loses the frozen amount; the controller gains it once.
    DrainLifeScaled(AmountValue),
    /// Opponent loss and controller gain use the same frozen quantity.
    LoseOpponentGainSame(AmountValue),
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
    OtherColoredCreaturesYouControl(ChoiceColor),
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
    ControllerHandDeficit(u16),
    /// Current hand size of the spell's surviving targeted player.
    TargetPlayerHandSize,
    Sum(&'static [AmountValue]),
    ManaSpent,
    ColorsSpent,
    SpellColors,
    CastSpellManaValue,
    CastSpellManaSymbols(ChoiceColor),
    SourcePower,
    SourceToughness,
    SourceEquipmentCount,
    #[allow(
        dead_code,
        reason = "Core quantity is scenario-tested while its consumer cards remain gated"
    )]
    DamageEventAmount,
    DamageReceivedAmount,
    #[allow(
        dead_code,
        reason = "Core quantity is scenario-tested while its consumer cards remain gated"
    )]
    SourceCounterQuantity(CounterKind),
    /// Frozen successful source-counter cost; never the source current counter count.
    PaidSourceCounterQuantity(CounterKind),
    SacrificedToughness,
    Fixed(u16),
    /// Explicit immutable parameter of the resolving spell or its exact ETB packet.
    ChosenX,
    MilledCreatures,
    Stat(StatValue),
    LifeGainedThisTurn,
    LifeEventAmount,
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
pub(crate) enum SourceCharacteristicFilter {
    Color(ChoiceColor),
    Instant,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum StatValue {
    ControlledCreatureDiedThisTurn,
    CardLeftGraveyardThisTurn,
    Fixed(u16),
    TappedCount(CountFilter),
    Not(&'static StatValue),
    CardsDrawnThisTurn,
    ControllerHandSize,
    NonlandEntriesThisTurn,
    SourceAttacking,
    AttackingCreatures,
    AttackingSubtype(&'static str),
    OtherNamedCreatures(&'static str),
    /// Unqualified all-player count; distinct from controller-owned attackers.
    AllAttackingCreatures,
    SourceModified,
    SourceEquipmentCount,
    Tokens(CountFilter),
    AtLeast {
        value: &'static StatValue,
        minimum: u16,
    },
    SpellsCastThisTurn,
    LifeGainOccurredThisTurn,
    CreaturesDiedThisTurn,
    LifeGainedThisTurn,
    ControllerLife,
    /// Boolean threshold relative to the rules configuration, not a printed 20.
    ControllerLifeAboveStarting {
        minimum: u16,
    },
    /// Boolean presence, rather than number of distinct colors.
    HasColor {
        filter: CountFilter,
        color: ChoiceColor,
    },
    Count(CountFilter),
    /// Every kind bit must occur on one matching object; ordinary Count is a union.
    CountAllKinds(CountFilter),
    NamedCount {
        filter: CountFilter,
        name: &'static str,
    },
    GreatestManaValue(CountFilter),
    DistinctNames(CountFilter),
    DistinctManaValues(CountFilter),
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
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn matches_characteristics(
        self,
        card: Card,
        token: bool,
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
            && self.subtype.is_none_or(|subtype| added.has(card, subtype))
            && (!self.attacking || attacking)
            && self.tokens.is_none_or(|tokens| token == tokens)
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
        token: bool,
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
            && self.filter.matches_characteristics(card, token, kinds, added, yours, same, attacking)
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
    /// Zero means any permanent, including Battles outside the narrow mask.
    pub kinds: u8,
    pub relation: i8,
    pub bonus: EquipmentSpec,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Subject {
    /// Exact triggering event object, including entrants and declared attackers.
    Entering,
    Source,
    Targets,
    Attached,
}

/// Immutable +1/+1 counter replacements evaluated against the pre-entry world.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EntryCounterAmount {
    GreatestOtherCreaturePower,
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
    /// Add one of each positive kind, only for the provider controller placing
    /// counters on their own permanent. General providers do not apply entering.
    CountersPlacedByController,
    DoubleCreatureDamage,
    DoubleOpponentDamage,
    PreventOtherCreatureNoncombatDamage,
    PreventAllDamageToSelf,
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
    AddArtifactToken(Card),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum CounterKind {
    Lore,
    PlusOne,
    MinusOne,
    Stun,
    Loyalty,
    Incubation,
    Bait,
    Page,
    Divinity,
    Revival,
    Soul,
    Fellowship,
    Wish,
    Stash,
    Charge,
    Growth,
    Plan,
    Invasion,
    Haste,
    Flying,
    FirstStrike,
    DoubleStrike,
    Deathtouch,
    Indestructible,
    Lifelink,
    Menace,
    Reach,
    Trample,
    Vigilance,
    Finality,
}
impl CounterKind {
    /// Stable allocation order for mixed counter costs; existing kinds stay first.
    pub const ALL: [Self; 30] = [
        Self::PlusOne,
        Self::MinusOne,
        Self::Stun,
        Self::Loyalty,
        Self::Lore,
        Self::Incubation,
        Self::Bait,
        Self::Page,
        Self::Divinity,
        Self::Revival,
        Self::Soul,
        Self::Fellowship,
        Self::Wish,
        Self::Stash,
        Self::Charge,
        Self::Growth,
        Self::Plan,
        Self::Invasion,
        Self::Haste,
        Self::Flying,
        Self::FirstStrike,
        Self::DoubleStrike,
        Self::Deathtouch,
        Self::Indestructible,
        Self::Lifelink,
        Self::Menace,
        Self::Reach,
        Self::Trample,
        Self::Vigilance,
        Self::Finality,
    ];
    pub(crate) fn is_named(self) -> bool {
        matches!(
            self,
            Self::Incubation
                | Self::Bait
                | Self::Page
                | Self::Divinity
                | Self::Revival
                | Self::Soul
                | Self::Fellowship
                | Self::Wish
                | Self::Stash
                | Self::Charge
                | Self::Growth
                | Self::Plan
                | Self::Invasion
                | Self::Haste
                | Self::Flying
                | Self::FirstStrike
                | Self::DoubleStrike
                | Self::Deathtouch
                | Self::Indestructible
                | Self::Lifelink
                | Self::Menace
                | Self::Reach
                | Self::Trample
                | Self::Vigilance
                | Self::Finality
        )
    }
}

/// Common live fields and catalogue initializers remain compact Copy values.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub(crate) struct InitialCounters {
    #[serde(default, skip_serializing_if = "is_zero_counter")]
    pub lore: u64,
    pub plus_one: u64,
    pub minus_one: u64,
    pub stun: u64,
    #[serde(default, skip_serializing_if = "is_zero_counter")]
    pub loyalty: u64,
}

/// Cold counter storage is allocated only when present. Sorting and removing
/// zero entries make equality, hashing, serialization and branching canonical.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub(crate) struct NamedCounters(pub(crate) Vec<(CounterKind, u64)>);
impl<'de> Deserialize<'de> for NamedCounters {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let mut values = Vec::<(CounterKind, u64)>::deserialize(deserializer)?;
        values.sort_unstable_by_key(|(kind, _)| *kind);
        if values.is_empty()
            || values.iter().any(|(kind, n)| !kind.is_named() || *n == 0)
            || values.windows(2).any(|pair| pair[0].0 == pair[1].0)
        {
            return Err(serde::de::Error::custom(
                "named counters must be distinct, nonzero named kinds",
            ));
        }
        values.shrink_to_fit();
        Ok(Self(values))
    }
}

/// Owned public counter values and payments. Live objects keep their hot fields
/// separately from the game state's optional copy-on-write named-counter rows.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Counters {
    #[serde(default, skip_serializing_if = "is_zero_counter")]
    pub lore: u64,
    pub plus_one: u64,
    pub minus_one: u64,
    pub stun: u64,
    #[serde(default, skip_serializing_if = "is_zero_counter")]
    pub loyalty: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) named: Option<Box<NamedCounters>>,
}
fn is_zero_counter(value: &u64) -> bool {
    *value == 0
}
impl From<InitialCounters> for Counters {
    fn from(value: InitialCounters) -> Self {
        Self {
            lore: value.lore,
            plus_one: value.plus_one,
            minus_one: value.minus_one,
            stun: value.stun,
            loyalty: value.loyalty,
            named: None,
        }
    }
}
/// Allocation-free amounts shared by actual and speculative rules queries.
#[derive(Clone, Copy, Default)]
pub(crate) struct CounterAmounts<'a> {
    pub hot: InitialCounters,
    pub named: Option<&'a NamedCounters>,
}
impl<'a> From<&'a Counters> for CounterAmounts<'a> {
    fn from(value: &'a Counters) -> Self {
        Self {
            hot: value.hot(),
            named: value.named.as_deref(),
        }
    }
}
impl CounterAmounts<'_> {
    pub(crate) fn get(self, kind: CounterKind) -> u64 {
        match kind {
            CounterKind::PlusOne => self.hot.plus_one,
            CounterKind::MinusOne => self.hot.minus_one,
            CounterKind::Stun => self.hot.stun,
            CounterKind::Loyalty => self.hot.loyalty,
            CounterKind::Lore => self.hot.lore,
            _ => self
                .named
                .and_then(|v| {
                    v.0.binary_search_by_key(&kind, |(k, _)| *k)
                        .ok()
                        .map(|i| v.0[i].1)
                })
                .unwrap_or(0),
        }
    }
    pub(crate) fn total(self) -> u128 {
        u128::from(self.hot.plus_one)
            + u128::from(self.hot.minus_one)
            + u128::from(self.hot.stun)
            + u128::from(self.hot.loyalty)
            + u128::from(self.hot.lore)
            + self
                .named
                .map_or(0, |v| v.0.iter().map(|(_, n)| u128::from(*n)).sum())
    }
    pub(crate) fn has_any(self) -> bool {
        (self.hot.plus_one | self.hot.minus_one | self.hot.stun | self.hot.loyalty | self.hot.lore)
            != 0
            || self.named.is_some()
    }
    pub(crate) fn available(self, kind: Option<CounterKind>) -> u128 {
        kind.map_or_else(|| self.total(), |kind| u128::from(self.get(kind)))
    }
    pub(crate) fn contains(self, payment: &Counters) -> bool {
        CounterKind::ALL
            .into_iter()
            .all(|kind| self.get(kind) >= payment.get(kind))
    }
    pub(crate) fn to_owned(self) -> Counters {
        let mut result: Counters = self.hot.into();
        result.named = self.named.map(|v| Box::new(v.clone()));
        result
    }
}
impl Counters {
    pub(crate) fn hot(&self) -> InitialCounters {
        InitialCounters {
            lore: self.lore,
            plus_one: self.plus_one,
            minus_one: self.minus_one,
            stun: self.stun,
            loyalty: self.loyalty,
        }
    }

    pub fn get(&self, kind: CounterKind) -> u64 {
        match kind {
            CounterKind::PlusOne => self.plus_one,
            CounterKind::MinusOne => self.minus_one,
            CounterKind::Stun => self.stun,
            CounterKind::Loyalty => self.loyalty,
            CounterKind::Lore => self.lore,
            _ => self
                .named
                .as_ref()
                .and_then(|v| {
                    v.0.binary_search_by_key(&kind, |(k, _)| *k)
                        .ok()
                        .map(|i| v.0[i].1)
                })
                .unwrap_or(0),
        }
    }
    /// Canonical named storage is nonempty whenever it exists. Presence queries
    /// need neither wide quantity arithmetic nor a scan of cold counter rows.
    #[inline]
    pub fn is_empty(&self) -> bool {
        (self.plus_one | self.minus_one | self.stun | self.loyalty | self.lore) == 0
            && self.named.is_none()
    }
    #[inline]
    pub fn has_any(&self) -> bool {
        !self.is_empty()
    }
    pub fn total(&self) -> u128 {
        u128::from(self.plus_one)
            + u128::from(self.minus_one)
            + u128::from(self.stun)
            + u128::from(self.loyalty)
            + u128::from(self.lore)
            + self
                .named
                .as_ref()
                .map_or(0, |v| v.0.iter().map(|(_, n)| u128::from(*n)).sum())
    }
    /// Construct an action's typed counter payment, including named counters.
    pub fn of(kind: CounterKind, amount: u64) -> Self {
        Self::default().with(kind, amount)
    }
    /// Set one kind's count while constructing an owned mixed-kind payment.
    /// A zero count removes that kind from sparse storage.
    pub fn with(mut self, kind: CounterKind, amount: u64) -> Self {
        self.set(kind, amount);
        self
    }
    pub(crate) fn set(&mut self, kind: CounterKind, amount: u64) {
        match kind {
            CounterKind::PlusOne => self.plus_one = amount,
            CounterKind::MinusOne => self.minus_one = amount,
            CounterKind::Stun => self.stun = amount,
            CounterKind::Loyalty => self.loyalty = amount,
            CounterKind::Lore => self.lore = amount,
            _ => {
                if amount == 0 && self.named.is_none() {
                    return;
                }
                let values = &mut self
                    .named
                    .get_or_insert_with(|| Box::new(NamedCounters(Vec::with_capacity(1))))
                    .0;
                match values.binary_search_by_key(&kind, |(k, _)| *k) {
                    Ok(i) if amount == 0 => {
                        values.remove(i);
                    }
                    Ok(i) => values[i].1 = amount,
                    Err(i) if amount > 0 => values.insert(i, (kind, amount)),
                    Err(_) => {}
                }
                if values.is_empty() {
                    self.named = None;
                }
            }
        }
    }
    pub(crate) fn add(&mut self, kind: CounterKind, amount: u64) {
        if amount > 0 {
            self.set(
                kind,
                self.get(kind)
                    .checked_add(amount)
                    .capacity("counter representation exhausted"),
            );
        }
    }
    pub(crate) fn remove(&mut self, kind: CounterKind, amount: u64) {
        if amount > 0 {
            self.set(
                kind,
                self.get(kind)
                    .checked_sub(amount)
                    .expect("validated counter payment"),
            );
        }
    }
    pub(crate) fn owned_bytes(&self) -> usize {
        self.named.as_ref().map_or(0, |v| {
            std::mem::size_of::<NamedCounters>()
                + v.0.len() * std::mem::size_of::<(CounterKind, u64)>()
        })
    }
    pub(crate) fn pay(&mut self, payment: &Self) {
        self.plus_one -= payment.plus_one;
        self.minus_one -= payment.minus_one;
        self.stun -= payment.stun;
        self.loyalty -= payment.loyalty;
        self.lore -= payment.lore;
        if let Some(values) = &payment.named {
            for (kind, n) in &values.0 {
                self.remove(*kind, *n);
            }
        }
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
    NotOwnedByYou,
    HasX,
    SourceChosenColor,
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
    Colors {
        filter: &'static CastFilter,
        mask: u8,
    },
    ManaSymbols {
        filter: &'static CastFilter,
        color: ChoiceColor,
    },
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
            Self::HasX => card.x_symbols() > 0,
            Self::PermanentAdventure => {
                card.adventure_spell().is_some()
                    && card.kind_mask() & (1 | 2 | 16 | 32 | 64 | 256) != 0
            }
            // Contextual cast observers use Game::cast_filter_matches instead.
            Self::NotOwnedByYou
            | Self::SourceChosenColor
            | Self::Targeted { .. }
            | Self::InstantSorceryOrdinal(_) => false,
            Self::Multicolored => card.colors().count_ones() >= 2,
            Self::Colors { filter, mask } => {
                card.colors() & mask != 0 && filter.matches_mana_value(card, mana_value)
            }
            Self::ManaSymbols { filter, color } => {
                card.mana_symbols_of_color(color) > 0 && filter.matches_mana_value(card, mana_value)
            }
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
    /// Trigger-time attachment relation; wrap the step event in When.
    AttachedControllerTurn,
    ArtifactEnteredThisTurn,
    HeroEnteredOrAttackedThisTurn,
    CardExiledThisTurn,
    OpponentHasMoreCardsInHand,
    OpponentControlsMoreCreatures,
    SourceReceivedCounterThisTurn,
    YourTurn,
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
    ControlledNoncombatSources,
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
    CreatureWithoutSubtype(&'static str),
    ArtifactManaValueAtLeast(u32),
    CreaturePowerAtMost(i32),
    CreaturePowerAtLeast(i32),
    SubtypeAny(&'static [&'static str]),
    KindOrSubtype { kinds: u16, subtype: &'static str },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ReturnDeathFilter {
    SourceNonDemon,
    SourceRevival,
    ControlledNontokenNonAngel,
    ControlledSubtype(&'static str),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ReturnKind {
    Demon,
    Angel,
    Revival,
    HeroFinality,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TriggerCondition {
    /// Pre-event observer and other permanent, with an actual owner-hand result.
    OtherControlledNonlandReturnedToOwnersHand,
    /// Global target acquisition, not a cast-completion or self-target event.
    ControlledAbilityTargetsPlayerOrPermanent,
    /// The physical source spell itself was cast; stack copies never dispatch it.
    SelfCast,
    CardsLeaveGraveyard,
    ReturnDeath(ReturnDeathFilter),
    /// Delayed ability carried by a produced mana unit, independent of its source.
    ManaSpentForSpell,
    /// Explicit printed source zone; legacy events keep their existing origins.
    InZone {
        zone: TargetZone,
        event: &'static TriggerCondition,
    },
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
    SelfPaysTeamwork,
    /// A paid promise belongs to this actual cast-to-battlefield entry event.
    SelfGiftEnters,
    SelfEntersIfCast,
    Foraged,
    FoodsSacrificed,
    Reflexive,
    /// A cast-event qualification, not an intervening-if resolution guard.
    DuringOpponentsTurn(&'static TriggerCondition),
    OncePerTurn(&'static TriggerCondition),
    /// Qualification checked only when the event occurs.
    When {
        guard: StepGuard,
        event: &'static TriggerCondition,
    },
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
    SourceDealtCombatDamage,
    AttachedDealtCombatDamage,
    ControlledCreatureCombatDamageOwnTurn,
    DamagedCreatureDies,
    /// Historical placement event: before < threshold <= after. Never a live recheck.
    CounterThresholdCrossed {
        kind: CounterKind,
        threshold: u64,
    },
    StateCounterAtLeast {
        kind: CounterKind,
        minimum: u64,
    },
    /// One occurrence per damaged creature in a simultaneous actual damage batch.
    /// Once for actual positive damage to this exact source in a completed batch.
    SelfDamaged,
    CreatureDamaged {
        relation: CastRelation,
    },
    PlayerDamaged {
        subject: DamageSubject,
        combat_only: bool,
        opponent_only: bool,
        one_or_more: bool,
    },
    LifeLost {
        relation: CastRelation,
    },
    /// The placing player controls this observer; the other creature may belong to either player.
    OtherCreatureCounterPlacedByYou {
        kind: CounterKind,
    },
    /// One event per battlefield recipient; the placing player controls this observer.
    CounterPlacedByYou {
        kind: CounterKind,
        filter: CountFilter,
    },
    /// One event for the whole simultaneous placement, across all recipients and kinds.
    CounterBatchPlacedByYou {
        kind: CounterKind,
        filter: CountFilter,
    },
    OtherCreatureCountersAdded {
        kind: CounterKind,
        excluded_subtype: &'static str,
    },
    SelfCountersAdded {
        kind: CounterKind,
        controller_only: bool,
    },
    LifeGained {
        first_only: bool,
        own_turn: bool,
    },
    AttachedCreatureDies,
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
    CreatureDiesWithMaximumStat {
        relation: CastRelation,
        maximum: i32,
    },
    CreatureDiesFiltered {
        filter: CreatureFilter,
    },
    /// Match the departing creature's pre-event combat membership.
    CreatureDiesAttackState {
        filter: CreatureFilter,
        attacking: bool,
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
    /// Historical departure condition; queued only if the frozen source had counters.
    SelfDiesWithCounters,
    SelfDies,
    SelfNontokenDies,
    SelfLeaves,
    SelfEntersOrDies,
    SelfAttacks,
    /// Intervening equipped clause, with exact source-departure quantities.
    SelfAttacksIfEquipped,
    SelfFirstAttack,
    CreatureAttacksYou {
        colors: u8,
    },
    /// Declaration-time condition; not a resolution-time intervening if.
    SelfAttacksAlone,
    /// A completed declaration with exactly one controlled creature, whether this source attacked or not.
    ControlledCreatureAttacksAlone,
    ControlledEquippedCreatureAttacks,
    EquippedAttacks,
    /// Attachment recipient was the sole creature declared in this attack event.
    EquippedAttacksAlone,
    /// One event per player attacked by at least one declared matching creature.
    AttackPlayer(CreatureFilter),
    GrantedAttacks,
    SelfBecomesTapped,
    /// One event after the complete simultaneous tap group, using post-event types.
    ControlledCreaturesBecomeTapped,
    /// Per-creature own-turn tap, with a first-tap intervening condition.
    ControlledCreatureFirstTapOwnTurn,
    AttachedBecomesUntapped,
    AttachedCreatureBecomesTapped,
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
    FilteredEntersYou(EntryPredicate),
    TokensEnterYou,
    OtherCreatureEntersYou,
    OtherCreatureEntersYouIfGreaterStats,
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
    pub(crate) fn origin(self) -> Option<TargetZone> {
        match self {
            Self::InZone { zone, .. } => Some(zone),
            Self::Granted(event)
            | Self::OncePerTurn(event)
            | Self::DuringOpponentsTurn(event)
            | Self::ManaSpent { event, .. }
            | Self::When { event, .. }
            | Self::InterveningIf { event, .. } => event.origin(),
            _ => None,
        }
    }
    pub(crate) fn mana_spent_rule(self) -> Option<(u64, bool)> {
        match self {
            Self::ManaSpent {
                minimum, increment, ..
            } => Some((minimum, increment)),
            Self::InZone { event: e, .. }
            | Self::Granted(e)
            | Self::OncePerTurn(e)
            | Self::DuringOpponentsTurn(e)
            | Self::When { event: e, .. }
            | Self::InterveningIf { event: e, .. } => e.mana_spent_rule(),
            _ => None,
        }
    }

    pub(crate) fn is_granted(self) -> bool {
        match self {
            Self::Granted(_) | Self::GrantedAttacks => true,
            Self::InZone { event: e, .. }
            | Self::OncePerTurn(e)
            | Self::DuringOpponentsTurn(e)
            | Self::When { event: e, .. }
            | Self::InterveningIf { event: e, .. }
            | Self::ManaSpent { event: e, .. } => e.is_granted(),
            _ => false,
        }
    }
    pub(crate) fn requires_opponents_turn(self) -> bool {
        match self {
            Self::DuringOpponentsTurn(_) => true,
            Self::InZone { event, .. }
            | Self::Granted(event)
            | Self::OncePerTurn(event)
            | Self::ManaSpent { event, .. }
            | Self::When { event, .. }
            | Self::InterveningIf { event, .. } => event.requires_opponents_turn(),
            _ => false,
        }
    }
    pub(crate) fn event(self) -> Self {
        match self {
            Self::SelfDiesWithCounters => Self::SelfDies,
            Self::InZone { event, .. }
            | Self::Granted(event)
            | Self::OncePerTurn(event)
            | Self::DuringOpponentsTurn(event)
            | Self::When { event, .. }
            | Self::InterveningIf { event, .. }
            | Self::ManaSpent { event, .. } => event.event(),
            event => event,
        }
    }
    pub(crate) fn once_per_turn(self) -> bool {
        match self {
            Self::OncePerTurn(_) => true,
            Self::InZone { event, .. }
            | Self::Granted(event)
            | Self::When { event, .. }
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
    CastInstantSorceryThisTurn,
    Always,
    OwnTurn,
    ControlSubtypeAny(&'static [&'static str]),
    ControlLandsAtLeast(u16),
    ControlCreatureToughnessAtLeast(u16),
    SourceEnteredOrControlsBasicLand,
    HandSizeAtLeast(u16),
    GraveyardCardsAtLeast { kind: u16, minimum: u16 },
    LifeAboveStarting(u16),
}
impl ActivationGuard {
    pub(crate) fn subtypes(self) -> &'static [&'static str] {
        match self {
            Self::Always
            | Self::CastInstantSorceryThisTurn
            | Self::OwnTurn
            | Self::ControlLandsAtLeast(_)
            | Self::ControlCreatureToughnessAtLeast(_)
            | Self::SourceEnteredOrControlsBasicLand
            | Self::HandSizeAtLeast(_)
            | Self::GraveyardCardsAtLeast { .. }
            | Self::LifeAboveStarting(_) => &[],
            Self::ControlSubtypeAny(types) => types,
        }
    }
}
/// Optional creature-tap cost and immutable choices unlocked by paying it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TeamworkSpec {
    pub minimum: u16,
    pub paid_targets: Option<TargetPlan>,
    /// Bitsets over the predeclared modal combinations; ignored for nonmodal spells.
    pub normal_modes: u64,
    pub paid_modes: u64,
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
    pub(crate) fn source_counter_cost(&self) -> Option<CounterKind> {
        match self.program.first() {
            Some(Instruction::SourceCounterCost { kind }) => {
                // Compact domains reserve the source tap for this payment; it
                // cannot also consume some chosen counters as a mana producer.
                assert!(
                    self.origin == ActivationOrigin::Battlefield
                        && self.cost.tap
                        && self.cost.remove.is_none(),
                    "variable counter cost requires its own tap cost and no fixed removal"
                );
                Some(*kind)
            }
            _ => None,
        }
    }
    pub(crate) fn mana_production(&self) -> Option<ManaProduction> {
        if self.cost.loyalty.is_some() {
            return None;
        }
        match self.program {
            [Instruction::AddMana(production)]
            | [
                Instruction::AddMana(production),
                Instruction::Reflexive { .. },
            ] if !self.targets.has_targets() => Some(*production),
            _ => None,
        }
    }
}

/// Immutable zone replacement behavior; each of these replaces a graveyard event.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ZoneReplacementRule {
    RevealShuffleSelf,
    ExileInstantSorceryCards,
    ExileAllCardsAndTokens,
    OpponentDiscardToBattlefield,
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
    pub const PREVENT_COMBAT_TO: Self = Self(16777216);
    pub const PREVENT_COMBAT_BY: Self = Self(33554432);
    pub const NO_HUMAN_BLOCKERS: Self = Self(67108864);
    pub const PROTECTION_EVERYTHING: Self = Self(134217728);
    pub const PROTECTION_INSTANT_SORCERY: Self = Self(268435456);
    /// A resolved blocking rule, not an ability granted to its recipient.
    pub const ONLY_HASTE_BLOCKERS: Self = Self(536870912);
    pub fn contains(self, keyword: Self) -> bool {
        self.0 & keyword.0 == keyword.0
    }
    pub const fn intersects(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
    /// Existing resolving instructions use these bits for rule restrictions,
    /// never for a quoted ability grant. Static/printed uses stay removable.
    pub(crate) const fn resolution_rules(self) -> Self {
        Self(
            self.0
                & (2048
                    | 16384
                    | 65536
                    | 131072
                    | 262144
                    | 2097152
                    | 4194304
                    | 8388608
                    | 16777216
                    | 33554432
                    | Self::ONLY_HASTE_BLOCKERS.0),
        )
    }
    pub(crate) const fn without(self, other: Self) -> Self {
        Self(self.0 & !other.0)
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
    ChosenXAtMost(u32),
}

#[derive(Debug)]
struct Definition {
    info: CardInfo,
    effect: Effect,
    triggers: &'static [TriggeredAbility],
    activated: &'static [ActivatedAbility],
    enters: InitialCounters,
    entry: EntryTapped,
    kind: u16,
    basic: bool,
    mana_color: Option<usize>,
    colored: Mana,
    generic: u32,
}

#[path = "card_definitions/mod.rs"]
mod authored;

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
        if let Some(identity) = generated::physical_identity(self) {
            return identity;
        }
        if let Some(identity) =
            authored::rules(self).and_then(|rules| rules.extras.prepared_identity)
        {
            return identity;
        }
        generated::adventure_identity(self).unwrap_or_else(|| {
            self.token_info()
                .map_or(self.name(), |token| token.factory_identity)
        })
    }
    /// Actual rules name, shared as a regular string across all games.
    /// Token recipes may share a name; use `identity()` for stable lookup.
    pub fn name(self) -> &'static str {
        self.token_info()
            .map_or(self.info().name, |token| token.name)
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
        generated::token_info(self)
    }
    pub fn emblem_info(self) -> Option<&'static EmblemInfo> {
        generated::emblem_info(self)
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
    pub fn back_face(self) -> Option<Card> {
        generated::back_face(self)
    }
    pub fn face_parent(self) -> Option<Card> {
        generated::face_parent(self)
    }
    pub fn front_face(self) -> Card {
        self.face_parent().unwrap_or(self)
    }
    pub(crate) fn other_face(self) -> Option<Card> {
        self.back_face().or_else(|| self.face_parent())
    }
    pub fn adventure_spell(self) -> Option<Card> {
        generated::adventure_spell(self)
    }
    /// Homogeneous optional target group whose maximum is the chosen X.
    pub(crate) fn life_x_cost(self) -> bool {
        authored::rules(self).is_some_and(|r| r.extras.life_x_cost)
    }
    pub(crate) fn targets_up_to_x(self) -> bool {
        authored::rules(self).is_some_and(|rules| rules.extras.targets_up_to_x)
    }
    pub(crate) fn spell_cannot_be_copied(self) -> bool {
        authored::rules(self).is_some_and(|r| r.extras.spell_cannot_be_copied)
    }
    pub(crate) fn prepared_spell(self) -> Option<Card> {
        authored::rules(self).and_then(|rules| rules.extras.prepared_spell)
    }
    pub(crate) fn x_complement_entry_counters(self) -> Option<(CounterKind, u32)> {
        authored::rules(self).and_then(|rules| rules.extras.x_complement_entry_counters)
    }
    pub(crate) fn enters_prepared(self) -> bool {
        authored::rules(self).is_some_and(|rules| rules.extras.enters_prepared)
    }
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
    pub(crate) fn mana_additional_alternative(self) -> Option<ManaCost> {
        let r = authored::rules(self)?;
        if let Some(extra) = r.extras.additional_mana_alternative {
            return Some(extra);
        }
        if !r.extras.mana_alternative_replaces_additional {
            return None;
        }
        let alternative = *r.extras.cost_alternatives.first()?;
        let (colored, generic) = self.cost();
        Some(ManaCost {
            colored: std::array::from_fn(|i| {
                alternative.colored[i]
                    .checked_sub(colored[i])
                    .expect("additional mana branch")
            }),
            generic: alternative
                .generic
                .checked_sub(generic)
                .expect("additional generic branch"),
        })
    }
    pub(crate) fn flashback_additional(self) -> Option<AdditionalCost> {
        authored::rules(self).and_then(|r| r.extras.flashback_additional)
    }
    pub(crate) fn additional_cost(self) -> Option<AdditionalCost> {
        generated::additional_cost(self)
    }
    pub(crate) fn static_ability_groups(self) -> &'static [StaticAbilityGroup] {
        authored::rules(self).map_or_else(
            || {
                let groups = generated::static_ability_groups(self);
                if self.controlled_entry_untapped() != 0 {
                    // These legacy providers were added after the original typed
                    // origin table. Their sole static effect gets its own source-
                    // local identity without interpreting English or card names.
                    assert!(
                        groups.is_empty(),
                        "new typed provider needs explicit combined groups"
                    );
                    &[StaticAbilityGroup {
                        line: 0,
                        first_layer: AbilityLayer::Rule,
                        fragments: &[StaticAbilityFragment::ControlledEntryUntapped],
                    }]
                } else {
                    groups
                }
            },
            |rules| rules.static_groups,
        )
    }
    pub(crate) fn live_static_effects(self) -> &'static [LiveStaticEffect] {
        generated::live_static_effects(self)
    }
    pub(crate) fn tiered_costs(self) -> Option<&'static [ManaCost]> {
        authored::mode_costs(self).or_else(|| generated::tiered_costs(self))
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
    pub(crate) fn controlled_entry_untapped(self) -> u16 {
        generated::controlled_entry_untapped(self)
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
        authored::rules(self).map_or_else(|| generated::aura(self), |rules| rules.aura)
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
        Self::kinds_are_instant(self.kind_mask())
    }
    /// One canonical Instant-kind interpretation for printed and live type queries.
    pub(crate) const fn kinds_are_instant(kinds: u16) -> bool {
        kinds & 8 != 0
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

    pub(crate) fn graveyard_counter_cost(self) -> Option<u16> {
        authored::rules(self).and_then(|r| r.extras.graveyard_counter_cost)
    }
    pub(crate) fn grants_activation(self, index: u16) -> bool {
        self.static_ability_groups().iter().any(|g| {
            g.fragments
                .iter()
                .any(|f| matches!(f, StaticAbilityFragment::AttachmentActivation(i) if *i == index))
        })
    }
    pub(crate) fn borrows_exiled_activations(self) -> bool {
        authored::RULES[self.index()]
            .extras
            .borrows_exiled_activations
    }
    /// Intrinsic land/dork mana is stored separately from printed activation tables.
    /// The reserved index is used only by a captured inherited-ability descriptor.
    pub(crate) fn activation(self, index: u16) -> &'static ActivatedAbility {
        if index == u16::MAX {
            static INTRINSIC: [ActivatedAbility; 6] = [
                ActivatedAbility {
                    exhaust: false,
                    guard: ActivationGuard::Always,
                    origin: ActivationOrigin::Battlefield,
                    cycling: false,
                    cost: ActivationCost {
                        colored: [0; 6],
                        generic: 0,
                        alternatives: &[],
                        additional: None,
                        mana_component: false,
                        tap: true,
                        sacrifice: false,
                        exile_self: false,
                        discard: false,
                        life: 0,
                        remove: None,
                        loyalty: None,
                    },
                    targets: TargetPlan::NONE,
                    program: &[Instruction::AddMana(ManaProduction::Fixed([
                        1, 0, 0, 0, 0, 0,
                    ]))],
                    sorcery: false,
                    once_per_turn: false,
                },
                ActivatedAbility {
                    exhaust: false,
                    guard: ActivationGuard::Always,
                    origin: ActivationOrigin::Battlefield,
                    cycling: false,
                    cost: ActivationCost {
                        colored: [0; 6],
                        generic: 0,
                        alternatives: &[],
                        additional: None,
                        mana_component: false,
                        tap: true,
                        sacrifice: false,
                        exile_self: false,
                        discard: false,
                        life: 0,
                        remove: None,
                        loyalty: None,
                    },
                    targets: TargetPlan::NONE,
                    program: &[Instruction::AddMana(ManaProduction::Fixed([
                        0, 1, 0, 0, 0, 0,
                    ]))],
                    sorcery: false,
                    once_per_turn: false,
                },
                ActivatedAbility {
                    exhaust: false,
                    guard: ActivationGuard::Always,
                    origin: ActivationOrigin::Battlefield,
                    cycling: false,
                    cost: ActivationCost {
                        colored: [0; 6],
                        generic: 0,
                        alternatives: &[],
                        additional: None,
                        mana_component: false,
                        tap: true,
                        sacrifice: false,
                        exile_self: false,
                        discard: false,
                        life: 0,
                        remove: None,
                        loyalty: None,
                    },
                    targets: TargetPlan::NONE,
                    program: &[Instruction::AddMana(ManaProduction::Fixed([
                        0, 0, 1, 0, 0, 0,
                    ]))],
                    sorcery: false,
                    once_per_turn: false,
                },
                ActivatedAbility {
                    exhaust: false,
                    guard: ActivationGuard::Always,
                    origin: ActivationOrigin::Battlefield,
                    cycling: false,
                    cost: ActivationCost {
                        colored: [0; 6],
                        generic: 0,
                        alternatives: &[],
                        additional: None,
                        mana_component: false,
                        tap: true,
                        sacrifice: false,
                        exile_self: false,
                        discard: false,
                        life: 0,
                        remove: None,
                        loyalty: None,
                    },
                    targets: TargetPlan::NONE,
                    program: &[Instruction::AddMana(ManaProduction::Fixed([
                        0, 0, 0, 1, 0, 0,
                    ]))],
                    sorcery: false,
                    once_per_turn: false,
                },
                ActivatedAbility {
                    exhaust: false,
                    guard: ActivationGuard::Always,
                    origin: ActivationOrigin::Battlefield,
                    cycling: false,
                    cost: ActivationCost {
                        colored: [0; 6],
                        generic: 0,
                        alternatives: &[],
                        additional: None,
                        mana_component: false,
                        tap: true,
                        sacrifice: false,
                        exile_self: false,
                        discard: false,
                        life: 0,
                        remove: None,
                        loyalty: None,
                    },
                    targets: TargetPlan::NONE,
                    program: &[Instruction::AddMana(ManaProduction::Fixed([
                        0, 0, 0, 0, 1, 0,
                    ]))],
                    sorcery: false,
                    once_per_turn: false,
                },
                ActivatedAbility {
                    exhaust: false,
                    guard: ActivationGuard::Always,
                    origin: ActivationOrigin::Battlefield,
                    cycling: false,
                    cost: ActivationCost {
                        colored: [0; 6],
                        generic: 0,
                        alternatives: &[],
                        additional: None,
                        mana_component: false,
                        tap: true,
                        sacrifice: false,
                        exile_self: false,
                        discard: false,
                        life: 0,
                        remove: None,
                        loyalty: None,
                    },
                    targets: TargetPlan::NONE,
                    program: &[Instruction::AddMana(ManaProduction::Fixed([
                        0, 0, 0, 0, 0, 1,
                    ]))],
                    sorcery: false,
                    once_per_turn: false,
                },
            ];
            &INTRINSIC[self.mana_color().expect("captured intrinsic mana")]
        } else {
            &self.activations()[usize::from(index)]
        }
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
        let mut counters: Counters = generated::DEFINITIONS[self.0 as usize].enters.into();
        counters.loyalty = self.starting_loyalty();
        counters.lore = u64::from(self.saga_final_chapter().is_some());
        for &(kind, amount) in authored::RULES[self.index()].extras.fixed_entry_counters {
            counters.add(kind, amount);
        }
        counters
    }
    pub(crate) fn opening_hand_battlefield(self) -> bool {
        authored::RULES[self.index()]
            .extras
            .opening_hand_battlefield
    }
    pub(crate) fn named_activation_lock(self) -> bool {
        authored::RULES[self.index()].extras.named_activation_lock
    }
    pub(crate) fn entry_choice(self) -> Option<EntryChoiceKind> {
        authored::RULES[self.index()].extras.entry_choice
    }
    pub(crate) fn chosen_entry_counters(self) -> Option<CounterKind> {
        authored::RULES[self.index()].extras.chosen_entry_counters
    }
    pub(crate) fn power_up_abilities(self) -> &'static [u16] {
        authored::RULES[self.index()].extras.power_up_abilities
    }
    pub(crate) fn power_up_additional_uses(self) -> u32 {
        authored::RULES[self.index()]
            .extras
            .power_up_additional_uses
    }
    pub(crate) fn power_up_other_creature_reduction(self) -> u32 {
        authored::RULES[self.index()]
            .extras
            .power_up_other_creature_reduction
    }
    pub(crate) fn power_up_mana_cost_alternatives(self) -> &'static [ManaCost] {
        authored::RULES[self.index()]
            .extras
            .power_up_mana_cost_alternatives
    }
    pub(crate) fn activation_target_reduction(
        self,
        ability: u16,
    ) -> Option<TargetActivationReduction> {
        authored::RULES[self.index()]
            .extras
            .activation_target_reductions
            .iter()
            .find(|(index, _)| *index == ability)
            .map(|(_, rule)| *rule)
    }
    pub(crate) fn activation_counter_discount(self, ability: u16) -> Option<CounterKind> {
        authored::RULES[self.index()]
            .extras
            .activation_counter_discounts
            .iter()
            .find(|(index, _)| *index == ability)
            .map(|(_, kind)| *kind)
    }
    pub(crate) fn activation_put_counter(self, ability: u16) -> Option<(CounterKind, u64)> {
        authored::RULES[self.index()]
            .extras
            .activation_put_counters
            .iter()
            .find(|(index, _, _)| *index == ability)
            .map(|(_, kind, amount)| (*kind, *amount))
    }
    pub(crate) fn cast_entry_counters(self) -> &'static [(CounterKind, u64)] {
        authored::RULES[self.index()].extras.cast_entry_counters
    }
    pub(crate) fn hand_cast_entry_counters(self) -> &'static [(CounterKind, u64)] {
        authored::RULES[self.index()]
            .extras
            .hand_cast_entry_counters
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
    /// Intrinsic mana value of the represented face; X is zero outside the stack.
    pub fn fixed_mana_value(self) -> u64 {
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
    pub(crate) fn cast_timing_requires_opponent_spell(self) -> bool {
        authored::rules(self).is_some_and(|r| r.extras.cast_timing_requires_opponent_spell)
    }
    pub(crate) fn cost_reductions(self) -> &'static [SpellReduction] {
        generated::cost_reductions(self)
    }
    pub(crate) fn cost_alternative_guard(self) -> Option<StepGuard> {
        generated::cost_alternative_guard(self)
    }
    pub(crate) fn mana_alternative_replaces_additional(self, cost: ManaCost) -> bool {
        (generated::mana_alternative_replaces_additional(self)
            && generated::cost_alternatives(self).contains(&cost))
            || self.explicit_additional_mana().is_some_and(|extra| {
                self.cost_options()
                    .any(|base| Self::add_mana_cost(base, extra) == cost)
            })
    }
    pub(crate) fn kicked_targets(self) -> Option<TargetPlan> {
        generated::kicked_targets(self)
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
    pub fn has_improvise(self) -> bool {
        authored::RULES[self.index()].extras.has_improvise
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
    pub(crate) fn teamwork(self) -> Option<TeamworkSpec> {
        authored::rules(self).and_then(|r| r.extras.teamwork)
    }
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
    pub(crate) fn zone_replacement(self) -> Option<ZoneReplacementRule> {
        generated::zone_replacement(self)
    }
    pub(crate) fn explicit_additional_mana(self) -> Option<ManaCost> {
        authored::rules(self).and_then(|r| r.extras.additional_mana_alternative)
    }
    pub(crate) fn add_mana_cost(base: ManaCost, extra: ManaCost) -> ManaCost {
        ManaCost {
            colored: std::array::from_fn(|i| {
                base.colored[i]
                    .checked_add(extra.colored[i])
                    .expect("additional colored cost")
            }),
            generic: base
                .generic
                .checked_add(extra.generic)
                .expect("additional generic cost"),
        }
    }
    pub(crate) fn additional_mana_options(self, base: ManaCost) -> impl Iterator<Item = ManaCost> {
        std::iter::once(base).chain(
            self.explicit_additional_mana()
                .map(|extra| Self::add_mana_cost(base, extra)),
        )
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
