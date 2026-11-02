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
pub(super) mod card_0011 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/giant_growth.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0017 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/a_i_m_labs.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0018 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/a_i_m_scientists.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0019 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/a_i_m_synthoids.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0033 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/abomination_terrifying_titan.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0038 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/absorbing_man.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0068 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/aerial_doombot.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0087 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/agent_13_sharon_carter.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0089 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/agent_maria_hill.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0090 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/agent_phil_coulson.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0092 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/agent_of_atlas.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0094 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/agents_of_hydra.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0095 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/agents_of_s_h_i_e_l_d.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0129 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/alien_invasion.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0174 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/ant_man_s_army.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0175 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/ant_man_colony_commander.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0202 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/arc_reactor.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0219 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/ares_god_of_war.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0225 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/armor_wars.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0231 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/arnim_zola_bio_fanatic.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0237 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/asgardian_citadel.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0253 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/atlantean_cavalry.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0254 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/atlantis_attacks.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0260 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/attuma_atlantean_warlord.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0282 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/avengers_assemble.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0283 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/avengers_disassembled.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0284 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/avengers_hangar.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0285 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/avengers_tower.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0286 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/avengers_under_siege.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0333 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/baron_helmut_zemo.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0334 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/baron_strucker_hydra_overlord.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0352 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/baxter_building.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0360 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/beast_erudite_aerialist.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0421 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/birnin_zana_plaza.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0431 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/black_panther_vanguard.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0433 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/black_widow_double_agent.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0434 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/black_widow_super_spy.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0484 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/bold_biochemist.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0507 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/borough_backup.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0529 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/brave_brawler.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0571 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/bruce_banner_the_incredible_hulk.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0579 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/bullseye_death_dealer.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0619 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/call_damage_control.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0641 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/captain_america_s_shield.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0642 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/captain_america_living_legend.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0643 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/captain_america_super_soldier.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0644 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/captain_america_wings_of_freedom.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0646 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/captain_mar_vell_space_born.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0647 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/captain_marvel_earth_s_protector.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0676 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/castle_doom.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0759 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/claim_the_kingdom.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0773 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/cloak_and_dagger_entwined.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0798 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/colleen_wing_street_samurai.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0835 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/construct_a_cosmic_cube.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0859 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/cosmic_cube.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0892 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/crossbones_malicious_mercenary.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0897 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/crowd_of_true_believers.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0901 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/cruel_alliance.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0957 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/daredevil_man_without_fear.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0964 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/dark_deed.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0966 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/dark_fortress.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1004 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/death_to_our_enemies.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1016 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/decoy_ploy.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1044 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/dependable_quinjet.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1045 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/depower.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1117 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/doc_samson_super_psychiatrist.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1119 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/doctor_doom.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1132 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/doom_reigns_supreme.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1240 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/earth_s_mightiest_heroes.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1250 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/echo_perceptive_prodigy.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1281 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/elektra_daughter_of_the_hand.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1353 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/epic_fight.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1390 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/evil_s_thrall.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1432 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/falcon_s_wing_harness.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1433 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/falcon_winged_wonder.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1508 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/fin_fang_foom.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1538 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/fisk_tower.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod frozen_in_ice {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/frozen_in_ice.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1655 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/futurist_forge.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1698 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/gathering_place.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1733 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/ghost_spectral_saboteur.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1741 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/giant_sized_flying_ant.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1771 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/gleaming_bastion.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1799 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/go_nuts.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1881 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/grim_reaper_lethal_legionnaire.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1902 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/guerrilla_gorilla.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1920 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/h_e_r_b_i_e_scout_unit.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1921 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/hulk_smash.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1922 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/hydra_assault_robot.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1923 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/hydra_infiltration.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1924 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/hydra_troopers.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1966 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/hawkeye_s_bow.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1967 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/hawkeye_master_marksman.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1968 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/hawkeye_young_avenger.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1995 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/helicarrier_strike.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1997 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/hell_s_kitchen.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1998 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/hellcat_undying_vigilante.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2009 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/hercules_prince_of_power.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2018 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/heroic_feast.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2020 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/hex_magic.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2028 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/hidden_lair.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2047 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/hire_a_crew.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2080 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/hour_of_defeat.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2092 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/hulk_gamma_goliath.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2094 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/hulkling_burgeoning_bruiser.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2097 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/human_torch_johnny_storm.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2111 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/hydraulic_helper.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2118 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/i_am_iron_man.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2216 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/invisible_woman_sue_storm.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2224 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/iron_fist_living_weapon.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2229 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/iron_lad_diverging_destiny.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2230 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/iron_man_armor.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2231 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/iron_man_master_of_machines.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2235 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/ironheart_clever_champion.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2263 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/jennifer_walters_the_sensational_she_hulk.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2274 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/jessica_jones_private_eye.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2301 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/justice_vance_astrovik.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2302 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/k_un_lun_warrior.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2303 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/ka_zar_of_the_savage_land.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2311 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/kang_the_conqueror.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2312 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/kang_temporal_tyrant.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2349 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/kid_loki.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2352 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/killmonger_scourge_of_wakanda.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2361 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/king_t_challa_black_panther_hope_enduring.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2362 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/kingpin_s_enforcers.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2381 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/klaw_sonic_subjugator.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2387 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/knight_of_wundagore.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2412 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/kree_commandos.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2413 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/kree_sentinel.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2463 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/leader_super_genius.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2497 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/lightning_strike.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2538 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/loki_laufeyson.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2539 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/loki_god_of_mischief.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2561 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/los_diablos_missile_base.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2574 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/luke_cage_power_man.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2596 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/m_o_d_o_k.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2604 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/machinesmith_automaton.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2606 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/madame_hydra.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2607 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/madame_masque.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2792 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/mister_fantastic_reed_richards.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2793 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/mister_hyde_monster_within.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2798 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/misty_knight_hero_for_hire.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2801 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/mjolnir_hammer_of_thor.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2808 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/mockingbird_ace_agent.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2811 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/mole_man_moloid_master.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2832 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/monica_rambeau_photon_living_light.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2840 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/moon_girl_and_devil_dinosaur.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2849 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/moonstone_harsh_mistress.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2869 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/ms_marvel_kamala_khan.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2876 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/multiversal_incursion.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2880 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/murdock_s_crusade.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2905 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/namor_the_sub_mariner.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2932 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/nick_fury_agent_of_s_h_i_e_l_d.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2934 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/night_nurse_healer_of_heroes.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2946 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/ninja_of_the_hand.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3002 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/okoye_dora_milaje_leader.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3040 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/origin_of_the_avengers.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3096 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/panther_pounce.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3114 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/patriot_shield_wielder.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3141 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/pet_avengers.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3150 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/photon_blast_barrage.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3197 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/political_triumph.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3212 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/powerful_broker.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3255 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/project_deathlok_soldier.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3281 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/punishing_punch.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3290 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/pym_particles.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3291 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/pym_technologies.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3301 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/quake_agent_of_s_h_i_e_l_d.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3315 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/quicksilver_brash_blur.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3336 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/raft_security_officer.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3382 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/rapid_rescue.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3426 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/red_guardian_super_soldier.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3428 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/red_hulk.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3430 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/red_room_recruit.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3463 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/reptil_dinomorpher.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3466 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/repulsor_blast.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3495 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/restorative_technique.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3511 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/rewrite_history.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3517 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/rick_jones_destined_sidekick.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3564 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/robot_domination.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3581 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/ronin_shadow_stalker.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3604 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/roxxon_brutes.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3635 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/s_h_i_e_l_d_deployment_drone.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3636 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/s_h_i_e_l_d_flying_car.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3637 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/s_h_i_e_l_d_helicarrier.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3638 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/s_h_i_e_l_d_spy_kit.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3692 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/savage_land_dinosaur.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3723 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/scientist_supreme_of_a_i_m.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3760 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/secret_invasion.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3794 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/serpent_specialist.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3810 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/shang_chi_master_of_kung_fu.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3823 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/she_hulk_jade_defender.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3857 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/shuri_wakandan_inventor.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4003 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/speed_young_avenger.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4004 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/speedball_new_warrior.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4031 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/spider_man_to_the_rescue.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4041 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/spider_woman_secret_agent.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4103 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/stark_industries.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4104 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/stark_industries_executive.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4120 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/stature_size_shifter.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4158 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/stolen_stark_tech.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4169 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/storm_windrider.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4197 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/subterranean_cavern.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4252 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/super_intelligence.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4254 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/super_speed.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4255 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/super_strength.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4256 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/super_suit.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4257 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/super_villain_lockup.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4258 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/super_adaptoid.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4259 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/super_skrull.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4260 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/super_soldier_serum.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4275 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/surveillance_room.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4297 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/swordsman_sharp_scoundrel.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4321 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/take_up_the_shield.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4342 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/taskmaster_mercenary_mimic.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4351 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/team_tactics.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4414 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/thanos_the_mad_titan.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4421 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/the_astonishing_ant_man.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4430 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/the_coming_of_galactus.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4458 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/the_kingpin_of_crime.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4473 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/the_masters_of_evil.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4475 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/the_mighty_thor_jane_foster.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4477 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/the_mind_stone.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4492 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/the_ruinous_wrecking_crew.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4494 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/the_scarlet_witch.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4495 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/the_sentry_golden_guardian.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4497 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/the_serpent_society.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4505 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/the_super_hero_civil_war.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4508 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/the_ten_rings.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4510 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/the_thing_ben_grimm.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4512 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/the_unbeatable_squirrel_girl.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4513 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/the_vision.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4520 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/the_wondrous_wasp.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4533 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/thirst_for_knowledge.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4537 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/thor_odinson.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4538 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/thor_god_of_thunder.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4582 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/thunderbolts_conspiracy.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4600 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/tigra_feline_fury.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4613 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/titania_rugged_rumbler.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4631 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/tony_stark_the_invincible_iron_man.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4632 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/too_evil_to_stay_dead.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4653 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/training_compound.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4654 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/training_regimen.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4682 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/trickster_s_stratagem.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4691 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/truck_toss.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4736 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/u_s_agent_john_walker.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4747 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/ultron_drone.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4748 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/ultron_artificial_malevolence.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4766 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/undercover_skrull.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4788 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/unliving_legionnaire.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4883 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/vibranium_energy_daggers.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4894 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/villainous_hideout.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4915 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/vision_quest.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4916 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/vision_of_love.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4918 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/visions_of_villainy.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4922 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/viv_vision_teen_synthezoid.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4933 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/volcanic_villain.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4953 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/wakandan_drone_flock.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4954 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/wakandan_royal_guard.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4972 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/war_machine_legacy_of_iron.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_5019 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/we_say_thee_nay.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_5024 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/web_up.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_5038 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/whiplash_vengeful_engineer.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_5047 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/white_tiger_ava_ayala.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_5048 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/white_widow_free_agent.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_5050 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/wiccan_rising_magician.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_5056 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/widow_s_bite.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_5082 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/winter_soldier_icy_assassin.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_5106 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/wolverine_fierce_fighter.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_5107 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/wonder_man_hollywood_hero.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_5113 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/world_war_hulk.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_5114 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/worlds_within_worlds.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_5136 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/yellowjacket_heartless_marauder.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_5391 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/photon_living_light.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_5392 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/black_panther_hope_enduring.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_5396 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/the_sensational_she_hulk.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_5403 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/the_invincible_iron_man.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_5404 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/msh/the_incredible_hulk.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
}
#[rustfmt::skip]
pub static ENTRIES: [CatalogueEntry; 278] = [
CatalogueEntry { definition: Definition { info: CardInfo { name: "Giant Growth", oracle_id: Some("5748ebf1-24e3-499d-ab7c-c2cebd462a24"), mana_cost: "{G}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0011::RULES.keywords, deck_limit: authored::card_0011::RULES.deck_limit }, effect: authored::card_0011::RULES.checked_effect(), triggers: authored::card_0011::RULES.triggers, activated: authored::card_0011::RULES.activated, enters: authored::card_0011::RULES.enters, entry: authored::card_0011::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 0 }, rules: &authored::card_0011::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "A.I.M. Labs", oracle_id: Some("8417f2d5-93c9-474a-a143-5c4e17097a88"), mana_cost: "", type_line: "Land", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0017::RULES.keywords, deck_limit: authored::card_0017::RULES.deck_limit }, effect: authored::card_0017::RULES.checked_effect(), triggers: authored::card_0017::RULES.triggers, activated: authored::card_0017::RULES.activated, enters: authored::card_0017::RULES.enters, entry: authored::card_0017::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_0017::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "A.I.M. Scientists", oracle_id: Some("fa732888-5b7d-4087-a6e5-1db1c98fbd4c"), mana_cost: "{3}{U}", type_line: "Creature — Human Scientist Villain", oracle_keywords: &["Landcycling", "Basic landcycling", "Typecycling", "Cycling", "Connive"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0018::RULES.keywords, deck_limit: authored::card_0018::RULES.deck_limit }, effect: authored::card_0018::RULES.checked_effect(), triggers: authored::card_0018::RULES.triggers, activated: authored::card_0018::RULES.activated, enters: authored::card_0018::RULES.enters, entry: authored::card_0018::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_0018::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "A.I.M. Synthoids", oracle_id: Some("10916f33-aaa6-43a1-9835-b60fd061161f"), mana_cost: "{2}", type_line: "Artifact Creature — Robot Villain", oracle_keywords: &["Surveil"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0019::RULES.keywords, deck_limit: authored::card_0019::RULES.deck_limit }, effect: authored::card_0019::RULES.checked_effect(), triggers: authored::card_0019::RULES.triggers, activated: authored::card_0019::RULES.activated, enters: authored::card_0019::RULES.enters, entry: authored::card_0019::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_0019::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Abomination, Terrifying Titan", oracle_id: Some("72101185-4336-480a-b4d7-f8ceb0007a51"), mana_cost: "{3}{R/G}", type_line: "Legendary Creature — Gamma Villain", oracle_keywords: &["Power-up", "Trample", "Fight"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_0033::RULES.keywords, deck_limit: authored::card_0033::RULES.deck_limit }, effect: authored::card_0033::RULES.checked_effect(), triggers: authored::card_0033::RULES.triggers, activated: authored::card_0033::RULES.activated, enters: authored::card_0033::RULES.enters, entry: authored::card_0033::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 3 }, rules: &authored::card_0033::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Absorbing Man", oracle_id: Some("ec8e310f-e871-498f-8d80-c3617c5e1433"), mana_cost: "{1}{G}{U}", type_line: "Legendary Creature — Human Villain", oracle_keywords: &["Vigilance"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_0038::RULES.keywords, deck_limit: authored::card_0038::RULES.deck_limit }, effect: authored::card_0038::RULES.checked_effect(), triggers: authored::card_0038::RULES.triggers, activated: authored::card_0038::RULES.activated, enters: authored::card_0038::RULES.enters, entry: authored::card_0038::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 1, 0], generic: 1 }, rules: &authored::card_0038::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Aerial Doombot", oracle_id: Some("f24efc60-fdac-4a5d-bcbc-784ec5ae6d96"), mana_cost: "{U}", type_line: "Artifact Creature — Robot Villain", oracle_keywords: &["Flying", "Power-up"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0068::RULES.keywords, deck_limit: authored::card_0068::RULES.deck_limit }, effect: authored::card_0068::RULES.checked_effect(), triggers: authored::card_0068::RULES.triggers, activated: authored::card_0068::RULES.activated, enters: authored::card_0068::RULES.enters, entry: authored::card_0068::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_0068::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Agent 13, Sharon Carter", oracle_id: Some("ddf7201a-6672-416d-be6b-202dafbeccab"), mana_cost: "{2}{W}", type_line: "Legendary Creature — Human Spy Hero", oracle_keywords: &["Investigate"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_0087::RULES.keywords, deck_limit: authored::card_0087::RULES.deck_limit }, effect: authored::card_0087::RULES.checked_effect(), triggers: authored::card_0087::RULES.triggers, activated: authored::card_0087::RULES.activated, enters: authored::card_0087::RULES.enters, entry: authored::card_0087::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_0087::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Agent Maria Hill", oracle_id: Some("a2d90efe-80f2-4069-b135-a8bd00ccd2b0"), mana_cost: "{W}", type_line: "Legendary Creature — Human Spy Hero", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_0089::RULES.keywords, deck_limit: authored::card_0089::RULES.deck_limit }, effect: authored::card_0089::RULES.checked_effect(), triggers: authored::card_0089::RULES.triggers, activated: authored::card_0089::RULES.activated, enters: authored::card_0089::RULES.enters, entry: authored::card_0089::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_0089::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Agent Phil Coulson", oracle_id: Some("dae20bcd-dc24-4f9d-9400-a6f013b49f14"), mana_cost: "{1}{W}", type_line: "Legendary Creature — Human Spy Hero", oracle_keywords: &["Vigilance"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_0090::RULES.keywords, deck_limit: authored::card_0090::RULES.deck_limit }, effect: authored::card_0090::RULES.checked_effect(), triggers: authored::card_0090::RULES.triggers, activated: authored::card_0090::RULES.activated, enters: authored::card_0090::RULES.enters, entry: authored::card_0090::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_0090::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Agent of Atlas", oracle_id: Some("cd403866-bef1-413a-85e2-c86e295b3589"), mana_cost: "{1}{W}", type_line: "Creature — Human Spy Hero", oracle_keywords: &["Prowess"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0092::RULES.keywords, deck_limit: authored::card_0092::RULES.deck_limit }, effect: authored::card_0092::RULES.checked_effect(), triggers: authored::card_0092::RULES.triggers, activated: authored::card_0092::RULES.activated, enters: authored::card_0092::RULES.enters, entry: authored::card_0092::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_0092::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Agents of HYDRA", oracle_id: Some("31816924-1430-4581-86e8-325e2ca16217"), mana_cost: "{1}{B}", type_line: "Creature — Human Spy Villain", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0094::RULES.keywords, deck_limit: authored::card_0094::RULES.deck_limit }, effect: authored::card_0094::RULES.checked_effect(), triggers: authored::card_0094::RULES.triggers, activated: authored::card_0094::RULES.activated, enters: authored::card_0094::RULES.enters, entry: authored::card_0094::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 1 }, rules: &authored::card_0094::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Agents of S.H.I.E.L.D.", oracle_id: Some("203b72d2-a273-49cc-bd10-2fbc07ed2fc8"), mana_cost: "{2}{W}", type_line: "Creature — Human Spy Hero", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0095::RULES.keywords, deck_limit: authored::card_0095::RULES.deck_limit }, effect: authored::card_0095::RULES.checked_effect(), triggers: authored::card_0095::RULES.triggers, activated: authored::card_0095::RULES.activated, enters: authored::card_0095::RULES.enters, entry: authored::card_0095::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_0095::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Alien Invasion", oracle_id: Some("7c854a07-e031-458e-a335-9c31865020b8"), mana_cost: "{1}{R}{R}{G}", type_line: "Enchantment", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0129::RULES.keywords, deck_limit: authored::card_0129::RULES.deck_limit }, effect: authored::card_0129::RULES.checked_effect(), triggers: authored::card_0129::RULES.triggers, activated: authored::card_0129::RULES.activated, enters: authored::card_0129::RULES.enters, entry: authored::card_0129::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 0, 0, 2, 1, 0], generic: 1 }, rules: &authored::card_0129::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Ant-Man's Army", oracle_id: Some("34c0f75d-10a9-428d-b202-3defb3949090"), mana_cost: "{2}{G}", type_line: "Creature — Insect", oracle_keywords: &["Treasure", "Food"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0174::RULES.keywords, deck_limit: authored::card_0174::RULES.deck_limit }, effect: authored::card_0174::RULES.checked_effect(), triggers: authored::card_0174::RULES.triggers, activated: authored::card_0174::RULES.activated, enters: authored::card_0174::RULES.enters, entry: authored::card_0174::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 2 }, rules: &authored::card_0174::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Ant-Man, Colony Commander", oracle_id: Some("b0ed9fbe-324d-467a-a326-c6f85a4c688a"), mana_cost: "{1}{G}{U}", type_line: "Legendary Creature — Human Rogue Hero", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_0175::RULES.keywords, deck_limit: authored::card_0175::RULES.deck_limit }, effect: authored::card_0175::RULES.checked_effect(), triggers: authored::card_0175::RULES.triggers, activated: authored::card_0175::RULES.activated, enters: authored::card_0175::RULES.enters, entry: authored::card_0175::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 1, 0], generic: 1 }, rules: &authored::card_0175::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Arc Reactor", oracle_id: Some("55f7b005-31ea-4865-a8f3-460d15bed795"), mana_cost: "{5}", type_line: "Artifact", oracle_keywords: &["Improvise"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0202::RULES.keywords, deck_limit: authored::card_0202::RULES.deck_limit }, effect: authored::card_0202::RULES.checked_effect(), triggers: authored::card_0202::RULES.triggers, activated: authored::card_0202::RULES.activated, enters: authored::card_0202::RULES.enters, entry: authored::card_0202::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 5 }, rules: &authored::card_0202::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Ares, God of War", oracle_id: Some("8a09d4a2-4796-4a15-ab75-a3a0b717f62d"), mana_cost: "{1}{B}{R}", type_line: "Legendary Creature — God Warrior Villain", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_0219::RULES.keywords, deck_limit: authored::card_0219::RULES.deck_limit }, effect: authored::card_0219::RULES.checked_effect(), triggers: authored::card_0219::RULES.triggers, activated: authored::card_0219::RULES.activated, enters: authored::card_0219::RULES.enters, entry: authored::card_0219::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 1, 0, 0], generic: 1 }, rules: &authored::card_0219::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Armor Wars", oracle_id: Some("1ad83cac-2c85-477b-ab00-1369f9683db5"), mana_cost: "{2}{U}{R}", type_line: "Enchantment — Saga", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0225::RULES.keywords, deck_limit: authored::card_0225::RULES.deck_limit }, effect: authored::card_0225::RULES.checked_effect(), triggers: authored::card_0225::RULES.triggers, activated: authored::card_0225::RULES.activated, enters: authored::card_0225::RULES.enters, entry: authored::card_0225::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 1, 0, 1, 0, 0], generic: 2 }, rules: &authored::card_0225::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Arnim Zola, Bio-Fanatic", oracle_id: Some("95161e23-e4c4-411f-bce5-58a02abc0958"), mana_cost: "{2}{B}", type_line: "Legendary Artifact Creature — Scientist Villain", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_0231::RULES.keywords, deck_limit: authored::card_0231::RULES.deck_limit }, effect: authored::card_0231::RULES.checked_effect(), triggers: authored::card_0231::RULES.triggers, activated: authored::card_0231::RULES.activated, enters: authored::card_0231::RULES.enters, entry: authored::card_0231::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 2 }, rules: &authored::card_0231::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Asgardian Citadel", oracle_id: Some("e3614df6-ce84-4180-ade1-a1751e8e3c9a"), mana_cost: "", type_line: "Land", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0237::RULES.keywords, deck_limit: authored::card_0237::RULES.deck_limit }, effect: authored::card_0237::RULES.checked_effect(), triggers: authored::card_0237::RULES.triggers, activated: authored::card_0237::RULES.activated, enters: authored::card_0237::RULES.enters, entry: authored::card_0237::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_0237::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Atlantean Cavalry", oracle_id: Some("e929b730-9aa2-49e1-9a65-50afc0fbe3e0"), mana_cost: "{2}{U}", type_line: "Creature — Merfolk Soldier", oracle_keywords: &["Vigilance"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0253::RULES.keywords, deck_limit: authored::card_0253::RULES.deck_limit }, effect: authored::card_0253::RULES.checked_effect(), triggers: authored::card_0253::RULES.triggers, activated: authored::card_0253::RULES.activated, enters: authored::card_0253::RULES.enters, entry: authored::card_0253::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_0253::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Atlantis Attacks", oracle_id: Some("b5c8e24a-f4d7-42dc-8117-905ad0bad888"), mana_cost: "{5}{U}{U}", type_line: "Sorcery", oracle_keywords: &["Teamwork"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0254::RULES.keywords, deck_limit: authored::card_0254::RULES.deck_limit }, effect: authored::card_0254::RULES.checked_effect(), triggers: authored::card_0254::RULES.triggers, activated: authored::card_0254::RULES.activated, enters: authored::card_0254::RULES.enters, entry: authored::card_0254::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 2, 0, 0, 0, 0], generic: 5 }, rules: &authored::card_0254::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Attuma, Atlantean Warlord", oracle_id: Some("e3b70a85-4296-411f-a673-7992b0c8a9aa"), mana_cost: "{2}{U}{U}", type_line: "Legendary Creature — Merfolk Warrior Villain", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_0260::RULES.keywords, deck_limit: authored::card_0260::RULES.deck_limit }, effect: authored::card_0260::RULES.checked_effect(), triggers: authored::card_0260::RULES.triggers, activated: authored::card_0260::RULES.activated, enters: authored::card_0260::RULES.enters, entry: authored::card_0260::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 2, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_0260::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Avengers Assemble!", oracle_id: Some("3477e1cc-46af-46f4-9ea2-bdf0bf1f880e"), mana_cost: "{4}{W}", type_line: "Enchantment", oracle_keywords: &["Flash"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0282::RULES.keywords, deck_limit: authored::card_0282::RULES.deck_limit }, effect: authored::card_0282::RULES.checked_effect(), triggers: authored::card_0282::RULES.triggers, activated: authored::card_0282::RULES.activated, enters: authored::card_0282::RULES.enters, entry: authored::card_0282::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 4 }, rules: &authored::card_0282::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Avengers Disassembled", oracle_id: Some("34493cf5-7586-48ef-806d-57273b2ab034"), mana_cost: "{1}{R}{R}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0283::RULES.keywords, deck_limit: authored::card_0283::RULES.deck_limit }, effect: authored::card_0283::RULES.checked_effect(), triggers: authored::card_0283::RULES.triggers, activated: authored::card_0283::RULES.activated, enters: authored::card_0283::RULES.enters, entry: authored::card_0283::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 0, 2, 0, 0], generic: 1 }, rules: &authored::card_0283::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Avengers Hangar", oracle_id: Some("10fc2009-4212-47c6-8a97-ab0f45c1f91b"), mana_cost: "", type_line: "Land", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0284::RULES.keywords, deck_limit: authored::card_0284::RULES.deck_limit }, effect: authored::card_0284::RULES.checked_effect(), triggers: authored::card_0284::RULES.triggers, activated: authored::card_0284::RULES.activated, enters: authored::card_0284::RULES.enters, entry: authored::card_0284::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_0284::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Avengers Tower", oracle_id: Some("c5fc8e7c-a87e-4586-a13c-d30e0a3aafbf"), mana_cost: "", type_line: "Land", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0285::RULES.keywords, deck_limit: authored::card_0285::RULES.deck_limit }, effect: authored::card_0285::RULES.checked_effect(), triggers: authored::card_0285::RULES.triggers, activated: authored::card_0285::RULES.activated, enters: authored::card_0285::RULES.enters, entry: authored::card_0285::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_0285::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Avengers: Under Siege", oracle_id: Some("d911cf65-7273-4162-a766-7d4107699dc2"), mana_cost: "{2}{B}{R}", type_line: "Enchantment — Saga", oracle_keywords: &["Treasure"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0286::RULES.keywords, deck_limit: authored::card_0286::RULES.deck_limit }, effect: authored::card_0286::RULES.checked_effect(), triggers: authored::card_0286::RULES.triggers, activated: authored::card_0286::RULES.activated, enters: authored::card_0286::RULES.enters, entry: authored::card_0286::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 0, 1, 1, 0, 0], generic: 2 }, rules: &authored::card_0286::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Baron Helmut Zemo", oracle_id: Some("0fe6c042-1182-4392-8938-fee36149d74f"), mana_cost: "{B}{B}{B}", type_line: "Legendary Creature — Human Noble Villain", oracle_keywords: &["Boast", "Connive"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_0333::RULES.keywords, deck_limit: authored::card_0333::RULES.deck_limit }, effect: authored::card_0333::RULES.checked_effect(), triggers: authored::card_0333::RULES.triggers, activated: authored::card_0333::RULES.activated, enters: authored::card_0333::RULES.enters, entry: authored::card_0333::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 3, 0, 0, 0], generic: 0 }, rules: &authored::card_0333::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Baron Strucker, HYDRA Overlord", oracle_id: Some("c91e64d4-e687-4ccd-a56e-796cff6e052c"), mana_cost: "{2}{B}", type_line: "Legendary Creature — Human Villain", oracle_keywords: &["Connive"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_0334::RULES.keywords, deck_limit: authored::card_0334::RULES.deck_limit }, effect: authored::card_0334::RULES.checked_effect(), triggers: authored::card_0334::RULES.triggers, activated: authored::card_0334::RULES.activated, enters: authored::card_0334::RULES.enters, entry: authored::card_0334::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 2 }, rules: &authored::card_0334::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Baxter Building", oracle_id: Some("71bc69a5-7cec-4abd-b97d-13f8e1f9afac"), mana_cost: "", type_line: "Land", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0352::RULES.keywords, deck_limit: authored::card_0352::RULES.deck_limit }, effect: authored::card_0352::RULES.checked_effect(), triggers: authored::card_0352::RULES.triggers, activated: authored::card_0352::RULES.activated, enters: authored::card_0352::RULES.enters, entry: authored::card_0352::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_0352::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Beast, Erudite Aerialist", oracle_id: Some("191ba5f0-45f4-4b7f-910d-4bd4d2d28b33"), mana_cost: "{3}{G/U}", type_line: "Legendary Creature — Mutant Scientist Hero", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_0360::RULES.keywords, deck_limit: authored::card_0360::RULES.deck_limit }, effect: authored::card_0360::RULES.checked_effect(), triggers: authored::card_0360::RULES.triggers, activated: authored::card_0360::RULES.activated, enters: authored::card_0360::RULES.enters, entry: authored::card_0360::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 3 }, rules: &authored::card_0360::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Birnin Zana Plaza", oracle_id: Some("c3172c5d-c745-4422-b651-cd414c8da11b"), mana_cost: "", type_line: "Land", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0421::RULES.keywords, deck_limit: authored::card_0421::RULES.deck_limit }, effect: authored::card_0421::RULES.checked_effect(), triggers: authored::card_0421::RULES.triggers, activated: authored::card_0421::RULES.activated, enters: authored::card_0421::RULES.enters, entry: authored::card_0421::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_0421::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Black Panther, Vanguard", oracle_id: Some("b0bf1964-c543-4c53-9f09-0dd7c608d5c5"), mana_cost: "{2}{G}{W}", type_line: "Legendary Creature — Human Warrior Hero", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_0431::RULES.keywords, deck_limit: authored::card_0431::RULES.deck_limit }, effect: authored::card_0431::RULES.checked_effect(), triggers: authored::card_0431::RULES.triggers, activated: authored::card_0431::RULES.activated, enters: authored::card_0431::RULES.enters, entry: authored::card_0431::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 1, 0], generic: 2 }, rules: &authored::card_0431::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Black Widow, Double Agent", oracle_id: Some("e6c181f1-f4f7-4a96-b50c-717cf2a02098"), mana_cost: "{1}{W}{B}", type_line: "Legendary Creature — Human Hero Villain", oracle_keywords: &["Deathtouch"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_0433::RULES.keywords, deck_limit: authored::card_0433::RULES.deck_limit }, effect: authored::card_0433::RULES.checked_effect(), triggers: authored::card_0433::RULES.triggers, activated: authored::card_0433::RULES.activated, enters: authored::card_0433::RULES.enters, entry: authored::card_0433::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 1, 0, 0, 0], generic: 1 }, rules: &authored::card_0433::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Black Widow, Super Spy", oracle_id: Some("579f9a86-a1af-4c2a-a7e5-8fea0bd0eb61"), mana_cost: "{1}{B}", type_line: "Legendary Creature — Human Spy Hero", oracle_keywords: &["Menace"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_0434::RULES.keywords, deck_limit: authored::card_0434::RULES.deck_limit }, effect: authored::card_0434::RULES.checked_effect(), triggers: authored::card_0434::RULES.triggers, activated: authored::card_0434::RULES.activated, enters: authored::card_0434::RULES.enters, entry: authored::card_0434::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 1 }, rules: &authored::card_0434::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Bold Biochemist", oracle_id: Some("45ddfa35-7b3c-4916-b231-dc721b7b56d9"), mana_cost: "{1}{U}", type_line: "Creature — Human Scientist", oracle_keywords: &["Power-up"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0484::RULES.keywords, deck_limit: authored::card_0484::RULES.deck_limit }, effect: authored::card_0484::RULES.checked_effect(), triggers: authored::card_0484::RULES.triggers, activated: authored::card_0484::RULES.activated, enters: authored::card_0484::RULES.enters, entry: authored::card_0484::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_0484::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Borough Backup", oracle_id: Some("6effbf59-3933-4a77-9a6a-fd3bedf45c26"), mana_cost: "{4}{W}", type_line: "Sorcery", oracle_keywords: &["Landcycling", "Basic landcycling", "Typecycling", "Cycling"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0507::RULES.keywords, deck_limit: authored::card_0507::RULES.deck_limit }, effect: authored::card_0507::RULES.checked_effect(), triggers: authored::card_0507::RULES.triggers, activated: authored::card_0507::RULES.activated, enters: authored::card_0507::RULES.enters, entry: authored::card_0507::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 4 }, rules: &authored::card_0507::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Brave Brawler", oracle_id: Some("a1a791be-71e6-4c1f-8717-08ca62d40de2"), mana_cost: "{1}{W}", type_line: "Creature — Human Warrior Hero", oracle_keywords: &["Power-up", "Lifelink"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0529::RULES.keywords, deck_limit: authored::card_0529::RULES.deck_limit }, effect: authored::card_0529::RULES.checked_effect(), triggers: authored::card_0529::RULES.triggers, activated: authored::card_0529::RULES.activated, enters: authored::card_0529::RULES.enters, entry: authored::card_0529::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_0529::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Bruce Banner", oracle_id: Some("b6c0ee14-2028-4e8b-9acd-ff78821bca9b"), mana_cost: "{U}", type_line: "Legendary Creature — Human Scientist Hero", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_0571::RULES.keywords, deck_limit: authored::card_0571::RULES.deck_limit }, effect: authored::card_0571::RULES.checked_effect(), triggers: authored::card_0571::RULES.triggers, activated: authored::card_0571::RULES.activated, enters: authored::card_0571::RULES.enters, entry: authored::card_0571::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_0571::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Bullseye, Death Dealer", oracle_id: Some("10db57b4-f895-4514-b610-2252ba676353"), mana_cost: "{2}{B/R}", type_line: "Legendary Creature — Human Assassin Villain", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_0579::RULES.keywords, deck_limit: authored::card_0579::RULES.deck_limit }, effect: authored::card_0579::RULES.checked_effect(), triggers: authored::card_0579::RULES.triggers, activated: authored::card_0579::RULES.activated, enters: authored::card_0579::RULES.enters, entry: authored::card_0579::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 2 }, rules: &authored::card_0579::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Call Damage Control", oracle_id: Some("49a32f34-01a9-4f25-bf38-b4d46ca67b2d"), mana_cost: "{1}{G}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0619::RULES.keywords, deck_limit: authored::card_0619::RULES.deck_limit }, effect: authored::card_0619::RULES.checked_effect(), triggers: authored::card_0619::RULES.triggers, activated: authored::card_0619::RULES.activated, enters: authored::card_0619::RULES.enters, entry: authored::card_0619::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 1 }, rules: &authored::card_0619::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Captain America's Shield", oracle_id: Some("8255a797-ddd3-4db0-a3f5-dd8f38c3b908"), mana_cost: "{2}", type_line: "Legendary Artifact — Equipment", oracle_keywords: &["Indestructible", "Equip"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_0641::RULES.keywords, deck_limit: authored::card_0641::RULES.deck_limit }, effect: authored::card_0641::RULES.checked_effect(), triggers: authored::card_0641::RULES.triggers, activated: authored::card_0641::RULES.activated, enters: authored::card_0641::RULES.enters, entry: authored::card_0641::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_0641::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Captain America, Living Legend", oracle_id: Some("67aa946d-e694-494e-ba0c-44880d5bc03e"), mana_cost: "{1}{W}{U}", type_line: "Legendary Creature — Human Soldier Hero", oracle_keywords: &["Vigilance"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_0642::RULES.keywords, deck_limit: authored::card_0642::RULES.deck_limit }, effect: authored::card_0642::RULES.checked_effect(), triggers: authored::card_0642::RULES.triggers, activated: authored::card_0642::RULES.activated, enters: authored::card_0642::RULES.enters, entry: authored::card_0642::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 1, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_0642::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Captain America, Super-Soldier", oracle_id: Some("ac1b6ffc-12a9-4d9c-a9fc-68a839a40738"), mana_cost: "{1}{W}{W}", type_line: "Legendary Creature — Human Soldier Hero", oracle_keywords: &["First strike"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_0643::RULES.keywords, deck_limit: authored::card_0643::RULES.deck_limit }, effect: authored::card_0643::RULES.checked_effect(), triggers: authored::card_0643::RULES.triggers, activated: authored::card_0643::RULES.activated, enters: authored::card_0643::RULES.enters, entry: authored::card_0643::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [2, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_0643::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Captain America, Wings of Freedom", oracle_id: Some("942ac2ac-7528-4768-8398-8520a8ee15e5"), mana_cost: "{2}{W}", type_line: "Legendary Creature — Human Soldier Hero", oracle_keywords: &["Flying", "First strike", "Ward"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_0644::RULES.keywords, deck_limit: authored::card_0644::RULES.deck_limit }, effect: authored::card_0644::RULES.checked_effect(), triggers: authored::card_0644::RULES.triggers, activated: authored::card_0644::RULES.activated, enters: authored::card_0644::RULES.enters, entry: authored::card_0644::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_0644::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Captain Mar-Vell, Space-Born", oracle_id: Some("b74c072d-77ac-460d-9d9c-75b25874f58b"), mana_cost: "{4}{W}", type_line: "Legendary Creature — Kree Soldier Hero", oracle_keywords: &["Flying", "Vigilance", "Cosmic Awareness"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_0646::RULES.keywords, deck_limit: authored::card_0646::RULES.deck_limit }, effect: authored::card_0646::RULES.checked_effect(), triggers: authored::card_0646::RULES.triggers, activated: authored::card_0646::RULES.activated, enters: authored::card_0646::RULES.enters, entry: authored::card_0646::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 4 }, rules: &authored::card_0646::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Captain Marvel, Earth's Protector", oracle_id: Some("2e665a03-bece-43df-bd15-b4496e233f2e"), mana_cost: "{3}{W}{W}", type_line: "Legendary Creature — Human Kree Hero", oracle_keywords: &["Flying", "Power-up", "Lifelink", "Flash"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_0647::RULES.keywords, deck_limit: authored::card_0647::RULES.deck_limit }, effect: authored::card_0647::RULES.checked_effect(), triggers: authored::card_0647::RULES.triggers, activated: authored::card_0647::RULES.activated, enters: authored::card_0647::RULES.enters, entry: authored::card_0647::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [2, 0, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_0647::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Castle Doom", oracle_id: Some("9bd013df-ad75-4099-940b-1765c58faf26"), mana_cost: "", type_line: "Land", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0676::RULES.keywords, deck_limit: authored::card_0676::RULES.deck_limit }, effect: authored::card_0676::RULES.checked_effect(), triggers: authored::card_0676::RULES.triggers, activated: authored::card_0676::RULES.activated, enters: authored::card_0676::RULES.enters, entry: authored::card_0676::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_0676::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Claim the Kingdom", oracle_id: Some("3cd15225-e9b6-4998-b545-76536c72e948"), mana_cost: "{1}{G}", type_line: "Enchantment — Plan", oracle_keywords: &["Landfall"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0759::RULES.keywords, deck_limit: authored::card_0759::RULES.deck_limit }, effect: authored::card_0759::RULES.checked_effect(), triggers: authored::card_0759::RULES.triggers, activated: authored::card_0759::RULES.activated, enters: authored::card_0759::RULES.enters, entry: authored::card_0759::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 1 }, rules: &authored::card_0759::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Cloak and Dagger, Entwined", oracle_id: Some("64132f93-66ac-4794-8c07-a88f9ed0e22b"), mana_cost: "{1}{W}{B}", type_line: "Legendary Creature — Human Hero", oracle_keywords: &["Lifelink", "Deathtouch"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_0773::RULES.keywords, deck_limit: authored::card_0773::RULES.deck_limit }, effect: authored::card_0773::RULES.checked_effect(), triggers: authored::card_0773::RULES.triggers, activated: authored::card_0773::RULES.activated, enters: authored::card_0773::RULES.enters, entry: authored::card_0773::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 1, 0, 0, 0], generic: 1 }, rules: &authored::card_0773::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Colleen Wing, Street Samurai", oracle_id: Some("685181dd-0124-4c76-9270-f9dd17e6a221"), mana_cost: "{1}{W}", type_line: "Legendary Creature — Human Samurai Hero", oracle_keywords: &["Scry"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_0798::RULES.keywords, deck_limit: authored::card_0798::RULES.deck_limit }, effect: authored::card_0798::RULES.checked_effect(), triggers: authored::card_0798::RULES.triggers, activated: authored::card_0798::RULES.activated, enters: authored::card_0798::RULES.enters, entry: authored::card_0798::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_0798::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Construct a Cosmic Cube", oracle_id: Some("369aa9a6-af8a-432c-adff-ced2bb203dc7"), mana_cost: "{2}{B}", type_line: "Enchantment — Plan", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0835::RULES.keywords, deck_limit: authored::card_0835::RULES.deck_limit }, effect: authored::card_0835::RULES.checked_effect(), triggers: authored::card_0835::RULES.triggers, activated: authored::card_0835::RULES.activated, enters: authored::card_0835::RULES.enters, entry: authored::card_0835::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 2 }, rules: &authored::card_0835::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Cosmic Cube", oracle_id: Some("0866a630-d962-4aa9-a15f-08b2947540ef"), mana_cost: "{5}", type_line: "Artifact", oracle_keywords: &["Ward"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0859::RULES.keywords, deck_limit: authored::card_0859::RULES.deck_limit }, effect: authored::card_0859::RULES.checked_effect(), triggers: authored::card_0859::RULES.triggers, activated: authored::card_0859::RULES.activated, enters: authored::card_0859::RULES.enters, entry: authored::card_0859::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 5 }, rules: &authored::card_0859::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Crossbones, Malicious Mercenary", oracle_id: Some("ac950d10-e373-42d3-907b-ee56e45a2084"), mana_cost: "{3}{B}", type_line: "Legendary Creature — Human Mercenary Villain", oracle_keywords: &["Deathtouch"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_0892::RULES.keywords, deck_limit: authored::card_0892::RULES.deck_limit }, effect: authored::card_0892::RULES.checked_effect(), triggers: authored::card_0892::RULES.triggers, activated: authored::card_0892::RULES.activated, enters: authored::card_0892::RULES.enters, entry: authored::card_0892::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 3 }, rules: &authored::card_0892::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Crowd of True Believers", oracle_id: Some("06296d4d-a1b3-4e9a-a2fd-a0183bb892a1"), mana_cost: "{W}", type_line: "Creature — Human Citizen", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0897::RULES.keywords, deck_limit: authored::card_0897::RULES.deck_limit }, effect: authored::card_0897::RULES.checked_effect(), triggers: authored::card_0897::RULES.triggers, activated: authored::card_0897::RULES.activated, enters: authored::card_0897::RULES.enters, entry: authored::card_0897::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_0897::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Cruel Alliance", oracle_id: Some("6897f9e0-f654-4c0a-9fda-2ad4e264bf9a"), mana_cost: "{2}{B}", type_line: "Sorcery", oracle_keywords: &["Teamwork"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0901::RULES.keywords, deck_limit: authored::card_0901::RULES.deck_limit }, effect: authored::card_0901::RULES.checked_effect(), triggers: authored::card_0901::RULES.triggers, activated: authored::card_0901::RULES.activated, enters: authored::card_0901::RULES.enters, entry: authored::card_0901::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 2 }, rules: &authored::card_0901::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Daredevil, Man Without Fear", oracle_id: Some("88aadf60-91f9-4932-a2bc-63365a139d36"), mana_cost: "{2}{R}{W}", type_line: "Legendary Creature — Human Hero", oracle_keywords: &["Radar Sense", "Vigilance", "Haste"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_0957::RULES.keywords, deck_limit: authored::card_0957::RULES.deck_limit }, effect: authored::card_0957::RULES.checked_effect(), triggers: authored::card_0957::RULES.triggers, activated: authored::card_0957::RULES.activated, enters: authored::card_0957::RULES.enters, entry: authored::card_0957::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 1, 0, 0], generic: 2 }, rules: &authored::card_0957::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Dark Deed", oracle_id: Some("e98cd00c-b812-4c6a-81d4-e3f94b64fc58"), mana_cost: "{1}{B}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0964::RULES.keywords, deck_limit: authored::card_0964::RULES.deck_limit }, effect: authored::card_0964::RULES.checked_effect(), triggers: authored::card_0964::RULES.triggers, activated: authored::card_0964::RULES.activated, enters: authored::card_0964::RULES.enters, entry: authored::card_0964::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 1 }, rules: &authored::card_0964::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Dark Fortress", oracle_id: Some("40760bfa-a423-487c-ba29-043b2d00c736"), mana_cost: "", type_line: "Land", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0966::RULES.keywords, deck_limit: authored::card_0966::RULES.deck_limit }, effect: authored::card_0966::RULES.checked_effect(), triggers: authored::card_0966::RULES.triggers, activated: authored::card_0966::RULES.activated, enters: authored::card_0966::RULES.enters, entry: authored::card_0966::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_0966::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Death to Our Enemies", oracle_id: Some("c2b2b743-dfc7-439e-8563-1ccce54422bd"), mana_cost: "{2}{R}", type_line: "Enchantment — Plan", oracle_keywords: &["Treasure"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1004::RULES.keywords, deck_limit: authored::card_1004::RULES.deck_limit }, effect: authored::card_1004::RULES.checked_effect(), triggers: authored::card_1004::RULES.triggers, activated: authored::card_1004::RULES.activated, enters: authored::card_1004::RULES.enters, entry: authored::card_1004::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 2 }, rules: &authored::card_1004::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Decoy Ploy", oracle_id: Some("035cddbc-39c9-42af-9e26-c88419b43b49"), mana_cost: "{1}{B}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1016::RULES.keywords, deck_limit: authored::card_1016::RULES.deck_limit }, effect: authored::card_1016::RULES.checked_effect(), triggers: authored::card_1016::RULES.triggers, activated: authored::card_1016::RULES.activated, enters: authored::card_1016::RULES.enters, entry: authored::card_1016::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 1 }, rules: &authored::card_1016::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Dependable Quinjet", oracle_id: Some("60105eb8-bdca-44b9-a970-68a7615526e8"), mana_cost: "{3}", type_line: "Artifact — Vehicle", oracle_keywords: &["Flying", "Crew"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1044::RULES.keywords, deck_limit: authored::card_1044::RULES.deck_limit }, effect: authored::card_1044::RULES.checked_effect(), triggers: authored::card_1044::RULES.triggers, activated: authored::card_1044::RULES.activated, enters: authored::card_1044::RULES.enters, entry: authored::card_1044::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_1044::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Depower", oracle_id: Some("33f89437-885b-4850-88b7-239f5dfcc8b1"), mana_cost: "{2}{U}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1045::RULES.keywords, deck_limit: authored::card_1045::RULES.deck_limit }, effect: authored::card_1045::RULES.checked_effect(), triggers: authored::card_1045::RULES.triggers, activated: authored::card_1045::RULES.activated, enters: authored::card_1045::RULES.enters, entry: authored::card_1045::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_1045::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Doc Samson, Super Psychiatrist", oracle_id: Some("699610c9-c991-4faf-b712-d015ef0799ad"), mana_cost: "{4}{G}", type_line: "Legendary Creature — Gamma Doctor Hero", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_1117::RULES.keywords, deck_limit: authored::card_1117::RULES.deck_limit }, effect: authored::card_1117::RULES.checked_effect(), triggers: authored::card_1117::RULES.triggers, activated: authored::card_1117::RULES.activated, enters: authored::card_1117::RULES.enters, entry: authored::card_1117::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 4 }, rules: &authored::card_1117::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Doctor Doom", oracle_id: Some("9e4fa592-71f8-4c71-8c91-78e6326cd736"), mana_cost: "{4}{B}{B}", type_line: "Legendary Creature — Human Scientist Villain", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_1119::RULES.keywords, deck_limit: authored::card_1119::RULES.deck_limit }, effect: authored::card_1119::RULES.checked_effect(), triggers: authored::card_1119::RULES.triggers, activated: authored::card_1119::RULES.activated, enters: authored::card_1119::RULES.enters, entry: authored::card_1119::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 2, 0, 0, 0], generic: 4 }, rules: &authored::card_1119::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Doom Reigns Supreme", oracle_id: Some("f398a742-22c1-4da9-9d49-fdce2eb93d01"), mana_cost: "{1}{B}", type_line: "Enchantment — Plan", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1132::RULES.keywords, deck_limit: authored::card_1132::RULES.deck_limit }, effect: authored::card_1132::RULES.checked_effect(), triggers: authored::card_1132::RULES.triggers, activated: authored::card_1132::RULES.activated, enters: authored::card_1132::RULES.enters, entry: authored::card_1132::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 1 }, rules: &authored::card_1132::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Earth's Mightiest Heroes", oracle_id: Some("2b87bcac-ec64-4012-9e75-f459d3640eb3"), mana_cost: "{4}{G}{G}", type_line: "Sorcery", oracle_keywords: &["Teamwork"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1240::RULES.keywords, deck_limit: authored::card_1240::RULES.deck_limit }, effect: authored::card_1240::RULES.checked_effect(), triggers: authored::card_1240::RULES.triggers, activated: authored::card_1240::RULES.activated, enters: authored::card_1240::RULES.enters, entry: authored::card_1240::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 0, 0, 2, 0], generic: 4 }, rules: &authored::card_1240::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Echo, Perceptive Prodigy", oracle_id: Some("1091c83a-a4b9-457f-a69a-9a44a7e10759"), mana_cost: "{2}{U}", type_line: "Legendary Creature — Human Hero", oracle_keywords: &["Vigilance"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_1250::RULES.keywords, deck_limit: authored::card_1250::RULES.deck_limit }, effect: authored::card_1250::RULES.checked_effect(), triggers: authored::card_1250::RULES.triggers, activated: authored::card_1250::RULES.activated, enters: authored::card_1250::RULES.enters, entry: authored::card_1250::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_1250::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Elektra, Daughter of the Hand", oracle_id: Some("b46647a1-0c91-451b-8c0b-be8fe02faee2"), mana_cost: "{2}{B}{B}", type_line: "Legendary Creature — Human Ninja Villain", oracle_keywords: &["Sneak"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_1281::RULES.keywords, deck_limit: authored::card_1281::RULES.deck_limit }, effect: authored::card_1281::RULES.checked_effect(), triggers: authored::card_1281::RULES.triggers, activated: authored::card_1281::RULES.activated, enters: authored::card_1281::RULES.enters, entry: authored::card_1281::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 2, 0, 0, 0], generic: 2 }, rules: &authored::card_1281::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Epic Fight", oracle_id: Some("4bdfc580-de51-445b-b170-8c6832e44704"), mana_cost: "{2}{G}", type_line: "Sorcery", oracle_keywords: &["Fight", "Double"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1353::RULES.keywords, deck_limit: authored::card_1353::RULES.deck_limit }, effect: authored::card_1353::RULES.checked_effect(), triggers: authored::card_1353::RULES.triggers, activated: authored::card_1353::RULES.activated, enters: authored::card_1353::RULES.enters, entry: authored::card_1353::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 2 }, rules: &authored::card_1353::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Evil's Thrall", oracle_id: Some("d7562f89-4808-44ab-8dc9-fc4078e8a47b"), mana_cost: "{2}{R}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1390::RULES.keywords, deck_limit: authored::card_1390::RULES.deck_limit }, effect: authored::card_1390::RULES.checked_effect(), triggers: authored::card_1390::RULES.triggers, activated: authored::card_1390::RULES.activated, enters: authored::card_1390::RULES.enters, entry: authored::card_1390::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 2 }, rules: &authored::card_1390::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Falcon's Wing Harness", oracle_id: Some("e69db5f0-beed-48e8-bc25-768b1e8be5ea"), mana_cost: "{1}{U}", type_line: "Artifact — Equipment", oracle_keywords: &["Equip"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1432::RULES.keywords, deck_limit: authored::card_1432::RULES.deck_limit }, effect: authored::card_1432::RULES.checked_effect(), triggers: authored::card_1432::RULES.triggers, activated: authored::card_1432::RULES.activated, enters: authored::card_1432::RULES.enters, entry: authored::card_1432::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_1432::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Falcon, Winged Wonder", oracle_id: Some("92e222c2-28e6-4d43-8b48-480ebcd88495"), mana_cost: "{4}{U}", type_line: "Legendary Creature — Human Hero", oracle_keywords: &["Flying", "Surveil", "Avian Telepathy"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_1433::RULES.keywords, deck_limit: authored::card_1433::RULES.deck_limit }, effect: authored::card_1433::RULES.checked_effect(), triggers: authored::card_1433::RULES.triggers, activated: authored::card_1433::RULES.activated, enters: authored::card_1433::RULES.enters, entry: authored::card_1433::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 4 }, rules: &authored::card_1433::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Fin Fang Foom", oracle_id: Some("52ec472e-a745-4e25-b9a7-8e1eb8873594"), mana_cost: "{2}{R}{R}", type_line: "Legendary Creature — Alien Dragon Villain", oracle_keywords: &["Flying"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_1508::RULES.keywords, deck_limit: authored::card_1508::RULES.deck_limit }, effect: authored::card_1508::RULES.checked_effect(), triggers: authored::card_1508::RULES.triggers, activated: authored::card_1508::RULES.activated, enters: authored::card_1508::RULES.enters, entry: authored::card_1508::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 2, 0, 0], generic: 2 }, rules: &authored::card_1508::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Fisk Tower", oracle_id: Some("5554db95-9676-4c24-b82a-8173513d7927"), mana_cost: "", type_line: "Land", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1538::RULES.keywords, deck_limit: authored::card_1538::RULES.deck_limit }, effect: authored::card_1538::RULES.checked_effect(), triggers: authored::card_1538::RULES.triggers, activated: authored::card_1538::RULES.activated, enters: authored::card_1538::RULES.enters, entry: authored::card_1538::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_1538::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Frozen in Ice", oracle_id: Some("82fa14d8-5019-4e1b-a86c-923d9292b87a"), mana_cost: "{2}{U}", type_line: "Enchantment — Aura", oracle_keywords: &["Enchant"], standard_legal: true, legendary: false, implemented: true, keywords: authored::frozen_in_ice::RULES.keywords, deck_limit: authored::frozen_in_ice::RULES.deck_limit }, effect: authored::frozen_in_ice::RULES.checked_effect(), triggers: authored::frozen_in_ice::RULES.triggers, activated: authored::frozen_in_ice::RULES.activated, enters: authored::frozen_in_ice::RULES.enters, entry: authored::frozen_in_ice::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 2 }, rules: &authored::frozen_in_ice::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Futurist Forge", oracle_id: Some("0e5f97b5-bb8c-4688-a141-5db4d4eeccf0"), mana_cost: "{1}{U}", type_line: "Artifact", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1655::RULES.keywords, deck_limit: authored::card_1655::RULES.deck_limit }, effect: authored::card_1655::RULES.checked_effect(), triggers: authored::card_1655::RULES.triggers, activated: authored::card_1655::RULES.activated, enters: authored::card_1655::RULES.enters, entry: authored::card_1655::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_1655::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Gathering Place", oracle_id: Some("36b58705-c5a5-4547-8d8b-a7c35e1f69ae"), mana_cost: "", type_line: "Land", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1698::RULES.keywords, deck_limit: authored::card_1698::RULES.deck_limit }, effect: authored::card_1698::RULES.checked_effect(), triggers: authored::card_1698::RULES.triggers, activated: authored::card_1698::RULES.activated, enters: authored::card_1698::RULES.enters, entry: authored::card_1698::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_1698::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Ghost, Spectral Saboteur", oracle_id: Some("88f34d15-c338-40a7-b56c-c5b650847430"), mana_cost: "{2}{U/B}", type_line: "Legendary Creature — Human Rogue Villain", oracle_keywords: &["Intangibility", "Flash"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_1733::RULES.keywords, deck_limit: authored::card_1733::RULES.deck_limit }, effect: authored::card_1733::RULES.checked_effect(), triggers: authored::card_1733::RULES.triggers, activated: authored::card_1733::RULES.activated, enters: authored::card_1733::RULES.enters, entry: authored::card_1733::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 2 }, rules: &authored::card_1733::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Giant-Sized Flying Ant", oracle_id: Some("233e3b4b-faaf-451d-8a2e-e9f5a6ced671"), mana_cost: "{3}{U}", type_line: "Creature — Insect", oracle_keywords: &["Flying", "Flash"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1741::RULES.keywords, deck_limit: authored::card_1741::RULES.deck_limit }, effect: authored::card_1741::RULES.checked_effect(), triggers: authored::card_1741::RULES.triggers, activated: authored::card_1741::RULES.activated, enters: authored::card_1741::RULES.enters, entry: authored::card_1741::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_1741::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Gleaming Bastion", oracle_id: Some("7785ffd4-f169-475d-9558-ce4877b3378a"), mana_cost: "", type_line: "Land", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1771::RULES.keywords, deck_limit: authored::card_1771::RULES.deck_limit }, effect: authored::card_1771::RULES.checked_effect(), triggers: authored::card_1771::RULES.triggers, activated: authored::card_1771::RULES.activated, enters: authored::card_1771::RULES.enters, entry: authored::card_1771::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_1771::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Go Nuts!", oracle_id: Some("0efe1357-bfbb-42c0-9cc9-6a919d686c66"), mana_cost: "{G}", type_line: "Sorcery", oracle_keywords: &["Fight", "Teamwork"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1799::RULES.keywords, deck_limit: authored::card_1799::RULES.deck_limit }, effect: authored::card_1799::RULES.checked_effect(), triggers: authored::card_1799::RULES.triggers, activated: authored::card_1799::RULES.activated, enters: authored::card_1799::RULES.enters, entry: authored::card_1799::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 0 }, rules: &authored::card_1799::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Grim Reaper, Lethal Legionnaire", oracle_id: Some("edf429a8-6a4d-4d2a-b0ca-c0c72a0698a3"), mana_cost: "{3}{B}", type_line: "Legendary Creature — Human Villain", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_1881::RULES.keywords, deck_limit: authored::card_1881::RULES.deck_limit }, effect: authored::card_1881::RULES.checked_effect(), triggers: authored::card_1881::RULES.triggers, activated: authored::card_1881::RULES.activated, enters: authored::card_1881::RULES.enters, entry: authored::card_1881::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 3 }, rules: &authored::card_1881::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Guerrilla Gorilla", oracle_id: Some("6cb65b68-91b2-45ba-867b-3b31d6dd5643"), mana_cost: "{1}{G}", type_line: "Creature — Ape Soldier Hero", oracle_keywords: &["Reach"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1902::RULES.keywords, deck_limit: authored::card_1902::RULES.deck_limit }, effect: authored::card_1902::RULES.checked_effect(), triggers: authored::card_1902::RULES.triggers, activated: authored::card_1902::RULES.activated, enters: authored::card_1902::RULES.enters, entry: authored::card_1902::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 1 }, rules: &authored::card_1902::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "H.E.R.B.I.E. Scout Unit", oracle_id: Some("393ff8c5-4da0-4290-b498-222cc0244026"), mana_cost: "{4}", type_line: "Artifact Creature — Robot Scout", oracle_keywords: &["Flying"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1920::RULES.keywords, deck_limit: authored::card_1920::RULES.deck_limit }, effect: authored::card_1920::RULES.checked_effect(), triggers: authored::card_1920::RULES.triggers, activated: authored::card_1920::RULES.activated, enters: authored::card_1920::RULES.enters, entry: authored::card_1920::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 4 }, rules: &authored::card_1920::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "HULK SMASH!", oracle_id: Some("5f97d49c-d0fa-4776-9455-93c1a172cd83"), mana_cost: "{1}{R}", type_line: "Instant", oracle_keywords: &["Teamwork"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1921::RULES.keywords, deck_limit: authored::card_1921::RULES.deck_limit }, effect: authored::card_1921::RULES.checked_effect(), triggers: authored::card_1921::RULES.triggers, activated: authored::card_1921::RULES.activated, enters: authored::card_1921::RULES.enters, entry: authored::card_1921::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 1 }, rules: &authored::card_1921::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "HYDRA Assault Robot", oracle_id: Some("a2558476-5506-4a09-93fc-b8ab33c98985"), mana_cost: "{1}{R}", type_line: "Artifact Creature — Robot Villain", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1922::RULES.keywords, deck_limit: authored::card_1922::RULES.deck_limit }, effect: authored::card_1922::RULES.checked_effect(), triggers: authored::card_1922::RULES.triggers, activated: authored::card_1922::RULES.activated, enters: authored::card_1922::RULES.enters, entry: authored::card_1922::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 1 }, rules: &authored::card_1922::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "HYDRA Infiltration", oracle_id: Some("27692079-a517-4d38-b452-7409980b5912"), mana_cost: "{3}{B}", type_line: "Enchantment", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1923::RULES.keywords, deck_limit: authored::card_1923::RULES.deck_limit }, effect: authored::card_1923::RULES.checked_effect(), triggers: authored::card_1923::RULES.triggers, activated: authored::card_1923::RULES.activated, enters: authored::card_1923::RULES.enters, entry: authored::card_1923::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 3 }, rules: &authored::card_1923::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "HYDRA Troopers", oracle_id: Some("6de67fad-0d01-443d-9ae0-4126ae903069"), mana_cost: "{2}{B}", type_line: "Creature — Human Soldier Villain", oracle_keywords: &["Mill"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1924::RULES.keywords, deck_limit: authored::card_1924::RULES.deck_limit }, effect: authored::card_1924::RULES.checked_effect(), triggers: authored::card_1924::RULES.triggers, activated: authored::card_1924::RULES.activated, enters: authored::card_1924::RULES.enters, entry: authored::card_1924::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 2 }, rules: &authored::card_1924::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Hawkeye's Bow", oracle_id: Some("83798eb8-b14f-4620-bce2-1a6804a24dc7"), mana_cost: "{R}", type_line: "Artifact — Equipment", oracle_keywords: &["Equip"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1966::RULES.keywords, deck_limit: authored::card_1966::RULES.deck_limit }, effect: authored::card_1966::RULES.checked_effect(), triggers: authored::card_1966::RULES.triggers, activated: authored::card_1966::RULES.activated, enters: authored::card_1966::RULES.enters, entry: authored::card_1966::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 0 }, rules: &authored::card_1966::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Hawkeye, Master Marksman", oracle_id: Some("6b59bb1f-f2d4-4d47-bd57-23927d588381"), mana_cost: "{1}{R}", type_line: "Legendary Creature — Human Archer Hero", oracle_keywords: &["Explosive", "Reach", "First strike", "Trick Arrows", "Boomerang", "Net"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_1967::RULES.keywords, deck_limit: authored::card_1967::RULES.deck_limit }, effect: authored::card_1967::RULES.checked_effect(), triggers: authored::card_1967::RULES.triggers, activated: authored::card_1967::RULES.activated, enters: authored::card_1967::RULES.enters, entry: authored::card_1967::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 1 }, rules: &authored::card_1967::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Hawkeye, Young Avenger", oracle_id: Some("c8f8b4ed-8455-45cd-a24c-e2f42cd5e5b1"), mana_cost: "{3}{R}", type_line: "Legendary Creature — Human Archer Hero", oracle_keywords: &["Reach"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_1968::RULES.keywords, deck_limit: authored::card_1968::RULES.deck_limit }, effect: authored::card_1968::RULES.checked_effect(), triggers: authored::card_1968::RULES.triggers, activated: authored::card_1968::RULES.activated, enters: authored::card_1968::RULES.enters, entry: authored::card_1968::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 3 }, rules: &authored::card_1968::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Helicarrier Strike", oracle_id: Some("ca3cda24-0ecd-4edf-b1b3-54311ad58a51"), mana_cost: "{W}", type_line: "Instant", oracle_keywords: &["Teamwork"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1995::RULES.keywords, deck_limit: authored::card_1995::RULES.deck_limit }, effect: authored::card_1995::RULES.checked_effect(), triggers: authored::card_1995::RULES.triggers, activated: authored::card_1995::RULES.activated, enters: authored::card_1995::RULES.enters, entry: authored::card_1995::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_1995::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Hell's Kitchen", oracle_id: Some("1337d89e-a883-4568-a14f-9ca3305cc3df"), mana_cost: "", type_line: "Land", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1997::RULES.keywords, deck_limit: authored::card_1997::RULES.deck_limit }, effect: authored::card_1997::RULES.checked_effect(), triggers: authored::card_1997::RULES.triggers, activated: authored::card_1997::RULES.activated, enters: authored::card_1997::RULES.enters, entry: authored::card_1997::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_1997::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Hellcat, Undying Vigilante", oracle_id: Some("4c27ba61-6812-4dd7-8efc-65435a8df7b5"), mana_cost: "{G}{G}", type_line: "Legendary Creature — Human Hero", oracle_keywords: &["Haste"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_1998::RULES.keywords, deck_limit: authored::card_1998::RULES.deck_limit }, effect: authored::card_1998::RULES.checked_effect(), triggers: authored::card_1998::RULES.triggers, activated: authored::card_1998::RULES.activated, enters: authored::card_1998::RULES.enters, entry: authored::card_1998::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 2, 0], generic: 0 }, rules: &authored::card_1998::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Hercules, Prince of Power", oracle_id: Some("c3a8cc79-3771-484d-84dd-1a97f8bd12c2"), mana_cost: "{2}{G}", type_line: "Legendary Creature — Demigod Warrior Hero", oracle_keywords: &["Power-up"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2009::RULES.keywords, deck_limit: authored::card_2009::RULES.deck_limit }, effect: authored::card_2009::RULES.checked_effect(), triggers: authored::card_2009::RULES.triggers, activated: authored::card_2009::RULES.activated, enters: authored::card_2009::RULES.enters, entry: authored::card_2009::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 2 }, rules: &authored::card_2009::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Heroic Feast", oracle_id: Some("31cae00a-3a74-4a59-80b0-a656f524e878"), mana_cost: "{2}{G}", type_line: "Enchantment", oracle_keywords: &["Food"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2018::RULES.keywords, deck_limit: authored::card_2018::RULES.deck_limit }, effect: authored::card_2018::RULES.checked_effect(), triggers: authored::card_2018::RULES.triggers, activated: authored::card_2018::RULES.activated, enters: authored::card_2018::RULES.enters, entry: authored::card_2018::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 2 }, rules: &authored::card_2018::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Hex Magic", oracle_id: Some("2283729c-11db-471c-9f63-46092c97bd00"), mana_cost: "{2}{R}", type_line: "Sorcery — Arcane", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2020::RULES.keywords, deck_limit: authored::card_2020::RULES.deck_limit }, effect: authored::card_2020::RULES.checked_effect(), triggers: authored::card_2020::RULES.triggers, activated: authored::card_2020::RULES.activated, enters: authored::card_2020::RULES.enters, entry: authored::card_2020::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 2 }, rules: &authored::card_2020::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Hidden Lair", oracle_id: Some("7069d241-4e66-40bf-afd1-551a4a5457f0"), mana_cost: "", type_line: "Land", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2028::RULES.keywords, deck_limit: authored::card_2028::RULES.deck_limit }, effect: authored::card_2028::RULES.checked_effect(), triggers: authored::card_2028::RULES.triggers, activated: authored::card_2028::RULES.activated, enters: authored::card_2028::RULES.enters, entry: authored::card_2028::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_2028::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Hire a Crew", oracle_id: Some("6eb7a870-b626-4326-9e4c-45193f8ce138"), mana_cost: "{2}{R}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2047::RULES.keywords, deck_limit: authored::card_2047::RULES.deck_limit }, effect: authored::card_2047::RULES.checked_effect(), triggers: authored::card_2047::RULES.triggers, activated: authored::card_2047::RULES.activated, enters: authored::card_2047::RULES.enters, entry: authored::card_2047::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 2 }, rules: &authored::card_2047::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Hour of Defeat", oracle_id: Some("2774fcb7-8988-4a18-b5ec-7534dd8b2e05"), mana_cost: "{3}{B}", type_line: "Instant", oracle_keywords: &["Surveil"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2080::RULES.keywords, deck_limit: authored::card_2080::RULES.deck_limit }, effect: authored::card_2080::RULES.checked_effect(), triggers: authored::card_2080::RULES.triggers, activated: authored::card_2080::RULES.activated, enters: authored::card_2080::RULES.enters, entry: authored::card_2080::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 3 }, rules: &authored::card_2080::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Hulk, Gamma Goliath", oracle_id: Some("3508fd7d-ca02-47f9-8b0a-525f7bde4f57"), mana_cost: "{3}{R}{G}", type_line: "Legendary Creature — Gamma Berserker Hero", oracle_keywords: &["Power-up", "Reach", "Trample"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2092::RULES.keywords, deck_limit: authored::card_2092::RULES.deck_limit }, effect: authored::card_2092::RULES.checked_effect(), triggers: authored::card_2092::RULES.triggers, activated: authored::card_2092::RULES.activated, enters: authored::card_2092::RULES.enters, entry: authored::card_2092::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 1, 0], generic: 3 }, rules: &authored::card_2092::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Hulkling, Burgeoning Bruiser", oracle_id: Some("03c934a6-be57-480d-8c84-4e46d9ccb406"), mana_cost: "{2}{G}", type_line: "Legendary Creature — Kree Skrull Hero", oracle_keywords: &["Vigilance"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2094::RULES.keywords, deck_limit: authored::card_2094::RULES.deck_limit }, effect: authored::card_2094::RULES.checked_effect(), triggers: authored::card_2094::RULES.triggers, activated: authored::card_2094::RULES.activated, enters: authored::card_2094::RULES.enters, entry: authored::card_2094::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 2 }, rules: &authored::card_2094::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Human Torch, Johnny Storm", oracle_id: Some("5b6f2770-e774-4a77-801f-a3fda093c3dd"), mana_cost: "{2}{R}", type_line: "Legendary Creature — Human Hero", oracle_keywords: &["Flying", "Power-up"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2097::RULES.keywords, deck_limit: authored::card_2097::RULES.deck_limit }, effect: authored::card_2097::RULES.checked_effect(), triggers: authored::card_2097::RULES.triggers, activated: authored::card_2097::RULES.activated, enters: authored::card_2097::RULES.enters, entry: authored::card_2097::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 2 }, rules: &authored::card_2097::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Hydraulic Helper", oracle_id: Some("503928cd-1abd-44cd-92c3-6dc19bb07f78"), mana_cost: "{1}{U}", type_line: "Artifact Creature — Robot", oracle_keywords: &["Defender"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2111::RULES.keywords, deck_limit: authored::card_2111::RULES.deck_limit }, effect: authored::card_2111::RULES.checked_effect(), triggers: authored::card_2111::RULES.triggers, activated: authored::card_2111::RULES.activated, enters: authored::card_2111::RULES.enters, entry: authored::card_2111::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_2111::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "I Am Iron Man", oracle_id: Some("deb16a9a-2fa9-4ad3-bb20-7c16066e3d14"), mana_cost: "{2}{U}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2118::RULES.keywords, deck_limit: authored::card_2118::RULES.deck_limit }, effect: authored::card_2118::RULES.checked_effect(), triggers: authored::card_2118::RULES.triggers, activated: authored::card_2118::RULES.activated, enters: authored::card_2118::RULES.enters, entry: authored::card_2118::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_2118::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Invisible Woman, Sue Storm", oracle_id: Some("e9d20b6d-fa4f-473b-9866-230fd92a1319"), mana_cost: "{4}{W}", type_line: "Legendary Creature — Human Hero", oracle_keywords: &["Lifelink"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2216::RULES.keywords, deck_limit: authored::card_2216::RULES.deck_limit }, effect: authored::card_2216::RULES.checked_effect(), triggers: authored::card_2216::RULES.triggers, activated: authored::card_2216::RULES.activated, enters: authored::card_2216::RULES.enters, entry: authored::card_2216::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 4 }, rules: &authored::card_2216::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Iron Fist, Living Weapon", oracle_id: Some("bbf873c2-94c1-46b8-912e-e29e48e0212c"), mana_cost: "{2}{R}", type_line: "Legendary Creature — Human Warrior Hero", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2224::RULES.keywords, deck_limit: authored::card_2224::RULES.deck_limit }, effect: authored::card_2224::RULES.checked_effect(), triggers: authored::card_2224::RULES.triggers, activated: authored::card_2224::RULES.activated, enters: authored::card_2224::RULES.enters, entry: authored::card_2224::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 2 }, rules: &authored::card_2224::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Iron Lad, Diverging Destiny", oracle_id: Some("3e715032-0d21-441b-b2d4-c91084728718"), mana_cost: "{2}{U}", type_line: "Legendary Artifact Creature — Human Hero", oracle_keywords: &["Flying", "Vigilance"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2229::RULES.keywords, deck_limit: authored::card_2229::RULES.deck_limit }, effect: authored::card_2229::RULES.checked_effect(), triggers: authored::card_2229::RULES.triggers, activated: authored::card_2229::RULES.activated, enters: authored::card_2229::RULES.enters, entry: authored::card_2229::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_2229::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Iron Man Armor", oracle_id: Some("958d81c2-ca09-4275-890e-3bb595317325"), mana_cost: "{3}", type_line: "Artifact — Equipment", oracle_keywords: &["Equip"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2230::RULES.keywords, deck_limit: authored::card_2230::RULES.deck_limit }, effect: authored::card_2230::RULES.checked_effect(), triggers: authored::card_2230::RULES.triggers, activated: authored::card_2230::RULES.activated, enters: authored::card_2230::RULES.enters, entry: authored::card_2230::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_2230::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Iron Man, Master of Machines", oracle_id: Some("94bc796d-8c04-4835-8a99-7e5dce9540ee"), mana_cost: "{2}{U}{R}", type_line: "Legendary Artifact Creature — Human Hero", oracle_keywords: &["Flying", "Vigilance"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2231::RULES.keywords, deck_limit: authored::card_2231::RULES.deck_limit }, effect: authored::card_2231::RULES.checked_effect(), triggers: authored::card_2231::RULES.triggers, activated: authored::card_2231::RULES.activated, enters: authored::card_2231::RULES.enters, entry: authored::card_2231::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 1, 0, 1, 0, 0], generic: 2 }, rules: &authored::card_2231::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Ironheart, Clever Champion", oracle_id: Some("df3d5b86-7b75-4df9-8985-7159cde12261"), mana_cost: "{4}{U}", type_line: "Legendary Artifact Creature — Human Hero", oracle_keywords: &["Flying", "Improvise"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2235::RULES.keywords, deck_limit: authored::card_2235::RULES.deck_limit }, effect: authored::card_2235::RULES.checked_effect(), triggers: authored::card_2235::RULES.triggers, activated: authored::card_2235::RULES.activated, enters: authored::card_2235::RULES.enters, entry: authored::card_2235::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 4 }, rules: &authored::card_2235::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Jennifer Walters", oracle_id: Some("c867a778-97ef-4872-b890-b088b96a1281"), mana_cost: "{1}{W}", type_line: "Legendary Creature — Human Advisor Hero", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2263::RULES.keywords, deck_limit: authored::card_2263::RULES.deck_limit }, effect: authored::card_2263::RULES.checked_effect(), triggers: authored::card_2263::RULES.triggers, activated: authored::card_2263::RULES.activated, enters: authored::card_2263::RULES.enters, entry: authored::card_2263::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_2263::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Jessica Jones, Private Eye", oracle_id: Some("6677cd5a-0785-499b-ba99-f6bdcfb196d5"), mana_cost: "{2}{R}", type_line: "Legendary Creature — Human Detective Hero", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2274::RULES.keywords, deck_limit: authored::card_2274::RULES.deck_limit }, effect: authored::card_2274::RULES.checked_effect(), triggers: authored::card_2274::RULES.triggers, activated: authored::card_2274::RULES.activated, enters: authored::card_2274::RULES.enters, entry: authored::card_2274::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 2 }, rules: &authored::card_2274::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Justice, Vance Astrovik", oracle_id: Some("184dba74-63ed-4820-9174-68adf26bfabf"), mana_cost: "{2}{U}", type_line: "Legendary Creature — Mutant Hero", oracle_keywords: &["Flying"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2301::RULES.keywords, deck_limit: authored::card_2301::RULES.deck_limit }, effect: authored::card_2301::RULES.checked_effect(), triggers: authored::card_2301::RULES.triggers, activated: authored::card_2301::RULES.activated, enters: authored::card_2301::RULES.enters, entry: authored::card_2301::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_2301::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "K'un-Lun Warrior", oracle_id: Some("0618a217-2a6e-4fa1-bfe6-89c6612a594d"), mana_cost: "{1}{R}", type_line: "Creature — Human Warrior Hero", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2302::RULES.keywords, deck_limit: authored::card_2302::RULES.deck_limit }, effect: authored::card_2302::RULES.checked_effect(), triggers: authored::card_2302::RULES.triggers, activated: authored::card_2302::RULES.activated, enters: authored::card_2302::RULES.enters, entry: authored::card_2302::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 1 }, rules: &authored::card_2302::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Ka-Zar of the Savage Land", oracle_id: Some("88eeb519-5782-4732-bd34-81d1ee6dbc8f"), mana_cost: "{4}{G}", type_line: "Legendary Creature — Human Barbarian Hero", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2303::RULES.keywords, deck_limit: authored::card_2303::RULES.deck_limit }, effect: authored::card_2303::RULES.checked_effect(), triggers: authored::card_2303::RULES.triggers, activated: authored::card_2303::RULES.activated, enters: authored::card_2303::RULES.enters, entry: authored::card_2303::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 4 }, rules: &authored::card_2303::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Kang the Conqueror", oracle_id: Some("da6e24a4-8adc-41a1-b5eb-3e31dc1250c9"), mana_cost: "{2}{U}{U}", type_line: "Legendary Creature — Human Villain", oracle_keywords: &["Flying", "Power-up"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2311::RULES.keywords, deck_limit: authored::card_2311::RULES.deck_limit }, effect: authored::card_2311::RULES.checked_effect(), triggers: authored::card_2311::RULES.triggers, activated: authored::card_2311::RULES.activated, enters: authored::card_2311::RULES.enters, entry: authored::card_2311::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 2, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_2311::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Kang, Temporal Tyrant", oracle_id: Some("b43350a8-fc4d-41ed-becd-f0277512bb5f"), mana_cost: "{2}{U}{B}", type_line: "Legendary Creature — Human Villain", oracle_keywords: &["Connive"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2312::RULES.keywords, deck_limit: authored::card_2312::RULES.deck_limit }, effect: authored::card_2312::RULES.checked_effect(), triggers: authored::card_2312::RULES.triggers, activated: authored::card_2312::RULES.activated, enters: authored::card_2312::RULES.enters, entry: authored::card_2312::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 1, 0, 0, 0], generic: 2 }, rules: &authored::card_2312::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Kid Loki", oracle_id: Some("19ef0329-08de-4835-8079-2e791e19f3a6"), mana_cost: "{U}", type_line: "Legendary Creature — God Hero Villain", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2349::RULES.keywords, deck_limit: authored::card_2349::RULES.deck_limit }, effect: authored::card_2349::RULES.checked_effect(), triggers: authored::card_2349::RULES.triggers, activated: authored::card_2349::RULES.activated, enters: authored::card_2349::RULES.enters, entry: authored::card_2349::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_2349::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Killmonger, Scourge of Wakanda", oracle_id: Some("1c3ba54a-a8ef-4838-b3aa-cb6df827a619"), mana_cost: "{2}{B}{G}", type_line: "Legendary Creature — Human Mercenary Villain", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2352::RULES.keywords, deck_limit: authored::card_2352::RULES.deck_limit }, effect: authored::card_2352::RULES.checked_effect(), triggers: authored::card_2352::RULES.triggers, activated: authored::card_2352::RULES.activated, enters: authored::card_2352::RULES.enters, entry: authored::card_2352::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 1, 0], generic: 2 }, rules: &authored::card_2352::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "King T'Challa", oracle_id: Some("fe00278d-4385-4e3f-919c-12a794fd86f5"), mana_cost: "{1}{W}{U}", type_line: "Legendary Creature — Human Noble Hero", oracle_keywords: &["Flash"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2361::RULES.keywords, deck_limit: authored::card_2361::RULES.deck_limit }, effect: authored::card_2361::RULES.checked_effect(), triggers: authored::card_2361::RULES.triggers, activated: authored::card_2361::RULES.activated, enters: authored::card_2361::RULES.enters, entry: authored::card_2361::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 1, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_2361::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Kingpin's Enforcers", oracle_id: Some("342df67f-44d8-4b0a-b1d7-4241966d0598"), mana_cost: "{2}{B}", type_line: "Creature — Human Villain", oracle_keywords: &["Lifelink"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2362::RULES.keywords, deck_limit: authored::card_2362::RULES.deck_limit }, effect: authored::card_2362::RULES.checked_effect(), triggers: authored::card_2362::RULES.triggers, activated: authored::card_2362::RULES.activated, enters: authored::card_2362::RULES.enters, entry: authored::card_2362::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 2 }, rules: &authored::card_2362::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Klaw, Sonic Subjugator", oracle_id: Some("1c233e08-d5a3-48d7-9074-20393896a480"), mana_cost: "{2}{B}", type_line: "Legendary Creature — Human Rogue Villain", oracle_keywords: &["Sonic Attack"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2381::RULES.keywords, deck_limit: authored::card_2381::RULES.deck_limit }, effect: authored::card_2381::RULES.checked_effect(), triggers: authored::card_2381::RULES.triggers, activated: authored::card_2381::RULES.activated, enters: authored::card_2381::RULES.enters, entry: authored::card_2381::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 2 }, rules: &authored::card_2381::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Knight of Wundagore", oracle_id: Some("39145960-a212-46b4-addb-4b187f6bf6fd"), mana_cost: "{1}{G}", type_line: "Creature — Cat Knight Villain", oracle_keywords: &["Trample"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2387::RULES.keywords, deck_limit: authored::card_2387::RULES.deck_limit }, effect: authored::card_2387::RULES.checked_effect(), triggers: authored::card_2387::RULES.triggers, activated: authored::card_2387::RULES.activated, enters: authored::card_2387::RULES.enters, entry: authored::card_2387::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 1 }, rules: &authored::card_2387::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Kree Commandos", oracle_id: Some("09eb988a-e2eb-42e1-a786-01be63d2263f"), mana_cost: "{2}{W}", type_line: "Creature — Kree Soldier Villain", oracle_keywords: &["Flying", "Prowess", "Vigilance"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2412::RULES.keywords, deck_limit: authored::card_2412::RULES.deck_limit }, effect: authored::card_2412::RULES.checked_effect(), triggers: authored::card_2412::RULES.triggers, activated: authored::card_2412::RULES.activated, enters: authored::card_2412::RULES.enters, entry: authored::card_2412::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_2412::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Kree Sentinel", oracle_id: Some("3c2b9bb2-8201-47e8-84d5-f3518cac03cb"), mana_cost: "{4}{R}", type_line: "Artifact Creature — Kree Robot Villain", oracle_keywords: &["Reach", "Landcycling", "Basic landcycling", "Typecycling", "Cycling"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2413::RULES.keywords, deck_limit: authored::card_2413::RULES.deck_limit }, effect: authored::card_2413::RULES.checked_effect(), triggers: authored::card_2413::RULES.triggers, activated: authored::card_2413::RULES.activated, enters: authored::card_2413::RULES.enters, entry: authored::card_2413::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 4 }, rules: &authored::card_2413::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Leader, Super-Genius", oracle_id: Some("bec686dd-a9ea-4db5-bc20-8e40420b9a9b"), mana_cost: "{2}{U}{U}", type_line: "Legendary Creature — Gamma Scientist Villain", oracle_keywords: &["Connive"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2463::RULES.keywords, deck_limit: authored::card_2463::RULES.deck_limit }, effect: authored::card_2463::RULES.checked_effect(), triggers: authored::card_2463::RULES.triggers, activated: authored::card_2463::RULES.activated, enters: authored::card_2463::RULES.enters, entry: authored::card_2463::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 2, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_2463::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Lightning Strike", oracle_id: Some("f34b9bc4-7bfe-47fd-ba23-4eeeb46026eb"), mana_cost: "{1}{R}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2497::RULES.keywords, deck_limit: authored::card_2497::RULES.deck_limit }, effect: authored::card_2497::RULES.checked_effect(), triggers: authored::card_2497::RULES.triggers, activated: authored::card_2497::RULES.activated, enters: authored::card_2497::RULES.enters, entry: authored::card_2497::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 1 }, rules: &authored::card_2497::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Loki Laufeyson", oracle_id: Some("9c78778a-6335-4949-8ade-11d0f085cb2b"), mana_cost: "{1}{R}", type_line: "Legendary Creature — God Sorcerer Villain", oracle_keywords: &["Power-up"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2538::RULES.keywords, deck_limit: authored::card_2538::RULES.deck_limit }, effect: authored::card_2538::RULES.checked_effect(), triggers: authored::card_2538::RULES.triggers, activated: authored::card_2538::RULES.activated, enters: authored::card_2538::RULES.enters, entry: authored::card_2538::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 1 }, rules: &authored::card_2538::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Loki, God of Mischief", oracle_id: Some("c112852f-61d5-4b94-b864-c7d0b5333b29"), mana_cost: "{1}{U}", type_line: "Legendary Creature — God Sorcerer Villain", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2539::RULES.keywords, deck_limit: authored::card_2539::RULES.deck_limit }, effect: authored::card_2539::RULES.checked_effect(), triggers: authored::card_2539::RULES.triggers, activated: authored::card_2539::RULES.activated, enters: authored::card_2539::RULES.enters, entry: authored::card_2539::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_2539::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Los Diablos Missile Base", oracle_id: Some("4b17048f-8c7f-4e29-8f56-16db4c2106e6"), mana_cost: "", type_line: "Land", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2561::RULES.keywords, deck_limit: authored::card_2561::RULES.deck_limit }, effect: authored::card_2561::RULES.checked_effect(), triggers: authored::card_2561::RULES.triggers, activated: authored::card_2561::RULES.activated, enters: authored::card_2561::RULES.enters, entry: authored::card_2561::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_2561::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Luke Cage, Power Man", oracle_id: Some("c95d1558-09a4-4dbc-b08a-fe21b47bc5d2"), mana_cost: "{3}{W}", type_line: "Legendary Creature — Human Hero", oracle_keywords: &["Unbreakable Skin"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2574::RULES.keywords, deck_limit: authored::card_2574::RULES.deck_limit }, effect: authored::card_2574::RULES.checked_effect(), triggers: authored::card_2574::RULES.triggers, activated: authored::card_2574::RULES.activated, enters: authored::card_2574::RULES.enters, entry: authored::card_2574::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_2574::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "M.O.D.O.K.", oracle_id: Some("fe7cc414-42d0-4dad-8516-7a0be0860a67"), mana_cost: "{3}{B}{B}", type_line: "Legendary Artifact Creature — Villain", oracle_keywords: &["Mental Organism", "Flying", "Lifelink", "Designed Only for Killing", "Connive"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2596::RULES.keywords, deck_limit: authored::card_2596::RULES.deck_limit }, effect: authored::card_2596::RULES.checked_effect(), triggers: authored::card_2596::RULES.triggers, activated: authored::card_2596::RULES.activated, enters: authored::card_2596::RULES.enters, entry: authored::card_2596::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 0, 2, 0, 0, 0], generic: 3 }, rules: &authored::card_2596::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Machinesmith Automaton", oracle_id: Some("945e8443-9647-4eab-b091-ee8e7bc347f5"), mana_cost: "{2}{R}", type_line: "Artifact Creature — Robot Villain", oracle_keywords: &["Trample"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2604::RULES.keywords, deck_limit: authored::card_2604::RULES.deck_limit }, effect: authored::card_2604::RULES.checked_effect(), triggers: authored::card_2604::RULES.triggers, activated: authored::card_2604::RULES.activated, enters: authored::card_2604::RULES.enters, entry: authored::card_2604::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 2 }, rules: &authored::card_2604::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Madame Hydra", oracle_id: Some("d812c8e3-a475-4ad2-a93f-7208ad873676"), mana_cost: "{2}{B}{R}", type_line: "Legendary Creature — Human Villain", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2606::RULES.keywords, deck_limit: authored::card_2606::RULES.deck_limit }, effect: authored::card_2606::RULES.checked_effect(), triggers: authored::card_2606::RULES.triggers, activated: authored::card_2606::RULES.activated, enters: authored::card_2606::RULES.enters, entry: authored::card_2606::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 1, 0, 0], generic: 2 }, rules: &authored::card_2606::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Madame Masque", oracle_id: Some("b5b0e85f-6651-4667-829c-2253ff8b130d"), mana_cost: "{4}{B}", type_line: "Legendary Creature — Human Villain", oracle_keywords: &["Connive"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2607::RULES.keywords, deck_limit: authored::card_2607::RULES.deck_limit }, effect: authored::card_2607::RULES.checked_effect(), triggers: authored::card_2607::RULES.triggers, activated: authored::card_2607::RULES.activated, enters: authored::card_2607::RULES.enters, entry: authored::card_2607::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 4 }, rules: &authored::card_2607::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Mister Fantastic, Reed Richards", oracle_id: Some("7fa5237e-dd79-4646-b935-cb8c6ee803ab"), mana_cost: "{3}{U}", type_line: "Legendary Creature — Human Scientist Hero", oracle_keywords: &["Reach"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2792::RULES.keywords, deck_limit: authored::card_2792::RULES.deck_limit }, effect: authored::card_2792::RULES.checked_effect(), triggers: authored::card_2792::RULES.triggers, activated: authored::card_2792::RULES.activated, enters: authored::card_2792::RULES.enters, entry: authored::card_2792::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_2792::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Mister Hyde, Monster Within", oracle_id: Some("4a7c7ecc-df72-4da0-af35-dd0af58ec02c"), mana_cost: "{2}{G}", type_line: "Legendary Creature — Human Villain", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2793::RULES.keywords, deck_limit: authored::card_2793::RULES.deck_limit }, effect: authored::card_2793::RULES.checked_effect(), triggers: authored::card_2793::RULES.triggers, activated: authored::card_2793::RULES.activated, enters: authored::card_2793::RULES.enters, entry: authored::card_2793::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 2 }, rules: &authored::card_2793::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Misty Knight, Hero for Hire", oracle_id: Some("5b090ba3-0ecb-4585-b5b7-9d6028c32487"), mana_cost: "{1}{R}", type_line: "Legendary Creature — Human Detective Hero", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2798::RULES.keywords, deck_limit: authored::card_2798::RULES.deck_limit }, effect: authored::card_2798::RULES.checked_effect(), triggers: authored::card_2798::RULES.triggers, activated: authored::card_2798::RULES.activated, enters: authored::card_2798::RULES.enters, entry: authored::card_2798::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 1 }, rules: &authored::card_2798::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Mjölnir, Hammer of Thor", oracle_id: Some("7f9a8845-d760-44a7-a4c9-8a20dba4e14a"), mana_cost: "{3}{R}", type_line: "Legendary Artifact — Equipment", oracle_keywords: &["Double"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2801::RULES.keywords, deck_limit: authored::card_2801::RULES.deck_limit }, effect: authored::card_2801::RULES.checked_effect(), triggers: authored::card_2801::RULES.triggers, activated: authored::card_2801::RULES.activated, enters: authored::card_2801::RULES.enters, entry: authored::card_2801::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 3 }, rules: &authored::card_2801::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Mockingbird, Ace Agent", oracle_id: Some("1fd00656-83ba-4feb-b089-224007936b99"), mana_cost: "{3}{W}", type_line: "Legendary Creature — Human Spy Hero", oracle_keywords: &["Double strike"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2808::RULES.keywords, deck_limit: authored::card_2808::RULES.deck_limit }, effect: authored::card_2808::RULES.checked_effect(), triggers: authored::card_2808::RULES.triggers, activated: authored::card_2808::RULES.activated, enters: authored::card_2808::RULES.enters, entry: authored::card_2808::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_2808::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Mole Man, Moloid Master", oracle_id: Some("34994475-b914-44f6-8ca1-c2747b6d5956"), mana_cost: "{2}{G}", type_line: "Legendary Creature — Human Villain", oracle_keywords: &["Mill", "Landfall"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2811::RULES.keywords, deck_limit: authored::card_2811::RULES.deck_limit }, effect: authored::card_2811::RULES.checked_effect(), triggers: authored::card_2811::RULES.triggers, activated: authored::card_2811::RULES.activated, enters: authored::card_2811::RULES.enters, entry: authored::card_2811::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 2 }, rules: &authored::card_2811::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Monica Rambeau", oracle_id: Some("8dd929cb-0fe1-4cb3-bc8d-96ca37f194f3"), mana_cost: "{2}{W}", type_line: "Legendary Creature — Human Hero", oracle_keywords: &["Flying", "Prowess"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2832::RULES.keywords, deck_limit: authored::card_2832::RULES.deck_limit }, effect: authored::card_2832::RULES.checked_effect(), triggers: authored::card_2832::RULES.triggers, activated: authored::card_2832::RULES.activated, enters: authored::card_2832::RULES.enters, entry: authored::card_2832::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_2832::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Moon Girl and Devil Dinosaur", oracle_id: Some("bfe0818b-ae13-4174-85c8-24b895d28df0"), mana_cost: "{1}{G}{U}", type_line: "Legendary Creature — Human Dinosaur Hero", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2840::RULES.keywords, deck_limit: authored::card_2840::RULES.deck_limit }, effect: authored::card_2840::RULES.checked_effect(), triggers: authored::card_2840::RULES.triggers, activated: authored::card_2840::RULES.activated, enters: authored::card_2840::RULES.enters, entry: authored::card_2840::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 1, 0], generic: 1 }, rules: &authored::card_2840::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Moonstone, Harsh Mistress", oracle_id: Some("aeeb50f2-8e02-4d40-a1dd-28b1c5c85c7e"), mana_cost: "{3}{B}", type_line: "Legendary Creature — Human Doctor Villain", oracle_keywords: &["Flying"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2849::RULES.keywords, deck_limit: authored::card_2849::RULES.deck_limit }, effect: authored::card_2849::RULES.checked_effect(), triggers: authored::card_2849::RULES.triggers, activated: authored::card_2849::RULES.activated, enters: authored::card_2849::RULES.enters, entry: authored::card_2849::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 3 }, rules: &authored::card_2849::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Ms. Marvel, Kamala Khan", oracle_id: Some("ebedff30-ae43-4861-aab2-afe1fc88337e"), mana_cost: "{2}{U}", type_line: "Legendary Creature — Mutant Inhuman Hero", oracle_keywords: &["Reach", "Vigilance", "Embiggen Fist"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2869::RULES.keywords, deck_limit: authored::card_2869::RULES.deck_limit }, effect: authored::card_2869::RULES.checked_effect(), triggers: authored::card_2869::RULES.triggers, activated: authored::card_2869::RULES.activated, enters: authored::card_2869::RULES.enters, entry: authored::card_2869::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_2869::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Multiversal Incursion", oracle_id: Some("d3678fce-6897-4f2c-bf50-243facb6581b"), mana_cost: "{5}{U}{U}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2876::RULES.keywords, deck_limit: authored::card_2876::RULES.deck_limit }, effect: authored::card_2876::RULES.checked_effect(), triggers: authored::card_2876::RULES.triggers, activated: authored::card_2876::RULES.activated, enters: authored::card_2876::RULES.enters, entry: authored::card_2876::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 2, 0, 0, 0, 0], generic: 5 }, rules: &authored::card_2876::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Murdock's Crusade", oracle_id: Some("36330947-5ef9-4bff-b541-bfbddd9715a3"), mana_cost: "{1}{W}", type_line: "Sorcery", oracle_keywords: &["Street Justice", "Legal Justice", "Teamwork"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2880::RULES.keywords, deck_limit: authored::card_2880::RULES.deck_limit }, effect: authored::card_2880::RULES.checked_effect(), triggers: authored::card_2880::RULES.triggers, activated: authored::card_2880::RULES.activated, enters: authored::card_2880::RULES.enters, entry: authored::card_2880::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_2880::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Namor the Sub-Mariner", oracle_id: Some("dff11626-ad4d-443a-b7db-f508790b5fd2"), mana_cost: "{1}{U}{U}", type_line: "Legendary Creature — Mutant Merfolk Villain", oracle_keywords: &["Flying"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2905::RULES.keywords, deck_limit: authored::card_2905::RULES.deck_limit }, effect: authored::card_2905::RULES.checked_effect(), triggers: authored::card_2905::RULES.triggers, activated: authored::card_2905::RULES.activated, enters: authored::card_2905::RULES.enters, entry: authored::card_2905::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 2, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_2905::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Nick Fury, Agent of S.H.I.E.L.D.", oracle_id: Some("62916107-14c0-4516-9fed-a032b1a80e5d"), mana_cost: "{W}", type_line: "Legendary Creature — Human Spy Hero", oracle_keywords: &["Power-up", "Transform"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2932::RULES.keywords, deck_limit: authored::card_2932::RULES.deck_limit }, effect: authored::card_2932::RULES.checked_effect(), triggers: authored::card_2932::RULES.triggers, activated: authored::card_2932::RULES.activated, enters: authored::card_2932::RULES.enters, entry: authored::card_2932::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_2932::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Night Nurse, Healer of Heroes", oracle_id: Some("e3559565-0512-4525-addb-bf27903bcd13"), mana_cost: "{1}{W}", type_line: "Legendary Creature — Human Doctor Hero", oracle_keywords: &["Lifelink", "Flash"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2934::RULES.keywords, deck_limit: authored::card_2934::RULES.deck_limit }, effect: authored::card_2934::RULES.checked_effect(), triggers: authored::card_2934::RULES.triggers, activated: authored::card_2934::RULES.activated, enters: authored::card_2934::RULES.enters, entry: authored::card_2934::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_2934::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Ninja of the Hand", oracle_id: Some("c38f7447-e646-438d-81bd-fe2a71c8a79c"), mana_cost: "{2}{B}", type_line: "Creature — Human Ninja Villain", oracle_keywords: &["Power-up", "Deathtouch"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2946::RULES.keywords, deck_limit: authored::card_2946::RULES.deck_limit }, effect: authored::card_2946::RULES.checked_effect(), triggers: authored::card_2946::RULES.triggers, activated: authored::card_2946::RULES.activated, enters: authored::card_2946::RULES.enters, entry: authored::card_2946::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 2 }, rules: &authored::card_2946::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Okoye, Dora Milaje Leader", oracle_id: Some("6d397d26-7e52-47b2-9880-2c10e90cfba4"), mana_cost: "{3}{W}", type_line: "Legendary Creature — Human Warrior Hero", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_3002::RULES.keywords, deck_limit: authored::card_3002::RULES.deck_limit }, effect: authored::card_3002::RULES.checked_effect(), triggers: authored::card_3002::RULES.triggers, activated: authored::card_3002::RULES.activated, enters: authored::card_3002::RULES.enters, entry: authored::card_3002::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_3002::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Origin of the Avengers", oracle_id: Some("d7f3178d-7b7c-4346-af50-a7b0206bee22"), mana_cost: "{1}{W}", type_line: "Enchantment — Saga", oracle_keywords: &["Scry"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3040::RULES.keywords, deck_limit: authored::card_3040::RULES.deck_limit }, effect: authored::card_3040::RULES.checked_effect(), triggers: authored::card_3040::RULES.triggers, activated: authored::card_3040::RULES.activated, enters: authored::card_3040::RULES.enters, entry: authored::card_3040::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_3040::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Panther Pounce", oracle_id: Some("ead0820e-fc6b-4ab2-9110-048d6d53985d"), mana_cost: "{W}", type_line: "Instant", oracle_keywords: &["Investigate"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3096::RULES.keywords, deck_limit: authored::card_3096::RULES.deck_limit }, effect: authored::card_3096::RULES.checked_effect(), triggers: authored::card_3096::RULES.triggers, activated: authored::card_3096::RULES.activated, enters: authored::card_3096::RULES.enters, entry: authored::card_3096::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_3096::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Patriot, Shield Wielder", oracle_id: Some("1df7a09f-fbc8-4c2a-a903-bab9e680eda7"), mana_cost: "{1}{W}", type_line: "Legendary Creature — Human Hero", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_3114::RULES.keywords, deck_limit: authored::card_3114::RULES.deck_limit }, effect: authored::card_3114::RULES.checked_effect(), triggers: authored::card_3114::RULES.triggers, activated: authored::card_3114::RULES.activated, enters: authored::card_3114::RULES.enters, entry: authored::card_3114::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_3114::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Pet Avengers", oracle_id: Some("bbee7921-19c9-4c70-8462-f16e7e9c25db"), mana_cost: "{3}{G}", type_line: "Creature — Dragon Cat Dog Bird Frog Hero", oracle_keywords: &["Power-up", "Reach"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3141::RULES.keywords, deck_limit: authored::card_3141::RULES.deck_limit }, effect: authored::card_3141::RULES.checked_effect(), triggers: authored::card_3141::RULES.triggers, activated: authored::card_3141::RULES.activated, enters: authored::card_3141::RULES.enters, entry: authored::card_3141::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 3 }, rules: &authored::card_3141::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Photon Blast Barrage", oracle_id: Some("a5424e33-ff4e-4c82-8aa9-9d25ced7830a"), mana_cost: "{X}{R}{R}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3150::RULES.keywords, deck_limit: authored::card_3150::RULES.deck_limit }, effect: authored::card_3150::RULES.checked_effect(), triggers: authored::card_3150::RULES.triggers, activated: authored::card_3150::RULES.activated, enters: authored::card_3150::RULES.enters, entry: authored::card_3150::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 0, 2, 0, 0], generic: 0 }, rules: &authored::card_3150::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Political Triumph", oracle_id: Some("0bbceb51-1fb8-44d7-8afc-b4746d292262"), mana_cost: "{W}", type_line: "Enchantment — Plan", oracle_keywords: &["Scry"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3197::RULES.keywords, deck_limit: authored::card_3197::RULES.deck_limit }, effect: authored::card_3197::RULES.checked_effect(), triggers: authored::card_3197::RULES.triggers, activated: authored::card_3197::RULES.activated, enters: authored::card_3197::RULES.enters, entry: authored::card_3197::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_3197::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Powerful Broker", oracle_id: Some("2b3d76e7-faaa-4e6f-9e21-a88dd719247e"), mana_cost: "{2}{G}", type_line: "Creature — Human Villain", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3212::RULES.keywords, deck_limit: authored::card_3212::RULES.deck_limit }, effect: authored::card_3212::RULES.checked_effect(), triggers: authored::card_3212::RULES.triggers, activated: authored::card_3212::RULES.activated, enters: authored::card_3212::RULES.enters, entry: authored::card_3212::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 2 }, rules: &authored::card_3212::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Project Deathlok Soldier", oracle_id: Some("d89aa87a-d59a-41e2-b92f-15c9bc6d3d13"), mana_cost: "{B}", type_line: "Artifact Creature — Zombie Soldier", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3255::RULES.keywords, deck_limit: authored::card_3255::RULES.deck_limit }, effect: authored::card_3255::RULES.checked_effect(), triggers: authored::card_3255::RULES.triggers, activated: authored::card_3255::RULES.activated, enters: authored::card_3255::RULES.enters, entry: authored::card_3255::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 0 }, rules: &authored::card_3255::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Punishing Punch", oracle_id: Some("454c4e8b-e417-4381-a931-6a11356ef64e"), mana_cost: "{2}{G}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3281::RULES.keywords, deck_limit: authored::card_3281::RULES.deck_limit }, effect: authored::card_3281::RULES.checked_effect(), triggers: authored::card_3281::RULES.triggers, activated: authored::card_3281::RULES.activated, enters: authored::card_3281::RULES.enters, entry: authored::card_3281::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 2 }, rules: &authored::card_3281::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Pym Particles", oracle_id: Some("636b44dc-f7e4-42d2-abf6-ae49432b6052"), mana_cost: "{U}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3290::RULES.keywords, deck_limit: authored::card_3290::RULES.deck_limit }, effect: authored::card_3290::RULES.checked_effect(), triggers: authored::card_3290::RULES.triggers, activated: authored::card_3290::RULES.activated, enters: authored::card_3290::RULES.enters, entry: authored::card_3290::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_3290::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Pym Technologies", oracle_id: Some("5a8977fc-0732-4807-b876-01819aedd70b"), mana_cost: "", type_line: "Land", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3291::RULES.keywords, deck_limit: authored::card_3291::RULES.deck_limit }, effect: authored::card_3291::RULES.checked_effect(), triggers: authored::card_3291::RULES.triggers, activated: authored::card_3291::RULES.activated, enters: authored::card_3291::RULES.enters, entry: authored::card_3291::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_3291::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Quake, Agent of S.H.I.E.L.D.", oracle_id: Some("3966c94b-e597-4b3f-a4f5-001d52fff9fe"), mana_cost: "{2}{W}", type_line: "Legendary Creature — Inhuman Spy Hero", oracle_keywords: &["Seismic Takedown"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_3301::RULES.keywords, deck_limit: authored::card_3301::RULES.deck_limit }, effect: authored::card_3301::RULES.checked_effect(), triggers: authored::card_3301::RULES.triggers, activated: authored::card_3301::RULES.activated, enters: authored::card_3301::RULES.enters, entry: authored::card_3301::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_3301::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Quicksilver, Brash Blur", oracle_id: Some("b76ce131-e51a-445e-8216-00df5904ab93"), mana_cost: "{R}", type_line: "Legendary Creature — Mutant Hero", oracle_keywords: &["Power-up", "Haste"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_3315::RULES.keywords, deck_limit: authored::card_3315::RULES.deck_limit }, effect: authored::card_3315::RULES.checked_effect(), triggers: authored::card_3315::RULES.triggers, activated: authored::card_3315::RULES.activated, enters: authored::card_3315::RULES.enters, entry: authored::card_3315::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 0 }, rules: &authored::card_3315::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Raft Security Officer", oracle_id: Some("60754d4b-2add-4928-8d30-8ce9c0f54f91"), mana_cost: "{1}{W}", type_line: "Creature — Human Soldier", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3336::RULES.keywords, deck_limit: authored::card_3336::RULES.deck_limit }, effect: authored::card_3336::RULES.checked_effect(), triggers: authored::card_3336::RULES.triggers, activated: authored::card_3336::RULES.activated, enters: authored::card_3336::RULES.enters, entry: authored::card_3336::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_3336::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Rapid Rescue", oracle_id: Some("de1ad369-5aa3-4c54-8fb1-30b7f1d4d542"), mana_cost: "{G}", type_line: "Instant", oracle_keywords: &["Mill"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3382::RULES.keywords, deck_limit: authored::card_3382::RULES.deck_limit }, effect: authored::card_3382::RULES.checked_effect(), triggers: authored::card_3382::RULES.triggers, activated: authored::card_3382::RULES.activated, enters: authored::card_3382::RULES.enters, entry: authored::card_3382::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 0 }, rules: &authored::card_3382::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Red Guardian, Super-Soldier", oracle_id: Some("e85567e8-1102-4d44-a15b-d5f1fdef8465"), mana_cost: "{2}{W}", type_line: "Legendary Creature — Human Soldier Villain", oracle_keywords: &["Flash"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_3426::RULES.keywords, deck_limit: authored::card_3426::RULES.deck_limit }, effect: authored::card_3426::RULES.checked_effect(), triggers: authored::card_3426::RULES.triggers, activated: authored::card_3426::RULES.activated, enters: authored::card_3426::RULES.enters, entry: authored::card_3426::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_3426::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Red Hulk", oracle_id: Some("a14a838a-49f4-45e3-9ec0-3c803af45a68"), mana_cost: "{4}{R}{R}", type_line: "Legendary Creature — Gamma Berserker Villain", oracle_keywords: &["Reach", "Enrage", "Trample"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_3428::RULES.keywords, deck_limit: authored::card_3428::RULES.deck_limit }, effect: authored::card_3428::RULES.checked_effect(), triggers: authored::card_3428::RULES.triggers, activated: authored::card_3428::RULES.activated, enters: authored::card_3428::RULES.enters, entry: authored::card_3428::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 2, 0, 0], generic: 4 }, rules: &authored::card_3428::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Red Room Recruit", oracle_id: Some("4e60c38c-cbf5-4ff7-8fff-634f7e3086cc"), mana_cost: "{1}{B}", type_line: "Creature — Human Spy Villain", oracle_keywords: &["Connive"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3430::RULES.keywords, deck_limit: authored::card_3430::RULES.deck_limit }, effect: authored::card_3430::RULES.checked_effect(), triggers: authored::card_3430::RULES.triggers, activated: authored::card_3430::RULES.activated, enters: authored::card_3430::RULES.enters, entry: authored::card_3430::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 1 }, rules: &authored::card_3430::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Reptil, Dinomorpher", oracle_id: Some("c6284597-1983-4a21-a461-724135220e0d"), mana_cost: "{G}", type_line: "Legendary Creature — Human Hero", oracle_keywords: &["Brontosaurus", "Tyrannosaurus Rex"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_3463::RULES.keywords, deck_limit: authored::card_3463::RULES.deck_limit }, effect: authored::card_3463::RULES.checked_effect(), triggers: authored::card_3463::RULES.triggers, activated: authored::card_3463::RULES.activated, enters: authored::card_3463::RULES.enters, entry: authored::card_3463::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 0 }, rules: &authored::card_3463::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Repulsor Blast", oracle_id: Some("150f920c-c942-4feb-824f-79dd9b531687"), mana_cost: "{3}{R}", type_line: "Sorcery", oracle_keywords: &["Teamwork"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3466::RULES.keywords, deck_limit: authored::card_3466::RULES.deck_limit }, effect: authored::card_3466::RULES.checked_effect(), triggers: authored::card_3466::RULES.triggers, activated: authored::card_3466::RULES.activated, enters: authored::card_3466::RULES.enters, entry: authored::card_3466::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 3 }, rules: &authored::card_3466::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Restorative Technique", oracle_id: Some("f7b2606c-42b8-46d6-bcaa-8d4506b348eb"), mana_cost: "{2}{G}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3495::RULES.keywords, deck_limit: authored::card_3495::RULES.deck_limit }, effect: authored::card_3495::RULES.checked_effect(), triggers: authored::card_3495::RULES.triggers, activated: authored::card_3495::RULES.activated, enters: authored::card_3495::RULES.enters, entry: authored::card_3495::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 2 }, rules: &authored::card_3495::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Rewrite History", oracle_id: Some("f91c860e-3feb-4a5a-b7ef-4e7d38d61c75"), mana_cost: "{2}{U}", type_line: "Enchantment — Plan", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3511::RULES.keywords, deck_limit: authored::card_3511::RULES.deck_limit }, effect: authored::card_3511::RULES.checked_effect(), triggers: authored::card_3511::RULES.triggers, activated: authored::card_3511::RULES.activated, enters: authored::card_3511::RULES.enters, entry: authored::card_3511::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_3511::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Rick Jones, Destined Sidekick", oracle_id: Some("4bccfa67-0c6f-42a1-b8d4-0842a8230d1e"), mana_cost: "{G}", type_line: "Legendary Creature — Human Advisor", oracle_keywords: &["Mill"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_3517::RULES.keywords, deck_limit: authored::card_3517::RULES.deck_limit }, effect: authored::card_3517::RULES.checked_effect(), triggers: authored::card_3517::RULES.triggers, activated: authored::card_3517::RULES.activated, enters: authored::card_3517::RULES.enters, entry: authored::card_3517::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 0 }, rules: &authored::card_3517::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Robot Domination", oracle_id: Some("4f69d2f9-c16a-4752-9026-794a29d73af8"), mana_cost: "{3}{B}", type_line: "Enchantment — Plan", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3564::RULES.keywords, deck_limit: authored::card_3564::RULES.deck_limit }, effect: authored::card_3564::RULES.checked_effect(), triggers: authored::card_3564::RULES.triggers, activated: authored::card_3564::RULES.activated, enters: authored::card_3564::RULES.enters, entry: authored::card_3564::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 3 }, rules: &authored::card_3564::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Ronin, Shadow Stalker", oracle_id: Some("8ceb1acf-5ff3-4f1a-af04-9f07ad454526"), mana_cost: "{2}{B}", type_line: "Legendary Creature — Human Rogue Hero", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_3581::RULES.keywords, deck_limit: authored::card_3581::RULES.deck_limit }, effect: authored::card_3581::RULES.checked_effect(), triggers: authored::card_3581::RULES.triggers, activated: authored::card_3581::RULES.activated, enters: authored::card_3581::RULES.enters, entry: authored::card_3581::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 2 }, rules: &authored::card_3581::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Roxxon Brutes", oracle_id: Some("b92c2da9-eaa4-404b-8114-5527eec6390e"), mana_cost: "{4}{B}", type_line: "Creature — Human Berserker Villain", oracle_keywords: &["Landcycling", "Menace", "Basic landcycling", "Typecycling", "Cycling"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3604::RULES.keywords, deck_limit: authored::card_3604::RULES.deck_limit }, effect: authored::card_3604::RULES.checked_effect(), triggers: authored::card_3604::RULES.triggers, activated: authored::card_3604::RULES.activated, enters: authored::card_3604::RULES.enters, entry: authored::card_3604::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 4 }, rules: &authored::card_3604::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "S.H.I.E.L.D. Deployment Drone", oracle_id: Some("c1669020-8f81-471a-9083-7e9ce3e8f387"), mana_cost: "{2}{U}", type_line: "Artifact Creature — Robot", oracle_keywords: &["Flying"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3635::RULES.keywords, deck_limit: authored::card_3635::RULES.deck_limit }, effect: authored::card_3635::RULES.checked_effect(), triggers: authored::card_3635::RULES.triggers, activated: authored::card_3635::RULES.activated, enters: authored::card_3635::RULES.enters, entry: authored::card_3635::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_3635::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "S.H.I.E.L.D. Flying Car", oracle_id: Some("12d51534-a98d-487b-9ba9-c10861dff992"), mana_cost: "{2}{U}", type_line: "Artifact — Vehicle", oracle_keywords: &["Flying", "Crew", "Flash"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3636::RULES.keywords, deck_limit: authored::card_3636::RULES.deck_limit }, effect: authored::card_3636::RULES.checked_effect(), triggers: authored::card_3636::RULES.triggers, activated: authored::card_3636::RULES.activated, enters: authored::card_3636::RULES.enters, entry: authored::card_3636::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_3636::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "S.H.I.E.L.D. Helicarrier", oracle_id: Some("0f2335bf-6a13-4483-997a-fd19d30c4d48"), mana_cost: "{4}", type_line: "Artifact — Vehicle", oracle_keywords: &["Flying", "Crew"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3637::RULES.keywords, deck_limit: authored::card_3637::RULES.deck_limit }, effect: authored::card_3637::RULES.checked_effect(), triggers: authored::card_3637::RULES.triggers, activated: authored::card_3637::RULES.activated, enters: authored::card_3637::RULES.enters, entry: authored::card_3637::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 4 }, rules: &authored::card_3637::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "S.H.I.E.L.D. Spy Kit", oracle_id: Some("6fd44fb9-3433-4cb0-95a4-4ac8170b2166"), mana_cost: "{W}", type_line: "Artifact — Equipment", oracle_keywords: &["Equip", "Scry"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3638::RULES.keywords, deck_limit: authored::card_3638::RULES.deck_limit }, effect: authored::card_3638::RULES.checked_effect(), triggers: authored::card_3638::RULES.triggers, activated: authored::card_3638::RULES.activated, enters: authored::card_3638::RULES.enters, entry: authored::card_3638::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_3638::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Savage Land Dinosaur", oracle_id: Some("6e21ebdf-b97c-494e-a1d5-19db1b7d3016"), mana_cost: "{4}{G}{G}", type_line: "Creature — Dinosaur", oracle_keywords: &["Landcycling", "Trample", "Basic landcycling", "Typecycling", "Cycling"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3692::RULES.keywords, deck_limit: authored::card_3692::RULES.deck_limit }, effect: authored::card_3692::RULES.checked_effect(), triggers: authored::card_3692::RULES.triggers, activated: authored::card_3692::RULES.activated, enters: authored::card_3692::RULES.enters, entry: authored::card_3692::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 2, 0], generic: 4 }, rules: &authored::card_3692::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Scientist Supreme of A.I.M.", oracle_id: Some("c09c424b-6aba-4190-9f55-f0307d746386"), mana_cost: "{U}{B}", type_line: "Legendary Creature — Human Scientist Villain", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_3723::RULES.keywords, deck_limit: authored::card_3723::RULES.deck_limit }, effect: authored::card_3723::RULES.checked_effect(), triggers: authored::card_3723::RULES.triggers, activated: authored::card_3723::RULES.activated, enters: authored::card_3723::RULES.enters, entry: authored::card_3723::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 1, 0, 0, 0], generic: 0 }, rules: &authored::card_3723::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Secret Invasion", oracle_id: Some("25443ea4-3177-4e29-a2b1-f3fee60b3205"), mana_cost: "{1}{U}{U}", type_line: "Enchantment — Aura", oracle_keywords: &["Enchant"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3760::RULES.keywords, deck_limit: authored::card_3760::RULES.deck_limit }, effect: authored::card_3760::RULES.checked_effect(), triggers: authored::card_3760::RULES.triggers, activated: authored::card_3760::RULES.activated, enters: authored::card_3760::RULES.enters, entry: authored::card_3760::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 2, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_3760::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Serpent Specialist", oracle_id: Some("556f127f-6131-4171-b1ed-11420c21d345"), mana_cost: "{G}", type_line: "Creature — Human Snake Villain", oracle_keywords: &["Power-up", "Deathtouch"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3794::RULES.keywords, deck_limit: authored::card_3794::RULES.deck_limit }, effect: authored::card_3794::RULES.checked_effect(), triggers: authored::card_3794::RULES.triggers, activated: authored::card_3794::RULES.activated, enters: authored::card_3794::RULES.enters, entry: authored::card_3794::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 0 }, rules: &authored::card_3794::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Shang-Chi, Master of Kung Fu", oracle_id: Some("700ef9c8-698b-45f7-a1fd-dec74e553224"), mana_cost: "{1}{G}", type_line: "Legendary Creature — Human Warrior Hero", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_3810::RULES.keywords, deck_limit: authored::card_3810::RULES.deck_limit }, effect: authored::card_3810::RULES.checked_effect(), triggers: authored::card_3810::RULES.triggers, activated: authored::card_3810::RULES.activated, enters: authored::card_3810::RULES.enters, entry: authored::card_3810::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 1 }, rules: &authored::card_3810::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "She-Hulk, Jade Defender", oracle_id: Some("bbd0a2a4-f81b-4a6b-bc4d-d7e53dc51333"), mana_cost: "{3}{G}", type_line: "Legendary Creature — Gamma Hero", oracle_keywords: &["Power-up", "Reach", "Trample"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_3823::RULES.keywords, deck_limit: authored::card_3823::RULES.deck_limit }, effect: authored::card_3823::RULES.checked_effect(), triggers: authored::card_3823::RULES.triggers, activated: authored::card_3823::RULES.activated, enters: authored::card_3823::RULES.enters, entry: authored::card_3823::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 3 }, rules: &authored::card_3823::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Shuri, Wakandan Inventor", oracle_id: Some("a8c047fc-20a9-4c62-b8a6-9ead56d60ad0"), mana_cost: "{1}{U}", type_line: "Legendary Creature — Human Artificer Hero", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_3857::RULES.keywords, deck_limit: authored::card_3857::RULES.deck_limit }, effect: authored::card_3857::RULES.checked_effect(), triggers: authored::card_3857::RULES.triggers, activated: authored::card_3857::RULES.activated, enters: authored::card_3857::RULES.enters, entry: authored::card_3857::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_3857::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Speed, Young Avenger", oracle_id: Some("2fb0f2d2-ed1f-4181-91f2-684a71c34998"), mana_cost: "{1}{R}", type_line: "Legendary Creature — Mutant Hero", oracle_keywords: &["Haste"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_4003::RULES.keywords, deck_limit: authored::card_4003::RULES.deck_limit }, effect: authored::card_4003::RULES.checked_effect(), triggers: authored::card_4003::RULES.triggers, activated: authored::card_4003::RULES.activated, enters: authored::card_4003::RULES.enters, entry: authored::card_4003::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 1 }, rules: &authored::card_4003::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Speedball, New Warrior", oracle_id: Some("53d3ef8f-8e0f-4119-8cdc-a0bf076b4e91"), mana_cost: "{2}{U/R}", type_line: "Legendary Creature — Human Hero", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_4004::RULES.keywords, deck_limit: authored::card_4004::RULES.deck_limit }, effect: authored::card_4004::RULES.checked_effect(), triggers: authored::card_4004::RULES.triggers, activated: authored::card_4004::RULES.activated, enters: authored::card_4004::RULES.enters, entry: authored::card_4004::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 2 }, rules: &authored::card_4004::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Spider-Man, To the Rescue", oracle_id: Some("85dec6cf-6f27-4eb0-834b-5f6fbbe25fc8"), mana_cost: "{2}{G/W}", type_line: "Legendary Creature — Spider Human Hero", oracle_keywords: &["Reach", "Vigilance", "Flash", "No One Dies!"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_4031::RULES.keywords, deck_limit: authored::card_4031::RULES.deck_limit }, effect: authored::card_4031::RULES.checked_effect(), triggers: authored::card_4031::RULES.triggers, activated: authored::card_4031::RULES.activated, enters: authored::card_4031::RULES.enters, entry: authored::card_4031::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 2 }, rules: &authored::card_4031::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Spider-Woman, Secret Agent", oracle_id: Some("04e53f36-9f16-4678-9caf-9afda85f76b4"), mana_cost: "{3}{W/U}", type_line: "Legendary Creature — Spider Human Spy Hero", oracle_keywords: &["Flash"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_4041::RULES.keywords, deck_limit: authored::card_4041::RULES.deck_limit }, effect: authored::card_4041::RULES.checked_effect(), triggers: authored::card_4041::RULES.triggers, activated: authored::card_4041::RULES.activated, enters: authored::card_4041::RULES.enters, entry: authored::card_4041::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_4041::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Stark Industries", oracle_id: Some("52401cb4-1bce-4191-9220-0d93ea4108b8"), mana_cost: "", type_line: "Land", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4103::RULES.keywords, deck_limit: authored::card_4103::RULES.deck_limit }, effect: authored::card_4103::RULES.checked_effect(), triggers: authored::card_4103::RULES.triggers, activated: authored::card_4103::RULES.activated, enters: authored::card_4103::RULES.enters, entry: authored::card_4103::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_4103::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Stark Industries Executive", oracle_id: Some("7c07970d-063a-44a1-9761-b80aafa90dac"), mana_cost: "{R}", type_line: "Creature — Human Advisor", oracle_keywords: &["Treasure"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4104::RULES.keywords, deck_limit: authored::card_4104::RULES.deck_limit }, effect: authored::card_4104::RULES.checked_effect(), triggers: authored::card_4104::RULES.triggers, activated: authored::card_4104::RULES.activated, enters: authored::card_4104::RULES.enters, entry: authored::card_4104::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 0 }, rules: &authored::card_4104::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Stature, Size Shifter", oracle_id: Some("bd49a05c-ed51-4d69-ab9b-11192442ec17"), mana_cost: "{U}", type_line: "Legendary Creature — Human Hero", oracle_keywords: &["Power-up"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_4120::RULES.keywords, deck_limit: authored::card_4120::RULES.deck_limit }, effect: authored::card_4120::RULES.checked_effect(), triggers: authored::card_4120::RULES.triggers, activated: authored::card_4120::RULES.activated, enters: authored::card_4120::RULES.enters, entry: authored::card_4120::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_4120::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Stolen Stark Tech", oracle_id: Some("51023277-5434-4913-b9dc-df2a3fde1652"), mana_cost: "{1}{B}", type_line: "Artifact — Equipment", oracle_keywords: &["Equip", "Flash"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4158::RULES.keywords, deck_limit: authored::card_4158::RULES.deck_limit }, effect: authored::card_4158::RULES.checked_effect(), triggers: authored::card_4158::RULES.triggers, activated: authored::card_4158::RULES.activated, enters: authored::card_4158::RULES.enters, entry: authored::card_4158::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 1 }, rules: &authored::card_4158::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Storm, Windrider", oracle_id: Some("06e9287b-56d4-443e-97ba-b5dcdc4fa989"), mana_cost: "{1}{G}{W}{W}", type_line: "Legendary Creature — Mutant Hero", oracle_keywords: &["Flying"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_4169::RULES.keywords, deck_limit: authored::card_4169::RULES.deck_limit }, effect: authored::card_4169::RULES.checked_effect(), triggers: authored::card_4169::RULES.triggers, activated: authored::card_4169::RULES.activated, enters: authored::card_4169::RULES.enters, entry: authored::card_4169::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [2, 0, 0, 0, 1, 0], generic: 1 }, rules: &authored::card_4169::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Subterranean Cavern", oracle_id: Some("d49a7525-eca5-48e1-b94b-d618a706fd02"), mana_cost: "", type_line: "Land", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4197::RULES.keywords, deck_limit: authored::card_4197::RULES.deck_limit }, effect: authored::card_4197::RULES.checked_effect(), triggers: authored::card_4197::RULES.triggers, activated: authored::card_4197::RULES.activated, enters: authored::card_4197::RULES.enters, entry: authored::card_4197::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_4197::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Super Intelligence", oracle_id: Some("d82929f2-57ad-45be-a834-984437046026"), mana_cost: "{U}", type_line: "Enchantment — Aura", oracle_keywords: &["Enchant"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4252::RULES.keywords, deck_limit: authored::card_4252::RULES.deck_limit }, effect: authored::card_4252::RULES.checked_effect(), triggers: authored::card_4252::RULES.triggers, activated: authored::card_4252::RULES.activated, enters: authored::card_4252::RULES.enters, entry: authored::card_4252::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_4252::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Super Speed", oracle_id: Some("88923ca1-a793-42f0-b9f8-ed9ff9c1185d"), mana_cost: "{R}", type_line: "Enchantment — Aura", oracle_keywords: &["Enchant", "Flash"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4254::RULES.keywords, deck_limit: authored::card_4254::RULES.deck_limit }, effect: authored::card_4254::RULES.checked_effect(), triggers: authored::card_4254::RULES.triggers, activated: authored::card_4254::RULES.activated, enters: authored::card_4254::RULES.enters, entry: authored::card_4254::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 0 }, rules: &authored::card_4254::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Super Strength", oracle_id: Some("e1b577ad-1488-43b5-83d9-e40bd60c74be"), mana_cost: "{4}{G}", type_line: "Enchantment — Aura", oracle_keywords: &["Enchant"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4255::RULES.keywords, deck_limit: authored::card_4255::RULES.deck_limit }, effect: authored::card_4255::RULES.checked_effect(), triggers: authored::card_4255::RULES.triggers, activated: authored::card_4255::RULES.activated, enters: authored::card_4255::RULES.enters, entry: authored::card_4255::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 4 }, rules: &authored::card_4255::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Super Suit", oracle_id: Some("606092eb-8fe3-4ef7-aaf1-2d3b2687d27a"), mana_cost: "{1}{U}", type_line: "Artifact — Equipment", oracle_keywords: &["Equip", "Flash"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4256::RULES.keywords, deck_limit: authored::card_4256::RULES.deck_limit }, effect: authored::card_4256::RULES.checked_effect(), triggers: authored::card_4256::RULES.triggers, activated: authored::card_4256::RULES.activated, enters: authored::card_4256::RULES.enters, entry: authored::card_4256::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_4256::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Super Villain Lockup", oracle_id: Some("192e150a-96b0-42d6-9d39-03d0bed61f41"), mana_cost: "{1}{W}", type_line: "Enchantment", oracle_keywords: &["Flash"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4257::RULES.keywords, deck_limit: authored::card_4257::RULES.deck_limit }, effect: authored::card_4257::RULES.checked_effect(), triggers: authored::card_4257::RULES.triggers, activated: authored::card_4257::RULES.activated, enters: authored::card_4257::RULES.enters, entry: authored::card_4257::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_4257::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Super-Adaptoid", oracle_id: Some("f93fa6ec-1f13-4109-aa60-11e148f7ba88"), mana_cost: "{2}", type_line: "Legendary Artifact Creature — Robot Villain", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_4258::RULES.keywords, deck_limit: authored::card_4258::RULES.deck_limit }, effect: authored::card_4258::RULES.checked_effect(), triggers: authored::card_4258::RULES.triggers, activated: authored::card_4258::RULES.activated, enters: authored::card_4258::RULES.enters, entry: authored::card_4258::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_4258::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Super-Skrull", oracle_id: Some("4644013f-c96a-450e-a7bc-ddfa7a563050"), mana_cost: "{1}{B}{B}{B}", type_line: "Legendary Creature — Skrull Shapeshifter Villain", oracle_keywords: &["Flying"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_4259::RULES.keywords, deck_limit: authored::card_4259::RULES.deck_limit }, effect: authored::card_4259::RULES.checked_effect(), triggers: authored::card_4259::RULES.triggers, activated: authored::card_4259::RULES.activated, enters: authored::card_4259::RULES.enters, entry: authored::card_4259::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 3, 0, 0, 0], generic: 1 }, rules: &authored::card_4259::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Super-Soldier Serum", oracle_id: Some("6f2ba142-5e7c-4709-ae03-e028cb2a9872"), mana_cost: "{1}{W}", type_line: "Enchantment — Aura", oracle_keywords: &["Enchant"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4260::RULES.keywords, deck_limit: authored::card_4260::RULES.deck_limit }, effect: authored::card_4260::RULES.checked_effect(), triggers: authored::card_4260::RULES.triggers, activated: authored::card_4260::RULES.activated, enters: authored::card_4260::RULES.enters, entry: authored::card_4260::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_4260::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Surveillance Room", oracle_id: Some("a3648376-dc8b-409b-b2d1-c29e326a059c"), mana_cost: "", type_line: "Land", oracle_keywords: &["Surveil"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4275::RULES.keywords, deck_limit: authored::card_4275::RULES.deck_limit }, effect: authored::card_4275::RULES.checked_effect(), triggers: authored::card_4275::RULES.triggers, activated: authored::card_4275::RULES.activated, enters: authored::card_4275::RULES.enters, entry: authored::card_4275::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_4275::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Swordsman, Sharp Scoundrel", oracle_id: Some("2f15abe7-54aa-40ed-a15d-5b827f1c3923"), mana_cost: "{1}{B}", type_line: "Legendary Creature — Human Hero Villain", oracle_keywords: &["Connive"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_4297::RULES.keywords, deck_limit: authored::card_4297::RULES.deck_limit }, effect: authored::card_4297::RULES.checked_effect(), triggers: authored::card_4297::RULES.triggers, activated: authored::card_4297::RULES.activated, enters: authored::card_4297::RULES.enters, entry: authored::card_4297::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 1 }, rules: &authored::card_4297::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Take Up the Shield", oracle_id: Some("e8d0eafe-540a-4b2e-a989-e98ff3c31105"), mana_cost: "{1}{W}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4321::RULES.keywords, deck_limit: authored::card_4321::RULES.deck_limit }, effect: authored::card_4321::RULES.checked_effect(), triggers: authored::card_4321::RULES.triggers, activated: authored::card_4321::RULES.activated, enters: authored::card_4321::RULES.enters, entry: authored::card_4321::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_4321::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Taskmaster, Mercenary Mimic", oracle_id: Some("dcd97730-b793-4c0c-b158-625ac3656b4a"), mana_cost: "{2}{U}{B}", type_line: "Legendary Creature — Human Mercenary Villain", oracle_keywords: &["Photographic Reflexes"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_4342::RULES.keywords, deck_limit: authored::card_4342::RULES.deck_limit }, effect: authored::card_4342::RULES.checked_effect(), triggers: authored::card_4342::RULES.triggers, activated: authored::card_4342::RULES.activated, enters: authored::card_4342::RULES.enters, entry: authored::card_4342::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 1, 0, 0, 0], generic: 2 }, rules: &authored::card_4342::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Team Tactics", oracle_id: Some("98b99d7c-d006-471a-8583-0468743c1597"), mana_cost: "{1}{R}", type_line: "Instant", oracle_keywords: &["Teamwork"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4351::RULES.keywords, deck_limit: authored::card_4351::RULES.deck_limit }, effect: authored::card_4351::RULES.checked_effect(), triggers: authored::card_4351::RULES.triggers, activated: authored::card_4351::RULES.activated, enters: authored::card_4351::RULES.enters, entry: authored::card_4351::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 1 }, rules: &authored::card_4351::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Thanos, the Mad Titan", oracle_id: Some("090d815b-b2c8-48cd-a0b3-dc662d5352ac"), mana_cost: "{R}{W}{B}", type_line: "Legendary Creature — Eternal Villain", oracle_keywords: &["Deathtouch", "Power-up", "Lifelink"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_4414::RULES.keywords, deck_limit: authored::card_4414::RULES.deck_limit }, effect: authored::card_4414::RULES.checked_effect(), triggers: authored::card_4414::RULES.triggers, activated: authored::card_4414::RULES.activated, enters: authored::card_4414::RULES.enters, entry: authored::card_4414::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 1, 1, 0, 0], generic: 0 }, rules: &authored::card_4414::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "The Astonishing Ant-Man", oracle_id: Some("6c7e4e51-099a-4398-a4b7-e2f0ddd36429"), mana_cost: "{G}{U}", type_line: "Legendary Creature — Human Scientist Hero", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_4421::RULES.keywords, deck_limit: authored::card_4421::RULES.deck_limit }, effect: authored::card_4421::RULES.checked_effect(), triggers: authored::card_4421::RULES.triggers, activated: authored::card_4421::RULES.activated, enters: authored::card_4421::RULES.enters, entry: authored::card_4421::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 1, 0], generic: 0 }, rules: &authored::card_4421::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "The Coming of Galactus", oracle_id: Some("e05f23ba-f2b2-4c40-83fa-8defce3cb978"), mana_cost: "{2}{B}{B}{G}", type_line: "Enchantment — Saga", oracle_keywords: &["Trample"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4430::RULES.keywords, deck_limit: authored::card_4430::RULES.deck_limit }, effect: authored::card_4430::RULES.checked_effect(), triggers: authored::card_4430::RULES.triggers, activated: authored::card_4430::RULES.activated, enters: authored::card_4430::RULES.enters, entry: authored::card_4430::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 0, 2, 0, 1, 0], generic: 2 }, rules: &authored::card_4430::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "The Kingpin of Crime", oracle_id: Some("3eeeb159-aa45-4a9b-9618-8542604b9830"), mana_cost: "{1}{W}{B}", type_line: "Legendary Creature — Human Villain", oracle_keywords: &["Extort"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_4458::RULES.keywords, deck_limit: authored::card_4458::RULES.deck_limit }, effect: authored::card_4458::RULES.checked_effect(), triggers: authored::card_4458::RULES.triggers, activated: authored::card_4458::RULES.activated, enters: authored::card_4458::RULES.enters, entry: authored::card_4458::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 1, 0, 0, 0], generic: 1 }, rules: &authored::card_4458::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "The Masters of Evil", oracle_id: Some("fcb1caea-e1ac-48f0-a67a-d4aadad490d9"), mana_cost: "{5}{B}", type_line: "Legendary Creature — Human Villain", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_4473::RULES.keywords, deck_limit: authored::card_4473::RULES.deck_limit }, effect: authored::card_4473::RULES.checked_effect(), triggers: authored::card_4473::RULES.triggers, activated: authored::card_4473::RULES.activated, enters: authored::card_4473::RULES.enters, entry: authored::card_4473::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 5 }, rules: &authored::card_4473::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "The Mighty Thor, Jane Foster", oracle_id: Some("57d02dc8-e22e-4874-9f02-490a2528a28f"), mana_cost: "{1}{W}{U}", type_line: "Legendary Creature — Human God Hero", oracle_keywords: &["Flying"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_4475::RULES.keywords, deck_limit: authored::card_4475::RULES.deck_limit }, effect: authored::card_4475::RULES.checked_effect(), triggers: authored::card_4475::RULES.triggers, activated: authored::card_4475::RULES.activated, enters: authored::card_4475::RULES.enters, entry: authored::card_4475::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 1, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_4475::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "The Mind Stone", oracle_id: Some("b175e826-09e8-4fae-9f2e-b902f95b282d"), mana_cost: "{1}{W}", type_line: "Legendary Artifact — Infinity Stone", oracle_keywords: &["Indestructible"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_4477::RULES.keywords, deck_limit: authored::card_4477::RULES.deck_limit }, effect: authored::card_4477::RULES.checked_effect(), triggers: authored::card_4477::RULES.triggers, activated: authored::card_4477::RULES.activated, enters: authored::card_4477::RULES.enters, entry: authored::card_4477::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_4477::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "The Ruinous Wrecking Crew", oracle_id: Some("0ff9c0cc-4fb3-4117-9a8e-5b51f62306c4"), mana_cost: "{X}{B}{R}", type_line: "Legendary Creature — Human Villain", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_4492::RULES.keywords, deck_limit: authored::card_4492::RULES.deck_limit }, effect: authored::card_4492::RULES.checked_effect(), triggers: authored::card_4492::RULES.triggers, activated: authored::card_4492::RULES.activated, enters: authored::card_4492::RULES.enters, entry: authored::card_4492::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 1, 0, 0], generic: 0 }, rules: &authored::card_4492::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "The Scarlet Witch", oracle_id: Some("d8ae8c38-501e-44ea-9faf-cf9d11ba6616"), mana_cost: "{2}{R}", type_line: "Legendary Creature — Mutant Warlock Hero", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_4494::RULES.keywords, deck_limit: authored::card_4494::RULES.deck_limit }, effect: authored::card_4494::RULES.checked_effect(), triggers: authored::card_4494::RULES.triggers, activated: authored::card_4494::RULES.activated, enters: authored::card_4494::RULES.enters, entry: authored::card_4494::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 2 }, rules: &authored::card_4494::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "The Sentry, Golden Guardian", oracle_id: Some("1fe549c3-79be-40ef-b57c-de59f24df5ab"), mana_cost: "{3}{W}", type_line: "Legendary Creature — Human Hero", oracle_keywords: &["Flying", "Indestructible", "Vigilance"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_4495::RULES.keywords, deck_limit: authored::card_4495::RULES.deck_limit }, effect: authored::card_4495::RULES.checked_effect(), triggers: authored::card_4495::RULES.triggers, activated: authored::card_4495::RULES.activated, enters: authored::card_4495::RULES.enters, entry: authored::card_4495::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_4495::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "The Serpent Society", oracle_id: Some("3688ad0d-653b-4d5d-a57e-444db679d83e"), mana_cost: "{1}{B}{G}", type_line: "Legendary Creature — Human Snake Villain", oracle_keywords: &["Ward", "Deathtouch"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_4497::RULES.keywords, deck_limit: authored::card_4497::RULES.deck_limit }, effect: authored::card_4497::RULES.checked_effect(), triggers: authored::card_4497::RULES.triggers, activated: authored::card_4497::RULES.activated, enters: authored::card_4497::RULES.enters, entry: authored::card_4497::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 1, 0], generic: 1 }, rules: &authored::card_4497::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "The Super Hero Civil War", oracle_id: Some("11d74dc2-8c11-4d4b-b727-8216d2eb6b4e"), mana_cost: "{3}{R}{W}", type_line: "Enchantment — Saga", oracle_keywords: &["Fight"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4505::RULES.keywords, deck_limit: authored::card_4505::RULES.deck_limit }, effect: authored::card_4505::RULES.checked_effect(), triggers: authored::card_4505::RULES.triggers, activated: authored::card_4505::RULES.activated, enters: authored::card_4505::RULES.enters, entry: authored::card_4505::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [1, 0, 0, 1, 0, 0], generic: 3 }, rules: &authored::card_4505::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "The Ten Rings", oracle_id: Some("a82d855c-50c8-43f2-99ba-a84fe84539c5"), mana_cost: "{8}", type_line: "Legendary Artifact", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_4508::RULES.keywords, deck_limit: authored::card_4508::RULES.deck_limit }, effect: authored::card_4508::RULES.checked_effect(), triggers: authored::card_4508::RULES.triggers, activated: authored::card_4508::RULES.activated, enters: authored::card_4508::RULES.enters, entry: authored::card_4508::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 8 }, rules: &authored::card_4508::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "The Thing, Ben Grimm", oracle_id: Some("d566d5f1-5abd-405b-8553-d267ecbe72ab"), mana_cost: "{5}{G}", type_line: "Legendary Creature — Human Hero", oracle_keywords: &["Trample"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_4510::RULES.keywords, deck_limit: authored::card_4510::RULES.deck_limit }, effect: authored::card_4510::RULES.checked_effect(), triggers: authored::card_4510::RULES.triggers, activated: authored::card_4510::RULES.activated, enters: authored::card_4510::RULES.enters, entry: authored::card_4510::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 5 }, rules: &authored::card_4510::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "The Unbeatable Squirrel Girl", oracle_id: Some("80312910-5d53-44f1-9982-e46dc7532abb"), mana_cost: "{1}{G}{G}{G}", type_line: "Legendary Creature — Squirrel Human Hero", oracle_keywords: &["I LOVE Squirrels!"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_4512::RULES.keywords, deck_limit: authored::card_4512::RULES.deck_limit }, effect: authored::card_4512::RULES.checked_effect(), triggers: authored::card_4512::RULES.triggers, activated: authored::card_4512::RULES.activated, enters: authored::card_4512::RULES.enters, entry: authored::card_4512::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 3, 0], generic: 1 }, rules: &authored::card_4512::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "The Vision", oracle_id: Some("32aee76a-e738-4898-985a-801d1cce15ea"), mana_cost: "{4}", type_line: "Legendary Artifact Creature — Robot Hero", oracle_keywords: &["Density Control", "Flying", "Vigilance", "Technopathy", "Solar Beam"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_4513::RULES.keywords, deck_limit: authored::card_4513::RULES.deck_limit }, effect: authored::card_4513::RULES.checked_effect(), triggers: authored::card_4513::RULES.triggers, activated: authored::card_4513::RULES.activated, enters: authored::card_4513::RULES.enters, entry: authored::card_4513::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 4 }, rules: &authored::card_4513::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "The Wondrous Wasp", oracle_id: Some("5ad8963b-8091-4787-9147-20a1fe7bf0fb"), mana_cost: "{1}{U}", type_line: "Legendary Creature — Human Hero", oracle_keywords: &["Flying", "Wasp's Sting", "Flash"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_4520::RULES.keywords, deck_limit: authored::card_4520::RULES.deck_limit }, effect: authored::card_4520::RULES.checked_effect(), triggers: authored::card_4520::RULES.triggers, activated: authored::card_4520::RULES.activated, enters: authored::card_4520::RULES.enters, entry: authored::card_4520::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_4520::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Thirst for Knowledge", oracle_id: Some("939e6f71-185e-41f2-9d54-72cce06f1dce"), mana_cost: "{2}{U}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4533::RULES.keywords, deck_limit: authored::card_4533::RULES.deck_limit }, effect: authored::card_4533::RULES.checked_effect(), triggers: authored::card_4533::RULES.triggers, activated: authored::card_4533::RULES.activated, enters: authored::card_4533::RULES.enters, entry: authored::card_4533::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_4533::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Thor Odinson", oracle_id: Some("ec9b74d0-e4fe-4764-b628-992a62f2a1cf"), mana_cost: "{3}{R}{W}", type_line: "Legendary Creature — God Warrior Hero", oracle_keywords: &["Flying", "Prowess", "Vigilance"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_4537::RULES.keywords, deck_limit: authored::card_4537::RULES.deck_limit }, effect: authored::card_4537::RULES.checked_effect(), triggers: authored::card_4537::RULES.triggers, activated: authored::card_4537::RULES.activated, enters: authored::card_4537::RULES.enters, entry: authored::card_4537::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 1, 0, 0], generic: 3 }, rules: &authored::card_4537::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Thor, God of Thunder", oracle_id: Some("2842217a-173e-4879-8b25-fefe2fc1123f"), mana_cost: "{3}{R}{R}", type_line: "Legendary Creature — God Warrior Hero", oracle_keywords: &["Flying"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_4538::RULES.keywords, deck_limit: authored::card_4538::RULES.deck_limit }, effect: authored::card_4538::RULES.checked_effect(), triggers: authored::card_4538::RULES.triggers, activated: authored::card_4538::RULES.activated, enters: authored::card_4538::RULES.enters, entry: authored::card_4538::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 2, 0, 0], generic: 3 }, rules: &authored::card_4538::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Thunderbolts Conspiracy", oracle_id: Some("d90a5213-3363-4cd1-97d6-4fc057f78b2a"), mana_cost: "{3}{B}", type_line: "Enchantment", oracle_keywords: &["Flash"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4582::RULES.keywords, deck_limit: authored::card_4582::RULES.deck_limit }, effect: authored::card_4582::RULES.checked_effect(), triggers: authored::card_4582::RULES.triggers, activated: authored::card_4582::RULES.activated, enters: authored::card_4582::RULES.enters, entry: authored::card_4582::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 3 }, rules: &authored::card_4582::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Tigra, Feline Fury", oracle_id: Some("39d993be-30ad-4756-a494-fd8808d474c9"), mana_cost: "{1}{G}", type_line: "Legendary Creature — Cat Human Hero", oracle_keywords: &["Trample", "Flash"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_4600::RULES.keywords, deck_limit: authored::card_4600::RULES.deck_limit }, effect: authored::card_4600::RULES.checked_effect(), triggers: authored::card_4600::RULES.triggers, activated: authored::card_4600::RULES.activated, enters: authored::card_4600::RULES.enters, entry: authored::card_4600::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 1 }, rules: &authored::card_4600::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Titania, Rugged Rumbler", oracle_id: Some("e380e37d-926b-4a4b-a275-7844bf4956d5"), mana_cost: "{2}{B/G}", type_line: "Legendary Creature — Human Villain", oracle_keywords: &["Ward"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_4613::RULES.keywords, deck_limit: authored::card_4613::RULES.deck_limit }, effect: authored::card_4613::RULES.checked_effect(), triggers: authored::card_4613::RULES.triggers, activated: authored::card_4613::RULES.activated, enters: authored::card_4613::RULES.enters, entry: authored::card_4613::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 2 }, rules: &authored::card_4613::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Tony Stark", oracle_id: Some("4a46ea59-6c5d-4bed-9951-b5590f9a1bdb"), mana_cost: "{1}{U}", type_line: "Legendary Creature — Human Artificer Hero", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_4631::RULES.keywords, deck_limit: authored::card_4631::RULES.deck_limit }, effect: authored::card_4631::RULES.checked_effect(), triggers: authored::card_4631::RULES.triggers, activated: authored::card_4631::RULES.activated, enters: authored::card_4631::RULES.enters, entry: authored::card_4631::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_4631::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Too Evil to Stay Dead", oracle_id: Some("e50fecff-8872-42f5-8882-41ad13d9d1ae"), mana_cost: "{2}{B}", type_line: "Sorcery", oracle_keywords: &["Teamwork"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4632::RULES.keywords, deck_limit: authored::card_4632::RULES.deck_limit }, effect: authored::card_4632::RULES.checked_effect(), triggers: authored::card_4632::RULES.triggers, activated: authored::card_4632::RULES.activated, enters: authored::card_4632::RULES.enters, entry: authored::card_4632::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 2 }, rules: &authored::card_4632::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Training Compound", oracle_id: Some("99c70f4e-de8a-426d-99aa-17b2f87625ba"), mana_cost: "", type_line: "Land", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4653::RULES.keywords, deck_limit: authored::card_4653::RULES.deck_limit }, effect: authored::card_4653::RULES.checked_effect(), triggers: authored::card_4653::RULES.triggers, activated: authored::card_4653::RULES.activated, enters: authored::card_4653::RULES.enters, entry: authored::card_4653::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_4653::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Training Regimen", oracle_id: Some("f44bde8b-43e9-4719-ac60-b8f0cee8df9b"), mana_cost: "{3}{G}", type_line: "Enchantment", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4654::RULES.keywords, deck_limit: authored::card_4654::RULES.deck_limit }, effect: authored::card_4654::RULES.checked_effect(), triggers: authored::card_4654::RULES.triggers, activated: authored::card_4654::RULES.activated, enters: authored::card_4654::RULES.enters, entry: authored::card_4654::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 3 }, rules: &authored::card_4654::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Trickster's Stratagem", oracle_id: Some("170ee399-d3cd-40ef-a340-193c0f0f6002"), mana_cost: "{3}{U}", type_line: "Sorcery", oracle_keywords: &["Connive"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4682::RULES.keywords, deck_limit: authored::card_4682::RULES.deck_limit }, effect: authored::card_4682::RULES.checked_effect(), triggers: authored::card_4682::RULES.triggers, activated: authored::card_4682::RULES.activated, enters: authored::card_4682::RULES.enters, entry: authored::card_4682::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_4682::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Truck Toss", oracle_id: Some("7d765055-da62-43f7-9580-98e9d164753b"), mana_cost: "{2}{R}{R}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4691::RULES.keywords, deck_limit: authored::card_4691::RULES.deck_limit }, effect: authored::card_4691::RULES.checked_effect(), triggers: authored::card_4691::RULES.triggers, activated: authored::card_4691::RULES.activated, enters: authored::card_4691::RULES.enters, entry: authored::card_4691::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 0, 2, 0, 0], generic: 2 }, rules: &authored::card_4691::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "U.S.Agent, John Walker", oracle_id: Some("2b6fa8b6-4865-4eb6-974b-00f6b16b6f0f"), mana_cost: "{3}{W/B}", type_line: "Legendary Creature — Human Soldier Hero", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_4736::RULES.keywords, deck_limit: authored::card_4736::RULES.deck_limit }, effect: authored::card_4736::RULES.checked_effect(), triggers: authored::card_4736::RULES.triggers, activated: authored::card_4736::RULES.activated, enters: authored::card_4736::RULES.enters, entry: authored::card_4736::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 3 }, rules: &authored::card_4736::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Ultron Drone", oracle_id: Some("51c1f21d-1512-4885-bfdc-58790cf77d51"), mana_cost: "{3}", type_line: "Artifact Creature — Robot Villain", oracle_keywords: &["Power-up"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4747::RULES.keywords, deck_limit: authored::card_4747::RULES.deck_limit }, effect: authored::card_4747::RULES.checked_effect(), triggers: authored::card_4747::RULES.triggers, activated: authored::card_4747::RULES.activated, enters: authored::card_4747::RULES.enters, entry: authored::card_4747::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_4747::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Ultron, Artificial Malevolence", oracle_id: Some("a8f25e67-2112-4355-9298-363ed04a5cb8"), mana_cost: "{3}", type_line: "Legendary Artifact Creature — Robot Villain", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_4748::RULES.keywords, deck_limit: authored::card_4748::RULES.deck_limit }, effect: authored::card_4748::RULES.checked_effect(), triggers: authored::card_4748::RULES.triggers, activated: authored::card_4748::RULES.activated, enters: authored::card_4748::RULES.enters, entry: authored::card_4748::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_4748::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Undercover Skrull", oracle_id: Some("33b1379f-b329-42cd-92ed-4811d44ed8dd"), mana_cost: "{1}{G}", type_line: "Creature — Skrull Shapeshifter Villain", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4766::RULES.keywords, deck_limit: authored::card_4766::RULES.deck_limit }, effect: authored::card_4766::RULES.checked_effect(), triggers: authored::card_4766::RULES.triggers, activated: authored::card_4766::RULES.activated, enters: authored::card_4766::RULES.enters, entry: authored::card_4766::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 1 }, rules: &authored::card_4766::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Unliving Legionnaire", oracle_id: Some("0edaf433-4ded-4841-bc86-136780c0767f"), mana_cost: "{3}{B}", type_line: "Creature — Vampire Villain", oracle_keywords: &["Flying", "Power-up"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4788::RULES.keywords, deck_limit: authored::card_4788::RULES.deck_limit }, effect: authored::card_4788::RULES.checked_effect(), triggers: authored::card_4788::RULES.triggers, activated: authored::card_4788::RULES.activated, enters: authored::card_4788::RULES.enters, entry: authored::card_4788::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 3 }, rules: &authored::card_4788::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Vibranium Energy Daggers", oracle_id: Some("be0b86fc-8450-41d4-9836-6de11cecb857"), mana_cost: "{1}", type_line: "Artifact — Equipment", oracle_keywords: &["Indestructible", "Equip"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4883::RULES.keywords, deck_limit: authored::card_4883::RULES.deck_limit }, effect: authored::card_4883::RULES.checked_effect(), triggers: authored::card_4883::RULES.triggers, activated: authored::card_4883::RULES.activated, enters: authored::card_4883::RULES.enters, entry: authored::card_4883::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_4883::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Villainous Hideout", oracle_id: Some("cd2888aa-71f3-47ee-ba33-7bb95d5bc836"), mana_cost: "", type_line: "Land", oracle_keywords: &["Connive"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4894::RULES.keywords, deck_limit: authored::card_4894::RULES.deck_limit }, effect: authored::card_4894::RULES.checked_effect(), triggers: authored::card_4894::RULES.triggers, activated: authored::card_4894::RULES.activated, enters: authored::card_4894::RULES.enters, entry: authored::card_4894::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_4894::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Vision Quest", oracle_id: Some("240410a0-3288-4336-b1ad-34936e64ccf2"), mana_cost: "{X}{U}{R}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4915::RULES.keywords, deck_limit: authored::card_4915::RULES.deck_limit }, effect: authored::card_4915::RULES.checked_effect(), triggers: authored::card_4915::RULES.triggers, activated: authored::card_4915::RULES.activated, enters: authored::card_4915::RULES.enters, entry: authored::card_4915::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 1, 0, 1, 0, 0], generic: 0 }, rules: &authored::card_4915::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Vision of Love", oracle_id: Some("70c96b50-ac2b-493d-bd15-fc66eb767111"), mana_cost: "{1}{R}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4916::RULES.keywords, deck_limit: authored::card_4916::RULES.deck_limit }, effect: authored::card_4916::RULES.checked_effect(), triggers: authored::card_4916::RULES.triggers, activated: authored::card_4916::RULES.activated, enters: authored::card_4916::RULES.enters, entry: authored::card_4916::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 1 }, rules: &authored::card_4916::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Visions of Villainy", oracle_id: Some("3b46800b-711b-4cbb-b296-26115b4f6839"), mana_cost: "{2}{B}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4918::RULES.keywords, deck_limit: authored::card_4918::RULES.deck_limit }, effect: authored::card_4918::RULES.checked_effect(), triggers: authored::card_4918::RULES.triggers, activated: authored::card_4918::RULES.activated, enters: authored::card_4918::RULES.enters, entry: authored::card_4918::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 2 }, rules: &authored::card_4918::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Viv Vision, Teen Synthezoid", oracle_id: Some("f67a599c-200a-4831-9e73-126c7435a396"), mana_cost: "{3}", type_line: "Legendary Artifact Creature — Robot Hero", oracle_keywords: &["Flying", "Power-up", "Cybernetic Senses"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_4922::RULES.keywords, deck_limit: authored::card_4922::RULES.deck_limit }, effect: authored::card_4922::RULES.checked_effect(), triggers: authored::card_4922::RULES.triggers, activated: authored::card_4922::RULES.activated, enters: authored::card_4922::RULES.enters, entry: authored::card_4922::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_4922::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Volcanic Villain", oracle_id: Some("a257426d-70c9-4d58-bd8b-97ec38f6e133"), mana_cost: "{2}{R}", type_line: "Creature — Elemental Villain", oracle_keywords: &["Power-up", "Haste"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4933::RULES.keywords, deck_limit: authored::card_4933::RULES.deck_limit }, effect: authored::card_4933::RULES.checked_effect(), triggers: authored::card_4933::RULES.triggers, activated: authored::card_4933::RULES.activated, enters: authored::card_4933::RULES.enters, entry: authored::card_4933::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 2 }, rules: &authored::card_4933::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Wakandan Drone Flock", oracle_id: Some("282d6e61-3240-4f1e-9c2d-8ddb59b03af4"), mana_cost: "{3}{W}", type_line: "Artifact Creature — Robot", oracle_keywords: &["Scry", "Flying"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4953::RULES.keywords, deck_limit: authored::card_4953::RULES.deck_limit }, effect: authored::card_4953::RULES.checked_effect(), triggers: authored::card_4953::RULES.triggers, activated: authored::card_4953::RULES.activated, enters: authored::card_4953::RULES.enters, entry: authored::card_4953::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_4953::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Wakandan Royal Guard", oracle_id: Some("4ae15bd1-e0ea-4c1a-a311-dae421381ebc"), mana_cost: "{4}{G}", type_line: "Creature — Human Soldier Hero", oracle_keywords: &["Vigilance"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4954::RULES.keywords, deck_limit: authored::card_4954::RULES.deck_limit }, effect: authored::card_4954::RULES.checked_effect(), triggers: authored::card_4954::RULES.triggers, activated: authored::card_4954::RULES.activated, enters: authored::card_4954::RULES.enters, entry: authored::card_4954::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 4 }, rules: &authored::card_4954::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "War Machine, Legacy of Iron", oracle_id: Some("ae6d1aa3-4ceb-4fc1-a3a6-033afd4b3bce"), mana_cost: "{2}{R/W}", type_line: "Legendary Artifact Creature — Human Hero", oracle_keywords: &["Flying"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_4972::RULES.keywords, deck_limit: authored::card_4972::RULES.deck_limit }, effect: authored::card_4972::RULES.checked_effect(), triggers: authored::card_4972::RULES.triggers, activated: authored::card_4972::RULES.activated, enters: authored::card_4972::RULES.enters, entry: authored::card_4972::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 2 }, rules: &authored::card_4972::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "We Say Thee Nay!", oracle_id: Some("1abe8246-d2f9-407b-8c16-83da0a6b7de3"), mana_cost: "{1}{U}", type_line: "Instant — Arcane", oracle_keywords: &["Teamwork"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_5019::RULES.keywords, deck_limit: authored::card_5019::RULES.deck_limit }, effect: authored::card_5019::RULES.checked_effect(), triggers: authored::card_5019::RULES.triggers, activated: authored::card_5019::RULES.activated, enters: authored::card_5019::RULES.enters, entry: authored::card_5019::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_5019::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Web Up", oracle_id: Some("9b9b086c-d201-4574-a1e9-0d6f8de53d7d"), mana_cost: "{2}{W}", type_line: "Enchantment", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_5024::RULES.keywords, deck_limit: authored::card_5024::RULES.deck_limit }, effect: authored::card_5024::RULES.checked_effect(), triggers: authored::card_5024::RULES.triggers, activated: authored::card_5024::RULES.activated, enters: authored::card_5024::RULES.enters, entry: authored::card_5024::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_5024::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Whiplash, Vengeful Engineer", oracle_id: Some("00ed7aca-b3c1-4d48-acd5-b1596daeb4c3"), mana_cost: "{B}", type_line: "Legendary Creature — Human Artificer Villain", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_5038::RULES.keywords, deck_limit: authored::card_5038::RULES.deck_limit }, effect: authored::card_5038::RULES.checked_effect(), triggers: authored::card_5038::RULES.triggers, activated: authored::card_5038::RULES.activated, enters: authored::card_5038::RULES.enters, entry: authored::card_5038::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 0 }, rules: &authored::card_5038::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "White Tiger, Ava Ayala", oracle_id: Some("e2084d73-5a4c-4b17-942a-93c1a1f7ac2a"), mana_cost: "{1}{G}", type_line: "Legendary Creature — Human Hero", oracle_keywords: &["Power-up"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_5047::RULES.keywords, deck_limit: authored::card_5047::RULES.deck_limit }, effect: authored::card_5047::RULES.checked_effect(), triggers: authored::card_5047::RULES.triggers, activated: authored::card_5047::RULES.activated, enters: authored::card_5047::RULES.enters, entry: authored::card_5047::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 1 }, rules: &authored::card_5047::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "White Widow, Free Agent", oracle_id: Some("4f0c055f-f614-44b0-9374-c53541f89185"), mana_cost: "{3}{W}", type_line: "Legendary Creature — Human Hero Villain", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_5048::RULES.keywords, deck_limit: authored::card_5048::RULES.deck_limit }, effect: authored::card_5048::RULES.checked_effect(), triggers: authored::card_5048::RULES.triggers, activated: authored::card_5048::RULES.activated, enters: authored::card_5048::RULES.enters, entry: authored::card_5048::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_5048::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Wiccan, Rising Magician", oracle_id: Some("fc17b7a3-7711-4449-80bd-692e8ed9087d"), mana_cost: "{4}{U}", type_line: "Legendary Creature — Mutant Warlock Hero", oracle_keywords: &["Flying"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_5050::RULES.keywords, deck_limit: authored::card_5050::RULES.deck_limit }, effect: authored::card_5050::RULES.checked_effect(), triggers: authored::card_5050::RULES.triggers, activated: authored::card_5050::RULES.activated, enters: authored::card_5050::RULES.enters, entry: authored::card_5050::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 4 }, rules: &authored::card_5050::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Widow's Bite", oracle_id: Some("39f2a632-30d2-4b35-9e7f-fef27713a4f7"), mana_cost: "{1}{B}", type_line: "Instant", oracle_keywords: &["Teamwork"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_5056::RULES.keywords, deck_limit: authored::card_5056::RULES.deck_limit }, effect: authored::card_5056::RULES.checked_effect(), triggers: authored::card_5056::RULES.triggers, activated: authored::card_5056::RULES.activated, enters: authored::card_5056::RULES.enters, entry: authored::card_5056::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 1 }, rules: &authored::card_5056::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Winter Soldier, Icy Assassin", oracle_id: Some("6f4c94b6-f829-4646-a574-083997b78dfb"), mana_cost: "{W}{B}", type_line: "Legendary Creature — Human Assassin Villain", oracle_keywords: &["Vigilance", "Menace"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_5082::RULES.keywords, deck_limit: authored::card_5082::RULES.deck_limit }, effect: authored::card_5082::RULES.checked_effect(), triggers: authored::card_5082::RULES.triggers, activated: authored::card_5082::RULES.activated, enters: authored::card_5082::RULES.enters, entry: authored::card_5082::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 1, 0, 0, 0], generic: 0 }, rules: &authored::card_5082::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Wolverine, Fierce Fighter", oracle_id: Some("8429ba6a-6c06-4a83-a6ee-023b64b825f8"), mana_cost: "{2}{R}{G}", type_line: "Legendary Creature — Mutant Berserker Hero", oracle_keywords: &["Heal", "Haste", "Fight"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_5106::RULES.keywords, deck_limit: authored::card_5106::RULES.deck_limit }, effect: authored::card_5106::RULES.checked_effect(), triggers: authored::card_5106::RULES.triggers, activated: authored::card_5106::RULES.activated, enters: authored::card_5106::RULES.enters, entry: authored::card_5106::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 1, 0], generic: 2 }, rules: &authored::card_5106::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Wonder Man, Hollywood Hero", oracle_id: Some("e2c49754-711e-43b3-82f1-5376a4d99701"), mana_cost: "{3}{R}{R}", type_line: "Legendary Creature — Human Performer Hero", oracle_keywords: &["Flying", "Power-up"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_5107::RULES.keywords, deck_limit: authored::card_5107::RULES.deck_limit }, effect: authored::card_5107::RULES.checked_effect(), triggers: authored::card_5107::RULES.triggers, activated: authored::card_5107::RULES.activated, enters: authored::card_5107::RULES.enters, entry: authored::card_5107::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 2, 0, 0], generic: 3 }, rules: &authored::card_5107::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "World War Hulk", oracle_id: Some("8c4da760-8913-46ca-b00c-bba6d877d482"), mana_cost: "{3}{G}{G}", type_line: "Enchantment — Saga", oracle_keywords: &["Double"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_5113::RULES.keywords, deck_limit: authored::card_5113::RULES.deck_limit }, effect: authored::card_5113::RULES.checked_effect(), triggers: authored::card_5113::RULES.triggers, activated: authored::card_5113::RULES.activated, enters: authored::card_5113::RULES.enters, entry: authored::card_5113::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 0, 0, 0, 2, 0], generic: 3 }, rules: &authored::card_5113::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Worlds Within Worlds", oracle_id: Some("e25062ba-7176-4d59-ac3e-26d19b26f02a"), mana_cost: "{5}{G}{U}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_5114::RULES.keywords, deck_limit: authored::card_5114::RULES.deck_limit }, effect: authored::card_5114::RULES.checked_effect(), triggers: authored::card_5114::RULES.triggers, activated: authored::card_5114::RULES.activated, enters: authored::card_5114::RULES.enters, entry: authored::card_5114::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 1, 0, 0, 1, 0], generic: 5 }, rules: &authored::card_5114::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Yellowjacket, Heartless Marauder", oracle_id: Some("0a364879-31bd-4760-833d-165cde1e96c6"), mana_cost: "{1}{B}", type_line: "Legendary Creature — Human Rogue Villain", oracle_keywords: &["Flying"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_5136::RULES.keywords, deck_limit: authored::card_5136::RULES.deck_limit }, effect: authored::card_5136::RULES.checked_effect(), triggers: authored::card_5136::RULES.triggers, activated: authored::card_5136::RULES.activated, enters: authored::card_5136::RULES.enters, entry: authored::card_5136::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 1 }, rules: &authored::card_5136::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Photon, Living Light", oracle_id: None, mana_cost: "{2}{R}{W}{W}", type_line: "Legendary Creature — Elemental Hero", oracle_keywords: &["Flying", "Hexproof", "Prowess"], standard_legal: false, legendary: true, implemented: true, keywords: authored::card_5391::RULES.keywords, deck_limit: authored::card_5391::RULES.deck_limit }, effect: authored::card_5391::RULES.checked_effect(), triggers: authored::card_5391::RULES.triggers, activated: authored::card_5391::RULES.activated, enters: authored::card_5391::RULES.enters, entry: authored::card_5391::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [2, 0, 0, 1, 0, 0], generic: 2 }, rules: &authored::card_5391::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Black Panther, Hope Enduring", oracle_id: None, mana_cost: "{4}{W}{U}", type_line: "Legendary Creature — Human Warrior Hero", oracle_keywords: &["Flash", "Double strike"], standard_legal: false, legendary: true, implemented: true, keywords: authored::card_5392::RULES.keywords, deck_limit: authored::card_5392::RULES.deck_limit }, effect: authored::card_5392::RULES.checked_effect(), triggers: authored::card_5392::RULES.triggers, activated: authored::card_5392::RULES.activated, enters: authored::card_5392::RULES.enters, entry: authored::card_5392::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 1, 0, 0, 0, 0], generic: 4 }, rules: &authored::card_5392::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "The Sensational She-Hulk", oracle_id: None, mana_cost: "{3}{G}{W}{W}", type_line: "Legendary Creature — Gamma Hero", oracle_keywords: &["Reach", "Trample"], standard_legal: false, legendary: true, implemented: true, keywords: authored::card_5396::RULES.keywords, deck_limit: authored::card_5396::RULES.deck_limit }, effect: authored::card_5396::RULES.checked_effect(), triggers: authored::card_5396::RULES.triggers, activated: authored::card_5396::RULES.activated, enters: authored::card_5396::RULES.enters, entry: authored::card_5396::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [2, 0, 0, 0, 1, 0], generic: 3 }, rules: &authored::card_5396::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "The Invincible Iron Man", oracle_id: None, mana_cost: "{4}{U}{R}", type_line: "Legendary Artifact Creature — Human Hero", oracle_keywords: &["Flying", "Haste"], standard_legal: false, legendary: true, implemented: true, keywords: authored::card_5403::RULES.keywords, deck_limit: authored::card_5403::RULES.deck_limit }, effect: authored::card_5403::RULES.checked_effect(), triggers: authored::card_5403::RULES.triggers, activated: authored::card_5403::RULES.activated, enters: authored::card_5403::RULES.enters, entry: authored::card_5403::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 1, 0, 1, 0, 0], generic: 4 }, rules: &authored::card_5403::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "The Incredible Hulk", oracle_id: None, mana_cost: "{2}{R}{R}{G}{G}", type_line: "Legendary Creature — Gamma Berserker Hero", oracle_keywords: &["Reach", "Trample"], standard_legal: false, legendary: true, implemented: true, keywords: authored::card_5404::RULES.keywords, deck_limit: authored::card_5404::RULES.deck_limit }, effect: authored::card_5404::RULES.checked_effect(), triggers: authored::card_5404::RULES.triggers, activated: authored::card_5404::RULES.activated, enters: authored::card_5404::RULES.enters, entry: authored::card_5404::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 2, 2, 0], generic: 2 }, rules: &authored::card_5404::RULES },
];
}
pub use cards::ENTRIES;
