#[doc(hidden)]
#[path = "ability_layers.rs"]
pub mod ability_layers;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub use ability_layers::{
    AbilityLayer, AdditionalTriggerScope, AnimationCreatureTypes, AnimationDuration, AnimationSpec,
    AttachedCharacteristics, CreatureSubtypeGrant, LandManaRecipients, PermanentForm,
    StaticAbilityFragment, StaticAbilityGroup, UntapRestriction,
};

#[doc(hidden)]
#[cfg(feature = "test-support")]
#[path = "cards_test_counter_providers.rs"]
pub mod test_counter_providers;

#[doc(hidden)]
#[cfg(feature = "test-support")]
#[path = "cards_fixture_rules.rs"]
pub mod fixture_rules;

#[doc(hidden)]
#[path = "card_mana_symbols.rs"]
pub mod mana_symbols;

use crate::capacity::CapacityExt;
use crate::engine::{GrantedSubtypes, Player, Zone};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Immutable additions made as part of a token-copy effect (CR707.9).
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub struct CopyExceptions {
    pub sacrifice_abilities: u16,
    /// Copiable intervening-if sacrifice triggers granted with haste.
    pub conditional_sacrifice_abilities: u16,
    pub nightmare: bool,
    pub artifact: bool,
    pub golem: bool,
    pub fractal: bool,
    pub base: Option<(i16, i16)>,
}
/// A layer-one copy between independently selected exact objects. Group absence
/// at resolution is intentional: losing either required group prevents the copy.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InPlaceCopySpec {
    /// None copies the ability source; Some selects this target group.
    pub recipient_group: Option<u16>,
    pub blueprint_group: u16,
    pub overrides: crate::engine::CopyOverrides,
    pub add_kinds: u16,
    pub base: Option<(i16, i16)>,
    pub next_turn: bool,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl CopyExceptions {
    pub const NONE: Self = Self {
        sacrifice_abilities: 0,
        conditional_sacrifice_abilities: 0,
        nightmare: false,
        artifact: false,
        golem: false,
        fractal: false,
        base: None,
    };
    pub const HASTE_AND_SACRIFICE: Self = Self {
        sacrifice_abilities: 1,
        conditional_sacrifice_abilities: 0,
        nightmare: false,
        artifact: false,
        golem: false,
        fractal: false,
        base: None,
    };
    pub const NIGHTMARE: Self = Self {
        sacrifice_abilities: 0,
        conditional_sacrifice_abilities: 0,
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
            conditional_sacrifice_abilities: self
                .conditional_sacrifice_abilities
                .checked_add(other.conditional_sacrifice_abilities)
                .capacity("copy conditional sacrifice abilities"),
            nightmare: self.nightmare || other.nightmare,
            artifact: self.artifact || other.artifact,
            golem: self.golem || other.golem,
            fractal: self.fractal || other.fractal,
            base: other.base.or(self.base),
        }
    }
}

/// A temporary permission on one graveyard incarnation, independent of its grantor.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum GraveyardCastKind {
    /// Play this exact card normally, including a land, through the deadline.
    Play,
    Cast,
    Flashback,
}

