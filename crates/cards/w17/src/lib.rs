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
pub(super) mod card_4806 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/w17/untamed_hunger.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
}
#[rustfmt::skip]
pub static ENTRIES: [CatalogueEntry; 1] = [
CatalogueEntry { definition: Definition { info: CardInfo { name: "Untamed Hunger", oracle_id: Some("550abd9b-9aae-4812-9bc0-7f131d6dcde8"), mana_cost: "{2}{B}", type_line: "Enchantment — Aura", oracle_keywords: &["Enchant"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4806::RULES.keywords, deck_limit: authored::card_4806::RULES.deck_limit }, effect: authored::card_4806::RULES.checked_effect(), triggers: authored::card_4806::RULES.triggers, activated: authored::card_4806::RULES.activated, enters: authored::card_4806::RULES.enters, entry: authored::card_4806::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 2 }, rules: &authored::card_4806::RULES },
];
}
pub use cards::ENTRIES;
