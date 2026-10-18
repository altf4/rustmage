//! Name-only Oracle metadata, independent of playable card implementation coverage.
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

const PACKED_NAMES: &str = include_str!("../data/card-names.txt");
static NAMES: OnceLock<Box<[&'static str]>> = OnceLock::new();
fn names() -> &'static [&'static str] {
    NAMES.get_or_init(|| PACKED_NAMES.lines().collect())
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
