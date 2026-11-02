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
pub(super) mod card_0051 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/tle/acrobatic_leap.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0489 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/tle/bolt_bend.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1088 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/tle/diresight.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1482 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/tle/feed_the_swarm.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2187 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/tle/inspiring_call.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
}
#[rustfmt::skip]
pub static ENTRIES: [CatalogueEntry; 5] = [
CatalogueEntry { definition: Definition { info: CardInfo { name: "Acrobatic Leap", oracle_id: Some("1347224f-421e-4270-8e50-cf25c875909c"), mana_cost: "{W}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0051::RULES.keywords, deck_limit: authored::card_0051::RULES.deck_limit }, effect: authored::card_0051::RULES.checked_effect(), triggers: authored::card_0051::RULES.triggers, activated: authored::card_0051::RULES.activated, enters: authored::card_0051::RULES.enters, entry: authored::card_0051::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_0051::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Bolt Bend", oracle_id: Some("c20a96f7-aa5a-4c15-b8b1-806685c99b27"), mana_cost: "{3}{R}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0489::RULES.keywords, deck_limit: authored::card_0489::RULES.deck_limit }, effect: authored::card_0489::RULES.checked_effect(), triggers: authored::card_0489::RULES.triggers, activated: authored::card_0489::RULES.activated, enters: authored::card_0489::RULES.enters, entry: authored::card_0489::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 3 }, rules: &authored::card_0489::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Diresight", oracle_id: Some("a7b12df9-fddd-4cd4-8c09-d54a47867ab6"), mana_cost: "{2}{B}", type_line: "Sorcery", oracle_keywords: &["Surveil"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1088::RULES.keywords, deck_limit: authored::card_1088::RULES.deck_limit }, effect: authored::card_1088::RULES.checked_effect(), triggers: authored::card_1088::RULES.triggers, activated: authored::card_1088::RULES.activated, enters: authored::card_1088::RULES.enters, entry: authored::card_1088::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 2 }, rules: &authored::card_1088::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Feed the Swarm", oracle_id: Some("5825997b-10d7-4a36-972c-a80ddd90b8ed"), mana_cost: "{1}{B}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1482::RULES.keywords, deck_limit: authored::card_1482::RULES.deck_limit }, effect: authored::card_1482::RULES.checked_effect(), triggers: authored::card_1482::RULES.triggers, activated: authored::card_1482::RULES.activated, enters: authored::card_1482::RULES.enters, entry: authored::card_1482::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 1 }, rules: &authored::card_1482::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Inspiring Call", oracle_id: Some("9b9a10ff-5a5d-4df8-88aa-18d84ff9117c"), mana_cost: "{2}{G}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2187::RULES.keywords, deck_limit: authored::card_2187::RULES.deck_limit }, effect: authored::card_2187::RULES.checked_effect(), triggers: authored::card_2187::RULES.triggers, activated: authored::card_2187::RULES.activated, enters: authored::card_2187::RULES.enters, entry: authored::card_2187::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 2 }, rules: &authored::card_2187::RULES },
];
}
pub use cards::ENTRIES;
