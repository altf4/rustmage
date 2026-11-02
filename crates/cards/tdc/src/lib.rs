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
pub(super) mod card_0426 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/tdc/bitter_triumph.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0838 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/tdc/consuming_aberration.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1164 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/tdc/dragonlord_s_servant.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1165 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/tdc/dragonmaster_outcast.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1403 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/tdc/expel_the_interlopers.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2453 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/tdc/lathliss_dragon_queen.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3027 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/tdc/opt.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3374 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/tdc/rapacious_dragon.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3417 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/tdc/reassembling_skeleton.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3449 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/tdc/release_the_dogs.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4127 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/tdc/steel_hellkite.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4346 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/tdc/taurean_mauler.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4362 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/tdc/temple_of_abandon.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4369 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/tdc/temple_of_plenty.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_5159 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/tdc/zetalpa_primal_dawn.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
}
#[rustfmt::skip]
pub static ENTRIES: [CatalogueEntry; 15] = [
CatalogueEntry { definition: Definition { info: CardInfo { name: "Bitter Triumph", oracle_id: Some("776341cb-d2ec-423f-9250-92dc8bd8d503"), mana_cost: "{1}{B}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0426::RULES.keywords, deck_limit: authored::card_0426::RULES.deck_limit }, effect: authored::card_0426::RULES.checked_effect(), triggers: authored::card_0426::RULES.triggers, activated: authored::card_0426::RULES.activated, enters: authored::card_0426::RULES.enters, entry: authored::card_0426::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 1 }, rules: &authored::card_0426::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Consuming Aberration", oracle_id: Some("9b55fb72-237d-4935-b645-8ebc6eb4140e"), mana_cost: "{3}{U}{B}", type_line: "Creature — Horror", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0838::RULES.keywords, deck_limit: authored::card_0838::RULES.deck_limit }, effect: authored::card_0838::RULES.checked_effect(), triggers: authored::card_0838::RULES.triggers, activated: authored::card_0838::RULES.activated, enters: authored::card_0838::RULES.enters, entry: authored::card_0838::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 1, 0, 0, 0], generic: 3 }, rules: &authored::card_0838::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Dragonlord's Servant", oracle_id: Some("faf81ae0-0b35-45ef-a50d-6cecf0c0eeeb"), mana_cost: "{1}{R}", type_line: "Creature — Goblin Shaman", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1164::RULES.keywords, deck_limit: authored::card_1164::RULES.deck_limit }, effect: authored::card_1164::RULES.checked_effect(), triggers: authored::card_1164::RULES.triggers, activated: authored::card_1164::RULES.activated, enters: authored::card_1164::RULES.enters, entry: authored::card_1164::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 1 }, rules: &authored::card_1164::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Dragonmaster Outcast", oracle_id: Some("b6fb79c3-cd32-4045-8177-e52841eea65b"), mana_cost: "{R}", type_line: "Creature — Human Shaman", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1165::RULES.keywords, deck_limit: authored::card_1165::RULES.deck_limit }, effect: authored::card_1165::RULES.checked_effect(), triggers: authored::card_1165::RULES.triggers, activated: authored::card_1165::RULES.activated, enters: authored::card_1165::RULES.enters, entry: authored::card_1165::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 0 }, rules: &authored::card_1165::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Expel the Interlopers", oracle_id: Some("744b7e93-c893-4770-a3bb-6a45b7f8e4f7"), mana_cost: "{3}{W}{W}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_1403::RULES.keywords, deck_limit: authored::card_1403::RULES.deck_limit }, effect: authored::card_1403::RULES.effect, triggers: authored::card_1403::RULES.triggers, activated: authored::card_1403::RULES.activated, enters: authored::card_1403::RULES.enters, entry: authored::card_1403::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [2, 0, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_1403::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Lathliss, Dragon Queen", oracle_id: Some("15fef45d-f1a9-49b2-abaa-fe77bb9d1afd"), mana_cost: "{4}{R}{R}", type_line: "Legendary Creature — Dragon", oracle_keywords: &["Flying"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2453::RULES.keywords, deck_limit: authored::card_2453::RULES.deck_limit }, effect: authored::card_2453::RULES.checked_effect(), triggers: authored::card_2453::RULES.triggers, activated: authored::card_2453::RULES.activated, enters: authored::card_2453::RULES.enters, entry: authored::card_2453::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 2, 0, 0], generic: 4 }, rules: &authored::card_2453::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Opt", oracle_id: Some("713332c1-5bd8-400f-bfff-c1ca0697a043"), mana_cost: "{U}", type_line: "Instant", oracle_keywords: &["Scry"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3027::RULES.keywords, deck_limit: authored::card_3027::RULES.deck_limit }, effect: authored::card_3027::RULES.checked_effect(), triggers: authored::card_3027::RULES.triggers, activated: authored::card_3027::RULES.activated, enters: authored::card_3027::RULES.enters, entry: authored::card_3027::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_3027::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Rapacious Dragon", oracle_id: Some("0944ec2e-1dd9-459f-8f1d-667242cf52fe"), mana_cost: "{4}{R}", type_line: "Creature — Dragon", oracle_keywords: &["Flying", "Treasure"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3374::RULES.keywords, deck_limit: authored::card_3374::RULES.deck_limit }, effect: authored::card_3374::RULES.checked_effect(), triggers: authored::card_3374::RULES.triggers, activated: authored::card_3374::RULES.activated, enters: authored::card_3374::RULES.enters, entry: authored::card_3374::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 4 }, rules: &authored::card_3374::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Reassembling Skeleton", oracle_id: Some("9dbc3530-b278-4c8d-b2cc-a09dfac9d5e5"), mana_cost: "{1}{B}", type_line: "Creature — Skeleton Warrior", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3417::RULES.keywords, deck_limit: authored::card_3417::RULES.deck_limit }, effect: authored::card_3417::RULES.checked_effect(), triggers: authored::card_3417::RULES.triggers, activated: authored::card_3417::RULES.activated, enters: authored::card_3417::RULES.enters, entry: authored::card_3417::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 1 }, rules: &authored::card_3417::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Release the Dogs", oracle_id: Some("b97d21b4-d730-479f-872f-3e1645b66751"), mana_cost: "{3}{W}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3449::RULES.keywords, deck_limit: authored::card_3449::RULES.deck_limit }, effect: authored::card_3449::RULES.checked_effect(), triggers: authored::card_3449::RULES.triggers, activated: authored::card_3449::RULES.activated, enters: authored::card_3449::RULES.enters, entry: authored::card_3449::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_3449::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Steel Hellkite", oracle_id: Some("6e64f1e0-ddd6-4ee7-8797-086e234b02e8"), mana_cost: "{6}", type_line: "Artifact Creature — Dragon", oracle_keywords: &["Flying"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4127::RULES.keywords, deck_limit: authored::card_4127::RULES.deck_limit }, effect: authored::card_4127::RULES.checked_effect(), triggers: authored::card_4127::RULES.triggers, activated: authored::card_4127::RULES.activated, enters: authored::card_4127::RULES.enters, entry: authored::card_4127::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 6 }, rules: &authored::card_4127::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Taurean Mauler", oracle_id: Some("fd5ee26c-cc54-4dc0-b611-5cda14354a5e"), mana_cost: "{2}{R}", type_line: "Creature — Shapeshifter", oracle_keywords: &["Changeling"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4346::RULES.keywords, deck_limit: authored::card_4346::RULES.deck_limit }, effect: authored::card_4346::RULES.checked_effect(), triggers: authored::card_4346::RULES.triggers, activated: authored::card_4346::RULES.activated, enters: authored::card_4346::RULES.enters, entry: authored::card_4346::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 2 }, rules: &authored::card_4346::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Temple of Abandon", oracle_id: Some("3baa8e38-ef93-435d-b63e-f781d5bfcc68"), mana_cost: "", type_line: "Land", oracle_keywords: &["Scry"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4362::RULES.keywords, deck_limit: authored::card_4362::RULES.deck_limit }, effect: authored::card_4362::RULES.checked_effect(), triggers: authored::card_4362::RULES.triggers, activated: authored::card_4362::RULES.activated, enters: authored::card_4362::RULES.enters, entry: authored::card_4362::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_4362::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Temple of Plenty", oracle_id: Some("e521322b-0e83-458c-8936-7021a80ee279"), mana_cost: "", type_line: "Land", oracle_keywords: &["Scry"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4369::RULES.keywords, deck_limit: authored::card_4369::RULES.deck_limit }, effect: authored::card_4369::RULES.checked_effect(), triggers: authored::card_4369::RULES.triggers, activated: authored::card_4369::RULES.activated, enters: authored::card_4369::RULES.enters, entry: authored::card_4369::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_4369::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Zetalpa, Primal Dawn", oracle_id: Some("7da0e5de-3e4c-420a-8685-991206100b9d"), mana_cost: "{6}{W}{W}", type_line: "Legendary Creature — Elder Dinosaur", oracle_keywords: &["Flying", "Indestructible", "Vigilance", "Trample", "Double strike"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_5159::RULES.keywords, deck_limit: authored::card_5159::RULES.deck_limit }, effect: authored::card_5159::RULES.checked_effect(), triggers: authored::card_5159::RULES.triggers, activated: authored::card_5159::RULES.activated, enters: authored::card_5159::RULES.enters, entry: authored::card_5159::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [2, 0, 0, 0, 0, 0], generic: 6 }, rules: &authored::card_5159::RULES },
];
}
pub use cards::ENTRIES;
