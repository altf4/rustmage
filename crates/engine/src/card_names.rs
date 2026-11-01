//! Name-only Oracle metadata, independent of playable card implementation coverage.
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn packed_lands() -> &'static str {
    crate::cards::generated::catalogue().land_names
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
static LANDS: OnceLock<Box<[CardName]>> = OnceLock::new();
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn lands() -> &'static [CardName] {
    LANDS.get_or_init(|| {
        packed_lands()
            .lines()
            .map(|name| CardName::from_name(name).expect("pinned selectable land name"))
            .collect()
    })
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn packed_names() -> &'static str {
    crate::cards::generated::catalogue().card_names
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn packed_interchangeable() -> &'static str {
    crate::cards::generated::catalogue().interchangeable_names
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
type NamePair = (&'static str, &'static str);
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
static INTERCHANGEABLE: OnceLock<Box<[NamePair]>> = OnceLock::new();
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
static NAMES: OnceLock<Box<[&'static str]>> = OnceLock::new();
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn names() -> &'static [&'static str] {
    NAMES.get_or_init(|| packed_names().lines().collect())
}
/// A comparison key only: observations and selected names retain their spelling.
/// Each original name maps to its interchangeable Oracle name (CR201.3).
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(crate) fn comparison_key(name: &str) -> &str {
    let pairs = INTERCHANGEABLE.get_or_init(|| {
        packed_interchangeable()
            .lines()
            .map(|line| line.split_once('\t').expect("interchangeable name pair"))
            .collect()
    });
    pairs
        .binary_search_by_key(&name, |(original, _)| *original)
        .map_or(name, |index| pairs[index].1)
}
/// Nameless objects never share a name, including with each other (CR201.2a).
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(crate) fn same_name(left: &str, right: &str) -> bool {
    !left.is_empty()
        && !right.is_empty()
        && (left == right || comparison_key(left) == comparison_key(right))
}
/// An interned rules name. Public JSON contains its ordinary string, never an ID.
/// Name choices include cards outside the currently implemented Standard pool.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CardName(u32);
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
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
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Serialize for CardName {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(self.as_str())
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl<'de> Deserialize<'de> for CardName {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let name = String::deserialize(d)?;
        Self::from_name(&name)
            .ok_or_else(|| serde::de::Error::custom("not a selectable Oracle card name"))
    }
}
