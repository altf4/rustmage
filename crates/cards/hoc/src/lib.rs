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
pub(super) mod card_0321 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/hoc/banishing_light.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1299 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/hoc/elvish_archdruid.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1915 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/hoc/guttersnipe.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2720 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/hoc/mentor_of_the_meek.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3576 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/hoc/rogue_s_passage.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
}
#[rustfmt::skip]
pub static ENTRIES: [CatalogueEntry; 5] = [
CatalogueEntry { definition: Definition { info: CardInfo { name: "Banishing Light", oracle_id: Some("f28b21a6-f7ce-437a-8c5b-0423cb55cefb"), mana_cost: "{2}{W}", type_line: "Enchantment", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0321::RULES.keywords, deck_limit: authored::card_0321::RULES.deck_limit }, effect: authored::card_0321::RULES.checked_effect(), triggers: authored::card_0321::RULES.triggers, activated: authored::card_0321::RULES.activated, enters: authored::card_0321::RULES.enters, entry: authored::card_0321::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_0321::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Elvish Archdruid", oracle_id: Some("6e2c2423-d854-4478-99e6-64f29851f026"), mana_cost: "{1}{G}{G}", type_line: "Creature — Elf Druid", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1299::RULES.keywords, deck_limit: authored::card_1299::RULES.deck_limit }, effect: authored::card_1299::RULES.checked_effect(), triggers: authored::card_1299::RULES.triggers, activated: authored::card_1299::RULES.activated, enters: authored::card_1299::RULES.enters, entry: authored::card_1299::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 2, 0], generic: 1 }, rules: &authored::card_1299::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Guttersnipe", oracle_id: Some("c6bdaf76-6a03-4695-9c4b-f040e73435af"), mana_cost: "{2}{R}", type_line: "Creature — Goblin Shaman", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1915::RULES.keywords, deck_limit: authored::card_1915::RULES.deck_limit }, effect: authored::card_1915::RULES.checked_effect(), triggers: authored::card_1915::RULES.triggers, activated: authored::card_1915::RULES.activated, enters: authored::card_1915::RULES.enters, entry: authored::card_1915::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 2 }, rules: &authored::card_1915::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Mentor of the Meek", oracle_id: Some("b9f4f96b-6e54-4fe6-8df7-623e0fc72409"), mana_cost: "{2}{W}", type_line: "Creature — Human Soldier", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2720::RULES.keywords, deck_limit: authored::card_2720::RULES.deck_limit }, effect: authored::card_2720::RULES.checked_effect(), triggers: authored::card_2720::RULES.triggers, activated: authored::card_2720::RULES.activated, enters: authored::card_2720::RULES.enters, entry: authored::card_2720::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_2720::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Rogue's Passage", oracle_id: Some("f29dc596-2121-4421-8463-15f6c2e8b9b3"), mana_cost: "", type_line: "Land", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3576::RULES.keywords, deck_limit: authored::card_3576::RULES.deck_limit }, effect: authored::card_3576::RULES.checked_effect(), triggers: authored::card_3576::RULES.triggers, activated: authored::card_3576::RULES.activated, enters: authored::card_3576::RULES.enters, entry: authored::card_3576::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_3576::RULES },
];
}
pub use cards::ENTRIES;
