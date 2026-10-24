//! Name-only Oracle metadata, independent of playable card implementation coverage.
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

const PACKED_LANDS: &str = include_str!("../data/land-card-names.txt");
static LANDS: OnceLock<Box<[CardName]>> = OnceLock::new();
fn lands() -> &'static [CardName] {
    LANDS.get_or_init(|| {
        PACKED_LANDS
            .lines()
            .map(|name| CardName::from_name(name).expect("pinned selectable land name"))
            .collect()
    })
}
const PACKED_NAMES: &str = include_str!("../data/card-names.txt");
const PACKED_INTERCHANGEABLE: &str = include_str!("../data/interchangeable-card-names.tsv");
type NamePair = (&'static str, &'static str);
static INTERCHANGEABLE: OnceLock<Box<[NamePair]>> = OnceLock::new();
static NAMES: OnceLock<Box<[&'static str]>> = OnceLock::new();
fn names() -> &'static [&'static str] {
    NAMES.get_or_init(|| PACKED_NAMES.lines().collect())
}
/// A comparison key only: observations and selected names retain their spelling.
/// Each original name maps to its interchangeable Oracle name (CR201.3).
pub(crate) fn comparison_key(name: &str) -> &str {
    let pairs = INTERCHANGEABLE.get_or_init(|| {
        PACKED_INTERCHANGEABLE
            .lines()
            .map(|line| line.split_once('\t').expect("interchangeable name pair"))
            .collect()
    });
    pairs
        .binary_search_by_key(&name, |(original, _)| *original)
        .map_or(name, |index| pairs[index].1)
}
/// Nameless objects never share a name, including with each other (CR201.2a).
pub(crate) fn same_name(left: &str, right: &str) -> bool {
    !left.is_empty()
        && !right.is_empty()
        && (left == right || comparison_key(left) == comparison_key(right))
}
/// An interned rules name. Public JSON contains its ordinary string, never an ID.
/// Name choices include cards outside the currently implemented Standard pool.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CardName(u32);
impl CardName {
    pub fn from_name(name: &str) -> Option<Self> {
        names().binary_search(&name).ok().map(|i| Self(i as u32))
    }
    pub fn as_str(self) -> &'static str {
        names()[self.0 as usize]
    }
    /// Rules equality, including interchangeable Oracle names (CR201.4g).
    pub fn matches(self, name: &str) -> bool {
        same_name(self.as_str(), name)
    }
    pub fn is_land(self) -> bool {
        lands().binary_search_by_key(&self.0, |name| name.0).is_ok()
    }
    pub fn lands() -> impl Iterator<Item = Self> {
        lands().iter().copied()
    }
    pub fn all() -> impl Iterator<Item = Self> {
        (0..names().len() as u32).map(Self)
    }
}
impl Serialize for CardName {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(self.as_str())
    }
}
impl<'de> Deserialize<'de> for CardName {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let name = String::deserialize(d)?;
        Self::from_name(&name)
            .ok_or_else(|| serde::de::Error::custom("not a selectable Oracle card name"))
    }
}
