use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Compact catalogue identity. The immutable definitions are shared across all
/// games; no card strings, JSON, or rule objects are cloned into game states.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Card(u16);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Effect {
    Creature(i32, i32),
    Spell(TargetSpec, &'static [Instruction]),
    Land,
    Permanent,
    Unsupported,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TargetRule {
    None,
    Creature,
    Any,
    Spell,
    Player {
        opponent_only: bool,
    },
    Object {
        zone: TargetZone,
        kinds: u8,
        relation: i8,
        other: bool,
        nonland: bool,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PlayerSet {
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

/// Immutable rule bytecode. Only the program counter and choices live in state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Instruction {
    AddMana(ManaProduction),
    Damage(i32),
    Grow(i32),
    Counter,
    ReturnToHand,
    ReturnToBattlefield,
    Destroy,
    Tap,
    Untap,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Subject {
    Source,
    Targets,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum CounterKind {
    PlusOne,
    MinusOne,
    Stun,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Counters {
    pub plus_one: u32,
    pub minus_one: u32,
    pub stun: u32,
}
impl Counters {
    pub(crate) fn add(&mut self, kind: CounterKind, amount: u32) {
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
pub(crate) enum TriggerCondition {
    SelfEnters,
    SelfDies,
    SelfAttacks,
    SelfEntersOrAttacks,
    OtherCreatureEntersYou,
    CreatureEntersYou,
    OtherArtifactEntersYou,
    LandEntersYou,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TriggeredAbility {
    pub condition: TriggerCondition,
    pub targets: TargetSpec,
    pub program: &'static [Instruction],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ActivationCost {
    pub colored: Mana,
    pub generic: u32,
    pub tap: bool,
    pub sacrifice: bool,
    pub life: u16,
    pub remove: Option<(Option<CounterKind>, u32)>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ActivatedAbility {
    pub cost: ActivationCost,
    pub targets: TargetSpec,
    pub program: &'static [Instruction],
    pub sorcery: bool,
    pub once_per_turn: bool,
}

impl ActivatedAbility {
    pub(crate) fn mana_production(&self) -> Option<ManaProduction> {
        match self.program {
            [Instruction::AddMana(production)] if self.targets.max == 0 => Some(*production),
            _ => None,
        }
    }
}

/// Implemented static keyword flags. Other keyword cards remain unsupported.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub struct Keywords(u16);

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

struct Definition {
    info: CardInfo,
    effect: Effect,
    triggers: &'static [TriggeredAbility],
    activated: &'static [ActivatedAbility],
    enters: Counters,
    kind: u8,
    basic: bool,
    mana_color: Option<usize>,
    colored: Mana,
    generic: u32,
}

mod generated {
    use super::*;
    include!("generated/catalogue.rs");
}
pub use generated::CATALOGUE_VERSION;

/// Mana is W U B R G C; C is colorless. Generic costs deliberately have no colour slot.
pub type Mana = [u32; 6];

impl Card {
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
    pub(crate) fn kind_mask(self) -> u8 {
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
    pub(crate) fn triggers(self) -> &'static [TriggeredAbility] {
        generated::DEFINITIONS[self.0 as usize].triggers
    }

    pub(crate) fn activations(self) -> &'static [ActivatedAbility] {
        generated::DEFINITIONS[self.0 as usize].activated
    }
    pub(crate) fn enters_counters(self) -> Counters {
        generated::DEFINITIONS[self.0 as usize].enters
    }
    pub(crate) fn mana_color(self) -> Option<usize> {
        generated::DEFINITIONS[self.0 as usize].mana_color
    }
    pub(crate) fn cost(self) -> (Mana, u32) {
        let definition = &generated::DEFINITIONS[self.0 as usize];
        (definition.colored, definition.generic)
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
