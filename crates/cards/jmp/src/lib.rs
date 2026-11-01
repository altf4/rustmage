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
pub(super) mod card_0165 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/jmp/angelic_edict.rs"));
}
pub(super) mod card_4839 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/jmp/vampire_neonate.rs"));
}
}
#[rustfmt::skip]
pub static ENTRIES: [CatalogueEntry; 2] = [
CatalogueEntry { definition: Definition { info: CardInfo { name: "Angelic Edict", oracle_id: Some("36de8401-ee4e-413f-91ad-06924e39c857"), mana_cost: "{4}{W}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0165::RULES.keywords, deck_limit: authored::card_0165::RULES.deck_limit }, effect: authored::card_0165::RULES.checked_effect(), triggers: authored::card_0165::RULES.triggers, activated: authored::card_0165::RULES.activated, enters: authored::card_0165::RULES.enters, entry: authored::card_0165::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 4 }, rules: &authored::card_0165::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Vampire Neonate", oracle_id: Some("c0045925-25bf-403c-b721-5f05ba30985b"), mana_cost: "{B}", type_line: "Creature — Vampire", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4839::RULES.keywords, deck_limit: authored::card_4839::RULES.deck_limit }, effect: authored::card_4839::RULES.checked_effect(), triggers: authored::card_4839::RULES.triggers, activated: authored::card_4839::RULES.activated, enters: authored::card_4839::RULES.enters, entry: authored::card_4839::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 0 }, rules: &authored::card_4839::RULES },
];
}
pub use cards::ENTRIES;
