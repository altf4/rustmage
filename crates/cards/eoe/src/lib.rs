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
pub(super) mod card_0053 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/adagia_windswept_bastion.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0131 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/all_fates_scroll.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0132 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/all_fates_stalker.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0138 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/alpharael_dreaming_acolyte.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0139 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/alpharael_stonechosen.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0171 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/annul.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0179 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/anticausal_vestige.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0210 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/archenemy_s_charm.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0250 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/astelli_reclaimer.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0255 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/atmospheric_greenhouse.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0256 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/atomic_microsizer.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0274 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/auxiliary_boosters.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0355 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/beamsaw_prospector.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0402 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/beyond_the_quiet.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0415 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/bioengineered_future.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0417 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/biomechan_engineer.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0419 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/biosynthic_burst.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0420 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/biotech_specialist.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0437 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/blade_of_the_swarm.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0467 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/blooming_stinger.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0491 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/bombard.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0543 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/breeding_pool.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0550 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/brightspear_zealot.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0567 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/broodguard_elite.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0606 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/bygone_colossus.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0702 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/cerebral_download.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0745 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/chorale_of_the_void.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0747 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/chrome_companion.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0775 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/close_encounter.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0781 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/cloudsculpt_technician.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0787 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/codecracker_hound.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0809 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/comet_crawler.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0811 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/command_bridge.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0836 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/consult_the_star_charts.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0863 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/cosmogoyf.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0864 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/cosmogrand_zenith.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0909 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/cryogen_relic.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0910 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/cryoshatter.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0941 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/cut_propulsion.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0965 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/dark_endurance.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0976 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/dauntless_scrapbot.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0983 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/dawnsire_sunstar_dreadnought.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0984 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/dawnstrike_vanguard.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1010 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/debris_field_crusher.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1013 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/decode_transmissions.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1046 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/depressurize.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1049 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/desculpting_blast.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1063 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/devastating_onslaught.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1084 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/diplomatic_relations.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1108 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/divert_disaster.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1118 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/dockworker_drone.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1187 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/drill_too_deep.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1189 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/drix_fatemaker.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1196 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/dual_sun_adepts.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1197 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/dual_sun_technique.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1198 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/dubious_delicacy.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1220 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/dyadrine_synthesis_amalgam.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1265 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/edge_rover.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1280 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/elegy_acolyte.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1308 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/embrace_oblivion.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1311 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/emergency_eject.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1321 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/emissary_escort.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1348 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/entropic_battlecruiser.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1383 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/eumidian_terrabotanist.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1384 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/eusocial_engineering.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1385 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/evendo_waking_haven.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1392 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/exalted_sunborn.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1400 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/exosuit_savior.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1413 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/extinguisher_battleship.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1434 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/faller_s_faithful.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1436 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/famished_worldsire.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1489 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/fell_gravship.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1559 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/flight_deck_coordinator.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1572 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/focus_fire.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1620 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/frenzied_baloth.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1636 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/frontline_war_rager.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1644 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/full_bore.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1650 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/fungal_colossus.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1663 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/galactic_wayfarer.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1675 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/galvanizing_sawship.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1704 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/gene_pollinator.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1705 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/genemorph_imago.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1716 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/germinating_wurm.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1748 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/gigastorm_titan.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1761 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/glacier_godmaw.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1815 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/godless_shrine.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1852 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/gravblade_heavy.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1857 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/gravkill.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1858 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/gravpack_monoist.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1928 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/haliya_ascendant_cadet.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1929 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/haliya_guided_by_light.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1946 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/hardlight_containment.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1949 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/harmonious_grovestrider.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2004 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/hemosymbic_mite.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2065 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/honor.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2068 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/honored_knight_captain.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2095 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/hullcarver.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2116 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/hylderblade.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2117 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/hymn_of_the_faller.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2123 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/icecave_crasher.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2125 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/icetill_explorer.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2138 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/illvoi_galeblade.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2139 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/illvoi_infiltrator.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2140 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/illvoi_light_jammer.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2141 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/illvoi_operative.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2163 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/infinite_guideline_station.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2178 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/insatiable_skittermaw.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2193 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/interceptor_mechan.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2206 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/intrepid_tenderfoot.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2213 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/invasive_maneuvers.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2326 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/kav_landseeker.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2327 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/kavaron_harrier.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2328 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/kavaron_skywarden.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2329 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/kavaron_turbodrone.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2330 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/kavaron_memorial_world.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2383 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/knight_luminary.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2446 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/larval_scoutlander.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2447 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/lashwhip_predator.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2495 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/lightless_evangel.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2502 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/lightstall_inquisitor.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2517 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/lithobraking.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2531 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/loading_zone.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2564 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/lost_in_space.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2578 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/lumen_class_frigate.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2589 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/luxknight_breacher.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2696 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/mechan_assembler.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2697 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/mechan_navigator.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2698 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/mechan_shieldmate.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2701 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/mechanozoa.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2707 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/melded_moxite.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2709 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/meltstrider_eulogist.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2710 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/meltstrider_s_gear.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2711 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/meltstrider_s_resolve.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2712 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/memorial_team_leader.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2713 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/memorial_vault.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2719 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/mental_modulation.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2752 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/mightform_harmonizer.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2802 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/mm_menon_uthros_exile.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2803 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/mm_menon_the_right_hand.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2812 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/molecular_modifier.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2834 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/monoist_circuit_feeder.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2835 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/monoist_sentry.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2844 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/moonlit_meditation.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2867 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/mouth_of_the_storm.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2892 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/mutinous_massacre.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2906 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/nanoform_sentinel.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2912 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/nebula_dragon.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2970 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/nova_hellkite.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2980 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/nutrient_block.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3032 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/orbital_plunge.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3036 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/oreplate_pangolin.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3055 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/ouroboroid.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3090 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/pain_for_all.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3129 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/perigee_beckoner.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3166 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/pinnacle_emissary.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3167 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/pinnacle_kill_ship.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3168 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/pinnacle_starcage.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3182 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/plasma_bolt.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3208 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/possibility_technician.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3275 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/pull_through_the_weft.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3277 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/pulsar_squadron_ace.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3306 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/quantum_riddler.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3334 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/radiant_strike.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3345 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/ragost_deft_gastronaut.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3404 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/rayblade_trooper.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3431 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/red_tiger_mechan.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3456 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/remnant_elemental.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3470 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/requiem_monolith.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3473 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/reroute_systems.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3476 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/rescue_skiff.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3523 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/rig_for_war.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3598 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/roving_actuator.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3613 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/ruinous_rampage.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3628 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/rust_harvester.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3644 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/sacred_foundry.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3662 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/sami_s_curiosity.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3663 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/sami_ship_s_engineer.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3664 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/sami_wildcat_captain.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3730 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/scour_for_scrap.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3732 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/scout_for_survivors.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3742 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/scrounge_for_eternity.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3746 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/seam_rip.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3758 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/secluded_starforge.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3765 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/seedship_agrarian.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3766 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/seedship_broodtender.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3767 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/seedship_impact.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3781 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/selfcraft_mechan.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3821 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/shattered_wings.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3880 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/singularity_rupture.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3881 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/sinister_cryologist.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3908 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/skystinger.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3911 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/slagdrill_scrapper.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3916 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/sledge_class_seedship.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3971 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/sothera_the_supervoid.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3990 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/space_time_anomaly.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3992 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/specimen_freighter.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4080 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/squire_s_lightblade.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4096 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/starbreach_whale.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4098 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/starfield_shepherd.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4099 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/starfield_vocalist.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4100 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/starfighter_pilot.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4108 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/starport_security.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4116 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/starwinder.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4119 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/station_monitor.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4129 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/steelswarm_operator.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4161 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/stomping_ground.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4244 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/sunset_saboteur.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4249 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/sunstar_chaplain.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4250 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/sunstar_expansionist.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4251 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/sunstar_lightsmith.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4276 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/survey_mechan.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4280 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/susur_secundi_void_altar.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4281 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/susurian_dirgecraft.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4282 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/susurian_voidborn.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4286 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/swarm_culler.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4305 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/synthesizer_labship.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4310 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/syr_vondam_sunstar_exemplar.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4311 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/syr_vondam_the_lucent.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4312 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/systems_override.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4334 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/tannuk_memorial_ensign.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4335 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/tannuk_steadfast_second.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4337 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/tapestry_warden.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4373 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/temporal_intervention.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4388 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/terminal_velocity.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4391 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/terrapact_intimidator.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4392 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/terrasymbiosis.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4395 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/territorial_bruntar.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4412 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/tezzeret_cruel_captain.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4415 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/thaumaton_torpedo.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4416 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/thawbringer.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4435 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/the_dominion_bracelet.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4442 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/the_endstone.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4444 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/the_eternity_elevator.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4496 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/the_seriema.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4576 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/thrumming_hivepool.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4602 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/timeline_culler.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4646 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/tractor_beam.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4651 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/tragic_trajectory.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4792 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/unravel.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4817 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/uthros_psionicist.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4818 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/uthros_scanship.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4819 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/uthros_titanic_godcore.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4854 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/vaultguard_trooper.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4911 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/virulent_silencer.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4912 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/virus_beetle.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4929 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/voidforged_titan.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4939 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/vote_out.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4981 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/warmaker_gunship.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_5004 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/watery_grave.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_5020 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/weapons_manufacturing.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_5029 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/wedgelight_rammer.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_5030 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/weftblade_enhancer.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_5031 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/weftstalker_ardent.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_5032 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/weftwalking.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_5126 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/wurmwall_sweeper.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_5129 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/xu_ifit_osteoharmonist.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_5154 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/zealous_display.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_5158 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/zero_point_ballad.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_5171 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/eoe/zookeeper_mechan.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
}
#[rustfmt::skip]
pub static ENTRIES: [CatalogueEntry; 259] = [
CatalogueEntry { definition: Definition { info: CardInfo { name: "Adagia, Windswept Bastion", oracle_id: Some("70d35dbd-1d91-4a2a-a643-6870d168f4f5"), mana_cost: "", type_line: "Land — Planet", oracle_keywords: &["Station"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_0053::RULES.keywords, deck_limit: authored::card_0053::RULES.deck_limit }, effect: authored::card_0053::RULES.effect, triggers: authored::card_0053::RULES.triggers, activated: authored::card_0053::RULES.activated, enters: authored::card_0053::RULES.enters, entry: authored::card_0053::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_0053::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "All-Fates Scroll", oracle_id: Some("cef5361a-a189-4e05-a8b8-fc764aed73b9"), mana_cost: "{3}", type_line: "Artifact", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0131::RULES.keywords, deck_limit: authored::card_0131::RULES.deck_limit }, effect: authored::card_0131::RULES.checked_effect(), triggers: authored::card_0131::RULES.triggers, activated: authored::card_0131::RULES.activated, enters: authored::card_0131::RULES.enters, entry: authored::card_0131::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_0131::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "All-Fates Stalker", oracle_id: Some("da9ec010-8b42-4927-b602-4e96bced4e99"), mana_cost: "{3}{W}", type_line: "Creature — Drix Assassin", oracle_keywords: &["Warp"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_0132::RULES.keywords, deck_limit: authored::card_0132::RULES.deck_limit }, effect: authored::card_0132::RULES.effect, triggers: authored::card_0132::RULES.triggers, activated: authored::card_0132::RULES.activated, enters: authored::card_0132::RULES.enters, entry: authored::card_0132::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_0132::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Alpharael, Dreaming Acolyte", oracle_id: Some("e4be4071-7910-460f-90f3-6c4d8cc9db65"), mana_cost: "{1}{U}{B}", type_line: "Legendary Creature — Human Cleric", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: false, keywords: authored::card_0138::RULES.keywords, deck_limit: authored::card_0138::RULES.deck_limit }, effect: authored::card_0138::RULES.effect, triggers: authored::card_0138::RULES.triggers, activated: authored::card_0138::RULES.activated, enters: authored::card_0138::RULES.enters, entry: authored::card_0138::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 1, 0, 0, 0], generic: 1 }, rules: &authored::card_0138::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Alpharael, Stonechosen", oracle_id: Some("ed43f1f9-ef96-47bc-8a54-e59154319f9d"), mana_cost: "{3}{B}{B}", type_line: "Legendary Creature — Human Cleric", oracle_keywords: &["Ward", "Void"], standard_legal: true, legendary: true, implemented: false, keywords: authored::card_0139::RULES.keywords, deck_limit: authored::card_0139::RULES.deck_limit }, effect: authored::card_0139::RULES.effect, triggers: authored::card_0139::RULES.triggers, activated: authored::card_0139::RULES.activated, enters: authored::card_0139::RULES.enters, entry: authored::card_0139::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 2, 0, 0, 0], generic: 3 }, rules: &authored::card_0139::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Annul", oracle_id: Some("d08e9784-75f7-4164-ac48-d06160f8c56b"), mana_cost: "{U}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0171::RULES.keywords, deck_limit: authored::card_0171::RULES.deck_limit }, effect: authored::card_0171::RULES.checked_effect(), triggers: authored::card_0171::RULES.triggers, activated: authored::card_0171::RULES.activated, enters: authored::card_0171::RULES.enters, entry: authored::card_0171::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_0171::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Anticausal Vestige", oracle_id: Some("aceea999-90ab-472c-86a7-48af1542cbcf"), mana_cost: "{6}", type_line: "Creature — Eldrazi", oracle_keywords: &["Warp"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_0179::RULES.keywords, deck_limit: authored::card_0179::RULES.deck_limit }, effect: authored::card_0179::RULES.effect, triggers: authored::card_0179::RULES.triggers, activated: authored::card_0179::RULES.activated, enters: authored::card_0179::RULES.enters, entry: authored::card_0179::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 6 }, rules: &authored::card_0179::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Archenemy's Charm", oracle_id: Some("edcb0248-e7c9-4df9-8dba-982048654fd2"), mana_cost: "{B}{B}{B}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0210::RULES.keywords, deck_limit: authored::card_0210::RULES.deck_limit }, effect: authored::card_0210::RULES.checked_effect(), triggers: authored::card_0210::RULES.triggers, activated: authored::card_0210::RULES.activated, enters: authored::card_0210::RULES.enters, entry: authored::card_0210::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 3, 0, 0, 0], generic: 0 }, rules: &authored::card_0210::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Astelli Reclaimer", oracle_id: Some("bb956492-e3ed-483a-9169-5b795a8e619e"), mana_cost: "{3}{W}{W}", type_line: "Creature — Angel Warrior", oracle_keywords: &["Flying", "Warp"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_0250::RULES.keywords, deck_limit: authored::card_0250::RULES.deck_limit }, effect: authored::card_0250::RULES.effect, triggers: authored::card_0250::RULES.triggers, activated: authored::card_0250::RULES.activated, enters: authored::card_0250::RULES.enters, entry: authored::card_0250::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [2, 0, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_0250::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Atmospheric Greenhouse", oracle_id: Some("52c6afc6-aff1-46a8-b1ea-b183facec1ae"), mana_cost: "{4}{G}", type_line: "Artifact — Spacecraft", oracle_keywords: &["Station"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_0255::RULES.keywords, deck_limit: authored::card_0255::RULES.deck_limit }, effect: authored::card_0255::RULES.effect, triggers: authored::card_0255::RULES.triggers, activated: authored::card_0255::RULES.activated, enters: authored::card_0255::RULES.enters, entry: authored::card_0255::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 4 }, rules: &authored::card_0255::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Atomic Microsizer", oracle_id: Some("048c3b0b-48ef-42f5-a2af-90aa1d133296"), mana_cost: "{U}", type_line: "Artifact — Equipment", oracle_keywords: &["Equip"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0256::RULES.keywords, deck_limit: authored::card_0256::RULES.deck_limit }, effect: authored::card_0256::RULES.checked_effect(), triggers: authored::card_0256::RULES.triggers, activated: authored::card_0256::RULES.activated, enters: authored::card_0256::RULES.enters, entry: authored::card_0256::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_0256::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Auxiliary Boosters", oracle_id: Some("ffa3b20b-d77b-43d5-86ac-6a591df1877f"), mana_cost: "{4}{W}", type_line: "Artifact — Equipment", oracle_keywords: &["Equip"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_0274::RULES.keywords, deck_limit: authored::card_0274::RULES.deck_limit }, effect: authored::card_0274::RULES.effect, triggers: authored::card_0274::RULES.triggers, activated: authored::card_0274::RULES.activated, enters: authored::card_0274::RULES.enters, entry: authored::card_0274::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 4 }, rules: &authored::card_0274::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Beamsaw Prospector", oracle_id: Some("bbe740f8-6e7c-4cfd-9134-9bf8d5f48cf5"), mana_cost: "{1}{B}", type_line: "Creature — Human Artificer", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0355::RULES.keywords, deck_limit: authored::card_0355::RULES.deck_limit }, effect: authored::card_0355::RULES.checked_effect(), triggers: authored::card_0355::RULES.triggers, activated: authored::card_0355::RULES.activated, enters: authored::card_0355::RULES.enters, entry: authored::card_0355::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 1 }, rules: &authored::card_0355::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Beyond the Quiet", oracle_id: Some("c96424e7-63c4-4f07-959f-6ad09f91279a"), mana_cost: "{3}{W}{W}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0402::RULES.keywords, deck_limit: authored::card_0402::RULES.deck_limit }, effect: authored::card_0402::RULES.checked_effect(), triggers: authored::card_0402::RULES.triggers, activated: authored::card_0402::RULES.activated, enters: authored::card_0402::RULES.enters, entry: authored::card_0402::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [2, 0, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_0402::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Bioengineered Future", oracle_id: Some("6fa78914-85b3-46fd-977a-0e5b4d29eca4"), mana_cost: "{1}{G}{G}", type_line: "Enchantment", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_0415::RULES.keywords, deck_limit: authored::card_0415::RULES.deck_limit }, effect: authored::card_0415::RULES.effect, triggers: authored::card_0415::RULES.triggers, activated: authored::card_0415::RULES.activated, enters: authored::card_0415::RULES.enters, entry: authored::card_0415::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 0, 0, 0, 2, 0], generic: 1 }, rules: &authored::card_0415::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Biomechan Engineer", oracle_id: Some("b395925b-2456-454e-9262-3b55c9f2799a"), mana_cost: "{G}{U}", type_line: "Creature — Insect Artificer", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0417::RULES.keywords, deck_limit: authored::card_0417::RULES.deck_limit }, effect: authored::card_0417::RULES.checked_effect(), triggers: authored::card_0417::RULES.triggers, activated: authored::card_0417::RULES.activated, enters: authored::card_0417::RULES.enters, entry: authored::card_0417::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 1, 0], generic: 0 }, rules: &authored::card_0417::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Biosynthic Burst", oracle_id: Some("3abd2342-f783-4871-ac78-1a54b35722c2"), mana_cost: "{1}{G}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_0419::RULES.keywords, deck_limit: authored::card_0419::RULES.deck_limit }, effect: authored::card_0419::RULES.effect, triggers: authored::card_0419::RULES.triggers, activated: authored::card_0419::RULES.activated, enters: authored::card_0419::RULES.enters, entry: authored::card_0419::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 1 }, rules: &authored::card_0419::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Biotech Specialist", oracle_id: Some("78de77ef-c325-4342-869f-042921229577"), mana_cost: "{R}{G}", type_line: "Creature — Insect Scientist", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0420::RULES.keywords, deck_limit: authored::card_0420::RULES.deck_limit }, effect: authored::card_0420::RULES.checked_effect(), triggers: authored::card_0420::RULES.triggers, activated: authored::card_0420::RULES.activated, enters: authored::card_0420::RULES.enters, entry: authored::card_0420::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 1, 0], generic: 0 }, rules: &authored::card_0420::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Blade of the Swarm", oracle_id: Some("33042d12-f08b-4b3e-b106-c424ef868c13"), mana_cost: "{3}{B}", type_line: "Creature — Insect Assassin", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_0437::RULES.keywords, deck_limit: authored::card_0437::RULES.deck_limit }, effect: authored::card_0437::RULES.effect, triggers: authored::card_0437::RULES.triggers, activated: authored::card_0437::RULES.activated, enters: authored::card_0437::RULES.enters, entry: authored::card_0437::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 3 }, rules: &authored::card_0437::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Blooming Stinger", oracle_id: Some("ab282bb1-900e-40b8-9821-8e2de88927bc"), mana_cost: "{1}{G}", type_line: "Creature — Plant Scorpion", oracle_keywords: &["Deathtouch"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0467::RULES.keywords, deck_limit: authored::card_0467::RULES.deck_limit }, effect: authored::card_0467::RULES.checked_effect(), triggers: authored::card_0467::RULES.triggers, activated: authored::card_0467::RULES.activated, enters: authored::card_0467::RULES.enters, entry: authored::card_0467::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 1 }, rules: &authored::card_0467::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Bombard", oracle_id: Some("c0899c12-5462-4ff1-a9e5-d99cffa7c6cc"), mana_cost: "{2}{R}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0491::RULES.keywords, deck_limit: authored::card_0491::RULES.deck_limit }, effect: authored::card_0491::RULES.checked_effect(), triggers: authored::card_0491::RULES.triggers, activated: authored::card_0491::RULES.activated, enters: authored::card_0491::RULES.enters, entry: authored::card_0491::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 2 }, rules: &authored::card_0491::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Breeding Pool", oracle_id: Some("20283c4a-f1f0-42f0-bc08-6da87474426b"), mana_cost: "", type_line: "Land — Forest Island", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0543::RULES.keywords, deck_limit: authored::card_0543::RULES.deck_limit }, effect: authored::card_0543::RULES.checked_effect(), triggers: authored::card_0543::RULES.triggers, activated: authored::card_0543::RULES.activated, enters: authored::card_0543::RULES.enters, entry: authored::card_0543::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_0543::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Brightspear Zealot", oracle_id: Some("31a0c9a2-c9bd-47f7-872b-a43fb64d873c"), mana_cost: "{2}{W}", type_line: "Creature — Human Soldier", oracle_keywords: &["Vigilance"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0550::RULES.keywords, deck_limit: authored::card_0550::RULES.deck_limit }, effect: authored::card_0550::RULES.checked_effect(), triggers: authored::card_0550::RULES.triggers, activated: authored::card_0550::RULES.activated, enters: authored::card_0550::RULES.enters, entry: authored::card_0550::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_0550::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Broodguard Elite", oracle_id: Some("c90da077-de8c-4402-abfc-b63dd15853da"), mana_cost: "{X}{G}{G}", type_line: "Creature — Insect Knight", oracle_keywords: &["Warp"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_0567::RULES.keywords, deck_limit: authored::card_0567::RULES.deck_limit }, effect: authored::card_0567::RULES.effect, triggers: authored::card_0567::RULES.triggers, activated: authored::card_0567::RULES.activated, enters: authored::card_0567::RULES.enters, entry: authored::card_0567::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_0567::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Bygone Colossus", oracle_id: Some("1bd584d5-4e11-428c-b51e-462e4292b07f"), mana_cost: "{9}", type_line: "Artifact Creature — Robot Giant", oracle_keywords: &["Warp"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0606::RULES.keywords, deck_limit: authored::card_0606::RULES.deck_limit }, effect: authored::card_0606::RULES.checked_effect(), triggers: authored::card_0606::RULES.triggers, activated: authored::card_0606::RULES.activated, enters: authored::card_0606::RULES.enters, entry: authored::card_0606::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 9 }, rules: &authored::card_0606::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Cerebral Download", oracle_id: Some("f27889b9-40d9-4cc7-b8af-8716c53c44f5"), mana_cost: "{4}{U}", type_line: "Instant", oracle_keywords: &["Surveil"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_0702::RULES.keywords, deck_limit: authored::card_0702::RULES.deck_limit }, effect: authored::card_0702::RULES.effect, triggers: authored::card_0702::RULES.triggers, activated: authored::card_0702::RULES.activated, enters: authored::card_0702::RULES.enters, entry: authored::card_0702::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 4 }, rules: &authored::card_0702::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Chorale of the Void", oracle_id: Some("cd50bff0-c461-4023-b62a-6746eaaeab0e"), mana_cost: "{3}{B}", type_line: "Enchantment — Aura", oracle_keywords: &["Enchant", "Void"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_0745::RULES.keywords, deck_limit: authored::card_0745::RULES.deck_limit }, effect: authored::card_0745::RULES.effect, triggers: authored::card_0745::RULES.triggers, activated: authored::card_0745::RULES.activated, enters: authored::card_0745::RULES.enters, entry: authored::card_0745::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 3 }, rules: &authored::card_0745::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Chrome Companion", oracle_id: Some("1ac6fabb-71db-442f-8c14-ba9e659f906c"), mana_cost: "{2}", type_line: "Artifact Creature — Dog", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0747::RULES.keywords, deck_limit: authored::card_0747::RULES.deck_limit }, effect: authored::card_0747::RULES.checked_effect(), triggers: authored::card_0747::RULES.triggers, activated: authored::card_0747::RULES.activated, enters: authored::card_0747::RULES.enters, entry: authored::card_0747::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_0747::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Close Encounter", oracle_id: Some("032fdd08-2ea7-442e-8162-0a8e5a585253"), mana_cost: "{1}{G}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_0775::RULES.keywords, deck_limit: authored::card_0775::RULES.deck_limit }, effect: authored::card_0775::RULES.effect, triggers: authored::card_0775::RULES.triggers, activated: authored::card_0775::RULES.activated, enters: authored::card_0775::RULES.enters, entry: authored::card_0775::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 1 }, rules: &authored::card_0775::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Cloudsculpt Technician", oracle_id: Some("7c5183be-cddf-48fb-a4b5-dc5f6faf26de"), mana_cost: "{2}{U}", type_line: "Creature — Jellyfish Artificer", oracle_keywords: &["Flying"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0781::RULES.keywords, deck_limit: authored::card_0781::RULES.deck_limit }, effect: authored::card_0781::RULES.checked_effect(), triggers: authored::card_0781::RULES.triggers, activated: authored::card_0781::RULES.activated, enters: authored::card_0781::RULES.enters, entry: authored::card_0781::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_0781::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Codecracker Hound", oracle_id: Some("ae791b61-5825-4987-a8c2-80813758d3f7"), mana_cost: "{2}{U}", type_line: "Creature — Dog", oracle_keywords: &["Warp"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_0787::RULES.keywords, deck_limit: authored::card_0787::RULES.deck_limit }, effect: authored::card_0787::RULES.effect, triggers: authored::card_0787::RULES.triggers, activated: authored::card_0787::RULES.activated, enters: authored::card_0787::RULES.enters, entry: authored::card_0787::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_0787::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Comet Crawler", oracle_id: Some("0c1b6305-8863-43e0-b50d-ae1c5e31be55"), mana_cost: "{2}{B}", type_line: "Creature — Insect Horror", oracle_keywords: &["Lifelink"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0809::RULES.keywords, deck_limit: authored::card_0809::RULES.deck_limit }, effect: authored::card_0809::RULES.checked_effect(), triggers: authored::card_0809::RULES.triggers, activated: authored::card_0809::RULES.activated, enters: authored::card_0809::RULES.enters, entry: authored::card_0809::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 2 }, rules: &authored::card_0809::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Command Bridge", oracle_id: Some("87c8e1ed-258a-4a89-bcc6-211405e49692"), mana_cost: "", type_line: "Land", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_0811::RULES.keywords, deck_limit: authored::card_0811::RULES.deck_limit }, effect: authored::card_0811::RULES.effect, triggers: authored::card_0811::RULES.triggers, activated: authored::card_0811::RULES.activated, enters: authored::card_0811::RULES.enters, entry: authored::card_0811::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_0811::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Consult the Star Charts", oracle_id: Some("e921839f-9d91-41a9-bc89-016af3c757aa"), mana_cost: "{1}{U}", type_line: "Instant", oracle_keywords: &["Kicker"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0836::RULES.keywords, deck_limit: authored::card_0836::RULES.deck_limit }, effect: authored::card_0836::RULES.checked_effect(), triggers: authored::card_0836::RULES.triggers, activated: authored::card_0836::RULES.activated, enters: authored::card_0836::RULES.enters, entry: authored::card_0836::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_0836::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Cosmogoyf", oracle_id: Some("99d8c5f9-ad8f-450e-9a7c-51a4c0ca5043"), mana_cost: "{B}{G}", type_line: "Creature — Elemental Lhurgoyf", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0863::RULES.keywords, deck_limit: authored::card_0863::RULES.deck_limit }, effect: authored::card_0863::RULES.checked_effect(), triggers: authored::card_0863::RULES.triggers, activated: authored::card_0863::RULES.activated, enters: authored::card_0863::RULES.enters, entry: authored::card_0863::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 1, 0], generic: 0 }, rules: &authored::card_0863::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Cosmogrand Zenith", oracle_id: Some("e2cd65f6-7cbc-497c-bdc9-698ff17da31e"), mana_cost: "{2}{W}", type_line: "Creature — Human Soldier", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0864::RULES.keywords, deck_limit: authored::card_0864::RULES.deck_limit }, effect: authored::card_0864::RULES.checked_effect(), triggers: authored::card_0864::RULES.triggers, activated: authored::card_0864::RULES.activated, enters: authored::card_0864::RULES.enters, entry: authored::card_0864::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_0864::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Cryogen Relic", oracle_id: Some("106e9c0d-67a2-4db7-97d9-03e1ea1b40b5"), mana_cost: "{1}{U}", type_line: "Artifact", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0909::RULES.keywords, deck_limit: authored::card_0909::RULES.deck_limit }, effect: authored::card_0909::RULES.checked_effect(), triggers: authored::card_0909::RULES.triggers, activated: authored::card_0909::RULES.activated, enters: authored::card_0909::RULES.enters, entry: authored::card_0909::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_0909::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Cryoshatter", oracle_id: Some("865715e9-6759-4e01-803d-702c959e386e"), mana_cost: "{U}", type_line: "Enchantment — Aura", oracle_keywords: &["Enchant"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_0910::RULES.keywords, deck_limit: authored::card_0910::RULES.deck_limit }, effect: authored::card_0910::RULES.effect, triggers: authored::card_0910::RULES.triggers, activated: authored::card_0910::RULES.activated, enters: authored::card_0910::RULES.enters, entry: authored::card_0910::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_0910::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Cut Propulsion", oracle_id: Some("cf3234f2-09d7-4d8b-b871-3bf0fcb75b6d"), mana_cost: "{2}{R}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_0941::RULES.keywords, deck_limit: authored::card_0941::RULES.deck_limit }, effect: authored::card_0941::RULES.effect, triggers: authored::card_0941::RULES.triggers, activated: authored::card_0941::RULES.activated, enters: authored::card_0941::RULES.enters, entry: authored::card_0941::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 2 }, rules: &authored::card_0941::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Dark Endurance", oracle_id: Some("ded4009d-be23-4e36-a7a3-9e07230712ee"), mana_cost: "{1}{B}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0965::RULES.keywords, deck_limit: authored::card_0965::RULES.deck_limit }, effect: authored::card_0965::RULES.checked_effect(), triggers: authored::card_0965::RULES.triggers, activated: authored::card_0965::RULES.activated, enters: authored::card_0965::RULES.enters, entry: authored::card_0965::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 1 }, rules: &authored::card_0965::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Dauntless Scrapbot", oracle_id: Some("b75ca527-4494-4957-b2e3-8c590df42e95"), mana_cost: "{3}", type_line: "Artifact Creature — Robot", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0976::RULES.keywords, deck_limit: authored::card_0976::RULES.deck_limit }, effect: authored::card_0976::RULES.checked_effect(), triggers: authored::card_0976::RULES.triggers, activated: authored::card_0976::RULES.activated, enters: authored::card_0976::RULES.enters, entry: authored::card_0976::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_0976::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Dawnsire, Sunstar Dreadnought", oracle_id: Some("afc9436b-8cad-4916-929d-ff33a37b42d5"), mana_cost: "{5}", type_line: "Legendary Artifact — Spacecraft", oracle_keywords: &["Station"], standard_legal: true, legendary: true, implemented: false, keywords: authored::card_0983::RULES.keywords, deck_limit: authored::card_0983::RULES.deck_limit }, effect: authored::card_0983::RULES.effect, triggers: authored::card_0983::RULES.triggers, activated: authored::card_0983::RULES.activated, enters: authored::card_0983::RULES.enters, entry: authored::card_0983::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 5 }, rules: &authored::card_0983::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Dawnstrike Vanguard", oracle_id: Some("69892dc7-4087-447c-a41c-99cf3d5c0c12"), mana_cost: "{5}{W}", type_line: "Creature — Human Knight", oracle_keywords: &["Lifelink"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_0984::RULES.keywords, deck_limit: authored::card_0984::RULES.deck_limit }, effect: authored::card_0984::RULES.effect, triggers: authored::card_0984::RULES.triggers, activated: authored::card_0984::RULES.activated, enters: authored::card_0984::RULES.enters, entry: authored::card_0984::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 5 }, rules: &authored::card_0984::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Debris Field Crusher", oracle_id: Some("db0894e1-2644-48d4-8de4-8cc43e940bc1"), mana_cost: "{4}{R}", type_line: "Artifact — Spacecraft", oracle_keywords: &["Station"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_1010::RULES.keywords, deck_limit: authored::card_1010::RULES.deck_limit }, effect: authored::card_1010::RULES.effect, triggers: authored::card_1010::RULES.triggers, activated: authored::card_1010::RULES.activated, enters: authored::card_1010::RULES.enters, entry: authored::card_1010::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 4 }, rules: &authored::card_1010::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Decode Transmissions", oracle_id: Some("42510758-6167-420b-bb70-65f914b7a259"), mana_cost: "{2}{B}", type_line: "Sorcery", oracle_keywords: &["Void"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_1013::RULES.keywords, deck_limit: authored::card_1013::RULES.deck_limit }, effect: authored::card_1013::RULES.effect, triggers: authored::card_1013::RULES.triggers, activated: authored::card_1013::RULES.activated, enters: authored::card_1013::RULES.enters, entry: authored::card_1013::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 2 }, rules: &authored::card_1013::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Depressurize", oracle_id: Some("abcb5321-c295-4abf-8622-7de79679d27a"), mana_cost: "{1}{B}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_1046::RULES.keywords, deck_limit: authored::card_1046::RULES.deck_limit }, effect: authored::card_1046::RULES.effect, triggers: authored::card_1046::RULES.triggers, activated: authored::card_1046::RULES.activated, enters: authored::card_1046::RULES.enters, entry: authored::card_1046::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 1 }, rules: &authored::card_1046::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Desculpting Blast", oracle_id: Some("2d80ee9c-55d4-4115-8a58-63e8460f9795"), mana_cost: "{1}{U}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_1049::RULES.keywords, deck_limit: authored::card_1049::RULES.deck_limit }, effect: authored::card_1049::RULES.effect, triggers: authored::card_1049::RULES.triggers, activated: authored::card_1049::RULES.activated, enters: authored::card_1049::RULES.enters, entry: authored::card_1049::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_1049::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Devastating Onslaught", oracle_id: Some("2e17cd4c-ebf0-40d0-b52d-b4d84bbcf7b3"), mana_cost: "{X}{X}{R}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_1063::RULES.keywords, deck_limit: authored::card_1063::RULES.deck_limit }, effect: authored::card_1063::RULES.effect, triggers: authored::card_1063::RULES.triggers, activated: authored::card_1063::RULES.activated, enters: authored::card_1063::RULES.enters, entry: authored::card_1063::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_1063::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Diplomatic Relations", oracle_id: Some("57b75be4-9229-4a48-a59e-a0f812f38637"), mana_cost: "{2}{G}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_1084::RULES.keywords, deck_limit: authored::card_1084::RULES.deck_limit }, effect: authored::card_1084::RULES.effect, triggers: authored::card_1084::RULES.triggers, activated: authored::card_1084::RULES.activated, enters: authored::card_1084::RULES.enters, entry: authored::card_1084::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 2 }, rules: &authored::card_1084::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Divert Disaster", oracle_id: Some("76fdbc2c-d2b1-466f-9230-06e187bcc9e6"), mana_cost: "{1}{U}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1108::RULES.keywords, deck_limit: authored::card_1108::RULES.deck_limit }, effect: authored::card_1108::RULES.checked_effect(), triggers: authored::card_1108::RULES.triggers, activated: authored::card_1108::RULES.activated, enters: authored::card_1108::RULES.enters, entry: authored::card_1108::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_1108::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Dockworker Drone", oracle_id: Some("aa704b26-729a-4d0f-b92d-56aadb1e20b9"), mana_cost: "{1}{W}", type_line: "Artifact Creature — Robot", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_1118::RULES.keywords, deck_limit: authored::card_1118::RULES.deck_limit }, effect: authored::card_1118::RULES.effect, triggers: authored::card_1118::RULES.triggers, activated: authored::card_1118::RULES.activated, enters: authored::card_1118::RULES.enters, entry: authored::card_1118::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_1118::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Drill Too Deep", oracle_id: Some("0413e0e4-c8f8-44c3-a749-c799766bc8cd"), mana_cost: "{1}{R}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_1187::RULES.keywords, deck_limit: authored::card_1187::RULES.deck_limit }, effect: authored::card_1187::RULES.effect, triggers: authored::card_1187::RULES.triggers, activated: authored::card_1187::RULES.activated, enters: authored::card_1187::RULES.enters, entry: authored::card_1187::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 1 }, rules: &authored::card_1187::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Drix Fatemaker", oracle_id: Some("583102f8-7232-4fbb-a715-087448536bb2"), mana_cost: "{3}{G}", type_line: "Creature — Drix Wizard", oracle_keywords: &["Warp"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1189::RULES.keywords, deck_limit: authored::card_1189::RULES.deck_limit }, effect: authored::card_1189::RULES.checked_effect(), triggers: authored::card_1189::RULES.triggers, activated: authored::card_1189::RULES.activated, enters: authored::card_1189::RULES.enters, entry: authored::card_1189::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 3 }, rules: &authored::card_1189::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Dual-Sun Adepts", oracle_id: Some("155e01a1-5809-4be9-ad38-c11553a4a723"), mana_cost: "{2}{W}", type_line: "Creature — Human Soldier", oracle_keywords: &["Double strike"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1196::RULES.keywords, deck_limit: authored::card_1196::RULES.deck_limit }, effect: authored::card_1196::RULES.checked_effect(), triggers: authored::card_1196::RULES.triggers, activated: authored::card_1196::RULES.activated, enters: authored::card_1196::RULES.enters, entry: authored::card_1196::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_1196::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Dual-Sun Technique", oracle_id: Some("89ec0f03-47d0-4657-8987-95e680404f24"), mana_cost: "{1}{W}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_1197::RULES.keywords, deck_limit: authored::card_1197::RULES.deck_limit }, effect: authored::card_1197::RULES.effect, triggers: authored::card_1197::RULES.triggers, activated: authored::card_1197::RULES.activated, enters: authored::card_1197::RULES.enters, entry: authored::card_1197::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_1197::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Dubious Delicacy", oracle_id: Some("697b8045-0edb-4f7b-a933-4d41ec2b53d5"), mana_cost: "{2}{B}", type_line: "Artifact — Food", oracle_keywords: &["Flash"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_1198::RULES.keywords, deck_limit: authored::card_1198::RULES.deck_limit }, effect: authored::card_1198::RULES.effect, triggers: authored::card_1198::RULES.triggers, activated: authored::card_1198::RULES.activated, enters: authored::card_1198::RULES.enters, entry: authored::card_1198::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 2 }, rules: &authored::card_1198::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Dyadrine, Synthesis Amalgam", oracle_id: Some("618a2272-62e0-4d0e-834d-7b81719eba12"), mana_cost: "{X}{G}{W}", type_line: "Legendary Artifact Creature — Construct", oracle_keywords: &["Trample"], standard_legal: true, legendary: true, implemented: false, keywords: authored::card_1220::RULES.keywords, deck_limit: authored::card_1220::RULES.deck_limit }, effect: authored::card_1220::RULES.effect, triggers: authored::card_1220::RULES.triggers, activated: authored::card_1220::RULES.activated, enters: authored::card_1220::RULES.enters, entry: authored::card_1220::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_1220::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Edge Rover", oracle_id: Some("b907d375-b715-446b-8966-79959feb9f58"), mana_cost: "{G}", type_line: "Artifact Creature — Robot Scout", oracle_keywords: &["Reach"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_1265::RULES.keywords, deck_limit: authored::card_1265::RULES.deck_limit }, effect: authored::card_1265::RULES.effect, triggers: authored::card_1265::RULES.triggers, activated: authored::card_1265::RULES.activated, enters: authored::card_1265::RULES.enters, entry: authored::card_1265::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 0 }, rules: &authored::card_1265::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Elegy Acolyte", oracle_id: Some("f7d52eac-edc9-464b-a163-07c136aa6e4c"), mana_cost: "{2}{B}{B}", type_line: "Creature — Human Cleric", oracle_keywords: &["Lifelink", "Void"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_1280::RULES.keywords, deck_limit: authored::card_1280::RULES.deck_limit }, effect: authored::card_1280::RULES.effect, triggers: authored::card_1280::RULES.triggers, activated: authored::card_1280::RULES.activated, enters: authored::card_1280::RULES.enters, entry: authored::card_1280::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 2, 0, 0, 0], generic: 2 }, rules: &authored::card_1280::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Embrace Oblivion", oracle_id: Some("0fad5713-c09c-43d3-9cc0-10c65aead488"), mana_cost: "{B}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1308::RULES.keywords, deck_limit: authored::card_1308::RULES.deck_limit }, effect: authored::card_1308::RULES.checked_effect(), triggers: authored::card_1308::RULES.triggers, activated: authored::card_1308::RULES.activated, enters: authored::card_1308::RULES.enters, entry: authored::card_1308::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 0 }, rules: &authored::card_1308::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Emergency Eject", oracle_id: Some("da363da1-aecf-4d49-8e77-8dc56981a741"), mana_cost: "{2}{W}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_1311::RULES.keywords, deck_limit: authored::card_1311::RULES.deck_limit }, effect: authored::card_1311::RULES.effect, triggers: authored::card_1311::RULES.triggers, activated: authored::card_1311::RULES.activated, enters: authored::card_1311::RULES.enters, entry: authored::card_1311::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_1311::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Emissary Escort", oracle_id: Some("7d8b53f4-bdb5-4ba1-b6c0-f4846d8d3565"), mana_cost: "{1}{U}", type_line: "Artifact Creature — Robot Soldier", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_1321::RULES.keywords, deck_limit: authored::card_1321::RULES.deck_limit }, effect: authored::card_1321::RULES.effect, triggers: authored::card_1321::RULES.triggers, activated: authored::card_1321::RULES.activated, enters: authored::card_1321::RULES.enters, entry: authored::card_1321::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_1321::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Entropic Battlecruiser", oracle_id: Some("90c5ea2b-d004-4219-b60c-d51bc1a5ee6b"), mana_cost: "{3}{B}", type_line: "Artifact — Spacecraft", oracle_keywords: &["Station"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_1348::RULES.keywords, deck_limit: authored::card_1348::RULES.deck_limit }, effect: authored::card_1348::RULES.effect, triggers: authored::card_1348::RULES.triggers, activated: authored::card_1348::RULES.activated, enters: authored::card_1348::RULES.enters, entry: authored::card_1348::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 3 }, rules: &authored::card_1348::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Eumidian Terrabotanist", oracle_id: Some("92226fd2-ad93-4722-89a0-ca88ea03e1b4"), mana_cost: "{1}{G}", type_line: "Creature — Insect Druid", oracle_keywords: &["Landfall"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1383::RULES.keywords, deck_limit: authored::card_1383::RULES.deck_limit }, effect: authored::card_1383::RULES.checked_effect(), triggers: authored::card_1383::RULES.triggers, activated: authored::card_1383::RULES.activated, enters: authored::card_1383::RULES.enters, entry: authored::card_1383::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 1 }, rules: &authored::card_1383::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Eusocial Engineering", oracle_id: Some("2f3f5a1c-8b88-4080-bd80-14db4d6c8c50"), mana_cost: "{3}{G}{G}", type_line: "Enchantment", oracle_keywords: &["Warp", "Landfall"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1384::RULES.keywords, deck_limit: authored::card_1384::RULES.deck_limit }, effect: authored::card_1384::RULES.checked_effect(), triggers: authored::card_1384::RULES.triggers, activated: authored::card_1384::RULES.activated, enters: authored::card_1384::RULES.enters, entry: authored::card_1384::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 0, 0, 0, 2, 0], generic: 3 }, rules: &authored::card_1384::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Evendo, Waking Haven", oracle_id: Some("83161d59-2520-4741-9328-e2a4a8b5d5bc"), mana_cost: "", type_line: "Land — Planet", oracle_keywords: &["Station"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_1385::RULES.keywords, deck_limit: authored::card_1385::RULES.deck_limit }, effect: authored::card_1385::RULES.effect, triggers: authored::card_1385::RULES.triggers, activated: authored::card_1385::RULES.activated, enters: authored::card_1385::RULES.enters, entry: authored::card_1385::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_1385::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Exalted Sunborn", oracle_id: Some("43febbcd-ecbe-4edb-ad90-7fc6dc381d05"), mana_cost: "{3}{W}{W}", type_line: "Creature — Angel Wizard", oracle_keywords: &["Flying", "Lifelink", "Warp"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1392::RULES.keywords, deck_limit: authored::card_1392::RULES.deck_limit }, effect: authored::card_1392::RULES.checked_effect(), triggers: authored::card_1392::RULES.triggers, activated: authored::card_1392::RULES.activated, enters: authored::card_1392::RULES.enters, entry: authored::card_1392::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [2, 0, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_1392::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Exosuit Savior", oracle_id: Some("8b13ec4d-812a-48b0-8a64-6847a694a03a"), mana_cost: "{2}{W}", type_line: "Creature — Human Soldier", oracle_keywords: &["Flying"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1400::RULES.keywords, deck_limit: authored::card_1400::RULES.deck_limit }, effect: authored::card_1400::RULES.checked_effect(), triggers: authored::card_1400::RULES.triggers, activated: authored::card_1400::RULES.activated, enters: authored::card_1400::RULES.enters, entry: authored::card_1400::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_1400::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Extinguisher Battleship", oracle_id: Some("cce3dcc3-57bb-4b95-8b70-337c67bb3c4e"), mana_cost: "{8}", type_line: "Artifact — Spacecraft", oracle_keywords: &["Station"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_1413::RULES.keywords, deck_limit: authored::card_1413::RULES.deck_limit }, effect: authored::card_1413::RULES.effect, triggers: authored::card_1413::RULES.triggers, activated: authored::card_1413::RULES.activated, enters: authored::card_1413::RULES.enters, entry: authored::card_1413::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 8 }, rules: &authored::card_1413::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Faller's Faithful", oracle_id: Some("df7500e8-76b4-49b3-a427-9a28ffa14637"), mana_cost: "{2}{B}", type_line: "Creature — Human Wizard", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_1434::RULES.keywords, deck_limit: authored::card_1434::RULES.deck_limit }, effect: authored::card_1434::RULES.effect, triggers: authored::card_1434::RULES.triggers, activated: authored::card_1434::RULES.activated, enters: authored::card_1434::RULES.enters, entry: authored::card_1434::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 2 }, rules: &authored::card_1434::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Famished Worldsire", oracle_id: Some("e7f53928-a4cc-41ed-877d-b72d786e77d9"), mana_cost: "{5}{G}{G}{G}", type_line: "Creature — Leviathan", oracle_keywords: &["Ward", "Devour"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_1436::RULES.keywords, deck_limit: authored::card_1436::RULES.deck_limit }, effect: authored::card_1436::RULES.effect, triggers: authored::card_1436::RULES.triggers, activated: authored::card_1436::RULES.activated, enters: authored::card_1436::RULES.enters, entry: authored::card_1436::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 3, 0], generic: 5 }, rules: &authored::card_1436::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Fell Gravship", oracle_id: Some("1a82be68-3b74-4dfc-9068-3abea61db709"), mana_cost: "{2}{B}", type_line: "Artifact — Spacecraft", oracle_keywords: &["Station", "Mill"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_1489::RULES.keywords, deck_limit: authored::card_1489::RULES.deck_limit }, effect: authored::card_1489::RULES.effect, triggers: authored::card_1489::RULES.triggers, activated: authored::card_1489::RULES.activated, enters: authored::card_1489::RULES.enters, entry: authored::card_1489::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 2 }, rules: &authored::card_1489::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Flight-Deck Coordinator", oracle_id: Some("57927909-11a9-44ef-989d-1dcca1f6e23f"), mana_cost: "{2}{W}", type_line: "Creature — Human Soldier", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1559::RULES.keywords, deck_limit: authored::card_1559::RULES.deck_limit }, effect: authored::card_1559::RULES.checked_effect(), triggers: authored::card_1559::RULES.triggers, activated: authored::card_1559::RULES.activated, enters: authored::card_1559::RULES.enters, entry: authored::card_1559::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_1559::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Focus Fire", oracle_id: Some("6c4514be-fda7-45e8-83fe-04d49e2b932c"), mana_cost: "{W}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_1572::RULES.keywords, deck_limit: authored::card_1572::RULES.deck_limit }, effect: authored::card_1572::RULES.effect, triggers: authored::card_1572::RULES.triggers, activated: authored::card_1572::RULES.activated, enters: authored::card_1572::RULES.enters, entry: authored::card_1572::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_1572::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Frenzied Baloth", oracle_id: Some("6a4ef075-9254-4d36-9572-622833fae54e"), mana_cost: "{G}{G}", type_line: "Creature — Beast", oracle_keywords: &["Haste", "Trample"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_1620::RULES.keywords, deck_limit: authored::card_1620::RULES.deck_limit }, effect: authored::card_1620::RULES.effect, triggers: authored::card_1620::RULES.triggers, activated: authored::card_1620::RULES.activated, enters: authored::card_1620::RULES.enters, entry: authored::card_1620::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 2, 0], generic: 0 }, rules: &authored::card_1620::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Frontline War-Rager", oracle_id: Some("610d006a-3ceb-4136-b92b-a500c74d06b0"), mana_cost: "{2}{R}", type_line: "Creature — Kavu Soldier", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1636::RULES.keywords, deck_limit: authored::card_1636::RULES.deck_limit }, effect: authored::card_1636::RULES.checked_effect(), triggers: authored::card_1636::RULES.triggers, activated: authored::card_1636::RULES.activated, enters: authored::card_1636::RULES.enters, entry: authored::card_1636::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 2 }, rules: &authored::card_1636::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Full Bore", oracle_id: Some("4b96d003-d3b4-4181-9f50-06a3e6eb1916"), mana_cost: "{R}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1644::RULES.keywords, deck_limit: authored::card_1644::RULES.deck_limit }, effect: authored::card_1644::RULES.checked_effect(), triggers: authored::card_1644::RULES.triggers, activated: authored::card_1644::RULES.activated, enters: authored::card_1644::RULES.enters, entry: authored::card_1644::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 0 }, rules: &authored::card_1644::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Fungal Colossus", oracle_id: Some("ce25dd89-dcc7-4576-b148-ef4c1cd5290c"), mana_cost: "{6}{G}", type_line: "Creature — Fungus Beast", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1650::RULES.keywords, deck_limit: authored::card_1650::RULES.deck_limit }, effect: authored::card_1650::RULES.checked_effect(), triggers: authored::card_1650::RULES.triggers, activated: authored::card_1650::RULES.activated, enters: authored::card_1650::RULES.enters, entry: authored::card_1650::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 6 }, rules: &authored::card_1650::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Galactic Wayfarer", oracle_id: Some("86746a84-3e8a-44b4-8494-60cbee9816b5"), mana_cost: "{2}{G}", type_line: "Creature — Human Scout", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1663::RULES.keywords, deck_limit: authored::card_1663::RULES.deck_limit }, effect: authored::card_1663::RULES.checked_effect(), triggers: authored::card_1663::RULES.triggers, activated: authored::card_1663::RULES.activated, enters: authored::card_1663::RULES.enters, entry: authored::card_1663::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 2 }, rules: &authored::card_1663::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Galvanizing Sawship", oracle_id: Some("dfe8f77a-cc26-438b-92ae-2ca7a91f813b"), mana_cost: "{5}{R}", type_line: "Artifact — Spacecraft", oracle_keywords: &["Station"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_1675::RULES.keywords, deck_limit: authored::card_1675::RULES.deck_limit }, effect: authored::card_1675::RULES.effect, triggers: authored::card_1675::RULES.triggers, activated: authored::card_1675::RULES.activated, enters: authored::card_1675::RULES.enters, entry: authored::card_1675::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 5 }, rules: &authored::card_1675::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Gene Pollinator", oracle_id: Some("0a356d04-2e86-435b-9766-27b93b468a61"), mana_cost: "{G}", type_line: "Artifact Creature — Robot Insect", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1704::RULES.keywords, deck_limit: authored::card_1704::RULES.deck_limit }, effect: authored::card_1704::RULES.checked_effect(), triggers: authored::card_1704::RULES.triggers, activated: authored::card_1704::RULES.activated, enters: authored::card_1704::RULES.enters, entry: authored::card_1704::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 0 }, rules: &authored::card_1704::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Genemorph Imago", oracle_id: Some("bf3a0585-3976-4078-8beb-6afa79f97027"), mana_cost: "{G}{U}", type_line: "Creature — Insect Druid", oracle_keywords: &["Landfall", "Flying"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1705::RULES.keywords, deck_limit: authored::card_1705::RULES.deck_limit }, effect: authored::card_1705::RULES.checked_effect(), triggers: authored::card_1705::RULES.triggers, activated: authored::card_1705::RULES.activated, enters: authored::card_1705::RULES.enters, entry: authored::card_1705::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 1, 0], generic: 0 }, rules: &authored::card_1705::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Germinating Wurm", oracle_id: Some("8735c62d-1508-4bdc-a5d0-f7c68d892a5e"), mana_cost: "{4}{G}", type_line: "Creature — Plant Wurm", oracle_keywords: &["Warp"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1716::RULES.keywords, deck_limit: authored::card_1716::RULES.deck_limit }, effect: authored::card_1716::RULES.checked_effect(), triggers: authored::card_1716::RULES.triggers, activated: authored::card_1716::RULES.activated, enters: authored::card_1716::RULES.enters, entry: authored::card_1716::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 4 }, rules: &authored::card_1716::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Gigastorm Titan", oracle_id: Some("ef06e10c-553b-45b8-abf0-1d2e87c22128"), mana_cost: "{4}{U}", type_line: "Creature — Elemental", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1748::RULES.keywords, deck_limit: authored::card_1748::RULES.deck_limit }, effect: authored::card_1748::RULES.checked_effect(), triggers: authored::card_1748::RULES.triggers, activated: authored::card_1748::RULES.activated, enters: authored::card_1748::RULES.enters, entry: authored::card_1748::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 4 }, rules: &authored::card_1748::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Glacier Godmaw", oracle_id: Some("300e48df-7f7f-47ff-b450-8ba7bd1b1c67"), mana_cost: "{5}{G}{G}", type_line: "Creature — Leviathan", oracle_keywords: &["Trample", "Landfall"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1761::RULES.keywords, deck_limit: authored::card_1761::RULES.deck_limit }, effect: authored::card_1761::RULES.checked_effect(), triggers: authored::card_1761::RULES.triggers, activated: authored::card_1761::RULES.activated, enters: authored::card_1761::RULES.enters, entry: authored::card_1761::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 2, 0], generic: 5 }, rules: &authored::card_1761::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Godless Shrine", oracle_id: Some("73864fcc-1bde-4bc0-831e-2b93e546e417"), mana_cost: "", type_line: "Land — Plains Swamp", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1815::RULES.keywords, deck_limit: authored::card_1815::RULES.deck_limit }, effect: authored::card_1815::RULES.checked_effect(), triggers: authored::card_1815::RULES.triggers, activated: authored::card_1815::RULES.activated, enters: authored::card_1815::RULES.enters, entry: authored::card_1815::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_1815::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Gravblade Heavy", oracle_id: Some("67821712-e58f-4fee-81af-021059ac72ea"), mana_cost: "{3}{B}", type_line: "Creature — Human Soldier", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1852::RULES.keywords, deck_limit: authored::card_1852::RULES.deck_limit }, effect: authored::card_1852::RULES.checked_effect(), triggers: authored::card_1852::RULES.triggers, activated: authored::card_1852::RULES.activated, enters: authored::card_1852::RULES.enters, entry: authored::card_1852::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 3 }, rules: &authored::card_1852::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Gravkill", oracle_id: Some("15c6837c-775b-4b4a-a2f7-9fd79215fa05"), mana_cost: "{3}{B}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1857::RULES.keywords, deck_limit: authored::card_1857::RULES.deck_limit }, effect: authored::card_1857::RULES.checked_effect(), triggers: authored::card_1857::RULES.triggers, activated: authored::card_1857::RULES.activated, enters: authored::card_1857::RULES.enters, entry: authored::card_1857::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 3 }, rules: &authored::card_1857::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Gravpack Monoist", oracle_id: Some("9c009b3f-6580-4ab8-84e8-d4cf7a456a02"), mana_cost: "{2}{B}", type_line: "Creature — Human Scout", oracle_keywords: &["Flying"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1858::RULES.keywords, deck_limit: authored::card_1858::RULES.deck_limit }, effect: authored::card_1858::RULES.checked_effect(), triggers: authored::card_1858::RULES.triggers, activated: authored::card_1858::RULES.activated, enters: authored::card_1858::RULES.enters, entry: authored::card_1858::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 2 }, rules: &authored::card_1858::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Haliya, Ascendant Cadet", oracle_id: Some("6846448b-d5a0-4ae0-bb11-ea7fa6ed2558"), mana_cost: "{2}{G}{W}{W}", type_line: "Legendary Creature — Human Soldier", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_1928::RULES.keywords, deck_limit: authored::card_1928::RULES.deck_limit }, effect: authored::card_1928::RULES.checked_effect(), triggers: authored::card_1928::RULES.triggers, activated: authored::card_1928::RULES.activated, enters: authored::card_1928::RULES.enters, entry: authored::card_1928::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [2, 0, 0, 0, 1, 0], generic: 2 }, rules: &authored::card_1928::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Haliya, Guided by Light", oracle_id: Some("9b0af3ad-1b4d-4480-af91-fe9baee5d1c5"), mana_cost: "{2}{W}", type_line: "Legendary Creature — Human Soldier", oracle_keywords: &["Warp"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_1929::RULES.keywords, deck_limit: authored::card_1929::RULES.deck_limit }, effect: authored::card_1929::RULES.checked_effect(), triggers: authored::card_1929::RULES.triggers, activated: authored::card_1929::RULES.activated, enters: authored::card_1929::RULES.enters, entry: authored::card_1929::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_1929::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Hardlight Containment", oracle_id: Some("8054092a-1940-4f4b-991d-81f4d384cdb1"), mana_cost: "{W}", type_line: "Enchantment — Aura", oracle_keywords: &["Enchant"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1946::RULES.keywords, deck_limit: authored::card_1946::RULES.deck_limit }, effect: authored::card_1946::RULES.checked_effect(), triggers: authored::card_1946::RULES.triggers, activated: authored::card_1946::RULES.activated, enters: authored::card_1946::RULES.enters, entry: authored::card_1946::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_1946::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Harmonious Grovestrider", oracle_id: Some("0ebec97e-3cb1-41e2-a693-0febb5623016"), mana_cost: "{3}{G}{G}", type_line: "Creature — Beast", oracle_keywords: &["Ward"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1949::RULES.keywords, deck_limit: authored::card_1949::RULES.deck_limit }, effect: authored::card_1949::RULES.checked_effect(), triggers: authored::card_1949::RULES.triggers, activated: authored::card_1949::RULES.activated, enters: authored::card_1949::RULES.enters, entry: authored::card_1949::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 2, 0], generic: 3 }, rules: &authored::card_1949::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Hemosymbic Mite", oracle_id: Some("103e0b63-fbde-4054-91f7-dd04b40b59ef"), mana_cost: "{G}", type_line: "Creature — Mite", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_2004::RULES.keywords, deck_limit: authored::card_2004::RULES.deck_limit }, effect: authored::card_2004::RULES.effect, triggers: authored::card_2004::RULES.triggers, activated: authored::card_2004::RULES.activated, enters: authored::card_2004::RULES.enters, entry: authored::card_2004::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 0 }, rules: &authored::card_2004::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Honor", oracle_id: Some("48c0cd06-465d-4901-bf23-948a003afc0b"), mana_cost: "{W}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2065::RULES.keywords, deck_limit: authored::card_2065::RULES.deck_limit }, effect: authored::card_2065::RULES.checked_effect(), triggers: authored::card_2065::RULES.triggers, activated: authored::card_2065::RULES.activated, enters: authored::card_2065::RULES.enters, entry: authored::card_2065::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_2065::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Honored Knight-Captain", oracle_id: Some("69741d3b-1259-43f5-897f-6f4c3ca0844b"), mana_cost: "{1}{W}", type_line: "Creature — Human Advisor Knight", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2068::RULES.keywords, deck_limit: authored::card_2068::RULES.deck_limit }, effect: authored::card_2068::RULES.checked_effect(), triggers: authored::card_2068::RULES.triggers, activated: authored::card_2068::RULES.activated, enters: authored::card_2068::RULES.enters, entry: authored::card_2068::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_2068::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Hullcarver", oracle_id: Some("931c2153-e5e5-4642-bff0-f9dadcc24c9a"), mana_cost: "{B}", type_line: "Artifact Creature — Robot Assassin", oracle_keywords: &["Deathtouch"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2095::RULES.keywords, deck_limit: authored::card_2095::RULES.deck_limit }, effect: authored::card_2095::RULES.checked_effect(), triggers: authored::card_2095::RULES.triggers, activated: authored::card_2095::RULES.activated, enters: authored::card_2095::RULES.enters, entry: authored::card_2095::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 0 }, rules: &authored::card_2095::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Hylderblade", oracle_id: Some("64ca59b8-6e90-46c8-8047-6172b9c84226"), mana_cost: "{B}", type_line: "Artifact — Equipment", oracle_keywords: &["Equip", "Void"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_2116::RULES.keywords, deck_limit: authored::card_2116::RULES.deck_limit }, effect: authored::card_2116::RULES.effect, triggers: authored::card_2116::RULES.triggers, activated: authored::card_2116::RULES.activated, enters: authored::card_2116::RULES.enters, entry: authored::card_2116::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 0 }, rules: &authored::card_2116::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Hymn of the Faller", oracle_id: Some("ab1379b7-0016-4d2a-a83b-952666e43b89"), mana_cost: "{1}{B}", type_line: "Sorcery", oracle_keywords: &["Surveil", "Void"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_2117::RULES.keywords, deck_limit: authored::card_2117::RULES.deck_limit }, effect: authored::card_2117::RULES.effect, triggers: authored::card_2117::RULES.triggers, activated: authored::card_2117::RULES.activated, enters: authored::card_2117::RULES.enters, entry: authored::card_2117::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 1 }, rules: &authored::card_2117::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Icecave Crasher", oracle_id: Some("b9650a0e-5795-42ff-84c6-6040e5cf4c90"), mana_cost: "{3}{G}", type_line: "Creature — Beast", oracle_keywords: &["Trample", "Landfall"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2123::RULES.keywords, deck_limit: authored::card_2123::RULES.deck_limit }, effect: authored::card_2123::RULES.checked_effect(), triggers: authored::card_2123::RULES.triggers, activated: authored::card_2123::RULES.activated, enters: authored::card_2123::RULES.enters, entry: authored::card_2123::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 3 }, rules: &authored::card_2123::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Icetill Explorer", oracle_id: Some("109cdefd-e8cc-4ac7-b6ba-2cfdef8d780f"), mana_cost: "{2}{G}{G}", type_line: "Creature — Insect Scout", oracle_keywords: &["Mill", "Landfall"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2125::RULES.keywords, deck_limit: authored::card_2125::RULES.deck_limit }, effect: authored::card_2125::RULES.checked_effect(), triggers: authored::card_2125::RULES.triggers, activated: authored::card_2125::RULES.activated, enters: authored::card_2125::RULES.enters, entry: authored::card_2125::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 2, 0], generic: 2 }, rules: &authored::card_2125::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Illvoi Galeblade", oracle_id: Some("d6273621-c36c-491b-85ca-37445ed30097"), mana_cost: "{U}", type_line: "Creature — Jellyfish Warrior", oracle_keywords: &["Flying", "Flash"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2138::RULES.keywords, deck_limit: authored::card_2138::RULES.deck_limit }, effect: authored::card_2138::RULES.checked_effect(), triggers: authored::card_2138::RULES.triggers, activated: authored::card_2138::RULES.activated, enters: authored::card_2138::RULES.enters, entry: authored::card_2138::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_2138::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Illvoi Infiltrator", oracle_id: Some("3084377f-73fb-4b00-9dd9-3844b1ecafc2"), mana_cost: "{2}{U}", type_line: "Creature — Jellyfish Rogue", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_2139::RULES.keywords, deck_limit: authored::card_2139::RULES.deck_limit }, effect: authored::card_2139::RULES.effect, triggers: authored::card_2139::RULES.triggers, activated: authored::card_2139::RULES.activated, enters: authored::card_2139::RULES.enters, entry: authored::card_2139::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_2139::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Illvoi Light Jammer", oracle_id: Some("cbc0b136-544d-45a6-be3e-cd21a573bf19"), mana_cost: "{1}{U}", type_line: "Artifact — Equipment", oracle_keywords: &["Equip", "Flash"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2140::RULES.keywords, deck_limit: authored::card_2140::RULES.deck_limit }, effect: authored::card_2140::RULES.checked_effect(), triggers: authored::card_2140::RULES.triggers, activated: authored::card_2140::RULES.activated, enters: authored::card_2140::RULES.enters, entry: authored::card_2140::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_2140::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Illvoi Operative", oracle_id: Some("2e0b225a-d4b4-4f53-8fea-8c6c37102920"), mana_cost: "{1}{U}", type_line: "Creature — Jellyfish Rogue", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2141::RULES.keywords, deck_limit: authored::card_2141::RULES.deck_limit }, effect: authored::card_2141::RULES.checked_effect(), triggers: authored::card_2141::RULES.triggers, activated: authored::card_2141::RULES.activated, enters: authored::card_2141::RULES.enters, entry: authored::card_2141::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_2141::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Infinite Guideline Station", oracle_id: Some("cda39514-86c9-483e-8d95-fd08138ce33a"), mana_cost: "{W}{U}{B}{R}{G}", type_line: "Legendary Artifact — Spacecraft", oracle_keywords: &["Station"], standard_legal: true, legendary: true, implemented: false, keywords: authored::card_2163::RULES.keywords, deck_limit: authored::card_2163::RULES.deck_limit }, effect: authored::card_2163::RULES.effect, triggers: authored::card_2163::RULES.triggers, activated: authored::card_2163::RULES.activated, enters: authored::card_2163::RULES.enters, entry: authored::card_2163::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [1, 1, 1, 1, 1, 0], generic: 0 }, rules: &authored::card_2163::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Insatiable Skittermaw", oracle_id: Some("69a68dec-4b01-4528-81fc-a3c5f5097f72"), mana_cost: "{2}{B}", type_line: "Creature — Insect Horror", oracle_keywords: &["Menace", "Void"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_2178::RULES.keywords, deck_limit: authored::card_2178::RULES.deck_limit }, effect: authored::card_2178::RULES.effect, triggers: authored::card_2178::RULES.triggers, activated: authored::card_2178::RULES.activated, enters: authored::card_2178::RULES.enters, entry: authored::card_2178::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 2 }, rules: &authored::card_2178::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Interceptor Mechan", oracle_id: Some("ded2b1b7-8c12-4f6f-a3be-a8f7b62b0819"), mana_cost: "{2}{B}{R}", type_line: "Artifact Creature — Robot", oracle_keywords: &["Flying", "Void"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_2193::RULES.keywords, deck_limit: authored::card_2193::RULES.deck_limit }, effect: authored::card_2193::RULES.effect, triggers: authored::card_2193::RULES.triggers, activated: authored::card_2193::RULES.activated, enters: authored::card_2193::RULES.enters, entry: authored::card_2193::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 0, 1, 1, 0, 0], generic: 2 }, rules: &authored::card_2193::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Intrepid Tenderfoot", oracle_id: Some("94023b23-339a-426e-85cc-cc361ef68754"), mana_cost: "{1}{G}", type_line: "Creature — Insect Citizen", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2206::RULES.keywords, deck_limit: authored::card_2206::RULES.deck_limit }, effect: authored::card_2206::RULES.checked_effect(), triggers: authored::card_2206::RULES.triggers, activated: authored::card_2206::RULES.activated, enters: authored::card_2206::RULES.enters, entry: authored::card_2206::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 1 }, rules: &authored::card_2206::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Invasive Maneuvers", oracle_id: Some("4c376349-1a50-4215-8a40-f702da81eb7b"), mana_cost: "{1}{R}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_2213::RULES.keywords, deck_limit: authored::card_2213::RULES.deck_limit }, effect: authored::card_2213::RULES.effect, triggers: authored::card_2213::RULES.triggers, activated: authored::card_2213::RULES.activated, enters: authored::card_2213::RULES.enters, entry: authored::card_2213::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 1 }, rules: &authored::card_2213::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Kav Landseeker", oracle_id: Some("528fe479-dab9-4cf8-bda0-1d226a644366"), mana_cost: "{3}{R}", type_line: "Creature — Kavu Soldier", oracle_keywords: &["Menace"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_2326::RULES.keywords, deck_limit: authored::card_2326::RULES.deck_limit }, effect: authored::card_2326::RULES.effect, triggers: authored::card_2326::RULES.triggers, activated: authored::card_2326::RULES.activated, enters: authored::card_2326::RULES.enters, entry: authored::card_2326::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 3 }, rules: &authored::card_2326::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Kavaron Harrier", oracle_id: Some("1972e0db-e595-4ec2-b97c-902561bb275b"), mana_cost: "{R}", type_line: "Artifact Creature — Robot Soldier", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_2327::RULES.keywords, deck_limit: authored::card_2327::RULES.deck_limit }, effect: authored::card_2327::RULES.effect, triggers: authored::card_2327::RULES.triggers, activated: authored::card_2327::RULES.activated, enters: authored::card_2327::RULES.enters, entry: authored::card_2327::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 0 }, rules: &authored::card_2327::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Kavaron Skywarden", oracle_id: Some("c1f1ba17-b5e8-4bb3-a767-befd52e2db72"), mana_cost: "{4}{R}", type_line: "Creature — Kavu Soldier", oracle_keywords: &["Reach", "Void"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_2328::RULES.keywords, deck_limit: authored::card_2328::RULES.deck_limit }, effect: authored::card_2328::RULES.effect, triggers: authored::card_2328::RULES.triggers, activated: authored::card_2328::RULES.activated, enters: authored::card_2328::RULES.enters, entry: authored::card_2328::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 4 }, rules: &authored::card_2328::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Kavaron Turbodrone", oracle_id: Some("3ebb8542-5d1d-4f0b-92a9-5a98425860fd"), mana_cost: "{2}{R}", type_line: "Artifact Creature — Robot Scout", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2329::RULES.keywords, deck_limit: authored::card_2329::RULES.deck_limit }, effect: authored::card_2329::RULES.checked_effect(), triggers: authored::card_2329::RULES.triggers, activated: authored::card_2329::RULES.activated, enters: authored::card_2329::RULES.enters, entry: authored::card_2329::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 2 }, rules: &authored::card_2329::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Kavaron, Memorial World", oracle_id: Some("4fa826ca-d361-4391-ad0d-989ebcfa4a91"), mana_cost: "", type_line: "Land — Planet", oracle_keywords: &["Station"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_2330::RULES.keywords, deck_limit: authored::card_2330::RULES.deck_limit }, effect: authored::card_2330::RULES.effect, triggers: authored::card_2330::RULES.triggers, activated: authored::card_2330::RULES.activated, enters: authored::card_2330::RULES.enters, entry: authored::card_2330::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_2330::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Knight Luminary", oracle_id: Some("11809d9b-30f3-49d5-9119-e45b4de044c8"), mana_cost: "{3}{W}", type_line: "Creature — Human Knight", oracle_keywords: &["Warp"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2383::RULES.keywords, deck_limit: authored::card_2383::RULES.deck_limit }, effect: authored::card_2383::RULES.checked_effect(), triggers: authored::card_2383::RULES.triggers, activated: authored::card_2383::RULES.activated, enters: authored::card_2383::RULES.enters, entry: authored::card_2383::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_2383::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Larval Scoutlander", oracle_id: Some("b3f3d55c-a3f6-441b-a380-60c0734dc514"), mana_cost: "{2}{G}", type_line: "Artifact — Spacecraft", oracle_keywords: &["Station"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_2446::RULES.keywords, deck_limit: authored::card_2446::RULES.deck_limit }, effect: authored::card_2446::RULES.effect, triggers: authored::card_2446::RULES.triggers, activated: authored::card_2446::RULES.activated, enters: authored::card_2446::RULES.enters, entry: authored::card_2446::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 2 }, rules: &authored::card_2446::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Lashwhip Predator", oracle_id: Some("56035e66-e730-436d-8de2-4ded7fd1b51e"), mana_cost: "{4}{G}{G}", type_line: "Creature — Plant Beast", oracle_keywords: &["Reach"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_2447::RULES.keywords, deck_limit: authored::card_2447::RULES.deck_limit }, effect: authored::card_2447::RULES.effect, triggers: authored::card_2447::RULES.triggers, activated: authored::card_2447::RULES.activated, enters: authored::card_2447::RULES.enters, entry: authored::card_2447::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 2, 0], generic: 4 }, rules: &authored::card_2447::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Lightless Evangel", oracle_id: Some("b162ccf8-e0c2-46ae-84e7-dff2b6fceec1"), mana_cost: "{1}{B}", type_line: "Creature — Vampire Cleric", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2495::RULES.keywords, deck_limit: authored::card_2495::RULES.deck_limit }, effect: authored::card_2495::RULES.checked_effect(), triggers: authored::card_2495::RULES.triggers, activated: authored::card_2495::RULES.activated, enters: authored::card_2495::RULES.enters, entry: authored::card_2495::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 1 }, rules: &authored::card_2495::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Lightstall Inquisitor", oracle_id: Some("57012049-06d7-4354-8125-9ea9e742f14d"), mana_cost: "{W}", type_line: "Creature — Angel Wizard", oracle_keywords: &["Vigilance"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_2502::RULES.keywords, deck_limit: authored::card_2502::RULES.deck_limit }, effect: authored::card_2502::RULES.effect, triggers: authored::card_2502::RULES.triggers, activated: authored::card_2502::RULES.activated, enters: authored::card_2502::RULES.enters, entry: authored::card_2502::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_2502::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Lithobraking", oracle_id: Some("118c4be1-2204-4133-a80e-990ef96c7e35"), mana_cost: "{2}{R}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_2517::RULES.keywords, deck_limit: authored::card_2517::RULES.deck_limit }, effect: authored::card_2517::RULES.effect, triggers: authored::card_2517::RULES.triggers, activated: authored::card_2517::RULES.activated, enters: authored::card_2517::RULES.enters, entry: authored::card_2517::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 2 }, rules: &authored::card_2517::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Loading Zone", oracle_id: Some("16156274-8dc0-439c-94c4-e8c89bbb5687"), mana_cost: "{3}{G}", type_line: "Enchantment", oracle_keywords: &["Warp"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2531::RULES.keywords, deck_limit: authored::card_2531::RULES.deck_limit }, effect: authored::card_2531::RULES.checked_effect(), triggers: authored::card_2531::RULES.triggers, activated: authored::card_2531::RULES.activated, enters: authored::card_2531::RULES.enters, entry: authored::card_2531::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 3 }, rules: &authored::card_2531::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Lost in Space", oracle_id: Some("cec95b9f-9d1e-4988-befe-d0c9e20e428d"), mana_cost: "{3}{U}", type_line: "Instant", oracle_keywords: &["Surveil"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2564::RULES.keywords, deck_limit: authored::card_2564::RULES.deck_limit }, effect: authored::card_2564::RULES.checked_effect(), triggers: authored::card_2564::RULES.triggers, activated: authored::card_2564::RULES.activated, enters: authored::card_2564::RULES.enters, entry: authored::card_2564::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_2564::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Lumen-Class Frigate", oracle_id: Some("3aa0cda4-fb2d-4fd4-84f6-482089a85547"), mana_cost: "{1}{W}", type_line: "Artifact — Spacecraft", oracle_keywords: &["Station"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_2578::RULES.keywords, deck_limit: authored::card_2578::RULES.deck_limit }, effect: authored::card_2578::RULES.effect, triggers: authored::card_2578::RULES.triggers, activated: authored::card_2578::RULES.activated, enters: authored::card_2578::RULES.enters, entry: authored::card_2578::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_2578::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Luxknight Breacher", oracle_id: Some("e0a508b5-0896-4620-a93e-60b958829703"), mana_cost: "{3}{W}", type_line: "Creature — Human Knight", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2589::RULES.keywords, deck_limit: authored::card_2589::RULES.deck_limit }, effect: authored::card_2589::RULES.checked_effect(), triggers: authored::card_2589::RULES.triggers, activated: authored::card_2589::RULES.activated, enters: authored::card_2589::RULES.enters, entry: authored::card_2589::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_2589::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Mechan Assembler", oracle_id: Some("2c2ef1ed-2f93-48c5-bf86-e46318599aeb"), mana_cost: "{4}{U}", type_line: "Artifact Creature — Robot Artificer", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2696::RULES.keywords, deck_limit: authored::card_2696::RULES.deck_limit }, effect: authored::card_2696::RULES.checked_effect(), triggers: authored::card_2696::RULES.triggers, activated: authored::card_2696::RULES.activated, enters: authored::card_2696::RULES.enters, entry: authored::card_2696::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 4 }, rules: &authored::card_2696::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Mechan Navigator", oracle_id: Some("fa72c607-7eb4-41b7-9af1-f3d6c69abf09"), mana_cost: "{1}{U}", type_line: "Artifact Creature — Robot Pilot", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2697::RULES.keywords, deck_limit: authored::card_2697::RULES.deck_limit }, effect: authored::card_2697::RULES.checked_effect(), triggers: authored::card_2697::RULES.triggers, activated: authored::card_2697::RULES.activated, enters: authored::card_2697::RULES.enters, entry: authored::card_2697::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_2697::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Mechan Shieldmate", oracle_id: Some("d0025d29-d652-4af8-b367-13142d5532df"), mana_cost: "{1}{U}", type_line: "Artifact Creature — Robot Soldier", oracle_keywords: &["Defender"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_2698::RULES.keywords, deck_limit: authored::card_2698::RULES.deck_limit }, effect: authored::card_2698::RULES.effect, triggers: authored::card_2698::RULES.triggers, activated: authored::card_2698::RULES.activated, enters: authored::card_2698::RULES.enters, entry: authored::card_2698::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_2698::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Mechanozoa", oracle_id: Some("1aeb43f7-4d8a-4648-9c13-fd101081b448"), mana_cost: "{4}{U}{U}", type_line: "Artifact Creature — Robot Jellyfish", oracle_keywords: &["Warp"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2701::RULES.keywords, deck_limit: authored::card_2701::RULES.deck_limit }, effect: authored::card_2701::RULES.checked_effect(), triggers: authored::card_2701::RULES.triggers, activated: authored::card_2701::RULES.activated, enters: authored::card_2701::RULES.enters, entry: authored::card_2701::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 2, 0, 0, 0, 0], generic: 4 }, rules: &authored::card_2701::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Melded Moxite", oracle_id: Some("86e0e718-82b5-4c99-9837-1b4d8729a27e"), mana_cost: "{1}{R}", type_line: "Artifact", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2707::RULES.keywords, deck_limit: authored::card_2707::RULES.deck_limit }, effect: authored::card_2707::RULES.checked_effect(), triggers: authored::card_2707::RULES.triggers, activated: authored::card_2707::RULES.activated, enters: authored::card_2707::RULES.enters, entry: authored::card_2707::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 1 }, rules: &authored::card_2707::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Meltstrider Eulogist", oracle_id: Some("046b60da-0a14-40b0-a36c-5328f6b8972b"), mana_cost: "{2}{G}", type_line: "Creature — Insect Soldier", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_2709::RULES.keywords, deck_limit: authored::card_2709::RULES.deck_limit }, effect: authored::card_2709::RULES.effect, triggers: authored::card_2709::RULES.triggers, activated: authored::card_2709::RULES.activated, enters: authored::card_2709::RULES.enters, entry: authored::card_2709::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 2 }, rules: &authored::card_2709::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Meltstrider's Gear", oracle_id: Some("1d6742a3-592b-4e67-8cc6-56e2962ad7c7"), mana_cost: "{G}", type_line: "Artifact — Equipment", oracle_keywords: &["Equip"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2710::RULES.keywords, deck_limit: authored::card_2710::RULES.deck_limit }, effect: authored::card_2710::RULES.checked_effect(), triggers: authored::card_2710::RULES.triggers, activated: authored::card_2710::RULES.activated, enters: authored::card_2710::RULES.enters, entry: authored::card_2710::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 0 }, rules: &authored::card_2710::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Meltstrider's Resolve", oracle_id: Some("6f9bc0c6-1792-4771-a839-95a22d9c699a"), mana_cost: "{G}", type_line: "Enchantment — Aura", oracle_keywords: &["Fight", "Enchant"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2711::RULES.keywords, deck_limit: authored::card_2711::RULES.deck_limit }, effect: authored::card_2711::RULES.checked_effect(), triggers: authored::card_2711::RULES.triggers, activated: authored::card_2711::RULES.activated, enters: authored::card_2711::RULES.enters, entry: authored::card_2711::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 0 }, rules: &authored::card_2711::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Memorial Team Leader", oracle_id: Some("c71d3775-0c71-4e37-979d-35e319f357f0"), mana_cost: "{3}{R}", type_line: "Creature — Kavu Soldier", oracle_keywords: &["Warp"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2712::RULES.keywords, deck_limit: authored::card_2712::RULES.deck_limit }, effect: authored::card_2712::RULES.checked_effect(), triggers: authored::card_2712::RULES.triggers, activated: authored::card_2712::RULES.activated, enters: authored::card_2712::RULES.enters, entry: authored::card_2712::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 3 }, rules: &authored::card_2712::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Memorial Vault", oracle_id: Some("10dda10f-2549-47e5-9a8b-fb414cf38d3f"), mana_cost: "{3}{R}", type_line: "Artifact", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_2713::RULES.keywords, deck_limit: authored::card_2713::RULES.deck_limit }, effect: authored::card_2713::RULES.effect, triggers: authored::card_2713::RULES.triggers, activated: authored::card_2713::RULES.activated, enters: authored::card_2713::RULES.enters, entry: authored::card_2713::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 3 }, rules: &authored::card_2713::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Mental Modulation", oracle_id: Some("448d9391-9d15-4aa1-87b3-a1d755666c03"), mana_cost: "{1}{U}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2719::RULES.keywords, deck_limit: authored::card_2719::RULES.deck_limit }, effect: authored::card_2719::RULES.checked_effect(), triggers: authored::card_2719::RULES.triggers, activated: authored::card_2719::RULES.activated, enters: authored::card_2719::RULES.enters, entry: authored::card_2719::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_2719::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Mightform Harmonizer", oracle_id: Some("8e025cda-71c5-4d06-8436-decd408667af"), mana_cost: "{2}{G}{G}", type_line: "Creature — Insect Druid", oracle_keywords: &["Warp", "Double", "Landfall"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2752::RULES.keywords, deck_limit: authored::card_2752::RULES.deck_limit }, effect: authored::card_2752::RULES.checked_effect(), triggers: authored::card_2752::RULES.triggers, activated: authored::card_2752::RULES.activated, enters: authored::card_2752::RULES.enters, entry: authored::card_2752::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 2, 0], generic: 2 }, rules: &authored::card_2752::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Mm'menon, Uthros Exile", oracle_id: Some("6c05b882-51d9-43d8-93b8-c3317590f76a"), mana_cost: "{1}{U}{R}", type_line: "Legendary Creature — Jellyfish Advisor", oracle_keywords: &["Flying"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2802::RULES.keywords, deck_limit: authored::card_2802::RULES.deck_limit }, effect: authored::card_2802::RULES.checked_effect(), triggers: authored::card_2802::RULES.triggers, activated: authored::card_2802::RULES.activated, enters: authored::card_2802::RULES.enters, entry: authored::card_2802::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 1, 0, 0], generic: 1 }, rules: &authored::card_2802::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Mm'menon, the Right Hand", oracle_id: Some("de58fa4b-eeeb-4dc8-aab9-6521b15678cc"), mana_cost: "{3}{U}{U}", type_line: "Legendary Creature — Jellyfish Advisor", oracle_keywords: &["Flying"], standard_legal: true, legendary: true, implemented: false, keywords: authored::card_2803::RULES.keywords, deck_limit: authored::card_2803::RULES.deck_limit }, effect: authored::card_2803::RULES.effect, triggers: authored::card_2803::RULES.triggers, activated: authored::card_2803::RULES.activated, enters: authored::card_2803::RULES.enters, entry: authored::card_2803::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 2, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_2803::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Molecular Modifier", oracle_id: Some("d0e58b8e-2c41-4602-a905-c2a6086ecf70"), mana_cost: "{2}{R}", type_line: "Creature — Kavu Artificer", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2812::RULES.keywords, deck_limit: authored::card_2812::RULES.deck_limit }, effect: authored::card_2812::RULES.checked_effect(), triggers: authored::card_2812::RULES.triggers, activated: authored::card_2812::RULES.activated, enters: authored::card_2812::RULES.enters, entry: authored::card_2812::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 2 }, rules: &authored::card_2812::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Monoist Circuit-Feeder", oracle_id: Some("de70a6bb-32ed-4a89-a522-c2a71f08fb02"), mana_cost: "{4}{B}{B}", type_line: "Artifact Creature — Nautilus", oracle_keywords: &["Flying"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_2834::RULES.keywords, deck_limit: authored::card_2834::RULES.deck_limit }, effect: authored::card_2834::RULES.effect, triggers: authored::card_2834::RULES.triggers, activated: authored::card_2834::RULES.activated, enters: authored::card_2834::RULES.enters, entry: authored::card_2834::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 0, 2, 0, 0, 0], generic: 4 }, rules: &authored::card_2834::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Monoist Sentry", oracle_id: Some("0b4cb111-ecf7-4ff7-a9e3-c944054f3370"), mana_cost: "{B}", type_line: "Artifact Creature — Robot", oracle_keywords: &["Defender"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2835::RULES.keywords, deck_limit: authored::card_2835::RULES.deck_limit }, effect: authored::card_2835::RULES.checked_effect(), triggers: authored::card_2835::RULES.triggers, activated: authored::card_2835::RULES.activated, enters: authored::card_2835::RULES.enters, entry: authored::card_2835::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 0 }, rules: &authored::card_2835::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Moonlit Meditation", oracle_id: Some("df702b0c-e011-497f-ae29-9876efac4a4c"), mana_cost: "{2}{U}", type_line: "Enchantment — Aura", oracle_keywords: &["Enchant"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_2844::RULES.keywords, deck_limit: authored::card_2844::RULES.deck_limit }, effect: authored::card_2844::RULES.effect, triggers: authored::card_2844::RULES.triggers, activated: authored::card_2844::RULES.activated, enters: authored::card_2844::RULES.enters, entry: authored::card_2844::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_2844::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Mouth of the Storm", oracle_id: Some("832fbb1f-0ff5-4d6c-b1ee-2231923a6f64"), mana_cost: "{6}{U}", type_line: "Creature — Elemental", oracle_keywords: &["Flying", "Ward"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_2867::RULES.keywords, deck_limit: authored::card_2867::RULES.deck_limit }, effect: authored::card_2867::RULES.effect, triggers: authored::card_2867::RULES.triggers, activated: authored::card_2867::RULES.activated, enters: authored::card_2867::RULES.enters, entry: authored::card_2867::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 6 }, rules: &authored::card_2867::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Mutinous Massacre", oracle_id: Some("9fdd445a-c8e1-4309-9d74-374676fcac1d"), mana_cost: "{3}{B}{B}{R}{R}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_2892::RULES.keywords, deck_limit: authored::card_2892::RULES.deck_limit }, effect: authored::card_2892::RULES.effect, triggers: authored::card_2892::RULES.triggers, activated: authored::card_2892::RULES.activated, enters: authored::card_2892::RULES.enters, entry: authored::card_2892::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 2, 2, 0, 0], generic: 3 }, rules: &authored::card_2892::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Nanoform Sentinel", oracle_id: Some("07155470-c5ab-41ae-ade3-bdc06d357379"), mana_cost: "{2}{U}", type_line: "Artifact Creature — Robot", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_2906::RULES.keywords, deck_limit: authored::card_2906::RULES.deck_limit }, effect: authored::card_2906::RULES.effect, triggers: authored::card_2906::RULES.triggers, activated: authored::card_2906::RULES.activated, enters: authored::card_2906::RULES.enters, entry: authored::card_2906::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_2906::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Nebula Dragon", oracle_id: Some("724b4a59-9c69-4f2b-a436-72183c62f1ff"), mana_cost: "{6}{R}", type_line: "Creature — Dragon", oracle_keywords: &["Flying"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2912::RULES.keywords, deck_limit: authored::card_2912::RULES.deck_limit }, effect: authored::card_2912::RULES.checked_effect(), triggers: authored::card_2912::RULES.triggers, activated: authored::card_2912::RULES.activated, enters: authored::card_2912::RULES.enters, entry: authored::card_2912::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 6 }, rules: &authored::card_2912::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Nova Hellkite", oracle_id: Some("9635d79e-62ec-4d80-ae37-2793489cc13b"), mana_cost: "{3}{R}{R}", type_line: "Creature — Dragon", oracle_keywords: &["Flying", "Haste", "Warp"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2970::RULES.keywords, deck_limit: authored::card_2970::RULES.deck_limit }, effect: authored::card_2970::RULES.checked_effect(), triggers: authored::card_2970::RULES.triggers, activated: authored::card_2970::RULES.activated, enters: authored::card_2970::RULES.enters, entry: authored::card_2970::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 2, 0, 0], generic: 3 }, rules: &authored::card_2970::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Nutrient Block", oracle_id: Some("04a0382f-7e4e-4450-ae70-18999379d875"), mana_cost: "{1}", type_line: "Artifact — Food", oracle_keywords: &["Indestructible"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2980::RULES.keywords, deck_limit: authored::card_2980::RULES.deck_limit }, effect: authored::card_2980::RULES.checked_effect(), triggers: authored::card_2980::RULES.triggers, activated: authored::card_2980::RULES.activated, enters: authored::card_2980::RULES.enters, entry: authored::card_2980::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_2980::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Orbital Plunge", oracle_id: Some("4f10b164-6d36-4935-92e7-53be73ac4c8a"), mana_cost: "{3}{R}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_3032::RULES.keywords, deck_limit: authored::card_3032::RULES.deck_limit }, effect: authored::card_3032::RULES.effect, triggers: authored::card_3032::RULES.triggers, activated: authored::card_3032::RULES.activated, enters: authored::card_3032::RULES.enters, entry: authored::card_3032::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 3 }, rules: &authored::card_3032::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Oreplate Pangolin", oracle_id: Some("72e8c978-f66d-42e1-ac5f-d1c307d7b93c"), mana_cost: "{1}{R}", type_line: "Artifact Creature — Robot Pangolin", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_3036::RULES.keywords, deck_limit: authored::card_3036::RULES.deck_limit }, effect: authored::card_3036::RULES.effect, triggers: authored::card_3036::RULES.triggers, activated: authored::card_3036::RULES.activated, enters: authored::card_3036::RULES.enters, entry: authored::card_3036::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 1 }, rules: &authored::card_3036::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Ouroboroid", oracle_id: Some("50d6fd91-23d3-4d32-804f-6233e4386904"), mana_cost: "{2}{G}{G}", type_line: "Creature — Plant Wurm", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_3055::RULES.keywords, deck_limit: authored::card_3055::RULES.deck_limit }, effect: authored::card_3055::RULES.effect, triggers: authored::card_3055::RULES.triggers, activated: authored::card_3055::RULES.activated, enters: authored::card_3055::RULES.enters, entry: authored::card_3055::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 2, 0], generic: 2 }, rules: &authored::card_3055::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Pain for All", oracle_id: Some("4eedf21c-0ad1-48da-a9fe-2ffdbf8371e9"), mana_cost: "{2}{R}", type_line: "Enchantment — Aura", oracle_keywords: &["Enchant"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_3090::RULES.keywords, deck_limit: authored::card_3090::RULES.deck_limit }, effect: authored::card_3090::RULES.effect, triggers: authored::card_3090::RULES.triggers, activated: authored::card_3090::RULES.activated, enters: authored::card_3090::RULES.enters, entry: authored::card_3090::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 2 }, rules: &authored::card_3090::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Perigee Beckoner", oracle_id: Some("e0a35a44-e56d-434e-8043-43d5031b2e84"), mana_cost: "{4}{B}", type_line: "Creature — Horror", oracle_keywords: &["Warp"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3129::RULES.keywords, deck_limit: authored::card_3129::RULES.deck_limit }, effect: authored::card_3129::RULES.checked_effect(), triggers: authored::card_3129::RULES.triggers, activated: authored::card_3129::RULES.activated, enters: authored::card_3129::RULES.enters, entry: authored::card_3129::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 4 }, rules: &authored::card_3129::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Pinnacle Emissary", oracle_id: Some("522e8db5-857e-492f-afed-d60cd2ce3dcd"), mana_cost: "{1}{U}{R}", type_line: "Artifact Creature — Robot", oracle_keywords: &["Warp"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_3166::RULES.keywords, deck_limit: authored::card_3166::RULES.deck_limit }, effect: authored::card_3166::RULES.effect, triggers: authored::card_3166::RULES.triggers, activated: authored::card_3166::RULES.activated, enters: authored::card_3166::RULES.enters, entry: authored::card_3166::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 1, 0, 1, 0, 0], generic: 1 }, rules: &authored::card_3166::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Pinnacle Kill-Ship", oracle_id: Some("ad556aee-3dbf-4c8b-9f3a-31947e26c6f5"), mana_cost: "{7}", type_line: "Artifact — Spacecraft", oracle_keywords: &["Station"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_3167::RULES.keywords, deck_limit: authored::card_3167::RULES.deck_limit }, effect: authored::card_3167::RULES.effect, triggers: authored::card_3167::RULES.triggers, activated: authored::card_3167::RULES.activated, enters: authored::card_3167::RULES.enters, entry: authored::card_3167::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 7 }, rules: &authored::card_3167::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Pinnacle Starcage", oracle_id: Some("10e0b563-4dfc-4530-a010-0ba40bdf02ac"), mana_cost: "{1}{W}{W}", type_line: "Artifact", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3168::RULES.keywords, deck_limit: authored::card_3168::RULES.deck_limit }, effect: authored::card_3168::RULES.checked_effect(), triggers: authored::card_3168::RULES.triggers, activated: authored::card_3168::RULES.activated, enters: authored::card_3168::RULES.enters, entry: authored::card_3168::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [2, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_3168::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Plasma Bolt", oracle_id: Some("ac5258c1-5b46-4892-a445-bfd56875f039"), mana_cost: "{R}", type_line: "Sorcery", oracle_keywords: &["Void"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_3182::RULES.keywords, deck_limit: authored::card_3182::RULES.deck_limit }, effect: authored::card_3182::RULES.effect, triggers: authored::card_3182::RULES.triggers, activated: authored::card_3182::RULES.activated, enters: authored::card_3182::RULES.enters, entry: authored::card_3182::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 0 }, rules: &authored::card_3182::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Possibility Technician", oracle_id: Some("ebbf84f7-d142-4de5-bfc1-ea86f082d733"), mana_cost: "{2}{R}", type_line: "Creature — Kavu Artificer", oracle_keywords: &["Warp"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_3208::RULES.keywords, deck_limit: authored::card_3208::RULES.deck_limit }, effect: authored::card_3208::RULES.effect, triggers: authored::card_3208::RULES.triggers, activated: authored::card_3208::RULES.activated, enters: authored::card_3208::RULES.enters, entry: authored::card_3208::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 2 }, rules: &authored::card_3208::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Pull Through the Weft", oracle_id: Some("37bd9e54-c93e-4fec-9b21-6940da75ab7a"), mana_cost: "{3}{G}{G}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_3275::RULES.keywords, deck_limit: authored::card_3275::RULES.deck_limit }, effect: authored::card_3275::RULES.effect, triggers: authored::card_3275::RULES.triggers, activated: authored::card_3275::RULES.activated, enters: authored::card_3275::RULES.enters, entry: authored::card_3275::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 0, 0, 2, 0], generic: 3 }, rules: &authored::card_3275::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Pulsar Squadron Ace", oracle_id: Some("8019e879-cfea-40a2-ac0d-86b47ec02b48"), mana_cost: "{1}{W}", type_line: "Creature — Human Pilot", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_3277::RULES.keywords, deck_limit: authored::card_3277::RULES.deck_limit }, effect: authored::card_3277::RULES.effect, triggers: authored::card_3277::RULES.triggers, activated: authored::card_3277::RULES.activated, enters: authored::card_3277::RULES.enters, entry: authored::card_3277::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_3277::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Quantum Riddler", oracle_id: Some("58582fac-4c42-4a1c-9ab1-4a892b500da4"), mana_cost: "{3}{U}{U}", type_line: "Creature — Sphinx", oracle_keywords: &["Flying", "Warp"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3306::RULES.keywords, deck_limit: authored::card_3306::RULES.deck_limit }, effect: authored::card_3306::RULES.checked_effect(), triggers: authored::card_3306::RULES.triggers, activated: authored::card_3306::RULES.activated, enters: authored::card_3306::RULES.enters, entry: authored::card_3306::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 2, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_3306::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Radiant Strike", oracle_id: Some("186efd85-83d4-40cc-b617-76f32dbe99ce"), mana_cost: "{3}{W}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3334::RULES.keywords, deck_limit: authored::card_3334::RULES.deck_limit }, effect: authored::card_3334::RULES.checked_effect(), triggers: authored::card_3334::RULES.triggers, activated: authored::card_3334::RULES.activated, enters: authored::card_3334::RULES.enters, entry: authored::card_3334::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_3334::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Ragost, Deft Gastronaut", oracle_id: Some("76f5445a-5193-4794-8593-b44f961334b6"), mana_cost: "{R}{W}", type_line: "Legendary Creature — Lobster Citizen", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: false, keywords: authored::card_3345::RULES.keywords, deck_limit: authored::card_3345::RULES.deck_limit }, effect: authored::card_3345::RULES.effect, triggers: authored::card_3345::RULES.triggers, activated: authored::card_3345::RULES.activated, enters: authored::card_3345::RULES.enters, entry: authored::card_3345::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 1, 0, 0], generic: 0 }, rules: &authored::card_3345::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Rayblade Trooper", oracle_id: Some("792da586-a687-428c-a10e-75431249df5b"), mana_cost: "{2}{W}", type_line: "Creature — Human Soldier", oracle_keywords: &["Warp"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_3404::RULES.keywords, deck_limit: authored::card_3404::RULES.deck_limit }, effect: authored::card_3404::RULES.effect, triggers: authored::card_3404::RULES.triggers, activated: authored::card_3404::RULES.activated, enters: authored::card_3404::RULES.enters, entry: authored::card_3404::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_3404::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Red Tiger Mechan", oracle_id: Some("94c9aea7-564d-4fc8-ae14-7b22cba18b95"), mana_cost: "{3}{R}", type_line: "Artifact Creature — Robot Cat", oracle_keywords: &["Haste", "Warp"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3431::RULES.keywords, deck_limit: authored::card_3431::RULES.deck_limit }, effect: authored::card_3431::RULES.checked_effect(), triggers: authored::card_3431::RULES.triggers, activated: authored::card_3431::RULES.activated, enters: authored::card_3431::RULES.enters, entry: authored::card_3431::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 3 }, rules: &authored::card_3431::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Remnant Elemental", oracle_id: Some("69e8333a-bddf-49e2-9c39-3dc46fec81e0"), mana_cost: "{1}{R}", type_line: "Creature — Elemental", oracle_keywords: &["Reach", "Landfall"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3456::RULES.keywords, deck_limit: authored::card_3456::RULES.deck_limit }, effect: authored::card_3456::RULES.checked_effect(), triggers: authored::card_3456::RULES.triggers, activated: authored::card_3456::RULES.activated, enters: authored::card_3456::RULES.enters, entry: authored::card_3456::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 1 }, rules: &authored::card_3456::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Requiem Monolith", oracle_id: Some("13a8b19b-23bd-4e45-81ff-23371665e4fc"), mana_cost: "{2}{B}", type_line: "Artifact", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_3470::RULES.keywords, deck_limit: authored::card_3470::RULES.deck_limit }, effect: authored::card_3470::RULES.effect, triggers: authored::card_3470::RULES.triggers, activated: authored::card_3470::RULES.activated, enters: authored::card_3470::RULES.enters, entry: authored::card_3470::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 2 }, rules: &authored::card_3470::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Reroute Systems", oracle_id: Some("26d2762b-4346-40d0-b93d-b0dbfeb6e2bc"), mana_cost: "{W}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_3473::RULES.keywords, deck_limit: authored::card_3473::RULES.deck_limit }, effect: authored::card_3473::RULES.effect, triggers: authored::card_3473::RULES.triggers, activated: authored::card_3473::RULES.activated, enters: authored::card_3473::RULES.enters, entry: authored::card_3473::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_3473::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Rescue Skiff", oracle_id: Some("112aaaf0-3301-4f79-9640-d36c75a0fb30"), mana_cost: "{5}{W}", type_line: "Artifact — Spacecraft", oracle_keywords: &["Station"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_3476::RULES.keywords, deck_limit: authored::card_3476::RULES.deck_limit }, effect: authored::card_3476::RULES.effect, triggers: authored::card_3476::RULES.triggers, activated: authored::card_3476::RULES.activated, enters: authored::card_3476::RULES.enters, entry: authored::card_3476::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 5 }, rules: &authored::card_3476::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Rig for War", oracle_id: Some("05bbd557-e2c0-4855-8fe6-00527fa39878"), mana_cost: "{1}{R}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3523::RULES.keywords, deck_limit: authored::card_3523::RULES.deck_limit }, effect: authored::card_3523::RULES.checked_effect(), triggers: authored::card_3523::RULES.triggers, activated: authored::card_3523::RULES.activated, enters: authored::card_3523::RULES.enters, entry: authored::card_3523::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 1 }, rules: &authored::card_3523::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Roving Actuator", oracle_id: Some("78ac1e6c-2704-4844-9e87-642324c23997"), mana_cost: "{3}{R}", type_line: "Artifact Creature — Robot", oracle_keywords: &["Void"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_3598::RULES.keywords, deck_limit: authored::card_3598::RULES.deck_limit }, effect: authored::card_3598::RULES.effect, triggers: authored::card_3598::RULES.triggers, activated: authored::card_3598::RULES.activated, enters: authored::card_3598::RULES.enters, entry: authored::card_3598::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 3 }, rules: &authored::card_3598::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Ruinous Rampage", oracle_id: Some("5020628c-5e47-4a83-89c3-7bf5726e69ad"), mana_cost: "{1}{R}{R}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3613::RULES.keywords, deck_limit: authored::card_3613::RULES.deck_limit }, effect: authored::card_3613::RULES.checked_effect(), triggers: authored::card_3613::RULES.triggers, activated: authored::card_3613::RULES.activated, enters: authored::card_3613::RULES.enters, entry: authored::card_3613::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 0, 2, 0, 0], generic: 1 }, rules: &authored::card_3613::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Rust Harvester", oracle_id: Some("9ad956c8-71d3-436f-961e-e2dfc1d2ca70"), mana_cost: "{R}", type_line: "Artifact Creature — Robot", oracle_keywords: &["Menace"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_3628::RULES.keywords, deck_limit: authored::card_3628::RULES.deck_limit }, effect: authored::card_3628::RULES.effect, triggers: authored::card_3628::RULES.triggers, activated: authored::card_3628::RULES.activated, enters: authored::card_3628::RULES.enters, entry: authored::card_3628::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 0 }, rules: &authored::card_3628::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Sacred Foundry", oracle_id: Some("45181cb8-2090-4471-ba90-e5a8f04d525f"), mana_cost: "", type_line: "Land — Mountain Plains", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3644::RULES.keywords, deck_limit: authored::card_3644::RULES.deck_limit }, effect: authored::card_3644::RULES.checked_effect(), triggers: authored::card_3644::RULES.triggers, activated: authored::card_3644::RULES.activated, enters: authored::card_3644::RULES.enters, entry: authored::card_3644::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_3644::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Sami's Curiosity", oracle_id: Some("5f532b7f-cb76-4be7-b361-0106c561dcb2"), mana_cost: "{G}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3662::RULES.keywords, deck_limit: authored::card_3662::RULES.deck_limit }, effect: authored::card_3662::RULES.checked_effect(), triggers: authored::card_3662::RULES.triggers, activated: authored::card_3662::RULES.activated, enters: authored::card_3662::RULES.enters, entry: authored::card_3662::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 0 }, rules: &authored::card_3662::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Sami, Ship's Engineer", oracle_id: Some("470c16c5-b078-47f5-89b0-42991e1f7157"), mana_cost: "{2}{R}{W}", type_line: "Legendary Creature — Human Artificer", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_3663::RULES.keywords, deck_limit: authored::card_3663::RULES.deck_limit }, effect: authored::card_3663::RULES.checked_effect(), triggers: authored::card_3663::RULES.triggers, activated: authored::card_3663::RULES.activated, enters: authored::card_3663::RULES.enters, entry: authored::card_3663::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 1, 0, 0], generic: 2 }, rules: &authored::card_3663::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Sami, Wildcat Captain", oracle_id: Some("c4175a34-70bb-47e5-8406-efdc4d2cd079"), mana_cost: "{4}{R}{W}", type_line: "Legendary Creature — Human Artificer Rogue", oracle_keywords: &["Vigilance", "Double strike"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_3664::RULES.keywords, deck_limit: authored::card_3664::RULES.deck_limit }, effect: authored::card_3664::RULES.checked_effect(), triggers: authored::card_3664::RULES.triggers, activated: authored::card_3664::RULES.activated, enters: authored::card_3664::RULES.enters, entry: authored::card_3664::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 1, 0, 0], generic: 4 }, rules: &authored::card_3664::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Scour for Scrap", oracle_id: Some("da422b3e-1092-47ad-97d6-a7a3ef060a53"), mana_cost: "{3}{U}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_3730::RULES.keywords, deck_limit: authored::card_3730::RULES.deck_limit }, effect: authored::card_3730::RULES.effect, triggers: authored::card_3730::RULES.triggers, activated: authored::card_3730::RULES.activated, enters: authored::card_3730::RULES.enters, entry: authored::card_3730::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_3730::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Scout for Survivors", oracle_id: Some("5a367c9c-9a7f-4587-aa05-c0fba46ed838"), mana_cost: "{2}{W}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_3732::RULES.keywords, deck_limit: authored::card_3732::RULES.deck_limit }, effect: authored::card_3732::RULES.effect, triggers: authored::card_3732::RULES.triggers, activated: authored::card_3732::RULES.activated, enters: authored::card_3732::RULES.enters, entry: authored::card_3732::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_3732::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Scrounge for Eternity", oracle_id: Some("71982b17-1d9a-4f98-b8b5-d232e3a8cf1d"), mana_cost: "{2}{B}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_3742::RULES.keywords, deck_limit: authored::card_3742::RULES.deck_limit }, effect: authored::card_3742::RULES.effect, triggers: authored::card_3742::RULES.triggers, activated: authored::card_3742::RULES.activated, enters: authored::card_3742::RULES.enters, entry: authored::card_3742::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 2 }, rules: &authored::card_3742::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Seam Rip", oracle_id: Some("dfd5939e-a71a-4e16-9b93-752162be4ccd"), mana_cost: "{W}", type_line: "Enchantment", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3746::RULES.keywords, deck_limit: authored::card_3746::RULES.deck_limit }, effect: authored::card_3746::RULES.checked_effect(), triggers: authored::card_3746::RULES.triggers, activated: authored::card_3746::RULES.activated, enters: authored::card_3746::RULES.enters, entry: authored::card_3746::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_3746::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Secluded Starforge", oracle_id: Some("69f55a7c-6ddf-412e-b63b-b395731a1ff2"), mana_cost: "", type_line: "Land", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_3758::RULES.keywords, deck_limit: authored::card_3758::RULES.deck_limit }, effect: authored::card_3758::RULES.effect, triggers: authored::card_3758::RULES.triggers, activated: authored::card_3758::RULES.activated, enters: authored::card_3758::RULES.enters, entry: authored::card_3758::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_3758::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Seedship Agrarian", oracle_id: Some("003d9c17-5d5c-469f-ad87-d78faf55e025"), mana_cost: "{3}{G}", type_line: "Creature — Insect Scientist", oracle_keywords: &["Landfall"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3765::RULES.keywords, deck_limit: authored::card_3765::RULES.deck_limit }, effect: authored::card_3765::RULES.checked_effect(), triggers: authored::card_3765::RULES.triggers, activated: authored::card_3765::RULES.activated, enters: authored::card_3765::RULES.enters, entry: authored::card_3765::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 3 }, rules: &authored::card_3765::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Seedship Broodtender", oracle_id: Some("15c994b9-901a-46d6-804b-24560a14a481"), mana_cost: "{B}{G}", type_line: "Creature — Insect Citizen", oracle_keywords: &["Mill"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_3766::RULES.keywords, deck_limit: authored::card_3766::RULES.deck_limit }, effect: authored::card_3766::RULES.effect, triggers: authored::card_3766::RULES.triggers, activated: authored::card_3766::RULES.activated, enters: authored::card_3766::RULES.enters, entry: authored::card_3766::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 1, 0], generic: 0 }, rules: &authored::card_3766::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Seedship Impact", oracle_id: Some("c6d89c4a-351f-4d63-ba57-b978a695191e"), mana_cost: "{1}{G}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_3767::RULES.keywords, deck_limit: authored::card_3767::RULES.deck_limit }, effect: authored::card_3767::RULES.effect, triggers: authored::card_3767::RULES.triggers, activated: authored::card_3767::RULES.activated, enters: authored::card_3767::RULES.enters, entry: authored::card_3767::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 1 }, rules: &authored::card_3767::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Selfcraft Mechan", oracle_id: Some("4ba4cb4b-f421-4abd-8fcf-a01174447790"), mana_cost: "{3}{U}", type_line: "Artifact Creature — Robot Artificer", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_3781::RULES.keywords, deck_limit: authored::card_3781::RULES.deck_limit }, effect: authored::card_3781::RULES.effect, triggers: authored::card_3781::RULES.triggers, activated: authored::card_3781::RULES.activated, enters: authored::card_3781::RULES.enters, entry: authored::card_3781::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_3781::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Shattered Wings", oracle_id: Some("709b9f76-0ea5-456c-b1db-2cf517370fde"), mana_cost: "{2}{G}", type_line: "Sorcery", oracle_keywords: &["Surveil"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3821::RULES.keywords, deck_limit: authored::card_3821::RULES.deck_limit }, effect: authored::card_3821::RULES.checked_effect(), triggers: authored::card_3821::RULES.triggers, activated: authored::card_3821::RULES.activated, enters: authored::card_3821::RULES.enters, entry: authored::card_3821::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 2 }, rules: &authored::card_3821::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Singularity Rupture", oracle_id: Some("e1976b6c-7e43-4f4a-b082-f95495a1b260"), mana_cost: "{3}{U}{B}{B}", type_line: "Sorcery", oracle_keywords: &["Mill"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_3880::RULES.keywords, deck_limit: authored::card_3880::RULES.deck_limit }, effect: authored::card_3880::RULES.effect, triggers: authored::card_3880::RULES.triggers, activated: authored::card_3880::RULES.activated, enters: authored::card_3880::RULES.enters, entry: authored::card_3880::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 1, 2, 0, 0, 0], generic: 3 }, rules: &authored::card_3880::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Sinister Cryologist", oracle_id: Some("be3038ff-b39e-4d2e-97ee-85de074ccfd3"), mana_cost: "{2}{U}", type_line: "Creature — Jellyfish Wizard", oracle_keywords: &["Warp"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3881::RULES.keywords, deck_limit: authored::card_3881::RULES.deck_limit }, effect: authored::card_3881::RULES.checked_effect(), triggers: authored::card_3881::RULES.triggers, activated: authored::card_3881::RULES.activated, enters: authored::card_3881::RULES.enters, entry: authored::card_3881::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_3881::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Skystinger", oracle_id: Some("6bb7419b-e085-4616-8e5c-4ed08cb2b4c1"), mana_cost: "{2}{G}", type_line: "Creature — Insect Warrior", oracle_keywords: &["Reach"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_3908::RULES.keywords, deck_limit: authored::card_3908::RULES.deck_limit }, effect: authored::card_3908::RULES.effect, triggers: authored::card_3908::RULES.triggers, activated: authored::card_3908::RULES.activated, enters: authored::card_3908::RULES.enters, entry: authored::card_3908::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 2 }, rules: &authored::card_3908::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Slagdrill Scrapper", oracle_id: Some("7f1edf4c-46a1-426d-8ce9-757a4c27367d"), mana_cost: "{R}", type_line: "Artifact Creature — Robot Scout", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3911::RULES.keywords, deck_limit: authored::card_3911::RULES.deck_limit }, effect: authored::card_3911::RULES.checked_effect(), triggers: authored::card_3911::RULES.triggers, activated: authored::card_3911::RULES.activated, enters: authored::card_3911::RULES.enters, entry: authored::card_3911::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 0 }, rules: &authored::card_3911::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Sledge-Class Seedship", oracle_id: Some("bcd086ce-de5c-4b34-92a1-ece22055e479"), mana_cost: "{2}{G}", type_line: "Artifact — Spacecraft", oracle_keywords: &["Station"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_3916::RULES.keywords, deck_limit: authored::card_3916::RULES.deck_limit }, effect: authored::card_3916::RULES.effect, triggers: authored::card_3916::RULES.triggers, activated: authored::card_3916::RULES.activated, enters: authored::card_3916::RULES.enters, entry: authored::card_3916::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 2 }, rules: &authored::card_3916::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Sothera, the Supervoid", oracle_id: Some("760ede92-a826-4144-9de8-84ba9b8743d1"), mana_cost: "{2}{B}{B}", type_line: "Legendary Enchantment", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: false, keywords: authored::card_3971::RULES.keywords, deck_limit: authored::card_3971::RULES.deck_limit }, effect: authored::card_3971::RULES.effect, triggers: authored::card_3971::RULES.triggers, activated: authored::card_3971::RULES.activated, enters: authored::card_3971::RULES.enters, entry: authored::card_3971::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 0, 2, 0, 0, 0], generic: 2 }, rules: &authored::card_3971::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Space-Time Anomaly", oracle_id: Some("241117a0-8305-48bb-9cb9-9b01e86e08f7"), mana_cost: "{2}{W}{U}", type_line: "Sorcery", oracle_keywords: &["Mill"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3990::RULES.keywords, deck_limit: authored::card_3990::RULES.deck_limit }, effect: authored::card_3990::RULES.checked_effect(), triggers: authored::card_3990::RULES.triggers, activated: authored::card_3990::RULES.activated, enters: authored::card_3990::RULES.enters, entry: authored::card_3990::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [1, 1, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_3990::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Specimen Freighter", oracle_id: Some("fb84b94c-dc8c-4182-9b96-abd092b20981"), mana_cost: "{5}{U}", type_line: "Artifact — Spacecraft", oracle_keywords: &["Station", "Mill"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_3992::RULES.keywords, deck_limit: authored::card_3992::RULES.deck_limit }, effect: authored::card_3992::RULES.effect, triggers: authored::card_3992::RULES.triggers, activated: authored::card_3992::RULES.activated, enters: authored::card_3992::RULES.enters, entry: authored::card_3992::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 5 }, rules: &authored::card_3992::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Squire's Lightblade", oracle_id: Some("0ea2a099-7c81-4012-8148-4d377f2d64e1"), mana_cost: "{W}", type_line: "Artifact — Equipment", oracle_keywords: &["Equip", "Flash"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4080::RULES.keywords, deck_limit: authored::card_4080::RULES.deck_limit }, effect: authored::card_4080::RULES.checked_effect(), triggers: authored::card_4080::RULES.triggers, activated: authored::card_4080::RULES.activated, enters: authored::card_4080::RULES.enters, entry: authored::card_4080::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_4080::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Starbreach Whale", oracle_id: Some("076865e9-ee10-4a87-9180-4274202d37c1"), mana_cost: "{4}{U}", type_line: "Creature — Whale", oracle_keywords: &["Flying", "Surveil", "Warp"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4096::RULES.keywords, deck_limit: authored::card_4096::RULES.deck_limit }, effect: authored::card_4096::RULES.checked_effect(), triggers: authored::card_4096::RULES.triggers, activated: authored::card_4096::RULES.activated, enters: authored::card_4096::RULES.enters, entry: authored::card_4096::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 4 }, rules: &authored::card_4096::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Starfield Shepherd", oracle_id: Some("42fa99ab-d35f-4439-ab00-fe99bad626ac"), mana_cost: "{3}{W}{W}", type_line: "Creature — Angel", oracle_keywords: &["Flying", "Warp"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_4098::RULES.keywords, deck_limit: authored::card_4098::RULES.deck_limit }, effect: authored::card_4098::RULES.effect, triggers: authored::card_4098::RULES.triggers, activated: authored::card_4098::RULES.activated, enters: authored::card_4098::RULES.enters, entry: authored::card_4098::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [2, 0, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_4098::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Starfield Vocalist", oracle_id: Some("4add809e-9caa-45ae-bb0d-4760a360b8e2"), mana_cost: "{3}{U}", type_line: "Creature — Human Bard", oracle_keywords: &["Warp"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_4099::RULES.keywords, deck_limit: authored::card_4099::RULES.deck_limit }, effect: authored::card_4099::RULES.effect, triggers: authored::card_4099::RULES.triggers, activated: authored::card_4099::RULES.activated, enters: authored::card_4099::RULES.enters, entry: authored::card_4099::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_4099::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Starfighter Pilot", oracle_id: Some("fd6afb5e-d99a-44a7-9af4-9f064da95d63"), mana_cost: "{1}{W}", type_line: "Creature — Human Pilot", oracle_keywords: &["Surveil"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4100::RULES.keywords, deck_limit: authored::card_4100::RULES.deck_limit }, effect: authored::card_4100::RULES.checked_effect(), triggers: authored::card_4100::RULES.triggers, activated: authored::card_4100::RULES.activated, enters: authored::card_4100::RULES.enters, entry: authored::card_4100::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_4100::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Starport Security", oracle_id: Some("5454dddf-c6e5-4b0d-b96d-4904c13606dc"), mana_cost: "{W}", type_line: "Artifact Creature — Robot Soldier", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_4108::RULES.keywords, deck_limit: authored::card_4108::RULES.deck_limit }, effect: authored::card_4108::RULES.effect, triggers: authored::card_4108::RULES.triggers, activated: authored::card_4108::RULES.activated, enters: authored::card_4108::RULES.enters, entry: authored::card_4108::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_4108::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Starwinder", oracle_id: Some("face2dc0-37ba-4efe-b900-b5521567c943"), mana_cost: "{5}{U}{U}", type_line: "Creature — Leviathan", oracle_keywords: &["Warp"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_4116::RULES.keywords, deck_limit: authored::card_4116::RULES.deck_limit }, effect: authored::card_4116::RULES.effect, triggers: authored::card_4116::RULES.triggers, activated: authored::card_4116::RULES.activated, enters: authored::card_4116::RULES.enters, entry: authored::card_4116::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 2, 0, 0, 0, 0], generic: 5 }, rules: &authored::card_4116::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Station Monitor", oracle_id: Some("94e8791b-e72f-4aed-b761-826779337168"), mana_cost: "{W}{U}", type_line: "Creature — Lizard Artificer", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_4119::RULES.keywords, deck_limit: authored::card_4119::RULES.deck_limit }, effect: authored::card_4119::RULES.effect, triggers: authored::card_4119::RULES.triggers, activated: authored::card_4119::RULES.activated, enters: authored::card_4119::RULES.enters, entry: authored::card_4119::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 1, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_4119::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Steelswarm Operator", oracle_id: Some("770b2097-5178-42d2-be6c-393b4bfe3727"), mana_cost: "{1}{U}", type_line: "Artifact Creature — Robot Soldier", oracle_keywords: &["Flying"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_4129::RULES.keywords, deck_limit: authored::card_4129::RULES.deck_limit }, effect: authored::card_4129::RULES.effect, triggers: authored::card_4129::RULES.triggers, activated: authored::card_4129::RULES.activated, enters: authored::card_4129::RULES.enters, entry: authored::card_4129::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_4129::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Stomping Ground", oracle_id: Some("16052b52-ade1-406f-a06b-ce7ea607fb63"), mana_cost: "", type_line: "Land — Mountain Forest", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4161::RULES.keywords, deck_limit: authored::card_4161::RULES.deck_limit }, effect: authored::card_4161::RULES.checked_effect(), triggers: authored::card_4161::RULES.triggers, activated: authored::card_4161::RULES.activated, enters: authored::card_4161::RULES.enters, entry: authored::card_4161::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_4161::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Sunset Saboteur", oracle_id: Some("c3731735-2b78-4a15-bff4-4d8559401f68"), mana_cost: "{1}{B}", type_line: "Creature — Human Rogue", oracle_keywords: &["Menace", "Ward"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4244::RULES.keywords, deck_limit: authored::card_4244::RULES.deck_limit }, effect: authored::card_4244::RULES.checked_effect(), triggers: authored::card_4244::RULES.triggers, activated: authored::card_4244::RULES.activated, enters: authored::card_4244::RULES.enters, entry: authored::card_4244::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 1 }, rules: &authored::card_4244::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Sunstar Chaplain", oracle_id: Some("de6b26e4-8700-490f-9df2-dac430463274"), mana_cost: "{1}{W}", type_line: "Creature — Human Cleric", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_4249::RULES.keywords, deck_limit: authored::card_4249::RULES.deck_limit }, effect: authored::card_4249::RULES.effect, triggers: authored::card_4249::RULES.triggers, activated: authored::card_4249::RULES.activated, enters: authored::card_4249::RULES.enters, entry: authored::card_4249::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_4249::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Sunstar Expansionist", oracle_id: Some("944a91ab-c217-4c1d-b920-2e5503c34546"), mana_cost: "{1}{W}", type_line: "Creature — Human Knight", oracle_keywords: &["Landfall"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_4250::RULES.keywords, deck_limit: authored::card_4250::RULES.deck_limit }, effect: authored::card_4250::RULES.effect, triggers: authored::card_4250::RULES.triggers, activated: authored::card_4250::RULES.activated, enters: authored::card_4250::RULES.enters, entry: authored::card_4250::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_4250::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Sunstar Lightsmith", oracle_id: Some("132de6c2-4ca2-41da-9c78-caa59838b641"), mana_cost: "{3}{W}", type_line: "Creature — Human Artificer", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_4251::RULES.keywords, deck_limit: authored::card_4251::RULES.deck_limit }, effect: authored::card_4251::RULES.effect, triggers: authored::card_4251::RULES.triggers, activated: authored::card_4251::RULES.activated, enters: authored::card_4251::RULES.enters, entry: authored::card_4251::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_4251::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Survey Mechan", oracle_id: Some("53745903-de8f-42cf-a949-842e5831de30"), mana_cost: "{4}", type_line: "Artifact Creature — Robot", oracle_keywords: &["Flying", "Hexproof"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_4276::RULES.keywords, deck_limit: authored::card_4276::RULES.deck_limit }, effect: authored::card_4276::RULES.effect, triggers: authored::card_4276::RULES.triggers, activated: authored::card_4276::RULES.activated, enters: authored::card_4276::RULES.enters, entry: authored::card_4276::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 4 }, rules: &authored::card_4276::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Susur Secundi, Void Altar", oracle_id: Some("50d6cadc-07e4-479e-90f4-e3a20f769bab"), mana_cost: "", type_line: "Land — Planet", oracle_keywords: &["Station"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_4280::RULES.keywords, deck_limit: authored::card_4280::RULES.deck_limit }, effect: authored::card_4280::RULES.effect, triggers: authored::card_4280::RULES.triggers, activated: authored::card_4280::RULES.activated, enters: authored::card_4280::RULES.enters, entry: authored::card_4280::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_4280::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Susurian Dirgecraft", oracle_id: Some("f1d826d1-d2da-46ce-b1db-81393c88d865"), mana_cost: "{4}{B}", type_line: "Artifact — Spacecraft", oracle_keywords: &["Station"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_4281::RULES.keywords, deck_limit: authored::card_4281::RULES.deck_limit }, effect: authored::card_4281::RULES.effect, triggers: authored::card_4281::RULES.triggers, activated: authored::card_4281::RULES.activated, enters: authored::card_4281::RULES.enters, entry: authored::card_4281::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 4 }, rules: &authored::card_4281::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Susurian Voidborn", oracle_id: Some("4f754f11-3b6d-4a4e-86d7-7495e475ef74"), mana_cost: "{2}{B}", type_line: "Creature — Vampire Soldier", oracle_keywords: &["Warp"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_4282::RULES.keywords, deck_limit: authored::card_4282::RULES.deck_limit }, effect: authored::card_4282::RULES.effect, triggers: authored::card_4282::RULES.triggers, activated: authored::card_4282::RULES.activated, enters: authored::card_4282::RULES.enters, entry: authored::card_4282::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 2 }, rules: &authored::card_4282::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Swarm Culler", oracle_id: Some("a6c28bde-10d9-471b-acd0-32a1c6195805"), mana_cost: "{3}{B}", type_line: "Creature — Insect Warrior", oracle_keywords: &["Flying"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4286::RULES.keywords, deck_limit: authored::card_4286::RULES.deck_limit }, effect: authored::card_4286::RULES.checked_effect(), triggers: authored::card_4286::RULES.triggers, activated: authored::card_4286::RULES.activated, enters: authored::card_4286::RULES.enters, entry: authored::card_4286::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 3 }, rules: &authored::card_4286::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Synthesizer Labship", oracle_id: Some("a37314cf-bbbd-4003-b94d-0e6e60edfc15"), mana_cost: "{U}", type_line: "Artifact — Spacecraft", oracle_keywords: &["Station"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_4305::RULES.keywords, deck_limit: authored::card_4305::RULES.deck_limit }, effect: authored::card_4305::RULES.effect, triggers: authored::card_4305::RULES.triggers, activated: authored::card_4305::RULES.activated, enters: authored::card_4305::RULES.enters, entry: authored::card_4305::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_4305::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Syr Vondam, Sunstar Exemplar", oracle_id: Some("9e597063-c077-4236-a9e4-b0721b24677f"), mana_cost: "{W}{B}", type_line: "Legendary Creature — Human Knight", oracle_keywords: &["Vigilance", "Menace"], standard_legal: true, legendary: true, implemented: false, keywords: authored::card_4310::RULES.keywords, deck_limit: authored::card_4310::RULES.deck_limit }, effect: authored::card_4310::RULES.effect, triggers: authored::card_4310::RULES.triggers, activated: authored::card_4310::RULES.activated, enters: authored::card_4310::RULES.enters, entry: authored::card_4310::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 1, 0, 0, 0], generic: 0 }, rules: &authored::card_4310::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Syr Vondam, the Lucent", oracle_id: Some("28225bae-2d96-4352-b256-0c2ff4ef65e9"), mana_cost: "{2}{W}{B}{B}", type_line: "Legendary Creature — Human Knight", oracle_keywords: &["Lifelink", "Deathtouch"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_4311::RULES.keywords, deck_limit: authored::card_4311::RULES.deck_limit }, effect: authored::card_4311::RULES.checked_effect(), triggers: authored::card_4311::RULES.triggers, activated: authored::card_4311::RULES.activated, enters: authored::card_4311::RULES.enters, entry: authored::card_4311::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 2, 0, 0, 0], generic: 2 }, rules: &authored::card_4311::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Systems Override", oracle_id: Some("4c9d9d47-1eb6-4981-ba83-d5ec04906e71"), mana_cost: "{2}{R}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_4312::RULES.keywords, deck_limit: authored::card_4312::RULES.deck_limit }, effect: authored::card_4312::RULES.effect, triggers: authored::card_4312::RULES.triggers, activated: authored::card_4312::RULES.activated, enters: authored::card_4312::RULES.enters, entry: authored::card_4312::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 2 }, rules: &authored::card_4312::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Tannuk, Memorial Ensign", oracle_id: Some("76cb1912-3791-44b8-b594-b92dab21bafc"), mana_cost: "{1}{R}{G}", type_line: "Legendary Creature — Kavu Pilot", oracle_keywords: &["Landfall"], standard_legal: true, legendary: true, implemented: false, keywords: authored::card_4334::RULES.keywords, deck_limit: authored::card_4334::RULES.deck_limit }, effect: authored::card_4334::RULES.effect, triggers: authored::card_4334::RULES.triggers, activated: authored::card_4334::RULES.activated, enters: authored::card_4334::RULES.enters, entry: authored::card_4334::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 1, 0], generic: 1 }, rules: &authored::card_4334::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Tannuk, Steadfast Second", oracle_id: Some("b5ae8016-92ed-4ae8-83a8-65068defc616"), mana_cost: "{2}{R}{R}", type_line: "Legendary Creature — Kavu Pilot", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: false, keywords: authored::card_4335::RULES.keywords, deck_limit: authored::card_4335::RULES.deck_limit }, effect: authored::card_4335::RULES.effect, triggers: authored::card_4335::RULES.triggers, activated: authored::card_4335::RULES.activated, enters: authored::card_4335::RULES.enters, entry: authored::card_4335::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 2, 0, 0], generic: 2 }, rules: &authored::card_4335::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Tapestry Warden", oracle_id: Some("981138d0-bf61-406a-85be-68cc5e58796c"), mana_cost: "{3}{G}", type_line: "Artifact Creature — Robot Soldier", oracle_keywords: &["Vigilance"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_4337::RULES.keywords, deck_limit: authored::card_4337::RULES.deck_limit }, effect: authored::card_4337::RULES.effect, triggers: authored::card_4337::RULES.triggers, activated: authored::card_4337::RULES.activated, enters: authored::card_4337::RULES.enters, entry: authored::card_4337::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 3 }, rules: &authored::card_4337::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Temporal Intervention", oracle_id: Some("5e8a2416-e311-41ac-a3cd-be1c2441d542"), mana_cost: "{2}{B}", type_line: "Sorcery", oracle_keywords: &["Void"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_4373::RULES.keywords, deck_limit: authored::card_4373::RULES.deck_limit }, effect: authored::card_4373::RULES.effect, triggers: authored::card_4373::RULES.triggers, activated: authored::card_4373::RULES.activated, enters: authored::card_4373::RULES.enters, entry: authored::card_4373::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 2 }, rules: &authored::card_4373::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Terminal Velocity", oracle_id: Some("cdda2f04-d59e-4ef8-a26d-964d80750940"), mana_cost: "{4}{R}{R}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_4388::RULES.keywords, deck_limit: authored::card_4388::RULES.deck_limit }, effect: authored::card_4388::RULES.effect, triggers: authored::card_4388::RULES.triggers, activated: authored::card_4388::RULES.activated, enters: authored::card_4388::RULES.enters, entry: authored::card_4388::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 0, 2, 0, 0], generic: 4 }, rules: &authored::card_4388::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Terrapact Intimidator", oracle_id: Some("c08e0958-8fa2-49c7-ab17-a70dfc1ef33a"), mana_cost: "{1}{R}", type_line: "Creature — Kavu Scout", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_4391::RULES.keywords, deck_limit: authored::card_4391::RULES.deck_limit }, effect: authored::card_4391::RULES.effect, triggers: authored::card_4391::RULES.triggers, activated: authored::card_4391::RULES.activated, enters: authored::card_4391::RULES.enters, entry: authored::card_4391::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 1 }, rules: &authored::card_4391::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Terrasymbiosis", oracle_id: Some("4ac94586-3ac7-4ade-af31-ad0d0e3ffc4a"), mana_cost: "{2}{G}", type_line: "Enchantment", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_4392::RULES.keywords, deck_limit: authored::card_4392::RULES.deck_limit }, effect: authored::card_4392::RULES.effect, triggers: authored::card_4392::RULES.triggers, activated: authored::card_4392::RULES.activated, enters: authored::card_4392::RULES.enters, entry: authored::card_4392::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 2 }, rules: &authored::card_4392::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Territorial Bruntar", oracle_id: Some("03f04c2c-1036-497e-8b13-06c61b0b0ba6"), mana_cost: "{4}{R}{R}", type_line: "Creature — Beast", oracle_keywords: &["Reach", "Landfall"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_4395::RULES.keywords, deck_limit: authored::card_4395::RULES.deck_limit }, effect: authored::card_4395::RULES.effect, triggers: authored::card_4395::RULES.triggers, activated: authored::card_4395::RULES.activated, enters: authored::card_4395::RULES.enters, entry: authored::card_4395::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 2, 0, 0], generic: 4 }, rules: &authored::card_4395::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Tezzeret, Cruel Captain", oracle_id: Some("0eb11b2d-a397-48da-a0f3-9f0f83f42282"), mana_cost: "{3}", type_line: "Legendary Planeswalker — Tezzeret", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_4412::RULES.keywords, deck_limit: authored::card_4412::RULES.deck_limit }, effect: authored::card_4412::RULES.checked_effect(), triggers: authored::card_4412::RULES.triggers, activated: authored::card_4412::RULES.activated, enters: authored::card_4412::RULES.enters, entry: authored::card_4412::RULES.entry, kind: 64, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_4412::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Thaumaton Torpedo", oracle_id: Some("9f9ae015-dc59-4ecb-a2bb-668496b48a48"), mana_cost: "{1}", type_line: "Artifact", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_4415::RULES.keywords, deck_limit: authored::card_4415::RULES.deck_limit }, effect: authored::card_4415::RULES.effect, triggers: authored::card_4415::RULES.triggers, activated: authored::card_4415::RULES.activated, enters: authored::card_4415::RULES.enters, entry: authored::card_4415::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_4415::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Thawbringer", oracle_id: Some("0863c4bc-1e3e-4ff9-8877-88d4021c85e5"), mana_cost: "{2}{G}", type_line: "Creature — Insect Scout", oracle_keywords: &["Surveil"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4416::RULES.keywords, deck_limit: authored::card_4416::RULES.deck_limit }, effect: authored::card_4416::RULES.checked_effect(), triggers: authored::card_4416::RULES.triggers, activated: authored::card_4416::RULES.activated, enters: authored::card_4416::RULES.enters, entry: authored::card_4416::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 2 }, rules: &authored::card_4416::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "The Dominion Bracelet", oracle_id: Some("c017f8ae-9103-4965-a07a-f259f44a157d"), mana_cost: "{2}", type_line: "Legendary Artifact — Equipment", oracle_keywords: &["Equip"], standard_legal: true, legendary: true, implemented: false, keywords: authored::card_4435::RULES.keywords, deck_limit: authored::card_4435::RULES.deck_limit }, effect: authored::card_4435::RULES.effect, triggers: authored::card_4435::RULES.triggers, activated: authored::card_4435::RULES.activated, enters: authored::card_4435::RULES.enters, entry: authored::card_4435::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_4435::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "The Endstone", oracle_id: Some("57ee0730-3ac5-4f17-ac82-bd4a6673c2bb"), mana_cost: "{7}", type_line: "Legendary Artifact", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: false, keywords: authored::card_4442::RULES.keywords, deck_limit: authored::card_4442::RULES.deck_limit }, effect: authored::card_4442::RULES.effect, triggers: authored::card_4442::RULES.triggers, activated: authored::card_4442::RULES.activated, enters: authored::card_4442::RULES.enters, entry: authored::card_4442::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 7 }, rules: &authored::card_4442::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "The Eternity Elevator", oracle_id: Some("11323af4-b8b8-4ca9-932f-377c7fd77dea"), mana_cost: "{5}", type_line: "Legendary Artifact — Spacecraft", oracle_keywords: &["Station"], standard_legal: true, legendary: true, implemented: false, keywords: authored::card_4444::RULES.keywords, deck_limit: authored::card_4444::RULES.deck_limit }, effect: authored::card_4444::RULES.effect, triggers: authored::card_4444::RULES.triggers, activated: authored::card_4444::RULES.activated, enters: authored::card_4444::RULES.enters, entry: authored::card_4444::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 5 }, rules: &authored::card_4444::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "The Seriema", oracle_id: Some("a6bcec1f-f515-4e63-9e84-8eb04cc582ff"), mana_cost: "{1}{W}{W}", type_line: "Legendary Artifact — Spacecraft", oracle_keywords: &["Station"], standard_legal: true, legendary: true, implemented: false, keywords: authored::card_4496::RULES.keywords, deck_limit: authored::card_4496::RULES.deck_limit }, effect: authored::card_4496::RULES.effect, triggers: authored::card_4496::RULES.triggers, activated: authored::card_4496::RULES.activated, enters: authored::card_4496::RULES.enters, entry: authored::card_4496::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [2, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_4496::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Thrumming Hivepool", oracle_id: Some("04ea21b3-084b-4746-9b49-99b6ef4f0885"), mana_cost: "{6}", type_line: "Artifact", oracle_keywords: &["Affinity"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4576::RULES.keywords, deck_limit: authored::card_4576::RULES.deck_limit }, effect: authored::card_4576::RULES.checked_effect(), triggers: authored::card_4576::RULES.triggers, activated: authored::card_4576::RULES.activated, enters: authored::card_4576::RULES.enters, entry: authored::card_4576::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 6 }, rules: &authored::card_4576::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Timeline Culler", oracle_id: Some("3c307122-bc18-41f1-87d5-ae6b3ca9b8b9"), mana_cost: "{B}{B}", type_line: "Creature — Drix Warlock", oracle_keywords: &["Haste", "Warp"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_4602::RULES.keywords, deck_limit: authored::card_4602::RULES.deck_limit }, effect: authored::card_4602::RULES.effect, triggers: authored::card_4602::RULES.triggers, activated: authored::card_4602::RULES.activated, enters: authored::card_4602::RULES.enters, entry: authored::card_4602::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 2, 0, 0, 0], generic: 0 }, rules: &authored::card_4602::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Tractor Beam", oracle_id: Some("ed99a0b8-d82d-4c80-af97-cabd333620df"), mana_cost: "{2}{U}{U}", type_line: "Enchantment — Aura", oracle_keywords: &["Enchant"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_4646::RULES.keywords, deck_limit: authored::card_4646::RULES.deck_limit }, effect: authored::card_4646::RULES.effect, triggers: authored::card_4646::RULES.triggers, activated: authored::card_4646::RULES.activated, enters: authored::card_4646::RULES.enters, entry: authored::card_4646::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 2, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_4646::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Tragic Trajectory", oracle_id: Some("e5e19326-3e90-439c-a3b2-749d43a1af0c"), mana_cost: "{B}", type_line: "Sorcery", oracle_keywords: &["Void"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_4651::RULES.keywords, deck_limit: authored::card_4651::RULES.deck_limit }, effect: authored::card_4651::RULES.effect, triggers: authored::card_4651::RULES.triggers, activated: authored::card_4651::RULES.activated, enters: authored::card_4651::RULES.enters, entry: authored::card_4651::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 0 }, rules: &authored::card_4651::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Unravel", oracle_id: Some("1525cbc1-0d95-4f58-99fb-136e54c4d486"), mana_cost: "{1}{U}{U}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4792::RULES.keywords, deck_limit: authored::card_4792::RULES.deck_limit }, effect: authored::card_4792::RULES.checked_effect(), triggers: authored::card_4792::RULES.triggers, activated: authored::card_4792::RULES.activated, enters: authored::card_4792::RULES.enters, entry: authored::card_4792::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 2, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_4792::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Uthros Psionicist", oracle_id: Some("7f51aede-fb86-4b7e-9027-2ba8be62b567"), mana_cost: "{2}{U}", type_line: "Creature — Jellyfish Scientist", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4817::RULES.keywords, deck_limit: authored::card_4817::RULES.deck_limit }, effect: authored::card_4817::RULES.checked_effect(), triggers: authored::card_4817::RULES.triggers, activated: authored::card_4817::RULES.activated, enters: authored::card_4817::RULES.enters, entry: authored::card_4817::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_4817::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Uthros Scanship", oracle_id: Some("7b4c37dc-8cb0-4870-929e-11c2a45952a2"), mana_cost: "{3}{U}", type_line: "Artifact — Spacecraft", oracle_keywords: &["Station"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_4818::RULES.keywords, deck_limit: authored::card_4818::RULES.deck_limit }, effect: authored::card_4818::RULES.effect, triggers: authored::card_4818::RULES.triggers, activated: authored::card_4818::RULES.activated, enters: authored::card_4818::RULES.enters, entry: authored::card_4818::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_4818::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Uthros, Titanic Godcore", oracle_id: Some("df08ac72-010f-42f8-beb3-6d645c638e1e"), mana_cost: "", type_line: "Land — Planet", oracle_keywords: &["Station"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_4819::RULES.keywords, deck_limit: authored::card_4819::RULES.deck_limit }, effect: authored::card_4819::RULES.effect, triggers: authored::card_4819::RULES.triggers, activated: authored::card_4819::RULES.activated, enters: authored::card_4819::RULES.enters, entry: authored::card_4819::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_4819::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Vaultguard Trooper", oracle_id: Some("e15c78d4-6fb1-4bd3-8219-c66b83fcaba1"), mana_cost: "{4}{R}", type_line: "Creature — Kavu Soldier", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_4854::RULES.keywords, deck_limit: authored::card_4854::RULES.deck_limit }, effect: authored::card_4854::RULES.effect, triggers: authored::card_4854::RULES.triggers, activated: authored::card_4854::RULES.activated, enters: authored::card_4854::RULES.enters, entry: authored::card_4854::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 4 }, rules: &authored::card_4854::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Virulent Silencer", oracle_id: Some("7d70a2f4-92cb-40c3-9e5a-f6c57d4a64c2"), mana_cost: "{3}", type_line: "Artifact Creature — Robot Assassin", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4911::RULES.keywords, deck_limit: authored::card_4911::RULES.deck_limit }, effect: authored::card_4911::RULES.checked_effect(), triggers: authored::card_4911::RULES.triggers, activated: authored::card_4911::RULES.activated, enters: authored::card_4911::RULES.enters, entry: authored::card_4911::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_4911::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Virus Beetle", oracle_id: Some("e1e489f6-37e5-469e-bca3-0e9d3fc3fa95"), mana_cost: "{1}{B}", type_line: "Artifact Creature — Insect", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4912::RULES.keywords, deck_limit: authored::card_4912::RULES.deck_limit }, effect: authored::card_4912::RULES.checked_effect(), triggers: authored::card_4912::RULES.triggers, activated: authored::card_4912::RULES.activated, enters: authored::card_4912::RULES.enters, entry: authored::card_4912::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 1 }, rules: &authored::card_4912::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Voidforged Titan", oracle_id: Some("f7afe534-fbb0-4f8b-9e98-aa49d4005465"), mana_cost: "{4}{B}", type_line: "Artifact Creature — Robot Warrior", oracle_keywords: &["Void"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_4929::RULES.keywords, deck_limit: authored::card_4929::RULES.deck_limit }, effect: authored::card_4929::RULES.effect, triggers: authored::card_4929::RULES.triggers, activated: authored::card_4929::RULES.activated, enters: authored::card_4929::RULES.enters, entry: authored::card_4929::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 4 }, rules: &authored::card_4929::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Vote Out", oracle_id: Some("7efe9fe5-d93c-46a6-afbd-9655a902c6bc"), mana_cost: "{3}{B}", type_line: "Sorcery", oracle_keywords: &["Convoke"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4939::RULES.keywords, deck_limit: authored::card_4939::RULES.deck_limit }, effect: authored::card_4939::RULES.checked_effect(), triggers: authored::card_4939::RULES.triggers, activated: authored::card_4939::RULES.activated, enters: authored::card_4939::RULES.enters, entry: authored::card_4939::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 3 }, rules: &authored::card_4939::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Warmaker Gunship", oracle_id: Some("c947171b-ed9e-4b83-af45-bd595a8d84ee"), mana_cost: "{2}{R}", type_line: "Artifact — Spacecraft", oracle_keywords: &["Station"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4981::RULES.keywords, deck_limit: authored::card_4981::RULES.deck_limit }, effect: authored::card_4981::RULES.checked_effect(), triggers: authored::card_4981::RULES.triggers, activated: authored::card_4981::RULES.activated, enters: authored::card_4981::RULES.enters, entry: authored::card_4981::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 2 }, rules: &authored::card_4981::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Watery Grave", oracle_id: Some("fc9ec820-4245-4a96-b009-5308a818ca58"), mana_cost: "", type_line: "Land — Island Swamp", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_5004::RULES.keywords, deck_limit: authored::card_5004::RULES.deck_limit }, effect: authored::card_5004::RULES.checked_effect(), triggers: authored::card_5004::RULES.triggers, activated: authored::card_5004::RULES.activated, enters: authored::card_5004::RULES.enters, entry: authored::card_5004::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_5004::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Weapons Manufacturing", oracle_id: Some("e48a8160-cffe-4e00-a3e9-a59d7fc7b3a2"), mana_cost: "{1}{R}", type_line: "Enchantment", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_5020::RULES.keywords, deck_limit: authored::card_5020::RULES.deck_limit }, effect: authored::card_5020::RULES.effect, triggers: authored::card_5020::RULES.triggers, activated: authored::card_5020::RULES.activated, enters: authored::card_5020::RULES.enters, entry: authored::card_5020::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 1 }, rules: &authored::card_5020::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Wedgelight Rammer", oracle_id: Some("03259ab0-caa8-4620-b070-127e2c712252"), mana_cost: "{3}{W}", type_line: "Artifact — Spacecraft", oracle_keywords: &["Station"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_5029::RULES.keywords, deck_limit: authored::card_5029::RULES.deck_limit }, effect: authored::card_5029::RULES.effect, triggers: authored::card_5029::RULES.triggers, activated: authored::card_5029::RULES.activated, enters: authored::card_5029::RULES.enters, entry: authored::card_5029::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_5029::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Weftblade Enhancer", oracle_id: Some("4d8ee903-17b3-43c9-9637-ee596955a9f4"), mana_cost: "{5}{W}", type_line: "Creature — Drix Artificer", oracle_keywords: &["Warp"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_5030::RULES.keywords, deck_limit: authored::card_5030::RULES.deck_limit }, effect: authored::card_5030::RULES.checked_effect(), triggers: authored::card_5030::RULES.triggers, activated: authored::card_5030::RULES.activated, enters: authored::card_5030::RULES.enters, entry: authored::card_5030::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 5 }, rules: &authored::card_5030::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Weftstalker Ardent", oracle_id: Some("926d52a5-4db1-46ce-9567-17c28bf56ae7"), mana_cost: "{2}{R}", type_line: "Creature — Drix Artificer", oracle_keywords: &["Warp"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_5031::RULES.keywords, deck_limit: authored::card_5031::RULES.deck_limit }, effect: authored::card_5031::RULES.effect, triggers: authored::card_5031::RULES.triggers, activated: authored::card_5031::RULES.activated, enters: authored::card_5031::RULES.enters, entry: authored::card_5031::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 2 }, rules: &authored::card_5031::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Weftwalking", oracle_id: Some("08af6751-5848-47fb-95f5-57215e899460"), mana_cost: "{4}{U}{U}", type_line: "Enchantment", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_5032::RULES.keywords, deck_limit: authored::card_5032::RULES.deck_limit }, effect: authored::card_5032::RULES.effect, triggers: authored::card_5032::RULES.triggers, activated: authored::card_5032::RULES.activated, enters: authored::card_5032::RULES.enters, entry: authored::card_5032::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 2, 0, 0, 0, 0], generic: 4 }, rules: &authored::card_5032::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Wurmwall Sweeper", oracle_id: Some("320ed2ec-ef06-41fb-ac8a-72579f11544c"), mana_cost: "{2}", type_line: "Artifact — Spacecraft", oracle_keywords: &["Surveil", "Station"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_5126::RULES.keywords, deck_limit: authored::card_5126::RULES.deck_limit }, effect: authored::card_5126::RULES.effect, triggers: authored::card_5126::RULES.triggers, activated: authored::card_5126::RULES.activated, enters: authored::card_5126::RULES.enters, entry: authored::card_5126::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_5126::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Xu-Ifit, Osteoharmonist", oracle_id: Some("39e32fdf-93bf-4390-8275-0d7d10ad7360"), mana_cost: "{1}{B}{B}", type_line: "Legendary Creature — Human Wizard", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: false, keywords: authored::card_5129::RULES.keywords, deck_limit: authored::card_5129::RULES.deck_limit }, effect: authored::card_5129::RULES.effect, triggers: authored::card_5129::RULES.triggers, activated: authored::card_5129::RULES.activated, enters: authored::card_5129::RULES.enters, entry: authored::card_5129::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 2, 0, 0, 0], generic: 1 }, rules: &authored::card_5129::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Zealous Display", oracle_id: Some("0ff3afba-7a90-4f62-a1b0-b553ad81c32b"), mana_cost: "{2}{W}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_5154::RULES.keywords, deck_limit: authored::card_5154::RULES.deck_limit }, effect: authored::card_5154::RULES.effect, triggers: authored::card_5154::RULES.triggers, activated: authored::card_5154::RULES.activated, enters: authored::card_5154::RULES.enters, entry: authored::card_5154::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_5154::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Zero Point Ballad", oracle_id: Some("835abd35-27ea-45fe-b55d-25081c57bd4a"), mana_cost: "{X}{B}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_5158::RULES.keywords, deck_limit: authored::card_5158::RULES.deck_limit }, effect: authored::card_5158::RULES.effect, triggers: authored::card_5158::RULES.triggers, activated: authored::card_5158::RULES.activated, enters: authored::card_5158::RULES.enters, entry: authored::card_5158::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_5158::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Zookeeper Mechan", oracle_id: Some("6d4d98a8-a5a3-4cc1-9277-c45dafe40ea0"), mana_cost: "{1}{R}", type_line: "Artifact Creature — Robot", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_5171::RULES.keywords, deck_limit: authored::card_5171::RULES.deck_limit }, effect: authored::card_5171::RULES.checked_effect(), triggers: authored::card_5171::RULES.triggers, activated: authored::card_5171::RULES.activated, enters: authored::card_5171::RULES.enters, entry: authored::card_5171::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 1 }, rules: &authored::card_5171::RULES },
];
}
pub use cards::ENTRIES;
