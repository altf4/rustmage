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
pub(super) mod card_0680 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/inr/cathar_commando.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1247 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/inr/eaten_alive.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2152 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/inr/imprisoned_in_the_moon.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2611 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/inr/maelstrom_pulse.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
}
#[rustfmt::skip]
pub static ENTRIES: [CatalogueEntry; 4] = [
CatalogueEntry { definition: Definition { info: CardInfo { name: "Cathar Commando", oracle_id: Some("774dce79-67e0-4820-8013-c7a7347993ce"), mana_cost: "{1}{W}", type_line: "Creature — Human Soldier", oracle_keywords: &["Flash"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0680::RULES.keywords, deck_limit: authored::card_0680::RULES.deck_limit }, effect: authored::card_0680::RULES.checked_effect(), triggers: authored::card_0680::RULES.triggers, activated: authored::card_0680::RULES.activated, enters: authored::card_0680::RULES.enters, entry: authored::card_0680::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_0680::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Eaten Alive", oracle_id: Some("d437ecc2-2fd3-4ad3-b23e-217f55e58dae"), mana_cost: "{B}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1247::RULES.keywords, deck_limit: authored::card_1247::RULES.deck_limit }, effect: authored::card_1247::RULES.checked_effect(), triggers: authored::card_1247::RULES.triggers, activated: authored::card_1247::RULES.activated, enters: authored::card_1247::RULES.enters, entry: authored::card_1247::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 0 }, rules: &authored::card_1247::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Imprisoned in the Moon", oracle_id: Some("339a7418-9a29-4eae-a3c2-ea590d175936"), mana_cost: "{2}{U}", type_line: "Enchantment — Aura", oracle_keywords: &["Enchant"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2152::RULES.keywords, deck_limit: authored::card_2152::RULES.deck_limit }, effect: authored::card_2152::RULES.checked_effect(), triggers: authored::card_2152::RULES.triggers, activated: authored::card_2152::RULES.activated, enters: authored::card_2152::RULES.enters, entry: authored::card_2152::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_2152::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Maelstrom Pulse", oracle_id: Some("95ce305f-34bc-4d6d-b7ba-ffd4b2a25336"), mana_cost: "{1}{B}{G}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2611::RULES.keywords, deck_limit: authored::card_2611::RULES.deck_limit }, effect: authored::card_2611::RULES.checked_effect(), triggers: authored::card_2611::RULES.triggers, activated: authored::card_2611::RULES.activated, enters: authored::card_2611::RULES.enters, entry: authored::card_2611::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 1, 0, 1, 0], generic: 1 }, rules: &authored::card_2611::RULES },
];
}
pub use cards::ENTRIES;
