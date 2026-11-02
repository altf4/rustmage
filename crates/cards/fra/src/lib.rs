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
pub(super) mod card_0000 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/plains.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0001 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/island.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0002 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/swamp.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0003 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/mountain.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0004 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/forest.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0046 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/academic_ascent.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0069 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/aerid_konstrari.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0079 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/afterthought_sentry.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0111 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/ajani_resolute.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0112 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/ajani_unrelenting.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0113 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/ajani_s_anguish.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0183 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/apex_witchstalker.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0203 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/arcane_amphisbaena.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0211 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/archive_arbiter.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0229 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/arni_humble_scribe.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0230 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/arni_renowned_champion.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0233 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/artifist_acumen.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0278 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/avatar_of_burgeoning_echoes.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0289 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/awaken_the_inferno.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0396 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/bestial_incursion.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0442 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/blazing_crescendo.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0447 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/blessed_ghoul.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0459 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/bloodline_recollector_ancestral_craving.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0464 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/bloombrute.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0469 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/blossom_blessed_angel_seed_suture.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0539 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/break_under_pressure.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0576 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/budding_insurgent.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0628 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/campus_crier.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0656 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/carnivorous_cultivator_enroot.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0675 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/cast_away_doubt.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0713 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/chandra_s_emberling.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0714 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/chandra_chill_of_compliance.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0717 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/chandra_torch_of_defiance.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0721 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/charge_the_sanctum.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0764 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/clash_of_elements.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0788 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/codie_ravenous_codex.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0812 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/command_the_stage.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0820 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/compel_brutality.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0869 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/countersculpt.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0877 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/craftwork_crusher.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0881 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/craterclaw_colossus.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0902 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/cruel_calculations.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0911 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/cryotheory_adept.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0935 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/curse_marred_demon.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0955 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/danitha_spear_of_agony.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0956 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/danitha_sword_of_hope.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0969 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/dark_matter_manipulator.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0970 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/darklight_phoenix.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1017 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/dedicated_commons.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1043 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/denzilore_fatehold.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1053 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/deserted_beach.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1058 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/desperate_futurescribe.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1110 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/diviner_of_victory_unwind_history.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1111 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/divining_duelist.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1148 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/draconic_visitor.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1230 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/eardrum_rattler.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1262 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/edgar_ancient_bloodlord.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1264 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/edgar_moonlit_sovereign.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1312 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/emergency_phytomedic_seed_suture.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1324 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/emrakul_the_exigent_doom.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1341 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/enlightened_confidant.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1349 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/entrust_the_spark.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1371 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/essence_burn.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1412 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/extended_absence.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1416 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/extrapolate_the_impossible.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1418 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/eye_of_jace.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1423 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/face_yourself.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1452 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/fatehold_annex.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1453 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/fatehold_charm.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1454 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/fatehold_chronologist_peer_review.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1455 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/fateshaper_aspirant.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1457 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/fblthp_impossibly_lost.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1458 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/fblthp_knows_the_way.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1498 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/ferocity_of_the_hunt.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1558 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/flickering_hound.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1568 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/flourishing_grapple.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1598 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/formidable_commons.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1637 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/frostbite_pyromental.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1647 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/fulminous_forte.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1670 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/gallia_tragic_host.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1671 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/gallia_the_merrymaker.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1680 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/gardenize.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1687 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/garruk_curse_breaker.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1688 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/garruk_veiled_butcher.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1703 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/geist_of_saint_thalia.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1708 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/generous_revival.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1715 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/germinate_recruits.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1726 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/ghalta_the_immovable.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1727 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/ghalta_the_unstoppable.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1743 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/gideon_the_oathless.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1744 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/gideon_s_memorial.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1840 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/graft_surgeon.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1874 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/greenhouse_propagator.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1882 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/grim_repriser.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1907 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/guiding_hydra.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1930 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/hall_of_echoes.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1932 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/hallway_heckler_vicious_verse.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1937 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/hapatra_the_desert_fang.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1938 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/hapatra_the_desert_frost.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1963 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/haunted_ridge.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1983 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/heartstring_puller.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1984 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/heartwood_crafter_soul_tether.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2021 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/hexhaven_battalion.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2022 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/hexhaven_dueling_arena.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2023 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/hexhaven_invigorator.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2100 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/hungering_puppetbeast.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2104 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/hunter_s_axe.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2128 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/icy_reception.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2129 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/identity_echo.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2162 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/infinite_coursework.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2168 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/ingris_stingerquill.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2175 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/innovative_commons.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2186 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/inspired_tethermage.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2248 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/jace_s_machinations.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2249 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/jace_reality_sculptor.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2277 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/jiang_yanggu_alone.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2278 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/jiang_yanggu_never_alone.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2319 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/karn_argent_defender.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2320 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/karn_gilded_guardian.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2335 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/keeper_of_the_quiet_hour.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2360 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/kindred_judgment.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2366 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/kiora_of_fire_and_ashes.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2367 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/kiora_of_salt_and_sand.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2395 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/konstrari_annex.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2396 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/konstrari_charm.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2397 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/konstrari_improviser_soul_tether.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2399 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/koth_of_the_homestead.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2400 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/koth_the_geomancer.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2425 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/kwia_vigorbloom.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2449 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/last_gasp.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2488 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/lich_s_relic.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2506 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/liliana_the_faultless.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2507 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/liliana_the_repentant.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2525 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/living_library.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2550 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/loot_the_anomaly.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2552 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/loot_the_nexus.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2573 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/loyal_tutor.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2592 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/lyra_archangel_of_dawn.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2593 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/lyra_tolarian_archangel.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2599 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/mabel_bitter_recluse.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2601 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/mabel_valley_hero.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2672 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/marwyn_the_clearcutter.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2673 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/marwyn_the_preserver.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2677 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/massacre_girl_most_wanted.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2681 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/master_of_barbs.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2703 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/medic_s_kitesail.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2716 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/memory_trap.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2735 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/meticulous_commons.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2761 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/mind_meanderer.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2768 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/mindseeker_oculus.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2875 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/multiply_by_zero.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2882 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/murmuring_volume.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2951 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/no_admittance.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2976 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/null_summoner.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3019 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/omnipresence.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3066 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/overgrown_farmland.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3078 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/overwrite_the_multiverse.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3098 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/paradox_shaper_omit_variables.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3127 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/perfected_theory.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3152 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/pia_aether_ascetic.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3153 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/pia_determined_rebuilder.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3178 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/plan_for_all_outcomes.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3201 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/pompous_battlemage_improvised_act.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3219 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/precise_redaction.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3221 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/predictive_preparations.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3232 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/primal_witchstalker.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3246 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/proctor_of_potential.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3252 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/proft_consulting_detective.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3253 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/proft_sinister_mastermind.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3259 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/prophesied_end.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3264 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/protege_s_awakening.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3268 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/prudent_fateseer_peer_review.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3283 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/puppet_crafting.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3292 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/pyre_rhymer_molten_tide.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3366 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/rampart_hunter.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3372 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/rank_rat.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3425 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/recursive_recruitment.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3443 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/refute_destiny.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3467 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/repurposed_enforcer.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3474 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/rescue_girl_first_responder.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3496 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/restore_with_empathy.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3505 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/return_to_the_light_realms.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3512 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/rewrite_regrets.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3539 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/rise_of_the_deathbringer.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3571 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/rockfall_vale.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3577 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/roiling_canopy.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3585 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/room_of_refuge.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3625 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/ruric_thar_biomagus.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3626 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/ruric_thar_magecrusher.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3654 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/saheeli_consul_of_oversight.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3655 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/saheeli_jewel_of_avishkar.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3667 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/samut_hazoret_s_champion.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3668 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/samut_tyrant_of_naktamun.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3673 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/sanctum_lurker.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3741 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/screeching_soulbreaker.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3755 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/seasoned_cryomancer.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3785 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/semester_foreseer_peer_review.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3822 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/shatterwing_pegasus.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3836 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/shipwreck_marsh.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3865 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/silence_the_echo.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3877 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/simulacrum_shaper.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3892 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/skilled_battlecarver.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3956 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/solarium_sentry.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3959 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/solitary_cell.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3962 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/solve_for_disappointment.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3963 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/something_worth_saving.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4013 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/sphinx_of_false_conclusions.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4016 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/sphinx_s_approach.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4142 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/stingcaster_mage.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4144 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/stingerquill_annex.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4145 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/stingerquill_charm.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4146 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/stingerquill_voxmancer_vicious_verse.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4148 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/stinging_vitriol.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4267 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/sureshot_sower.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4268 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/surgical_precision.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4274 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/surveillance_phantasm.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4327 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/tam_s_resistance.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4330 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/tam_the_possibility.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4338 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/tarmogoyf.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4385 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/tenured_tethermage.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4387 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/terminal_criticism.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4403 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/tether_technician.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4404 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/tethermage_s_advantage.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4405 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/tetsuko_umezawa_fugitive.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4406 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/tetsuko_umezawa_pursuer.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4409 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/teyo_diamondblade_mage.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4410 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/teyo_lightshield_expert.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4413 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/thalia_the_survivor.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4439 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/the_echoverse_fulcrum.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4509 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/the_theorist_jace_beleren.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4521 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/theoretical_necromancer.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4522 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/theorist_s_proxy.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4523 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/theorist_s_sanctum.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4524 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/theorix_annex.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4525 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/theorix_charm.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4526 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/theorix_metamage_omit_variables.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4608 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/tinybones_pocket_nuisance.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4612 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/titanbones_towering_heart.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4627 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/tomik_izzet_sparkmage.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4628 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/tomik_orzhov_lawmage.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4658 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/transformative_commons.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4667 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/traxos_academy_guardian.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4668 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/traxos_scourge_eternal.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4724 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/twinned_vision.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4726 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/twisted_fates.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4739 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/uldaros_theorix.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4772 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/undulating_witness.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4780 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/unflinching_hortimancer.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4804 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/unsummon.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4850 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/variable_chaser_arc_of_fortune.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4874 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/verdant_kraken.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4889 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/vigorbloom_annex.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4890 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/vigorbloom_charm.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4891 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/vigorbloom_vanguard_seed_suture.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4898 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/vindictive_triumph.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4901 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/vinelasher_adept.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4903 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/violent_echoes.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4928 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/void_extrapolator_omit_variables.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4945 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/vraska_s_final_mercy.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4946 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/vraska_soul_of_stone.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4947 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/vraska_the_cutting_glare.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4985 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/warrior_s_blades.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_5007 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/way_of_the_cryomancer.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_5008 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/way_of_the_deathbringer.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_5009 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/way_of_the_healer.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_5010 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/way_of_the_mentor.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_5011 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/way_of_the_mind_sculptor.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_5012 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/way_of_the_necromancer.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_5013 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/way_of_the_paradox.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_5014 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/way_of_the_pyromancer.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_5015 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/way_of_the_warlord.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_5016 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/way_of_the_wildspeaker.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_5037 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/whiplash_wordsmith_vicious_verse.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_5086 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/winter_team_player.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_5087 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/winter_tormented_loner.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_5111 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/woodwork_prodigy_soul_tether.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_5120 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/wrath_of_the_bloodmane.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_5123 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/wrecking_gecko.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_5131 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/yargle_glutton_of_urborg.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_5132 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/yargle_goliath_of_otaria.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_5140 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/yoshimaru_beloved_companion.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_5141 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/yoshimaru_scrappy_stray.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_5143 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/your_fate_ends_here.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_5147 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/yuriko_blade_of_the_mighty.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_5148 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fra/yuriko_hope_from_the_shadows.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
}
#[rustfmt::skip]
pub static ENTRIES: [CatalogueEntry; 285] = [
CatalogueEntry { definition: Definition { info: CardInfo { name: "Plains", oracle_id: Some("bc71ebf6-2056-41f7-be35-b2e5c34afa99"), mana_cost: "", type_line: "Basic Land — Plains", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0000::RULES.keywords, deck_limit: authored::card_0000::RULES.deck_limit }, effect: authored::card_0000::RULES.checked_effect(), triggers: authored::card_0000::RULES.triggers, activated: authored::card_0000::RULES.activated, enters: authored::card_0000::RULES.enters, entry: authored::card_0000::RULES.entry, kind: 2, basic: true, mana_color: Some(0), colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_0000::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Island", oracle_id: Some("b2c6aa39-2d2a-459c-a555-fb48ba993373"), mana_cost: "", type_line: "Basic Land — Island", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0001::RULES.keywords, deck_limit: authored::card_0001::RULES.deck_limit }, effect: authored::card_0001::RULES.checked_effect(), triggers: authored::card_0001::RULES.triggers, activated: authored::card_0001::RULES.activated, enters: authored::card_0001::RULES.enters, entry: authored::card_0001::RULES.entry, kind: 2, basic: true, mana_color: Some(1), colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_0001::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Swamp", oracle_id: Some("56719f6a-1a6c-4c0a-8d21-18f7d7350b68"), mana_cost: "", type_line: "Basic Land — Swamp", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0002::RULES.keywords, deck_limit: authored::card_0002::RULES.deck_limit }, effect: authored::card_0002::RULES.checked_effect(), triggers: authored::card_0002::RULES.triggers, activated: authored::card_0002::RULES.activated, enters: authored::card_0002::RULES.enters, entry: authored::card_0002::RULES.entry, kind: 2, basic: true, mana_color: Some(2), colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_0002::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Mountain", oracle_id: Some("a3fb7228-e76b-4e96-a40e-20b5fed75685"), mana_cost: "", type_line: "Basic Land — Mountain", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0003::RULES.keywords, deck_limit: authored::card_0003::RULES.deck_limit }, effect: authored::card_0003::RULES.checked_effect(), triggers: authored::card_0003::RULES.triggers, activated: authored::card_0003::RULES.activated, enters: authored::card_0003::RULES.enters, entry: authored::card_0003::RULES.entry, kind: 2, basic: true, mana_color: Some(3), colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_0003::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Forest", oracle_id: Some("b34bb2dc-c1af-4d77-b0b3-a0fb342a5fc6"), mana_cost: "", type_line: "Basic Land — Forest", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0004::RULES.keywords, deck_limit: authored::card_0004::RULES.deck_limit }, effect: authored::card_0004::RULES.checked_effect(), triggers: authored::card_0004::RULES.triggers, activated: authored::card_0004::RULES.activated, enters: authored::card_0004::RULES.enters, entry: authored::card_0004::RULES.entry, kind: 2, basic: true, mana_color: Some(4), colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_0004::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Academic Ascent", oracle_id: Some("406e853b-713f-4c89-9efb-aca74a11183f"), mana_cost: "{1}{W}", type_line: "Instant", oracle_keywords: &["Surveil", "Empower Jace"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0046::RULES.keywords, deck_limit: authored::card_0046::RULES.deck_limit }, effect: authored::card_0046::RULES.checked_effect(), triggers: authored::card_0046::RULES.triggers, activated: authored::card_0046::RULES.activated, enters: authored::card_0046::RULES.enters, entry: authored::card_0046::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_0046::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Aerid Konstrari", oracle_id: Some("172a8f58-c420-4497-b962-c5853f923667"), mana_cost: "{1}{R}{G}{G}", type_line: "Legendary Creature — Elder Sphinx", oracle_keywords: &["Flying"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_0069::RULES.keywords, deck_limit: authored::card_0069::RULES.deck_limit }, effect: authored::card_0069::RULES.checked_effect(), triggers: authored::card_0069::RULES.triggers, activated: authored::card_0069::RULES.activated, enters: authored::card_0069::RULES.enters, entry: authored::card_0069::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 2, 0], generic: 1 }, rules: &authored::card_0069::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Afterthought Sentry", oracle_id: Some("f0c533a2-751a-4c5a-8cec-b83da3c8c62f"), mana_cost: "{2}", type_line: "Artifact Creature — Gargoyle", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0079::RULES.keywords, deck_limit: authored::card_0079::RULES.deck_limit }, effect: authored::card_0079::RULES.checked_effect(), triggers: authored::card_0079::RULES.triggers, activated: authored::card_0079::RULES.activated, enters: authored::card_0079::RULES.enters, entry: authored::card_0079::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_0079::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Ajani Resolute", oracle_id: Some("0d60c095-4c6c-4618-8ef3-508f3d454efa"), mana_cost: "{1}{W}", type_line: "Legendary Planeswalker — Ajani", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_0111::RULES.keywords, deck_limit: authored::card_0111::RULES.deck_limit }, effect: authored::card_0111::RULES.checked_effect(), triggers: authored::card_0111::RULES.triggers, activated: authored::card_0111::RULES.activated, enters: authored::card_0111::RULES.enters, entry: authored::card_0111::RULES.entry, kind: 64, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_0111::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Ajani Unrelenting", oracle_id: Some("71620e92-45a4-49cb-94d5-348013ecf886"), mana_cost: "{4}{R}{R}", type_line: "Legendary Planeswalker — Ajani", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_0112::RULES.keywords, deck_limit: authored::card_0112::RULES.deck_limit }, effect: authored::card_0112::RULES.checked_effect(), triggers: authored::card_0112::RULES.triggers, activated: authored::card_0112::RULES.activated, enters: authored::card_0112::RULES.enters, entry: authored::card_0112::RULES.entry, kind: 64, basic: false, mana_color: None, colored: [0, 0, 0, 2, 0, 0], generic: 4 }, rules: &authored::card_0112::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Ajani's Anguish", oracle_id: Some("4a37f4a0-2869-4f81-b51b-4c3e7e0e0671"), mana_cost: "{X}{R}", type_line: "Enchantment", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0113::RULES.keywords, deck_limit: authored::card_0113::RULES.deck_limit }, effect: authored::card_0113::RULES.checked_effect(), triggers: authored::card_0113::RULES.triggers, activated: authored::card_0113::RULES.activated, enters: authored::card_0113::RULES.enters, entry: authored::card_0113::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 0 }, rules: &authored::card_0113::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Apex Witchstalker", oracle_id: Some("253d0cf7-65d3-4f13-aaf2-aae5bb60ac46"), mana_cost: "{4}{B}{B}", type_line: "Creature — Wolf", oracle_keywords: &["Landcycling", "Menace", "Basic landcycling", "Typecycling", "Cycling"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0183::RULES.keywords, deck_limit: authored::card_0183::RULES.deck_limit }, effect: authored::card_0183::RULES.checked_effect(), triggers: authored::card_0183::RULES.triggers, activated: authored::card_0183::RULES.activated, enters: authored::card_0183::RULES.enters, entry: authored::card_0183::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 2, 0, 0, 0], generic: 4 }, rules: &authored::card_0183::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Arcane Amphisbaena", oracle_id: Some("5c151e3b-fcf2-4adb-a60b-9eff0c360705"), mana_cost: "{1}{G}", type_line: "Creature — Snake", oracle_keywords: &["Surveil", "Empower Jace", "Deathtouch"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0203::RULES.keywords, deck_limit: authored::card_0203::RULES.deck_limit }, effect: authored::card_0203::RULES.checked_effect(), triggers: authored::card_0203::RULES.triggers, activated: authored::card_0203::RULES.activated, enters: authored::card_0203::RULES.enters, entry: authored::card_0203::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 1 }, rules: &authored::card_0203::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Archive Arbiter", oracle_id: Some("36cb3d18-019c-464d-b591-8c87e1b81f31"), mana_cost: "{6}", type_line: "Artifact Creature — Sphinx", oracle_keywords: &["Flying"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0211::RULES.keywords, deck_limit: authored::card_0211::RULES.deck_limit }, effect: authored::card_0211::RULES.checked_effect(), triggers: authored::card_0211::RULES.triggers, activated: authored::card_0211::RULES.activated, enters: authored::card_0211::RULES.enters, entry: authored::card_0211::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 6 }, rules: &authored::card_0211::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Arni, Humble Scribe", oracle_id: Some("a680d1ab-99f2-43d0-b289-5fdba4083d03"), mana_cost: "{2}{U}", type_line: "Legendary Creature — Human Wizard", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_0229::RULES.keywords, deck_limit: authored::card_0229::RULES.deck_limit }, effect: authored::card_0229::RULES.checked_effect(), triggers: authored::card_0229::RULES.triggers, activated: authored::card_0229::RULES.activated, enters: authored::card_0229::RULES.enters, entry: authored::card_0229::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_0229::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Arni, Renowned Champion", oracle_id: Some("1a929c4f-ae1f-44d9-a0a7-2ca2dc928dee"), mana_cost: "{3}{R}", type_line: "Legendary Creature — Human Berserker", oracle_keywords: &["Trample"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_0230::RULES.keywords, deck_limit: authored::card_0230::RULES.deck_limit }, effect: authored::card_0230::RULES.checked_effect(), triggers: authored::card_0230::RULES.triggers, activated: authored::card_0230::RULES.activated, enters: authored::card_0230::RULES.enters, entry: authored::card_0230::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 3 }, rules: &authored::card_0230::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Artifist Acumen", oracle_id: Some("129b41cf-8327-42f0-b186-1111b42c08ab"), mana_cost: "{R}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0233::RULES.keywords, deck_limit: authored::card_0233::RULES.deck_limit }, effect: authored::card_0233::RULES.checked_effect(), triggers: authored::card_0233::RULES.triggers, activated: authored::card_0233::RULES.activated, enters: authored::card_0233::RULES.enters, entry: authored::card_0233::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 0 }, rules: &authored::card_0233::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Avatar of Burgeoning Echoes", oracle_id: Some("08e8a63c-9fcc-48cf-a9af-15cbaeec943f"), mana_cost: "{G}{U}", type_line: "Creature — Avatar", oracle_keywords: &["Surveil", "Empower Jace", "Landfall"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0278::RULES.keywords, deck_limit: authored::card_0278::RULES.deck_limit }, effect: authored::card_0278::RULES.checked_effect(), triggers: authored::card_0278::RULES.triggers, activated: authored::card_0278::RULES.activated, enters: authored::card_0278::RULES.enters, entry: authored::card_0278::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 1, 0], generic: 0 }, rules: &authored::card_0278::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Awaken the Inferno", oracle_id: Some("e729fce5-a1cf-4e95-9fd5-fccf4f704d31"), mana_cost: "{4}{R}", type_line: "Sorcery", oracle_keywords: &["Landcycling", "Basic landcycling", "Typecycling", "Cycling"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0289::RULES.keywords, deck_limit: authored::card_0289::RULES.deck_limit }, effect: authored::card_0289::RULES.checked_effect(), triggers: authored::card_0289::RULES.triggers, activated: authored::card_0289::RULES.activated, enters: authored::card_0289::RULES.enters, entry: authored::card_0289::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 4 }, rules: &authored::card_0289::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Bestial Incursion", oracle_id: Some("bc00f29a-e9d2-4b86-bb22-33b686ac4365"), mana_cost: "{3}{G}", type_line: "Sorcery", oracle_keywords: &["Flashback"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0396::RULES.keywords, deck_limit: authored::card_0396::RULES.deck_limit }, effect: authored::card_0396::RULES.checked_effect(), triggers: authored::card_0396::RULES.triggers, activated: authored::card_0396::RULES.activated, enters: authored::card_0396::RULES.enters, entry: authored::card_0396::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 3 }, rules: &authored::card_0396::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Blazing Crescendo", oracle_id: Some("6344c96a-efa5-4125-8219-d333228391cf"), mana_cost: "{1}{R}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0442::RULES.keywords, deck_limit: authored::card_0442::RULES.deck_limit }, effect: authored::card_0442::RULES.checked_effect(), triggers: authored::card_0442::RULES.triggers, activated: authored::card_0442::RULES.activated, enters: authored::card_0442::RULES.enters, entry: authored::card_0442::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 1 }, rules: &authored::card_0442::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Blessed Ghoul", oracle_id: Some("fe99d90e-4809-4310-a3f7-ceb540511d55"), mana_cost: "{W/B}", type_line: "Creature — Zombie Cleric", oracle_keywords: &["Lifelink"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0447::RULES.keywords, deck_limit: authored::card_0447::RULES.deck_limit }, effect: authored::card_0447::RULES.checked_effect(), triggers: authored::card_0447::RULES.triggers, activated: authored::card_0447::RULES.activated, enters: authored::card_0447::RULES.enters, entry: authored::card_0447::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 0 }, rules: &authored::card_0447::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Bloodline Recollector", oracle_id: Some("c8e9a7e1-28ae-40e3-8715-1bb0aa2057c7"), mana_cost: "{1}{B}", type_line: "Creature — Vampire Warlock", oracle_keywords: &["Prepared"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0459::RULES.keywords, deck_limit: authored::card_0459::RULES.deck_limit }, effect: authored::card_0459::RULES.checked_effect(), triggers: authored::card_0459::RULES.triggers, activated: authored::card_0459::RULES.activated, enters: authored::card_0459::RULES.enters, entry: authored::card_0459::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 1 }, rules: &authored::card_0459::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Bloombrute", oracle_id: Some("b7040175-7411-43a8-9b46-dba1d7c2135d"), mana_cost: "{2}{G}{W}", type_line: "Creature — Plant Elemental", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0464::RULES.keywords, deck_limit: authored::card_0464::RULES.deck_limit }, effect: authored::card_0464::RULES.checked_effect(), triggers: authored::card_0464::RULES.triggers, activated: authored::card_0464::RULES.activated, enters: authored::card_0464::RULES.enters, entry: authored::card_0464::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 1, 0], generic: 2 }, rules: &authored::card_0464::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Blossom-Blessed Angel", oracle_id: Some("cd61c26b-8d3c-4730-92f2-0d94d961bafd"), mana_cost: "{3}{W}", type_line: "Creature — Angel Cleric", oracle_keywords: &["Flying", "Vigilance", "Prepared"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0469::RULES.keywords, deck_limit: authored::card_0469::RULES.deck_limit }, effect: authored::card_0469::RULES.checked_effect(), triggers: authored::card_0469::RULES.triggers, activated: authored::card_0469::RULES.activated, enters: authored::card_0469::RULES.enters, entry: authored::card_0469::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_0469::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Break Under Pressure", oracle_id: Some("8dd23c50-cc84-4064-89cd-8d52fbfbd4ab"), mana_cost: "{2}{B}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0539::RULES.keywords, deck_limit: authored::card_0539::RULES.deck_limit }, effect: authored::card_0539::RULES.checked_effect(), triggers: authored::card_0539::RULES.triggers, activated: authored::card_0539::RULES.activated, enters: authored::card_0539::RULES.enters, entry: authored::card_0539::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 2 }, rules: &authored::card_0539::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Budding Insurgent", oracle_id: Some("723669e0-4a04-45ec-8c76-b1a369aa6892"), mana_cost: "{2}{G}", type_line: "Creature — Dryad Scout", oracle_keywords: &["Vigilance"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0576::RULES.keywords, deck_limit: authored::card_0576::RULES.deck_limit }, effect: authored::card_0576::RULES.checked_effect(), triggers: authored::card_0576::RULES.triggers, activated: authored::card_0576::RULES.activated, enters: authored::card_0576::RULES.enters, entry: authored::card_0576::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 2 }, rules: &authored::card_0576::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Campus Crier", oracle_id: Some("c1a9d60f-a469-48f9-8f1c-3ca5ae86bd64"), mana_cost: "{1}{W}", type_line: "Creature — Human Advisor", oracle_keywords: &["Surveil", "Empower Jace"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0628::RULES.keywords, deck_limit: authored::card_0628::RULES.deck_limit }, effect: authored::card_0628::RULES.checked_effect(), triggers: authored::card_0628::RULES.triggers, activated: authored::card_0628::RULES.activated, enters: authored::card_0628::RULES.enters, entry: authored::card_0628::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_0628::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Carnivorous Cultivator", oracle_id: Some("db655c5a-06f6-42bc-8180-d9ca1c1d939a"), mana_cost: "{1}{G}", type_line: "Creature — Elf Warlock", oracle_keywords: &["Prepared", "Deathtouch"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0656::RULES.keywords, deck_limit: authored::card_0656::RULES.deck_limit }, effect: authored::card_0656::RULES.checked_effect(), triggers: authored::card_0656::RULES.triggers, activated: authored::card_0656::RULES.activated, enters: authored::card_0656::RULES.enters, entry: authored::card_0656::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 1 }, rules: &authored::card_0656::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Cast Away Doubt", oracle_id: Some("bcf29dd7-6d5a-4139-958e-7c59434c0770"), mana_cost: "{2}{B}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0675::RULES.keywords, deck_limit: authored::card_0675::RULES.deck_limit }, effect: authored::card_0675::RULES.checked_effect(), triggers: authored::card_0675::RULES.triggers, activated: authored::card_0675::RULES.activated, enters: authored::card_0675::RULES.enters, entry: authored::card_0675::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 2 }, rules: &authored::card_0675::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Chandra's Emberling", oracle_id: Some("d3576617-7867-4c3d-aa31-d6742483465b"), mana_cost: "{2}{R}", type_line: "Creature — Gremlin Elemental", oracle_keywords: &["Haste"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0713::RULES.keywords, deck_limit: authored::card_0713::RULES.deck_limit }, effect: authored::card_0713::RULES.checked_effect(), triggers: authored::card_0713::RULES.triggers, activated: authored::card_0713::RULES.activated, enters: authored::card_0713::RULES.enters, entry: authored::card_0713::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 2 }, rules: &authored::card_0713::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Chandra, Chill of Compliance", oracle_id: Some("c3dfa1e2-6785-49a0-a194-fb842a8eb63c"), mana_cost: "{1}{U}{U}", type_line: "Legendary Planeswalker — Chandra", oracle_keywords: &["Surveil"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_0714::RULES.keywords, deck_limit: authored::card_0714::RULES.deck_limit }, effect: authored::card_0714::RULES.checked_effect(), triggers: authored::card_0714::RULES.triggers, activated: authored::card_0714::RULES.activated, enters: authored::card_0714::RULES.enters, entry: authored::card_0714::RULES.entry, kind: 64, basic: false, mana_color: None, colored: [0, 2, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_0714::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Chandra, Torch of Defiance", oracle_id: Some("12cc800c-c7be-4811-8207-fd01a99cc892"), mana_cost: "{2}{R}{R}", type_line: "Legendary Planeswalker — Chandra", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_0717::RULES.keywords, deck_limit: authored::card_0717::RULES.deck_limit }, effect: authored::card_0717::RULES.checked_effect(), triggers: authored::card_0717::RULES.triggers, activated: authored::card_0717::RULES.activated, enters: authored::card_0717::RULES.enters, entry: authored::card_0717::RULES.entry, kind: 64, basic: false, mana_color: None, colored: [0, 0, 0, 2, 0, 0], generic: 2 }, rules: &authored::card_0717::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Charge the Sanctum", oracle_id: Some("9d835137-584c-46bd-952e-4b2812bba1a3"), mana_cost: "{2}{R/W}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0721::RULES.keywords, deck_limit: authored::card_0721::RULES.deck_limit }, effect: authored::card_0721::RULES.checked_effect(), triggers: authored::card_0721::RULES.triggers, activated: authored::card_0721::RULES.activated, enters: authored::card_0721::RULES.enters, entry: authored::card_0721::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 2 }, rules: &authored::card_0721::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Clash of Elements", oracle_id: Some("78304a06-4c9a-4cdf-b67f-067d88762381"), mana_cost: "{1}{U}{R}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0764::RULES.keywords, deck_limit: authored::card_0764::RULES.deck_limit }, effect: authored::card_0764::RULES.checked_effect(), triggers: authored::card_0764::RULES.triggers, activated: authored::card_0764::RULES.activated, enters: authored::card_0764::RULES.enters, entry: authored::card_0764::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 1, 0, 1, 0, 0], generic: 1 }, rules: &authored::card_0764::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Codie, Ravenous Codex", oracle_id: Some("a17e414c-8de0-47fd-bcbc-540c7fec7642"), mana_cost: "{3}", type_line: "Legendary Artifact Creature — Book Construct", oracle_keywords: &["Prepared"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_0788::RULES.keywords, deck_limit: authored::card_0788::RULES.deck_limit }, effect: authored::card_0788::RULES.checked_effect(), triggers: authored::card_0788::RULES.triggers, activated: authored::card_0788::RULES.activated, enters: authored::card_0788::RULES.enters, entry: authored::card_0788::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_0788::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Command the Stage", oracle_id: Some("13bfc51f-d079-40ef-a4ba-47d06d3150a3"), mana_cost: "{2}{R}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0812::RULES.keywords, deck_limit: authored::card_0812::RULES.deck_limit }, effect: authored::card_0812::RULES.checked_effect(), triggers: authored::card_0812::RULES.triggers, activated: authored::card_0812::RULES.activated, enters: authored::card_0812::RULES.enters, entry: authored::card_0812::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 2 }, rules: &authored::card_0812::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Compel Brutality", oracle_id: Some("3a9059ac-c595-4e71-b704-b04d2d47db70"), mana_cost: "{1}{G}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0820::RULES.keywords, deck_limit: authored::card_0820::RULES.deck_limit }, effect: authored::card_0820::RULES.checked_effect(), triggers: authored::card_0820::RULES.triggers, activated: authored::card_0820::RULES.activated, enters: authored::card_0820::RULES.enters, entry: authored::card_0820::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 1 }, rules: &authored::card_0820::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Countersculpt", oracle_id: Some("fc3353e9-fe24-4ca7-bff8-9da767f2903a"), mana_cost: "{U}{U}", type_line: "Instant", oracle_keywords: &["Surveil", "Empower Jace", "Behold"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0869::RULES.keywords, deck_limit: authored::card_0869::RULES.deck_limit }, effect: authored::card_0869::RULES.checked_effect(), triggers: authored::card_0869::RULES.triggers, activated: authored::card_0869::RULES.activated, enters: authored::card_0869::RULES.enters, entry: authored::card_0869::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 2, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_0869::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Craftwork Crusher", oracle_id: Some("d25284b9-50e4-4dcb-9aaf-6f6478c4a90a"), mana_cost: "{3}{R}{R}{G}{G}", type_line: "Artifact Creature — Boar Construct", oracle_keywords: &["Trample"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0877::RULES.keywords, deck_limit: authored::card_0877::RULES.deck_limit }, effect: authored::card_0877::RULES.checked_effect(), triggers: authored::card_0877::RULES.triggers, activated: authored::card_0877::RULES.activated, enters: authored::card_0877::RULES.enters, entry: authored::card_0877::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 0, 0, 2, 2, 0], generic: 3 }, rules: &authored::card_0877::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Craterclaw Colossus", oracle_id: Some("71355b51-e70e-47b0-8460-933af3567141"), mana_cost: "{4}{R}{R}{R}", type_line: "Artifact Creature — Beast Construct", oracle_keywords: &["Haste"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0881::RULES.keywords, deck_limit: authored::card_0881::RULES.deck_limit }, effect: authored::card_0881::RULES.checked_effect(), triggers: authored::card_0881::RULES.triggers, activated: authored::card_0881::RULES.activated, enters: authored::card_0881::RULES.enters, entry: authored::card_0881::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 0, 0, 3, 0, 0], generic: 4 }, rules: &authored::card_0881::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Cruel Calculations", oracle_id: Some("69b96fe5-9733-4b43-bd66-075742e142d9"), mana_cost: "{2}{U}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0902::RULES.keywords, deck_limit: authored::card_0902::RULES.deck_limit }, effect: authored::card_0902::RULES.checked_effect(), triggers: authored::card_0902::RULES.triggers, activated: authored::card_0902::RULES.activated, enters: authored::card_0902::RULES.enters, entry: authored::card_0902::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_0902::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Cryotheory Adept", oracle_id: Some("a9b3ee21-1af1-4e7e-8e48-d7c90d056f5c"), mana_cost: "{1}{U}", type_line: "Creature — Human Wizard", oracle_keywords: &["Prowess"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0911::RULES.keywords, deck_limit: authored::card_0911::RULES.deck_limit }, effect: authored::card_0911::RULES.checked_effect(), triggers: authored::card_0911::RULES.triggers, activated: authored::card_0911::RULES.activated, enters: authored::card_0911::RULES.enters, entry: authored::card_0911::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_0911::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Curse-Marred Demon", oracle_id: Some("f30a084c-5705-40f5-8e0d-2083d33973c7"), mana_cost: "{2}{R}{R}", type_line: "Creature — Demon", oracle_keywords: &["Flying", "Trample"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0935::RULES.keywords, deck_limit: authored::card_0935::RULES.deck_limit }, effect: authored::card_0935::RULES.checked_effect(), triggers: authored::card_0935::RULES.triggers, activated: authored::card_0935::RULES.activated, enters: authored::card_0935::RULES.enters, entry: authored::card_0935::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 2, 0, 0], generic: 2 }, rules: &authored::card_0935::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Danitha, Spear of Agony", oracle_id: Some("4b5c7f11-75f2-4bad-86cd-f0232861f569"), mana_cost: "{2}{B}", type_line: "Legendary Creature — Human Knight", oracle_keywords: &["First strike"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_0955::RULES.keywords, deck_limit: authored::card_0955::RULES.deck_limit }, effect: authored::card_0955::RULES.checked_effect(), triggers: authored::card_0955::RULES.triggers, activated: authored::card_0955::RULES.activated, enters: authored::card_0955::RULES.enters, entry: authored::card_0955::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 2 }, rules: &authored::card_0955::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Danitha, Sword of Hope", oracle_id: Some("f7b754d3-9863-420d-895d-00e70b8b078a"), mana_cost: "{2}{W}", type_line: "Legendary Creature — Human Knight", oracle_keywords: &["First strike"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_0956::RULES.keywords, deck_limit: authored::card_0956::RULES.deck_limit }, effect: authored::card_0956::RULES.checked_effect(), triggers: authored::card_0956::RULES.triggers, activated: authored::card_0956::RULES.activated, enters: authored::card_0956::RULES.enters, entry: authored::card_0956::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_0956::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Dark Matter Manipulator", oracle_id: Some("6a3862f0-8dfc-4ac9-8eb9-faefcad685d4"), mana_cost: "{B}", type_line: "Creature — Human Warlock", oracle_keywords: &["Mill"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0969::RULES.keywords, deck_limit: authored::card_0969::RULES.deck_limit }, effect: authored::card_0969::RULES.checked_effect(), triggers: authored::card_0969::RULES.triggers, activated: authored::card_0969::RULES.activated, enters: authored::card_0969::RULES.enters, entry: authored::card_0969::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 0 }, rules: &authored::card_0969::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Darklight Phoenix", oracle_id: Some("e4b51c2d-7e56-435b-89d6-85e33df65879"), mana_cost: "{3}{B}", type_line: "Creature — Phoenix", oracle_keywords: &["Flying", "Haste"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0970::RULES.keywords, deck_limit: authored::card_0970::RULES.deck_limit }, effect: authored::card_0970::RULES.checked_effect(), triggers: authored::card_0970::RULES.triggers, activated: authored::card_0970::RULES.activated, enters: authored::card_0970::RULES.enters, entry: authored::card_0970::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 3 }, rules: &authored::card_0970::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Dedicated Commons", oracle_id: Some("d4e0749a-e179-4c19-8536-bf8f1a198549"), mana_cost: "", type_line: "Land", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1017::RULES.keywords, deck_limit: authored::card_1017::RULES.deck_limit }, effect: authored::card_1017::RULES.checked_effect(), triggers: authored::card_1017::RULES.triggers, activated: authored::card_1017::RULES.activated, enters: authored::card_1017::RULES.enters, entry: authored::card_1017::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_1017::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Denzilore Fatehold", oracle_id: Some("0f8de7f0-c61d-415d-b4db-4959374343da"), mana_cost: "{1}{W}{U}{U}", type_line: "Legendary Creature — Elder Sphinx", oracle_keywords: &["Flying", "Flash"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_1043::RULES.keywords, deck_limit: authored::card_1043::RULES.deck_limit }, effect: authored::card_1043::RULES.checked_effect(), triggers: authored::card_1043::RULES.triggers, activated: authored::card_1043::RULES.activated, enters: authored::card_1043::RULES.enters, entry: authored::card_1043::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 2, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_1043::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Deserted Beach", oracle_id: Some("f0ec8681-da50-466b-8cdd-1dc710deccd9"), mana_cost: "", type_line: "Land", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1053::RULES.keywords, deck_limit: authored::card_1053::RULES.deck_limit }, effect: authored::card_1053::RULES.checked_effect(), triggers: authored::card_1053::RULES.triggers, activated: authored::card_1053::RULES.activated, enters: authored::card_1053::RULES.enters, entry: authored::card_1053::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_1053::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Desperate Futurescribe", oracle_id: Some("83c12f50-e5a1-479d-afab-edb0b40412f2"), mana_cost: "{2}{W}{U}", type_line: "Creature — Kor Scout", oracle_keywords: &["Flying"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1058::RULES.keywords, deck_limit: authored::card_1058::RULES.deck_limit }, effect: authored::card_1058::RULES.checked_effect(), triggers: authored::card_1058::RULES.triggers, activated: authored::card_1058::RULES.activated, enters: authored::card_1058::RULES.enters, entry: authored::card_1058::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 1, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_1058::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Diviner of Victory", oracle_id: Some("e7b78acd-5288-4528-97a8-a5b43de90bf5"), mana_cost: "{U}", type_line: "Creature — Dwarf Wizard", oracle_keywords: &["Surveil", "Prepared"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1110::RULES.keywords, deck_limit: authored::card_1110::RULES.deck_limit }, effect: authored::card_1110::RULES.checked_effect(), triggers: authored::card_1110::RULES.triggers, activated: authored::card_1110::RULES.activated, enters: authored::card_1110::RULES.enters, entry: authored::card_1110::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_1110::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Divining Duelist", oracle_id: Some("d0ce03fa-2284-4217-85b2-4db6b8a88c94"), mana_cost: "{2}{U}", type_line: "Creature — Merfolk Wizard", oracle_keywords: &["Flash"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1111::RULES.keywords, deck_limit: authored::card_1111::RULES.deck_limit }, effect: authored::card_1111::RULES.checked_effect(), triggers: authored::card_1111::RULES.triggers, activated: authored::card_1111::RULES.activated, enters: authored::card_1111::RULES.enters, entry: authored::card_1111::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_1111::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Draconic Visitor", oracle_id: Some("6745e850-68f6-480b-a1e0-e06180dde169"), mana_cost: "{3}{R}{R}", type_line: "Creature — Dragon", oracle_keywords: &["Flying"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1148::RULES.keywords, deck_limit: authored::card_1148::RULES.deck_limit }, effect: authored::card_1148::RULES.checked_effect(), triggers: authored::card_1148::RULES.triggers, activated: authored::card_1148::RULES.activated, enters: authored::card_1148::RULES.enters, entry: authored::card_1148::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 2, 0, 0], generic: 3 }, rules: &authored::card_1148::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Eardrum Rattler", oracle_id: Some("25151d18-8e73-4b0d-a8d2-e5fa291e5729"), mana_cost: "{1}{R}", type_line: "Creature — Human Bard", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1230::RULES.keywords, deck_limit: authored::card_1230::RULES.deck_limit }, effect: authored::card_1230::RULES.checked_effect(), triggers: authored::card_1230::RULES.triggers, activated: authored::card_1230::RULES.activated, enters: authored::card_1230::RULES.enters, entry: authored::card_1230::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 1 }, rules: &authored::card_1230::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Edgar, Ancient Bloodlord", oracle_id: Some("cab2bc93-f38b-4301-ac42-05765615586b"), mana_cost: "{W}{B}", type_line: "Legendary Creature — Vampire Noble", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_1262::RULES.keywords, deck_limit: authored::card_1262::RULES.deck_limit }, effect: authored::card_1262::RULES.checked_effect(), triggers: authored::card_1262::RULES.triggers, activated: authored::card_1262::RULES.activated, enters: authored::card_1262::RULES.enters, entry: authored::card_1262::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 1, 0, 0, 0], generic: 0 }, rules: &authored::card_1262::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Edgar, Moonlit Sovereign", oracle_id: Some("1917bda6-c0e1-4c80-8009-74c28cf6b8e9"), mana_cost: "{3}{G}{G}", type_line: "Legendary Creature — Werewolf Noble", oracle_keywords: &["Flash"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_1264::RULES.keywords, deck_limit: authored::card_1264::RULES.deck_limit }, effect: authored::card_1264::RULES.checked_effect(), triggers: authored::card_1264::RULES.triggers, activated: authored::card_1264::RULES.activated, enters: authored::card_1264::RULES.enters, entry: authored::card_1264::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 2, 0], generic: 3 }, rules: &authored::card_1264::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Emergency Phytomedic", oracle_id: Some("7737ccdc-49f2-463e-8ffd-43ab98d11f45"), mana_cost: "{G/W}", type_line: "Creature — Dryad Cleric", oracle_keywords: &["Prepared"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1312::RULES.keywords, deck_limit: authored::card_1312::RULES.deck_limit }, effect: authored::card_1312::RULES.checked_effect(), triggers: authored::card_1312::RULES.triggers, activated: authored::card_1312::RULES.activated, enters: authored::card_1312::RULES.enters, entry: authored::card_1312::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_1312::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Emrakul, the Exigent Doom", oracle_id: Some("4421ab7d-6d9b-4edd-b5a0-53a8ed84da6f"), mana_cost: "{10}", type_line: "Legendary Creature — Eldrazi", oracle_keywords: &["Flying", "Trample", "Ward"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_1324::RULES.keywords, deck_limit: authored::card_1324::RULES.deck_limit }, effect: authored::card_1324::RULES.checked_effect(), triggers: authored::card_1324::RULES.triggers, activated: authored::card_1324::RULES.activated, enters: authored::card_1324::RULES.enters, entry: authored::card_1324::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 10 }, rules: &authored::card_1324::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Enlightened Confidant", oracle_id: Some("a139eb7f-3853-490c-a839-35d0e5aed7b5"), mana_cost: "{1}{W}", type_line: "Creature — Kor Cleric", oracle_keywords: &["Surveil", "Lifelink"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1341::RULES.keywords, deck_limit: authored::card_1341::RULES.deck_limit }, effect: authored::card_1341::RULES.checked_effect(), triggers: authored::card_1341::RULES.triggers, activated: authored::card_1341::RULES.activated, enters: authored::card_1341::RULES.enters, entry: authored::card_1341::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_1341::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Entrust the Spark", oracle_id: Some("b796ecc8-a2e8-4ab9-8736-6825b650434d"), mana_cost: "{3}{G}{U}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1349::RULES.keywords, deck_limit: authored::card_1349::RULES.deck_limit }, effect: authored::card_1349::RULES.checked_effect(), triggers: authored::card_1349::RULES.triggers, activated: authored::card_1349::RULES.activated, enters: authored::card_1349::RULES.enters, entry: authored::card_1349::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 1, 0, 0, 1, 0], generic: 3 }, rules: &authored::card_1349::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Essence Burn", oracle_id: Some("f1216f36-00fd-4d03-85c1-f4910a02d4d0"), mana_cost: "{1}{R}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1371::RULES.keywords, deck_limit: authored::card_1371::RULES.deck_limit }, effect: authored::card_1371::RULES.checked_effect(), triggers: authored::card_1371::RULES.triggers, activated: authored::card_1371::RULES.activated, enters: authored::card_1371::RULES.enters, entry: authored::card_1371::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 1 }, rules: &authored::card_1371::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Extended Absence", oracle_id: Some("973fb1e3-83d5-4763-bea8-799b426a8848"), mana_cost: "{3}{B}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1412::RULES.keywords, deck_limit: authored::card_1412::RULES.deck_limit }, effect: authored::card_1412::RULES.checked_effect(), triggers: authored::card_1412::RULES.triggers, activated: authored::card_1412::RULES.activated, enters: authored::card_1412::RULES.enters, entry: authored::card_1412::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 3 }, rules: &authored::card_1412::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Extrapolate the Impossible", oracle_id: Some("1f47c869-f4a9-43b0-aec0-8da4e3a3fec5"), mana_cost: "{1}{B}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1416::RULES.keywords, deck_limit: authored::card_1416::RULES.deck_limit }, effect: authored::card_1416::RULES.checked_effect(), triggers: authored::card_1416::RULES.triggers, activated: authored::card_1416::RULES.activated, enters: authored::card_1416::RULES.enters, entry: authored::card_1416::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 1 }, rules: &authored::card_1416::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Eye of Jace", oracle_id: Some("c8f802ad-4621-4f71-b52d-fbc55853dc88"), mana_cost: "{1}", type_line: "Artifact", oracle_keywords: &["Surveil"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1418::RULES.keywords, deck_limit: authored::card_1418::RULES.deck_limit }, effect: authored::card_1418::RULES.checked_effect(), triggers: authored::card_1418::RULES.triggers, activated: authored::card_1418::RULES.activated, enters: authored::card_1418::RULES.enters, entry: authored::card_1418::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_1418::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Face Yourself", oracle_id: Some("3c0fe71c-bee1-41d3-aeb5-1723028eedde"), mana_cost: "{5}{R}{R}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1423::RULES.keywords, deck_limit: authored::card_1423::RULES.deck_limit }, effect: authored::card_1423::RULES.checked_effect(), triggers: authored::card_1423::RULES.triggers, activated: authored::card_1423::RULES.activated, enters: authored::card_1423::RULES.enters, entry: authored::card_1423::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 0, 2, 0, 0], generic: 5 }, rules: &authored::card_1423::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Fatehold Annex", oracle_id: Some("d35876a3-e891-43d3-a50c-69e8469497e0"), mana_cost: "", type_line: "Land", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1452::RULES.keywords, deck_limit: authored::card_1452::RULES.deck_limit }, effect: authored::card_1452::RULES.checked_effect(), triggers: authored::card_1452::RULES.triggers, activated: authored::card_1452::RULES.activated, enters: authored::card_1452::RULES.enters, entry: authored::card_1452::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_1452::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Fatehold Charm", oracle_id: Some("313aee0e-4090-4589-b323-a4edbda21c68"), mana_cost: "{W}{U}", type_line: "Instant", oracle_keywords: &["Empower Jace"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1453::RULES.keywords, deck_limit: authored::card_1453::RULES.deck_limit }, effect: authored::card_1453::RULES.checked_effect(), triggers: authored::card_1453::RULES.triggers, activated: authored::card_1453::RULES.activated, enters: authored::card_1453::RULES.enters, entry: authored::card_1453::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [1, 1, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_1453::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Fatehold Chronologist", oracle_id: Some("1063822f-47d3-42e9-8a21-f62b12609fe1"), mana_cost: "{1}{W/U}", type_line: "Creature — Bird Wizard", oracle_keywords: &["Flying", "Surveil", "Prepared"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1454::RULES.keywords, deck_limit: authored::card_1454::RULES.deck_limit }, effect: authored::card_1454::RULES.checked_effect(), triggers: authored::card_1454::RULES.triggers, activated: authored::card_1454::RULES.activated, enters: authored::card_1454::RULES.enters, entry: authored::card_1454::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_1454::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Fateshaper Aspirant", oracle_id: Some("bb3996b6-b68d-4ea1-b3dd-1600f8da2f1c"), mana_cost: "{4}{W}", type_line: "Creature — Rhino Cleric", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1455::RULES.keywords, deck_limit: authored::card_1455::RULES.deck_limit }, effect: authored::card_1455::RULES.checked_effect(), triggers: authored::card_1455::RULES.triggers, activated: authored::card_1455::RULES.activated, enters: authored::card_1455::RULES.enters, entry: authored::card_1455::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 4 }, rules: &authored::card_1455::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Fblthp, Impossibly Lost", oracle_id: Some("17e7212e-69d8-4014-af98-447baed2cae7"), mana_cost: "{1}{U}", type_line: "Legendary Creature — Homunculus", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_1457::RULES.keywords, deck_limit: authored::card_1457::RULES.deck_limit }, effect: authored::card_1457::RULES.checked_effect(), triggers: authored::card_1457::RULES.triggers, activated: authored::card_1457::RULES.activated, enters: authored::card_1457::RULES.enters, entry: authored::card_1457::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_1457::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Fblthp, Knows the Way", oracle_id: Some("48b891b6-3175-4864-bb98-af131098c557"), mana_cost: "{X}{G}{G}", type_line: "Legendary Creature — Homunculus Scout", oracle_keywords: &["Domain"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_1458::RULES.keywords, deck_limit: authored::card_1458::RULES.deck_limit }, effect: authored::card_1458::RULES.checked_effect(), triggers: authored::card_1458::RULES.triggers, activated: authored::card_1458::RULES.activated, enters: authored::card_1458::RULES.enters, entry: authored::card_1458::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 2, 0], generic: 0 }, rules: &authored::card_1458::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Ferocity of the Hunt", oracle_id: Some("747a1b36-eb4f-44e6-9e5a-bb1a0259a471"), mana_cost: "{1}{B/G}", type_line: "Enchantment — Aura", oracle_keywords: &["Enchant", "Flash"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1498::RULES.keywords, deck_limit: authored::card_1498::RULES.deck_limit }, effect: authored::card_1498::RULES.checked_effect(), triggers: authored::card_1498::RULES.triggers, activated: authored::card_1498::RULES.activated, enters: authored::card_1498::RULES.enters, entry: authored::card_1498::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 1 }, rules: &authored::card_1498::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Flickering Hound", oracle_id: Some("162421d2-8761-437f-bed9-578b61c96f1f"), mana_cost: "{3}{W}", type_line: "Creature — Dog", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1558::RULES.keywords, deck_limit: authored::card_1558::RULES.deck_limit }, effect: authored::card_1558::RULES.checked_effect(), triggers: authored::card_1558::RULES.triggers, activated: authored::card_1558::RULES.activated, enters: authored::card_1558::RULES.enters, entry: authored::card_1558::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_1558::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Flourishing Grapple", oracle_id: Some("6b4f1569-2025-46a8-89ba-79859208b04e"), mana_cost: "{G}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1568::RULES.keywords, deck_limit: authored::card_1568::RULES.deck_limit }, effect: authored::card_1568::RULES.checked_effect(), triggers: authored::card_1568::RULES.triggers, activated: authored::card_1568::RULES.activated, enters: authored::card_1568::RULES.enters, entry: authored::card_1568::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 0 }, rules: &authored::card_1568::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Formidable Commons", oracle_id: Some("3e51c060-1e86-4a0e-8fee-8b7273161468"), mana_cost: "", type_line: "Land", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1598::RULES.keywords, deck_limit: authored::card_1598::RULES.deck_limit }, effect: authored::card_1598::RULES.checked_effect(), triggers: authored::card_1598::RULES.triggers, activated: authored::card_1598::RULES.activated, enters: authored::card_1598::RULES.enters, entry: authored::card_1598::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_1598::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Frostbite Pyromental", oracle_id: Some("9a104f73-597b-49e3-8088-13db31f61900"), mana_cost: "{U}{R}{R}", type_line: "Creature — Elemental", oracle_keywords: &["Haste", "Trample"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1637::RULES.keywords, deck_limit: authored::card_1637::RULES.deck_limit }, effect: authored::card_1637::RULES.checked_effect(), triggers: authored::card_1637::RULES.triggers, activated: authored::card_1637::RULES.activated, enters: authored::card_1637::RULES.enters, entry: authored::card_1637::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 2, 0, 0], generic: 0 }, rules: &authored::card_1637::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Fulminous Forte", oracle_id: Some("2a29d028-d1a4-413f-9c01-d6842294420a"), mana_cost: "{2}{R}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1647::RULES.keywords, deck_limit: authored::card_1647::RULES.deck_limit }, effect: authored::card_1647::RULES.checked_effect(), triggers: authored::card_1647::RULES.triggers, activated: authored::card_1647::RULES.activated, enters: authored::card_1647::RULES.enters, entry: authored::card_1647::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 2 }, rules: &authored::card_1647::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Gallia, Tragic Host", oracle_id: Some("58285b70-0d13-4723-b5a4-91fb6f9ccf03"), mana_cost: "{1}{B}", type_line: "Legendary Creature — Zombie Satyr", oracle_keywords: &["Menace"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_1670::RULES.keywords, deck_limit: authored::card_1670::RULES.deck_limit }, effect: authored::card_1670::RULES.checked_effect(), triggers: authored::card_1670::RULES.triggers, activated: authored::card_1670::RULES.activated, enters: authored::card_1670::RULES.enters, entry: authored::card_1670::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 1 }, rules: &authored::card_1670::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Gallia, the Merrymaker", oracle_id: Some("87f39199-3e4b-44fa-8406-62019eb43c10"), mana_cost: "{1}{R}", type_line: "Legendary Creature — Satyr", oracle_keywords: &["Haste"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_1671::RULES.keywords, deck_limit: authored::card_1671::RULES.deck_limit }, effect: authored::card_1671::RULES.checked_effect(), triggers: authored::card_1671::RULES.triggers, activated: authored::card_1671::RULES.activated, enters: authored::card_1671::RULES.enters, entry: authored::card_1671::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 1 }, rules: &authored::card_1671::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Gardenize", oracle_id: Some("e74a2a4c-ca3c-49e8-9b0f-b4d1bb91fb54"), mana_cost: "{1}{G}{G}", type_line: "Enchantment", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1680::RULES.keywords, deck_limit: authored::card_1680::RULES.deck_limit }, effect: authored::card_1680::RULES.checked_effect(), triggers: authored::card_1680::RULES.triggers, activated: authored::card_1680::RULES.activated, enters: authored::card_1680::RULES.enters, entry: authored::card_1680::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 0, 0, 0, 2, 0], generic: 1 }, rules: &authored::card_1680::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Garruk, Curse Breaker", oracle_id: Some("184d4672-05a9-4182-8b3f-3561fb33ed62"), mana_cost: "{3}{G}{G}", type_line: "Legendary Planeswalker — Garruk", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_1687::RULES.keywords, deck_limit: authored::card_1687::RULES.deck_limit }, effect: authored::card_1687::RULES.checked_effect(), triggers: authored::card_1687::RULES.triggers, activated: authored::card_1687::RULES.activated, enters: authored::card_1687::RULES.enters, entry: authored::card_1687::RULES.entry, kind: 64, basic: false, mana_color: None, colored: [0, 0, 0, 0, 2, 0], generic: 3 }, rules: &authored::card_1687::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Garruk, Veiled Butcher", oracle_id: Some("4a6972fe-348a-4a19-a1d0-bdf8f1f9c792"), mana_cost: "{3}{B}{B}", type_line: "Legendary Planeswalker — Garruk", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_1688::RULES.keywords, deck_limit: authored::card_1688::RULES.deck_limit }, effect: authored::card_1688::RULES.checked_effect(), triggers: authored::card_1688::RULES.triggers, activated: authored::card_1688::RULES.activated, enters: authored::card_1688::RULES.enters, entry: authored::card_1688::RULES.entry, kind: 64, basic: false, mana_color: None, colored: [0, 0, 2, 0, 0, 0], generic: 3 }, rules: &authored::card_1688::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Geist of Saint Thalia", oracle_id: Some("ef32a4a9-14e2-4738-b4c2-53ce5e1d2a53"), mana_cost: "{1}{U}", type_line: "Legendary Creature — Spirit Cleric", oracle_keywords: &["Flying"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_1703::RULES.keywords, deck_limit: authored::card_1703::RULES.deck_limit }, effect: authored::card_1703::RULES.checked_effect(), triggers: authored::card_1703::RULES.triggers, activated: authored::card_1703::RULES.activated, enters: authored::card_1703::RULES.enters, entry: authored::card_1703::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_1703::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Generous Revival", oracle_id: Some("f8601f5f-0f7c-4689-9d15-137f1e82c49c"), mana_cost: "{2}{W}", type_line: "Sorcery", oracle_keywords: &["Flashback"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1708::RULES.keywords, deck_limit: authored::card_1708::RULES.deck_limit }, effect: authored::card_1708::RULES.checked_effect(), triggers: authored::card_1708::RULES.triggers, activated: authored::card_1708::RULES.activated, enters: authored::card_1708::RULES.enters, entry: authored::card_1708::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_1708::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Germinate Recruits", oracle_id: Some("2c3a1313-a66e-4317-9732-e4ff9288b0db"), mana_cost: "{2}{W}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1715::RULES.keywords, deck_limit: authored::card_1715::RULES.deck_limit }, effect: authored::card_1715::RULES.checked_effect(), triggers: authored::card_1715::RULES.triggers, activated: authored::card_1715::RULES.activated, enters: authored::card_1715::RULES.enters, entry: authored::card_1715::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_1715::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Ghalta the Immovable", oracle_id: Some("0d496c6e-8f7b-420f-9197-dacd2feb528f"), mana_cost: "{8}{W}", type_line: "Legendary Creature — Elder Dinosaur", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_1726::RULES.keywords, deck_limit: authored::card_1726::RULES.deck_limit }, effect: authored::card_1726::RULES.checked_effect(), triggers: authored::card_1726::RULES.triggers, activated: authored::card_1726::RULES.activated, enters: authored::card_1726::RULES.enters, entry: authored::card_1726::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 8 }, rules: &authored::card_1726::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Ghalta the Unstoppable", oracle_id: Some("8e5ea773-a60b-4502-9db5-ce93fa61cc89"), mana_cost: "{8}{G}", type_line: "Legendary Creature — Elder Dinosaur", oracle_keywords: &["Trample"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_1727::RULES.keywords, deck_limit: authored::card_1727::RULES.deck_limit }, effect: authored::card_1727::RULES.checked_effect(), triggers: authored::card_1727::RULES.triggers, activated: authored::card_1727::RULES.activated, enters: authored::card_1727::RULES.enters, entry: authored::card_1727::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 8 }, rules: &authored::card_1727::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Gideon the Oathless", oracle_id: Some("7afa530f-ea51-4bee-af3b-59b83ea5cb25"), mana_cost: "{2}{B}", type_line: "Legendary Creature — Human Mercenary", oracle_keywords: &["Ward"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_1743::RULES.keywords, deck_limit: authored::card_1743::RULES.deck_limit }, effect: authored::card_1743::RULES.checked_effect(), triggers: authored::card_1743::RULES.triggers, activated: authored::card_1743::RULES.activated, enters: authored::card_1743::RULES.enters, entry: authored::card_1743::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 2 }, rules: &authored::card_1743::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Gideon's Memorial", oracle_id: Some("c8f08704-7330-4fa9-b456-2e96aa252a2c"), mana_cost: "{1}{W}", type_line: "Legendary Artifact", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_1744::RULES.keywords, deck_limit: authored::card_1744::RULES.deck_limit }, effect: authored::card_1744::RULES.checked_effect(), triggers: authored::card_1744::RULES.triggers, activated: authored::card_1744::RULES.activated, enters: authored::card_1744::RULES.enters, entry: authored::card_1744::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_1744::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Graft Surgeon", oracle_id: Some("fe7938ce-6289-4bf8-a8f5-85ecccbb7f86"), mana_cost: "{2}{W}", type_line: "Creature — Human Cleric", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1840::RULES.keywords, deck_limit: authored::card_1840::RULES.deck_limit }, effect: authored::card_1840::RULES.checked_effect(), triggers: authored::card_1840::RULES.triggers, activated: authored::card_1840::RULES.activated, enters: authored::card_1840::RULES.enters, entry: authored::card_1840::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_1840::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Greenhouse Propagator", oracle_id: Some("b6b77cb9-49a5-4d1a-ae2d-02ae176dc1fa"), mana_cost: "{2}{G}", type_line: "Creature — Cat Druid", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1874::RULES.keywords, deck_limit: authored::card_1874::RULES.deck_limit }, effect: authored::card_1874::RULES.checked_effect(), triggers: authored::card_1874::RULES.triggers, activated: authored::card_1874::RULES.activated, enters: authored::card_1874::RULES.enters, entry: authored::card_1874::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 2 }, rules: &authored::card_1874::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Grim Repriser", oracle_id: Some("e853e2fb-f90d-4934-aa70-c8d3cdf12e56"), mana_cost: "{B}{R}", type_line: "Creature — Zombie Bard", oracle_keywords: &["Prowess"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1882::RULES.keywords, deck_limit: authored::card_1882::RULES.deck_limit }, effect: authored::card_1882::RULES.checked_effect(), triggers: authored::card_1882::RULES.triggers, activated: authored::card_1882::RULES.activated, enters: authored::card_1882::RULES.enters, entry: authored::card_1882::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 1, 0, 0], generic: 0 }, rules: &authored::card_1882::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Guiding Hydra", oracle_id: Some("24f1445c-16c9-45c0-be56-0266e6c78cdb"), mana_cost: "{X}{W}", type_line: "Creature — Hydra Horror", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1907::RULES.keywords, deck_limit: authored::card_1907::RULES.deck_limit }, effect: authored::card_1907::RULES.checked_effect(), triggers: authored::card_1907::RULES.triggers, activated: authored::card_1907::RULES.activated, enters: authored::card_1907::RULES.enters, entry: authored::card_1907::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_1907::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Hall of Echoes", oracle_id: Some("dc0a51d4-5b7e-4987-b1fe-349fd118a625"), mana_cost: "", type_line: "Land", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1930::RULES.keywords, deck_limit: authored::card_1930::RULES.deck_limit }, effect: authored::card_1930::RULES.checked_effect(), triggers: authored::card_1930::RULES.triggers, activated: authored::card_1930::RULES.activated, enters: authored::card_1930::RULES.enters, entry: authored::card_1930::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_1930::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Hallway Heckler", oracle_id: Some("d1af8018-35b5-4b6f-93c8-1ed6bd8c92b1"), mana_cost: "{2}{R}", type_line: "Creature — Elemental Sorcerer", oracle_keywords: &["Prepared"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1932::RULES.keywords, deck_limit: authored::card_1932::RULES.deck_limit }, effect: authored::card_1932::RULES.checked_effect(), triggers: authored::card_1932::RULES.triggers, activated: authored::card_1932::RULES.activated, enters: authored::card_1932::RULES.enters, entry: authored::card_1932::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 2 }, rules: &authored::card_1932::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Hapatra, the Desert Fang", oracle_id: Some("6894345f-52a6-46e4-b278-0783087daee1"), mana_cost: "{2}{B}{B}{G}", type_line: "Legendary Creature — Human Cleric", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_1937::RULES.keywords, deck_limit: authored::card_1937::RULES.deck_limit }, effect: authored::card_1937::RULES.checked_effect(), triggers: authored::card_1937::RULES.triggers, activated: authored::card_1937::RULES.activated, enters: authored::card_1937::RULES.enters, entry: authored::card_1937::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 2, 0, 1, 0], generic: 2 }, rules: &authored::card_1937::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Hapatra, the Desert Frost", oracle_id: Some("ac0f161a-e5ec-4a95-9f37-305bdf1ac900"), mana_cost: "{3}{U}", type_line: "Legendary Creature — Human Wizard", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_1938::RULES.keywords, deck_limit: authored::card_1938::RULES.deck_limit }, effect: authored::card_1938::RULES.checked_effect(), triggers: authored::card_1938::RULES.triggers, activated: authored::card_1938::RULES.activated, enters: authored::card_1938::RULES.enters, entry: authored::card_1938::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_1938::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Haunted Ridge", oracle_id: Some("e2a37967-4212-4553-9f77-bcb613405807"), mana_cost: "", type_line: "Land", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1963::RULES.keywords, deck_limit: authored::card_1963::RULES.deck_limit }, effect: authored::card_1963::RULES.checked_effect(), triggers: authored::card_1963::RULES.triggers, activated: authored::card_1963::RULES.activated, enters: authored::card_1963::RULES.enters, entry: authored::card_1963::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_1963::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Heartstring Puller", oracle_id: Some("2cdcf65c-03d2-416f-8a3f-0322f75e8595"), mana_cost: "{3}{R}", type_line: "Creature — Elf Sorcerer", oracle_keywords: &["Trample"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1983::RULES.keywords, deck_limit: authored::card_1983::RULES.deck_limit }, effect: authored::card_1983::RULES.checked_effect(), triggers: authored::card_1983::RULES.triggers, activated: authored::card_1983::RULES.activated, enters: authored::card_1983::RULES.enters, entry: authored::card_1983::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 3 }, rules: &authored::card_1983::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Heartwood Crafter", oracle_id: Some("c8b3a070-408e-4a8f-8597-6476eeec0a5f"), mana_cost: "{G}", type_line: "Creature — Elf Artificer", oracle_keywords: &["Prepared"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1984::RULES.keywords, deck_limit: authored::card_1984::RULES.deck_limit }, effect: authored::card_1984::RULES.checked_effect(), triggers: authored::card_1984::RULES.triggers, activated: authored::card_1984::RULES.activated, enters: authored::card_1984::RULES.enters, entry: authored::card_1984::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 0 }, rules: &authored::card_1984::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Hexhaven Battalion", oracle_id: Some("158823ed-6015-437a-82b0-8dadb1e9aab2"), mana_cost: "{4}{W}{W}", type_line: "Sorcery", oracle_keywords: &["Surveil", "Empower Jace", "Landcycling", "Basic landcycling", "Typecycling", "Cycling"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2021::RULES.keywords, deck_limit: authored::card_2021::RULES.deck_limit }, effect: authored::card_2021::RULES.checked_effect(), triggers: authored::card_2021::RULES.triggers, activated: authored::card_2021::RULES.activated, enters: authored::card_2021::RULES.enters, entry: authored::card_2021::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [2, 0, 0, 0, 0, 0], generic: 4 }, rules: &authored::card_2021::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Hexhaven Dueling Arena", oracle_id: Some("2cc075da-f79b-4050-84eb-2c1b9a30260c"), mana_cost: "", type_line: "Land", oracle_keywords: &["Prepared"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2022::RULES.keywords, deck_limit: authored::card_2022::RULES.deck_limit }, effect: authored::card_2022::RULES.checked_effect(), triggers: authored::card_2022::RULES.triggers, activated: authored::card_2022::RULES.activated, enters: authored::card_2022::RULES.enters, entry: authored::card_2022::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_2022::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Hexhaven Invigorator", oracle_id: Some("12d617b1-c95b-4b1c-a586-ccfdf5898b03"), mana_cost: "{G}{G}{G}{G}", type_line: "Creature — Chimera Horror", oracle_keywords: &["Vigilance"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2023::RULES.keywords, deck_limit: authored::card_2023::RULES.deck_limit }, effect: authored::card_2023::RULES.checked_effect(), triggers: authored::card_2023::RULES.triggers, activated: authored::card_2023::RULES.activated, enters: authored::card_2023::RULES.enters, entry: authored::card_2023::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 4, 0], generic: 0 }, rules: &authored::card_2023::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Hungering Puppetbeast", oracle_id: Some("690ae865-87bd-46b2-8e65-c53bd80c1a18"), mana_cost: "{3}{G}{G}", type_line: "Artifact Creature — Beast Construct", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2100::RULES.keywords, deck_limit: authored::card_2100::RULES.deck_limit }, effect: authored::card_2100::RULES.checked_effect(), triggers: authored::card_2100::RULES.triggers, activated: authored::card_2100::RULES.activated, enters: authored::card_2100::RULES.enters, entry: authored::card_2100::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 0, 0, 0, 2, 0], generic: 3 }, rules: &authored::card_2100::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Hunter's Axe", oracle_id: Some("d7d59fef-1401-464b-b1bb-ee5da92cde51"), mana_cost: "{G}", type_line: "Artifact — Equipment", oracle_keywords: &["Equip"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2104::RULES.keywords, deck_limit: authored::card_2104::RULES.deck_limit }, effect: authored::card_2104::RULES.checked_effect(), triggers: authored::card_2104::RULES.triggers, activated: authored::card_2104::RULES.activated, enters: authored::card_2104::RULES.enters, entry: authored::card_2104::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 0 }, rules: &authored::card_2104::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Icy Reception", oracle_id: Some("612c7807-be70-4e2f-afd2-3d3c03eddbbe"), mana_cost: "{1}{U}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2128::RULES.keywords, deck_limit: authored::card_2128::RULES.deck_limit }, effect: authored::card_2128::RULES.checked_effect(), triggers: authored::card_2128::RULES.triggers, activated: authored::card_2128::RULES.activated, enters: authored::card_2128::RULES.enters, entry: authored::card_2128::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_2128::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Identity Echo", oracle_id: Some("4dbf8c43-d7c5-4037-875e-6accd87aaf6f"), mana_cost: "{2}{R}", type_line: "Enchantment", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2129::RULES.keywords, deck_limit: authored::card_2129::RULES.deck_limit }, effect: authored::card_2129::RULES.checked_effect(), triggers: authored::card_2129::RULES.triggers, activated: authored::card_2129::RULES.activated, enters: authored::card_2129::RULES.enters, entry: authored::card_2129::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 2 }, rules: &authored::card_2129::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Infinite Coursework", oracle_id: Some("666b34b5-a0c2-43e3-befd-7c3979313682"), mana_cost: "{2}{U}", type_line: "Enchantment — Aura", oracle_keywords: &["Enchant", "Prepared"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2162::RULES.keywords, deck_limit: authored::card_2162::RULES.deck_limit }, effect: authored::card_2162::RULES.checked_effect(), triggers: authored::card_2162::RULES.triggers, activated: authored::card_2162::RULES.activated, enters: authored::card_2162::RULES.enters, entry: authored::card_2162::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_2162::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Ingris Stingerquill", oracle_id: Some("bda098cb-31ec-41a1-a9d7-122877cc69d2"), mana_cost: "{B}{R}{R}", type_line: "Legendary Creature — Elder Sphinx", oracle_keywords: &["Flying"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2168::RULES.keywords, deck_limit: authored::card_2168::RULES.deck_limit }, effect: authored::card_2168::RULES.checked_effect(), triggers: authored::card_2168::RULES.triggers, activated: authored::card_2168::RULES.activated, enters: authored::card_2168::RULES.enters, entry: authored::card_2168::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 2, 0, 0], generic: 0 }, rules: &authored::card_2168::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Innovative Commons", oracle_id: Some("6812e4d3-0034-4fc8-9ee9-520f76a0ea94"), mana_cost: "", type_line: "Land", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2175::RULES.keywords, deck_limit: authored::card_2175::RULES.deck_limit }, effect: authored::card_2175::RULES.checked_effect(), triggers: authored::card_2175::RULES.triggers, activated: authored::card_2175::RULES.activated, enters: authored::card_2175::RULES.enters, entry: authored::card_2175::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_2175::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Inspired Tethermage", oracle_id: Some("80c2784b-f27d-4a51-9380-171eb32b961d"), mana_cost: "{2}{G}", type_line: "Creature — Elf Warrior", oracle_keywords: &["Surveil", "Empower Jace"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2186::RULES.keywords, deck_limit: authored::card_2186::RULES.deck_limit }, effect: authored::card_2186::RULES.checked_effect(), triggers: authored::card_2186::RULES.triggers, activated: authored::card_2186::RULES.activated, enters: authored::card_2186::RULES.enters, entry: authored::card_2186::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 2 }, rules: &authored::card_2186::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Jace's Machinations", oracle_id: Some("89bd056d-8f5b-4d0e-b80a-58a984e21100"), mana_cost: "{2}{U}", type_line: "Instant", oracle_keywords: &["Surveil", "Empower Jace"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2248::RULES.keywords, deck_limit: authored::card_2248::RULES.deck_limit }, effect: authored::card_2248::RULES.checked_effect(), triggers: authored::card_2248::RULES.triggers, activated: authored::card_2248::RULES.activated, enters: authored::card_2248::RULES.enters, entry: authored::card_2248::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_2248::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Jace, Reality Sculptor", oracle_id: Some("bf0e9e00-fba5-448a-9052-ae730aea4bf8"), mana_cost: "{3}{U}{U}", type_line: "Legendary Planeswalker — Jace", oracle_keywords: &["Empower Jace"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2249::RULES.keywords, deck_limit: authored::card_2249::RULES.deck_limit }, effect: authored::card_2249::RULES.checked_effect(), triggers: authored::card_2249::RULES.triggers, activated: authored::card_2249::RULES.activated, enters: authored::card_2249::RULES.enters, entry: authored::card_2249::RULES.entry, kind: 64, basic: false, mana_color: None, colored: [0, 2, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_2249::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Jiang Yanggu, Alone", oracle_id: Some("fa8b3557-b35a-4a68-ab85-cf09683f3bc5"), mana_cost: "{4}{R}", type_line: "Legendary Creature — Human Berserker", oracle_keywords: &["Menace"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2277::RULES.keywords, deck_limit: authored::card_2277::RULES.deck_limit }, effect: authored::card_2277::RULES.checked_effect(), triggers: authored::card_2277::RULES.triggers, activated: authored::card_2277::RULES.activated, enters: authored::card_2277::RULES.enters, entry: authored::card_2277::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 4 }, rules: &authored::card_2277::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Jiang Yanggu, Never Alone", oracle_id: Some("4ba661d7-1cb8-4fec-bfaa-5f2797235fcf"), mana_cost: "{3}{G}", type_line: "Legendary Creature — Human Druid", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2278::RULES.keywords, deck_limit: authored::card_2278::RULES.deck_limit }, effect: authored::card_2278::RULES.checked_effect(), triggers: authored::card_2278::RULES.triggers, activated: authored::card_2278::RULES.activated, enters: authored::card_2278::RULES.enters, entry: authored::card_2278::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 3 }, rules: &authored::card_2278::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Karn, Argent Defender", oracle_id: Some("aecc621f-67da-41a7-9d47-05ded302ae35"), mana_cost: "{2}", type_line: "Legendary Artifact Creature — Golem", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2319::RULES.keywords, deck_limit: authored::card_2319::RULES.deck_limit }, effect: authored::card_2319::RULES.checked_effect(), triggers: authored::card_2319::RULES.triggers, activated: authored::card_2319::RULES.activated, enters: authored::card_2319::RULES.enters, entry: authored::card_2319::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_2319::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Karn, Gilded Guardian", oracle_id: Some("20b90a18-ff62-4995-b0d6-fbef44dca357"), mana_cost: "{2/W}{2/U}{2/B}{2/R}{2/G}", type_line: "Legendary Artifact Creature — Golem", oracle_keywords: &["Vigilance", "Trample"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2320::RULES.keywords, deck_limit: authored::card_2320::RULES.deck_limit }, effect: authored::card_2320::RULES.checked_effect(), triggers: authored::card_2320::RULES.triggers, activated: authored::card_2320::RULES.activated, enters: authored::card_2320::RULES.enters, entry: authored::card_2320::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 10 }, rules: &authored::card_2320::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Keeper of the Quiet Hour", oracle_id: Some("ecdfee75-15fd-41ea-89d4-4bfc1fa4d032"), mana_cost: "{3}", type_line: "Artifact Creature — Chimera", oracle_keywords: &["Surveil", "Empower Jace"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2335::RULES.keywords, deck_limit: authored::card_2335::RULES.deck_limit }, effect: authored::card_2335::RULES.checked_effect(), triggers: authored::card_2335::RULES.triggers, activated: authored::card_2335::RULES.activated, enters: authored::card_2335::RULES.enters, entry: authored::card_2335::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_2335::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Kindred Judgment", oracle_id: Some("b5dce42a-a769-4d7d-b29e-8722f79d4092"), mana_cost: "{5}{W}{W}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2360::RULES.keywords, deck_limit: authored::card_2360::RULES.deck_limit }, effect: authored::card_2360::RULES.checked_effect(), triggers: authored::card_2360::RULES.triggers, activated: authored::card_2360::RULES.activated, enters: authored::card_2360::RULES.enters, entry: authored::card_2360::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [2, 0, 0, 0, 0, 0], generic: 5 }, rules: &authored::card_2360::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Kiora of Fire and Ashes", oracle_id: Some("02b41d0e-82e7-4307-8cc4-8175ff78ea1f"), mana_cost: "{4}{R}{R}", type_line: "Legendary Creature — Merfolk Noble", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2366::RULES.keywords, deck_limit: authored::card_2366::RULES.deck_limit }, effect: authored::card_2366::RULES.checked_effect(), triggers: authored::card_2366::RULES.triggers, activated: authored::card_2366::RULES.activated, enters: authored::card_2366::RULES.enters, entry: authored::card_2366::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 2, 0, 0], generic: 4 }, rules: &authored::card_2366::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Kiora of Salt and Sand", oracle_id: Some("e70bb7f8-8098-49c5-929c-68f63cd821c5"), mana_cost: "{1}{G}{U}", type_line: "Legendary Creature — Merfolk Noble", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2367::RULES.keywords, deck_limit: authored::card_2367::RULES.deck_limit }, effect: authored::card_2367::RULES.checked_effect(), triggers: authored::card_2367::RULES.triggers, activated: authored::card_2367::RULES.activated, enters: authored::card_2367::RULES.enters, entry: authored::card_2367::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 1, 0], generic: 1 }, rules: &authored::card_2367::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Konstrari Annex", oracle_id: Some("31ea4f3f-392a-4319-a95b-eb28790a80e6"), mana_cost: "", type_line: "Land", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2395::RULES.keywords, deck_limit: authored::card_2395::RULES.deck_limit }, effect: authored::card_2395::RULES.checked_effect(), triggers: authored::card_2395::RULES.triggers, activated: authored::card_2395::RULES.activated, enters: authored::card_2395::RULES.enters, entry: authored::card_2395::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_2395::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Konstrari Charm", oracle_id: Some("40961b23-b153-434a-aeb9-b9c2ff613dcb"), mana_cost: "{R}{G}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2396::RULES.keywords, deck_limit: authored::card_2396::RULES.deck_limit }, effect: authored::card_2396::RULES.checked_effect(), triggers: authored::card_2396::RULES.triggers, activated: authored::card_2396::RULES.activated, enters: authored::card_2396::RULES.enters, entry: authored::card_2396::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 0, 1, 1, 0], generic: 0 }, rules: &authored::card_2396::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Konstrari Improviser", oracle_id: Some("bad2e3b2-ad47-4e17-b35e-0fcc13e82937"), mana_cost: "{1}{R/G}", type_line: "Creature — Human Artificer", oracle_keywords: &["Prepared"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2397::RULES.keywords, deck_limit: authored::card_2397::RULES.deck_limit }, effect: authored::card_2397::RULES.checked_effect(), triggers: authored::card_2397::RULES.triggers, activated: authored::card_2397::RULES.activated, enters: authored::card_2397::RULES.enters, entry: authored::card_2397::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 1 }, rules: &authored::card_2397::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Koth of the Homestead", oracle_id: Some("567aeb60-a441-41b1-98f4-54510d299317"), mana_cost: "{2}{W}", type_line: "Legendary Creature — Human Citizen", oracle_keywords: &["Landfall"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2399::RULES.keywords, deck_limit: authored::card_2399::RULES.deck_limit }, effect: authored::card_2399::RULES.checked_effect(), triggers: authored::card_2399::RULES.triggers, activated: authored::card_2399::RULES.activated, enters: authored::card_2399::RULES.enters, entry: authored::card_2399::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_2399::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Koth, the Geomancer", oracle_id: Some("3b17da84-ae4d-4fc1-a327-85e54e6156c6"), mana_cost: "{2}{R}", type_line: "Legendary Creature — Human Warrior", oracle_keywords: &["Reach", "Landfall"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2400::RULES.keywords, deck_limit: authored::card_2400::RULES.deck_limit }, effect: authored::card_2400::RULES.checked_effect(), triggers: authored::card_2400::RULES.triggers, activated: authored::card_2400::RULES.activated, enters: authored::card_2400::RULES.enters, entry: authored::card_2400::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 2 }, rules: &authored::card_2400::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Kwia Vigorbloom", oracle_id: Some("07a28621-e617-46b0-af3e-2efb03b5056a"), mana_cost: "{3}{G}{W}{W}", type_line: "Legendary Creature — Elder Sphinx", oracle_keywords: &["Flying", "Lifelink", "Vigilance", "Ward"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2425::RULES.keywords, deck_limit: authored::card_2425::RULES.deck_limit }, effect: authored::card_2425::RULES.checked_effect(), triggers: authored::card_2425::RULES.triggers, activated: authored::card_2425::RULES.activated, enters: authored::card_2425::RULES.enters, entry: authored::card_2425::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [2, 0, 0, 0, 1, 0], generic: 3 }, rules: &authored::card_2425::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Last Gasp", oracle_id: Some("a82c3860-4dd6-4ffd-aa8f-ab8df687db6c"), mana_cost: "{1}{B}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2449::RULES.keywords, deck_limit: authored::card_2449::RULES.deck_limit }, effect: authored::card_2449::RULES.checked_effect(), triggers: authored::card_2449::RULES.triggers, activated: authored::card_2449::RULES.activated, enters: authored::card_2449::RULES.enters, entry: authored::card_2449::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 1 }, rules: &authored::card_2449::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Lich's Relic", oracle_id: Some("0540eaca-0e03-4831-955a-192ae2b87d35"), mana_cost: "{B}", type_line: "Artifact — Equipment", oracle_keywords: &["Equip"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2488::RULES.keywords, deck_limit: authored::card_2488::RULES.deck_limit }, effect: authored::card_2488::RULES.checked_effect(), triggers: authored::card_2488::RULES.triggers, activated: authored::card_2488::RULES.activated, enters: authored::card_2488::RULES.enters, entry: authored::card_2488::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 0 }, rules: &authored::card_2488::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Liliana the Faultless", oracle_id: Some("22c2e66d-a2a1-46db-a70a-dff5ad5d6c14"), mana_cost: "{W}", type_line: "Legendary Creature — Human Cleric", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2506::RULES.keywords, deck_limit: authored::card_2506::RULES.deck_limit }, effect: authored::card_2506::RULES.checked_effect(), triggers: authored::card_2506::RULES.triggers, activated: authored::card_2506::RULES.activated, enters: authored::card_2506::RULES.enters, entry: authored::card_2506::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_2506::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Liliana the Repentant", oracle_id: Some("5eb4403f-f199-4f75-a7c6-e76783f9b07d"), mana_cost: "{1}{B}", type_line: "Legendary Creature — Human Warlock", oracle_keywords: &["Exhaust", "Mill"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2507::RULES.keywords, deck_limit: authored::card_2507::RULES.deck_limit }, effect: authored::card_2507::RULES.checked_effect(), triggers: authored::card_2507::RULES.triggers, activated: authored::card_2507::RULES.activated, enters: authored::card_2507::RULES.enters, entry: authored::card_2507::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 1 }, rules: &authored::card_2507::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Living Library", oracle_id: Some("81f022a7-fd19-4340-b8a0-6b11ab3e9d91"), mana_cost: "{2}", type_line: "Artifact Creature — Book Illusion", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2525::RULES.keywords, deck_limit: authored::card_2525::RULES.deck_limit }, effect: authored::card_2525::RULES.checked_effect(), triggers: authored::card_2525::RULES.triggers, activated: authored::card_2525::RULES.activated, enters: authored::card_2525::RULES.enters, entry: authored::card_2525::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_2525::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Loot, the Anomaly", oracle_id: Some("e0142e7d-7b0b-4290-a56c-5fa950fdccf8"), mana_cost: "{2}{B}", type_line: "Legendary Creature — Beast Horror", oracle_keywords: &["Threshold"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2550::RULES.keywords, deck_limit: authored::card_2550::RULES.deck_limit }, effect: authored::card_2550::RULES.checked_effect(), triggers: authored::card_2550::RULES.triggers, activated: authored::card_2550::RULES.activated, enters: authored::card_2550::RULES.enters, entry: authored::card_2550::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 2 }, rules: &authored::card_2550::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Loot, the Nexus", oracle_id: Some("a99ce9db-9b7d-45a0-bee7-0875fc62ca48"), mana_cost: "{2}{G}", type_line: "Legendary Creature — Beast Noble", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2552::RULES.keywords, deck_limit: authored::card_2552::RULES.deck_limit }, effect: authored::card_2552::RULES.checked_effect(), triggers: authored::card_2552::RULES.triggers, activated: authored::card_2552::RULES.activated, enters: authored::card_2552::RULES.enters, entry: authored::card_2552::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 2 }, rules: &authored::card_2552::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Loyal Tutor", oracle_id: Some("7e52f151-d1f6-4fcb-9b21-1baecae27da6"), mana_cost: "{W}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2573::RULES.keywords, deck_limit: authored::card_2573::RULES.deck_limit }, effect: authored::card_2573::RULES.checked_effect(), triggers: authored::card_2573::RULES.triggers, activated: authored::card_2573::RULES.activated, enters: authored::card_2573::RULES.enters, entry: authored::card_2573::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_2573::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Lyra, Archangel of Dawn", oracle_id: Some("0fca3328-484f-4050-925e-91840710b5d6"), mana_cost: "{2}{W}", type_line: "Legendary Creature — Angel Knight", oracle_keywords: &["Flying"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2592::RULES.keywords, deck_limit: authored::card_2592::RULES.deck_limit }, effect: authored::card_2592::RULES.checked_effect(), triggers: authored::card_2592::RULES.triggers, activated: authored::card_2592::RULES.activated, enters: authored::card_2592::RULES.enters, entry: authored::card_2592::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_2592::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Lyra, Tolarian Archangel", oracle_id: Some("d9060fff-0b65-4b3d-930d-6e36315b802c"), mana_cost: "{1}{U}{U}", type_line: "Legendary Creature — Angel Wizard", oracle_keywords: &["Flying"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2593::RULES.keywords, deck_limit: authored::card_2593::RULES.deck_limit }, effect: authored::card_2593::RULES.checked_effect(), triggers: authored::card_2593::RULES.triggers, activated: authored::card_2593::RULES.activated, enters: authored::card_2593::RULES.enters, entry: authored::card_2593::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 2, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_2593::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Mabel, Bitter Recluse", oracle_id: Some("36924990-8e3a-434e-bcd1-f03603cb350d"), mana_cost: "{B}", type_line: "Legendary Creature — Mouse Warlock", oracle_keywords: &["Deathtouch"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2599::RULES.keywords, deck_limit: authored::card_2599::RULES.deck_limit }, effect: authored::card_2599::RULES.checked_effect(), triggers: authored::card_2599::RULES.triggers, activated: authored::card_2599::RULES.activated, enters: authored::card_2599::RULES.enters, entry: authored::card_2599::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 0 }, rules: &authored::card_2599::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Mabel, Valley Hero", oracle_id: Some("ab56f7cf-9538-45b5-a657-1e2ebef1e92b"), mana_cost: "{1}{R}{W}", type_line: "Legendary Creature — Mouse Soldier", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2601::RULES.keywords, deck_limit: authored::card_2601::RULES.deck_limit }, effect: authored::card_2601::RULES.checked_effect(), triggers: authored::card_2601::RULES.triggers, activated: authored::card_2601::RULES.activated, enters: authored::card_2601::RULES.enters, entry: authored::card_2601::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 1, 0, 0], generic: 1 }, rules: &authored::card_2601::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Marwyn, the Clearcutter", oracle_id: Some("aff4e43d-cafa-47de-8e39-011f9254a26a"), mana_cost: "{R}", type_line: "Legendary Creature — Elf Warrior", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2672::RULES.keywords, deck_limit: authored::card_2672::RULES.deck_limit }, effect: authored::card_2672::RULES.checked_effect(), triggers: authored::card_2672::RULES.triggers, activated: authored::card_2672::RULES.activated, enters: authored::card_2672::RULES.enters, entry: authored::card_2672::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 0 }, rules: &authored::card_2672::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Marwyn, the Preserver", oracle_id: Some("eb03f6c1-c94f-4f09-b2e9-a55ce5533653"), mana_cost: "{1}{G}", type_line: "Legendary Creature — Elf Druid", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2673::RULES.keywords, deck_limit: authored::card_2673::RULES.deck_limit }, effect: authored::card_2673::RULES.checked_effect(), triggers: authored::card_2673::RULES.triggers, activated: authored::card_2673::RULES.activated, enters: authored::card_2673::RULES.enters, entry: authored::card_2673::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 1 }, rules: &authored::card_2673::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Massacre Girl, Most Wanted", oracle_id: Some("81f35914-73bf-429f-ba20-32e23ac30159"), mana_cost: "{4}{B}", type_line: "Legendary Creature — Human Assassin", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2677::RULES.keywords, deck_limit: authored::card_2677::RULES.deck_limit }, effect: authored::card_2677::RULES.checked_effect(), triggers: authored::card_2677::RULES.triggers, activated: authored::card_2677::RULES.activated, enters: authored::card_2677::RULES.enters, entry: authored::card_2677::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 4 }, rules: &authored::card_2677::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Master of Barbs", oracle_id: Some("e8c36337-135c-460d-a1da-cc01743dda71"), mana_cost: "{1}{R}", type_line: "Creature — Lizard Bard", oracle_keywords: &["Menace"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2681::RULES.keywords, deck_limit: authored::card_2681::RULES.deck_limit }, effect: authored::card_2681::RULES.checked_effect(), triggers: authored::card_2681::RULES.triggers, activated: authored::card_2681::RULES.activated, enters: authored::card_2681::RULES.enters, entry: authored::card_2681::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 1 }, rules: &authored::card_2681::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Medic's Kitesail", oracle_id: Some("eacfe895-5106-437c-b8c1-2ff03c107744"), mana_cost: "{2}", type_line: "Artifact — Equipment", oracle_keywords: &["Equip"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2703::RULES.keywords, deck_limit: authored::card_2703::RULES.deck_limit }, effect: authored::card_2703::RULES.checked_effect(), triggers: authored::card_2703::RULES.triggers, activated: authored::card_2703::RULES.activated, enters: authored::card_2703::RULES.enters, entry: authored::card_2703::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_2703::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Memory Trap", oracle_id: Some("70616a8c-9b6c-408e-8f3e-2c348ec136d8"), mana_cost: "{2}{W}", type_line: "Enchantment", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2716::RULES.keywords, deck_limit: authored::card_2716::RULES.deck_limit }, effect: authored::card_2716::RULES.checked_effect(), triggers: authored::card_2716::RULES.triggers, activated: authored::card_2716::RULES.activated, enters: authored::card_2716::RULES.enters, entry: authored::card_2716::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_2716::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Meticulous Commons", oracle_id: Some("7fdd471f-ec95-4aa7-8f8a-ad8946de5ab4"), mana_cost: "", type_line: "Land", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2735::RULES.keywords, deck_limit: authored::card_2735::RULES.deck_limit }, effect: authored::card_2735::RULES.checked_effect(), triggers: authored::card_2735::RULES.triggers, activated: authored::card_2735::RULES.activated, enters: authored::card_2735::RULES.enters, entry: authored::card_2735::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_2735::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Mind Meanderer", oracle_id: Some("18d1765b-3b60-4dda-93a6-d02a4e53f185"), mana_cost: "{3}{G}{U}{U}", type_line: "Creature — Bird Fish Illusion", oracle_keywords: &["Flying", "Fight"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2761::RULES.keywords, deck_limit: authored::card_2761::RULES.deck_limit }, effect: authored::card_2761::RULES.checked_effect(), triggers: authored::card_2761::RULES.triggers, activated: authored::card_2761::RULES.activated, enters: authored::card_2761::RULES.enters, entry: authored::card_2761::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 2, 0, 0, 1, 0], generic: 3 }, rules: &authored::card_2761::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Mindseeker Oculus", oracle_id: Some("aa9547b3-e4e4-4557-9dd1-3aa9a7cb9937"), mana_cost: "{2}{U}", type_line: "Creature — Homunculus", oracle_keywords: &["Surveil", "Empower Jace"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2768::RULES.keywords, deck_limit: authored::card_2768::RULES.deck_limit }, effect: authored::card_2768::RULES.checked_effect(), triggers: authored::card_2768::RULES.triggers, activated: authored::card_2768::RULES.activated, enters: authored::card_2768::RULES.enters, entry: authored::card_2768::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_2768::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Multiply by Zero", oracle_id: Some("d940611f-f84d-43fa-82d6-eecb4fb54164"), mana_cost: "{1}{B}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2875::RULES.keywords, deck_limit: authored::card_2875::RULES.deck_limit }, effect: authored::card_2875::RULES.checked_effect(), triggers: authored::card_2875::RULES.triggers, activated: authored::card_2875::RULES.activated, enters: authored::card_2875::RULES.enters, entry: authored::card_2875::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 1 }, rules: &authored::card_2875::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Murmuring Volume", oracle_id: Some("4baa7847-dfcc-4aa3-aa64-2deeaa3f013b"), mana_cost: "{3}", type_line: "Artifact — Book", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2882::RULES.keywords, deck_limit: authored::card_2882::RULES.deck_limit }, effect: authored::card_2882::RULES.checked_effect(), triggers: authored::card_2882::RULES.triggers, activated: authored::card_2882::RULES.activated, enters: authored::card_2882::RULES.enters, entry: authored::card_2882::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_2882::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "No Admittance", oracle_id: Some("d53ae01b-87e1-48e1-9fc3-23a734ce6622"), mana_cost: "{1}{R}", type_line: "Sorcery", oracle_keywords: &["Surveil", "Empower Jace"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2951::RULES.keywords, deck_limit: authored::card_2951::RULES.deck_limit }, effect: authored::card_2951::RULES.checked_effect(), triggers: authored::card_2951::RULES.triggers, activated: authored::card_2951::RULES.activated, enters: authored::card_2951::RULES.enters, entry: authored::card_2951::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 1 }, rules: &authored::card_2951::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Null Summoner", oracle_id: Some("56c412be-daab-4461-931f-1e2ea8ecd75d"), mana_cost: "{2}{U}{B}", type_line: "Creature — Human Warlock", oracle_keywords: &["Threshold"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2976::RULES.keywords, deck_limit: authored::card_2976::RULES.deck_limit }, effect: authored::card_2976::RULES.checked_effect(), triggers: authored::card_2976::RULES.triggers, activated: authored::card_2976::RULES.activated, enters: authored::card_2976::RULES.enters, entry: authored::card_2976::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 1, 0, 0, 0], generic: 2 }, rules: &authored::card_2976::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Omnipresence", oracle_id: Some("8dad1fa8-3533-4e3c-b9b3-fcd57b20a33d"), mana_cost: "{5}{G}{G}{G}", type_line: "Enchantment", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3019::RULES.keywords, deck_limit: authored::card_3019::RULES.deck_limit }, effect: authored::card_3019::RULES.checked_effect(), triggers: authored::card_3019::RULES.triggers, activated: authored::card_3019::RULES.activated, enters: authored::card_3019::RULES.enters, entry: authored::card_3019::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 0, 0, 0, 3, 0], generic: 5 }, rules: &authored::card_3019::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Overgrown Farmland", oracle_id: Some("709d2f10-1585-48c3-9058-ddd5f62f0452"), mana_cost: "", type_line: "Land", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3066::RULES.keywords, deck_limit: authored::card_3066::RULES.deck_limit }, effect: authored::card_3066::RULES.checked_effect(), triggers: authored::card_3066::RULES.triggers, activated: authored::card_3066::RULES.activated, enters: authored::card_3066::RULES.enters, entry: authored::card_3066::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_3066::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Overwrite the Multiverse", oracle_id: Some("d0372cde-bf30-4a1b-93bb-9ff4ac1c116d"), mana_cost: "{4}{B}{B}", type_line: "Sorcery", oracle_keywords: &["Surveil", "Empower Jace"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3078::RULES.keywords, deck_limit: authored::card_3078::RULES.deck_limit }, effect: authored::card_3078::RULES.checked_effect(), triggers: authored::card_3078::RULES.triggers, activated: authored::card_3078::RULES.activated, enters: authored::card_3078::RULES.enters, entry: authored::card_3078::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 2, 0, 0, 0], generic: 4 }, rules: &authored::card_3078::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Paradox Shaper", oracle_id: Some("511951ed-fbff-4e44-9429-27f237496672"), mana_cost: "{1}{U/B}", type_line: "Creature — Octopus Wizard", oracle_keywords: &["Prepared", "Mill"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3098::RULES.keywords, deck_limit: authored::card_3098::RULES.deck_limit }, effect: authored::card_3098::RULES.checked_effect(), triggers: authored::card_3098::RULES.triggers, activated: authored::card_3098::RULES.activated, enters: authored::card_3098::RULES.enters, entry: authored::card_3098::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_3098::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Perfected Theory", oracle_id: Some("d824a319-fa74-42e4-9cbe-dba1576d6bad"), mana_cost: "{U}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3127::RULES.keywords, deck_limit: authored::card_3127::RULES.deck_limit }, effect: authored::card_3127::RULES.checked_effect(), triggers: authored::card_3127::RULES.triggers, activated: authored::card_3127::RULES.activated, enters: authored::card_3127::RULES.enters, entry: authored::card_3127::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_3127::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Pia, Aether Ascetic", oracle_id: Some("6519dc16-4960-4220-aaa8-7e4279743714"), mana_cost: "{2}{G}", type_line: "Legendary Creature — Human Druid", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_3152::RULES.keywords, deck_limit: authored::card_3152::RULES.deck_limit }, effect: authored::card_3152::RULES.checked_effect(), triggers: authored::card_3152::RULES.triggers, activated: authored::card_3152::RULES.activated, enters: authored::card_3152::RULES.enters, entry: authored::card_3152::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 2 }, rules: &authored::card_3152::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Pia, Determined Rebuilder", oracle_id: Some("a49c0c64-ea87-44f4-9acf-e7181962aeb2"), mana_cost: "{2}{R}", type_line: "Legendary Creature — Human Artificer", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_3153::RULES.keywords, deck_limit: authored::card_3153::RULES.deck_limit }, effect: authored::card_3153::RULES.checked_effect(), triggers: authored::card_3153::RULES.triggers, activated: authored::card_3153::RULES.activated, enters: authored::card_3153::RULES.enters, entry: authored::card_3153::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 2 }, rules: &authored::card_3153::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Plan for All Outcomes", oracle_id: Some("7fd2cac5-cd29-4a47-9e89-9c6be39a0b67"), mana_cost: "{3}{U}", type_line: "Enchantment", oracle_keywords: &["Surveil", "Empower Jace"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3178::RULES.keywords, deck_limit: authored::card_3178::RULES.deck_limit }, effect: authored::card_3178::RULES.checked_effect(), triggers: authored::card_3178::RULES.triggers, activated: authored::card_3178::RULES.activated, enters: authored::card_3178::RULES.enters, entry: authored::card_3178::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_3178::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Pompous Battlemage", oracle_id: Some("33e3793c-098f-4897-88f3-9f9ce9081e0f"), mana_cost: "{R}", type_line: "Creature — Goblin Sorcerer", oracle_keywords: &["Prowess", "Prepared"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3201::RULES.keywords, deck_limit: authored::card_3201::RULES.deck_limit }, effect: authored::card_3201::RULES.checked_effect(), triggers: authored::card_3201::RULES.triggers, activated: authored::card_3201::RULES.activated, enters: authored::card_3201::RULES.enters, entry: authored::card_3201::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 0 }, rules: &authored::card_3201::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Precise Redaction", oracle_id: Some("c251b676-0e98-4047-bed1-d72c83aa8da0"), mana_cost: "{1}{U}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3219::RULES.keywords, deck_limit: authored::card_3219::RULES.deck_limit }, effect: authored::card_3219::RULES.checked_effect(), triggers: authored::card_3219::RULES.triggers, activated: authored::card_3219::RULES.activated, enters: authored::card_3219::RULES.enters, entry: authored::card_3219::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_3219::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Predictive Preparations", oracle_id: Some("d06a5642-861b-4b0d-9ea5-07046b7e1d37"), mana_cost: "{1}{W}", type_line: "Sorcery", oracle_keywords: &["Flashback"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3221::RULES.keywords, deck_limit: authored::card_3221::RULES.deck_limit }, effect: authored::card_3221::RULES.checked_effect(), triggers: authored::card_3221::RULES.triggers, activated: authored::card_3221::RULES.activated, enters: authored::card_3221::RULES.enters, entry: authored::card_3221::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_3221::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Primal Witchstalker", oracle_id: Some("c21fafaf-4510-4752-a501-606ae68e6699"), mana_cost: "{1}{B}{G}", type_line: "Creature — Wolf", oracle_keywords: &["Menace", "Mill"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3232::RULES.keywords, deck_limit: authored::card_3232::RULES.deck_limit }, effect: authored::card_3232::RULES.checked_effect(), triggers: authored::card_3232::RULES.triggers, activated: authored::card_3232::RULES.activated, enters: authored::card_3232::RULES.enters, entry: authored::card_3232::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 1, 0], generic: 1 }, rules: &authored::card_3232::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Proctor of Potential", oracle_id: Some("c612ba0a-5134-45fa-a762-f4621bda4f83"), mana_cost: "{W}{U}", type_line: "Creature — Human Cleric", oracle_keywords: &["Surveil"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3246::RULES.keywords, deck_limit: authored::card_3246::RULES.deck_limit }, effect: authored::card_3246::RULES.checked_effect(), triggers: authored::card_3246::RULES.triggers, activated: authored::card_3246::RULES.activated, enters: authored::card_3246::RULES.enters, entry: authored::card_3246::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 1, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_3246::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Proft, Consulting Detective", oracle_id: Some("45473bdb-b96f-4213-a46d-b2dbdf6b5e56"), mana_cost: "{1}{U}", type_line: "Legendary Creature — Human Detective", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_3252::RULES.keywords, deck_limit: authored::card_3252::RULES.deck_limit }, effect: authored::card_3252::RULES.checked_effect(), triggers: authored::card_3252::RULES.triggers, activated: authored::card_3252::RULES.activated, enters: authored::card_3252::RULES.enters, entry: authored::card_3252::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_3252::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Proft, Sinister Mastermind", oracle_id: Some("a350856c-87f2-4605-999f-4e26aa362a55"), mana_cost: "{2}{B}", type_line: "Legendary Creature — Human Rogue", oracle_keywords: &["Threshold", "Menace"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_3253::RULES.keywords, deck_limit: authored::card_3253::RULES.deck_limit }, effect: authored::card_3253::RULES.checked_effect(), triggers: authored::card_3253::RULES.triggers, activated: authored::card_3253::RULES.activated, enters: authored::card_3253::RULES.enters, entry: authored::card_3253::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 2 }, rules: &authored::card_3253::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Prophesied End", oracle_id: Some("45f4d057-5134-40a2-9590-ddba73a65582"), mana_cost: "{1}{W}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3259::RULES.keywords, deck_limit: authored::card_3259::RULES.deck_limit }, effect: authored::card_3259::RULES.checked_effect(), triggers: authored::card_3259::RULES.triggers, activated: authored::card_3259::RULES.activated, enters: authored::card_3259::RULES.enters, entry: authored::card_3259::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_3259::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Protege's Awakening", oracle_id: Some("07617648-6490-4719-acec-7705671f7e01"), mana_cost: "{3}{U}", type_line: "Sorcery", oracle_keywords: &["Surveil", "Empower Jace"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3264::RULES.keywords, deck_limit: authored::card_3264::RULES.deck_limit }, effect: authored::card_3264::RULES.checked_effect(), triggers: authored::card_3264::RULES.triggers, activated: authored::card_3264::RULES.activated, enters: authored::card_3264::RULES.enters, entry: authored::card_3264::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_3264::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Prudent Fateseer", oracle_id: Some("63f67baa-3f37-41b8-9ee6-3c37f828917d"), mana_cost: "{1}{W/U}{W/U}", type_line: "Creature — Dwarf Wizard", oracle_keywords: &["Surveil", "Prepared"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3268::RULES.keywords, deck_limit: authored::card_3268::RULES.deck_limit }, effect: authored::card_3268::RULES.checked_effect(), triggers: authored::card_3268::RULES.triggers, activated: authored::card_3268::RULES.activated, enters: authored::card_3268::RULES.enters, entry: authored::card_3268::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [2, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_3268::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Puppet Crafting", oracle_id: Some("d13c111a-ebfc-48ac-b734-f203c7c29d21"), mana_cost: "{1}{G}", type_line: "Enchantment — Aura", oracle_keywords: &["Enchant"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3283::RULES.keywords, deck_limit: authored::card_3283::RULES.deck_limit }, effect: authored::card_3283::RULES.checked_effect(), triggers: authored::card_3283::RULES.triggers, activated: authored::card_3283::RULES.activated, enters: authored::card_3283::RULES.enters, entry: authored::card_3283::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 1 }, rules: &authored::card_3283::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Pyre Rhymer", oracle_id: Some("8940f503-fc6d-4fae-be74-22c660cae1a7"), mana_cost: "{1}{R}{R}", type_line: "Creature — Elemental Sorcerer", oracle_keywords: &["Prowess", "Prepared"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3292::RULES.keywords, deck_limit: authored::card_3292::RULES.deck_limit }, effect: authored::card_3292::RULES.checked_effect(), triggers: authored::card_3292::RULES.triggers, activated: authored::card_3292::RULES.activated, enters: authored::card_3292::RULES.enters, entry: authored::card_3292::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 2, 0, 0], generic: 1 }, rules: &authored::card_3292::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Rampart Hunter", oracle_id: Some("0867755a-9fb9-42ad-897e-6206fbfbc1f8"), mana_cost: "{3}{B}", type_line: "Creature — Horror", oracle_keywords: &["Deathtouch"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3366::RULES.keywords, deck_limit: authored::card_3366::RULES.deck_limit }, effect: authored::card_3366::RULES.checked_effect(), triggers: authored::card_3366::RULES.triggers, activated: authored::card_3366::RULES.activated, enters: authored::card_3366::RULES.enters, entry: authored::card_3366::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 3 }, rules: &authored::card_3366::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Rank Rat", oracle_id: Some("64e7ef06-ba30-4b19-9d44-ca77944cc930"), mana_cost: "{1}{B}", type_line: "Creature — Zombie Rat", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3372::RULES.keywords, deck_limit: authored::card_3372::RULES.deck_limit }, effect: authored::card_3372::RULES.checked_effect(), triggers: authored::card_3372::RULES.triggers, activated: authored::card_3372::RULES.activated, enters: authored::card_3372::RULES.enters, entry: authored::card_3372::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 1 }, rules: &authored::card_3372::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Recursive Recruitment", oracle_id: Some("b73e35b0-b8c9-471a-bdff-f7dc8f453d7a"), mana_cost: "{2}{U}{B}", type_line: "Sorcery", oracle_keywords: &["Flashback"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3425::RULES.keywords, deck_limit: authored::card_3425::RULES.deck_limit }, effect: authored::card_3425::RULES.checked_effect(), triggers: authored::card_3425::RULES.triggers, activated: authored::card_3425::RULES.activated, enters: authored::card_3425::RULES.enters, entry: authored::card_3425::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 1, 1, 0, 0, 0], generic: 2 }, rules: &authored::card_3425::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Refute Destiny", oracle_id: Some("2a18b3f0-8a78-46d8-b60f-3cd634062f1b"), mana_cost: "{1}{W}", type_line: "Sorcery", oracle_keywords: &["Surveil"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3443::RULES.keywords, deck_limit: authored::card_3443::RULES.deck_limit }, effect: authored::card_3443::RULES.checked_effect(), triggers: authored::card_3443::RULES.triggers, activated: authored::card_3443::RULES.activated, enters: authored::card_3443::RULES.enters, entry: authored::card_3443::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_3443::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Repurposed Enforcer", oracle_id: Some("b775a404-ec0a-47ff-bb9d-eae2273d945b"), mana_cost: "{1}{W}", type_line: "Creature — Human Soldier", oracle_keywords: &["Surveil", "Empower Jace"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3467::RULES.keywords, deck_limit: authored::card_3467::RULES.deck_limit }, effect: authored::card_3467::RULES.checked_effect(), triggers: authored::card_3467::RULES.triggers, activated: authored::card_3467::RULES.activated, enters: authored::card_3467::RULES.enters, entry: authored::card_3467::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_3467::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Rescue Girl, First Responder", oracle_id: Some("df6fc0b4-333b-4f6c-804c-ba6c8e29c88e"), mana_cost: "{2}{W}", type_line: "Legendary Creature — Human Cleric", oracle_keywords: &["Flying"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_3474::RULES.keywords, deck_limit: authored::card_3474::RULES.deck_limit }, effect: authored::card_3474::RULES.checked_effect(), triggers: authored::card_3474::RULES.triggers, activated: authored::card_3474::RULES.activated, enters: authored::card_3474::RULES.enters, entry: authored::card_3474::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_3474::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Restore with Empathy", oracle_id: Some("cb1e697f-6689-4f0b-8725-61d51432b8dd"), mana_cost: "{2}{G}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3496::RULES.keywords, deck_limit: authored::card_3496::RULES.deck_limit }, effect: authored::card_3496::RULES.checked_effect(), triggers: authored::card_3496::RULES.triggers, activated: authored::card_3496::RULES.activated, enters: authored::card_3496::RULES.enters, entry: authored::card_3496::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 2 }, rules: &authored::card_3496::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Return to the Light Realms", oracle_id: Some("217e76ca-ea94-417c-b5b0-82e2302eeee8"), mana_cost: "{7}{W}{W}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3505::RULES.keywords, deck_limit: authored::card_3505::RULES.deck_limit }, effect: authored::card_3505::RULES.checked_effect(), triggers: authored::card_3505::RULES.triggers, activated: authored::card_3505::RULES.activated, enters: authored::card_3505::RULES.enters, entry: authored::card_3505::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [2, 0, 0, 0, 0, 0], generic: 7 }, rules: &authored::card_3505::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Rewrite Regrets", oracle_id: Some("42565432-3a17-4274-87fc-016ab900e313"), mana_cost: "{3}{B}", type_line: "Sorcery", oracle_keywords: &["Surveil", "Empower Jace"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3512::RULES.keywords, deck_limit: authored::card_3512::RULES.deck_limit }, effect: authored::card_3512::RULES.checked_effect(), triggers: authored::card_3512::RULES.triggers, activated: authored::card_3512::RULES.activated, enters: authored::card_3512::RULES.enters, entry: authored::card_3512::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 3 }, rules: &authored::card_3512::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Rise of the Deathbringer", oracle_id: Some("57ea9f63-8267-4303-a3dd-9ded220294dc"), mana_cost: "{4}{B}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3539::RULES.keywords, deck_limit: authored::card_3539::RULES.deck_limit }, effect: authored::card_3539::RULES.checked_effect(), triggers: authored::card_3539::RULES.triggers, activated: authored::card_3539::RULES.activated, enters: authored::card_3539::RULES.enters, entry: authored::card_3539::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 4 }, rules: &authored::card_3539::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Rockfall Vale", oracle_id: Some("185c70c1-8403-4ae5-b45d-3679d4ee092a"), mana_cost: "", type_line: "Land", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3571::RULES.keywords, deck_limit: authored::card_3571::RULES.deck_limit }, effect: authored::card_3571::RULES.checked_effect(), triggers: authored::card_3571::RULES.triggers, activated: authored::card_3571::RULES.activated, enters: authored::card_3571::RULES.enters, entry: authored::card_3571::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_3571::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Roiling Canopy", oracle_id: Some("c6e87760-4cd0-4281-82d4-7a377ea960ad"), mana_cost: "", type_line: "Land", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3577::RULES.keywords, deck_limit: authored::card_3577::RULES.deck_limit }, effect: authored::card_3577::RULES.checked_effect(), triggers: authored::card_3577::RULES.triggers, activated: authored::card_3577::RULES.activated, enters: authored::card_3577::RULES.enters, entry: authored::card_3577::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_3577::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Room of Refuge", oracle_id: Some("27d5b2eb-94fe-45fb-806c-354044de7ba5"), mana_cost: "", type_line: "Land", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3585::RULES.keywords, deck_limit: authored::card_3585::RULES.deck_limit }, effect: authored::card_3585::RULES.checked_effect(), triggers: authored::card_3585::RULES.triggers, activated: authored::card_3585::RULES.activated, enters: authored::card_3585::RULES.enters, entry: authored::card_3585::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_3585::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Ruric Thar, Biomagus", oracle_id: Some("873fa095-5662-407a-bd24-9a1c4712a428"), mana_cost: "{4}{U}{U}", type_line: "Legendary Creature — Ogre Crab Wizard", oracle_keywords: &["Flying", "Prowess"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_3625::RULES.keywords, deck_limit: authored::card_3625::RULES.deck_limit }, effect: authored::card_3625::RULES.checked_effect(), triggers: authored::card_3625::RULES.triggers, activated: authored::card_3625::RULES.activated, enters: authored::card_3625::RULES.enters, entry: authored::card_3625::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 2, 0, 0, 0, 0], generic: 4 }, rules: &authored::card_3625::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Ruric Thar, Magecrusher", oracle_id: Some("453b2b3c-7df5-4515-9565-82f603cf451b"), mana_cost: "{5}{G}{G}", type_line: "Legendary Creature — Ogre Warrior", oracle_keywords: &["Reach", "Vigilance", "Trample"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_3626::RULES.keywords, deck_limit: authored::card_3626::RULES.deck_limit }, effect: authored::card_3626::RULES.checked_effect(), triggers: authored::card_3626::RULES.triggers, activated: authored::card_3626::RULES.activated, enters: authored::card_3626::RULES.enters, entry: authored::card_3626::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 2, 0], generic: 5 }, rules: &authored::card_3626::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Saheeli, Consul of Oversight", oracle_id: Some("dfa95c0e-393d-4b22-9de3-45a7efc0ba14"), mana_cost: "{3}{W}{W}", type_line: "Legendary Creature — Human Advisor", oracle_keywords: &["Flying"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_3654::RULES.keywords, deck_limit: authored::card_3654::RULES.deck_limit }, effect: authored::card_3654::RULES.checked_effect(), triggers: authored::card_3654::RULES.triggers, activated: authored::card_3654::RULES.activated, enters: authored::card_3654::RULES.enters, entry: authored::card_3654::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [2, 0, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_3654::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Saheeli, Jewel of Avishkar", oracle_id: Some("83705258-1f42-41db-a32e-de99ffd759eb"), mana_cost: "{2}{U}{R}", type_line: "Legendary Creature — Human Artificer", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_3655::RULES.keywords, deck_limit: authored::card_3655::RULES.deck_limit }, effect: authored::card_3655::RULES.checked_effect(), triggers: authored::card_3655::RULES.triggers, activated: authored::card_3655::RULES.activated, enters: authored::card_3655::RULES.enters, entry: authored::card_3655::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 1, 0, 0], generic: 2 }, rules: &authored::card_3655::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Samut, Hazoret's Champion", oracle_id: Some("add02a36-8811-438b-a8e1-f5c8a8146b4e"), mana_cost: "{1}{R}", type_line: "Legendary Creature — Human Warrior Cleric", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_3667::RULES.keywords, deck_limit: authored::card_3667::RULES.deck_limit }, effect: authored::card_3667::RULES.checked_effect(), triggers: authored::card_3667::RULES.triggers, activated: authored::card_3667::RULES.activated, enters: authored::card_3667::RULES.enters, entry: authored::card_3667::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 1 }, rules: &authored::card_3667::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Samut, Tyrant of Naktamun", oracle_id: Some("a275f7ac-7dae-4422-875a-f34f24b6f28a"), mana_cost: "{1}{U}", type_line: "Legendary Creature — Human Wizard", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_3668::RULES.keywords, deck_limit: authored::card_3668::RULES.deck_limit }, effect: authored::card_3668::RULES.checked_effect(), triggers: authored::card_3668::RULES.triggers, activated: authored::card_3668::RULES.activated, enters: authored::card_3668::RULES.enters, entry: authored::card_3668::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_3668::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Sanctum Lurker", oracle_id: Some("3ebd64d2-c178-45a2-a51f-9869754baa0f"), mana_cost: "{2}{B}", type_line: "Creature — Horror", oracle_keywords: &["Empower Jace"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3673::RULES.keywords, deck_limit: authored::card_3673::RULES.deck_limit }, effect: authored::card_3673::RULES.checked_effect(), triggers: authored::card_3673::RULES.triggers, activated: authored::card_3673::RULES.activated, enters: authored::card_3673::RULES.enters, entry: authored::card_3673::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 2 }, rules: &authored::card_3673::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Screeching Soulbreaker", oracle_id: Some("47e33aa1-2f54-471c-a484-de13bcd2dceb"), mana_cost: "{2}{B}", type_line: "Creature — Siren Bard", oracle_keywords: &["Flying"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3741::RULES.keywords, deck_limit: authored::card_3741::RULES.deck_limit }, effect: authored::card_3741::RULES.checked_effect(), triggers: authored::card_3741::RULES.triggers, activated: authored::card_3741::RULES.activated, enters: authored::card_3741::RULES.enters, entry: authored::card_3741::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 2 }, rules: &authored::card_3741::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Seasoned Cryomancer", oracle_id: Some("c3936db3-d3b6-4a92-9773-6f3c52dc419d"), mana_cost: "{1}{U}{U}", type_line: "Creature — Human Wizard", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3755::RULES.keywords, deck_limit: authored::card_3755::RULES.deck_limit }, effect: authored::card_3755::RULES.checked_effect(), triggers: authored::card_3755::RULES.triggers, activated: authored::card_3755::RULES.activated, enters: authored::card_3755::RULES.enters, entry: authored::card_3755::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 2, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_3755::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Semester Foreseer", oracle_id: Some("21b8d59f-a90d-44ff-a2e3-9f439ac3e14a"), mana_cost: "{3}{U}", type_line: "Creature — Human Wizard", oracle_keywords: &["Surveil", "Prepared"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3785::RULES.keywords, deck_limit: authored::card_3785::RULES.deck_limit }, effect: authored::card_3785::RULES.checked_effect(), triggers: authored::card_3785::RULES.triggers, activated: authored::card_3785::RULES.activated, enters: authored::card_3785::RULES.enters, entry: authored::card_3785::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_3785::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Shatterwing Pegasus", oracle_id: Some("7d6641e7-4d14-4a06-b588-0c752af529bd"), mana_cost: "{2}{W}", type_line: "Creature — Pegasus", oracle_keywords: &["Flying"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3822::RULES.keywords, deck_limit: authored::card_3822::RULES.deck_limit }, effect: authored::card_3822::RULES.checked_effect(), triggers: authored::card_3822::RULES.triggers, activated: authored::card_3822::RULES.activated, enters: authored::card_3822::RULES.enters, entry: authored::card_3822::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_3822::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Shipwreck Marsh", oracle_id: Some("5f42b67f-87fd-4f98-a0e8-0c8313f4bbc8"), mana_cost: "", type_line: "Land", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3836::RULES.keywords, deck_limit: authored::card_3836::RULES.deck_limit }, effect: authored::card_3836::RULES.checked_effect(), triggers: authored::card_3836::RULES.triggers, activated: authored::card_3836::RULES.activated, enters: authored::card_3836::RULES.enters, entry: authored::card_3836::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_3836::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Silence the Echo", oracle_id: Some("43654381-c55c-4d6d-abfd-12932ceb5a4a"), mana_cost: "{1}{B}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3865::RULES.keywords, deck_limit: authored::card_3865::RULES.deck_limit }, effect: authored::card_3865::RULES.checked_effect(), triggers: authored::card_3865::RULES.triggers, activated: authored::card_3865::RULES.activated, enters: authored::card_3865::RULES.enters, entry: authored::card_3865::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 1 }, rules: &authored::card_3865::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Simulacrum Shaper", oracle_id: Some("9e45e2da-7064-4ec0-8d09-55fc7d1aeaa8"), mana_cost: "{1}{G}{G}", type_line: "Creature — Elf Druid", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3877::RULES.keywords, deck_limit: authored::card_3877::RULES.deck_limit }, effect: authored::card_3877::RULES.checked_effect(), triggers: authored::card_3877::RULES.triggers, activated: authored::card_3877::RULES.activated, enters: authored::card_3877::RULES.enters, entry: authored::card_3877::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 2, 0], generic: 1 }, rules: &authored::card_3877::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Skilled Battlecarver", oracle_id: Some("4fc7c7a0-bfff-4833-a0d0-a38dc29e9dfb"), mana_cost: "{1}{R}", type_line: "Creature — Human Warrior", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3892::RULES.keywords, deck_limit: authored::card_3892::RULES.deck_limit }, effect: authored::card_3892::RULES.checked_effect(), triggers: authored::card_3892::RULES.triggers, activated: authored::card_3892::RULES.activated, enters: authored::card_3892::RULES.enters, entry: authored::card_3892::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 1 }, rules: &authored::card_3892::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Solarium Sentry", oracle_id: Some("afddba56-f9a0-4917-8167-c9fd17ca9954"), mana_cost: "{G}{W}", type_line: "Creature — Cat Soldier", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3956::RULES.keywords, deck_limit: authored::card_3956::RULES.deck_limit }, effect: authored::card_3956::RULES.checked_effect(), triggers: authored::card_3956::RULES.triggers, activated: authored::card_3956::RULES.activated, enters: authored::card_3956::RULES.enters, entry: authored::card_3956::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 1, 0], generic: 0 }, rules: &authored::card_3956::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Solitary Cell", oracle_id: Some("66a22b35-3dc0-4a74-8474-e73293403de6"), mana_cost: "{R}{W}", type_line: "Artifact", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3959::RULES.keywords, deck_limit: authored::card_3959::RULES.deck_limit }, effect: authored::card_3959::RULES.checked_effect(), triggers: authored::card_3959::RULES.triggers, activated: authored::card_3959::RULES.activated, enters: authored::card_3959::RULES.enters, entry: authored::card_3959::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [1, 0, 0, 1, 0, 0], generic: 0 }, rules: &authored::card_3959::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Solve for Disappointment", oracle_id: Some("3241a3f5-7ae5-4497-9dd2-a55f3370a055"), mana_cost: "{1}{B}", type_line: "Sorcery", oracle_keywords: &["Surveil", "Empower Jace"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3962::RULES.keywords, deck_limit: authored::card_3962::RULES.deck_limit }, effect: authored::card_3962::RULES.checked_effect(), triggers: authored::card_3962::RULES.triggers, activated: authored::card_3962::RULES.activated, enters: authored::card_3962::RULES.enters, entry: authored::card_3962::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 1 }, rules: &authored::card_3962::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Something Worth Saving", oracle_id: Some("46cf0e3e-3c3e-4eb8-a71a-4b4b87744ced"), mana_cost: "{1}{G}", type_line: "Instant", oracle_keywords: &["Mill"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3963::RULES.keywords, deck_limit: authored::card_3963::RULES.deck_limit }, effect: authored::card_3963::RULES.checked_effect(), triggers: authored::card_3963::RULES.triggers, activated: authored::card_3963::RULES.activated, enters: authored::card_3963::RULES.enters, entry: authored::card_3963::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 1 }, rules: &authored::card_3963::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Sphinx of False Conclusions", oracle_id: Some("9f004382-c57b-4fde-af97-0c85516c3cf4"), mana_cost: "{2}{U}{U}", type_line: "Creature — Sphinx Illusion", oracle_keywords: &["Flying", "Flash"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4013::RULES.keywords, deck_limit: authored::card_4013::RULES.deck_limit }, effect: authored::card_4013::RULES.checked_effect(), triggers: authored::card_4013::RULES.triggers, activated: authored::card_4013::RULES.activated, enters: authored::card_4013::RULES.enters, entry: authored::card_4013::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 2, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_4013::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Sphinx's Approach", oracle_id: Some("48b1ac27-430f-4fd9-a71b-6ad125c57fe3"), mana_cost: "{1}{U}{U}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4016::RULES.keywords, deck_limit: authored::card_4016::RULES.deck_limit }, effect: authored::card_4016::RULES.checked_effect(), triggers: authored::card_4016::RULES.triggers, activated: authored::card_4016::RULES.activated, enters: authored::card_4016::RULES.enters, entry: authored::card_4016::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 2, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_4016::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Stingcaster Mage", oracle_id: Some("056b651e-e0e2-4333-9235-d1ffe8fcca29"), mana_cost: "{1}{R}", type_line: "Creature — Human Wizard", oracle_keywords: &["Haste"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4142::RULES.keywords, deck_limit: authored::card_4142::RULES.deck_limit }, effect: authored::card_4142::RULES.checked_effect(), triggers: authored::card_4142::RULES.triggers, activated: authored::card_4142::RULES.activated, enters: authored::card_4142::RULES.enters, entry: authored::card_4142::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 1 }, rules: &authored::card_4142::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Stingerquill Annex", oracle_id: Some("26e20ae5-4059-4d63-9db5-6e12421d1aba"), mana_cost: "", type_line: "Land", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4144::RULES.keywords, deck_limit: authored::card_4144::RULES.deck_limit }, effect: authored::card_4144::RULES.checked_effect(), triggers: authored::card_4144::RULES.triggers, activated: authored::card_4144::RULES.activated, enters: authored::card_4144::RULES.enters, entry: authored::card_4144::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_4144::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Stingerquill Charm", oracle_id: Some("987aadac-d6a6-4e50-9468-9e8e53cc5529"), mana_cost: "{B}{R}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4145::RULES.keywords, deck_limit: authored::card_4145::RULES.deck_limit }, effect: authored::card_4145::RULES.checked_effect(), triggers: authored::card_4145::RULES.triggers, activated: authored::card_4145::RULES.activated, enters: authored::card_4145::RULES.enters, entry: authored::card_4145::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 1, 1, 0, 0], generic: 0 }, rules: &authored::card_4145::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Stingerquill Voxmancer", oracle_id: Some("e8f755f7-ec93-4d2e-bffc-c719984fa13d"), mana_cost: "{B/R}", type_line: "Creature — Goblin Sorcerer", oracle_keywords: &["Prepared"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4146::RULES.keywords, deck_limit: authored::card_4146::RULES.deck_limit }, effect: authored::card_4146::RULES.checked_effect(), triggers: authored::card_4146::RULES.triggers, activated: authored::card_4146::RULES.activated, enters: authored::card_4146::RULES.enters, entry: authored::card_4146::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 0 }, rules: &authored::card_4146::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Stinging Vitriol", oracle_id: Some("624cf822-0b4e-4f0c-9893-a53cb094f616"), mana_cost: "{B}{R}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4148::RULES.keywords, deck_limit: authored::card_4148::RULES.deck_limit }, effect: authored::card_4148::RULES.checked_effect(), triggers: authored::card_4148::RULES.triggers, activated: authored::card_4148::RULES.activated, enters: authored::card_4148::RULES.enters, entry: authored::card_4148::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 1, 1, 0, 0], generic: 0 }, rules: &authored::card_4148::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Sureshot Sower", oracle_id: Some("5bbc9d7d-91bd-4a58-8d19-f7be650720b7"), mana_cost: "{1}{G}", type_line: "Creature — Human Archer", oracle_keywords: &["Reach"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4267::RULES.keywords, deck_limit: authored::card_4267::RULES.deck_limit }, effect: authored::card_4267::RULES.checked_effect(), triggers: authored::card_4267::RULES.triggers, activated: authored::card_4267::RULES.activated, enters: authored::card_4267::RULES.enters, entry: authored::card_4267::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 1 }, rules: &authored::card_4267::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Surgical Precision", oracle_id: Some("0caccec3-4942-499a-9434-0c94389afcf8"), mana_cost: "{1}{W}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4268::RULES.keywords, deck_limit: authored::card_4268::RULES.deck_limit }, effect: authored::card_4268::RULES.checked_effect(), triggers: authored::card_4268::RULES.triggers, activated: authored::card_4268::RULES.activated, enters: authored::card_4268::RULES.enters, entry: authored::card_4268::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_4268::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Surveillance Phantasm", oracle_id: Some("b97648b7-ae98-45c9-8f0f-a52d6faf2d2d"), mana_cost: "{1}{U}", type_line: "Creature — Bird Illusion", oracle_keywords: &["Flying", "Surveil", "Vigilance", "Defender"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4274::RULES.keywords, deck_limit: authored::card_4274::RULES.deck_limit }, effect: authored::card_4274::RULES.checked_effect(), triggers: authored::card_4274::RULES.triggers, activated: authored::card_4274::RULES.activated, enters: authored::card_4274::RULES.enters, entry: authored::card_4274::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_4274::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Tam's Resistance", oracle_id: Some("7500b7dc-588d-46bd-acc5-885dcd464668"), mana_cost: "{1}{G/U}", type_line: "Sorcery", oracle_keywords: &["Surveil", "Empower Jace"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4327::RULES.keywords, deck_limit: authored::card_4327::RULES.deck_limit }, effect: authored::card_4327::RULES.checked_effect(), triggers: authored::card_4327::RULES.triggers, activated: authored::card_4327::RULES.activated, enters: authored::card_4327::RULES.enters, entry: authored::card_4327::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 1 }, rules: &authored::card_4327::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Tam, the Possibility", oracle_id: Some("c92c7744-0c92-4a36-9ada-4ffca812009e"), mana_cost: "{1}{G}{U}", type_line: "Legendary Creature — Gorgon Wizard", oracle_keywords: &["Proliferate"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_4330::RULES.keywords, deck_limit: authored::card_4330::RULES.deck_limit }, effect: authored::card_4330::RULES.checked_effect(), triggers: authored::card_4330::RULES.triggers, activated: authored::card_4330::RULES.activated, enters: authored::card_4330::RULES.enters, entry: authored::card_4330::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 1, 0], generic: 1 }, rules: &authored::card_4330::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Tarmogoyf", oracle_id: Some("45900b2f-f6a9-4c42-9642-008f3c1cf6dd"), mana_cost: "{1}{G}", type_line: "Creature — Lhurgoyf", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4338::RULES.keywords, deck_limit: authored::card_4338::RULES.deck_limit }, effect: authored::card_4338::RULES.checked_effect(), triggers: authored::card_4338::RULES.triggers, activated: authored::card_4338::RULES.activated, enters: authored::card_4338::RULES.enters, entry: authored::card_4338::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 1 }, rules: &authored::card_4338::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Tenured Tethermage", oracle_id: Some("571cb06d-55b5-46e4-9678-1f74aca73068"), mana_cost: "{1}{R}{G}", type_line: "Creature — Human Artificer", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4385::RULES.keywords, deck_limit: authored::card_4385::RULES.deck_limit }, effect: authored::card_4385::RULES.checked_effect(), triggers: authored::card_4385::RULES.triggers, activated: authored::card_4385::RULES.activated, enters: authored::card_4385::RULES.enters, entry: authored::card_4385::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 1, 0], generic: 1 }, rules: &authored::card_4385::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Terminal Criticism", oracle_id: Some("213b0814-f768-4408-9cb7-6f5960bcbb7a"), mana_cost: "{1}{B}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4387::RULES.keywords, deck_limit: authored::card_4387::RULES.deck_limit }, effect: authored::card_4387::RULES.checked_effect(), triggers: authored::card_4387::RULES.triggers, activated: authored::card_4387::RULES.activated, enters: authored::card_4387::RULES.enters, entry: authored::card_4387::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 1 }, rules: &authored::card_4387::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Tether Technician", oracle_id: Some("a9d0657f-2c5e-4b6f-b8ae-d657cc0f7aef"), mana_cost: "{4}{R}", type_line: "Creature — Minotaur Artificer", oracle_keywords: &["Reach"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4403::RULES.keywords, deck_limit: authored::card_4403::RULES.deck_limit }, effect: authored::card_4403::RULES.checked_effect(), triggers: authored::card_4403::RULES.triggers, activated: authored::card_4403::RULES.activated, enters: authored::card_4403::RULES.enters, entry: authored::card_4403::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 4 }, rules: &authored::card_4403::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Tethermage's Advantage", oracle_id: Some("b4f89885-eeb9-46c7-9820-9758b624ca24"), mana_cost: "{G}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4404::RULES.keywords, deck_limit: authored::card_4404::RULES.deck_limit }, effect: authored::card_4404::RULES.checked_effect(), triggers: authored::card_4404::RULES.triggers, activated: authored::card_4404::RULES.activated, enters: authored::card_4404::RULES.enters, entry: authored::card_4404::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 0 }, rules: &authored::card_4404::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Tetsuko Umezawa, Fugitive", oracle_id: Some("ceeeacbc-01b0-4421-aaca-2ce6cdbe45d7"), mana_cost: "{1}{U}", type_line: "Legendary Creature — Human Rogue", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_4405::RULES.keywords, deck_limit: authored::card_4405::RULES.deck_limit }, effect: authored::card_4405::RULES.checked_effect(), triggers: authored::card_4405::RULES.triggers, activated: authored::card_4405::RULES.activated, enters: authored::card_4405::RULES.enters, entry: authored::card_4405::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_4405::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Tetsuko Umezawa, Pursuer", oracle_id: Some("708d4567-a5a3-437f-bae4-ae10f1537aa9"), mana_cost: "{3}{R}", type_line: "Legendary Creature — Human Mercenary", oracle_keywords: &["Prowess", "Double strike"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_4406::RULES.keywords, deck_limit: authored::card_4406::RULES.deck_limit }, effect: authored::card_4406::RULES.checked_effect(), triggers: authored::card_4406::RULES.triggers, activated: authored::card_4406::RULES.activated, enters: authored::card_4406::RULES.enters, entry: authored::card_4406::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 3 }, rules: &authored::card_4406::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Teyo, Diamondblade Mage", oracle_id: Some("a3bfbe1a-7831-45bd-9e42-790c28f6054c"), mana_cost: "{3}{B}", type_line: "Legendary Creature — Human Warlock", oracle_keywords: &["Flash"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_4409::RULES.keywords, deck_limit: authored::card_4409::RULES.deck_limit }, effect: authored::card_4409::RULES.checked_effect(), triggers: authored::card_4409::RULES.triggers, activated: authored::card_4409::RULES.activated, enters: authored::card_4409::RULES.enters, entry: authored::card_4409::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 3 }, rules: &authored::card_4409::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Teyo, Lightshield Expert", oracle_id: Some("77a9fac6-3c26-4456-9e0b-f5839a3dcae2"), mana_cost: "{1}{W}", type_line: "Legendary Creature — Human Cleric", oracle_keywords: &["Flash"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_4410::RULES.keywords, deck_limit: authored::card_4410::RULES.deck_limit }, effect: authored::card_4410::RULES.checked_effect(), triggers: authored::card_4410::RULES.triggers, activated: authored::card_4410::RULES.activated, enters: authored::card_4410::RULES.enters, entry: authored::card_4410::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_4410::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Thalia, the Survivor", oracle_id: Some("55ef129e-698e-424f-be3c-3fbba6c2cc3e"), mana_cost: "{3}{W}", type_line: "Legendary Creature — Human Soldier", oracle_keywords: &["Lifelink"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_4413::RULES.keywords, deck_limit: authored::card_4413::RULES.deck_limit }, effect: authored::card_4413::RULES.checked_effect(), triggers: authored::card_4413::RULES.triggers, activated: authored::card_4413::RULES.activated, enters: authored::card_4413::RULES.enters, entry: authored::card_4413::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_4413::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "The Echoverse Fulcrum", oracle_id: Some("c04197ad-d2f3-499f-b863-bed0c869519c"), mana_cost: "{2}", type_line: "Legendary Artifact", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_4439::RULES.keywords, deck_limit: authored::card_4439::RULES.deck_limit }, effect: authored::card_4439::RULES.checked_effect(), triggers: authored::card_4439::RULES.triggers, activated: authored::card_4439::RULES.activated, enters: authored::card_4439::RULES.enters, entry: authored::card_4439::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_4439::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "The Theorist, Jace Beleren", oracle_id: Some("89a6e876-0b00-4671-9652-766fd7ef9bf2"), mana_cost: "{2}{U}{U}", type_line: "Legendary Planeswalker — Jace", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_4509::RULES.keywords, deck_limit: authored::card_4509::RULES.deck_limit }, effect: authored::card_4509::RULES.checked_effect(), triggers: authored::card_4509::RULES.triggers, activated: authored::card_4509::RULES.activated, enters: authored::card_4509::RULES.enters, entry: authored::card_4509::RULES.entry, kind: 64, basic: false, mana_color: None, colored: [0, 2, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_4509::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Theoretical Necromancer", oracle_id: Some("6c5ed756-b999-49f7-9a52-fb8196202f0f"), mana_cost: "{2}{B}", type_line: "Creature — Vampire Warlock", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4521::RULES.keywords, deck_limit: authored::card_4521::RULES.deck_limit }, effect: authored::card_4521::RULES.checked_effect(), triggers: authored::card_4521::RULES.triggers, activated: authored::card_4521::RULES.activated, enters: authored::card_4521::RULES.enters, entry: authored::card_4521::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 2 }, rules: &authored::card_4521::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Theorist's Proxy", oracle_id: Some("0089acfe-da66-4dd7-b1e5-4d7407f58257"), mana_cost: "{1}{U}", type_line: "Creature — Illusion", oracle_keywords: &["Surveil", "Empower Jace", "Flash"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4522::RULES.keywords, deck_limit: authored::card_4522::RULES.deck_limit }, effect: authored::card_4522::RULES.checked_effect(), triggers: authored::card_4522::RULES.triggers, activated: authored::card_4522::RULES.activated, enters: authored::card_4522::RULES.enters, entry: authored::card_4522::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_4522::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Theorist's Sanctum", oracle_id: Some("37513bd8-7303-4ac6-a7a7-224184faf758"), mana_cost: "", type_line: "Land — Island", oracle_keywords: &["Empower Jace", "Behold"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4523::RULES.keywords, deck_limit: authored::card_4523::RULES.deck_limit }, effect: authored::card_4523::RULES.checked_effect(), triggers: authored::card_4523::RULES.triggers, activated: authored::card_4523::RULES.activated, enters: authored::card_4523::RULES.enters, entry: authored::card_4523::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_4523::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Theorix Annex", oracle_id: Some("00ad5200-5179-4d1b-8bb3-a09e1b31be58"), mana_cost: "", type_line: "Land", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4524::RULES.keywords, deck_limit: authored::card_4524::RULES.deck_limit }, effect: authored::card_4524::RULES.checked_effect(), triggers: authored::card_4524::RULES.triggers, activated: authored::card_4524::RULES.activated, enters: authored::card_4524::RULES.enters, entry: authored::card_4524::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_4524::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Theorix Charm", oracle_id: Some("cf527bbd-e898-4aa9-909d-daec2f4b62ad"), mana_cost: "{U}{B}", type_line: "Instant", oracle_keywords: &["Mill"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4525::RULES.keywords, deck_limit: authored::card_4525::RULES.deck_limit }, effect: authored::card_4525::RULES.checked_effect(), triggers: authored::card_4525::RULES.triggers, activated: authored::card_4525::RULES.activated, enters: authored::card_4525::RULES.enters, entry: authored::card_4525::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 1, 1, 0, 0, 0], generic: 0 }, rules: &authored::card_4525::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Theorix Metamage", oracle_id: Some("92a0aa44-0ef7-4e6f-8288-7c33067c5fd1"), mana_cost: "{2}{U/B}", type_line: "Creature — Shade Wizard", oracle_keywords: &["Threshold", "Prepared", "Mill"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4526::RULES.keywords, deck_limit: authored::card_4526::RULES.deck_limit }, effect: authored::card_4526::RULES.checked_effect(), triggers: authored::card_4526::RULES.triggers, activated: authored::card_4526::RULES.activated, enters: authored::card_4526::RULES.enters, entry: authored::card_4526::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_4526::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Tinybones, Pocket Nuisance", oracle_id: Some("97dcf9fb-3f2f-4aa6-959a-e3c4889c1673"), mana_cost: "{2}{B}", type_line: "Legendary Creature — Skeleton Rogue", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_4608::RULES.keywords, deck_limit: authored::card_4608::RULES.deck_limit }, effect: authored::card_4608::RULES.checked_effect(), triggers: authored::card_4608::RULES.triggers, activated: authored::card_4608::RULES.activated, enters: authored::card_4608::RULES.enters, entry: authored::card_4608::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 2 }, rules: &authored::card_4608::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Titanbones, Towering Heart", oracle_id: Some("510bccb1-61cd-49f1-a54a-80352653d1e3"), mana_cost: "{3}{G}", type_line: "Legendary Creature — Skeleton Druid", oracle_keywords: &["Reach"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_4612::RULES.keywords, deck_limit: authored::card_4612::RULES.deck_limit }, effect: authored::card_4612::RULES.checked_effect(), triggers: authored::card_4612::RULES.triggers, activated: authored::card_4612::RULES.activated, enters: authored::card_4612::RULES.enters, entry: authored::card_4612::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 3 }, rules: &authored::card_4612::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Tomik, Izzet Sparkmage", oracle_id: Some("16604fee-cd8d-41e4-8269-3c7ff756613d"), mana_cost: "{1}{R}", type_line: "Legendary Creature — Human Wizard", oracle_keywords: &["Prowess"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_4627::RULES.keywords, deck_limit: authored::card_4627::RULES.deck_limit }, effect: authored::card_4627::RULES.checked_effect(), triggers: authored::card_4627::RULES.triggers, activated: authored::card_4627::RULES.activated, enters: authored::card_4627::RULES.enters, entry: authored::card_4627::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 1 }, rules: &authored::card_4627::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Tomik, Orzhov Lawmage", oracle_id: Some("7a501f7e-eec8-45f7-9ac3-483fd8f5ca5e"), mana_cost: "{1}{W}", type_line: "Legendary Creature — Human Advisor", oracle_keywords: &["Flying"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_4628::RULES.keywords, deck_limit: authored::card_4628::RULES.deck_limit }, effect: authored::card_4628::RULES.checked_effect(), triggers: authored::card_4628::RULES.triggers, activated: authored::card_4628::RULES.activated, enters: authored::card_4628::RULES.enters, entry: authored::card_4628::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_4628::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Transformative Commons", oracle_id: Some("8ec56e2a-380f-4de7-9fb7-52de803008fc"), mana_cost: "", type_line: "Land", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4658::RULES.keywords, deck_limit: authored::card_4658::RULES.deck_limit }, effect: authored::card_4658::RULES.checked_effect(), triggers: authored::card_4658::RULES.triggers, activated: authored::card_4658::RULES.activated, enters: authored::card_4658::RULES.enters, entry: authored::card_4658::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_4658::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Traxos, Academy Guardian", oracle_id: Some("7b9a574f-6d56-421b-8fbd-94da1e6e3ce1"), mana_cost: "{3}{U}", type_line: "Legendary Artifact Creature — Dragon Construct", oracle_keywords: &["Flying", "Prowess", "Vigilance"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_4667::RULES.keywords, deck_limit: authored::card_4667::RULES.deck_limit }, effect: authored::card_4667::RULES.checked_effect(), triggers: authored::card_4667::RULES.triggers, activated: authored::card_4667::RULES.activated, enters: authored::card_4667::RULES.enters, entry: authored::card_4667::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_4667::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Traxos, Scourge Eternal", oracle_id: Some("acf4abde-4034-4fcf-b531-78c935aecd8c"), mana_cost: "{4}", type_line: "Legendary Artifact Creature — Dragon Construct", oracle_keywords: &["Trample"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_4668::RULES.keywords, deck_limit: authored::card_4668::RULES.deck_limit }, effect: authored::card_4668::RULES.checked_effect(), triggers: authored::card_4668::RULES.triggers, activated: authored::card_4668::RULES.activated, enters: authored::card_4668::RULES.enters, entry: authored::card_4668::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 4 }, rules: &authored::card_4668::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Twinned Vision", oracle_id: Some("fa1cc78f-26bc-4d5d-973b-7cb7d7490973"), mana_cost: "{1}{U/R}", type_line: "Instant", oracle_keywords: &["Flashback"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4724::RULES.keywords, deck_limit: authored::card_4724::RULES.deck_limit }, effect: authored::card_4724::RULES.checked_effect(), triggers: authored::card_4724::RULES.triggers, activated: authored::card_4724::RULES.activated, enters: authored::card_4724::RULES.enters, entry: authored::card_4724::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 1 }, rules: &authored::card_4724::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Twisted Fates", oracle_id: Some("3ce30a19-e716-478f-be89-e2d95753cfed"), mana_cost: "{2}{W}{W}{B}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4726::RULES.keywords, deck_limit: authored::card_4726::RULES.deck_limit }, effect: authored::card_4726::RULES.checked_effect(), triggers: authored::card_4726::RULES.triggers, activated: authored::card_4726::RULES.activated, enters: authored::card_4726::RULES.enters, entry: authored::card_4726::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [2, 0, 1, 0, 0, 0], generic: 2 }, rules: &authored::card_4726::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Uldaros Theorix", oracle_id: Some("020d561b-2e5d-439e-9e6a-318d23e73542"), mana_cost: "{3}{U}{B}{B}", type_line: "Legendary Creature — Elder Sphinx", oracle_keywords: &["Flying"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_4739::RULES.keywords, deck_limit: authored::card_4739::RULES.deck_limit }, effect: authored::card_4739::RULES.checked_effect(), triggers: authored::card_4739::RULES.triggers, activated: authored::card_4739::RULES.activated, enters: authored::card_4739::RULES.enters, entry: authored::card_4739::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 2, 0, 0, 0], generic: 3 }, rules: &authored::card_4739::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Undulating Witness", oracle_id: Some("77b684b8-227b-40de-80b4-15d0e70914f4"), mana_cost: "{4}{U}", type_line: "Creature — Serpent", oracle_keywords: &["Flying", "Landcycling", "Basic landcycling", "Typecycling", "Cycling"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4772::RULES.keywords, deck_limit: authored::card_4772::RULES.deck_limit }, effect: authored::card_4772::RULES.checked_effect(), triggers: authored::card_4772::RULES.triggers, activated: authored::card_4772::RULES.activated, enters: authored::card_4772::RULES.enters, entry: authored::card_4772::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 4 }, rules: &authored::card_4772::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Unflinching Hortimancer", oracle_id: Some("7cd0bd74-2463-4e9d-9802-69ac4ef6d543"), mana_cost: "{1}{W}", type_line: "Creature — Human Cleric", oracle_keywords: &["Ward"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4780::RULES.keywords, deck_limit: authored::card_4780::RULES.deck_limit }, effect: authored::card_4780::RULES.checked_effect(), triggers: authored::card_4780::RULES.triggers, activated: authored::card_4780::RULES.activated, enters: authored::card_4780::RULES.enters, entry: authored::card_4780::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_4780::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Unsummon", oracle_id: Some("837182db-1bf3-4a2c-bd01-1af9d9873561"), mana_cost: "{U}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4804::RULES.keywords, deck_limit: authored::card_4804::RULES.deck_limit }, effect: authored::card_4804::RULES.checked_effect(), triggers: authored::card_4804::RULES.triggers, activated: authored::card_4804::RULES.activated, enters: authored::card_4804::RULES.enters, entry: authored::card_4804::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_4804::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Variable Chaser", oracle_id: Some("67c603cc-ad66-4a1c-8386-5901c9c01bfb"), mana_cost: "{2}{U}", type_line: "Creature — Human Wizard", oracle_keywords: &["Flying", "Prowess", "Prepared"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4850::RULES.keywords, deck_limit: authored::card_4850::RULES.deck_limit }, effect: authored::card_4850::RULES.checked_effect(), triggers: authored::card_4850::RULES.triggers, activated: authored::card_4850::RULES.activated, enters: authored::card_4850::RULES.enters, entry: authored::card_4850::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_4850::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Verdant Kraken", oracle_id: Some("83c61083-b357-4775-a855-5a9013cc2bdf"), mana_cost: "{4}{G}{G}{G}", type_line: "Creature — Plant Kraken", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4874::RULES.keywords, deck_limit: authored::card_4874::RULES.deck_limit }, effect: authored::card_4874::RULES.checked_effect(), triggers: authored::card_4874::RULES.triggers, activated: authored::card_4874::RULES.activated, enters: authored::card_4874::RULES.enters, entry: authored::card_4874::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 3, 0], generic: 4 }, rules: &authored::card_4874::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Vigorbloom Annex", oracle_id: Some("fcb35d42-bcd2-427c-9640-ff3602b79912"), mana_cost: "", type_line: "Land", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4889::RULES.keywords, deck_limit: authored::card_4889::RULES.deck_limit }, effect: authored::card_4889::RULES.checked_effect(), triggers: authored::card_4889::RULES.triggers, activated: authored::card_4889::RULES.activated, enters: authored::card_4889::RULES.enters, entry: authored::card_4889::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_4889::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Vigorbloom Charm", oracle_id: Some("826f17d1-3806-4579-8b1b-2082b7b03d69"), mana_cost: "{G}{W}", type_line: "Instant", oracle_keywords: &["Fight"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4890::RULES.keywords, deck_limit: authored::card_4890::RULES.deck_limit }, effect: authored::card_4890::RULES.checked_effect(), triggers: authored::card_4890::RULES.triggers, activated: authored::card_4890::RULES.activated, enters: authored::card_4890::RULES.enters, entry: authored::card_4890::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [1, 0, 0, 0, 1, 0], generic: 0 }, rules: &authored::card_4890::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Vigorbloom Vanguard", oracle_id: Some("bd416426-d037-45e3-9647-816bf982cbea"), mana_cost: "{1}{G/W}", type_line: "Creature — Troll Druid", oracle_keywords: &["Prepared"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4891::RULES.keywords, deck_limit: authored::card_4891::RULES.deck_limit }, effect: authored::card_4891::RULES.checked_effect(), triggers: authored::card_4891::RULES.triggers, activated: authored::card_4891::RULES.activated, enters: authored::card_4891::RULES.enters, entry: authored::card_4891::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 1 }, rules: &authored::card_4891::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Vindictive Triumph", oracle_id: Some("d6b478d1-5015-49ba-b1aa-cbc2b08107a6"), mana_cost: "{W}{B}{B}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4898::RULES.keywords, deck_limit: authored::card_4898::RULES.deck_limit }, effect: authored::card_4898::RULES.checked_effect(), triggers: authored::card_4898::RULES.triggers, activated: authored::card_4898::RULES.activated, enters: authored::card_4898::RULES.enters, entry: authored::card_4898::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [1, 0, 2, 0, 0, 0], generic: 0 }, rules: &authored::card_4898::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Vinelasher Adept", oracle_id: Some("b5918fb0-c33f-4e33-ac6d-8f94723d4296"), mana_cost: "{4}{G}{G}", type_line: "Creature — Rhino Soldier", oracle_keywords: &["Reach", "Landcycling", "Basic landcycling", "Typecycling", "Cycling"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4901::RULES.keywords, deck_limit: authored::card_4901::RULES.deck_limit }, effect: authored::card_4901::RULES.checked_effect(), triggers: authored::card_4901::RULES.triggers, activated: authored::card_4901::RULES.activated, enters: authored::card_4901::RULES.enters, entry: authored::card_4901::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 2, 0], generic: 4 }, rules: &authored::card_4901::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Violent Echoes", oracle_id: Some("e8ecec61-cc47-4051-a599-0e7470764488"), mana_cost: "{2}{R}{R}", type_line: "Instant", oracle_keywords: &["Surveil", "Empower Jace"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4903::RULES.keywords, deck_limit: authored::card_4903::RULES.deck_limit }, effect: authored::card_4903::RULES.checked_effect(), triggers: authored::card_4903::RULES.triggers, activated: authored::card_4903::RULES.activated, enters: authored::card_4903::RULES.enters, entry: authored::card_4903::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 0, 2, 0, 0], generic: 2 }, rules: &authored::card_4903::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Void Extrapolator", oracle_id: Some("517f84b5-cf99-410d-96ea-910740231dac"), mana_cost: "{1}{B}", type_line: "Creature — Aetherborn Warlock", oracle_keywords: &["Threshold", "Prepared", "Mill"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4928::RULES.keywords, deck_limit: authored::card_4928::RULES.deck_limit }, effect: authored::card_4928::RULES.checked_effect(), triggers: authored::card_4928::RULES.triggers, activated: authored::card_4928::RULES.activated, enters: authored::card_4928::RULES.enters, entry: authored::card_4928::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 1 }, rules: &authored::card_4928::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Vraska's Final Mercy", oracle_id: Some("7799413d-e313-4a05-ad28-f4c926c26d28"), mana_cost: "{B}{B}", type_line: "Sorcery", oracle_keywords: &["Surveil", "Empower Jace"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4945::RULES.keywords, deck_limit: authored::card_4945::RULES.deck_limit }, effect: authored::card_4945::RULES.checked_effect(), triggers: authored::card_4945::RULES.triggers, activated: authored::card_4945::RULES.activated, enters: authored::card_4945::RULES.enters, entry: authored::card_4945::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 2, 0, 0, 0], generic: 0 }, rules: &authored::card_4945::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Vraska, Soul of Stone", oracle_id: Some("cad63b31-667b-4518-bc49-be4ffb82bc63"), mana_cost: "{U}{R}{W}", type_line: "Legendary Creature — Gorgon Wizard", oracle_keywords: &["Treasure"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_4946::RULES.keywords, deck_limit: authored::card_4946::RULES.deck_limit }, effect: authored::card_4946::RULES.checked_effect(), triggers: authored::card_4946::RULES.triggers, activated: authored::card_4946::RULES.activated, enters: authored::card_4946::RULES.enters, entry: authored::card_4946::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 1, 0, 1, 0, 0], generic: 0 }, rules: &authored::card_4946::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Vraska, the Cutting Glare", oracle_id: Some("580de511-863c-4b7a-9d2e-fcdf6e12d54b"), mana_cost: "{B}{B}{G}", type_line: "Legendary Creature — Gorgon Assassin", oracle_keywords: &["Treasure", "Deathtouch"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_4947::RULES.keywords, deck_limit: authored::card_4947::RULES.deck_limit }, effect: authored::card_4947::RULES.checked_effect(), triggers: authored::card_4947::RULES.triggers, activated: authored::card_4947::RULES.activated, enters: authored::card_4947::RULES.enters, entry: authored::card_4947::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 2, 0, 1, 0], generic: 0 }, rules: &authored::card_4947::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Warrior's Blades", oracle_id: Some("f47419a5-b975-4934-a987-ea064a0c7c1a"), mana_cost: "{2}{R}{W}", type_line: "Artifact — Equipment", oracle_keywords: &["Equip"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4985::RULES.keywords, deck_limit: authored::card_4985::RULES.deck_limit }, effect: authored::card_4985::RULES.checked_effect(), triggers: authored::card_4985::RULES.triggers, activated: authored::card_4985::RULES.activated, enters: authored::card_4985::RULES.enters, entry: authored::card_4985::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [1, 0, 0, 1, 0, 0], generic: 2 }, rules: &authored::card_4985::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Way of the Cryomancer", oracle_id: Some("7d32c63d-aa9e-4f5d-a3e6-51377a786009"), mana_cost: "{2}{U}", type_line: "Legendary Enchantment", oracle_keywords: &["Surveil", "Empower Jace"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_5007::RULES.keywords, deck_limit: authored::card_5007::RULES.deck_limit }, effect: authored::card_5007::RULES.checked_effect(), triggers: authored::card_5007::RULES.triggers, activated: authored::card_5007::RULES.activated, enters: authored::card_5007::RULES.enters, entry: authored::card_5007::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_5007::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Way of the Deathbringer", oracle_id: Some("54775625-bf44-4035-969e-7e70578c8b98"), mana_cost: "{2}{B}", type_line: "Legendary Enchantment", oracle_keywords: &["Surveil", "Empower Jace"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_5008::RULES.keywords, deck_limit: authored::card_5008::RULES.deck_limit }, effect: authored::card_5008::RULES.checked_effect(), triggers: authored::card_5008::RULES.triggers, activated: authored::card_5008::RULES.activated, enters: authored::card_5008::RULES.enters, entry: authored::card_5008::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 2 }, rules: &authored::card_5008::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Way of the Healer", oracle_id: Some("cb27dc83-85ee-4f54-b032-41cd1806d3ac"), mana_cost: "{3}{W}", type_line: "Legendary Enchantment", oracle_keywords: &["Surveil", "Empower Jace"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_5009::RULES.keywords, deck_limit: authored::card_5009::RULES.deck_limit }, effect: authored::card_5009::RULES.checked_effect(), triggers: authored::card_5009::RULES.triggers, activated: authored::card_5009::RULES.activated, enters: authored::card_5009::RULES.enters, entry: authored::card_5009::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_5009::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Way of the Mentor", oracle_id: Some("ce924285-f4c5-44e6-909b-f5f7d302ebaf"), mana_cost: "{2}{W}", type_line: "Legendary Enchantment", oracle_keywords: &["Surveil", "Empower Jace"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_5010::RULES.keywords, deck_limit: authored::card_5010::RULES.deck_limit }, effect: authored::card_5010::RULES.checked_effect(), triggers: authored::card_5010::RULES.triggers, activated: authored::card_5010::RULES.activated, enters: authored::card_5010::RULES.enters, entry: authored::card_5010::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_5010::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Way of the Mind Sculptor", oracle_id: Some("94be2e86-ba0e-4b8d-9b9c-114adacb26db"), mana_cost: "{4}{U}", type_line: "Legendary Enchantment", oracle_keywords: &["Surveil", "Empower Jace"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_5011::RULES.keywords, deck_limit: authored::card_5011::RULES.deck_limit }, effect: authored::card_5011::RULES.checked_effect(), triggers: authored::card_5011::RULES.triggers, activated: authored::card_5011::RULES.activated, enters: authored::card_5011::RULES.enters, entry: authored::card_5011::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 4 }, rules: &authored::card_5011::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Way of the Necromancer", oracle_id: Some("f3087daa-d2ff-4227-abff-3398da6297af"), mana_cost: "{1}{B}", type_line: "Legendary Enchantment", oracle_keywords: &["Surveil", "Empower Jace"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_5012::RULES.keywords, deck_limit: authored::card_5012::RULES.deck_limit }, effect: authored::card_5012::RULES.checked_effect(), triggers: authored::card_5012::RULES.triggers, activated: authored::card_5012::RULES.activated, enters: authored::card_5012::RULES.enters, entry: authored::card_5012::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 1 }, rules: &authored::card_5012::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Way of the Paradox", oracle_id: Some("76771cd1-6b74-49bb-9421-8845fe344567"), mana_cost: "{2}{G}", type_line: "Legendary Enchantment", oracle_keywords: &["Surveil", "Empower Jace"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_5013::RULES.keywords, deck_limit: authored::card_5013::RULES.deck_limit }, effect: authored::card_5013::RULES.checked_effect(), triggers: authored::card_5013::RULES.triggers, activated: authored::card_5013::RULES.activated, enters: authored::card_5013::RULES.enters, entry: authored::card_5013::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 2 }, rules: &authored::card_5013::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Way of the Pyromancer", oracle_id: Some("68d3f547-674a-4d8a-b02f-01bc2f161916"), mana_cost: "{1}{R}", type_line: "Legendary Enchantment", oracle_keywords: &["Surveil", "Empower Jace"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_5014::RULES.keywords, deck_limit: authored::card_5014::RULES.deck_limit }, effect: authored::card_5014::RULES.checked_effect(), triggers: authored::card_5014::RULES.triggers, activated: authored::card_5014::RULES.activated, enters: authored::card_5014::RULES.enters, entry: authored::card_5014::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 1 }, rules: &authored::card_5014::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Way of the Warlord", oracle_id: Some("10721d62-dc68-43b4-bfa2-dcb8cf8ea980"), mana_cost: "{2}{R}", type_line: "Legendary Enchantment", oracle_keywords: &["Surveil", "Empower Jace"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_5015::RULES.keywords, deck_limit: authored::card_5015::RULES.deck_limit }, effect: authored::card_5015::RULES.checked_effect(), triggers: authored::card_5015::RULES.triggers, activated: authored::card_5015::RULES.activated, enters: authored::card_5015::RULES.enters, entry: authored::card_5015::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 2 }, rules: &authored::card_5015::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Way of the Wildspeaker", oracle_id: Some("501580b9-692c-4804-be34-012a25ba8adf"), mana_cost: "{4}{G}", type_line: "Legendary Enchantment", oracle_keywords: &["Surveil", "Empower Jace"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_5016::RULES.keywords, deck_limit: authored::card_5016::RULES.deck_limit }, effect: authored::card_5016::RULES.checked_effect(), triggers: authored::card_5016::RULES.triggers, activated: authored::card_5016::RULES.activated, enters: authored::card_5016::RULES.enters, entry: authored::card_5016::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 4 }, rules: &authored::card_5016::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Whiplash Wordsmith", oracle_id: Some("e1ffb884-a89e-4e2a-9beb-8bd5838b6a94"), mana_cost: "{3}{B/R}", type_line: "Creature — Vampire Sorcerer", oracle_keywords: &["Prepared"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_5037::RULES.keywords, deck_limit: authored::card_5037::RULES.deck_limit }, effect: authored::card_5037::RULES.checked_effect(), triggers: authored::card_5037::RULES.triggers, activated: authored::card_5037::RULES.activated, enters: authored::card_5037::RULES.enters, entry: authored::card_5037::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 3 }, rules: &authored::card_5037::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Winter, Team Player", oracle_id: Some("254f133a-eee5-48b9-b19f-110555e49300"), mana_cost: "{4}{R}", type_line: "Legendary Creature — Human Warrior", oracle_keywords: &["Convoke"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_5086::RULES.keywords, deck_limit: authored::card_5086::RULES.deck_limit }, effect: authored::card_5086::RULES.checked_effect(), triggers: authored::card_5086::RULES.triggers, activated: authored::card_5086::RULES.activated, enters: authored::card_5086::RULES.enters, entry: authored::card_5086::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 4 }, rules: &authored::card_5086::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Winter, Tormented Loner", oracle_id: Some("84eaac2e-74ce-4a16-9faa-409df7c58eb3"), mana_cost: "{2}{B}", type_line: "Legendary Creature — Human Warlock", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_5087::RULES.keywords, deck_limit: authored::card_5087::RULES.deck_limit }, effect: authored::card_5087::RULES.checked_effect(), triggers: authored::card_5087::RULES.triggers, activated: authored::card_5087::RULES.activated, enters: authored::card_5087::RULES.enters, entry: authored::card_5087::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 2 }, rules: &authored::card_5087::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Woodwork Prodigy", oracle_id: Some("f3ed157f-a09b-405f-aec4-0e7a81d80fc8"), mana_cost: "{2}{R/G}", type_line: "Creature — Cat Druid", oracle_keywords: &["Prepared"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_5111::RULES.keywords, deck_limit: authored::card_5111::RULES.deck_limit }, effect: authored::card_5111::RULES.checked_effect(), triggers: authored::card_5111::RULES.triggers, activated: authored::card_5111::RULES.activated, enters: authored::card_5111::RULES.enters, entry: authored::card_5111::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 2 }, rules: &authored::card_5111::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Wrath of the Bloodmane", oracle_id: Some("c1994d6a-984d-4b75-9198-1f1822eab6eb"), mana_cost: "{2}{R}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_5120::RULES.keywords, deck_limit: authored::card_5120::RULES.deck_limit }, effect: authored::card_5120::RULES.checked_effect(), triggers: authored::card_5120::RULES.triggers, activated: authored::card_5120::RULES.activated, enters: authored::card_5120::RULES.enters, entry: authored::card_5120::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 2 }, rules: &authored::card_5120::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Wrecking Gecko", oracle_id: Some("4a9e0294-c967-4e90-b776-6d3195034a23"), mana_cost: "{4}{G}", type_line: "Artifact Creature — Lizard Construct", oracle_keywords: &["Ward"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_5123::RULES.keywords, deck_limit: authored::card_5123::RULES.deck_limit }, effect: authored::card_5123::RULES.checked_effect(), triggers: authored::card_5123::RULES.triggers, activated: authored::card_5123::RULES.activated, enters: authored::card_5123::RULES.enters, entry: authored::card_5123::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 4 }, rules: &authored::card_5123::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Yargle, Glutton of Urborg", oracle_id: Some("ed66cd31-958f-4b28-82a3-e04acc819afc"), mana_cost: "{4}{B}", type_line: "Legendary Creature — Frog Spirit", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_5131::RULES.keywords, deck_limit: authored::card_5131::RULES.deck_limit }, effect: authored::card_5131::RULES.checked_effect(), triggers: authored::card_5131::RULES.triggers, activated: authored::card_5131::RULES.activated, enters: authored::card_5131::RULES.enters, entry: authored::card_5131::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 4 }, rules: &authored::card_5131::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Yargle, Goliath of Otaria", oracle_id: Some("014d0bcd-b80f-4c43-8f00-f4c31e8f3370"), mana_cost: "{4}{U}", type_line: "Legendary Creature — Frog Spirit", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_5132::RULES.keywords, deck_limit: authored::card_5132::RULES.deck_limit }, effect: authored::card_5132::RULES.checked_effect(), triggers: authored::card_5132::RULES.triggers, activated: authored::card_5132::RULES.activated, enters: authored::card_5132::RULES.enters, entry: authored::card_5132::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 4 }, rules: &authored::card_5132::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Yoshimaru, Beloved Companion", oracle_id: Some("c00a0be5-d9a9-4263-a87e-7bf2d128ab3a"), mana_cost: "{2}{W}", type_line: "Legendary Creature — Dog", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_5140::RULES.keywords, deck_limit: authored::card_5140::RULES.deck_limit }, effect: authored::card_5140::RULES.checked_effect(), triggers: authored::card_5140::RULES.triggers, activated: authored::card_5140::RULES.activated, enters: authored::card_5140::RULES.enters, entry: authored::card_5140::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_5140::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Yoshimaru, Scrappy Stray", oracle_id: Some("c87da757-3daa-4874-a1e4-be0aa2adcb28"), mana_cost: "{1}{G}", type_line: "Legendary Creature — Dog", oracle_keywords: &["Fight"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_5141::RULES.keywords, deck_limit: authored::card_5141::RULES.deck_limit }, effect: authored::card_5141::RULES.checked_effect(), triggers: authored::card_5141::RULES.triggers, activated: authored::card_5141::RULES.activated, enters: authored::card_5141::RULES.enters, entry: authored::card_5141::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 1 }, rules: &authored::card_5141::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Your Fate Ends Here", oracle_id: Some("28a362c2-0c99-48e0-b9cb-485a6c1f350b"), mana_cost: "{2}{W}", type_line: "Instant", oracle_keywords: &["Surveil"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_5143::RULES.keywords, deck_limit: authored::card_5143::RULES.deck_limit }, effect: authored::card_5143::RULES.checked_effect(), triggers: authored::card_5143::RULES.triggers, activated: authored::card_5143::RULES.activated, enters: authored::card_5143::RULES.enters, entry: authored::card_5143::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_5143::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Yuriko, Blade of the Mighty", oracle_id: Some("f89b0143-4b19-4ab1-a96b-356b21e435ef"), mana_cost: "{3}{W}", type_line: "Legendary Creature — Human Samurai", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_5147::RULES.keywords, deck_limit: authored::card_5147::RULES.deck_limit }, effect: authored::card_5147::RULES.checked_effect(), triggers: authored::card_5147::RULES.triggers, activated: authored::card_5147::RULES.activated, enters: authored::card_5147::RULES.enters, entry: authored::card_5147::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_5147::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Yuriko, Hope from the Shadows", oracle_id: Some("983f4fde-ab04-44f0-a294-c3b472de0b19"), mana_cost: "{U}", type_line: "Legendary Creature — Human Ninja", oracle_keywords: &["Surveil", "Flash"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_5148::RULES.keywords, deck_limit: authored::card_5148::RULES.deck_limit }, effect: authored::card_5148::RULES.checked_effect(), triggers: authored::card_5148::RULES.triggers, activated: authored::card_5148::RULES.activated, enters: authored::card_5148::RULES.enters, entry: authored::card_5148::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_5148::RULES },
];
}
pub use cards::ENTRIES;
