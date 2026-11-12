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
pub(super) mod card_0005 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/legacy/grizzly_bears.rs"));
const _: () = assert!(matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0006 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/legacy/goblin_piker.rs"));
const _: () = assert!(matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0007 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/legacy/hill_giant.rs"));
const _: () = assert!(matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0009 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/legacy/lightning_bolt.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0012 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/legacy/divination.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0013 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/legacy/counterspell.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0304 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/legacy/badgermole_cub.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1842 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/legacy/gran_gran.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
}
#[rustfmt::skip]
pub static ENTRIES: [CatalogueEntry; 8] = [
CatalogueEntry { definition: Definition { info: CardInfo { name: "Grizzly Bears", oracle_id: None, mana_cost: "{1}{G}", type_line: "Creature — Bear", oracle_keywords: &[], standard_legal: false, legendary: false, implemented: true, keywords: authored::card_0005::RULES.keywords, deck_limit: authored::card_0005::RULES.deck_limit }, effect: authored::card_0005::RULES.checked_effect(), triggers: authored::card_0005::RULES.triggers, activated: authored::card_0005::RULES.activated, enters: authored::card_0005::RULES.enters, entry: authored::card_0005::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 1 }, rules: &authored::card_0005::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Goblin Piker", oracle_id: None, mana_cost: "{1}{R}", type_line: "Creature — Goblin Warrior", oracle_keywords: &[], standard_legal: false, legendary: false, implemented: true, keywords: authored::card_0006::RULES.keywords, deck_limit: authored::card_0006::RULES.deck_limit }, effect: authored::card_0006::RULES.checked_effect(), triggers: authored::card_0006::RULES.triggers, activated: authored::card_0006::RULES.activated, enters: authored::card_0006::RULES.enters, entry: authored::card_0006::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 1 }, rules: &authored::card_0006::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Hill Giant", oracle_id: None, mana_cost: "{3}{R}", type_line: "Creature — Giant", oracle_keywords: &[], standard_legal: false, legendary: false, implemented: true, keywords: authored::card_0007::RULES.keywords, deck_limit: authored::card_0007::RULES.deck_limit }, effect: authored::card_0007::RULES.checked_effect(), triggers: authored::card_0007::RULES.triggers, activated: authored::card_0007::RULES.activated, enters: authored::card_0007::RULES.enters, entry: authored::card_0007::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 3 }, rules: &authored::card_0007::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Lightning Bolt", oracle_id: None, mana_cost: "{R}", type_line: "Instant", oracle_keywords: &[], standard_legal: false, legendary: false, implemented: true, keywords: authored::card_0009::RULES.keywords, deck_limit: authored::card_0009::RULES.deck_limit }, effect: authored::card_0009::RULES.checked_effect(), triggers: authored::card_0009::RULES.triggers, activated: authored::card_0009::RULES.activated, enters: authored::card_0009::RULES.enters, entry: authored::card_0009::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 0 }, rules: &authored::card_0009::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Divination", oracle_id: None, mana_cost: "{2}{U}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: false, legendary: false, implemented: true, keywords: authored::card_0012::RULES.keywords, deck_limit: authored::card_0012::RULES.deck_limit }, effect: authored::card_0012::RULES.checked_effect(), triggers: authored::card_0012::RULES.triggers, activated: authored::card_0012::RULES.activated, enters: authored::card_0012::RULES.enters, entry: authored::card_0012::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_0012::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Counterspell", oracle_id: None, mana_cost: "{U}{U}", type_line: "Instant", oracle_keywords: &[], standard_legal: false, legendary: false, implemented: true, keywords: authored::card_0013::RULES.keywords, deck_limit: authored::card_0013::RULES.deck_limit }, effect: authored::card_0013::RULES.checked_effect(), triggers: authored::card_0013::RULES.triggers, activated: authored::card_0013::RULES.activated, enters: authored::card_0013::RULES.enters, entry: authored::card_0013::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 2, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_0013::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Badgermole Cub", oracle_id: None, mana_cost: "", type_line: "", oracle_keywords: &[], standard_legal: false, legendary: false, implemented: false, keywords: authored::card_0304::RULES.keywords, deck_limit: authored::card_0304::RULES.deck_limit }, effect: authored::card_0304::RULES.effect, triggers: authored::card_0304::RULES.triggers, activated: authored::card_0304::RULES.activated, enters: authored::card_0304::RULES.enters, entry: authored::card_0304::RULES.entry, kind: 0, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_0304::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Gran-Gran", oracle_id: None, mana_cost: "", type_line: "", oracle_keywords: &[], standard_legal: false, legendary: false, implemented: false, keywords: authored::card_1842::RULES.keywords, deck_limit: authored::card_1842::RULES.deck_limit }, effect: authored::card_1842::RULES.effect, triggers: authored::card_1842::RULES.triggers, activated: authored::card_1842::RULES.activated, enters: authored::card_1842::RULES.enters, entry: authored::card_1842::RULES.entry, kind: 0, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_1842::RULES },
];
}
pub use cards::ENTRIES;
