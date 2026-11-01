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
pub(super) mod card_1747 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/m19/gigantosaurus.rs"));
}
pub(super) mod card_2478 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/m19/leonin_vanguard.rs"));
}
}
#[rustfmt::skip]
pub static ENTRIES: [CatalogueEntry; 2] = [
CatalogueEntry { definition: Definition { info: CardInfo { name: "Gigantosaurus", oracle_id: Some("e666bae7-dd51-4921-8b89-7e8d423caba0"), mana_cost: "{G}{G}{G}{G}{G}", type_line: "Creature — Dinosaur", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1747::RULES.keywords, deck_limit: authored::card_1747::RULES.deck_limit }, effect: authored::card_1747::RULES.checked_effect(), triggers: authored::card_1747::RULES.triggers, activated: authored::card_1747::RULES.activated, enters: authored::card_1747::RULES.enters, entry: authored::card_1747::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 5, 0], generic: 0 }, rules: &authored::card_1747::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Leonin Vanguard", oracle_id: Some("de1e1352-adb2-4ac1-bb2a-ee654d5c727d"), mana_cost: "{W}", type_line: "Creature — Cat Soldier", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2478::RULES.keywords, deck_limit: authored::card_2478::RULES.deck_limit }, effect: authored::card_2478::RULES.checked_effect(), triggers: authored::card_2478::RULES.triggers, activated: authored::card_2478::RULES.activated, enters: authored::card_2478::RULES.enters, entry: authored::card_2478::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_2478::RULES },
];
}
pub use cards::ENTRIES;
