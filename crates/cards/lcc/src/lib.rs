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
pub(super) mod card_0895 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/lcc/crossway_troublemakers.rs"));
}
}
#[rustfmt::skip]
pub static ENTRIES: [CatalogueEntry; 1] = [
CatalogueEntry { definition: Definition { info: CardInfo { name: "Crossway Troublemakers", oracle_id: Some("1a362e4d-6c02-4b67-ab63-c6622e505195"), mana_cost: "{5}{B}", type_line: "Creature — Vampire", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0895::RULES.keywords, deck_limit: authored::card_0895::RULES.deck_limit }, effect: authored::card_0895::RULES.checked_effect(), triggers: authored::card_0895::RULES.triggers, activated: authored::card_0895::RULES.activated, enters: authored::card_0895::RULES.enters, entry: authored::card_0895::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 5 }, rules: &authored::card_0895::RULES },
];
}
pub use cards::ENTRIES;
