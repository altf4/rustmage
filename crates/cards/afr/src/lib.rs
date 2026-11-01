// Generated module links and metadata, never rule behavior.
#![allow(unused_imports)]
use rustmage_core::engine;
#[rustfmt::skip]
mod cards {
pub use rustmage_core::cards::*;
pub type Card = rustmage_core::Card;
#[allow(dead_code, non_snake_case)]
pub const fn Card(index: u16) -> Card { Card::from_catalogue_id(index) }
use rustmage_core::cards::authored::{CardRules, CardExtras, generic, one, object};
mod authored {
use super::*;
pub(super) mod card_4842 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/afr/vampire_spawn.rs"));
}
}
#[rustfmt::skip]
pub static ENTRIES: [CatalogueEntry; 1] = [
CatalogueEntry { definition: Definition { info: CardInfo { name: "Vampire Spawn", oracle_id: Some("33cee4b4-3d80-46f1-9223-49ed785891a1"), mana_cost: "{2}{B}", type_line: "Creature — Vampire", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4842::RULES.keywords, deck_limit: authored::card_4842::RULES.deck_limit }, effect: authored::card_4842::RULES.checked_effect(), triggers: authored::card_4842::RULES.triggers, activated: authored::card_4842::RULES.activated, enters: authored::card_4842::RULES.enters, entry: authored::card_4842::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 2 }, rules: &authored::card_4842::RULES },
];
}
pub use cards::ENTRIES;
