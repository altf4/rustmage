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
pub(super) mod card_0737 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/ecc/chimil_the_inner_sun.rs"));
}
pub(super) mod card_1633 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/ecc/frontier_bivouac.rs"));
}
pub(super) mod card_1686 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/ecc/garruk_s_uprising.rs"));
}
pub(super) mod card_2676 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/ecc/massacre_girl_known_killer.rs"));
}
pub(super) mod card_2874 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/ecc/muldrotha_the_gravetide.rs"));
}
pub(super) mod card_3029 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/ecc/opulent_palace.rs"));
}
pub(super) mod card_3679 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/ecc/sandsteppe_citadel.rs"));
}
}
#[rustfmt::skip]
pub static ENTRIES: [CatalogueEntry; 7] = [
CatalogueEntry { definition: Definition { info: CardInfo { name: "Chimil, the Inner Sun", oracle_id: Some("119d9671-61ba-4629-89b1-f94bdec5cb74"), mana_cost: "{6}", type_line: "Legendary Artifact", oracle_keywords: &["Discover"], standard_legal: true, legendary: true, implemented: false, keywords: authored::card_0737::RULES.keywords, deck_limit: authored::card_0737::RULES.deck_limit }, effect: authored::card_0737::RULES.effect, triggers: authored::card_0737::RULES.triggers, activated: authored::card_0737::RULES.activated, enters: authored::card_0737::RULES.enters, entry: authored::card_0737::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 6 }, rules: &authored::card_0737::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Frontier Bivouac", oracle_id: Some("e4cf6c2f-0f1e-4980-9ef9-e4eabcae42a9"), mana_cost: "", type_line: "Land", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1633::RULES.keywords, deck_limit: authored::card_1633::RULES.deck_limit }, effect: authored::card_1633::RULES.checked_effect(), triggers: authored::card_1633::RULES.triggers, activated: authored::card_1633::RULES.activated, enters: authored::card_1633::RULES.enters, entry: authored::card_1633::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_1633::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Garruk's Uprising", oracle_id: Some("3127ae9b-a7a7-43ec-89d7-688f8445b33d"), mana_cost: "{2}{G}", type_line: "Enchantment", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1686::RULES.keywords, deck_limit: authored::card_1686::RULES.deck_limit }, effect: authored::card_1686::RULES.checked_effect(), triggers: authored::card_1686::RULES.triggers, activated: authored::card_1686::RULES.activated, enters: authored::card_1686::RULES.enters, entry: authored::card_1686::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 2 }, rules: &authored::card_1686::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Massacre Girl, Known Killer", oracle_id: Some("ccb39f4a-2684-4eb7-9066-aad81ae08962"), mana_cost: "{2}{B}{B}", type_line: "Legendary Creature — Human Assassin", oracle_keywords: &["Menace"], standard_legal: true, legendary: true, implemented: false, keywords: authored::card_2676::RULES.keywords, deck_limit: authored::card_2676::RULES.deck_limit }, effect: authored::card_2676::RULES.effect, triggers: authored::card_2676::RULES.triggers, activated: authored::card_2676::RULES.activated, enters: authored::card_2676::RULES.enters, entry: authored::card_2676::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 2, 0, 0, 0], generic: 2 }, rules: &authored::card_2676::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Muldrotha, the Gravetide", oracle_id: Some("e4625704-1d52-44e4-804f-2f45644d76ac"), mana_cost: "{3}{B}{G}{U}", type_line: "Legendary Creature — Elemental Avatar", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2874::RULES.keywords, deck_limit: authored::card_2874::RULES.deck_limit }, effect: authored::card_2874::RULES.checked_effect(), triggers: authored::card_2874::RULES.triggers, activated: authored::card_2874::RULES.activated, enters: authored::card_2874::RULES.enters, entry: authored::card_2874::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 1, 0, 1, 0], generic: 3 }, rules: &authored::card_2874::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Opulent Palace", oracle_id: Some("f9e7e855-1e3b-42d3-91b0-64ba8b5b8982"), mana_cost: "", type_line: "Land", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3029::RULES.keywords, deck_limit: authored::card_3029::RULES.deck_limit }, effect: authored::card_3029::RULES.checked_effect(), triggers: authored::card_3029::RULES.triggers, activated: authored::card_3029::RULES.activated, enters: authored::card_3029::RULES.enters, entry: authored::card_3029::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_3029::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Sandsteppe Citadel", oracle_id: Some("544dbabd-cbfc-40da-a5ba-2fea9cddb453"), mana_cost: "", type_line: "Land", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3679::RULES.keywords, deck_limit: authored::card_3679::RULES.deck_limit }, effect: authored::card_3679::RULES.checked_effect(), triggers: authored::card_3679::RULES.triggers, activated: authored::card_3679::RULES.activated, enters: authored::card_3679::RULES.enters, entry: authored::card_3679::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_3679::RULES },
];
}
pub use cards::ENTRIES;
