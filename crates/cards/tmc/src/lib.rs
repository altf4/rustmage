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
pub(super) mod card_4884 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/tmc/vibrant_cityscape.rs"));
}
}
#[rustfmt::skip]
pub static ENTRIES: [CatalogueEntry; 1] = [
CatalogueEntry { definition: Definition { info: CardInfo { name: "Vibrant Cityscape", oracle_id: Some("6a7f3e1f-6798-4644-b64c-7765f81f0938"), mana_cost: "", type_line: "Land", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4884::RULES.keywords, deck_limit: authored::card_4884::RULES.deck_limit }, effect: authored::card_4884::RULES.checked_effect(), triggers: authored::card_4884::RULES.triggers, activated: authored::card_4884::RULES.activated, enters: authored::card_4884::RULES.enters, entry: authored::card_4884::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_4884::RULES },
];
}
pub use cards::ENTRIES;
