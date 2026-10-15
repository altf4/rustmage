use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Compact catalogue identity. The immutable definitions are shared across all
/// games; no card strings, JSON, or rule objects are cloned into game states.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Card(u16);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Effect {
    Creature(i32, i32),
    Damage(i32),
    Grow,
    Draw,
    Counter,
    ReturnToHand,
    Destroy,
    Land,
    Unsupported,
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

struct Definition {
    info: CardInfo,
    effect: Effect,
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

/// Mana is W U B R G. Generic costs deliberately have no colour slot.
pub type Mana = [u32; 5];

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
    pub fn keywords(self) -> Keywords {
        self.info().keywords
    }
    pub fn is_creature(self) -> bool {
        generated::DEFINITIONS[self.0 as usize].kind & 1 != 0
    }
    pub fn is_land(self) -> bool {
        generated::DEFINITIONS[self.0 as usize].kind & 2 != 0
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
    if (0..5).any(|i| pool[i] < colored[i]) {
        return;
    }
    fn visit(i: usize, left: u32, pool: Mana, mut paid: Mana, emit: &mut impl FnMut(Mana)) {
        if i == 4 {
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
