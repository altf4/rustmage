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
pub(super) mod card_3996 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/pspl/spectacular_spider_man.rs"));
}
}
#[rustfmt::skip]
pub static ENTRIES: [CatalogueEntry; 1] = [
CatalogueEntry { definition: Definition { info: CardInfo { name: "Spectacular Spider-Man", oracle_id: Some("eed5e0cc-8f88-4779-bf10-6a823c72df7f"), mana_cost: "{1}{W}", type_line: "Legendary Creature — Spider Human Hero", oracle_keywords: &["Flash"], standard_legal: true, legendary: true, implemented: false, keywords: authored::card_3996::RULES.keywords, deck_limit: authored::card_3996::RULES.deck_limit }, effect: authored::card_3996::RULES.effect, triggers: authored::card_3996::RULES.triggers, activated: authored::card_3996::RULES.activated, enters: authored::card_3996::RULES.enters, entry: authored::card_3996::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_3996::RULES },
];
}
pub use cards::ENTRIES;
