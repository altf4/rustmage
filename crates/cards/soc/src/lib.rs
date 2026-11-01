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
pub(super) mod card_0034 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/soc/abrade.rs"));
}
pub(super) mod card_0164 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/soc/angelic_destiny.rs"));
}
pub(super) mod card_0246 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/soc/assassin_s_trophy.rs"));
}
pub(super) mod card_0456 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/soc/bloodghast.rs"));
}
pub(super) mod card_0758 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/soc/claim_jumper.rs"));
}
pub(super) mod card_0994 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/soc/deadly_brew.rs"));
}
pub(super) mod card_1294 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/soc/elusive_otter_grove_s_bounty.rs"));
}
pub(super) mod card_1355 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/soc/eriette_of_the_charmed_apple.rs"));
}
pub(super) mod card_1823 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/soc/goldvein_hydra.rs"));
}
pub(super) mod card_2167 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/soc/ingenious_prodigy.rs"));
}
pub(super) mod card_2847 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/soc/moonshaker_cavalry.rs"));
}
pub(super) mod card_3545 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/soc/rite_of_replication.rs"));
}
pub(super) mod card_3825 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/soc/sheltered_by_ghosts.rs"));
}
pub(super) mod card_4172 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/soc/stormcatch_mentor.rs"));
}
pub(super) mod card_4365 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/soc/temple_of_epiphany.rs"));
}
pub(super) mod card_4366 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/soc/temple_of_malady.rs"));
}
pub(super) mod card_4368 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/soc/temple_of_mystery.rs"));
}
pub(super) mod card_4370 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/soc/temple_of_silence.rs"));
}
pub(super) mod card_4371 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/soc/temple_of_triumph.rs"));
}
pub(super) mod card_4451 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/soc/the_goose_mother.rs"));
}
pub(super) mod card_4629 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/soc/tomik_wielder_of_law.rs"));
}
pub(super) mod card_4690 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/soc/troyan_gutsy_explorer.rs"));
}
pub(super) mod card_4750 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/soc/umbral_collar_zealot.rs"));
}
pub(super) mod card_5164 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/soc/zimone_all_questioning.rs"));
}
}
#[rustfmt::skip]
pub static ENTRIES: [CatalogueEntry; 24] = [
CatalogueEntry { definition: Definition { info: CardInfo { name: "Abrade", oracle_id: Some("f9db72dc-9a5b-48a4-a86e-7464d9a2166a"), mana_cost: "{1}{R}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0034::RULES.keywords, deck_limit: authored::card_0034::RULES.deck_limit }, effect: authored::card_0034::RULES.checked_effect(), triggers: authored::card_0034::RULES.triggers, activated: authored::card_0034::RULES.activated, enters: authored::card_0034::RULES.enters, entry: authored::card_0034::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 1 }, rules: &authored::card_0034::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Angelic Destiny", oracle_id: Some("8ba36878-0816-44c7-b543-8ebfc28c2ecd"), mana_cost: "{2}{W}{W}", type_line: "Enchantment — Aura", oracle_keywords: &["Enchant"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0164::RULES.keywords, deck_limit: authored::card_0164::RULES.deck_limit }, effect: authored::card_0164::RULES.checked_effect(), triggers: authored::card_0164::RULES.triggers, activated: authored::card_0164::RULES.activated, enters: authored::card_0164::RULES.enters, entry: authored::card_0164::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [2, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_0164::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Assassin's Trophy", oracle_id: Some("ac10d218-f9a6-4058-9cda-a15ca1b0b7b5"), mana_cost: "{B}{G}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_0246::RULES.keywords, deck_limit: authored::card_0246::RULES.deck_limit }, effect: authored::card_0246::RULES.effect, triggers: authored::card_0246::RULES.triggers, activated: authored::card_0246::RULES.activated, enters: authored::card_0246::RULES.enters, entry: authored::card_0246::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 1, 0, 1, 0], generic: 0 }, rules: &authored::card_0246::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Bloodghast", oracle_id: Some("e97f9c2b-b41e-4f36-9245-77c0ac125647"), mana_cost: "{B}{B}", type_line: "Creature — Vampire Spirit", oracle_keywords: &["Landfall"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0456::RULES.keywords, deck_limit: authored::card_0456::RULES.deck_limit }, effect: authored::card_0456::RULES.checked_effect(), triggers: authored::card_0456::RULES.triggers, activated: authored::card_0456::RULES.activated, enters: authored::card_0456::RULES.enters, entry: authored::card_0456::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 2, 0, 0, 0], generic: 0 }, rules: &authored::card_0456::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Claim Jumper", oracle_id: Some("18f0cd0b-3e4f-4637-a62e-75dd1b2f3fce"), mana_cost: "{2}{W}", type_line: "Creature — Rabbit Mercenary", oracle_keywords: &["Vigilance"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_0758::RULES.keywords, deck_limit: authored::card_0758::RULES.deck_limit }, effect: authored::card_0758::RULES.effect, triggers: authored::card_0758::RULES.triggers, activated: authored::card_0758::RULES.activated, enters: authored::card_0758::RULES.enters, entry: authored::card_0758::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_0758::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Deadly Brew", oracle_id: Some("b831d3f8-07ce-4172-ba29-213cac414c9a"), mana_cost: "{B}{G}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0994::RULES.keywords, deck_limit: authored::card_0994::RULES.deck_limit }, effect: authored::card_0994::RULES.checked_effect(), triggers: authored::card_0994::RULES.triggers, activated: authored::card_0994::RULES.activated, enters: authored::card_0994::RULES.enters, entry: authored::card_0994::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 1, 0, 1, 0], generic: 0 }, rules: &authored::card_0994::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Elusive Otter", oracle_id: Some("54175132-2c44-4749-8dfd-d08dcc63e4b3"), mana_cost: "{U}", type_line: "Creature — Otter", oracle_keywords: &["Prowess"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1294::RULES.keywords, deck_limit: authored::card_1294::RULES.deck_limit }, effect: authored::card_1294::RULES.checked_effect(), triggers: authored::card_1294::RULES.triggers, activated: authored::card_1294::RULES.activated, enters: authored::card_1294::RULES.enters, entry: authored::card_1294::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_1294::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Eriette of the Charmed Apple", oracle_id: Some("f43f5855-f297-4433-984b-18263b734cfc"), mana_cost: "{1}{W}{B}", type_line: "Legendary Creature — Human Warlock", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: false, keywords: authored::card_1355::RULES.keywords, deck_limit: authored::card_1355::RULES.deck_limit }, effect: authored::card_1355::RULES.effect, triggers: authored::card_1355::RULES.triggers, activated: authored::card_1355::RULES.activated, enters: authored::card_1355::RULES.enters, entry: authored::card_1355::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 1, 0, 0, 0], generic: 1 }, rules: &authored::card_1355::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Goldvein Hydra", oracle_id: Some("2b62543f-a475-457a-a96b-b5d070383d3c"), mana_cost: "{X}{G}", type_line: "Creature — Hydra", oracle_keywords: &["Vigilance", "Treasure", "Haste", "Trample"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_1823::RULES.keywords, deck_limit: authored::card_1823::RULES.deck_limit }, effect: authored::card_1823::RULES.effect, triggers: authored::card_1823::RULES.triggers, activated: authored::card_1823::RULES.activated, enters: authored::card_1823::RULES.enters, entry: authored::card_1823::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_1823::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Ingenious Prodigy", oracle_id: Some("6f498ff7-38c9-44ea-9cb5-994a003825ee"), mana_cost: "{X}{U}", type_line: "Creature — Human Wizard", oracle_keywords: &["Skulk"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_2167::RULES.keywords, deck_limit: authored::card_2167::RULES.deck_limit }, effect: authored::card_2167::RULES.effect, triggers: authored::card_2167::RULES.triggers, activated: authored::card_2167::RULES.activated, enters: authored::card_2167::RULES.enters, entry: authored::card_2167::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_2167::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Moonshaker Cavalry", oracle_id: Some("b3f16f34-d78c-4b61-821b-05fc6bd0da37"), mana_cost: "{5}{W}{W}{W}", type_line: "Creature — Spirit Knight", oracle_keywords: &["Flying"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_2847::RULES.keywords, deck_limit: authored::card_2847::RULES.deck_limit }, effect: authored::card_2847::RULES.effect, triggers: authored::card_2847::RULES.triggers, activated: authored::card_2847::RULES.activated, enters: authored::card_2847::RULES.enters, entry: authored::card_2847::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [3, 0, 0, 0, 0, 0], generic: 5 }, rules: &authored::card_2847::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Rite of Replication", oracle_id: Some("fb60739e-1dc3-481d-a056-ad72e665c680"), mana_cost: "{2}{U}{U}", type_line: "Sorcery", oracle_keywords: &["Kicker"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3545::RULES.keywords, deck_limit: authored::card_3545::RULES.deck_limit }, effect: authored::card_3545::RULES.checked_effect(), triggers: authored::card_3545::RULES.triggers, activated: authored::card_3545::RULES.activated, enters: authored::card_3545::RULES.enters, entry: authored::card_3545::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 2, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_3545::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Sheltered by Ghosts", oracle_id: Some("d13fc657-c6fc-4394-bc70-691050550226"), mana_cost: "{1}{W}", type_line: "Enchantment — Aura", oracle_keywords: &["Enchant"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3825::RULES.keywords, deck_limit: authored::card_3825::RULES.deck_limit }, effect: authored::card_3825::RULES.checked_effect(), triggers: authored::card_3825::RULES.triggers, activated: authored::card_3825::RULES.activated, enters: authored::card_3825::RULES.enters, entry: authored::card_3825::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_3825::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Stormcatch Mentor", oracle_id: Some("b2252471-01ca-4d58-aa99-4ab0aa5eae12"), mana_cost: "{U}{R}", type_line: "Creature — Otter Wizard", oracle_keywords: &["Prowess", "Haste"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4172::RULES.keywords, deck_limit: authored::card_4172::RULES.deck_limit }, effect: authored::card_4172::RULES.checked_effect(), triggers: authored::card_4172::RULES.triggers, activated: authored::card_4172::RULES.activated, enters: authored::card_4172::RULES.enters, entry: authored::card_4172::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 1, 0, 0], generic: 0 }, rules: &authored::card_4172::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Temple of Epiphany", oracle_id: Some("79f94050-d850-41ca-b1db-5ae0cf743f0a"), mana_cost: "", type_line: "Land", oracle_keywords: &["Scry"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4365::RULES.keywords, deck_limit: authored::card_4365::RULES.deck_limit }, effect: authored::card_4365::RULES.checked_effect(), triggers: authored::card_4365::RULES.triggers, activated: authored::card_4365::RULES.activated, enters: authored::card_4365::RULES.enters, entry: authored::card_4365::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_4365::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Temple of Malady", oracle_id: Some("dc55421f-dee8-4263-9df0-2365df5f14bb"), mana_cost: "", type_line: "Land", oracle_keywords: &["Scry"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4366::RULES.keywords, deck_limit: authored::card_4366::RULES.deck_limit }, effect: authored::card_4366::RULES.checked_effect(), triggers: authored::card_4366::RULES.triggers, activated: authored::card_4366::RULES.activated, enters: authored::card_4366::RULES.enters, entry: authored::card_4366::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_4366::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Temple of Mystery", oracle_id: Some("7e26f0b7-20e6-46d5-8130-d98c14d6aa29"), mana_cost: "", type_line: "Land", oracle_keywords: &["Scry"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4368::RULES.keywords, deck_limit: authored::card_4368::RULES.deck_limit }, effect: authored::card_4368::RULES.checked_effect(), triggers: authored::card_4368::RULES.triggers, activated: authored::card_4368::RULES.activated, enters: authored::card_4368::RULES.enters, entry: authored::card_4368::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_4368::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Temple of Silence", oracle_id: Some("e6e6fce8-0f6a-4b84-865e-d4e4a4182f9f"), mana_cost: "", type_line: "Land", oracle_keywords: &["Scry"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4370::RULES.keywords, deck_limit: authored::card_4370::RULES.deck_limit }, effect: authored::card_4370::RULES.checked_effect(), triggers: authored::card_4370::RULES.triggers, activated: authored::card_4370::RULES.activated, enters: authored::card_4370::RULES.enters, entry: authored::card_4370::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_4370::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Temple of Triumph", oracle_id: Some("6f0d94d9-64bb-4175-83bc-301e8f79f54f"), mana_cost: "", type_line: "Land", oracle_keywords: &["Scry"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4371::RULES.keywords, deck_limit: authored::card_4371::RULES.deck_limit }, effect: authored::card_4371::RULES.checked_effect(), triggers: authored::card_4371::RULES.triggers, activated: authored::card_4371::RULES.activated, enters: authored::card_4371::RULES.enters, entry: authored::card_4371::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_4371::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "The Goose Mother", oracle_id: Some("de595f1b-3f7d-45e0-a31b-ed23e5d1ee48"), mana_cost: "{X}{G}{U}", type_line: "Legendary Creature — Bird Hydra", oracle_keywords: &["Flying", "Food"], standard_legal: true, legendary: true, implemented: false, keywords: authored::card_4451::RULES.keywords, deck_limit: authored::card_4451::RULES.deck_limit }, effect: authored::card_4451::RULES.effect, triggers: authored::card_4451::RULES.triggers, activated: authored::card_4451::RULES.activated, enters: authored::card_4451::RULES.enters, entry: authored::card_4451::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_4451::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Tomik, Wielder of Law", oracle_id: Some("2ef0961a-0543-4371-8c2a-333881b735e5"), mana_cost: "{1}{W}{B}", type_line: "Legendary Creature — Human Advisor", oracle_keywords: &["Flying", "Affinity", "Vigilance"], standard_legal: true, legendary: true, implemented: false, keywords: authored::card_4629::RULES.keywords, deck_limit: authored::card_4629::RULES.deck_limit }, effect: authored::card_4629::RULES.effect, triggers: authored::card_4629::RULES.triggers, activated: authored::card_4629::RULES.activated, enters: authored::card_4629::RULES.enters, entry: authored::card_4629::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 1, 0, 0, 0], generic: 1 }, rules: &authored::card_4629::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Troyan, Gutsy Explorer", oracle_id: Some("1135ebc3-9537-4341-956b-6e22d8be5f20"), mana_cost: "{1}{G}{U}", type_line: "Legendary Creature — Vedalken Scout", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: false, keywords: authored::card_4690::RULES.keywords, deck_limit: authored::card_4690::RULES.deck_limit }, effect: authored::card_4690::RULES.effect, triggers: authored::card_4690::RULES.triggers, activated: authored::card_4690::RULES.activated, enters: authored::card_4690::RULES.enters, entry: authored::card_4690::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 1, 0], generic: 1 }, rules: &authored::card_4690::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Umbral Collar Zealot", oracle_id: Some("12db6263-75c2-442f-a1a5-7af7915f8f9f"), mana_cost: "{1}{B}", type_line: "Creature — Human Cleric", oracle_keywords: &["Surveil"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4750::RULES.keywords, deck_limit: authored::card_4750::RULES.deck_limit }, effect: authored::card_4750::RULES.checked_effect(), triggers: authored::card_4750::RULES.triggers, activated: authored::card_4750::RULES.activated, enters: authored::card_4750::RULES.enters, entry: authored::card_4750::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 1 }, rules: &authored::card_4750::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Zimone, All-Questioning", oracle_id: Some("6c044dd1-8938-4d62-a7b2-e2298de08f70"), mana_cost: "{1}{G}{U}", type_line: "Legendary Creature — Human Wizard", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: false, keywords: authored::card_5164::RULES.keywords, deck_limit: authored::card_5164::RULES.deck_limit }, effect: authored::card_5164::RULES.effect, triggers: authored::card_5164::RULES.triggers, activated: authored::card_5164::RULES.activated, enters: authored::card_5164::RULES.enters, entry: authored::card_5164::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 1, 0], generic: 1 }, rules: &authored::card_5164::RULES },
];
}
pub use cards::ENTRIES;
