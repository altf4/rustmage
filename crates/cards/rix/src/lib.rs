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
pub(super) mod card_2820 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/rix/moment_of_craving.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2823 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/rix/moment_of_triumph.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
}
#[rustfmt::skip]
pub static ENTRIES: [CatalogueEntry; 2] = [
CatalogueEntry { definition: Definition { info: CardInfo { name: "Moment of Craving", oracle_id: Some("1489551f-5714-471b-a0e6-f40bc3085afc"), mana_cost: "{1}{B}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2820::RULES.keywords, deck_limit: authored::card_2820::RULES.deck_limit }, effect: authored::card_2820::RULES.checked_effect(), triggers: authored::card_2820::RULES.triggers, activated: authored::card_2820::RULES.activated, enters: authored::card_2820::RULES.enters, entry: authored::card_2820::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 1 }, rules: &authored::card_2820::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Moment of Triumph", oracle_id: Some("57579799-2582-417f-9864-4560cac39ac3"), mana_cost: "{W}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2823::RULES.keywords, deck_limit: authored::card_2823::RULES.deck_limit }, effect: authored::card_2823::RULES.checked_effect(), triggers: authored::card_2823::RULES.triggers, activated: authored::card_2823::RULES.activated, enters: authored::card_2823::RULES.enters, entry: authored::card_2823::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_2823::RULES },
];
}
pub use cards::ENTRIES;