/// Compact catalogue identity. The immutable definitions are shared across all
/// games; no card strings, JSON, or rule objects are cloned into game states.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Card(u16);

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Effect {
    Creature(i32, i32),
    Spell(TargetPlan, &'static [Instruction]),
    Modal(&'static [SpellMode]),
    Land,
    Permanent,
    Emblem,
    Aura(TargetPlan),
    Unsupported,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpellMode {
    pub text: &'static str,
    pub targets: TargetPlan,
    pub program: &'static [Instruction],
}

/// The four complete Gift payloads present in the pinned Standard catalogue.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum GiftKind {
    Card,
    Food,
    Treasure,
    TappedFish,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GiftSpec {
    pub kind: GiftKind,
    pub token: Option<Card>,
    pub promised_targets: TargetPlan,
}

/// A relationship between distinct bindings within one occurrence of target.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TargetGroupConstraint {
    SameGraveyard,
    /// Each distinct target must fit a distinct card-type slot.
    DistinctCardTypeSlots,
    /// Pair shares at least one current card type, not a subtype or supertype.
    SharedCardType,
    /// Controlled by the player chosen in a distinct target group.
    ControllerOfPlayerGroup(u16),
    /// Attached to the exact object selected in another target group.
    AttachedToGroup(u16),
    DifferentControllers,
    OtherThanGroup(u16),
    TotalManaValueAtMost(u64),
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TargetRule {
    ManaValueExactly {
        base: &'static TargetRule,
        amount: AmountValue,
        value: Option<u64>,
    },
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
    /// Any spell controlled by the choosing player, regardless of its card types.
    YourSpell,
    SingleTargetStackObject,
    /// Any activated or triggered ability on the stack.
    Ability,
    YourSpellKinds {
        kinds: u8,
    },
    /// An independent stack ability, qualified by its actual source or exact LKI.
    YourAbilityFromSourceKinds {
        kinds: u16,
    },
    /// A spell that is a creature or legendary.
    CreatureOrLegendarySpell,
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

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
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
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TargetPredicate {
    OwnedByController,
    EnteredGraveyardThisTurn,
    /// This exact battlefield incarnation entered during the current turn.
    EnteredBattlefieldThisTurn,
    ManaValueAtMostLifeGained,
    OtherThanSource,
    OtherThanSourceAttachment,
    Blocking,
    Nontoken,
    Permanent,
    Attacking,
    /// This exact permanent was declared as an attacker earlier this turn.
    AttackedThisTurn,
    Nonattacking,
    /// The sole creature currently attacking, irrespective of its defender.
    AttackingAlone,
    AttackingOrBlocking,
    Damaged,
    DealtDamage,
    HasCounter(CounterKind),
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
    NoAbilities,
    Legendary,
    Nonlegendary,
    NonOutlaw,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TargetZone {
    Battlefield,
    Graveyard,
}

/// One homogeneous occurrence of "target"; distinct choices, incremental bounds.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TargetSpec {
    pub rule: TargetRule,
    pub min: u16,
    /// usize::MAX denotes any number; only arena/address-space limits apply.
    pub max: usize,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
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
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TargetPlan {
    Single(TargetSpec),
    Groups(&'static [TargetSpec]),
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
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

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlayerSet {
    /// The immutable owner of the exact source card, independent of controller.
    SourceOwner,
    /// Frozen controller of the first surviving creature target in this group.
    TargetController,
    CastingPlayer,
    /// Untargeted player frozen when the triggering event occurred.
    EventPlayer,
    DrawingPlayer,
    Active,
    DamagedPlayer,
    Controller,
    Opponent,
    Each,
    /// Each player whose own persistent speed is strictly below this threshold.
    SpeedBelow(u8),
    Targeted,
}

/// A color choice excludes colorless (CR105.2).
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChoiceColor {
    White,
    Blue,
    Black,
    Red,
    Green,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl ChoiceColor {
    pub const ALL: [Self; 5] = [Self::White, Self::Blue, Self::Black, Self::Red, Self::Green];
    pub(crate) const fn index(self) -> usize {
        self as usize
    }
}
/// Validated index into the pinned complete creature-type taxonomy; wire values
/// remain readable names and no owned String is stored in a game branch.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CreatureType(u16);
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl CreatureType {
    pub fn from_name(name: &str) -> Option<Self> {
        CREATURE_TYPES
            .binary_search(&name)
            .ok()
            .map(|i| Self(u16::try_from(i).expect("creature taxonomy fits u16")))
    }
    pub fn name(self) -> &'static str {
        CREATURE_TYPES[self.index()]
    }
    pub(crate) const fn index(self) -> usize {
        self.0 as usize
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Serialize for CreatureType {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(self.name())
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl<'de> Deserialize<'de> for CreatureType {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let name = String::deserialize(d)?;
        Self::from_name(&name).ok_or_else(|| serde::de::Error::custom("not a legal creature type"))
    }
}
/// A basic land type chosen as an object enters, in WUBRG order.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum BasicLandType {
    Plains,
    Island,
    Swamp,
    Mountain,
    Forest,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl BasicLandType {
    pub const ALL: [Self; 5] = [
        Self::Plains,
        Self::Island,
        Self::Swamp,
        Self::Mountain,
        Self::Forest,
    ];
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::Plains => "Plains",
            Self::Island => "Island",
            Self::Swamp => "Swamp",
            Self::Mountain => "Mountain",
            Self::Forest => "Forest",
        }
    }
    pub(crate) const fn index(self) -> usize {
        self as usize
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EntryChoiceValue {
    CardName(crate::CardName),
    Color(ChoiceColor),
    BasicLandType(BasicLandType),
    CreatureType(CreatureType),
    ManaValueParity(crate::engine::ManaValueParity),
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum EntryChoiceKind {
    CardName,
    NonlandCardName,
    CardNameAfterOpponentHand,
    ManaValueParity,
    LandCardName,
    Color,
    BasicLandType,
    CreatureType,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl EntryChoiceKind {
    pub(crate) fn matches(self, value: EntryChoiceValue) -> bool {
        if self == Self::NonlandCardName {
            return matches!(value, EntryChoiceValue::CardName(name) if !name.is_land());
        }
        if self == Self::LandCardName {
            return matches!(value, EntryChoiceValue::CardName(name) if name.is_land());
        }
        matches!(
            (self, value),
            (
                Self::CardNameAfterOpponentHand | Self::CardName,
                EntryChoiceValue::CardName(_)
            ) | (Self::ManaValueParity, EntryChoiceValue::ManaValueParity(_))
                | (Self::Color, EntryChoiceValue::Color(_))
                | (Self::BasicLandType, EntryChoiceValue::BasicLandType(_))
                | (Self::CreatureType, EntryChoiceValue::CreatureType(_))
        )
    }
}
/// Layer-seven own-creature bonus using one source's entry choice. The optional
/// counter factor borrows the same sparse counter world as mana planning.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChosenModifier {
    pub kind: EntryChoiceKind,
    pub other: bool,
    pub stats: Stats,
    pub source_counter: Option<CounterKind>,
}
/// Rules attached to produced mana, captured independently of its producer.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ManaRestriction {
    /// Any cost except casting a spell from a hand.
    ExceptCastFromHand,
    /// Only spells cast from exile.
    CastFromExile,
    /// Only the costs of casting a spell with the artifact card type.
    ArtifactSpell,
    /// Only artifact spell casts or activated-ability costs; never unrelated payments.
    ArtifactSpellOrActivation,
    /// Any cost except casting a spell without the artifact card type.
    ExceptNonArtifactSpells,
    /// Spells or activated abilities of any source with this subtype.
    SubtypeSpellOrActivation(&'static str),
    InstantSorcery,
    /// Only the cost of casting a noncreature spell.
    NoncreatureSpell,
    /// Only the cost of casting a planeswalker spell.
    PlaneswalkerSpell,
    Unrestricted,
    SpellSubtype(&'static str),
    ChosenCreatureSpellOrActivation,
    ChosenCreatureSpell,
    EquipmentSpellOrEquip,
    CreatureActivation,
    /// Only costs of activating abilities, including mana abilities.
    AbilityActivation,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ManaSpellBonus {
    None,
    DragonCreatureHaste,
    Uncounterable,
    CopyRedInstantSorcery,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize)]
pub enum ManaRetention {
    #[default]
    Step,
    Combat,
    Turn,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ManaQuality {
    pub restriction: ManaRestriction,
    pub spell_bonus: ManaSpellBonus,
    pub retention: ManaRetention,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ManaProduction {
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
    /// Choose one color and produce one mana per distinct controlled creature power.
    DistinctCreaturePowers,
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
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl ManaProduction {
    pub const fn any_color(amount: u32) -> Self {
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
            | Self::DistinctCreaturePowers
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

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LibraryRemainder {
    /// Shuffle the entire library after selected entries finish.
    Shuffle,
    Graveyard,
    Hand,
    BottomAny,
    BottomRandom,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LibrarySelectionFilter {
    HasX,
    Any,
    Kinds(u16),
    ExcludingKinds(u16),
    Subtype(&'static str),
    AnyOf(&'static [LibrarySelectionFilter]),
    AllOf(&'static [LibrarySelectionFilter]),
    CreaturePowerAtMost(i32),
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LibrarySelectionCount {
    Fixed(u16),
    /// Any number of the eligible cards in the looked group.
    All,
    /// One uniformly random eligible card, without a player selection.
    RandomOne,
    Lessons,
    InstantAndSorcery,
    GainedLife,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LibrarySelectionSpec {
    pub look: u16,
    pub count: LibrarySelectionCount,
    pub filter: LibrarySelectionFilter,
    pub optional: bool,
    pub reveal: bool,
    pub remainder: LibraryRemainder,
}

/// Explicit destinations for cards looked at without changing zones first.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LibraryPickDestination {
    /// A single forced candidate; evaluate the threshold before any entry choices.
    BattlefieldIfManaValueAtMostElseHand(AmountValue),
    BattlefieldTapped,
    LandsTappedRestHand,
    Hideaway,
    Exile,
    Hand,
    Battlefield,
    Top,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LibraryLook {
    Amount(AmountValue),
    Fixed(u16),
    ChosenX,
    UntilKinds(u16),
    UntilKindsCount { kinds: u16, count: u16 },
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LibrarySelectionEffect {
    pub spec: LibrarySelectionSpec,
    pub look: LibraryLook,
    pub destination: LibraryPickDestination,
    pub public: bool,
    pub maximum_mana_value: Option<AmountValue>,
}

/// Fixed qualities admitted by library-search instructions.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum SearchFilter {
    Any,
    NoAbilities,
    Colorless,
    Nonland,
    ManaValue {
        minimum: u64,
        maximum: u64,
    },
    KindsManaValue {
        kinds: u16,
        minimum: u64,
        maximum: u64,
    },
    AnyOf(&'static [SearchFilter]),
    AllOf(&'static [SearchFilter]),
    Name(&'static str),
    Kinds(u16),
    Keyword(Keywords),
    BasicLand,
    Land,
    Creature,
    Legendary,
    Artifact,
    Enchantment,
    Subtype(&'static str),
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl SearchFilter {
    pub(crate) fn matches(self, card: Card) -> bool {
        match self {
            Self::Any => true,
            Self::NoAbilities => !card.has_rules_abilities(),
            Self::Colorless => card.colors() == 0,
            Self::Nonland => !card.is_land(),
            Self::ManaValue { minimum, maximum } => {
                (minimum..=maximum).contains(&card.fixed_mana_value())
            }
            Self::KindsManaValue {
                kinds,
                minimum,
                maximum,
            } => {
                card.kind_mask() & kinds != 0
                    && (minimum..=maximum).contains(&card.fixed_mana_value())
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
            Self::Legendary => card.is_legendary(),
            Self::Artifact => card.is_artifact(),
            Self::Enchantment => card.kind_mask() & 32 != 0,
            Self::Subtype(subtype) => card.has_subtype(subtype),
        }
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SearchDestination {
    /// Reveal the selected cards, then choose one destination for the selection.
    HandOrGraveyard,
    /// Face-up exile linked to this exact source incarnation, without an automatic return.
    LinkedExile,
    /// Exile the found cards and allow their controller to cast them free this turn.
    ExileFreeThisTurn,
    Graveyard,
    /// Shuffle, then put the selected cards on top (last selected uppermost).
    Top,
    Hand,
    Battlefield,
    BattlefieldThenHand {
        battlefield: u16,
    },
    BattlefieldIfManaValueAtMostElseHand(u64),
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct SearchSpec {
    pub filter: SearchFilter,
    pub maximum: u64,
    pub destination: SearchDestination,
    pub tapped: bool,
    pub reveal: bool,
}

/// An immutable disjunction over the selected cost object's live characteristics.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum CostMatch {
    Kinds(u16),
    Subtype(&'static str),
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum CostAction {
    Sacrifice,
    TapPermanents(u16),
    SacrificeRange { minimum: u16 },
    ExileGraveyardRange { minimum: u16 },
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum DelayedTargetEvent {
    DiesOnce,
    CombatDamageToPlayer,
}

/// A resolution choice; discard here is a cost, not a forced discard effect.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum EffectChoiceKind {
    /// Apply one ordinary payment operation with an additional typed OR filter.
    FilteredCost {
        action: CostAction,
        any: &'static [CostMatch],
    },
    /// Choose a controlled permanent or reveal a matching card in hand.
    Behold,
    /// Non-targeted Army selection during amass.
    AmassGoblins(u64),
    EmpowerJace(u64),
    ExileGreatestManaValue,
    /// Sacrifice one controlled permanent of greatest mana value among matching kinds.
    SacrificeGreatestManaValue,
    ExileGraveyard,
    /// Craft material: an artifact you control or an artifact card in your graveyard.
    ExileControlledOrGraveyard,
    /// Craft with at least this many nonlands that have activated abilities.
    ExileCraftNonlands(u16),
    /// Internal projected route for a controlled battlefield material.
    ExileControlled,
    /// During resolution, choose a card from either graveyard without targeting.
    ExileCardFromAnyGraveyard,
    /// Targeted player chooses a creature to exile with their graveyard.
    ExileCreatureAndGraveyard,
    CoverUpGraveyardCard,
    /// Optional combined search of an opponent's graveyard, hand and library.
    NamedSearch {
        owner: Player,
        name: crate::CardName,
        maximum: u16,
    },
    CoverUpSearch {
        owner: Player,
        name: &'static str,
    },
    /// Optionally choose a creature card captured by an exile-batch trigger.
    CopyExiledTriggerCreature,
    SacrificeDifferentNames(u16),
    /// Incremental non-target selection; the group is explicitly finished before payment.
    SacrificeRange {
        minimum: u16,
    },
    ExileGraveyardRange {
        minimum: u16,
    },
    /// Choose distinct untapped permanents as a non-targeting activation cost.
    TapPermanents(u16),
    ExileGraveyardCards(u16),
    /// Resolution cost: exile this resolving spell and named graveyard cards.
    ExileSourceAndNamed {
        name: &'static str,
        amount: u16,
    },
    /// Non-targeted graveyard cards of this color, with this printed-symbol sum.
    ExileGraveyardSymbols {
        color: ChoiceColor,
        minimum: u16,
    },
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
    /// Exile a nonland card in hand and mark the resulting exile object plotted.
    HandPlot {
        maximum: u32,
    },
    HandLand {
        tapped: bool,
    },
    /// Exact public cards from the current simultaneous discard batch.
    DiscardedLand {
        tapped: bool,
    },
    /// Non-targeted graveyard return with a shared total mana-value budget.
    GraveyardCreatures {
        maximum: u16,
        total_mana_value: u64,
    },
    LandCards {
        maximum: u32,
        tapped: bool,
    },
    CollectEvidence(u16),
    Forage,
    ForageOrBlack,
    Sacrifice,
    /// One controlled Equipment attached to this exact activated source.
    SacrificeAttachedEquipment,
    /// Non-targeted return payment for Sneak, after blockers are declared.
    ReturnUnblockedAttacker,
    ReturnToHand,
    Discard,
    /// Choose one card to discard or pay the specified life.
    DiscardOrLife(u32),
    DiscardNamed(&'static str),
    /// Discard a legendary card sharing a name with a controlled legendary permanent.
    DiscardMatchingLegendary,
    /// Discard any legendary card from the payer’s hand.
    DiscardLegendary,
    /// One controlled permanent matching the sacrifice filter, or one private hand card.
    /// The flag restricts the discard route, not the permanent, for resolution or activation costs.
    SacrificeOrDiscard {
        nonland: bool,
    },
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl EffectChoiceKind {
    pub(crate) fn payment_kind(self) -> Self {
        match self {
            Self::FilteredCost {
                action: CostAction::Sacrifice,
                ..
            } => Self::Sacrifice,
            Self::FilteredCost {
                action: CostAction::TapPermanents(n),
                ..
            } => Self::TapPermanents(n),
            Self::FilteredCost {
                action: CostAction::SacrificeRange { minimum },
                ..
            } => Self::SacrificeRange { minimum },
            Self::FilteredCost {
                action: CostAction::ExileGraveyardRange { minimum },
                ..
            } => Self::ExileGraveyardRange { minimum },
            other => other,
        }
    }
    pub(crate) fn variable_cost_minimum(self) -> Option<u16> {
        match self.payment_kind() {
            Self::SacrificeRange { minimum } | Self::ExileGraveyardRange { minimum } => {
                Some(minimum)
            }
            _ => None,
        }
    }
    pub(crate) fn cost_matches_any(
        self,
        card: Card,
        kinds: u16,
        subtypes: GrantedSubtypes,
    ) -> bool {
        match self {
            Self::FilteredCost { any, .. } => any.iter().any(|predicate| match predicate {
                CostMatch::Kinds(mask) => kinds & mask != 0,
                CostMatch::Subtype(name) => subtypes.has(card, name),
            }),
            _ => true,
        }
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SacrificeFilter {
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

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AdditionalCost {
    pub kind: EffectChoiceKind,
    pub filter: SacrificeFilter,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl AdditionalCost {
    /// A union cost selects exactly one route using the selected object's zone.
    /// The hand route has its own printed restriction; the permanent filter
    /// applies only to the sacrifice route.
    pub(crate) fn in_zone(mut self, zone: Zone) -> Self {
        if matches!(
            self.kind,
            EffectChoiceKind::ExileControlledOrGraveyard | EffectChoiceKind::ExileCraftNonlands(_)
        ) {
            match zone {
                Zone::Battlefield => self.kind = EffectChoiceKind::ExileControlled,
                Zone::Graveyard => self.kind = EffectChoiceKind::ExileGraveyard,
                _ => {}
            }
        }
        if self.kind == EffectChoiceKind::SacrificeAttachedEquipment && zone == Zone::Battlefield {
            self.kind = EffectChoiceKind::Sacrifice;
        }
        if matches!(self.kind, EffectChoiceKind::DiscardOrLife(_)) && zone == Zone::Hand {
            self.kind = EffectChoiceKind::Discard;
        }
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
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum MilledFilter {
    Any,
    Kinds(u8),
    KindsOrSubtype { kinds: u8, subtype: &'static str },
    Adventure,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum MilledDestination {
    PlayPermission { next_turn: bool },
    Hand,
    Battlefield,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MilledCount {
    Range { minimum: u16, maximum: u16 },
    All,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MilledSelectionSpec {
    pub filter: MilledFilter,
    pub count: MilledCount,
    pub destination: MilledDestination,
}

/// Card qualities for a non-targeted choice from an exact milled batch.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum RecoveryFilter {
    Permanent,
    Land,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
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

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MillAmount {
    RecipientGraveyard,
    LifeEvent,
    ChosenX,
    Fixed(u16),
    ControllerLife,
}

/// A fixed mana payment, independent of casting or activation costs.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManaCost {
    pub colored: Mana,
    pub generic: u32,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TargetActivationReduction {
    pub rule: TargetRule,
    pub generic: u32,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TargetSpellReduction {
    pub rule: TargetRule,
    pub discount: ManaCost,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub type ManaCostOptions = std::iter::Chain<
    std::iter::Once<ManaCost>,
    std::iter::Copied<std::slice::Iter<'static, ManaCost>>,
>;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CounterSubject {
    Targets,
    Related,
}

/// Immutable rule bytecode. Only the program counter and choices live in state.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BattlefieldFilter {
    pub kinds: u16,
    pub relation: i8,
    pub nonland: bool,
    pub tapped: Option<bool>,
    pub without: Keywords,
    pub subtype: Option<&'static str>,
    pub max_mana: u32,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Instruction {
    /// Marks the printed waterbend component of an activation cost.
    WaterbendCost(u16),
    /// Definition-only marker: exert the exact permanent as an activation cost.
    ExertCost,
    /// Increase the exact source permanent to this Class level.
    GainClassLevel(u8),
    /// First half of amass: create an Army only if none is controlled.
    /// Create a Jace token if none is controlled; replacements finish before empower.
    EnsureJaceToken,
    /// Put loyalty counters on a chosen controlled Jace planeswalker token.
    EmpowerJace(u16),
    #[allow(
        dead_code,
        reason = "Dynamic consumers remain gated until their interactions are tested"
    )]
    EmpowerJaceScaled(AmountValue),
    EnsureGoblinArmy,
    /// Complete amass after token creation and its replacements.
    AmassGoblins(u16),
    /// Complete amass, then attach the exact source Equipment to that Army.
    AmassGoblinsThenAttach(u16),
    EnsureGoblinArmyFor(PlayerSet),
    AmassGoblinsScaled {
        player: PlayerSet,
        amount: AmountValue,
    },
    DrawIfTargetController,
    /// Draw for the target's controller if it was not attacking at resolution.
    DrawTargetControllerIfNotAttacking,
    /// Draw if the target was legendary and had any of the given kinds at resolution.
    DrawIfTargetLegendaryKind(u16),
    /// Real nonmana variable activation cost; amount is declared before payment.
    SourceCounterCost {
        kind: CounterKind,
    },
    /// Mandatory untargeted resolution payment, then draw only if paid.
    RemoveCreatureCounterThenDraw,
    /// One simultaneous creation event, with all original recipes frozen first.
    CopyControlledNontokenCreaturesNonlegendary,
    /// Simultaneously copy each other controlled nonland permanent.
    CopyOtherControlledNonlands,
    CopyTargetPlayersCreaturesWithConditionalSacrifice,
    /// Preserve all copiable values except the legendary supertype.
    CreateNonlegendaryCopyTokens {
        subject: Subject,
        amount: u16,
    },
    RevealTopDrawIfArtifact,
    RevealTopMoveIfPermanent,
    ExileSourceReturnTransformed {
        counter: CounterKind,
    },
    /// Return the exact source incarnation exiled as an activation cost, transformed.
    ReturnCraftedSourceTransformed,
    /// Privately exile top cards; paid play remains conditional on a current subtype.
    ExileOpponentTopFaceDownPlayX,
    /// Independent, permanent exact-exile play permission for captured players.
    ExileLibraryTopFaceDownPlay {
        library: PlayerSet,
        beneficiary: PlayerSet,
        amount: u16,
        spend_any_mana: bool,
    },
    ExileTopFaceDownPlay {
        amount: u16,
        subtype: &'static str,
    },
    ExileTopPlayBoostSourceIfSubtype {
        subtype: &'static str,
        power: i32,
        toughness: i32,
    },
    /// First program header: a homogeneous optional target bound from the packet.
    UpToTargets(AmountValue),
    OfferMiracleCast(ManaCost),
    Cascade,
    /// Exile until a nonland card at or below this value; cast it or take it.
    Discover(u16),
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
    PrepareControlledCreatures,
    ChooseLandCardName,
    ChooseCardName {
        nonland: bool,
    },
    ExileTargetNamedCards {
        maximum: u16,
    },
    Prepare {
        subject: Subject,
    },
    Unprepare {
        subject: Subject,
    },
    CreateTokenChoice(&'static [Card]),
    /// Choose a keyword during resolution for the current exact subjects.
    ChooseGrantKeywords {
        subject: Subject,
        choices: &'static [Keywords],
    },
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
    /// Exile that many cards from each indicated player's library, with no play permission.
    ExileTopLibrary {
        players: PlayerSet,
        amount: AmountValue,
    },
    SearchDifferentNames(SearchSpec),
    /// Distinct-name search capped by the triggering entry's chosen X.
    SearchDifferentNamesChosenX(SearchSpec),
    RandomLinkedExileToBattlefield,
    PartitionLibraryPiles(u16),
    KeepOneOfEachPermanentType,
    RememberOwnSacrifice,
    RecoverAnotherPermanent,
    ReturnLinkedExileToHand,
    ReturnOneLinkedExileToHand,
    /// Trigger once per declaration attacking the resolving controller's opponent,
    /// through (but not including) the beginning of that controller's next turn.
    ScheduleOpponentAttackUntilNextTurn {
        ability: u16,
    },
    /// Apply one temporary bonus to the exact attackers captured by that event.
    GrowDeclaredAttackers {
        power: i32,
        toughness: i32,
        keywords: Keywords,
    },
    WeakenDeclaredAttacker(i32),
    ExileOpponentLibrariesExceptBottom,
    /// Create a source-independent delayed ability for each exact legal target.
    DelayTargetEvent {
        event: DelayedTargetEvent,
        ability: u16,
    },
    ScheduleAttackTriggerThisTurn {
        ability: u16,
    },
    BlinkImmediately {
        counter: Option<(CounterKind, u64)>,
        tapped: bool,
    },
    /// Exact historical cast origin; copied spells were not cast.
    IfCastFromGraveyard {
        skip: usize,
    },
    /// Copies and spells not cast from a hand take the non-hand branch.
    IfNotCastFromHand {
        skip: usize,
    },
    IfCastFromNonHand {
        skip: usize,
    },
    /// Exile a target; return small physical cards tapped under your control,
    /// then exile the returned incarnation at the next end step.
    ExileSmallReturnUntilEndStep,
    BlinkUntilEndStep {
        under_owner: bool,
    },
    ReturnDeathCard(ReturnKind),
    ResolveDelayedReturn,
    CopyNextInstantSorcery {
        ability: u16,
    },
    /// Register a source-independent next nonmana exhaust-activation callback.
    CopyNextExhaustAbility {
        ability: u16,
    },
    /// Create a source-independent delayed trigger for every spell cast this turn.
    CopySpellsThisTurn {
        ability: u16,
    },
    ScheduleSacrificedArtifactCopy {
        ability: u16,
    },
    CreateCapturedArtifactCopy,
    /// Copy one current/latest-departure entrant, then animate its actual results after entry.
    CopyEnteringPermanentThenAnimate(AnimationSpec),
    CopyCastSpell {
        prior_spells: bool,
    },
    /// Copy the actual triggering spell its chosen X times.
    CopyCastSpellChosenX,
    CopyPreparedCastSpell,
    /// Exile legal graveyard targets together, then copy the exiled cards and offer free casts.
    ExileTargetsCopyFreeWithBudget {
        mana_value: u64,
    },
    CopyPaidExiledCardsFree {
        maximum_casts: u16,
    },
    CopyStormSpell,
    ChangeSingleStackTarget,
    /// Optionally choose new targets for the exact spell captured by this cast trigger.
    ChangeCastSpellTargets,
    GrantGraveyardCast(GraveyardCastKind),
    GrantCreatureGraveyardCast,
    SolveSource,
    ReduceArtifactSpellCostsThisTurn(u32),
    NextRedOrGreenCreatureFreeThisTurn,
    /// Permission for the exact public card(s) just milled by this resolution.
    GrantMilledPlay,
    ExileDiscardedWithStash,
    /// Optional exact discarded-card exile; only actual success grants paid plays.
    MayExileDiscardedPlay {
        next_turn: bool,
    },
    ExileEachLibraryTopCastFree,
    /// Until cleanup, each tap of a Mountain for mana triggers an extra red mana.
    AddMountainManaTrigger,
    AllowJaceLoyaltyAnytime,
    /// Exile the controller’s top card, then offer one normal-cost cast.
    ExileTopMayCastPaid,
    /// Skip the following instructions if that offered cast committed.
    IfOfferedCastCommitted {
        skip: usize,
    },
    ExileTargetLibraryTopCastFree {
        amount: u16,
        maximum_casts: u16,
    },
    CastTargetedGraveyardFree,
    /// Optional paid cast from the controller’s graveyard, during resolution.
    CastGraveyard {
        kinds: u8,
        exile_instants_sorceries: bool,
    },
    CounterRelatedUnlessPayLife(u32),
    /// Ask the triggering opponent to pay by getting poison counters.
    MayPayWardPoison {
        amount: u16,
        skip: usize,
    },
    PayWardPoison(u16),
    /// One captured ward payer may discard a card or pay this mana cost.
    CounterRelatedUnlessDiscardOrPay(ManaCost),
    /// Ward payer may discard one card or decline and have the associated object countered.
    CounterRelatedUnlessDiscard,
    /// Captured ward payer may sacrifice exactly this many permanents.
    CounterRelatedUnlessSacrifice(u16),
    /// One combined ward cost, paid atomically or the related item is countered.
    CounterRelatedUnlessPayManaLife {
        cost: ManaCost,
        life: u32,
    },
    CreateTemporaryArtifactCopy,
    /// A normal token copy with source-independent next-end-step sacrifice.
    CreateCopyWithDelayedSacrifice,
    BecomeSourceLandUntilNextTurn,
    BecomeCopy(InPlaceCopySpec),
    /// Copy a chosen card from the trigger's exact exile batch onto its token target.
    CopyChosenExiledTriggerCreature,
    /// The resolving controller ignores the legend rule for the rest of this turn.
    IgnoreLegendRuleThisTurn,
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
    HarnessSource,
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
    /// Damage by this spell records actual recipients for an until-cleanup death replacement.
    ExileDamagedIfDiesThisTurn,
    /// Queue a reflexive ability only when this exact Role actually enters attached.
    CreateTokensReflexive {
        token: Card,
        amount: u16,
        ability: u16,
    },
    SacrificeSourceIfAttachedCounters {
        minimum: u64,
    },
    /// Capture all actual created tokens for one reflexive trigger.
    CreateTokensReflexiveWithSubjects {
        token: Card,
        amount: u16,
        ability: u16,
    },
    AttachReflexiveSubjects,
    GrantEquippedAttackers(Keywords),
    CreateRoleReflexive {
        token: Card,
        ability: u16,
    },
    /// The first fighter is the captured non-targeted successful-action subject.
    ReflexiveFight,
    /// Untargeted exact event object deals its current (or last-known) power.
    EventObjectPowerDamage,
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
    /// Branch on the legal target spell before any counter attempt changes zones.
    IfTargetSpellManaValueAtMost {
        maximum: u64,
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
    /// The currently enchanted creature fights the targeted creature.
    FightAttached,
    DiscardScaled(PlayerSet, AmountValue),
    DiscardManaValueLifeLoss,
    /// Mandatory one-card discard; only its actual land in your graveyard enters.
    DiscardThenReturnLand {
        tapped: bool,
    },
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
    PutHandAndAttachEquipment {
        filter: SearchFilter,
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
    /// One reflexive trigger after one or more unit generic payments, with a finite bound.
    PayXReflexiveLimited {
        ability: u16,
        maximum: u32,
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
    SacrificeSourceIfNoPlaneswalker,
    /// Continue in this resolution only if sacrificing the exact source succeeds.
    SacrificeSourceOrSkip {
        skip: usize,
    },
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
    DistributeCountersX {
        kind: CounterKind,
    },
    DamageByGroup {
        first: i32,
        other: i32,
    },
    DamageForExcess(AmountValue),
    /// Skip the following instructions when no positive excess was dealt.
    IfNoExcessDamage {
        skip: usize,
    },
    /// Consume the tracked excess and amass it for the packet controller.
    AmassExcessDamage,
    /// Consume the tracked excess and empower Jace by that amount.
    EmpowerJaceExcessDamage,
    /// Track actual positive damage and the recipient's subtype at that event.
    DamageForSubtypeDestruction {
        amount: u16,
        subtype: &'static str,
    },
    DestroyDamagedSubtype,
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
    CopyTargetAbility {
        group: u16,
    },
    /// Copy the nonmana activated ability captured by this trigger, if it
    /// remains on the stack, with optional new targets.
    CopyCapturedActivatedAbility,
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
    DiscardUpToOneDraw,
    /// Choose zero through `maximum` cards; draw only the actual discard count.
    DiscardUpToDraw {
        maximum: u16,
    },
    /// Mandatory opponent sacrifice, falling back to discard only if none is legal.
    OpponentSacrificeIfAbleElseDiscard {
        any: &'static [CostMatch],
    },
    /// Compare current target mana values at this instruction, after prior effects.
    DestroyTargetsAtMostManaValue(AmountValue),
    /// Reward once if an actual destruction included a creature controlled by this packet.
    DestroyTargetsThenSourceCountersIfOwnCreature {
        kind: CounterKind,
        amount: u16,
    },

    AdditionalCombat,
    /// Insert this many combats after the current main, then resume its frozen successor.
    AdditionalCombatsAfterCurrentMain(u16),
    /// Source-independent repeated delayed trigger, expiring at cleanup.
    ScheduleBeginCombatThisTurn {
        ability: u16,
    },
    /// All current creatures whose exact objects declared an attack this turn.
    UntapCreaturesThatAttackedThisTurn,
    ExtraTurn {
        power_up_prohibited: bool,
    },
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
    ResolveSiegeDefeat,
    Crew {
        minimum: u16,
    },
    BecomeSaddled {
        minimum: u16,
    },
    /// A resolving designation, independent of any Saddle ability or payment.
    MarkSaddled(Subject),
    ExileTopChoosePlay {
        amount: u16,
        next_turn: bool,
    },
    /// Exile cards with paid play permission until the controller next begins an end step.
    ExileTopPlayUntilNextEndStep(u16),
    ExileTopPlay {
        amount: u16,
        next_turn: bool,
    },
    ExileTargetLibraryPlayLife {
        amount: AmountValue,
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
    /// Queue only after a successful sacrifice, carrying its frozen power.
    ReflexiveSacrificedPower {
        ability: u16,
    },
    /// Definition-only triggered header whose modes may each be chosen once per source incarnation.
    ModalOnce(&'static [SpellMode]),
    /// Finite, nonrepeating combinations bounded by the packet's captured X.
    ModalUpToX {
        modes: &'static [SpellMode],
        counts: &'static [u8],
    },
    /// Triggered modes each available once during the current actual turn.
    ModalOncePerTurn(&'static [SpellMode]),
    /// Immediately lose the game, independently of life total or state-based actions.
    LoseGame(PlayerSet),
    /// Win if the resolving controller has no cards in their library.
    WinIfLibraryEmpty,
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
    /// Destroy all other creatures if the exact source's current or last power equals 20.
    DestroyOthersIfSourcePowerTwenty,
    /// Look at the top two cards, manifest one and put the rest into the graveyard.
    ManifestDread,
    /// When this exact target dies this turn, its controller's spell manifests dread.
    ManifestDreadWhenTargetDies,
    DestroyAndCreateForController {
        token: Card,
        amount: u16,
    },
    IfTeamwork {
        skip: usize,
    },
    /// Skip when the spell's optional blight cost was not paid.
    IfBlighted {
        skip: usize,
    },
    IfKicked {
        skip: usize,
    },
    IfBargained {
        skip: usize,
    },
    /// Live predicate on an exact resolving object, never on a reincarnation.
    /// Check the exact event object's current subtype, or its last known subtype.
    IfEventObjectSubtype {
        subtype: &'static str,
        skip: usize,
    },
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
    /// An all-or-nothing, simultaneous fixed exchange of exactly two legal objects.
    ExchangeControlTargets,
    GainControlUntilEndTurn,
    /// Lock control through your next turn when a controlled Villain has greater mana value.
    GainControlConditionalVillainManaValue,
    /// A fixed controller for this exact permanent, with no expiry or source dependence.
    GainControl {
        subject: Subject,
        controller: PlayerSet,
    },
    /// Choose up to this many creature cards with a combined mana-value limit.
    ChooseGraveyardReturn {
        maximum: u16,
        total_mana_value: u64,
    },
    ReturnGraveyardManaValue {
        kinds: u16,
        value: AmountValue,
    },
    DestroyManaValue {
        kinds: u16,
        maximum: AmountValue,
    },
    /// Simultaneously return the controller's nonland permanent cards.
    ReturnOwnedGraveyardNonlandPermanents,
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
    /// One damage batch to all creatures except the controller’s tokens.
    DamageCreaturesExceptControlledTokens(i32),
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
    /// Choose a creature type, then create an emblem carrying that choice.
    CreateEmblemForChosenCreatureType(Card),
    /// The controller’s next committed spell cast this turn cannot be countered.
    GrantNextSpellUncounterable,
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
    /// Stop a targeted permanent’s activated abilities until its next actual untap.
    PreventTargetActivationsWhileTapped,
    /// Return the exact resulting graveyard card captured from this death event.
    ReturnEventCardToHand,
    /// Return the exact death-event card tapped under its owner’s control.
    ReturnEventCardToBattlefieldTapped,
    PutSourceOnLibraryTop {
        origin: TargetZone,
    },
    ReturnSourceWithCounterAndEquipment(CounterKind),
    /// Return the exact death-event card, then modify only its actual entrant.
    ReturnSourceBlankWithHaste,
    /// Return this exact Aura from your graveyard already attached to its target.
    ReturnSourceAuraAttached,
    ReturnOpponentsBattlefieldGraveyardCardsAsFood,
    /// Return the exact death incarnation with its noncreature artifact form as it enters.
    ReturnSourceAsArtifact,
    /// Return the exact died source under its owner as a noncreature enchantment.
    ReturnSourceAsEnchantment,
    /// Return the exact graveyard source and legal targets together, tapped.
    ReturnSourceAndTargetsTapped,
    ReturnSourceToBattlefield {
        tapped: bool,
    },
    /// Resolve a hand activation by putting its exact source in tapped and
    /// attacking the defender captured when its unblocked attacker was returned.
    PutHandSourceAttacking,
    /// Return the exact graveyard source with an entry counter.
    ReturnSourceToBattlefieldWithCounter {
        tapped: bool,
        counter: CounterKind,
    },
    CounterRelated,
    AddPoison(PlayerSet, u16),
    #[allow(dead_code, reason = "Gated typed proliferate foundation")]
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
    /// Discard every selected player's hand; retain actual completed discard count.
    DiscardHandRememberCount(PlayerSet),
    TapCreatures(CreatureFilter),
    Mill(PlayerSet, MillAmount),
    /// Inherent rad-counter trigger: mill, lose life, then remove rad counters.
    ResolveRadiation,
    IncreaseSpeed,
    /// Decrease a selected player's speed by one only while they strictly lead, with a floor.
    DecreaseLeadingSpeed {
        players: PlayerSet,
        minimum: u8,
    },
    /// Event-local self-replacements belong to this resolving instruction.
    #[allow(dead_code, reason = "Gated typed proliferate foundation")]
    PutPlayerCountersWithSelf {
        players: PlayerSet,
        kind: PlayerCounterKind,
        amount: u64,
        effects: &'static [SelfPlayerCounterTemplate],
    },
    /// Mill the specified number, then queue a reflexive ability only if all were milled.
    MillThenReflexive {
        amount: u16,
        ability: u16,
    },
    MillLifeEvent,
    DrawForGraveyardsAtLeast(u16),
    MayMill(u16),
    RecoverMilled(RecoveryFilter),
    CollectGraveyard,
    CastLibraryTopUpToAttackingPower(u16),
    BottomLookedLibraryRandom,
    ControlTargetPlayerNextTurn,
    ExileAllCreaturesRemember,
    /// Exile creatures simultaneously and count only those actually exiled, including tokens.
    ExileAllCreaturesCount,
    DeployHandCreaturesEach,
    ReturnExiledResultToHands,
    ExileDamagedLibraryUntilNonland,
    MayCounterSourceOtherwiseCast,
    /// Private look, optional exact card choice, then discard and a draw for that player.
    LookTargetHandMayDiscardThenDraw,
    RevealTargetHandSubsetDiscard(AmountValue),
    RevealTargetHandExileUntilSourceLeaves,
    RevealTargetHandExileLinked,
    RevealHandDiscard {
        excluded_kinds: u16,
    },
    DestroyLoseManaValue,
    SelectMilled(MilledSelectionSpec),
    AttachSource,
    /// One optional nontargeting Equipment attachment to the selected target group.
    MayAttachControlledEquipment {
        recipient: u16,
        required_subtype: Option<&'static str>,
    },
    AttachEquipmentByGroups {
        equipment: u16,
        recipient: u16,
    },
    /// Queue one reflexive ability only for actual new attachments in this batch.
    AttachEquipmentGroupsReflexive {
        equipment: u16,
        recipient: u16,
        ability: u16,
    },
    AttachEquipmentToAttached {
        group: u16,
    },
    TapAttached,
    PutGrantorCounter(CounterKind),
    PutCombatDealerCounters,
    RemoveSourceCounterIfAny {
        kind: CounterKind,
        skip: usize,
    },
    /// Offer an optional removal only while the exact source has this counter.
    MayRemoveSourceCounter {
        kind: CounterKind,
        skip: usize,
    },
    CreateTokenIfControllerSacrificed(Card),
    OpponentsDiscardTwoDrawUnlessTwoNonland,
    RevealTwoOutsideCards,
    /// The instruction owns these self-replacements; they precede other effects.
    #[allow(
        dead_code,
        reason = "Synthetic outside admission self-replacement foundation"
    )]
    RevealTwoOutsideCardsWithSelf(&'static [SelfOutsideEntryTemplate]),
    /// Discard two, then create a reflexive trigger for actual nonland discards.
    DiscardTwoNonlandReflexive(u16),
    TargetStatsUntilNextTurn {
        power: i32,
        toughness: i32,
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
    CreaturePowerDamageAny {
        source: u16,
        recipient: u16,
    },
    SacrificeTarget {
        group: u16,
    },
    /// A targeted planeswalker deals damage equal to its current loyalty.
    LoyaltyDamage {
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
    /// The resolving controller chooses one color; indicated recipients receive the same production.
    AddManaChosenScaled {
        players: PlayerSet,
        amount: AmountValue,
    },
    IfSacrificedVehicle {
        skip: usize,
    },
    AddManaScaled {
        color: ChoiceColor,
        amount: AmountValue,
    },
    AddCombatRed(u16),
    AddCombatRedScaled(AmountValue),
    Scry(u16),
    Surveil(u16),
    /// Return a surveilled card that meets the life gained threshold after movement.
    SurveilOneReturnByLifeGained,
    /// Surveil one, returning noncreature nonland cards actually moved to the graveyard.
    SurveilOneReturnNoncreatureNonland,
    AnimateSource(AnimationSpec),
    AnimateSourceColored(AnimationSpec, u8),
    AnimateSourceWithAbilityIfNoncreature {
        spec: AnimationSpec,
        kind: crate::engine::SelfGrantedAbilityKind,
    },
    AnimateTargets(AnimationSpec),
    /// Animate an exact non-targeted entrant, source, attachment or legal target.
    AnimateSubject {
        subject: Subject,
        spec: AnimationSpec,
    },
    AnimateNoncreatureTargets(AnimationSpec),
    /// Add artifact without also granting creature or changing other characteristics.
    AddArtifactToTargets,
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
    SearchLibraryAndGraveyardWithXCounters {
        filter: SearchFilter,
        counter: CounterKind,
        haste_at: Option<u32>,
    },
    Search(SearchSpec),
    SearchScaled {
        spec: SearchSpec,
        maximum: AmountValue,
    },
    SearchLibraryAndOrGraveyardToHand(SearchFilter),
    /// Qualified battlefield search using the frozen mana value of a paid sacrifice.
    SearchSacrificedManaValue {
        kinds: u16,
        increment: u16,
    },
    /// Search up to the damage actually dealt to the source in this trigger event.
    SearchDamageReceived(SearchSpec),
    /// Choose hidden hand/library search zones, then find up to one card.
    SearchHandAndOrLibrary(SearchFilter),
    /// Remember the exact result of the immediately following library search.
    CaptureSearchResult,
    /// Capture the first attempted entry of the next entry operation, including failure/decline.
    CaptureEntryResult,
    AnimateEntryResultIfSubtype {
        subtype: &'static str,
        spec: AnimationSpec,
    },
    PutEntryResultCountersIfKinds {
        kinds: u16,
        kind: CounterKind,
        amount: u16,
    },
    GrantEntryResultKeywords {
        keywords: Keywords,
        permanent: bool,
    },
    ScheduleEntryResultExile {
        own_end_step: bool,
    },
    ReturnGraveyardWithDelayedSacrifice(SearchFilter),
    AnimatePermanentGroup {
        filters: &'static [BattlefieldFilter],
        spec: AnimationSpec,
    },
    /// Optionally behold a subtype, then untap the searched-for permanent.
    BeholdToUntapSearchResult(&'static str),
    /// Untap the searched permanent if its controller controls enough lands.
    UntapSearchResultIfLandsAtLeast(u32),
    SearchTargetController(SearchSpec),
    /// The legal player target in the currently selected target group searches.
    SearchTargetPlayer(SearchSpec),
    /// Search up to the saved actual result of the preceding move instruction.
    SearchTargetPlayerMovedCount(SearchSpec),
    /// Single-player optional effect, including zero-card searches.
    /// Record APNAP optional wheel decisions before performing either player's action.
    BeginOptionalWheel(u16),
    AskOptionalWheel {
        first: bool,
        skip: usize,
    },
    RecordOptionalWheel {
        first: bool,
        accept: bool,
    },
    ExecuteOptionalWheel,
    MayPlayer {
        player: PlayerSet,
        skip: usize,
    },
    /// Freeze the current controlled attacking creatures, then save successful moves.
    ExileAttackingCreatures(PlayerSet),
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
    /// Continue only when this sampled nonnegative amount has the requested parity.
    IfAmountParity {
        value: AmountValue,
        odd: bool,
        skip: usize,
    },
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
    /// The resolving ability’s source damages the current or last-known controller of its event object.
    DamageEventObjectController(i32),
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
    /// Untap every token currently controlled by the resolving player.
    UntapControlledTokens,
    /// Untap every land currently controlled by the resolving player.
    UntapControlledLands,
    /// Untap the current deduplicated non-targeted battlefield recipient union.
    UntapPermanentGroup(&'static [BattlefieldFilter]),
    /// Freeze controlled creatures with any kind of counter, then grant through cleanup.
    GrantCounterBearingCreatures(Keywords),
    /// Remove only these keyword abilities from a frozen union through cleanup.
    RemovePermanentGroupKeywords {
        filters: &'static [BattlefieldFilter],
        keywords: Keywords,
    },
    /// Grant target land colorless mana until the paid exiled source is cast.
    GrantLandManaUntilSourceCast,
    UntapPlayerCreatures(PlayerSet),
    /// Controlled creatures other than exact objects in the selected target group.
    PutOtherControlledCreatureCounters {
        kind: CounterKind,
        amount: u32,
    },
    /// All controlled creatures except the exact source, in one placement event.
    PutOtherControlledCreaturesExceptSource {
        kind: CounterKind,
        amount: u32,
    },
    PutPlayerCreatureCounters {
        players: PlayerSet,
        kind: CounterKind,
        amount: u32,
    },
    /// Put counters on controlled creatures that already have the specified kind.
    PutControlledCreatureCountersWith {
        required: CounterKind,
        kind: CounterKind,
        amount: u32,
    },
    SetBaseStats {
        subject: Subject,
        power: Option<i128>,
        toughness: Option<i128>,
    },
    /// Sample signed source P/T once, using departure LKI if its exact object left.
    SetBaseStatsFromSource {
        subject: Subject,
    },
    /// Freeze one non-targeted group and sampled signed values until cleanup.
    SetPermanentGroupBaseStats {
        filters: &'static [BattlefieldFilter],
        exclude_source: bool,
        power: Option<ResolvedStatValue>,
        toughness: Option<ResolvedStatValue>,
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
    /// Apply a sampled expression as a nonnegative growth multiplier (CR107.1b).
    GrowResolved {
        subject: Subject,
        value: ResolvedStatValue,
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
    /// Permanent additive creature subtypes on the exact selected subjects.
    AddCreatureTypes {
        subject: Subject,
        types: &'static [&'static str],
    },
    /// A lasting type-changing effect, not a grant of changeling.
    GrantAllCreatureTypes(Subject),
    /// Add a basic land subtype to the selected exact land incarnations.
    #[allow(
        dead_code,
        reason = "Shared rule remains gated until a complete card consumes it"
    )]
    AddBasicLandType {
        subject: Subject,
        kind: BasicLandType,
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
    /// Grant to the exact, non-target creature subjects frozen by a cast event.
    GrantCastCreatureKeywords(Keywords),
    /// Bestow a real self-owned static ability, rather than a resolved stat modifier.
    GrantSelfAbility(crate::engine::SelfGrantedAbilityKind),
    GrantPermanentSelfAbility(crate::engine::SelfGrantedAbilityKind),
    /// Grant the exact source a triggered template until it changes zones.
    GrantSourcePermanentTrigger(u16),
    /// Grant a triggered ability to this exact source until cleanup.
    GrantSourceTriggerUntilEndOfTurn(u16),
    /// Grant keywords only to targets cast for their warp cost.
    GrantWarpedTargetsKeywords(Keywords),
    /// Register the exact targets for earthbend's delayed return.
    RegisterEarthbendReturn,
    AnimateEarthbendTargets,
    GrantKeywords {
        subject: Subject,
        keywords: Keywords,
    },
    /// Freeze a union of current battlefield recipients, including noncreatures.
    GrantPermanentGroupKeywords {
        filters: &'static [BattlefieldFilter],
        keywords: Keywords,
    },
    GrantCreatures {
        filter: CreatureFilter,
        keywords: Keywords,
    },
    GrantRememberedCreaturesUntilNextTurn(Keywords),
    /// Freeze current creatures controlled by the selected players until cleanup.
    GrowPlayerCreatures {
        players: PlayerSet,
        power: i32,
        toughness: i32,
    },
    GrowCreatures {
        filter: CreatureFilter,
        power: i32,
        toughness: i32,
    },
    /// Freeze a non-targeted permanent set and one scalar at resolution.
    GrowPermanentGroup {
        filters: &'static [BattlefieldFilter],
        power: i32,
        toughness: i32,
    },
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
    /// Counter; a successfully exiled permanent card gains a lasting free cast.
    CounterPermanentExileCast,
    CounterAbilityAndRemoveSourceAbilities,
    RemoveAbilitiesThenDestroyCreaturesManaAtMost(AmountValue),
    Exile,
    /// Exile valid targets and grant each actual exiled card's owner a {2} alternate cast.
    AirbendTargets,
    /// Airbend all creatures except the selected exact target, if any.
    AirbendOtherCreatures,
    /// Opponents may cast only from their hands until this controller's next turn.
    ProhibitOpponentNonhandCastsUntilNextTurn,
    ExileTargetsForModifiedCast,
    ExileTargetsPlay {
        next_turn: bool,
    },
    ExileTargetsAndTopPlayForOwners,
    ExileUntilSourceLeaves,
    ExileBattlefieldUntilSourceLeaves {
        filters: &'static [BattlefieldFilter],
    },
    /// Count only linked cards actually moved to a graveyard, then create tokens.
    LinkedExileToGraveyardTokens {
        token: Card,
    },
    /// Freeze copiable values before a successful linked exile; wait for immediate returns.
    ExileAndCaptureAttachedCopy,
    /// Apply the captured blueprint to the Aura's current exact attached creature.
    ApplyCapturedAttachedCopy,
    ExileGraveyard(PlayerSet),
    ReturnToHand,
    /// Return selected permanents, skipping a following block if none actually return.
    ReturnToHandOrSkip {
        skip: usize,
    },
    /// Move all legal target groups together, deduplicating exact incarnations.
    ReturnAllTargetsToHand,
    /// Return every creature lacking the named subtype in one simultaneous batch.
    ReturnCreaturesWithoutSubtype(&'static str),
    /// Freeze successful returns, then one token-creation trigger next upkeep.
    ReturnTargetsCreateNextUpkeep {
        token: Card,
    },
    ResolveDelayedTokenCreation,
    /// Capture a legal graveyard target's subtype, return it, then draw for a match.
    ReturnToHandDrawForSubtype(&'static str),
    BottomLibrary,
    TopLibrary,
    AdditionalLandPlays(u64),
    OwnerLibrary,
    /// Its owner chooses top or bottom; successful top movement deals damage.
    OwnerLibraryTopOrBottomDamage(u16),
    OwnerLibrarySecondFromTop,
    OwnerLibraryNthFromTop(u16),
    /// Put the exact live source in its owner’s library below its top two cards.
    PutSourceThirdFromTop,
    ChooseManaValueParityDestroyOthers,
    /// Shuffle even when the exact source permanent has left (CR701.24c).
    ShuffleSourceIntoOwnerLibrary,
    ShuffleTargetIntoLibrary,
    /// Shuffle the live source and valid target into their owners' libraries together.
    ShuffleSourceAndTargetIntoLibraries,
    ShuffleGraveyardIntoLibrary,
    DoubleTargetPower,
    DrawAndProtectCounterCreatures,
    MultiplyTargetStats {
        factor: u8,
    },
    ReturnToBattlefield,
    ReturnToBattlefieldUnderControl,
    /// Return under your control; grant attack-each-combat and a permanent triggered template.
    ReturnToBattlefieldWithAttackTrigger(u16),
    /// Return legal target cards with an additional entry counter.
    ReturnToBattlefieldWithCounter(CounterKind),
    ReturnToBattlefieldTapped,
    ReturnTargetAttackingWithCounter(CounterKind),
    Destroy,
    Tap,
    GainControlTargetsUntilSourceLeaves,
    /// Linked control plus combat prohibition, ending irreversibly when source control is lost.
    GainControlAndPreventCombatWhileSourceControlled,
    RemoveTargetAbilitiesUntilSourceLeaves,
    RemoveTargetAbilitiesUntilEndOfTurn,
    GrantTargetHexproofUntilSourceLeaves,
    DamageEachPlayerPerNonbasicLand,
    PreventTargetDamageUntilSourceLeaves,
    PreventTargetUntapWhileSourceControlled,
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
    PutCreatureCountersAndRemember {
        filter: CreatureFilter,
        kind: CounterKind,
        amount: u32,
    },
    DoubleCounters,
    /// Add exactly one of every currently present kind to a permanent or player.
    AddOneOfEachCounterKind,
    /// Give the exact source keyword counters only for current target keywords it lacks.
    GainMissingTargetKeywordCounters,
    DoubleCounterKind {
        subject: Subject,
        kind: CounterKind,
    },
    ExileGraveTargets,
    ExileGraveTargetsLinked,
    ReturnLinkedGhosts,
    IfExiledCreature {
        skip: usize,
    },
    /// Repeat this non-targeting counter event a frozen number of times.
    Proliferate(AmountValue),
    RemoveAllCounters,
    RemoveAllCountersFrom(Subject),
    SnapshotDoubleCreatures(CreatureFilter),
    PutCounters {
        subject: Subject,
        kind: CounterKind,
        amount: u32,
    },
    /// Place counters simultaneously on the current controlled tapped creatures.
    PutCountersOnTappedControlledCreatures {
        kind: CounterKind,
        amount: u32,
    },
    /// Place counters on surviving targets that currently have any specified kind.
    /// Put counters only on legal targets in the selected target group.
    PutCountersOnTargetGroup {
        group: u16,
        kind: CounterKind,
        amount: u16,
    },
    PutCountersOnTargetKinds {
        kinds: u16,
        kind: CounterKind,
        amount: u32,
    },
    PutCountersOnTargetAllKinds {
        kinds: u16,
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
    /// Public resolution choice followed by one current controlled-type count.
    /// Choose a creature type, then destroy every creature without it.
    DestroyCreaturesExceptChosenType,
    CreateTokensForChosenCreatureType {
        token: Card,
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
    /// Choose and remove up to this many counters from one targeted permanent.
    RemoveUpToTargetCounters(u16),
    /// Optional non-targeted source-counter removal; an actual removal queues the named reflexive trigger.
    MayRemoveAnySourceCounterReflexive(u16),
    CreateTokensWithCounters {
        token: Card,
        amount: u16,
        kind: CounterKind,
        counters: AmountValue,
    },
    /// If cast from a graveyard, count its controller's graveyard after creation.
    CreateTokensWithGraveyardGroups {
        token: Card,
        amount: u16,
        group: std::num::NonZeroU16,
    },
    /// Create first, then attach every exact resulting Equipment to this source.
    CreateEquipmentForSource {
        token: Card,
        amount: u16,
    },
    /// Create a token, then put counters on other controlled Wizard tokens.
    CreateTokenThenCounterOtherWizards {
        token: Card,
    },
    /// Actual created tokens must attack only the current combat if able.
    CreateTokensAttackThisCombat {
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
    /// Draw greatest controlled creature power and remember successful draws.
    DrawGreatestCreaturePowerRemember,
    /// Lose life for the successful draws in the preceding remembered draw.
    LoseLifeRememberedDraws,
    AddCastEntryCounters {
        kind: CounterKind,
        amount: AmountValue,
    },
    /// Draw, discard, and create a Human Soldier if a nonland was discarded.
    Recruit {
        token: Card,
    },
    DrawThenDiscardIfDrew {
        amount: AmountValue,
        discard: u16,
    },
    Discard(PlayerSet, u16),
    /// Choose uniformly without replacement, then discard the batch.
    DiscardRandom(PlayerSet, u16),
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
    /// Untargeted loss followed by a gain of the life actually lost.
    DrainOpponents(u16),
    CombatDamageByGreaterToughnessThisTurn,
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
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
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
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum LiveStatValue {
    Fixed(i128),
    ControllerLife,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct LiveStatCondition {
    pub subtype: &'static str,
    pub negated: bool,
}
/// Immutable complete continuous-effect clause. Values and membership remain live.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
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
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Stats {
    pub power: i128,
    pub toughness: i128,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl From<i128> for Stats {
    fn from(value: i128) -> Self {
        Self {
            power: value,
            toughness: value,
        }
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
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

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CountZone {
    Battlefield,
    Graveyard,
    Exile,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CountFilter {
    pub zone: CountZone,
    pub relation: i8,
    pub kind: u16,
    pub exclude: u16,
    pub subtype: Option<&'static str>,
    pub basic: bool,
    pub other: bool,
    pub cards_only: bool,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AmountValue {
    TappedCostCreaturePower,
    OpponentCreaturesExiledThisTurn,
    ControllerHandDeficit(u16),
    /// Current hand size of the spell's surviving targeted player.
    TargetPlayerHandSize,
    /// Successful library-to-graveyard transitions for the surviving target this turn.
    TargetPlayerLibraryToGraveyardThisTurn,
    /// Current creatures controlled by the surviving player in the selected target group.
    TargetPlayerCreatureCount,
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
    /// Damage dealt by the equipped creature in one completed combat batch.
    CreatureCombatDamageAmount,
    #[allow(
        dead_code,
        reason = "Core quantity is scenario-tested while its consumer cards remain gated"
    )]
    SourceCounterQuantity(CounterKind),
    /// All counter kinds on the exact source, using departure LKI after it leaves.
    SourceTotalCounters,
    /// Frozen successful source-counter cost; never the source current counter count.
    PaidSourceCounterQuantity(CounterKind),
    /// Frozen before the selected sacrifice changes any battlefield object.
    SacrificedPower,
    /// Power saved on a successful sacrifice's separate reflexive packet.
    ReflexiveSacrificedPower,
    /// Frozen count bounding a reflexive trigger’s optional targets.
    ReflexiveTargetLimit,
    /// Current power frozen immediately before paying a sacrifice additional cost.
    SacrificeCostPower,
    /// Frozen at the beginning of resolution, before target destruction.
    TargetCreaturePower,
    /// Exact pre-event source facts for a death trigger, never a new incarnation.
    SourceLastKnownPower,
    SacrificedToughness,
    Fixed(u16),
    /// Explicit immutable parameter of the resolving spell or its exact ETB packet.
    ChosenX,
    MilledCreatures,
    Stat(StatValue),
    LifeGainedThisTurn,
    LifeEventAmount,
    CardsDiscardedThisTurn,
    /// Actual cards in this packet's one completed simultaneous discard event.
    DiscardEventAmount,
    /// Actual result of the preceding remembered zone-move instruction.
    MovedCount,
    /// Actual completed departure events of this packet's additional cost.
    PaidAdditionalCostCount,
    /// Chosen cardinality for an X nonmana cost, retained even if replaced.
    PaidAdditionalCostSelectionCount,
    Multiple {
        value: &'static AmountValue,
        factor: u16,
    },
    Divided {
        value: &'static AmountValue,
        divisor: std::num::NonZeroU16,
    },
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResolvedStatValue {
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
    /// Signed current power of the exact source, or its departure LKI.
    SourcePower,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceCharacteristicFilter {
    Color(ChoiceColor),
    Instant,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StatValue {
    /// Number of different planeswalker types among planeswalkers controlled by the source controller.
    DistinctControlledPlaneswalkerTypes,
    /// The source has not dealt combat damage in its current incarnation.
    SourceHasNotDealtCombatDamage,
    ControlledCreatureDiedThisTurn,
    CardLeftGraveyardThisTurn,
    Fixed(u16),
    TappedCount(CountFilter),
    Not(&'static StatValue),
    CardsDrawnThisTurn,
    PermanentsSacrificedThisTurn,
    ControllerHandSize,
    /// Number of distinct basic land types among lands the controller controls.
    ControllerDomain,
    NonlandEntriesThisTurn,
    MountOrVehicleEntriesThisTurn,
    SourceAttacking,
    AttackingCreatures,
    AttackingSubtype(&'static str),
    OtherNamedCreatures(&'static str),
    /// Unqualified all-player count; distinct from controller-owned attackers.
    AllAttackingCreatures,
    SourceModified,
    SourceEquipmentCount,
    LegendaryCreaturesYouControl,
    HasEnduringStory,
    SourceCounters(CounterKind),
    /// Total across hot and named counter kinds on the exact source.
    SourceTotalCounters,
    /// A boolean parity predicate; zero is even.
    Parity {
        value: &'static StatValue,
        odd: bool,
    },
    Tokens(CountFilter),
    AtLeast {
        value: &'static StatValue,
        minimum: u16,
    },
    SpellsCastThisTurn,
    CreatureSpellsCastThisTurn,
    /// Count of this player's committed noncreature spell casts this turn.
    NoncreatureSpellsCastThisTurn,
    LifeGainOccurredThisTurn,
    /// Whether an opponent has actually been dealt noncombat damage this turn.
    OpponentNoncombatDamageThisTurn,
    LifeLostThisTurn,
    LinkedExileCardTypes,
    CreaturesDiedThisTurn,
    LifeGainedThisTurn,
    ControllerLife,
    /// Persistent player designation; zero means the controller has no speed.
    ControllerSpeed,
    /// Compare current land counts using the same actual or speculative world.
    OpponentControlsMoreLands,
    /// Whether the opposing player's life is at most this threshold.
    OpponentLifeAtMost(u16),
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
    TotalManaValue(CountFilter),
    DistinctNames(CountFilter),
    DistinctManaValues(CountFilter),
    CountUnion(&'static [CountFilter]),
    Colors(CountFilter),
    CardTypes(CountFilter),
    GraveyardCardTypes,
    /// Number of complete positive-sized groups in a nonnegative quantity.
    GroupsOf {
        value: &'static StatValue,
        size: std::num::NonZeroU16,
    },
    Sum(&'static [StatValue]),
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StatExpr {
    pub value: StatValue,
    pub multiplier: i128,
    pub offset: i128,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BaseStatSetter {
    pub power: Option<i128>,
    pub toughness: Option<i128>,
    pub own_turn: bool,
    pub condition: Option<StatValue>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DynamicStats {
    pub power: Option<StatExpr>,
    pub toughness: Option<StatExpr>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScalingModifier {
    pub subject: Subject,
    pub value: StatValue,
    pub stats: Stats,
}

/// Predicates depend only on earlier-layer characteristics and combat membership.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CreatureFilter {
    pub relation: i8,
    pub other: bool,
    pub subtype: Option<&'static str>,
    pub keyword: Keywords,
    pub attacking: bool,
    pub tokens: Option<bool>,
    pub artifact: bool,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
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
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeywordGrant {
    pub condition: Option<StatValue>,
    pub requires_life_gain: bool,
    pub filter: CreatureFilter,
    pub source_only: bool,
    pub own_turn: bool,
    pub keywords: Keywords,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
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

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StaticModifier {
    pub own_turn: bool,
    pub requires_life_gain: bool,
    pub filter: CreatureFilter,
    pub stats: Stats,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EnchantedModifier {
    pub filter: CreatureFilter,
    pub stats: Stats,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EquipmentSpec {
    pub stats: Stats,
    pub keywords: Keywords,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AuraSpec {
    /// Enchant object-kind union, evaluated using live characteristics.
    /// Zero means any permanent, including Battles outside the narrow mask.
    pub kinds: u8,
    pub relation: i8,
    pub bonus: EquipmentSpec,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Subject {
    /// Exact triggering event object, including entrants and declared attackers.
    Entering,
    Source,
    Targets,
    Attached,
}

/// Immutable +1/+1 counter replacements evaluated against the pre-entry world.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntryCounterAmount {
    ControlledCreatureDeaths,
    GreatestOtherCreaturePower,
    ColorsSpent,
    IfNotCastOrZero(u16),
    Fixed(u16),
    Stat(StatValue),
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EntryCounterRule {
    pub guard: StepGuard,
    pub amount: EntryCounterAmount,
}

/// Immutable replacement programs; active games retain only exact event/source identities.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReplacementFilter {
    /// A creature whose current power is at most this signed threshold.
    #[cfg_attr(not(test), allow(dead_code))]
    CreaturePowerAtMost(i128),
    Permanent,
    Creature,
    CreatureOrVehicle,
    CreatureSpacecraftPlanet,
    BattleProtectedByOpponent,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReplacementSpec {
    /// Draw one card before the original creature connive proceeds.
    ConniveDraw,
    /// Add one of each positive kind, only for the provider controller placing
    /// counters on their own permanent. General providers do not apply entering.
    CountersPlacedByController,
    DoubleCreatureDamage,
    DoubleOpponentDamage,
    NoncombatDamagePlusSourcePower,
    /// Add a fixed amount to controlled-source noncombat damage to opponents or their permanents.
    NoncombatDamageToOpponentPlus(u16),
    /// Add to any controlled-source damage to an opposing player or permanent.
    ConditionalControlledDamageToOpponentPlus {
        condition: StatValue,
        amount: u16,
    },
    DoubleEquippedDamage,
    HealPreviousDamageToSelf,
    PreventOtherCreatureNoncombatDamage,
    PreventAllDamageToSelf,
    Counters {
        filter: ReplacementFilter,
        kind: Option<CounterKind>,
        factor: u32,
        add: u32,
        effects_only: bool,
    },
    /// Replace an outside-card admission. The outside card has no game object
    /// until the resulting event is committed.
    #[allow(dead_code, reason = "Outside admission scheduler is not yet exposed")]
    OutsideEntry {
        relation: CastRelation,
        from: OutsideEntryEffectDestination,
        to: OutsideEntryEffectDestination,
    },
    /// Replace a proliferate event by a number of events for the related player.
    #[allow(dead_code, reason = "Gated typed proliferate foundation")]
    Proliferate {
        relation: CastRelation,
        factor: u32,
        add: u32,
    },
    /// Replace counters placed on a player related to the source controller.
    #[allow(dead_code, reason = "Gated typed proliferate foundation")]
    PlayerCounters {
        kind: PlayerCounterKind,
        relation: CastRelation,
        /// Relation of the player placing the counter to the provider controller.
        placed_by: Option<CastRelation>,
        effects_only: bool,
        factor: u32,
        add: u32,
    },
    DoubleTokens {
        effects_only: bool,
    },
    AddToken(Card),
    ReplaceArtifactTokens(Card),
    AddArtifactToken(Card),
}

/// Destinations used by an outside-card admission replacement. Prevention is
/// terminal, so it cannot be the input of a subsequent replacement.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutsideEntryEffectDestination {
    Hand,
    Exile,
    #[allow(
        dead_code,
        reason = "Only synthetic outside-entry prevention providers exist"
    )]
    Prevent,
}

/// A replacement belonging to the resolving outside-admission instruction.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SelfOutsideEntryTemplate {
    pub from: OutsideEntryEffectDestination,
    pub to: OutsideEntryEffectDestination,
}

/// One descendant event produced by a split player-counter replacement.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct PlayerCounterSplit {
    pub to_opponent: bool,
    pub kind: Option<PlayerCounterKind>,
    pub factor: u32,
    pub add: u32,
}

/// One typed same-effect counter replacement. Redirects are relative to each
/// affected player; absence retains the original player or counter kind.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SelfPlayerCounterTemplate {
    pub factor: u32,
    pub add: u32,
    pub redirect_to_opponent: bool,
    pub redirect_kind: Option<PlayerCounterKind>,
    pub split: &'static [PlayerCounterSplit],
}

/// Player counters currently represented by the game state.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
pub enum PlayerCounterKind {
    Poison,
    Energy,
    Experience,
    Rad,
    Ticket,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl PlayerCounterKind {
    pub const ALL: [Self; 5] = [
        Self::Poison,
        Self::Energy,
        Self::Experience,
        Self::Rad,
        Self::Ticket,
    ];
    pub(crate) fn index(self) -> usize {
        self as usize
    }
}

/// Card-type bit reserved for battles.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(crate) const BATTLE_KIND: u16 = 256;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
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
    Shield,
    Hone,
    Quest,
    Defense,
    Hexproof,
    Blight,
    Bloodstain,
    Bore,
    Chorus,
    Coin,
    Conqueror,
    Decayed,
    Dread,
    Dream,
    Film,
    Fire,
    Flood,
    Impostor,
    Ingenuity,
    Landmark,
    Loot,
    Nest,
    Net,
    Point,
    Possession,
    Rev,
    Skewer,
    Supply,
    Time,
    Unlock,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl CounterKind {
    /// Stable allocation order for mixed counter costs; existing kinds stay first.
    pub const ALL: [Self; 60] = [
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
        Self::Shield,
        Self::Hone,
        Self::Quest,
        Self::Defense,
        Self::Hexproof,
        Self::Blight,
        Self::Bloodstain,
        Self::Bore,
        Self::Chorus,
        Self::Coin,
        Self::Conqueror,
        Self::Decayed,
        Self::Dread,
        Self::Dream,
        Self::Film,
        Self::Fire,
        Self::Flood,
        Self::Impostor,
        Self::Ingenuity,
        Self::Landmark,
        Self::Loot,
        Self::Nest,
        Self::Net,
        Self::Point,
        Self::Possession,
        Self::Rev,
        Self::Skewer,
        Self::Supply,
        Self::Time,
        Self::Unlock,
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
                | Self::Shield
                | Self::Hone
                | Self::Quest
                | Self::Defense
                | Self::Hexproof
                | Self::Blight
                | Self::Bloodstain
                | Self::Bore
                | Self::Chorus
                | Self::Coin
                | Self::Conqueror
                | Self::Decayed
                | Self::Dread
                | Self::Dream
                | Self::Film
                | Self::Fire
                | Self::Flood
                | Self::Impostor
                | Self::Ingenuity
                | Self::Landmark
                | Self::Loot
                | Self::Nest
                | Self::Net
                | Self::Point
                | Self::Possession
                | Self::Rev
                | Self::Skewer
                | Self::Supply
                | Self::Time
                | Self::Unlock
        )
    }
}

/// Common live fields and catalogue initializers remain compact Copy values.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InitialCounters {
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
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct NamedCounters(pub(crate) Vec<(CounterKind, u64)>);
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
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
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
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
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn is_zero_counter(value: &u64) -> bool {
    *value == 0
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
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
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Default)]
pub struct CounterAmounts<'a> {
    pub hot: InitialCounters,
    pub named: Option<&'a NamedCounters>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl<'a> From<&'a Counters> for CounterAmounts<'a> {
    fn from(value: &'a Counters) -> Self {
        Self {
            hot: value.hot(),
            named: value.named.as_deref(),
        }
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
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
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
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

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CastTarget {
    Creature(CastRelation),
    OpponentOrCreature,
    PermanentKinds(u16),
    Source,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CastFilter {
    /// A spell cast from an actual prepared card copy.
    PreparedSpell,
    NotOwnedByYou,
    /// Physical ownership is compared to the actual caster, not the observer.
    NotOwnedByCaster,
    /// Event qualification against the actual caster's turn, never the observer's.
    CasterOffTurn {
        filter: &'static CastFilter,
    },
    HasX,
    SourceChosenColor,
    SourceChosenManaValueParity,
    PermanentAdventure,
    Any,
    Colorless,
    Creature,
    Noncreature,
    /// Historical qualifying cast ordinal, evaluated only with the actual caster.
    InstantSorceryOrdinal(u64),
    /// Actual per-caster ordinal, including casts before the observer entered.
    NonCreatureOrdinal(u64),
    InstantSorcery,
    Sorcery,
    Artifact,
    Planeswalker,
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
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl CastFilter {
    pub fn matches(self, card: Card) -> bool {
        self.matches_mana_value(card, card.fixed_mana_value())
    }
    pub(crate) fn matches_mana_value(self, card: Card, mana_value: u64) -> bool {
        match self {
            Self::Any => true,
            Self::Colorless => card.colors() == 0,
            Self::HasX => card.x_symbols() > 0,
            Self::PermanentAdventure => {
                card.adventure_spell().is_some()
                    && card.kind_mask() & (1 | 2 | 16 | 32 | 64 | 256) != 0
            }
            // Contextual cast observers use Game::cast_filter_matches instead.
            Self::PreparedSpell
            | Self::NotOwnedByYou
            | Self::NotOwnedByCaster
            | Self::CasterOffTurn { .. }
            | Self::SourceChosenColor
            | Self::SourceChosenManaValueParity
            | Self::Targeted { .. }
            | Self::InstantSorceryOrdinal(_)
            | Self::NonCreatureOrdinal(_) => false,
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
            Self::Planeswalker => card.kind_mask() & 64 != 0,
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
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CostTurn {
    Any,
    /// Cast from the caster's graveyard or from exile.
    GraveyardOrExile,
    /// Historical origin is not this caster’s hand (independent of payment).
    NonHand,
    // Parsed vocabulary; all complete own-turn bodies still have other dependencies.
    #[allow(dead_code)]
    Own,
    Other,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpellReductionAmount {
    /// Graveyard cards that are instant or sorcery cards or have an Adventure.
    GraveyardInstantSorceryAdventure,
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
    GreatestCreatureToughness,
    EquippedPower,
}
/// Permission belonging to the chosen spell, checked before costs (CR601.6a).
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelfCastTiming {
    ControlSubtype(&'static str),
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpellReduction {
    pub source_only: bool,
    pub filter: CastFilter,
    pub amount: SpellReductionAmount,
    pub turn: CostTurn,
    pub guard: StepGuard,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CastRelation {
    You,
    // Complete-text gating currently excludes the corresponding printed cards.
    #[allow(dead_code)]
    Opponent,
    #[allow(dead_code)]
    Any,
}

// The compiler has exact spellings for this condition vocabulary. Some values
// remain unused until the rest of the corresponding card text is implemented.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepValue {
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
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CombatRestriction {
    pub subject: Subject,
    pub attack: bool,
    pub block: bool,
    pub unless: Option<StepGuard>,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepGuard {
    SourceUnprepared,
    ClassLevelAtLeast(u8),
    UnsolvedLifeGainedAtLeast(u128),
    /// Trigger-time attachment relation; wrap the step event in When.
    AttachedControllerTurn,
    ArtifactEnteredThisTurn,
    HeroEnteredOrAttackedThisTurn,
    CardExiledThisTurn,
    OpponentHasMoreCardsInHand,
    OpponentControlsMoreCreatures,
    SourceReceivedCounterThisTurn,
    YourTurn,
    YourMainPhase,
    SourcePowerAtLeast(u16),
    SourceCreature,
    SourceToughnessAtMost(u16),
    Outlaw,
    Always,
    Attacked,
    /// Historical declaration facts; one attacker may satisfy several subtypes.
    DeclaredAttackSubtypesAll(&'static [&'static str]),
    CreatureDied,
    OpponentLostLife,
    CommittedCrimeThisTurn,
    GainedLife,
    ScriedOrSurveilledThisTurn,
    LoyaltyAbilityActivatedThisTurn,
    OpponentNoncombatDamageLastTurn,
    GainedAndLostLife,
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
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TriggerStep {
    Upkeep,
    Draw,
    Main1,
    Main2,
    MainPhases,
    BeginCombat,
    EndCombat,
    End,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DamageQualification {
    Any,
    PowerAboveBase,
    Modified,
    Counter,
    PlusOne,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DamageSubject {
    /// Damage from any source, combat or noncombat.
    AnySources,
    AnyNoncombatSources,
    ControlledNoncombatSources,
    Source,
    Attached,
    Creatures {
        filter: CreatureFilter,
        qualification: DamageQualification,
    },
}

/// Complete entry-time predicates over the post-event permanent.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntryPredicate {
    /// A predicate on each actual entrant, distinct from batch token-entry events.
    Token,
    LegendarySubtype(&'static str),
    CreatureWithoutSubtype(&'static str),
    ArtifactManaValueAtLeast(u32),
    CreaturePowerAtMost(i32),
    CreaturePowerAtLeast(i32),
    SubtypeAny(&'static [&'static str]),
    KindOrSubtype {
        kinds: u16,
        subtype: &'static str,
    },
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReturnDeathFilter {
    SourceNonDemon,
    SourceRevival,
    ControlledNontokenNonAngel,
    ControlledSubtype(&'static str),
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReturnKind {
    Demon,
    Angel,
    Revival,
    HeroFinality,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TriggerCondition {
    /// The controller completed a positive scry or surveil instruction.
    ScryOrSurveil,
    /// One event for each completed proliferate, even when no object was chosen.
    #[allow(dead_code, reason = "Gated typed proliferate foundation")]
    Proliferated(CastRelation),
    /// First declaration in the turn whose signed creature-power total reaches this threshold.
    FirstAttackWithTotalPowerAtLeast(u16),
    /// Pre-event observer and other permanent, with an actual owner-hand result.
    OtherControlledNonlandReturnedToOwnersHand,
    /// Global target acquisition, not a cast-completion or self-target event.
    ControlledAbilityTargetsPlayerOrPermanent,
    OpponentTargetsControlledCreatureOrSpell,
    /// The physical source spell itself was cast; stack copies never dispatch it.
    SelfCast,
    /// The controller actually activates an ability of a creature permanent.
    CreatureAbilityActivated,
    /// The controller activated a nonmana ability.
    NonmanaAbilityActivated,
    /// A committed exhaust activation, including mana abilities.
    ExhaustAbilityActivated,
    /// Source-independent next nonmana exhaust activation until cleanup.
    DelayedExhaustAbilityActivated,
    /// Template dispatched only by an owned delayed target-event registration.
    DelayedTargetEvent,
    /// The controller committed an activation of a loyalty ability.
    LoyaltyAbilityActivated,
    /// Committed loyalty activation paid at least this many loyalty counters.
    LoyaltyAbilityActivatedRemoving(u64),
    /// An opponent committed a loyalty activation.
    OpponentLoyaltyAbilityActivated,
    CardsLeaveGraveyard,
    /// One simultaneous departure batch containing a physical card of these kinds.
    CardsLeaveGraveyardKinds(u16),
    /// One physical-card batch from either battlefield or graveyard to exile.
    CardsExiledFromBattlefieldOrGraveyard,
    /// One trigger per actual creature CARD departing its owner's graveyard.
    CreatureCardLeavesYourGraveyard,
    /// One post-event batch of creature cards entering the controller's graveyard.
    CreatureCardsEnterYourGraveyard,
    /// A whole event batch put one or more owned permanent cards into a graveyard.
    PermanentCardsEnterYourGraveyard,
    /// One trigger for a batch exiling controlled creatures or cards from your graveyard.
    ControlledCreaturesOrGraveyardCardsExiled,
    /// This permanent was exiled from the battlefield while its controller
    /// was activating a craft ability.
    SelfExiledWhileCrafting,
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
    TreasureManaSpent(&'static TriggerCondition),
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
    /// This creature was tapped to pay a committed Crew or Saddle cost.
    SelfCrewsOrSaddles,
    SelfPaysTeamwork,
    /// A paid promise belongs to this actual cast-to-battlefield entry event.
    SelfGiftEnters,
    SelfEntersIfCast,
    Foraged,
    FoodsSacrificed,
    Reflexive,
    /// A cast-event qualification, not an intervening-if resolution guard.
    DuringOpponentsTurn(&'static TriggerCondition),
    /// Cast-event origin qualification, independent of spell payment method.
    CastFromHand(&'static TriggerCondition),
    /// This cast originated in exile, as recorded at announcement.
    CastFromExile(&'static TriggerCondition),
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
    /// Completion of a library search, including a search that finds no card.
    LibrarySearched {
        relation: CastRelation,
    },
    /// This exact permanent has actually changed to the specified Class level.
    BecomesClassLevel(u8),
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
    /// One trigger per qualifying recipient in a placement event, irrespective of counter kinds.
    CounterPlacedByYouOnControlledSubtypes {
        subtypes: &'static [&'static str],
    },
    /// Look back at another controlled permanent's characteristics on death.
    OtherControlledPermanentWithSubtypeDies {
        subtypes: &'static [&'static str],
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
    /// An opponent sacrifices a permanent matching its pre-departure characteristics.
    SacrificedByOpponent {
        filter: SacrificeFilter,
    },
    CreatureDiesWithMaximumStat {
        relation: CastRelation,
        maximum: i32,
    },
    /// One observer trigger for a simultaneous batch containing a matching death.
    CreatureDiesBatch {
        filter: CreatureFilter,
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
    /// A registered next-spell callback, never a passive battlefield observer.
    DelayedSpellCastSourcePower,
    SpellCast {
        relation: CastRelation,
        filter: CastFilter,
        ordinal: Option<u64>,
    },
    SelfEnters,
    SelfOffspring,
    SelfKicked,
    CreatureExploresYou,
    // Only the gated observer fixture currently constructs these triggers.
    #[cfg_attr(not(test), allow(dead_code))]
    SelfTurnedFaceUp,
    #[cfg_attr(not(test), allow(dead_code))]
    PermanentYouControlTurnedFaceUp,
    YouManifestDread,
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
    /// Frozen source was a creature immediately before its actual death.
    SelfCreatureDies,
    /// Token status is immutable for an incarnation, including through source LKI.
    SelfNontokenEnters,
    SelfLeaves,
    SelfEntersOrDies,
    SelfAttacks,
    /// Intervening equipped clause, with exact source-departure quantities.
    SelfAttacksIfEquipped,
    SelfFirstAttack,
    CreatureAttacksYou {
        colors: u8,
    },
    /// Once per declaration, capturing only creatures declared attacking this controller as a player.
    CreaturesAttackYou,
    /// Declaration-time condition; not a resolution-time intervening if.
    SelfAttacksAlone,
    /// A completed declaration with exactly one controlled creature, whether this source attacked or not.
    ControlledCreatureAttacksAlone,
    /// One event for each creature declared attacking by the source controller.
    ControlledCreatureAttacks,
    ControlledCreatureAttacksPlayerAlone,
    /// Once per opponent-controlled blocker meeting either stat ceiling.
    OpponentCreatureBlocksSmall,
    ControlledEquippedCreatureAttacks,
    EquippedAttacks,
    /// Attachment recipient was the sole creature declared in this attack event.
    EquippedAttacksAlone,
    /// One event per player attacked by at least one declared matching creature.
    AttackPlayer(CreatureFilter),
    GrantedAttacks,
    AttachedAttacksOrBlocks,
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
    /// The entrant must match, and the intervening threshold excludes its exact incarnation.
    SubtypeEntersYouWithOthers {
        subtype: &'static str,
        minimum: u16,
    },
    ArtifactEntersYou,
    EnchantmentEntersYou,
    CreatureEntersYou,
    OtherArtifactEntersYou,
    LandEntersYou,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl TriggerCondition {
    pub(crate) fn requires_treasure_mana(self) -> bool {
        match self {
            Self::TreasureManaSpent(_) => true,
            Self::InZone { event, .. }
            | Self::Granted(event)
            | Self::OncePerTurn(event)
            | Self::DuringOpponentsTurn(event)
            | Self::CastFromHand(event)
            | Self::CastFromExile(event)
            | Self::ManaSpent { event, .. }
            | Self::When { event, .. }
            | Self::InterveningIf { event, .. } => event.requires_treasure_mana(),
            _ => false,
        }
    }
    pub(crate) fn requires_hand_cast(self) -> bool {
        match self {
            Self::CastFromHand(_) => true,
            Self::InZone { event, .. }
            | Self::Granted(event)
            | Self::OncePerTurn(event)
            | Self::DuringOpponentsTurn(event)
            | Self::CastFromExile(event)
            | Self::TreasureManaSpent(event)
            | Self::ManaSpent { event, .. }
            | Self::When { event, .. }
            | Self::InterveningIf { event, .. } => event.requires_hand_cast(),
            _ => false,
        }
    }
    pub(crate) fn requires_exile_cast(self) -> bool {
        match self {
            Self::CastFromExile(_) => true,
            Self::InZone { event, .. }
            | Self::Granted(event)
            | Self::OncePerTurn(event)
            | Self::DuringOpponentsTurn(event)
            | Self::CastFromHand(event)
            | Self::TreasureManaSpent(event)
            | Self::ManaSpent { event, .. }
            | Self::When { event, .. }
            | Self::InterveningIf { event, .. } => event.requires_exile_cast(),
            _ => false,
        }
    }
    pub(crate) fn origin(self) -> Option<TargetZone> {
        match self {
            Self::InZone { zone, .. } => Some(zone),
            Self::Granted(event)
            | Self::OncePerTurn(event)
            | Self::DuringOpponentsTurn(event)
            | Self::CastFromHand(event)
            | Self::CastFromExile(event)
            | Self::TreasureManaSpent(event)
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
            | Self::CastFromHand(e)
            | Self::CastFromExile(e)
            | Self::TreasureManaSpent(e)
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
            | Self::CastFromHand(e)
            | Self::CastFromExile(e)
            | Self::When { event: e, .. }
            | Self::InterveningIf { event: e, .. }
            | Self::TreasureManaSpent(e)
            | Self::ManaSpent { event: e, .. } => e.is_granted(),
            _ => false,
        }
    }
    pub(crate) fn requires_opponents_turn(self) -> bool {
        match self {
            Self::DuringOpponentsTurn(_) => true,
            Self::InZone { event, .. }
            | Self::CastFromHand(event)
            | Self::CastFromExile(event)
            | Self::Granted(event)
            | Self::OncePerTurn(event)
            | Self::TreasureManaSpent(event)
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
            | Self::CastFromHand(event)
            | Self::CastFromExile(event)
            | Self::When { event, .. }
            | Self::InterveningIf { event, .. }
            | Self::TreasureManaSpent(event)
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
            | Self::TreasureManaSpent(event)
            | Self::ManaSpent { event, .. }
            | Self::DuringOpponentsTurn(event)
            | Self::CastFromHand(event)
            | Self::CastFromExile(event) => event.once_per_turn(),
            _ => false,
        }
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TriggeredAbility {
    pub condition: TriggerCondition,
    pub targets: TargetPlan,
    pub program: &'static [Instruction],
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ActivationCost {
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
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl ActivationCost {
    pub const ZERO: Self = Self {
        colored: [0; 6],
        generic: 0,
        alternatives: &[],
        additional: None,
        mana_component: false,
        tap: false,
        sacrifice: false,
        exile_self: false,
        discard: false,
        life: 0,
        remove: None,
        loyalty: None,
    };
    pub(crate) fn mana_options(self) -> ManaCostOptions {
        std::iter::once(ManaCost {
            colored: self.colored,
            generic: self.generic,
        })
        .chain(self.alternatives.iter().copied())
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActivationOrigin {
    Battlefield,
    Hand,
    Graveyard,
}
/// Starting restrictions, not costs that must still hold during payment.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActivationGuard {
    ClassLevelExactly(u8),
    SourceSolved,
    CastInstantSorceryThisTurn,
    ScriedOrSurveilledThisTurn,
    OpponentNoncombatDamageThisTurn,
    SourceAttackedThisTurn,
    Always,
    OwnTurn,
    ControlSubtypeAny(&'static [&'static str]),
    ControlLandsAtLeast(u16),
    ControlledPlaneswalkerSubtypeLoyaltyAtLeast { subtype: &'static str, minimum: u64 },
    ControlCreatureToughnessAtLeast(u16),
    SourceEnteredOrControlsBasicLand,
    HandSizeAtLeast(u16),
    GraveyardCardsAtLeast { kind: u16, minimum: u16 },
    LifeAboveStarting(u16),
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl ActivationGuard {
    pub(crate) fn subtypes(self) -> &'static [&'static str] {
        match self {
            Self::ClassLevelExactly(_)
            | Self::Always
            | Self::SourceSolved
            | Self::SourceAttackedThisTurn
            | Self::CastInstantSorceryThisTurn
            | Self::ScriedOrSurveilledThisTurn
            | Self::OpponentNoncombatDamageThisTurn
            | Self::OwnTurn
            | Self::ControlLandsAtLeast(_)
            | Self::ControlledPlaneswalkerSubtypeLoyaltyAtLeast { .. }
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
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlightSpec {
    pub amount: u16,
    pub normal_modes: u64,
    pub paid_modes: u64,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TeamworkSpec {
    pub minimum: u16,
    pub paid_targets: Option<TargetPlan>,
    /// Bitsets over the predeclared modal combinations; ignored for nonmodal spells.
    pub normal_modes: u64,
    pub paid_modes: u64,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ActivatedAbility {
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

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl ActivatedAbility {
    /// The first marker identifies the generic component that can be paid by
    /// tapping artifacts or creatures. It has no effect during resolution.
    pub(crate) fn waterbend_cost(&self) -> u16 {
        match self.program.first() {
            Some(Instruction::WaterbendCost(amount)) => *amount,
            _ => 0,
        }
    }
    /// Existing typed equip slots are exactly the battlefield AttachSource program.
    /// Entry attachments, channel and animation programs are different abilities.
    pub(crate) fn is_equip(&self) -> bool {
        self.origin == ActivationOrigin::Battlefield && self.program == [Instruction::AttachSource]
    }
    pub(crate) fn exerts(&self) -> bool {
        let exerts = self
            .program
            .iter()
            .any(|instruction| matches!(instruction, Instruction::ExertCost));
        assert!(
            !exerts || self.origin == ActivationOrigin::Battlefield,
            "only battlefield permanents can be exerted"
        );
        exerts
    }
    pub(crate) fn source_counter_cost(&self) -> Option<CounterKind> {
        match self.program.first() {
            Some(Instruction::SourceCounterCost { kind }) => {
                // Compact domains reserve the source tap for this payment; it
                // cannot also consume some chosen counters as a mana producer.
                assert!(
                    self.origin == ActivationOrigin::Battlefield
                        && self.cost.remove.is_none()
                        && (self.cost.tap
                            || (*kind == CounterKind::Loyalty && self.cost.loyalty == Some(0))),
                    "variable counter cost requires a tap cost or variable loyalty cost"
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
        let program = self
            .program
            .strip_prefix(&[Instruction::ExertCost])
            .unwrap_or(self.program);
        match program {
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
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZoneReplacementRule {
    RevealShuffleSelf,
    ExileInstantSorceryCards,
    ExileAllCardsAndTokens,
    ExileOpposingCreatureDeath { ability: u16 },
    ExileOpposingCreatureDeathPlain,
    OpponentDiscardToBattlefield,
}

/// Implemented static keyword flags. Other keyword cards remain unsupported.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct Keywords(u32);

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
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
    /// Contribution bonus while paying crew or saddle costs.
    pub const CREW_SADDLE_PLUS_TWO: Self = Self(1073741824);
    /// The spell prevents further spells and nonmana activations while on the stack.
    pub const SPLIT_SECOND: Self = Self(1u32 << 31);
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

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
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
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct TokenInfo {
    /// Catalogue-scoped factory lookup identity, distinct from the copied name.
    pub factory_identity: &'static str,
    pub name: &'static str,
    pub subtypes: &'static str,
    pub colors: u8,
}

/// Immutable abilities of a generated command-zone emblem.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct EmblemInfo {
    pub oracle_text: &'static str,
}

/// Self replacement evaluated against the battlefield immediately before entry.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntryTapped {
    UnlessPlainsOrIsland,
    UnlessControlsEitherSubtype(&'static str, &'static str),
    Never,
    Always,
    OpponentsTurn,
    DuringFirstThreeTurns,
    UnlessBasic,
    UnlessAtMostTwo,
    UnlessAtLeastTwo,
    UnlessLowLife,
    UnlessPlaneswalker,
    UnlessControlsSubtype(&'static str),
    PayLife(u32),
    UnlessBehold(&'static str),
    ChosenXAtMost(u32),
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Debug)]
pub struct Definition {
    pub info: CardInfo,
    pub effect: Effect,
    pub triggers: &'static [TriggeredAbility],
    pub activated: &'static [ActivatedAbility],
    pub enters: InitialCounters,
    pub entry: EntryTapped,
    pub kind: u16,
    pub basic: bool,
    pub mana_color: Option<usize>,
    pub colored: Mana,
    pub generic: u32,
}

#[doc(hidden)]
#[path = "card_definitions/mod.rs"]
pub mod authored;

#[doc(hidden)]
#[path = "catalogue.rs"]
pub mod generated;
pub use generated::{CREATURE_TYPES, CatalogueEntry};

/// Creature types from the pinned Comprehensive Rules, including Time Lord.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub fn is_creature_type(subtype: &str) -> bool {
    CREATURE_TYPES.binary_search(&subtype).is_ok()
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
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
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub type Mana = [u32; 6];

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Card {
    /// Constructor for generated, catalogue-scoped metadata. Prefer `from_name`.
    /// An out-of-catalogue ID will panic when its characteristics are queried.
    #[doc(hidden)]
    pub const fn from_catalogue_id(index: u16) -> Self {
        Self(index)
    }
    /// Supported physical cards in the catalogue linked into this application.
    pub const ALL: generated::SupportedCards = generated::SupportedCards;

    #[cfg(feature = "test-support")]
    pub(crate) const fn test_index(index: u16) -> Self {
        Self(index)
    }

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
        #[cfg(feature = "test-support")]
        if let Some((_, replacements)) = test_counter_providers::provider(self) {
            return replacements;
        }
        generated::replacement_effects(self)
    }
    pub fn colors(self) -> u8 {
        generated::catalogue().colors[self.index()]
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
        generated::catalogue()
            .names
            .binary_search_by_key(&canonical, |&(name, _)| name)
            .ok()
            .map(|index| Self(generated::catalogue().names[index].1))
    }
    pub fn catalogue() -> impl ExactSizeIterator<Item = Card> {
        (0..generated::catalogue().definitions.len()).map(|index| Card(index as u16))
    }
    pub fn catalogue_len() -> usize {
        generated::catalogue().definitions.len()
    }
    pub fn info(self) -> &'static CardInfo {
        &generated::catalogue().definitions[self.0 as usize].info
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
    pub fn is_token(self) -> bool {
        self.token_info().is_some()
    }
    pub fn vehicle_stats(self) -> Option<(i128, i128)> {
        generated::vehicle_stats(self).map(|s| (s.power, s.toughness))
    }
    /// Immutable inset spell definition; never a second physical deck card.
    pub(crate) fn transforming_double_face(self) -> bool {
        authored::rules(self)
            .unwrap()
            .extras
            .transforming_double_face
    }
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
    pub(crate) fn is_prepared_face(self) -> bool {
        authored::rules(self).is_some_and(|rules| rules.extras.prepared_identity.is_some())
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
    pub(crate) fn mayhem_cost(self) -> Option<ManaCost> {
        authored::rules(self).and_then(|r| r.extras.mayhem_cost)
    }
    pub(crate) fn harmonize_cost(self) -> Option<ManaCost> {
        authored::rules(self).and_then(|r| r.extras.harmonize_cost)
    }
    pub(crate) fn sneak_cost(self) -> Option<ManaCost> {
        authored::rules(self).and_then(|r| r.extras.sneak_cost)
    }
    pub(crate) fn disguise_cost(self) -> Option<ManaCost> {
        authored::rules(self).and_then(|r| r.extras.disguise_cost)
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
        #[cfg(feature = "test-support")]
        if let Some((groups, _)) = test_counter_providers::provider(self) {
            return groups;
        }
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
    pub(crate) fn aura_artifact_or_non_aura_enchantment(self) -> bool {
        authored::rules(self)
            .is_some_and(|rules| rules.extras.aura_artifact_or_non_aura_enchantment)
    }
    pub(crate) fn aura_kind_or_subtype(self) -> Option<&'static str> {
        authored::rules(self).and_then(|rules| rules.extras.aura_kind_or_subtype)
    }
    pub(crate) fn aura(self) -> Option<AuraSpec> {
        authored::rules(self).map_or_else(|| generated::aura(self), |rules| rules.aura)
    }
    pub(crate) fn attachment_bonus(self) -> Option<EquipmentSpec> {
        self.equipment().or_else(|| self.aura().map(|a| a.bonus))
    }
    /// Explicit immutable authored fact. Unreviewed/gated cards conservatively have abilities.
    pub fn has_rules_abilities(self) -> bool {
        self.is_basic_land()
            || authored::rules(self)
                .is_none_or(|rules| rules.extras.has_rules_abilities.unwrap_or(true))
    }
    pub fn keywords(self) -> Keywords {
        #[cfg(feature = "test-support")]
        if let Some(rules) = fixture_rules::rules(self) {
            return rules.keywords;
        }
        self.info().keywords
    }
    pub fn is_creature(self) -> bool {
        generated::catalogue().definitions[self.0 as usize].kind & 1 != 0
    }
    pub fn is_land(self) -> bool {
        generated::catalogue().definitions[self.0 as usize].kind & 2 != 0
    }
    pub fn is_artifact(self) -> bool {
        generated::catalogue().definitions[self.0 as usize].kind & 16 != 0
    }
    pub(crate) fn kind_mask(self) -> u16 {
        generated::catalogue().definitions[self.0 as usize].kind
    }
    pub fn is_basic_land(self) -> bool {
        generated::catalogue().definitions[self.0 as usize].basic
    }
    pub fn is_instant(self) -> bool {
        Self::kinds_are_instant(self.kind_mask())
    }
    /// One canonical Instant-kind interpretation for printed and live type queries.
    pub(crate) const fn kinds_are_instant(kinds: u16) -> bool {
        kinds & 8 != 0
    }
    pub(crate) fn effect(self) -> Effect {
        #[cfg(feature = "test-support")]
        if let Some(rules) = fixture_rules::rules(self) {
            return rules.effect;
        }
        #[cfg(feature = "test-support")]
        if let Some(effect) = test_counter_providers::outside_self_spell(self) {
            return effect;
        }
        generated::catalogue().definitions[self.0 as usize].effect
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
        #[cfg(feature = "test-support")]
        if let Some(rules) = fixture_rules::rules(self) {
            return rules.triggers;
        }
        generated::catalogue().definitions[self.0 as usize].triggers
    }
    pub(crate) fn keyword_grants(self) -> &'static [KeywordGrant] {
        generated::keyword_grants(self)
    }

    pub(crate) fn graveyard_counter_cost(self) -> Option<u16> {
        authored::rules(self).and_then(|r| r.extras.graveyard_counter_cost)
    }
    pub(crate) fn self_granted_activation_template(self, index: u16) -> bool {
        authored::rules(self)
            .unwrap()
            .extras
            .self_granted_activation_templates
            .contains(&index)
    }
    pub(crate) fn grants_activation(self, index: u16) -> bool {
        self.static_ability_groups().iter().any(|g| {
            g.fragments
                .iter()
                .any(|f| matches!(f, StaticAbilityFragment::AttachmentActivation(i) | StaticAbilityFragment::ControlledPlaneswalkerActivation(i) | StaticAbilityFragment::SourceMaxSpeedActivation(i) if *i == index))
        })
    }
    pub(crate) fn graveyard_activation_subtype(self) -> Option<&'static str> {
        self.static_ability_groups()
            .iter()
            .flat_map(|g| g.fragments)
            .find_map(|f| {
                if let StaticAbilityFragment::BorrowGraveyardActivations(subtype) = f {
                    Some(*subtype)
                } else {
                    None
                }
            })
    }
    pub(crate) fn borrows_activations(self) -> bool {
        self.borrows_exiled_activations() || self.graveyard_activation_subtype().is_some()
    }
    pub(crate) fn borrows_exiled_activations(self) -> bool {
        authored::rules(self)
            .unwrap()
            .extras
            .borrows_exiled_activations
    }
    /// Intrinsic land/dork mana is stored separately from printed activation tables.
    /// The reserved index is used only by a captured inherited-ability descriptor.
    pub(crate) fn activation(self, index: u16) -> &'static ActivatedAbility {
        if index >= u16::MAX - 5 {
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
            let color = if index == u16::MAX {
                self.mana_color().expect("captured printed intrinsic mana")
            } else {
                usize::from(u16::MAX - index - 1)
            };
            &INTRINSIC[color]
        } else {
            &self.activations()[usize::from(index)]
        }
    }
    pub(crate) fn activations(self) -> &'static [ActivatedAbility] {
        #[cfg(feature = "test-support")]
        if let Some(rules) = fixture_rules::rules(self) {
            return rules.activated;
        }
        #[cfg(feature = "test-support")]
        if self == Card::from_name("Fanatical Firebrand").unwrap()
            && issue_93_copy_cost_fixture::enabled()
        {
            return issue_93_copy_cost_fixture::abilities();
        }
        #[cfg(feature = "test-support")]
        if self == Card::JacePlaneswalkerToken && issue_93_copy_cost_fixture::prepaid_enabled() {
            return issue_93_copy_cost_fixture::prepaid_abilities();
        }
        #[cfg(feature = "test-support")]
        if self == Card::test_index(5437) && sacrifice_mana_fixture::enabled() {
            return sacrifice_mana_fixture::abilities();
        }
        #[cfg(feature = "test-support")]
        if self == Card::test_index(470) && sacrifice_mana_fixture::enabled() {
            return sacrifice_mana_fixture::abilities();
        }
        #[cfg(feature = "test-support")]
        if self == Card::JacePlaneswalkerToken && composite_loyalty_fixture::enabled() {
            return composite_loyalty_fixture::abilities();
        }
        generated::catalogue().definitions[self.0 as usize].activated
    }
    pub(crate) fn entry_tapped(self) -> EntryTapped {
        #[cfg(feature = "test-support")]
        if let Some(rules) = fixture_rules::rules(self) {
            return rules.entry;
        }
        generated::catalogue().definitions[self.0 as usize].entry
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
        let initial = generated::catalogue().definitions[self.0 as usize].enters;
        #[cfg(feature = "test-support")]
        let initial = fixture_rules::rules(self).map_or(initial, |rules| rules.enters);
        let mut counters: Counters = initial.into();
        counters.loyalty = self.starting_loyalty();
        counters.lore = u64::from(self.saga_final_chapter().is_some());
        counters.add(CounterKind::Defense, self.printed_defense());
        for &(kind, amount) in authored::rules(self).unwrap().extras.fixed_entry_counters {
            counters.add(kind, amount);
        }
        counters
    }
    pub(crate) fn printed_defense(self) -> u64 {
        authored::rules(self).unwrap().extras.battle_defense
    }
    pub(crate) fn as_turned_face_up_counters(self) -> &'static [(CounterKind, u64)] {
        authored::rules(self)
            .unwrap()
            .extras
            .as_turned_face_up_counters
    }
    pub(crate) fn opening_hand_battlefield(self) -> bool {
        authored::rules(self)
            .unwrap()
            .extras
            .opening_hand_battlefield
    }
    pub(crate) fn named_activation_lock(self) -> bool {
        authored::rules(self).unwrap().extras.named_activation_lock
    }
    pub(crate) fn entry_choice(self) -> Option<EntryChoiceKind> {
        authored::rules(self).unwrap().extras.entry_choice
    }
    pub(crate) fn chosen_entry_counters(self) -> Option<CounterKind> {
        authored::rules(self).unwrap().extras.chosen_entry_counters
    }
    pub(crate) fn power_up_abilities(self) -> &'static [u16] {
        authored::rules(self).unwrap().extras.power_up_abilities
    }
    pub(crate) fn power_up_additional_uses(self) -> u32 {
        authored::rules(self)
            .unwrap()
            .extras
            .power_up_additional_uses
    }
    pub(crate) fn power_up_other_creature_reduction(self) -> u32 {
        authored::rules(self)
            .unwrap()
            .extras
            .power_up_other_creature_reduction
    }
    pub(crate) fn power_up_mana_cost_alternatives(self) -> &'static [ManaCost] {
        authored::rules(self)
            .unwrap()
            .extras
            .power_up_mana_cost_alternatives
    }
    pub(crate) fn activation_target_reduction(
        self,
        ability: u16,
    ) -> Option<TargetActivationReduction> {
        #[cfg(feature = "test-support")]
        if self == Card::JacePlaneswalkerToken
            && ability == 0
            && composite_loyalty_fixture::target_discount()
        {
            return Some(TargetActivationReduction {
                rule: TargetRule::Creature,
                generic: 1,
            });
        }
        authored::rules(self)
            .unwrap()
            .extras
            .activation_target_reductions
            .iter()
            .find(|(index, _)| *index == ability)
            .map(|(_, rule)| *rule)
    }
    pub(crate) fn activation_target_counter_discount(self, ability: u16) -> Option<CounterKind> {
        authored::rules(self)
            .unwrap()
            .extras
            .activation_target_counter_discounts
            .iter()
            .find(|(index, _)| *index == ability)
            .map(|(_, kind)| *kind)
    }
    pub(crate) fn activation_subtype_discount(self, ability: u16) -> Option<&'static str> {
        authored::rules(self)
            .unwrap()
            .extras
            .activation_subtype_discounts
            .iter()
            .find(|(index, _)| *index == ability)
            .map(|(_, subtype)| *subtype)
    }
    pub(crate) fn activation_counter_discount(self, ability: u16) -> Option<CounterKind> {
        authored::rules(self)
            .unwrap()
            .extras
            .activation_counter_discounts
            .iter()
            .find(|(index, _)| *index == ability)
            .map(|(_, kind)| *kind)
    }
    pub(crate) fn activation_put_counter(self, ability: u16) -> Option<(CounterKind, u64)> {
        #[cfg(feature = "test-support")]
        if ability == 0
            && let Some(kind) = issue_93_mana_counter_fixture::counter_kind(self)
        {
            return Some((kind, 1));
        }
        authored::rules(self)
            .unwrap()
            .extras
            .activation_put_counters
            .iter()
            .find(|(index, _, _)| *index == ability)
            .map(|(_, kind, amount)| (*kind, *amount))
    }
    pub(crate) fn cast_entry_counters(self) -> &'static [(CounterKind, u64)] {
        authored::rules(self).unwrap().extras.cast_entry_counters
    }
    pub(crate) fn hand_cast_entry_counters(self) -> &'static [(CounterKind, u64)] {
        authored::rules(self)
            .unwrap()
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
    pub(crate) fn legacy_subtype_mana_activation(self, index: u16) -> bool {
        authored::rules(self)
            .is_some_and(|r| r.extras.legacy_subtype_mana_activation == Some(index))
    }
    /// Printed intrinsic mana abilities include all basic land subtypes, even in exile.
    pub(crate) fn printed_intrinsic_mana_colors(self) -> impl Iterator<Item = usize> {
        let land = self.is_land();
        let fallback = self.mana_color();
        (0..6).filter(move |&color| {
            if land && color < 5 {
                self.has_subtype(["Plains", "Island", "Swamp", "Mountain", "Forest"][color])
            } else {
                !land && fallback == Some(color)
            }
        })
    }
    pub(crate) fn mana_color(self) -> Option<usize> {
        generated::catalogue().definitions[self.0 as usize].mana_color
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
        let definition = &generated::catalogue().definitions[self.0 as usize];
        (definition.colored, definition.generic)
    }
    pub(crate) fn target_cost_reduction(self) -> Option<&'static TargetSpellReduction> {
        generated::target_cost_reduction(self)
    }
    /// Timing permission is an as-though effect, not a granted Flash keyword.
    pub(crate) fn cast_timing_grants(self) -> &'static [CastFilter] {
        generated::cast_timing_grants(self)
    }
    pub(crate) fn self_cast_timing(self) -> Option<SelfCastTiming> {
        authored::rules(self).and_then(|r| r.extras.self_cast_timing)
    }
    pub(crate) fn cast_graveyard_minimum(self) -> u8 {
        authored::rules(self).map_or(0, |r| r.extras.cast_graveyard_minimum)
    }
    pub(crate) fn cast_after_first_three_turns(self) -> bool {
        authored::rules(self).is_some_and(|r| r.extras.cast_after_first_three_turns)
    }
    pub(crate) fn cast_timing_guard(self) -> StepGuard {
        authored::rules(self).map_or(StepGuard::Always, |r| r.extras.cast_timing_guard)
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
        authored::rules(self).unwrap().extras.has_improvise
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
    pub(crate) fn blight(self) -> Option<BlightSpec> {
        authored::rules(self).and_then(|r| r.extras.blight)
    }
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
    pub(crate) fn flashback_cost_options(self) -> ManaCostOptions {
        std::iter::once(self.flashback_cost().expect("flashback cost")).chain(
            authored::rules(self)
                .map_or(&[][..], |r| r.extras.flashback_cost_alternatives)
                .iter()
                .copied(),
        )
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
        let definition = &generated::catalogue().definitions[self.0 as usize];
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
    #[doc(hidden)]
    pub fn index(self) -> usize {
        self.0 as usize
    }
}

/// Synthetic cost overlay for testing retained mana with replacement choices.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[cfg(feature = "test-support")]
pub mod issue_93_mana_counter_fixture {
    use super::{Card, CounterKind};
    std::thread_local! { static MODE: std::cell::Cell<u8> = const { std::cell::Cell::new(0) }; }
    pub struct Guard;
    impl Guard {
        pub(crate) fn new() -> Self {
            Self::set(1)
        }
        pub(crate) fn with_plus_one() -> Self {
            Self::set(2)
        }
        fn set(mode: u8) -> Self {
            MODE.with(|enabled| assert_eq!(enabled.replace(mode), 0));
            Self
        }
    }
    impl Drop for Guard {
        fn drop(&mut self) {
            MODE.with(|enabled| enabled.set(0));
        }
    }
    pub(super) fn counter_kind(card: Card) -> Option<CounterKind> {
        let mode = MODE.with(|enabled| enabled.get());
        let loot = card == Card::from_name("Loot, the Pathfinder").unwrap();
        let devotee = card == Card::from_name("Abzan Devotee").unwrap();
        match mode {
            1 if loot => Some(CounterKind::Stun),
            2 if loot || devotee => Some(CounterKind::PlusOne),
            _ => None,
        }
    }
}

/// Synthetic self-sacrifice source-copy ability for announcement LKI tests.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[cfg(feature = "test-support")]
pub mod issue_93_copy_cost_fixture {
    use super::*;
    std::thread_local! { static MODE: std::cell::Cell<u8> = const { std::cell::Cell::new(0) }; }
    pub struct Guard;
    impl Guard {
        pub(crate) fn new() -> Self {
            MODE.with(|mode| assert_eq!(mode.replace(1), 0));
            Self
        }
        pub(crate) fn prepaid() -> Self {
            MODE.with(|mode| assert_eq!(mode.replace(2), 0));
            Self
        }
    }
    impl Drop for Guard {
        fn drop(&mut self) {
            MODE.with(|mode| mode.set(0));
        }
    }
    pub(super) fn enabled() -> bool {
        MODE.with(|mode| mode.get() == 1)
    }
    pub(super) fn prepaid_enabled() -> bool {
        MODE.with(|mode| mode.get() == 2)
    }
    pub(super) const fn abilities() -> &'static [ActivatedAbility] {
        &[ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            cycling: false,
            origin: ActivationOrigin::Battlefield,
            cost: ActivationCost {
                colored: [0; 6],
                generic: 0,
                alternatives: &[],
                additional: None,
                mana_component: false,
                tap: false,
                sacrifice: true,
                exile_self: false,
                discard: false,
                life: 0,
                remove: None,
                loyalty: None,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateCopyToken {
                subject: Subject::Source,
                amount: 1,
            }],
            sorcery: false,
            once_per_turn: false,
        }]
    }
    pub(super) fn prepaid_abilities() -> &'static [ActivatedAbility] {
        const PREPAID: &[ActivatedAbility] = &[ActivatedAbility {
            cost: ActivationCost {
                colored: [0; 6],
                generic: 0,
                alternatives: &[],
                additional: None,
                mana_component: false,
                tap: false,
                sacrifice: true,
                exile_self: false,
                discard: false,
                life: 0,
                remove: None,
                loyalty: Some(-1),
            },
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::Object {
                    zone: TargetZone::Battlefield,
                    kinds: 1,
                    relation: 1,
                    other: false,
                    nonland: false,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                },
                min: 0,
                max: 2,
            }),
            ..abilities()[0]
        }];
        PREPAID
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Serialize for Card {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.identity())
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl<'de> Deserialize<'de> for Card {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let name = String::deserialize(deserializer)?;
        Self::from_name(&name)
            .ok_or_else(|| serde::de::Error::custom(format!("unknown card: {name}")))
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
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
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoleKind {
    Cursed,
    Monster,
    Royal,
    Sorcerer,
    Wicked,
    YoungHero,
}

/// A scoped, thread-local typed provider for exercising composite loyalty costs
/// through the public action path. It never changes non-test gameplay.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[cfg(feature = "test-support")]
pub mod composite_loyalty_fixture {
    use super::*;
    std::thread_local! { static MODE: std::cell::Cell<u8> = const { std::cell::Cell::new(0) }; }
    pub struct Guard;
    impl Guard {
        pub(crate) fn new() -> Self {
            Self::set(1)
        }
        pub(crate) fn with_mana() -> Self {
            Self::set(2)
        }
        pub(crate) fn with_mana_two_counters() -> Self {
            Self::set(3)
        }
        pub(crate) fn with_mana_positive() -> Self {
            Self::set(4)
        }
        pub(crate) fn with_mana_positive_target() -> Self {
            Self::set(5)
        }
        fn set(mode: u8) -> Self {
            MODE.with(|value| {
                assert_eq!(value.replace(mode), 0);
            });
            Self
        }
    }
    impl Drop for Guard {
        fn drop(&mut self) {
            MODE.with(|value| value.set(0));
        }
    }
    pub(super) fn enabled() -> bool {
        MODE.with(|value| value.get() != 0)
    }
    pub(super) fn target_discount() -> bool {
        MODE.with(|value| value.get() == 5)
    }
    pub(super) fn abilities() -> &'static [ActivatedAbility] {
        match MODE.with(|value| value.get()) {
            2 => MANA_ABILITIES,
            3 => TWO_COUNTER_ABILITIES,
            4 => POSITIVE_MANA_ABILITIES,
            5 => POSITIVE_MANA_TARGET_ABILITIES,
            _ => ABILITIES,
        }
    }
    const fn ability(mana_component: bool, amount: u16) -> ActivatedAbility {
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            once_per_turn: false,
            sorcery: true,
            cost: ActivationCost {
                colored: [0; 6],
                generic: if mana_component { 1 } else { 0 },
                alternatives: &[],
                additional: Some(AdditionalCost {
                    kind: EffectChoiceKind::RemoveCreatureCounters(amount),
                    filter: SacrificeFilter {
                        keywords: Keywords::NONE,
                        source_only: false,
                        kinds: 1,
                        nonland: false,
                        other: false,
                        tokens: None,
                        artifact_enchantment_or_token: false,
                        subtype: None,
                    },
                }),
                mana_component,
                tap: false,
                sacrifice: false,
                exile_self: false,
                discard: false,
                life: 0,
                remove: Some((None, 1)),
                loyalty: Some(-1),
            },
            targets: TargetPlan::NONE,
            program: &[],
        }
    }
    const ABILITIES: &[ActivatedAbility] = &[ability(false, 1)];
    const MANA_ABILITIES: &[ActivatedAbility] = &[ability(true, 1)];
    const TWO_COUNTER_ABILITIES: &[ActivatedAbility] = &[ability(true, 2)];
    const POSITIVE_MANA_ABILITIES: &[ActivatedAbility] = &[ActivatedAbility {
        cost: ActivationCost {
            colored: [0; 6],
            generic: 1,
            alternatives: &[],
            additional: None,
            mana_component: true,
            tap: false,
            sacrifice: false,
            exile_self: false,
            discard: false,
            life: 0,
            remove: None,
            loyalty: Some(1),
        },
        ..ability(false, 0)
    }];
    const POSITIVE_MANA_TARGET_ABILITIES: &[ActivatedAbility] = &[ActivatedAbility {
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        ..POSITIVE_MANA_ABILITIES[0]
    }];
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[cfg(feature = "test-support")]
pub mod sacrifice_mana_fixture {
    use super::*;
    std::thread_local! { static MODE: std::cell::Cell<u8> = const { std::cell::Cell::new(0) }; }
    pub struct Guard;
    impl Guard {
        pub(crate) fn new() -> Self {
            Self::set(1)
        }
        pub(crate) fn artifact_only() -> Self {
            Self::set(2)
        }
        fn set(mode: u8) -> Self {
            MODE.with(|v| assert_eq!(v.replace(mode), 0));
            Self
        }
    }
    impl Drop for Guard {
        fn drop(&mut self) {
            MODE.with(|v| v.set(0));
        }
    }
    pub(super) fn enabled() -> bool {
        MODE.with(|v| v.get() != 0)
    }
    const fn ability(program: &'static [Instruction]) -> ActivatedAbility {
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            cycling: false,
            origin: ActivationOrigin::Battlefield,
            cost: ActivationCost {
                colored: [0; 6],
                generic: 0,
                alternatives: &[],
                additional: None,
                mana_component: false,
                tap: false,
                sacrifice: true,
                exile_self: false,
                discard: false,
                life: 0,
                remove: None,
                loyalty: None,
            },
            targets: TargetPlan::NONE,
            program,
            sorcery: false,
            once_per_turn: false,
        }
    }
    const ARTIFACT_ONLY: &[ActivatedAbility] =
        &[ability(&[Instruction::AddMana(ManaProduction::Tagged {
            production: &ManaProduction::Fixed([0, 0, 0, 0, 1, 0]),
            quality: &ManaQuality {
                restriction: ManaRestriction::ArtifactSpell,
                spell_bonus: ManaSpellBonus::None,
                retention: ManaRetention::Step,
            },
        })])];
    const ORDINARY: &[ActivatedAbility] =
        &[ability(&[Instruction::AddMana(ManaProduction::Fixed([
            0, 0, 0, 0, 1, 0,
        ]))])];
    pub(super) fn abilities() -> &'static [ActivatedAbility] {
        match MODE.with(|v| v.get()) {
            2 => ARTIFACT_ONLY,
            _ => ORDINARY,
        }
    }
}
