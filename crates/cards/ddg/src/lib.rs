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
pub(super) mod card_2476 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/ddg/leonin_skyhunter.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
}
#[rustfmt::skip]
pub static ENTRIES: [CatalogueEntry; 1] = [
CatalogueEntry { definition: Definition { info: CardInfo { name: "Leonin Skyhunter", oracle_id: Some("517c7295-8ba2-47a6-a1c6-aee722f8ca88"), mana_cost: "{W}{W}", type_line: "Creature — Cat Knight", oracle_keywords: &["Flying"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2476::RULES.keywords, deck_limit: authored::card_2476::RULES.deck_limit }, effect: authored::card_2476::RULES.checked_effect(), triggers: authored::card_2476::RULES.triggers, activated: authored::card_2476::RULES.activated, enters: authored::card_2476::RULES.enters, entry: authored::card_2476::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [2, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_2476::RULES },
];
}
pub use cards::ENTRIES;
