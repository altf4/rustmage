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
pub(super) mod card_1421 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/frc/fabled_passage.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3074 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/frc/overlord_of_the_mistmoors.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3483 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/frc/restless_anchorage.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3490 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/frc/restless_spire.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4190 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/frc/stroke_of_midnight.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_5076 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/frc/windcrag_siege.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
}
#[rustfmt::skip]
pub static ENTRIES: [CatalogueEntry; 6] = [
CatalogueEntry { definition: Definition { info: CardInfo { name: "Fabled Passage", oracle_id: Some("0c85b8f7-0bd0-4680-9ec5-d4b110460a54"), mana_cost: "", type_line: "Land", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1421::RULES.keywords, deck_limit: authored::card_1421::RULES.deck_limit }, effect: authored::card_1421::RULES.checked_effect(), triggers: authored::card_1421::RULES.triggers, activated: authored::card_1421::RULES.activated, enters: authored::card_1421::RULES.enters, entry: authored::card_1421::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_1421::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Overlord of the Mistmoors", oracle_id: Some("7e64b1dc-a238-4bff-98ff-2bea44340568"), mana_cost: "{5}{W}{W}", type_line: "Enchantment Creature — Avatar Horror", oracle_keywords: &["Impending"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_3074::RULES.keywords, deck_limit: authored::card_3074::RULES.deck_limit }, effect: authored::card_3074::RULES.effect, triggers: authored::card_3074::RULES.triggers, activated: authored::card_3074::RULES.activated, enters: authored::card_3074::RULES.enters, entry: authored::card_3074::RULES.entry, kind: 33, basic: false, mana_color: None, colored: [2, 0, 0, 0, 0, 0], generic: 5 }, rules: &authored::card_3074::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Restless Anchorage", oracle_id: Some("91320daf-f69c-4350-b0fc-4bb37a6904b1"), mana_cost: "", type_line: "Land", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_3483::RULES.keywords, deck_limit: authored::card_3483::RULES.deck_limit }, effect: authored::card_3483::RULES.effect, triggers: authored::card_3483::RULES.triggers, activated: authored::card_3483::RULES.activated, enters: authored::card_3483::RULES.enters, entry: authored::card_3483::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_3483::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Restless Spire", oracle_id: Some("0ca4e80e-c19c-4b74-b531-c5a4dc5a8ba9"), mana_cost: "", type_line: "Land", oracle_keywords: &["Scry"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_3490::RULES.keywords, deck_limit: authored::card_3490::RULES.deck_limit }, effect: authored::card_3490::RULES.effect, triggers: authored::card_3490::RULES.triggers, activated: authored::card_3490::RULES.activated, enters: authored::card_3490::RULES.enters, entry: authored::card_3490::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_3490::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Stroke of Midnight", oracle_id: Some("9a107e48-3d50-4941-95b1-10f2b29a4245"), mana_cost: "{2}{W}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4190::RULES.keywords, deck_limit: authored::card_4190::RULES.deck_limit }, effect: authored::card_4190::RULES.checked_effect(), triggers: authored::card_4190::RULES.triggers, activated: authored::card_4190::RULES.activated, enters: authored::card_4190::RULES.enters, entry: authored::card_4190::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_4190::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Windcrag Siege", oracle_id: Some("64df560a-905f-45d1-bc70-14d99e3112d3"), mana_cost: "{1}{R}{W}", type_line: "Enchantment", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_5076::RULES.keywords, deck_limit: authored::card_5076::RULES.deck_limit }, effect: authored::card_5076::RULES.effect, triggers: authored::card_5076::RULES.triggers, activated: authored::card_5076::RULES.activated, enters: authored::card_5076::RULES.enters, entry: authored::card_5076::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [1, 0, 0, 1, 0, 0], generic: 1 }, rules: &authored::card_5076::RULES },
];
}
pub use cards::ENTRIES;
