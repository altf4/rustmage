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
pub(super) mod card_0078 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoc/aftermath_analyst.rs"));
}
pub(super) mod card_1648 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoc/fumigate.rs"));
}
pub(super) mod card_3362 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoc/rampaging_baloths.rs"));
}
pub(super) mod card_3975 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoc/soul_guide_lantern.rs"));
}
pub(super) mod card_4364 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoc/temple_of_enlightenment.rs"));
}
pub(super) mod card_4571 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoc/threefold_thunderhulk.rs"));
}
pub(super) mod card_4991 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoc/wastes.rs"));
}
pub(super) mod card_5115 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoc/worldsoul_s_rage.rs"));
}
}
#[rustfmt::skip]
pub static ENTRIES: [CatalogueEntry; 8] = [
CatalogueEntry { definition: Definition { info: CardInfo { name: "Aftermath Analyst", oracle_id: Some("374e54d8-8e73-4268-9dde-c28c77bbbf32"), mana_cost: "{1}{G}", type_line: "Creature — Elf Detective", oracle_keywords: &["Mill"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_0078::RULES.keywords, deck_limit: authored::card_0078::RULES.deck_limit }, effect: authored::card_0078::RULES.effect, triggers: authored::card_0078::RULES.triggers, activated: authored::card_0078::RULES.activated, enters: authored::card_0078::RULES.enters, entry: authored::card_0078::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 1 }, rules: &authored::card_0078::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Fumigate", oracle_id: Some("b17ea905-0696-4e58-b564-557e87236e27"), mana_cost: "{3}{W}{W}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1648::RULES.keywords, deck_limit: authored::card_1648::RULES.deck_limit }, effect: authored::card_1648::RULES.checked_effect(), triggers: authored::card_1648::RULES.triggers, activated: authored::card_1648::RULES.activated, enters: authored::card_1648::RULES.enters, entry: authored::card_1648::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [2, 0, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_1648::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Rampaging Baloths", oracle_id: Some("2d3e6549-6cc6-434f-a189-ba3b55e64c34"), mana_cost: "{4}{G}{G}", type_line: "Creature — Beast", oracle_keywords: &["Trample", "Landfall"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3362::RULES.keywords, deck_limit: authored::card_3362::RULES.deck_limit }, effect: authored::card_3362::RULES.checked_effect(), triggers: authored::card_3362::RULES.triggers, activated: authored::card_3362::RULES.activated, enters: authored::card_3362::RULES.enters, entry: authored::card_3362::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 2, 0], generic: 4 }, rules: &authored::card_3362::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Soul-Guide Lantern", oracle_id: Some("1b5e6560-ff2e-4475-96cb-63f64c8a86db"), mana_cost: "{1}", type_line: "Artifact", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3975::RULES.keywords, deck_limit: authored::card_3975::RULES.deck_limit }, effect: authored::card_3975::RULES.checked_effect(), triggers: authored::card_3975::RULES.triggers, activated: authored::card_3975::RULES.activated, enters: authored::card_3975::RULES.enters, entry: authored::card_3975::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_3975::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Temple of Enlightenment", oracle_id: Some("89f43e27-790b-4ca1-8ba7-0882b31e0783"), mana_cost: "", type_line: "Land", oracle_keywords: &["Scry"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4364::RULES.keywords, deck_limit: authored::card_4364::RULES.deck_limit }, effect: authored::card_4364::RULES.checked_effect(), triggers: authored::card_4364::RULES.triggers, activated: authored::card_4364::RULES.activated, enters: authored::card_4364::RULES.enters, entry: authored::card_4364::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_4364::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Threefold Thunderhulk", oracle_id: Some("b8020a8b-557b-465d-865d-b59fecd7abc1"), mana_cost: "{7}", type_line: "Artifact Creature — Gnome", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_4571::RULES.keywords, deck_limit: authored::card_4571::RULES.deck_limit }, effect: authored::card_4571::RULES.effect, triggers: authored::card_4571::RULES.triggers, activated: authored::card_4571::RULES.activated, enters: authored::card_4571::RULES.enters, entry: authored::card_4571::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 7 }, rules: &authored::card_4571::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Wastes", oracle_id: Some("05d24b0c-904a-46b6-b42a-96a4d91a0dd4"), mana_cost: "", type_line: "Basic Land", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4991::RULES.keywords, deck_limit: authored::card_4991::RULES.deck_limit }, effect: authored::card_4991::RULES.checked_effect(), triggers: authored::card_4991::RULES.triggers, activated: authored::card_4991::RULES.activated, enters: authored::card_4991::RULES.enters, entry: authored::card_4991::RULES.entry, kind: 2, basic: true, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_4991::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Worldsoul's Rage", oracle_id: Some("3b6ef144-bb98-4686-9718-204f1c3cf020"), mana_cost: "{X}{R}{G}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_5115::RULES.keywords, deck_limit: authored::card_5115::RULES.deck_limit }, effect: authored::card_5115::RULES.checked_effect(), triggers: authored::card_5115::RULES.triggers, activated: authored::card_5115::RULES.activated, enters: authored::card_5115::RULES.enters, entry: authored::card_5115::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 0, 1, 1, 0], generic: 0 }, rules: &authored::card_5115::RULES },
];
}
pub use cards::ENTRIES;
