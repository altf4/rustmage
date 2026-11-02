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
pub(super) mod card_0010 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msc/shock.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0130 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msc/alien_symbiosis.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0263 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msc/aunt_may.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0519 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msc/bovine_intervention.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0705 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msc/chameleon_master_of_disguise.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0756 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msc/city_pigeon.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0814 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msc/common_crook.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0866 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msc/costume_closet.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0891 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msc/crimson_operative.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0949 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msc/daily_bugle_reporters.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1114 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msc/doc_ock_s_henchmen.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1116 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msc/doc_ock_sinister_scientist.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1268 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msc/eerie_gravestone.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1391 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msc/evolving_wilds.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1548 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msc/flash_thompson_spider_fan.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1571 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msc/flying_octobot.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1666 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msc/gallant_citizen.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1751 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msc/gilded_lotus.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2014 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msc/hero_in_training.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2017 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msc/heroes_hangout.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2054 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msc/hobgoblin_mantled_marauder.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2171 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msc/inner_demons_gangsters.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2586 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msc/lurking_lizards.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2632 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msc/make_your_move.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2675 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msc/masked_meower.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2731 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msc/meteor_golem.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2804 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msc/mob_lookout.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2899 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msc/mystic_monastery.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3086 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msc/pacifism.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3108 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msc/patchwork_banner.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3267 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msc/prowler_clawed_thief.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3280 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msc/pumpkin_bombardment.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3513 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msc/rhino_s_rampage.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3565 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msc/robotics_mastery.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3568 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msc/rocket_powered_goblin_glider.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3580 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msc/romantic_rendezvous.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3733 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msc/scout_the_city.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3757 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msc/secluded_courtyard.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3782 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msc/selfless_police_captain.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3958 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msc/solemn_simulacrum.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3997 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msc/spectacular_tactics.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4023 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msc/spider_gwen_free_spirit.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4032 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msc/spider_man_web_slinger.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4033 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msc/spider_mobile.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4035 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msc/spider_rex_daring_dino.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4039 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msc/spider_uk.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4106 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msc/starling_aerial_ally.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4131 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msc/stegron_the_dinosaur_man.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4203 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msc/sudden_strike.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4261 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msc/superior_foes_of_spider_man.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4287 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msc/swarm_being_of_bees.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4292 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msc/swiftfoot_boots.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4347 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msc/taxi_driver.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4390 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msc/terramorphic_expanse.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4394 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msc/terrific_team_up.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4428 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msc/the_clone_saga.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4503 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msc/the_spot_s_portal.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4530 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msc/think_twice.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4590 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msc/thwip.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4625 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msc/tombstone_career_criminal.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4799 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msc/unstable_experiment.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4836 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msc/valorous_stance.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4870 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msc/venom_evil_unleashed.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4871 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msc/venomized_cat.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_5049 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msc/whoosh.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_5058 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msc/wild_pack_squad.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_5090 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msc/wisecrack.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_5101 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msc/withering_torment.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
}
#[rustfmt::skip]
pub static ENTRIES: [CatalogueEntry; 68] = [
CatalogueEntry { definition: Definition { info: CardInfo { name: "Shock", oracle_id: Some("a9d288b8-cdc1-4e55-a0c9-d6edfc95e65d"), mana_cost: "{R}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0010::RULES.keywords, deck_limit: authored::card_0010::RULES.deck_limit }, effect: authored::card_0010::RULES.checked_effect(), triggers: authored::card_0010::RULES.triggers, activated: authored::card_0010::RULES.activated, enters: authored::card_0010::RULES.enters, entry: authored::card_0010::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 0 }, rules: &authored::card_0010::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Alien Symbiosis", oracle_id: Some("19504e19-2886-4d28-a813-b798a9a446a2"), mana_cost: "{1}{B}", type_line: "Enchantment — Aura", oracle_keywords: &["Enchant"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_0130::RULES.keywords, deck_limit: authored::card_0130::RULES.deck_limit }, effect: authored::card_0130::RULES.effect, triggers: authored::card_0130::RULES.triggers, activated: authored::card_0130::RULES.activated, enters: authored::card_0130::RULES.enters, entry: authored::card_0130::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 1 }, rules: &authored::card_0130::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Aunt May", oracle_id: Some("bdb769b5-c861-4ec1-ad6b-e2a56ca05fd1"), mana_cost: "{W}", type_line: "Legendary Creature — Human Citizen", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_0263::RULES.keywords, deck_limit: authored::card_0263::RULES.deck_limit }, effect: authored::card_0263::RULES.checked_effect(), triggers: authored::card_0263::RULES.triggers, activated: authored::card_0263::RULES.activated, enters: authored::card_0263::RULES.enters, entry: authored::card_0263::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_0263::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Bovine Intervention", oracle_id: Some("7402d98f-6d58-4ff2-a0db-9eec4417a75f"), mana_cost: "{1}{W}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0519::RULES.keywords, deck_limit: authored::card_0519::RULES.deck_limit }, effect: authored::card_0519::RULES.checked_effect(), triggers: authored::card_0519::RULES.triggers, activated: authored::card_0519::RULES.activated, enters: authored::card_0519::RULES.enters, entry: authored::card_0519::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_0519::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Chameleon, Master of Disguise", oracle_id: Some("1ce239f2-79ad-4a30-8e8f-f985045f78dc"), mana_cost: "{3}{U}", type_line: "Legendary Creature — Human Shapeshifter Villain", oracle_keywords: &["Mayhem"], standard_legal: true, legendary: true, implemented: false, keywords: authored::card_0705::RULES.keywords, deck_limit: authored::card_0705::RULES.deck_limit }, effect: authored::card_0705::RULES.effect, triggers: authored::card_0705::RULES.triggers, activated: authored::card_0705::RULES.activated, enters: authored::card_0705::RULES.enters, entry: authored::card_0705::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_0705::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "City Pigeon", oracle_id: Some("ac8cae63-270c-4f73-b29b-50f8f2395fd9"), mana_cost: "{W}", type_line: "Creature — Bird", oracle_keywords: &["Flying", "Food"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_0756::RULES.keywords, deck_limit: authored::card_0756::RULES.deck_limit }, effect: authored::card_0756::RULES.effect, triggers: authored::card_0756::RULES.triggers, activated: authored::card_0756::RULES.activated, enters: authored::card_0756::RULES.enters, entry: authored::card_0756::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_0756::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Common Crook", oracle_id: Some("ebcbfdba-34c7-40eb-91a9-d306ff5ae8b9"), mana_cost: "{1}{B}", type_line: "Creature — Human Rogue Villain", oracle_keywords: &["Treasure"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0814::RULES.keywords, deck_limit: authored::card_0814::RULES.deck_limit }, effect: authored::card_0814::RULES.checked_effect(), triggers: authored::card_0814::RULES.triggers, activated: authored::card_0814::RULES.activated, enters: authored::card_0814::RULES.enters, entry: authored::card_0814::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 1 }, rules: &authored::card_0814::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Costume Closet", oracle_id: Some("072d05c4-e0d7-47af-b2a9-16b90810fd30"), mana_cost: "{1}{W}", type_line: "Artifact", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_0866::RULES.keywords, deck_limit: authored::card_0866::RULES.deck_limit }, effect: authored::card_0866::RULES.effect, triggers: authored::card_0866::RULES.triggers, activated: authored::card_0866::RULES.activated, enters: authored::card_0866::RULES.enters, entry: authored::card_0866::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_0866::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Crimson Operative", oracle_id: Some("d8863742-6c1c-415a-ad27-7def6d119c6a"), mana_cost: "{3}{R}", type_line: "Artifact Creature — Human Villain", oracle_keywords: &["Prowess"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0891::RULES.keywords, deck_limit: authored::card_0891::RULES.deck_limit }, effect: authored::card_0891::RULES.checked_effect(), triggers: authored::card_0891::RULES.triggers, activated: authored::card_0891::RULES.activated, enters: authored::card_0891::RULES.enters, entry: authored::card_0891::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 3 }, rules: &authored::card_0891::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Daily Bugle Reporters", oracle_id: Some("2e677ded-7af3-4af0-81df-b70d6ee616ec"), mana_cost: "{3}{W}", type_line: "Creature — Human Citizen", oracle_keywords: &["Puff Piece", "Investigative Journalism"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_0949::RULES.keywords, deck_limit: authored::card_0949::RULES.deck_limit }, effect: authored::card_0949::RULES.effect, triggers: authored::card_0949::RULES.triggers, activated: authored::card_0949::RULES.activated, enters: authored::card_0949::RULES.enters, entry: authored::card_0949::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_0949::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Doc Ock's Henchmen", oracle_id: Some("ae91c16c-5267-4957-b36c-5aa787fa6b5e"), mana_cost: "{2}{U}", type_line: "Creature — Human Villain", oracle_keywords: &["Flash", "Connive"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1114::RULES.keywords, deck_limit: authored::card_1114::RULES.deck_limit }, effect: authored::card_1114::RULES.checked_effect(), triggers: authored::card_1114::RULES.triggers, activated: authored::card_1114::RULES.activated, enters: authored::card_1114::RULES.enters, entry: authored::card_1114::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_1114::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Doc Ock, Sinister Scientist", oracle_id: Some("dbaa70e6-26de-4ce9-9543-e652cbe387c1"), mana_cost: "{4}{U}", type_line: "Legendary Creature — Human Scientist Villain", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_1116::RULES.keywords, deck_limit: authored::card_1116::RULES.deck_limit }, effect: authored::card_1116::RULES.checked_effect(), triggers: authored::card_1116::RULES.triggers, activated: authored::card_1116::RULES.activated, enters: authored::card_1116::RULES.enters, entry: authored::card_1116::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 4 }, rules: &authored::card_1116::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Eerie Gravestone", oracle_id: Some("cbf5bd0a-20ef-4dce-9f70-13339d438ff7"), mana_cost: "{2}", type_line: "Artifact", oracle_keywords: &["Mill"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_1268::RULES.keywords, deck_limit: authored::card_1268::RULES.deck_limit }, effect: authored::card_1268::RULES.effect, triggers: authored::card_1268::RULES.triggers, activated: authored::card_1268::RULES.activated, enters: authored::card_1268::RULES.enters, entry: authored::card_1268::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_1268::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Evolving Wilds", oracle_id: Some("a75445d3-1303-4bb5-89ad-26ea93fecd48"), mana_cost: "", type_line: "Land", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1391::RULES.keywords, deck_limit: authored::card_1391::RULES.deck_limit }, effect: authored::card_1391::RULES.checked_effect(), triggers: authored::card_1391::RULES.triggers, activated: authored::card_1391::RULES.activated, enters: authored::card_1391::RULES.enters, entry: authored::card_1391::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_1391::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Flash Thompson, Spider-Fan", oracle_id: Some("f536a518-160e-4267-89da-3d34accc6d2c"), mana_cost: "{1}{W}", type_line: "Legendary Creature — Human Citizen", oracle_keywords: &["Hero Worship", "Heckle", "Flash"], standard_legal: true, legendary: true, implemented: false, keywords: authored::card_1548::RULES.keywords, deck_limit: authored::card_1548::RULES.deck_limit }, effect: authored::card_1548::RULES.effect, triggers: authored::card_1548::RULES.triggers, activated: authored::card_1548::RULES.activated, enters: authored::card_1548::RULES.enters, entry: authored::card_1548::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_1548::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Flying Octobot", oracle_id: Some("5ef8925c-5f43-49ca-83f0-e2ca8e655600"), mana_cost: "{1}{U}", type_line: "Artifact Creature — Robot Villain", oracle_keywords: &["Flying"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1571::RULES.keywords, deck_limit: authored::card_1571::RULES.deck_limit }, effect: authored::card_1571::RULES.checked_effect(), triggers: authored::card_1571::RULES.triggers, activated: authored::card_1571::RULES.activated, enters: authored::card_1571::RULES.enters, entry: authored::card_1571::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_1571::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Gallant Citizen", oracle_id: Some("2fcb1c1c-6157-4955-b899-e8a8b337d188"), mana_cost: "{G/W}{G/W}", type_line: "Creature — Human Citizen", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1666::RULES.keywords, deck_limit: authored::card_1666::RULES.deck_limit }, effect: authored::card_1666::RULES.checked_effect(), triggers: authored::card_1666::RULES.triggers, activated: authored::card_1666::RULES.activated, enters: authored::card_1666::RULES.enters, entry: authored::card_1666::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 2, 0], generic: 0 }, rules: &authored::card_1666::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Gilded Lotus", oracle_id: Some("9a02a9a7-39d9-4763-85d3-747a0540b60b"), mana_cost: "{5}", type_line: "Artifact", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1751::RULES.keywords, deck_limit: authored::card_1751::RULES.deck_limit }, effect: authored::card_1751::RULES.checked_effect(), triggers: authored::card_1751::RULES.triggers, activated: authored::card_1751::RULES.activated, enters: authored::card_1751::RULES.enters, entry: authored::card_1751::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 5 }, rules: &authored::card_1751::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Hero in Training", oracle_id: Some("87f1a840-021c-4699-a2c4-f2926daf98eb"), mana_cost: "{2}{W}", type_line: "Creature — Human Hero", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2014::RULES.keywords, deck_limit: authored::card_2014::RULES.deck_limit }, effect: authored::card_2014::RULES.checked_effect(), triggers: authored::card_2014::RULES.triggers, activated: authored::card_2014::RULES.activated, enters: authored::card_2014::RULES.enters, entry: authored::card_2014::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_2014::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Heroes' Hangout", oracle_id: Some("43306a78-85b5-4df4-a018-75dd75aa8997"), mana_cost: "{R}", type_line: "Sorcery", oracle_keywords: &["Patrol Night", "Date Night"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_2017::RULES.keywords, deck_limit: authored::card_2017::RULES.deck_limit }, effect: authored::card_2017::RULES.effect, triggers: authored::card_2017::RULES.triggers, activated: authored::card_2017::RULES.activated, enters: authored::card_2017::RULES.enters, entry: authored::card_2017::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 0 }, rules: &authored::card_2017::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Hobgoblin, Mantled Marauder", oracle_id: Some("1d453331-6b8e-4d3d-b658-683eab2760da"), mana_cost: "{1}{R}", type_line: "Legendary Creature — Goblin Human Villain", oracle_keywords: &["Flying", "Haste"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2054::RULES.keywords, deck_limit: authored::card_2054::RULES.deck_limit }, effect: authored::card_2054::RULES.checked_effect(), triggers: authored::card_2054::RULES.triggers, activated: authored::card_2054::RULES.activated, enters: authored::card_2054::RULES.enters, entry: authored::card_2054::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 1 }, rules: &authored::card_2054::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Inner Demons Gangsters", oracle_id: Some("40b43763-3fbf-4ecd-8187-5d2c2c9cda82"), mana_cost: "{3}{B}", type_line: "Creature — Human Rogue Villain", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2171::RULES.keywords, deck_limit: authored::card_2171::RULES.deck_limit }, effect: authored::card_2171::RULES.checked_effect(), triggers: authored::card_2171::RULES.triggers, activated: authored::card_2171::RULES.activated, enters: authored::card_2171::RULES.enters, entry: authored::card_2171::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 3 }, rules: &authored::card_2171::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Lurking Lizards", oracle_id: Some("616312a9-83c8-4fb7-87ce-079ccd3231fa"), mana_cost: "{1}{G}", type_line: "Creature — Lizard Villain", oracle_keywords: &["Trample"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2586::RULES.keywords, deck_limit: authored::card_2586::RULES.deck_limit }, effect: authored::card_2586::RULES.checked_effect(), triggers: authored::card_2586::RULES.triggers, activated: authored::card_2586::RULES.activated, enters: authored::card_2586::RULES.enters, entry: authored::card_2586::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 1 }, rules: &authored::card_2586::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Make Your Move", oracle_id: Some("8226f31d-6f51-49c3-87f7-0c68f7f4f9ce"), mana_cost: "{2}{W}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2632::RULES.keywords, deck_limit: authored::card_2632::RULES.deck_limit }, effect: authored::card_2632::RULES.checked_effect(), triggers: authored::card_2632::RULES.triggers, activated: authored::card_2632::RULES.activated, enters: authored::card_2632::RULES.enters, entry: authored::card_2632::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_2632::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Masked Meower", oracle_id: Some("a0037ff8-64db-45dd-bba1-db5c49095083"), mana_cost: "{R}", type_line: "Creature — Spider Cat Hero", oracle_keywords: &["Haste"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2675::RULES.keywords, deck_limit: authored::card_2675::RULES.deck_limit }, effect: authored::card_2675::RULES.checked_effect(), triggers: authored::card_2675::RULES.triggers, activated: authored::card_2675::RULES.activated, enters: authored::card_2675::RULES.enters, entry: authored::card_2675::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 0 }, rules: &authored::card_2675::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Meteor Golem", oracle_id: Some("d9f11aa1-9219-42a8-85a9-a8f204160706"), mana_cost: "{7}", type_line: "Artifact Creature — Golem", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2731::RULES.keywords, deck_limit: authored::card_2731::RULES.deck_limit }, effect: authored::card_2731::RULES.checked_effect(), triggers: authored::card_2731::RULES.triggers, activated: authored::card_2731::RULES.activated, enters: authored::card_2731::RULES.enters, entry: authored::card_2731::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 7 }, rules: &authored::card_2731::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Mob Lookout", oracle_id: Some("47654b36-b215-4333-aa70-d88d627447e3"), mana_cost: "{1}{U/B}", type_line: "Creature — Human Rogue Villain", oracle_keywords: &["Connive"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2804::RULES.keywords, deck_limit: authored::card_2804::RULES.deck_limit }, effect: authored::card_2804::RULES.checked_effect(), triggers: authored::card_2804::RULES.triggers, activated: authored::card_2804::RULES.activated, enters: authored::card_2804::RULES.enters, entry: authored::card_2804::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 1 }, rules: &authored::card_2804::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Mystic Monastery", oracle_id: Some("834b8f71-9a45-42ae-9e99-e749fa6fb45e"), mana_cost: "", type_line: "Land", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2899::RULES.keywords, deck_limit: authored::card_2899::RULES.deck_limit }, effect: authored::card_2899::RULES.checked_effect(), triggers: authored::card_2899::RULES.triggers, activated: authored::card_2899::RULES.activated, enters: authored::card_2899::RULES.enters, entry: authored::card_2899::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_2899::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Pacifism", oracle_id: Some("5f5e0b10-c8cf-450c-bfd3-bcb0528ec330"), mana_cost: "{1}{W}", type_line: "Enchantment — Aura", oracle_keywords: &["Enchant"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3086::RULES.keywords, deck_limit: authored::card_3086::RULES.deck_limit }, effect: authored::card_3086::RULES.checked_effect(), triggers: authored::card_3086::RULES.triggers, activated: authored::card_3086::RULES.activated, enters: authored::card_3086::RULES.enters, entry: authored::card_3086::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_3086::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Patchwork Banner", oracle_id: Some("4fb00dbe-1f82-4ba6-b18c-97e816d10d3a"), mana_cost: "{3}", type_line: "Artifact", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_3108::RULES.keywords, deck_limit: authored::card_3108::RULES.deck_limit }, effect: authored::card_3108::RULES.effect, triggers: authored::card_3108::RULES.triggers, activated: authored::card_3108::RULES.activated, enters: authored::card_3108::RULES.enters, entry: authored::card_3108::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_3108::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Prowler, Clawed Thief", oracle_id: Some("76742bfc-ae99-4b06-b29d-e0e41d92ae14"), mana_cost: "{1}{U}{B}", type_line: "Legendary Creature — Human Rogue Villain", oracle_keywords: &["Menace", "Connive"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_3267::RULES.keywords, deck_limit: authored::card_3267::RULES.deck_limit }, effect: authored::card_3267::RULES.checked_effect(), triggers: authored::card_3267::RULES.triggers, activated: authored::card_3267::RULES.activated, enters: authored::card_3267::RULES.enters, entry: authored::card_3267::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 1, 0, 0, 0], generic: 1 }, rules: &authored::card_3267::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Pumpkin Bombardment", oracle_id: Some("37827e84-9f5f-49ed-b939-0cf80dac82e7"), mana_cost: "{B/R}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_3280::RULES.keywords, deck_limit: authored::card_3280::RULES.deck_limit }, effect: authored::card_3280::RULES.effect, triggers: authored::card_3280::RULES.triggers, activated: authored::card_3280::RULES.activated, enters: authored::card_3280::RULES.enters, entry: authored::card_3280::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 0 }, rules: &authored::card_3280::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Rhino's Rampage", oracle_id: Some("0a744206-04c4-46cb-b64d-cf192c9324df"), mana_cost: "{R/G}", type_line: "Sorcery", oracle_keywords: &["Fight"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_3513::RULES.keywords, deck_limit: authored::card_3513::RULES.deck_limit }, effect: authored::card_3513::RULES.effect, triggers: authored::card_3513::RULES.triggers, activated: authored::card_3513::RULES.activated, enters: authored::card_3513::RULES.enters, entry: authored::card_3513::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 0 }, rules: &authored::card_3513::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Robotics Mastery", oracle_id: Some("855318cb-9382-4183-87c7-93bdfac86431"), mana_cost: "{4}{U}", type_line: "Enchantment — Aura", oracle_keywords: &["Enchant", "Flash"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3565::RULES.keywords, deck_limit: authored::card_3565::RULES.deck_limit }, effect: authored::card_3565::RULES.checked_effect(), triggers: authored::card_3565::RULES.triggers, activated: authored::card_3565::RULES.activated, enters: authored::card_3565::RULES.enters, entry: authored::card_3565::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 4 }, rules: &authored::card_3565::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Rocket-Powered Goblin Glider", oracle_id: Some("5db9121d-934a-44f1-b178-b389c6595af7"), mana_cost: "{3}", type_line: "Artifact — Equipment", oracle_keywords: &["Mayhem", "Equip"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_3568::RULES.keywords, deck_limit: authored::card_3568::RULES.deck_limit }, effect: authored::card_3568::RULES.effect, triggers: authored::card_3568::RULES.triggers, activated: authored::card_3568::RULES.activated, enters: authored::card_3568::RULES.enters, entry: authored::card_3568::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_3568::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Romantic Rendezvous", oracle_id: Some("8ca585b5-25c4-41cc-a0b9-dc66ae803d04"), mana_cost: "{1}{R}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3580::RULES.keywords, deck_limit: authored::card_3580::RULES.deck_limit }, effect: authored::card_3580::RULES.checked_effect(), triggers: authored::card_3580::RULES.triggers, activated: authored::card_3580::RULES.activated, enters: authored::card_3580::RULES.enters, entry: authored::card_3580::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 1 }, rules: &authored::card_3580::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Scout the City", oracle_id: Some("ecb8e283-5cdb-4e6a-bdff-546c1617363c"), mana_cost: "{1}{G}", type_line: "Sorcery", oracle_keywords: &["Look Around", "Bring Down", "Mill"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3733::RULES.keywords, deck_limit: authored::card_3733::RULES.deck_limit }, effect: authored::card_3733::RULES.checked_effect(), triggers: authored::card_3733::RULES.triggers, activated: authored::card_3733::RULES.activated, enters: authored::card_3733::RULES.enters, entry: authored::card_3733::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 1 }, rules: &authored::card_3733::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Secluded Courtyard", oracle_id: Some("79ba18fd-f184-43c1-86df-56ee18ce806c"), mana_cost: "", type_line: "Land", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3757::RULES.keywords, deck_limit: authored::card_3757::RULES.deck_limit }, effect: authored::card_3757::RULES.checked_effect(), triggers: authored::card_3757::RULES.triggers, activated: authored::card_3757::RULES.activated, enters: authored::card_3757::RULES.enters, entry: authored::card_3757::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_3757::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Selfless Police Captain", oracle_id: Some("ece77a64-8d99-430e-8407-bdb321828d63"), mana_cost: "{1}{W}", type_line: "Creature — Human Detective", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_3782::RULES.keywords, deck_limit: authored::card_3782::RULES.deck_limit }, effect: authored::card_3782::RULES.effect, triggers: authored::card_3782::RULES.triggers, activated: authored::card_3782::RULES.activated, enters: authored::card_3782::RULES.enters, entry: authored::card_3782::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_3782::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Solemn Simulacrum", oracle_id: Some("00c0543c-2a1f-4425-8283-4062d74a1637"), mana_cost: "{4}", type_line: "Artifact Creature — Golem", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3958::RULES.keywords, deck_limit: authored::card_3958::RULES.deck_limit }, effect: authored::card_3958::RULES.checked_effect(), triggers: authored::card_3958::RULES.triggers, activated: authored::card_3958::RULES.activated, enters: authored::card_3958::RULES.enters, entry: authored::card_3958::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 4 }, rules: &authored::card_3958::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Spectacular Tactics", oracle_id: Some("15948fdf-3f9e-4eed-b85b-c5a554ff0571"), mana_cost: "{1}{W}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3997::RULES.keywords, deck_limit: authored::card_3997::RULES.deck_limit }, effect: authored::card_3997::RULES.checked_effect(), triggers: authored::card_3997::RULES.triggers, activated: authored::card_3997::RULES.activated, enters: authored::card_3997::RULES.enters, entry: authored::card_3997::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_3997::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Spider-Gwen, Free Spirit", oracle_id: Some("2648559d-1d96-47d2-a769-cfa2cbca2be5"), mana_cost: "{2}{R}", type_line: "Legendary Creature — Spider Human Hero", oracle_keywords: &["Reach"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_4023::RULES.keywords, deck_limit: authored::card_4023::RULES.deck_limit }, effect: authored::card_4023::RULES.checked_effect(), triggers: authored::card_4023::RULES.triggers, activated: authored::card_4023::RULES.activated, enters: authored::card_4023::RULES.enters, entry: authored::card_4023::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 2 }, rules: &authored::card_4023::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Spider-Man, Web-Slinger", oracle_id: Some("505231d8-714f-4c5b-a635-d1662bd18a78"), mana_cost: "{2}{W}", type_line: "Legendary Creature — Spider Human Hero", oracle_keywords: &["Web-slinging"], standard_legal: true, legendary: true, implemented: false, keywords: authored::card_4032::RULES.keywords, deck_limit: authored::card_4032::RULES.deck_limit }, effect: authored::card_4032::RULES.effect, triggers: authored::card_4032::RULES.triggers, activated: authored::card_4032::RULES.activated, enters: authored::card_4032::RULES.enters, entry: authored::card_4032::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_4032::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Spider-Mobile", oracle_id: Some("3c322d99-2bc4-4618-aeb3-5583fce54f1b"), mana_cost: "{3}", type_line: "Artifact — Vehicle", oracle_keywords: &["Crew", "Trample"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_4033::RULES.keywords, deck_limit: authored::card_4033::RULES.deck_limit }, effect: authored::card_4033::RULES.effect, triggers: authored::card_4033::RULES.triggers, activated: authored::card_4033::RULES.activated, enters: authored::card_4033::RULES.enters, entry: authored::card_4033::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_4033::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Spider-Rex, Daring Dino", oracle_id: Some("58b771d4-dfaf-4189-a5f6-4bc0fb4c0fa3"), mana_cost: "{4}{G}{G}", type_line: "Legendary Creature — Spider Dinosaur Hero", oracle_keywords: &["Reach", "Trample", "Ward"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_4035::RULES.keywords, deck_limit: authored::card_4035::RULES.deck_limit }, effect: authored::card_4035::RULES.checked_effect(), triggers: authored::card_4035::RULES.triggers, activated: authored::card_4035::RULES.activated, enters: authored::card_4035::RULES.enters, entry: authored::card_4035::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 2, 0], generic: 4 }, rules: &authored::card_4035::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Spider-UK", oracle_id: Some("ebf8d006-2997-431b-b4cb-9692741b45ca"), mana_cost: "{3}{W}", type_line: "Legendary Creature — Spider Human Hero", oracle_keywords: &["Web-slinging"], standard_legal: true, legendary: true, implemented: false, keywords: authored::card_4039::RULES.keywords, deck_limit: authored::card_4039::RULES.deck_limit }, effect: authored::card_4039::RULES.effect, triggers: authored::card_4039::RULES.triggers, activated: authored::card_4039::RULES.activated, enters: authored::card_4039::RULES.enters, entry: authored::card_4039::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_4039::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Starling, Aerial Ally", oracle_id: Some("13fd7ca6-c278-409f-a9c4-57f16051ffce"), mana_cost: "{4}{W}", type_line: "Legendary Creature — Human Hero", oracle_keywords: &["Flying"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_4106::RULES.keywords, deck_limit: authored::card_4106::RULES.deck_limit }, effect: authored::card_4106::RULES.checked_effect(), triggers: authored::card_4106::RULES.triggers, activated: authored::card_4106::RULES.activated, enters: authored::card_4106::RULES.enters, entry: authored::card_4106::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 4 }, rules: &authored::card_4106::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Stegron the Dinosaur Man", oracle_id: Some("5c94b616-8edf-465c-b316-4599df95c52a"), mana_cost: "{4}{R}", type_line: "Legendary Creature — Dinosaur Villain", oracle_keywords: &["Dinosaur Formula", "Menace"], standard_legal: true, legendary: true, implemented: false, keywords: authored::card_4131::RULES.keywords, deck_limit: authored::card_4131::RULES.deck_limit }, effect: authored::card_4131::RULES.effect, triggers: authored::card_4131::RULES.triggers, activated: authored::card_4131::RULES.activated, enters: authored::card_4131::RULES.enters, entry: authored::card_4131::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 4 }, rules: &authored::card_4131::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Sudden Strike", oracle_id: Some("8147a759-e387-48f6-9c1e-bb39622ed553"), mana_cost: "{1}{W}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4203::RULES.keywords, deck_limit: authored::card_4203::RULES.deck_limit }, effect: authored::card_4203::RULES.checked_effect(), triggers: authored::card_4203::RULES.triggers, activated: authored::card_4203::RULES.activated, enters: authored::card_4203::RULES.enters, entry: authored::card_4203::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_4203::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Superior Foes of Spider-Man", oracle_id: Some("a6a7af21-e52c-4f7f-a57f-bd365d075966"), mana_cost: "{2}{R}", type_line: "Creature — Human Rogue Villain", oracle_keywords: &["Trample"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_4261::RULES.keywords, deck_limit: authored::card_4261::RULES.deck_limit }, effect: authored::card_4261::RULES.effect, triggers: authored::card_4261::RULES.triggers, activated: authored::card_4261::RULES.activated, enters: authored::card_4261::RULES.enters, entry: authored::card_4261::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 2 }, rules: &authored::card_4261::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Swarm, Being of Bees", oracle_id: Some("10c8cf3b-2530-49e7-b38e-95cdaae8a6e7"), mana_cost: "{2}{B}", type_line: "Legendary Creature — Insect Villain", oracle_keywords: &["Flying", "Mayhem", "Flash"], standard_legal: true, legendary: true, implemented: false, keywords: authored::card_4287::RULES.keywords, deck_limit: authored::card_4287::RULES.deck_limit }, effect: authored::card_4287::RULES.effect, triggers: authored::card_4287::RULES.triggers, activated: authored::card_4287::RULES.activated, enters: authored::card_4287::RULES.enters, entry: authored::card_4287::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 2 }, rules: &authored::card_4287::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Swiftfoot Boots", oracle_id: Some("c8b143ad-43ec-4e0d-a440-e348daa31391"), mana_cost: "{2}", type_line: "Artifact — Equipment", oracle_keywords: &["Equip"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4292::RULES.keywords, deck_limit: authored::card_4292::RULES.deck_limit }, effect: authored::card_4292::RULES.checked_effect(), triggers: authored::card_4292::RULES.triggers, activated: authored::card_4292::RULES.activated, enters: authored::card_4292::RULES.enters, entry: authored::card_4292::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_4292::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Taxi Driver", oracle_id: Some("a17a92f7-e95c-4d0e-a43e-7e6b41a59139"), mana_cost: "{1}{R}", type_line: "Creature — Human Pilot", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4347::RULES.keywords, deck_limit: authored::card_4347::RULES.deck_limit }, effect: authored::card_4347::RULES.checked_effect(), triggers: authored::card_4347::RULES.triggers, activated: authored::card_4347::RULES.activated, enters: authored::card_4347::RULES.enters, entry: authored::card_4347::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 1 }, rules: &authored::card_4347::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Terramorphic Expanse", oracle_id: Some("1bd3e453-aa21-4ee6-95c2-d6d920ee8e7a"), mana_cost: "", type_line: "Land", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4390::RULES.keywords, deck_limit: authored::card_4390::RULES.deck_limit }, effect: authored::card_4390::RULES.checked_effect(), triggers: authored::card_4390::RULES.triggers, activated: authored::card_4390::RULES.activated, enters: authored::card_4390::RULES.enters, entry: authored::card_4390::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_4390::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Terrific Team-Up", oracle_id: Some("431c5b04-7387-4724-adfa-d1eb7874e79e"), mana_cost: "{3}{G}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_4394::RULES.keywords, deck_limit: authored::card_4394::RULES.deck_limit }, effect: authored::card_4394::RULES.effect, triggers: authored::card_4394::RULES.triggers, activated: authored::card_4394::RULES.activated, enters: authored::card_4394::RULES.enters, entry: authored::card_4394::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 3 }, rules: &authored::card_4394::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "The Clone Saga", oracle_id: Some("0f0282c6-aceb-4879-8ec8-482d0501204a"), mana_cost: "{3}{U}", type_line: "Enchantment — Saga", oracle_keywords: &["Surveil"], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_4428::RULES.keywords, deck_limit: authored::card_4428::RULES.deck_limit }, effect: authored::card_4428::RULES.effect, triggers: authored::card_4428::RULES.triggers, activated: authored::card_4428::RULES.activated, enters: authored::card_4428::RULES.enters, entry: authored::card_4428::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_4428::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "The Spot's Portal", oracle_id: Some("3ceac6ce-acde-42ab-b79d-ff153074d2d8"), mana_cost: "{2}{B}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4503::RULES.keywords, deck_limit: authored::card_4503::RULES.deck_limit }, effect: authored::card_4503::RULES.checked_effect(), triggers: authored::card_4503::RULES.triggers, activated: authored::card_4503::RULES.activated, enters: authored::card_4503::RULES.enters, entry: authored::card_4503::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 2 }, rules: &authored::card_4503::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Think Twice", oracle_id: Some("fa85c5a2-8e83-4624-a35a-a0bbf17ecbb4"), mana_cost: "{1}{U}", type_line: "Instant", oracle_keywords: &["Flashback"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4530::RULES.keywords, deck_limit: authored::card_4530::RULES.deck_limit }, effect: authored::card_4530::RULES.checked_effect(), triggers: authored::card_4530::RULES.triggers, activated: authored::card_4530::RULES.activated, enters: authored::card_4530::RULES.enters, entry: authored::card_4530::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_4530::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Thwip!", oracle_id: Some("f7d11254-b48a-4016-a035-c0d6642fdfae"), mana_cost: "{W}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4590::RULES.keywords, deck_limit: authored::card_4590::RULES.deck_limit }, effect: authored::card_4590::RULES.checked_effect(), triggers: authored::card_4590::RULES.triggers, activated: authored::card_4590::RULES.activated, enters: authored::card_4590::RULES.enters, entry: authored::card_4590::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_4590::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Tombstone, Career Criminal", oracle_id: Some("6ad2cbb6-2291-420c-bfc8-ed3f079b59bf"), mana_cost: "{2}{B}", type_line: "Legendary Creature — Human Villain", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_4625::RULES.keywords, deck_limit: authored::card_4625::RULES.deck_limit }, effect: authored::card_4625::RULES.checked_effect(), triggers: authored::card_4625::RULES.triggers, activated: authored::card_4625::RULES.activated, enters: authored::card_4625::RULES.enters, entry: authored::card_4625::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 2 }, rules: &authored::card_4625::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Unstable Experiment", oracle_id: Some("5b7bbc76-22cc-482e-8d65-34cff34c93c8"), mana_cost: "{1}{U}", type_line: "Instant", oracle_keywords: &["Connive"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4799::RULES.keywords, deck_limit: authored::card_4799::RULES.deck_limit }, effect: authored::card_4799::RULES.checked_effect(), triggers: authored::card_4799::RULES.triggers, activated: authored::card_4799::RULES.activated, enters: authored::card_4799::RULES.enters, entry: authored::card_4799::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_4799::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Valorous Stance", oracle_id: Some("1f938353-9081-4dc1-b8e7-d18ece038bd4"), mana_cost: "{1}{W}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4836::RULES.keywords, deck_limit: authored::card_4836::RULES.deck_limit }, effect: authored::card_4836::RULES.checked_effect(), triggers: authored::card_4836::RULES.triggers, activated: authored::card_4836::RULES.activated, enters: authored::card_4836::RULES.enters, entry: authored::card_4836::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_4836::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Venom, Evil Unleashed", oracle_id: Some("6731d861-cc52-4a2e-8c33-154681fd50e4"), mana_cost: "{4}{B}", type_line: "Legendary Creature — Symbiote Villain", oracle_keywords: &["Deathtouch"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_4870::RULES.keywords, deck_limit: authored::card_4870::RULES.deck_limit }, effect: authored::card_4870::RULES.checked_effect(), triggers: authored::card_4870::RULES.triggers, activated: authored::card_4870::RULES.activated, enters: authored::card_4870::RULES.enters, entry: authored::card_4870::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 4 }, rules: &authored::card_4870::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Venomized Cat", oracle_id: Some("eb54d807-624b-471e-8c44-07322003903e"), mana_cost: "{2}{B}", type_line: "Creature — Symbiote Cat Villain", oracle_keywords: &["Mill", "Deathtouch"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4871::RULES.keywords, deck_limit: authored::card_4871::RULES.deck_limit }, effect: authored::card_4871::RULES.checked_effect(), triggers: authored::card_4871::RULES.triggers, activated: authored::card_4871::RULES.activated, enters: authored::card_4871::RULES.enters, entry: authored::card_4871::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 2 }, rules: &authored::card_4871::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Whoosh!", oracle_id: Some("855353e2-b686-482a-b987-c6ae456f088c"), mana_cost: "{1}{U}", type_line: "Instant", oracle_keywords: &["Kicker"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_5049::RULES.keywords, deck_limit: authored::card_5049::RULES.deck_limit }, effect: authored::card_5049::RULES.checked_effect(), triggers: authored::card_5049::RULES.triggers, activated: authored::card_5049::RULES.activated, enters: authored::card_5049::RULES.enters, entry: authored::card_5049::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_5049::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Wild Pack Squad", oracle_id: Some("294b2aa7-cb46-46bd-8241-69b26d73aa2d"), mana_cost: "{2}{W}", type_line: "Creature — Human Mercenary", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_5058::RULES.keywords, deck_limit: authored::card_5058::RULES.deck_limit }, effect: authored::card_5058::RULES.effect, triggers: authored::card_5058::RULES.triggers, activated: authored::card_5058::RULES.activated, enters: authored::card_5058::RULES.enters, entry: authored::card_5058::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_5058::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Wisecrack", oracle_id: Some("e9c9a445-4386-4616-ad00-f08064d73dcd"), mana_cost: "{2}{R}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: false, keywords: authored::card_5090::RULES.keywords, deck_limit: authored::card_5090::RULES.deck_limit }, effect: authored::card_5090::RULES.effect, triggers: authored::card_5090::RULES.triggers, activated: authored::card_5090::RULES.activated, enters: authored::card_5090::RULES.enters, entry: authored::card_5090::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 2 }, rules: &authored::card_5090::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Withering Torment", oracle_id: Some("ffce81c5-1b58-4882-a4e7-6f8d7cb170de"), mana_cost: "{2}{B}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_5101::RULES.keywords, deck_limit: authored::card_5101::RULES.deck_limit }, effect: authored::card_5101::RULES.checked_effect(), triggers: authored::card_5101::RULES.triggers, activated: authored::card_5101::RULES.activated, enters: authored::card_5101::RULES.enters, entry: authored::card_5101::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 2 }, rules: &authored::card_5101::RULES },
];
}
pub use cards::ENTRIES;
