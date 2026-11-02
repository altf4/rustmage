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
pub(super) mod card_2260 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/cmm/jazal_goldmane.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
}
#[rustfmt::skip]
pub static ENTRIES: [CatalogueEntry; 1] = [
CatalogueEntry { definition: Definition { info: CardInfo { name: "Jazal Goldmane", oracle_id: Some("609d13dc-c850-4877-8ad3-e84171b58069"), mana_cost: "{2}{W}{W}", type_line: "Legendary Creature — Cat Warrior", oracle_keywords: &["First strike"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2260::RULES.keywords, deck_limit: authored::card_2260::RULES.deck_limit }, effect: authored::card_2260::RULES.checked_effect(), triggers: authored::card_2260::RULES.triggers, activated: authored::card_2260::RULES.activated, enters: authored::card_2260::RULES.enters, entry: authored::card_2260::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [2, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_2260::RULES },
];
}
pub use cards::ENTRIES;
