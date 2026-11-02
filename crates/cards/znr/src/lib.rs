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
pub(super) mod card_2042 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/znr/highborn_vampire.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
}
#[rustfmt::skip]
pub static ENTRIES: [CatalogueEntry; 1] = [
CatalogueEntry { definition: Definition { info: CardInfo { name: "Highborn Vampire", oracle_id: Some("1adcf587-9035-4299-a2bd-62f805fdd897"), mana_cost: "{3}{B}", type_line: "Creature — Vampire Warrior", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2042::RULES.keywords, deck_limit: authored::card_2042::RULES.deck_limit }, effect: authored::card_2042::RULES.checked_effect(), triggers: authored::card_2042::RULES.triggers, activated: authored::card_2042::RULES.activated, enters: authored::card_2042::RULES.enters, entry: authored::card_2042::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 3 }, rules: &authored::card_2042::RULES },
];
}
pub use cards::ENTRIES;
