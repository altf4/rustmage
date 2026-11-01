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
}
pub(super) mod card_0006 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/legacy/goblin_piker.rs"));
}
pub(super) mod card_0007 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/legacy/hill_giant.rs"));
}
pub(super) mod card_0009 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/legacy/lightning_bolt.rs"));
}
pub(super) mod card_0012 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/legacy/divination.rs"));
}
pub(super) mod card_0013 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/legacy/counterspell.rs"));
}
pub(super) mod card_0040 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/legacy/abuelo_s_awakening.rs"));
}
pub(super) mod card_0304 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/legacy/badgermole_cub.rs"));
}
pub(super) mod card_0849 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/legacy/cori_steel_cutter.rs"));
}
pub(super) mod card_1842 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/legacy/gran_gran.rs"));
}
pub(super) mod card_1977 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/legacy/heartfire_hero.rs"));
}
pub(super) mod card_2072 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/legacy/hopeless_nightmare.rs"));
}
pub(super) mod card_2837 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/legacy/monstrous_rage.rs"));
}
pub(super) mod card_3251 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/legacy/proft_s_eidetic_memory.rs"));
}
pub(super) mod card_3739 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/legacy/screaming_nemesis.rs"));
}
pub(super) mod card_4173 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/legacy/stormchaser_s_talent.rs"));
}
pub(super) mod card_4534 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/legacy/this_town_ain_t_big_enough.rs"));
}
pub(super) mod card_4812 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/legacy/up_the_beanstalk.rs"));
}
pub(super) mod card_4923 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/legacy/vivi_ornitier.rs"));
}
}
#[rustfmt::skip]
pub static ENTRIES: [CatalogueEntry; 19] = [
CatalogueEntry { definition: Definition { info: CardInfo { name: "Grizzly Bears", oracle_id: None, mana_cost: "{1}{G}", type_line: "Creature — Bear", oracle_keywords: &[], standard_legal: false, legendary: false, implemented: true, keywords: authored::card_0005::RULES.keywords, deck_limit: authored::card_0005::RULES.deck_limit }, effect: authored::card_0005::RULES.checked_effect(), triggers: authored::card_0005::RULES.triggers, activated: authored::card_0005::RULES.activated, enters: authored::card_0005::RULES.enters, entry: authored::card_0005::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 1 }, rules: &authored::card_0005::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Goblin Piker", oracle_id: None, mana_cost: "{1}{R}", type_line: "Creature — Goblin Warrior", oracle_keywords: &[], standard_legal: false, legendary: false, implemented: true, keywords: authored::card_0006::RULES.keywords, deck_limit: authored::card_0006::RULES.deck_limit }, effect: authored::card_0006::RULES.checked_effect(), triggers: authored::card_0006::RULES.triggers, activated: authored::card_0006::RULES.activated, enters: authored::card_0006::RULES.enters, entry: authored::card_0006::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 1 }, rules: &authored::card_0006::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Hill Giant", oracle_id: None, mana_cost: "{3}{R}", type_line: "Creature — Giant", oracle_keywords: &[], standard_legal: false, legendary: false, implemented: true, keywords: authored::card_0007::RULES.keywords, deck_limit: authored::card_0007::RULES.deck_limit }, effect: authored::card_0007::RULES.checked_effect(), triggers: authored::card_0007::RULES.triggers, activated: authored::card_0007::RULES.activated, enters: authored::card_0007::RULES.enters, entry: authored::card_0007::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 3 }, rules: &authored::card_0007::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Lightning Bolt", oracle_id: None, mana_cost: "{R}", type_line: "Instant", oracle_keywords: &[], standard_legal: false, legendary: false, implemented: true, keywords: authored::card_0009::RULES.keywords, deck_limit: authored::card_0009::RULES.deck_limit }, effect: authored::card_0009::RULES.checked_effect(), triggers: authored::card_0009::RULES.triggers, activated: authored::card_0009::RULES.activated, enters: authored::card_0009::RULES.enters, entry: authored::card_0009::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 0 }, rules: &authored::card_0009::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Divination", oracle_id: None, mana_cost: "{2}{U}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: false, legendary: false, implemented: true, keywords: authored::card_0012::RULES.keywords, deck_limit: authored::card_0012::RULES.deck_limit }, effect: authored::card_0012::RULES.checked_effect(), triggers: authored::card_0012::RULES.triggers, activated: authored::card_0012::RULES.activated, enters: authored::card_0012::RULES.enters, entry: authored::card_0012::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_0012::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Counterspell", oracle_id: None, mana_cost: "{U}{U}", type_line: "Instant", oracle_keywords: &[], standard_legal: false, legendary: false, implemented: true, keywords: authored::card_0013::RULES.keywords, deck_limit: authored::card_0013::RULES.deck_limit }, effect: authored::card_0013::RULES.checked_effect(), triggers: authored::card_0013::RULES.triggers, activated: authored::card_0013::RULES.activated, enters: authored::card_0013::RULES.enters, entry: authored::card_0013::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 2, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_0013::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Abuelo's Awakening", oracle_id: None, mana_cost: "", type_line: "", oracle_keywords: &[], standard_legal: false, legendary: false, implemented: false, keywords: authored::card_0040::RULES.keywords, deck_limit: authored::card_0040::RULES.deck_limit }, effect: authored::card_0040::RULES.effect, triggers: authored::card_0040::RULES.triggers, activated: authored::card_0040::RULES.activated, enters: authored::card_0040::RULES.enters, entry: authored::card_0040::RULES.entry, kind: 0, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_0040::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Badgermole Cub", oracle_id: None, mana_cost: "", type_line: "", oracle_keywords: &[], standard_legal: false, legendary: false, implemented: false, keywords: authored::card_0304::RULES.keywords, deck_limit: authored::card_0304::RULES.deck_limit }, effect: authored::card_0304::RULES.effect, triggers: authored::card_0304::RULES.triggers, activated: authored::card_0304::RULES.activated, enters: authored::card_0304::RULES.enters, entry: authored::card_0304::RULES.entry, kind: 0, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_0304::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Cori-Steel Cutter", oracle_id: None, mana_cost: "", type_line: "", oracle_keywords: &[], standard_legal: false, legendary: false, implemented: false, keywords: authored::card_0849::RULES.keywords, deck_limit: authored::card_0849::RULES.deck_limit }, effect: authored::card_0849::RULES.effect, triggers: authored::card_0849::RULES.triggers, activated: authored::card_0849::RULES.activated, enters: authored::card_0849::RULES.enters, entry: authored::card_0849::RULES.entry, kind: 0, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_0849::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Gran-Gran", oracle_id: None, mana_cost: "", type_line: "", oracle_keywords: &[], standard_legal: false, legendary: false, implemented: false, keywords: authored::card_1842::RULES.keywords, deck_limit: authored::card_1842::RULES.deck_limit }, effect: authored::card_1842::RULES.effect, triggers: authored::card_1842::RULES.triggers, activated: authored::card_1842::RULES.activated, enters: authored::card_1842::RULES.enters, entry: authored::card_1842::RULES.entry, kind: 0, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_1842::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Heartfire Hero", oracle_id: None, mana_cost: "", type_line: "", oracle_keywords: &[], standard_legal: false, legendary: false, implemented: false, keywords: authored::card_1977::RULES.keywords, deck_limit: authored::card_1977::RULES.deck_limit }, effect: authored::card_1977::RULES.effect, triggers: authored::card_1977::RULES.triggers, activated: authored::card_1977::RULES.activated, enters: authored::card_1977::RULES.enters, entry: authored::card_1977::RULES.entry, kind: 0, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_1977::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Hopeless Nightmare", oracle_id: None, mana_cost: "", type_line: "", oracle_keywords: &[], standard_legal: false, legendary: false, implemented: false, keywords: authored::card_2072::RULES.keywords, deck_limit: authored::card_2072::RULES.deck_limit }, effect: authored::card_2072::RULES.effect, triggers: authored::card_2072::RULES.triggers, activated: authored::card_2072::RULES.activated, enters: authored::card_2072::RULES.enters, entry: authored::card_2072::RULES.entry, kind: 0, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_2072::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Monstrous Rage", oracle_id: None, mana_cost: "", type_line: "", oracle_keywords: &[], standard_legal: false, legendary: false, implemented: false, keywords: authored::card_2837::RULES.keywords, deck_limit: authored::card_2837::RULES.deck_limit }, effect: authored::card_2837::RULES.effect, triggers: authored::card_2837::RULES.triggers, activated: authored::card_2837::RULES.activated, enters: authored::card_2837::RULES.enters, entry: authored::card_2837::RULES.entry, kind: 0, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_2837::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Proft's Eidetic Memory", oracle_id: None, mana_cost: "", type_line: "", oracle_keywords: &[], standard_legal: false, legendary: false, implemented: false, keywords: authored::card_3251::RULES.keywords, deck_limit: authored::card_3251::RULES.deck_limit }, effect: authored::card_3251::RULES.effect, triggers: authored::card_3251::RULES.triggers, activated: authored::card_3251::RULES.activated, enters: authored::card_3251::RULES.enters, entry: authored::card_3251::RULES.entry, kind: 0, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_3251::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Screaming Nemesis", oracle_id: None, mana_cost: "", type_line: "", oracle_keywords: &[], standard_legal: false, legendary: false, implemented: false, keywords: authored::card_3739::RULES.keywords, deck_limit: authored::card_3739::RULES.deck_limit }, effect: authored::card_3739::RULES.effect, triggers: authored::card_3739::RULES.triggers, activated: authored::card_3739::RULES.activated, enters: authored::card_3739::RULES.enters, entry: authored::card_3739::RULES.entry, kind: 0, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_3739::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Stormchaser's Talent", oracle_id: None, mana_cost: "", type_line: "", oracle_keywords: &[], standard_legal: false, legendary: false, implemented: false, keywords: authored::card_4173::RULES.keywords, deck_limit: authored::card_4173::RULES.deck_limit }, effect: authored::card_4173::RULES.effect, triggers: authored::card_4173::RULES.triggers, activated: authored::card_4173::RULES.activated, enters: authored::card_4173::RULES.enters, entry: authored::card_4173::RULES.entry, kind: 0, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_4173::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "This Town Ain't Big Enough", oracle_id: None, mana_cost: "", type_line: "", oracle_keywords: &[], standard_legal: false, legendary: false, implemented: false, keywords: authored::card_4534::RULES.keywords, deck_limit: authored::card_4534::RULES.deck_limit }, effect: authored::card_4534::RULES.effect, triggers: authored::card_4534::RULES.triggers, activated: authored::card_4534::RULES.activated, enters: authored::card_4534::RULES.enters, entry: authored::card_4534::RULES.entry, kind: 0, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_4534::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Up the Beanstalk", oracle_id: None, mana_cost: "", type_line: "", oracle_keywords: &[], standard_legal: false, legendary: false, implemented: false, keywords: authored::card_4812::RULES.keywords, deck_limit: authored::card_4812::RULES.deck_limit }, effect: authored::card_4812::RULES.effect, triggers: authored::card_4812::RULES.triggers, activated: authored::card_4812::RULES.activated, enters: authored::card_4812::RULES.enters, entry: authored::card_4812::RULES.entry, kind: 0, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_4812::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Vivi Ornitier", oracle_id: None, mana_cost: "", type_line: "", oracle_keywords: &[], standard_legal: false, legendary: false, implemented: false, keywords: authored::card_4923::RULES.keywords, deck_limit: authored::card_4923::RULES.deck_limit }, effect: authored::card_4923::RULES.effect, triggers: authored::card_4923::RULES.triggers, activated: authored::card_4923::RULES.activated, enters: authored::card_4923::RULES.enters, entry: authored::card_4923::RULES.entry, kind: 0, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_4923::RULES },
];
}
pub use cards::ENTRIES;
