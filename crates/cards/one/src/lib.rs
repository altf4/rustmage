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
pub(super) mod card_2992 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/one/offer_immortality.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
}
#[rustfmt::skip]
pub static ENTRIES: [CatalogueEntry; 1] = [
CatalogueEntry { definition: Definition { info: CardInfo { name: "Offer Immortality", oracle_id: Some("be4499e6-616a-4c11-b37e-9201d2492682"), mana_cost: "{1}{B}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2992::RULES.keywords, deck_limit: authored::card_2992::RULES.deck_limit }, effect: authored::card_2992::RULES.checked_effect(), triggers: authored::card_2992::RULES.triggers, activated: authored::card_2992::RULES.activated, enters: authored::card_2992::RULES.enters, entry: authored::card_2992::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 1 }, rules: &authored::card_2992::RULES },
];
}
pub use cards::ENTRIES;
