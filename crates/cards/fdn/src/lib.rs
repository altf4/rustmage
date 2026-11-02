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
pub(super) mod card_0008 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/llanowar_elves.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0043 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/abyssal_harvester.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0054 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/adamant_will.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0055 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/adaptive_automaton.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0064 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/adventuring_gear.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0067 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/aegis_turtle.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0073 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/aetherize.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0076 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/affectionate_indrik.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0096 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/aggressive_mammoth.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0114 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/ajani_s_pridemate.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0116 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/ajani_caller_of_the_pride.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0128 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/alesha_who_laughs_at_fate.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0148 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/ambush_wolf.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0150 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/an_offer_you_can_t_refuse.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0154 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/ancestor_dragon.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0162 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/angel_of_finality.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0163 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/angel_of_vitality.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0176 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/anthem_of_champions.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0185 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/apothecary_stomper.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0198 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/arahbo_the_first_fang.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0201 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/arbiter_of_woe.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0204 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/arcane_epiphany.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0206 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/arcanis_the_omnipotent.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0213 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/archmage_of_runes.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0217 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/archway_angel.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0224 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/armasaur_guide.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0244 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/ashroot_animist.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0267 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/aurelia_the_warleader.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0273 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/authority_of_the_consuls.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0291 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/axgard_cavalry.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0292 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/ayli_eternal_pilgrim.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0295 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/azorius_guildgate.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0307 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/bake_into_a_pie.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0313 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/ball_lightning.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0314 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/ballyrush_banneret.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0315 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/balmor_battlemage_captain.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0322 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/banner_of_kinship.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0345 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/basilisk_collar.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0350 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/battle_rattle_shaman.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0351 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/battlesong_berserker.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0357 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/bear_cub.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0361 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/beast_kin_ranger.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0406 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/bigfin_bouncer.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0413 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/billowing_shriekmass.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0416 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/biogenic_upgrade.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0422 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/bishop_s_soldier.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0423 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/bite_down.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0438 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/blanchwood_armor.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0439 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/blasphemous_edict.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0460 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/bloodthirsty_conqueror.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0462 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/bloodtithe_collector.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0490 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/boltwave.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0505 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/boros_charm.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0506 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/boros_guildgate.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0527 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/brass_s_bounty.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0534 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/brazen_scourge.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0555 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/brineborn_cutthroat.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0578 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/bulk_up.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0587 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/burglar_rat.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0594 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/burnished_hart.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0598 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/burrog_befuddler.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0600 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/burst_lightning.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0601 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/bushwhack.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0610 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/cackling_prowler.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0629 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/campus_guide.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0630 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/cancel.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0655 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/carnelian_orb_of_dragonkind.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0677 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/cat_collector.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0695 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/celestial_armor.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0697 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/cemetery_recruitment.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0700 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/cephalid_inkmage.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0715 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/chandra_flameshaper.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0726 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/charming_prince.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0729 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/chart_a_course.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0755 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/circuitous_route.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0766 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/claws_out.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0770 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/clinquant_skymage.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0779 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/cloudblazer.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0828 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/confiscate.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0857 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/corsair_captain.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0871 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/courageous_goblin.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0876 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/crackling_cyclops.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0878 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/crash_through.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0883 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/crawling_barrens.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0896 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/crow_of_dark_tidings.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0907 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/crusader_of_odric.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0912 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/crypt_feaster.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0915 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/cryptic_caves.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0918 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/crystal_barricade.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0923 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/cultivator_s_caravan.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0927 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/curator_of_destinies.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0973 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/darksteel_colossus.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0977 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/dauntless_veteran.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0985 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/dawnwing_marshal.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0987 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/day_of_judgment.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0989 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/dazzling_angel.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_0998 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/deadly_plot.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1000 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/deadly_riposte.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1001 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/death_baron.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1008 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/deathmark.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1037 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/demolition_field.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1041 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/demonic_pact.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1050 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/desecration_demon.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1066 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/devout_decree.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1068 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/diamond_mare.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1072 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/dictate_of_kruphix.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1080 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/dimir_guildgate.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1087 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/diregraf_ghoul.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1093 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/disenchant.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1104 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/dive_down.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1109 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/divine_resilience.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1140 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/doubling_season.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1152 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/dragon_fodder.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1153 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/dragon_mage.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1155 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/dragon_trainer.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1171 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/drake_hatcher.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1172 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/drakuseth_maw_of_flames.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1174 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/dread_summons.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1177 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/dreadwing_scavenger.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1188 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/driver_of_the_dead.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1190 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/drogskol_reaver.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1191 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/dropkick_bomber.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1193 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/druid_of_the_cowl.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1195 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/dryad_militant.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1204 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/duress.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1218 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/dwynen_s_elite.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1219 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/dwynen_gilt_leaf_daen.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1227 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/eager_trufflesnout.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1248 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/eaten_by_piranhas.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1278 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/electroduplicate.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1284 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/elementalist_adept.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1285 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/elenda_saint_of_dusk.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1286 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/elfsworn_giant.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1302 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/elvish_regrower.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1323 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/empyrean_eagle.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1340 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/enigma_drake.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1364 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/erudite_wizard.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1377 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/etali_primal_storm.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1394 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/exclusion_mage.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1396 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/exemplar_of_light.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1402 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/expedition_map.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1411 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/exsanguinate.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1417 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/extravagant_replication.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1425 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/faebloom_trick.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1431 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/fake_your_own_death.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1438 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/fanatical_firebrand.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1484 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/feldon_s_cane.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1485 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/felidar_cub.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1486 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/felidar_retreat.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1487 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/felidar_savior.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1491 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/felling_blow.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1503 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/fiendish_panda.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1504 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/fierce_empath.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1505 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/fiery_annihilation.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1511 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/finale_of_revelation.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1515 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/fire_elemental.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1531 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/firebrand_archer.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1533 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/fireshrieker.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1534 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/firespitter_whelp.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1537 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/fishing_pole.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1546 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/flamewake_phoenix.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1550 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/flashfreeze.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1551 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/fleeting_distraction.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1553 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/fleeting_flight.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1574 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/fog_bank.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1621 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/frenzied_goblin.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1656 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/fynn_the_fangbearer.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1683 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/garna_bloodfist_of_keld.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1694 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/gate_colossus.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1695 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/gatekeeper_of_malakir.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1696 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/gateway_sneak.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1709 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/genesis_wave.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1728 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/ghalta_primal_hunger.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1731 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/ghitu_lavarunner.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1736 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/giada_font_of_hope.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1738 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/giant_cindermaw.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1770 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/gleaming_barrier.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1793 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/gnarlback_rhino.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1795 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/gnarlid_colony.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1801 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/goblin_boarders.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1802 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/goblin_firebomb.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1805 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/goblin_negotiation.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1806 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/goblin_oriflamme.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1808 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/goblin_smuggler.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1809 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/goblin_surprise.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1824 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/goldvein_pick.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1825 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/golgari_guildgate.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1833 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/good_fortune_unicorn.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1834 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/gorehorn_raider.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1848 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/grappling_kraken.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1851 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/gratuitous_violence.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1888 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/grow_from_the_ashes.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1895 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/gruul_guildgate.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1898 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/guarded_heir.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1913 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/gutless_plunderer.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1927 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/halana_and_alena_partners.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1939 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/harbinger_of_the_tides.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1947 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/hare_apparent.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1948 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/harmless_offering.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1975 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/healer_s_hawk.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1978 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/heartfire_immolator.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_1990 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/hedron_archive.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2002 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/helpful_hunter.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2006 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/herald_of_eternal_dawn.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2007 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/herald_of_faith.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2008 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/heraldic_banner.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2015 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/hero_s_downfall.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2016 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/heroes_bane.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2019 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/heroic_reinforcements.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2034 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/hidetsugu_s_second_rite.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2036 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/high_fae_trickster.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2041 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/high_society_hunter.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2046 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/hinterland_sanctifier.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2052 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/hoarding_dragon.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2062 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/homunculus_horde.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2101 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/hungry_ghoul.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2126 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/icewind_elemental.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2142 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/immersturm_predator.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2145 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/impact_tremors.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2147 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/imperious_perfect.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2156 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/incinerating_blast.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2160 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/infernal_vessel.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2161 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/infestation_sage.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2166 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/ingenious_leonin.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2185 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/inspiration_from_beyond.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2188 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/inspiring_overseer.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2189 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/inspiring_paladin.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2202 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/into_the_roil.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2217 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/involuntary_employment.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2245 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/izzet_guildgate.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2289 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/joraga_invocation.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2291 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/joust_through.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2295 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/juggernaut.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2307 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/kaito_cunning_infiltrator.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2309 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/kalastria_highborn.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2317 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/kargan_dragonrider.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2341 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/kellan_planar_trailblazer.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2357 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/kindled_fury.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2368 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/kiora_the_rising_tide.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2374 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/kitesail_corsair.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2385 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/knight_of_grace.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2386 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/knight_of_malice.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2393 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/koma_world_eater.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2416 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/krenko_mob_boss.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2426 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/kykar_zephyr_awakener.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2454 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/lathril_blade_of_the_elves.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2481 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/leyline_axe.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2500 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/lightshell_duo.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2508 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/liliana_dreadhorde_general.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2513 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/linden_the_steadfast_queen.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2549 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/loot_exuberant_explorer.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2580 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/luminous_rebuke.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2583 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/lunar_insight.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2591 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/lyra_dawnbringer.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2597 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/maalfeld_twins.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2603 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/macabre_waltz.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2628 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/magnigoth_sentry.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2634 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/make_a_stand.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2655 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/marauding_blight_priest.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2678 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/massacre_wurm.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2691 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/maze_s_end.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2692 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/mazemind_tome.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2743 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/micromancer.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2747 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/midnight_reaper.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2748 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/midnight_snack.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2757 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/mild_mannered_librarian.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2769 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/mindsparker.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2787 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/mischievous_mystic.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2788 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/mischievous_pup.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2806 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/mocking_sprite.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2809 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/mold_adder.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2856 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/mortify.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2858 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/mossborn_hydra.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2894 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/myojin_of_night_s_reach.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2898 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/mystic_archaeologist.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2900 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/mystical_teachings.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2913 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/needletooth_pack.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2919 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/nessian_hornbeetle.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2925 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/new_horizons.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2944 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/nine_lives_familiar.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2950 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/niv_mizzet_visionary.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2977 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/nullpriest_of_oblivion.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_2985 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/obliterating_bolt.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3020 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/omniscience.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3034 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/ordeal_of_nylea.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3045 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/orzhov_guildgate.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3076 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/overrun.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3080 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/ovika_enigma_goliath.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3091 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/painful_quandary.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3123 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/pelakka_wurm.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3128 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/perforating_artist.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3151 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/phyrexian_arena.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3162 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/pilfer.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3172 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/pirate_s_cutlass.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3217 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/prayer_of_binding.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3220 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/predator_ooze.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3223 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/preposterous_proportions.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3230 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/prideful_parent.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3231 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/primal_might.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3234 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/prime_speaker_zegana.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3235 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/primeval_bounty.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3254 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/progenitus.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3278 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/pulse_tracker.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3295 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/pyromancer_s_goggles.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3302 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/quakestrider_ceratops.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3313 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/quick_draw_katana.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3318 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/quilled_greatwurm.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3343 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/raging_redcap.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3349 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/raise_the_past.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3350 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/rakdos_guildgate.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3361 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/ramos_dragon_engine.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3398 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/ravenous_amulet.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3399 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/ravenous_giant.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3424 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/reclamation_sage.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3432 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/redcap_gutter_dweller.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3442 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/refute.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3445 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/regal_caracal.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3479 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/resolute_reinforcements.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3507 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/revenge_of_the_rats.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3538 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/rise_of_the_dark_realms.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3546 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/rite_of_the_dragoncaller.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3550 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/river_s_rebuke.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3609 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/ruby_daring_tracker.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3622 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/rune_scarred_demon.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3623 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/rune_sealed_wall.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3684 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/sanguine_indulgence.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3686 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/sanguine_syphoner.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3694 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/savage_ventmaw.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3695 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/savannah_lions.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3719 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/scavenging_ooze.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3725 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/scorching_dragonfire.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3737 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/scrawling_crawler.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3748 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/searslicer_goblin.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3769 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/seeker_s_folly.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3772 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/seismic_rupture.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3778 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/selesnya_guildgate.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3780 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/self_reflection.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3797 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/serra_angel.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3835 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/shipwreck_dowser.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3838 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/shivan_dragon.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3876 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/simic_guildgate.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3886 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/sire_of_seven_deaths.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3890 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/skeleton_archer.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3902 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/skyknight_squire.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3903 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/skyraker_giant.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3906 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/skyship_buccaneer.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3912 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/slagstorm.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3929 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/slumbering_cerberus.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3970 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/sorcerous_spyglass.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3976 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/soul_shackled_zombie.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3980 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/soulstone_sanctuary.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_3988 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/sower_of_chaos.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4001 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/spectral_sailor.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4014 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/sphinx_of_forgotten_lore.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4015 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/sphinx_of_the_final_word.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4050 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/spinner_of_souls.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4059 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/spitfire_lagac.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4073 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/springbloom_druid.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4077 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/squad_rallier.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4083 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/stab.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4105 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/starlight_snare.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4117 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/stasis_snare.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4168 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/storm_fleet_spy.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4188 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/strix_lookout.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4191 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/stromkirk_bloodthief.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4192 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/stromkirk_noble.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4193 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/strongbox_raider.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4231 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/sun_blessed_healer.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4266 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/sure_strike.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4272 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/surrak_the_hunt_caller.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4279 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/suspicious_shambler.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4283 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/swab_goblin.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4291 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/swiftblade_vindicator.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4300 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/sylvan_scavenging.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4307 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/syr_alin_the_lion_s_claw.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4319 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/tajuru_pathwarden.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4345 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/tatyova_benthic_druid.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4348 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/teach_by_example.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4358 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/tempest_djinn.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4363 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/temple_of_deceit.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4367 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/temple_of_malice.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4399 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/terror_of_mount_velus.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4546 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/thornweald_archer.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4556 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/thousand_year_storm.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4561 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/thrashing_brontodon.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4568 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/three_tree_mascot.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4572 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/thrill_of_possibility.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4601 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/time_stop.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4607 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/tinybones_bauble_burglar.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4620 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/tolarian_terror.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4650 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/tragic_banshee.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4675 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/treetop_snarespinner.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4679 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/tribute_to_hunger.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4695 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/trygon_predator.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4718 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/twinblade_blessing.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4719 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/twinblade_paladin.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4721 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/twinflame_tyrant.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4757 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/uncharted_voyage.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4773 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/undying_malice.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4779 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/unflinching_courage.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4828 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/valkyrie_s_call.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4837 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/vampire_gourmand.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4838 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/vampire_interloper.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4840 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/vampire_nighthawk.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4841 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/vampire_soulcaller.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4843 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/vampiric_rites.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4844 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/vanguard_seraph.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4862 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/vengeful_bloodwitch.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4868 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/venom_connoisseur.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4881 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/viashino_pyromancer.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4892 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/vile_entomber.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4924 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/vivien_reid.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4925 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/vizier_of_the_menagerie.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4934 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/volley_veteran.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4935 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/voracious_greatshark.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4976 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/wardens_of_the_cycle.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_4989 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/wary_thespian.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_5062 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/wildborn_preserver.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_5067 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/wildheart_invoker.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_5070 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/wildwood_scourge.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_5074 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/wilt_leaf_liege.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_5091 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/wishclaw_talisman.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_5102 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/witness_protection.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_5144 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/youthful_valkyrie.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_5165 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/zimone_paradox_sculptor.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_5169 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/zombify.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
pub(super) mod card_5179 {
use crate::cards::*;
use crate::cards::InitialCounters as Counters;
use super::{CardRules, CardExtras, generic, one, object};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../src/card_definitions/fdn/zul_ashur_lich_lord.rs"));
const _: () = assert!(!matches!(RULES.extras.has_rules_abilities, Some(false)), "authored ability-presence fact differs from provenance");
}
}
#[rustfmt::skip]
pub static ENTRIES: [CatalogueEntry; 427] = [
CatalogueEntry { definition: Definition { info: CardInfo { name: "Llanowar Elves", oracle_id: Some("68954295-54e3-4303-a6bc-fc4547a4e3a3"), mana_cost: "{G}", type_line: "Creature — Elf Druid", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0008::RULES.keywords, deck_limit: authored::card_0008::RULES.deck_limit }, effect: authored::card_0008::RULES.checked_effect(), triggers: authored::card_0008::RULES.triggers, activated: authored::card_0008::RULES.activated, enters: authored::card_0008::RULES.enters, entry: authored::card_0008::RULES.entry, kind: 1, basic: false, mana_color: Some(4), colored: [0, 0, 0, 0, 1, 0], generic: 0 }, rules: &authored::card_0008::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Abyssal Harvester", oracle_id: Some("68cc5065-0514-4adf-885f-da554e1db00a"), mana_cost: "{1}{B}{B}", type_line: "Creature — Demon Warlock", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0043::RULES.keywords, deck_limit: authored::card_0043::RULES.deck_limit }, effect: authored::card_0043::RULES.checked_effect(), triggers: authored::card_0043::RULES.triggers, activated: authored::card_0043::RULES.activated, enters: authored::card_0043::RULES.enters, entry: authored::card_0043::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 2, 0, 0, 0], generic: 1 }, rules: &authored::card_0043::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Adamant Will", oracle_id: Some("467e22c3-6107-40ff-afc7-5960710c970b"), mana_cost: "{1}{W}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0054::RULES.keywords, deck_limit: authored::card_0054::RULES.deck_limit }, effect: authored::card_0054::RULES.checked_effect(), triggers: authored::card_0054::RULES.triggers, activated: authored::card_0054::RULES.activated, enters: authored::card_0054::RULES.enters, entry: authored::card_0054::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_0054::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Adaptive Automaton", oracle_id: Some("53c730c6-2f8c-4af8-b400-b9d573a71e60"), mana_cost: "{3}", type_line: "Artifact Creature — Construct", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0055::RULES.keywords, deck_limit: authored::card_0055::RULES.deck_limit }, effect: authored::card_0055::RULES.checked_effect(), triggers: authored::card_0055::RULES.triggers, activated: authored::card_0055::RULES.activated, enters: authored::card_0055::RULES.enters, entry: authored::card_0055::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_0055::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Adventuring Gear", oracle_id: Some("17042684-7af4-43a6-88c7-885032cfb27c"), mana_cost: "{1}", type_line: "Artifact — Equipment", oracle_keywords: &["Equip", "Landfall"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0064::RULES.keywords, deck_limit: authored::card_0064::RULES.deck_limit }, effect: authored::card_0064::RULES.checked_effect(), triggers: authored::card_0064::RULES.triggers, activated: authored::card_0064::RULES.activated, enters: authored::card_0064::RULES.enters, entry: authored::card_0064::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_0064::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Aegis Turtle", oracle_id: Some("f84c30e7-2ea8-43ff-9356-1f907558cfd9"), mana_cost: "{U}", type_line: "Creature — Turtle", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0067::RULES.keywords, deck_limit: authored::card_0067::RULES.deck_limit }, effect: authored::card_0067::RULES.checked_effect(), triggers: authored::card_0067::RULES.triggers, activated: authored::card_0067::RULES.activated, enters: authored::card_0067::RULES.enters, entry: authored::card_0067::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_0067::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Aetherize", oracle_id: Some("7c779721-cd1b-4696-9ae9-68ccc284ed2a"), mana_cost: "{3}{U}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0073::RULES.keywords, deck_limit: authored::card_0073::RULES.deck_limit }, effect: authored::card_0073::RULES.checked_effect(), triggers: authored::card_0073::RULES.triggers, activated: authored::card_0073::RULES.activated, enters: authored::card_0073::RULES.enters, entry: authored::card_0073::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_0073::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Affectionate Indrik", oracle_id: Some("88aa032d-dc11-4cae-81f3-ce66353963e0"), mana_cost: "{5}{G}", type_line: "Creature — Beast", oracle_keywords: &["Fight"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0076::RULES.keywords, deck_limit: authored::card_0076::RULES.deck_limit }, effect: authored::card_0076::RULES.checked_effect(), triggers: authored::card_0076::RULES.triggers, activated: authored::card_0076::RULES.activated, enters: authored::card_0076::RULES.enters, entry: authored::card_0076::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 5 }, rules: &authored::card_0076::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Aggressive Mammoth", oracle_id: Some("032b5fc4-5aca-41b6-9bf6-2c1ed0018968"), mana_cost: "{3}{G}{G}{G}", type_line: "Creature — Elephant", oracle_keywords: &["Trample"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0096::RULES.keywords, deck_limit: authored::card_0096::RULES.deck_limit }, effect: authored::card_0096::RULES.checked_effect(), triggers: authored::card_0096::RULES.triggers, activated: authored::card_0096::RULES.activated, enters: authored::card_0096::RULES.enters, entry: authored::card_0096::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 3, 0], generic: 3 }, rules: &authored::card_0096::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Ajani's Pridemate", oracle_id: Some("95e94dea-5ac0-4d6f-adec-ca147aee861f"), mana_cost: "{1}{W}", type_line: "Creature — Cat Soldier", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0114::RULES.keywords, deck_limit: authored::card_0114::RULES.deck_limit }, effect: authored::card_0114::RULES.checked_effect(), triggers: authored::card_0114::RULES.triggers, activated: authored::card_0114::RULES.activated, enters: authored::card_0114::RULES.enters, entry: authored::card_0114::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_0114::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Ajani, Caller of the Pride", oracle_id: Some("1d5ff280-7f4d-4801-aefd-9b6e6b1f2818"), mana_cost: "{1}{W}{W}", type_line: "Legendary Planeswalker — Ajani", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_0116::RULES.keywords, deck_limit: authored::card_0116::RULES.deck_limit }, effect: authored::card_0116::RULES.checked_effect(), triggers: authored::card_0116::RULES.triggers, activated: authored::card_0116::RULES.activated, enters: authored::card_0116::RULES.enters, entry: authored::card_0116::RULES.entry, kind: 64, basic: false, mana_color: None, colored: [2, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_0116::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Alesha, Who Laughs at Fate", oracle_id: Some("17ce2c17-b69b-4ebf-9c3c-c64e771af101"), mana_cost: "{1}{B}{R}", type_line: "Legendary Creature — Human Warrior", oracle_keywords: &["First strike", "Raid"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_0128::RULES.keywords, deck_limit: authored::card_0128::RULES.deck_limit }, effect: authored::card_0128::RULES.checked_effect(), triggers: authored::card_0128::RULES.triggers, activated: authored::card_0128::RULES.activated, enters: authored::card_0128::RULES.enters, entry: authored::card_0128::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 1, 0, 0], generic: 1 }, rules: &authored::card_0128::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Ambush Wolf", oracle_id: Some("4c48de92-e053-4af7-b936-d7ec0875e0d3"), mana_cost: "{2}{G}", type_line: "Creature — Wolf", oracle_keywords: &["Flash"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0148::RULES.keywords, deck_limit: authored::card_0148::RULES.deck_limit }, effect: authored::card_0148::RULES.checked_effect(), triggers: authored::card_0148::RULES.triggers, activated: authored::card_0148::RULES.activated, enters: authored::card_0148::RULES.enters, entry: authored::card_0148::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 2 }, rules: &authored::card_0148::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "An Offer You Can't Refuse", oracle_id: Some("234a734b-ba28-4f1b-9d01-3c3e7d516590"), mana_cost: "{U}", type_line: "Instant", oracle_keywords: &["Treasure"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0150::RULES.keywords, deck_limit: authored::card_0150::RULES.deck_limit }, effect: authored::card_0150::RULES.checked_effect(), triggers: authored::card_0150::RULES.triggers, activated: authored::card_0150::RULES.activated, enters: authored::card_0150::RULES.enters, entry: authored::card_0150::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_0150::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Ancestor Dragon", oracle_id: Some("60b03a44-a7bd-48fa-8c8f-1704aed02cd7"), mana_cost: "{4}{W}{W}", type_line: "Creature — Dragon", oracle_keywords: &["Flying"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0154::RULES.keywords, deck_limit: authored::card_0154::RULES.deck_limit }, effect: authored::card_0154::RULES.checked_effect(), triggers: authored::card_0154::RULES.triggers, activated: authored::card_0154::RULES.activated, enters: authored::card_0154::RULES.enters, entry: authored::card_0154::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [2, 0, 0, 0, 0, 0], generic: 4 }, rules: &authored::card_0154::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Angel of Finality", oracle_id: Some("48d14b5c-711a-4f8a-9de5-55415cb7a79a"), mana_cost: "{3}{W}", type_line: "Creature — Angel", oracle_keywords: &["Flying"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0162::RULES.keywords, deck_limit: authored::card_0162::RULES.deck_limit }, effect: authored::card_0162::RULES.checked_effect(), triggers: authored::card_0162::RULES.triggers, activated: authored::card_0162::RULES.activated, enters: authored::card_0162::RULES.enters, entry: authored::card_0162::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_0162::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Angel of Vitality", oracle_id: Some("635fe908-6f58-4c9e-ac55-0775a7c6f278"), mana_cost: "{2}{W}", type_line: "Creature — Angel", oracle_keywords: &["Flying"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0163::RULES.keywords, deck_limit: authored::card_0163::RULES.deck_limit }, effect: authored::card_0163::RULES.checked_effect(), triggers: authored::card_0163::RULES.triggers, activated: authored::card_0163::RULES.activated, enters: authored::card_0163::RULES.enters, entry: authored::card_0163::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_0163::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Anthem of Champions", oracle_id: Some("f1d8e9a6-1903-4be5-8609-1009a463f393"), mana_cost: "{G}{W}", type_line: "Enchantment", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0176::RULES.keywords, deck_limit: authored::card_0176::RULES.deck_limit }, effect: authored::card_0176::RULES.checked_effect(), triggers: authored::card_0176::RULES.triggers, activated: authored::card_0176::RULES.activated, enters: authored::card_0176::RULES.enters, entry: authored::card_0176::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [1, 0, 0, 0, 1, 0], generic: 0 }, rules: &authored::card_0176::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Apothecary Stomper", oracle_id: Some("6a381121-9134-489c-8c7e-44be1936b0e2"), mana_cost: "{4}{G}{G}", type_line: "Creature — Elephant", oracle_keywords: &["Vigilance"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0185::RULES.keywords, deck_limit: authored::card_0185::RULES.deck_limit }, effect: authored::card_0185::RULES.checked_effect(), triggers: authored::card_0185::RULES.triggers, activated: authored::card_0185::RULES.activated, enters: authored::card_0185::RULES.enters, entry: authored::card_0185::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 2, 0], generic: 4 }, rules: &authored::card_0185::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Arahbo, the First Fang", oracle_id: Some("dfec65a0-6fb6-463d-aa62-e84a427db217"), mana_cost: "{2}{W}", type_line: "Legendary Creature — Cat Avatar", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_0198::RULES.keywords, deck_limit: authored::card_0198::RULES.deck_limit }, effect: authored::card_0198::RULES.checked_effect(), triggers: authored::card_0198::RULES.triggers, activated: authored::card_0198::RULES.activated, enters: authored::card_0198::RULES.enters, entry: authored::card_0198::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_0198::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Arbiter of Woe", oracle_id: Some("540693f3-985c-4a4a-945c-c957c5aad395"), mana_cost: "{4}{B}{B}", type_line: "Creature — Demon", oracle_keywords: &["Flying"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0201::RULES.keywords, deck_limit: authored::card_0201::RULES.deck_limit }, effect: authored::card_0201::RULES.checked_effect(), triggers: authored::card_0201::RULES.triggers, activated: authored::card_0201::RULES.activated, enters: authored::card_0201::RULES.enters, entry: authored::card_0201::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 2, 0, 0, 0], generic: 4 }, rules: &authored::card_0201::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Arcane Epiphany", oracle_id: Some("fe1556cc-32d0-4e51-bb15-f3fce669a5e1"), mana_cost: "{3}{U}{U}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0204::RULES.keywords, deck_limit: authored::card_0204::RULES.deck_limit }, effect: authored::card_0204::RULES.checked_effect(), triggers: authored::card_0204::RULES.triggers, activated: authored::card_0204::RULES.activated, enters: authored::card_0204::RULES.enters, entry: authored::card_0204::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 2, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_0204::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Arcanis the Omnipotent", oracle_id: Some("8a7183cc-161c-444d-a889-a17519c8061b"), mana_cost: "{3}{U}{U}{U}", type_line: "Legendary Creature — Wizard", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_0206::RULES.keywords, deck_limit: authored::card_0206::RULES.deck_limit }, effect: authored::card_0206::RULES.checked_effect(), triggers: authored::card_0206::RULES.triggers, activated: authored::card_0206::RULES.activated, enters: authored::card_0206::RULES.enters, entry: authored::card_0206::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 3, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_0206::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Archmage of Runes", oracle_id: Some("28045b32-4c1a-40e5-a15d-524d0f8fe6ec"), mana_cost: "{3}{U}{U}", type_line: "Creature — Giant Wizard", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0213::RULES.keywords, deck_limit: authored::card_0213::RULES.deck_limit }, effect: authored::card_0213::RULES.checked_effect(), triggers: authored::card_0213::RULES.triggers, activated: authored::card_0213::RULES.activated, enters: authored::card_0213::RULES.enters, entry: authored::card_0213::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 2, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_0213::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Archway Angel", oracle_id: Some("f2cf087a-d89a-4145-9394-945ce41c8cca"), mana_cost: "{5}{W}", type_line: "Creature — Angel", oracle_keywords: &["Flying"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0217::RULES.keywords, deck_limit: authored::card_0217::RULES.deck_limit }, effect: authored::card_0217::RULES.checked_effect(), triggers: authored::card_0217::RULES.triggers, activated: authored::card_0217::RULES.activated, enters: authored::card_0217::RULES.enters, entry: authored::card_0217::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 5 }, rules: &authored::card_0217::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Armasaur Guide", oracle_id: Some("1d4e7422-2e26-49ce-beef-08f61583089a"), mana_cost: "{4}{W}", type_line: "Creature — Dinosaur", oracle_keywords: &["Vigilance"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0224::RULES.keywords, deck_limit: authored::card_0224::RULES.deck_limit }, effect: authored::card_0224::RULES.checked_effect(), triggers: authored::card_0224::RULES.triggers, activated: authored::card_0224::RULES.activated, enters: authored::card_0224::RULES.enters, entry: authored::card_0224::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 4 }, rules: &authored::card_0224::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Ashroot Animist", oracle_id: Some("3f008b57-6471-4785-9571-3b90193da7e2"), mana_cost: "{2}{R}{G}", type_line: "Creature — Lizard Druid", oracle_keywords: &["Trample"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0244::RULES.keywords, deck_limit: authored::card_0244::RULES.deck_limit }, effect: authored::card_0244::RULES.checked_effect(), triggers: authored::card_0244::RULES.triggers, activated: authored::card_0244::RULES.activated, enters: authored::card_0244::RULES.enters, entry: authored::card_0244::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 1, 0], generic: 2 }, rules: &authored::card_0244::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Aurelia, the Warleader", oracle_id: Some("0f5a3a09-2f07-4774-9e0f-e99d9a444166"), mana_cost: "{2}{R}{R}{W}{W}", type_line: "Legendary Creature — Angel", oracle_keywords: &["Flying", "Vigilance", "Haste"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_0267::RULES.keywords, deck_limit: authored::card_0267::RULES.deck_limit }, effect: authored::card_0267::RULES.checked_effect(), triggers: authored::card_0267::RULES.triggers, activated: authored::card_0267::RULES.activated, enters: authored::card_0267::RULES.enters, entry: authored::card_0267::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [2, 0, 0, 2, 0, 0], generic: 2 }, rules: &authored::card_0267::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Authority of the Consuls", oracle_id: Some("55f3c721-e13a-406e-bc8e-d6cdc91ac477"), mana_cost: "{W}", type_line: "Enchantment", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0273::RULES.keywords, deck_limit: authored::card_0273::RULES.deck_limit }, effect: authored::card_0273::RULES.checked_effect(), triggers: authored::card_0273::RULES.triggers, activated: authored::card_0273::RULES.activated, enters: authored::card_0273::RULES.enters, entry: authored::card_0273::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_0273::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Axgard Cavalry", oracle_id: Some("6a92b011-fd01-4b6a-a17c-73ac1967ff98"), mana_cost: "{1}{R}", type_line: "Creature — Dwarf Berserker", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0291::RULES.keywords, deck_limit: authored::card_0291::RULES.deck_limit }, effect: authored::card_0291::RULES.checked_effect(), triggers: authored::card_0291::RULES.triggers, activated: authored::card_0291::RULES.activated, enters: authored::card_0291::RULES.enters, entry: authored::card_0291::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 1 }, rules: &authored::card_0291::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Ayli, Eternal Pilgrim", oracle_id: Some("e6a35729-edee-4a8c-9aa1-041114d66c69"), mana_cost: "{W}{B}", type_line: "Legendary Creature — Kor Cleric", oracle_keywords: &["Deathtouch"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_0292::RULES.keywords, deck_limit: authored::card_0292::RULES.deck_limit }, effect: authored::card_0292::RULES.checked_effect(), triggers: authored::card_0292::RULES.triggers, activated: authored::card_0292::RULES.activated, enters: authored::card_0292::RULES.enters, entry: authored::card_0292::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 1, 0, 0, 0], generic: 0 }, rules: &authored::card_0292::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Azorius Guildgate", oracle_id: Some("ad1712d8-809f-410c-8b91-ffe6fb8a69a1"), mana_cost: "", type_line: "Land — Gate", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0295::RULES.keywords, deck_limit: authored::card_0295::RULES.deck_limit }, effect: authored::card_0295::RULES.checked_effect(), triggers: authored::card_0295::RULES.triggers, activated: authored::card_0295::RULES.activated, enters: authored::card_0295::RULES.enters, entry: authored::card_0295::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_0295::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Bake into a Pie", oracle_id: Some("1b9ec782-0ba1-41f1-bc39-d3302494ecb3"), mana_cost: "{2}{B}{B}", type_line: "Instant", oracle_keywords: &["Food"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0307::RULES.keywords, deck_limit: authored::card_0307::RULES.deck_limit }, effect: authored::card_0307::RULES.checked_effect(), triggers: authored::card_0307::RULES.triggers, activated: authored::card_0307::RULES.activated, enters: authored::card_0307::RULES.enters, entry: authored::card_0307::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 2, 0, 0, 0], generic: 2 }, rules: &authored::card_0307::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Ball Lightning", oracle_id: Some("7485cf25-eb41-4397-be13-7f0b8c10c70a"), mana_cost: "{R}{R}{R}", type_line: "Creature — Elemental", oracle_keywords: &["Haste", "Trample"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0313::RULES.keywords, deck_limit: authored::card_0313::RULES.deck_limit }, effect: authored::card_0313::RULES.checked_effect(), triggers: authored::card_0313::RULES.triggers, activated: authored::card_0313::RULES.activated, enters: authored::card_0313::RULES.enters, entry: authored::card_0313::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 3, 0, 0], generic: 0 }, rules: &authored::card_0313::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Ballyrush Banneret", oracle_id: Some("0ad27d0e-b6c3-4c3f-a389-285d889b6a65"), mana_cost: "{1}{W}", type_line: "Creature — Kithkin Soldier", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0314::RULES.keywords, deck_limit: authored::card_0314::RULES.deck_limit }, effect: authored::card_0314::RULES.checked_effect(), triggers: authored::card_0314::RULES.triggers, activated: authored::card_0314::RULES.activated, enters: authored::card_0314::RULES.enters, entry: authored::card_0314::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_0314::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Balmor, Battlemage Captain", oracle_id: Some("7da19fc5-cb85-4ec7-beea-95210378bbfe"), mana_cost: "{U}{R}", type_line: "Legendary Creature — Bird Wizard", oracle_keywords: &["Flying"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_0315::RULES.keywords, deck_limit: authored::card_0315::RULES.deck_limit }, effect: authored::card_0315::RULES.checked_effect(), triggers: authored::card_0315::RULES.triggers, activated: authored::card_0315::RULES.activated, enters: authored::card_0315::RULES.enters, entry: authored::card_0315::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 1, 0, 0], generic: 0 }, rules: &authored::card_0315::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Banner of Kinship", oracle_id: Some("8c220dbd-6572-4715-aae6-dd09a4252d68"), mana_cost: "{5}", type_line: "Artifact", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0322::RULES.keywords, deck_limit: authored::card_0322::RULES.deck_limit }, effect: authored::card_0322::RULES.checked_effect(), triggers: authored::card_0322::RULES.triggers, activated: authored::card_0322::RULES.activated, enters: authored::card_0322::RULES.enters, entry: authored::card_0322::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 5 }, rules: &authored::card_0322::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Basilisk Collar", oracle_id: Some("f5f4dd28-f4ae-4d39-b9b8-6ebfd63c93fe"), mana_cost: "{1}", type_line: "Artifact — Equipment", oracle_keywords: &["Equip"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0345::RULES.keywords, deck_limit: authored::card_0345::RULES.deck_limit }, effect: authored::card_0345::RULES.checked_effect(), triggers: authored::card_0345::RULES.triggers, activated: authored::card_0345::RULES.activated, enters: authored::card_0345::RULES.enters, entry: authored::card_0345::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_0345::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Battle-Rattle Shaman", oracle_id: Some("72dc807b-35fb-4d69-ae4c-04eed0646170"), mana_cost: "{3}{R}", type_line: "Creature — Goblin Shaman", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0350::RULES.keywords, deck_limit: authored::card_0350::RULES.deck_limit }, effect: authored::card_0350::RULES.checked_effect(), triggers: authored::card_0350::RULES.triggers, activated: authored::card_0350::RULES.activated, enters: authored::card_0350::RULES.enters, entry: authored::card_0350::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 3 }, rules: &authored::card_0350::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Battlesong Berserker", oracle_id: Some("396cc8ce-9efa-4fc7-90a6-b439b2b021db"), mana_cost: "{3}{R}", type_line: "Creature — Human Berserker", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0351::RULES.keywords, deck_limit: authored::card_0351::RULES.deck_limit }, effect: authored::card_0351::RULES.checked_effect(), triggers: authored::card_0351::RULES.triggers, activated: authored::card_0351::RULES.activated, enters: authored::card_0351::RULES.enters, entry: authored::card_0351::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 3 }, rules: &authored::card_0351::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Bear Cub", oracle_id: Some("ed206651-3d9b-4546-8d45-0682817192fd"), mana_cost: "{1}{G}", type_line: "Creature — Bear", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0357::RULES.keywords, deck_limit: authored::card_0357::RULES.deck_limit }, effect: authored::card_0357::RULES.checked_effect(), triggers: authored::card_0357::RULES.triggers, activated: authored::card_0357::RULES.activated, enters: authored::card_0357::RULES.enters, entry: authored::card_0357::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 1 }, rules: &authored::card_0357::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Beast-Kin Ranger", oracle_id: Some("db794af5-ba2e-4b10-b213-e481e8a50432"), mana_cost: "{2}{G}", type_line: "Creature — Elf Ranger", oracle_keywords: &["Trample"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0361::RULES.keywords, deck_limit: authored::card_0361::RULES.deck_limit }, effect: authored::card_0361::RULES.checked_effect(), triggers: authored::card_0361::RULES.triggers, activated: authored::card_0361::RULES.activated, enters: authored::card_0361::RULES.enters, entry: authored::card_0361::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 2 }, rules: &authored::card_0361::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Bigfin Bouncer", oracle_id: Some("4e473298-0b67-42b8-90a8-3bcdd73da473"), mana_cost: "{3}{U}", type_line: "Creature — Shark Pirate", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0406::RULES.keywords, deck_limit: authored::card_0406::RULES.deck_limit }, effect: authored::card_0406::RULES.checked_effect(), triggers: authored::card_0406::RULES.triggers, activated: authored::card_0406::RULES.activated, enters: authored::card_0406::RULES.enters, entry: authored::card_0406::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_0406::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Billowing Shriekmass", oracle_id: Some("ce45d6bd-74e3-401b-983d-028d3fb942da"), mana_cost: "{3}{B}", type_line: "Creature — Spirit", oracle_keywords: &["Flying", "Threshold", "Mill"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0413::RULES.keywords, deck_limit: authored::card_0413::RULES.deck_limit }, effect: authored::card_0413::RULES.checked_effect(), triggers: authored::card_0413::RULES.triggers, activated: authored::card_0413::RULES.activated, enters: authored::card_0413::RULES.enters, entry: authored::card_0413::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 3 }, rules: &authored::card_0413::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Biogenic Upgrade", oracle_id: Some("30812870-87c0-469c-95a5-811d5a181e72"), mana_cost: "{4}{G}{G}", type_line: "Sorcery", oracle_keywords: &["Double"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0416::RULES.keywords, deck_limit: authored::card_0416::RULES.deck_limit }, effect: authored::card_0416::RULES.checked_effect(), triggers: authored::card_0416::RULES.triggers, activated: authored::card_0416::RULES.activated, enters: authored::card_0416::RULES.enters, entry: authored::card_0416::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 0, 0, 2, 0], generic: 4 }, rules: &authored::card_0416::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Bishop's Soldier", oracle_id: Some("84487586-4f71-457a-b983-4f1f7a71deef"), mana_cost: "{1}{W}", type_line: "Creature — Vampire Soldier", oracle_keywords: &["Lifelink"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0422::RULES.keywords, deck_limit: authored::card_0422::RULES.deck_limit }, effect: authored::card_0422::RULES.checked_effect(), triggers: authored::card_0422::RULES.triggers, activated: authored::card_0422::RULES.activated, enters: authored::card_0422::RULES.enters, entry: authored::card_0422::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_0422::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Bite Down", oracle_id: Some("623903de-3c04-4745-9af3-d7ec9fb2574d"), mana_cost: "{1}{G}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0423::RULES.keywords, deck_limit: authored::card_0423::RULES.deck_limit }, effect: authored::card_0423::RULES.checked_effect(), triggers: authored::card_0423::RULES.triggers, activated: authored::card_0423::RULES.activated, enters: authored::card_0423::RULES.enters, entry: authored::card_0423::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 1 }, rules: &authored::card_0423::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Blanchwood Armor", oracle_id: Some("80ea56ad-e741-4a85-b4e8-ce62e7d593d5"), mana_cost: "{2}{G}", type_line: "Enchantment — Aura", oracle_keywords: &["Enchant"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0438::RULES.keywords, deck_limit: authored::card_0438::RULES.deck_limit }, effect: authored::card_0438::RULES.checked_effect(), triggers: authored::card_0438::RULES.triggers, activated: authored::card_0438::RULES.activated, enters: authored::card_0438::RULES.enters, entry: authored::card_0438::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 2 }, rules: &authored::card_0438::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Blasphemous Edict", oracle_id: Some("6cf50e21-0f60-4e9a-b910-ebe1bad2a29e"), mana_cost: "{3}{B}{B}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0439::RULES.keywords, deck_limit: authored::card_0439::RULES.deck_limit }, effect: authored::card_0439::RULES.checked_effect(), triggers: authored::card_0439::RULES.triggers, activated: authored::card_0439::RULES.activated, enters: authored::card_0439::RULES.enters, entry: authored::card_0439::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 2, 0, 0, 0], generic: 3 }, rules: &authored::card_0439::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Bloodthirsty Conqueror", oracle_id: Some("fd505c02-c59e-476d-8e88-35da862ddc23"), mana_cost: "{3}{B}{B}", type_line: "Creature — Vampire Knight", oracle_keywords: &["Deathtouch", "Flying"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0460::RULES.keywords, deck_limit: authored::card_0460::RULES.deck_limit }, effect: authored::card_0460::RULES.checked_effect(), triggers: authored::card_0460::RULES.triggers, activated: authored::card_0460::RULES.activated, enters: authored::card_0460::RULES.enters, entry: authored::card_0460::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 2, 0, 0, 0], generic: 3 }, rules: &authored::card_0460::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Bloodtithe Collector", oracle_id: Some("93088b0c-d158-473a-9c06-aa2f344a9868"), mana_cost: "{4}{B}", type_line: "Creature — Vampire Noble", oracle_keywords: &["Flying"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0462::RULES.keywords, deck_limit: authored::card_0462::RULES.deck_limit }, effect: authored::card_0462::RULES.checked_effect(), triggers: authored::card_0462::RULES.triggers, activated: authored::card_0462::RULES.activated, enters: authored::card_0462::RULES.enters, entry: authored::card_0462::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 4 }, rules: &authored::card_0462::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Boltwave", oracle_id: Some("bb8cccc8-e56f-42fa-a0d7-fc889b5e1c7b"), mana_cost: "{R}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0490::RULES.keywords, deck_limit: authored::card_0490::RULES.deck_limit }, effect: authored::card_0490::RULES.checked_effect(), triggers: authored::card_0490::RULES.triggers, activated: authored::card_0490::RULES.activated, enters: authored::card_0490::RULES.enters, entry: authored::card_0490::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 0 }, rules: &authored::card_0490::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Boros Charm", oracle_id: Some("2679d0dd-ba30-4a1c-b6a0-b3ac6c790496"), mana_cost: "{R}{W}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0505::RULES.keywords, deck_limit: authored::card_0505::RULES.deck_limit }, effect: authored::card_0505::RULES.checked_effect(), triggers: authored::card_0505::RULES.triggers, activated: authored::card_0505::RULES.activated, enters: authored::card_0505::RULES.enters, entry: authored::card_0505::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [1, 0, 0, 1, 0, 0], generic: 0 }, rules: &authored::card_0505::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Boros Guildgate", oracle_id: Some("73c423b7-cab8-4e69-8070-9edbf96a6c2c"), mana_cost: "", type_line: "Land — Gate", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0506::RULES.keywords, deck_limit: authored::card_0506::RULES.deck_limit }, effect: authored::card_0506::RULES.checked_effect(), triggers: authored::card_0506::RULES.triggers, activated: authored::card_0506::RULES.activated, enters: authored::card_0506::RULES.enters, entry: authored::card_0506::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_0506::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Brass's Bounty", oracle_id: Some("63b0538e-aa94-4e23-adf1-704cbf5bc3e5"), mana_cost: "{6}{R}", type_line: "Sorcery", oracle_keywords: &["Treasure"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0527::RULES.keywords, deck_limit: authored::card_0527::RULES.deck_limit }, effect: authored::card_0527::RULES.checked_effect(), triggers: authored::card_0527::RULES.triggers, activated: authored::card_0527::RULES.activated, enters: authored::card_0527::RULES.enters, entry: authored::card_0527::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 6 }, rules: &authored::card_0527::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Brazen Scourge", oracle_id: Some("422ef4ec-863a-450e-9577-a0707bc16532"), mana_cost: "{1}{R}{R}", type_line: "Creature — Gremlin", oracle_keywords: &["Haste"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0534::RULES.keywords, deck_limit: authored::card_0534::RULES.deck_limit }, effect: authored::card_0534::RULES.checked_effect(), triggers: authored::card_0534::RULES.triggers, activated: authored::card_0534::RULES.activated, enters: authored::card_0534::RULES.enters, entry: authored::card_0534::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 2, 0, 0], generic: 1 }, rules: &authored::card_0534::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Brineborn Cutthroat", oracle_id: Some("916cb70f-3b06-48ed-972d-75f805aa0892"), mana_cost: "{1}{U}", type_line: "Creature — Merfolk Pirate", oracle_keywords: &["Flash"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0555::RULES.keywords, deck_limit: authored::card_0555::RULES.deck_limit }, effect: authored::card_0555::RULES.checked_effect(), triggers: authored::card_0555::RULES.triggers, activated: authored::card_0555::RULES.activated, enters: authored::card_0555::RULES.enters, entry: authored::card_0555::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_0555::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Bulk Up", oracle_id: Some("fade8af0-5fdb-4237-9dd6-48bfd1d62767"), mana_cost: "{1}{R}", type_line: "Instant", oracle_keywords: &["Flashback", "Double"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0578::RULES.keywords, deck_limit: authored::card_0578::RULES.deck_limit }, effect: authored::card_0578::RULES.checked_effect(), triggers: authored::card_0578::RULES.triggers, activated: authored::card_0578::RULES.activated, enters: authored::card_0578::RULES.enters, entry: authored::card_0578::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 1 }, rules: &authored::card_0578::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Burglar Rat", oracle_id: Some("2f807301-37df-4724-871a-08e3512b07b3"), mana_cost: "{1}{B}", type_line: "Creature — Rat", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0587::RULES.keywords, deck_limit: authored::card_0587::RULES.deck_limit }, effect: authored::card_0587::RULES.checked_effect(), triggers: authored::card_0587::RULES.triggers, activated: authored::card_0587::RULES.activated, enters: authored::card_0587::RULES.enters, entry: authored::card_0587::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 1 }, rules: &authored::card_0587::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Burnished Hart", oracle_id: Some("893fed41-c144-433f-af88-bc7d419b7fb3"), mana_cost: "{3}", type_line: "Artifact Creature — Elk", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0594::RULES.keywords, deck_limit: authored::card_0594::RULES.deck_limit }, effect: authored::card_0594::RULES.checked_effect(), triggers: authored::card_0594::RULES.triggers, activated: authored::card_0594::RULES.activated, enters: authored::card_0594::RULES.enters, entry: authored::card_0594::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_0594::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Burrog Befuddler", oracle_id: Some("73b60c2c-7c74-4554-8861-daa0d6fc22b4"), mana_cost: "{1}{U}", type_line: "Creature — Frog Wizard", oracle_keywords: &["Flash"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0598::RULES.keywords, deck_limit: authored::card_0598::RULES.deck_limit }, effect: authored::card_0598::RULES.checked_effect(), triggers: authored::card_0598::RULES.triggers, activated: authored::card_0598::RULES.activated, enters: authored::card_0598::RULES.enters, entry: authored::card_0598::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_0598::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Burst Lightning", oracle_id: Some("ac2086fe-98ee-4280-9c7c-c5c2d6548a8b"), mana_cost: "{R}", type_line: "Instant", oracle_keywords: &["Kicker"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0600::RULES.keywords, deck_limit: authored::card_0600::RULES.deck_limit }, effect: authored::card_0600::RULES.checked_effect(), triggers: authored::card_0600::RULES.triggers, activated: authored::card_0600::RULES.activated, enters: authored::card_0600::RULES.enters, entry: authored::card_0600::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 0 }, rules: &authored::card_0600::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Bushwhack", oracle_id: Some("c61374e5-a7f6-455e-a40b-a481751b536b"), mana_cost: "{G}", type_line: "Sorcery", oracle_keywords: &["Fight"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0601::RULES.keywords, deck_limit: authored::card_0601::RULES.deck_limit }, effect: authored::card_0601::RULES.checked_effect(), triggers: authored::card_0601::RULES.triggers, activated: authored::card_0601::RULES.activated, enters: authored::card_0601::RULES.enters, entry: authored::card_0601::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 0 }, rules: &authored::card_0601::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Cackling Prowler", oracle_id: Some("898da62e-d29a-4ddc-be69-2ddec54614f5"), mana_cost: "{3}{G}", type_line: "Creature — Hyena Rogue", oracle_keywords: &["Ward", "Morbid"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0610::RULES.keywords, deck_limit: authored::card_0610::RULES.deck_limit }, effect: authored::card_0610::RULES.checked_effect(), triggers: authored::card_0610::RULES.triggers, activated: authored::card_0610::RULES.activated, enters: authored::card_0610::RULES.enters, entry: authored::card_0610::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 3 }, rules: &authored::card_0610::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Campus Guide", oracle_id: Some("5b8e0913-47e9-4d56-971d-abd91b0f7587"), mana_cost: "{2}", type_line: "Artifact Creature — Golem", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0629::RULES.keywords, deck_limit: authored::card_0629::RULES.deck_limit }, effect: authored::card_0629::RULES.checked_effect(), triggers: authored::card_0629::RULES.triggers, activated: authored::card_0629::RULES.activated, enters: authored::card_0629::RULES.enters, entry: authored::card_0629::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_0629::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Cancel", oracle_id: Some("7d00fb28-ea6c-49a9-b4af-ffb38860a9a7"), mana_cost: "{1}{U}{U}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0630::RULES.keywords, deck_limit: authored::card_0630::RULES.deck_limit }, effect: authored::card_0630::RULES.checked_effect(), triggers: authored::card_0630::RULES.triggers, activated: authored::card_0630::RULES.activated, enters: authored::card_0630::RULES.enters, entry: authored::card_0630::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 2, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_0630::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Carnelian Orb of Dragonkind", oracle_id: Some("651c967c-8f71-4eb5-b22f-545e55ea050e"), mana_cost: "{2}{R}", type_line: "Artifact", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0655::RULES.keywords, deck_limit: authored::card_0655::RULES.deck_limit }, effect: authored::card_0655::RULES.checked_effect(), triggers: authored::card_0655::RULES.triggers, activated: authored::card_0655::RULES.activated, enters: authored::card_0655::RULES.enters, entry: authored::card_0655::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 2 }, rules: &authored::card_0655::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Cat Collector", oracle_id: Some("52f1b5a4-62ff-44de-80da-503d4e52d312"), mana_cost: "{2}{W}", type_line: "Creature — Human Citizen", oracle_keywords: &["Food"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0677::RULES.keywords, deck_limit: authored::card_0677::RULES.deck_limit }, effect: authored::card_0677::RULES.checked_effect(), triggers: authored::card_0677::RULES.triggers, activated: authored::card_0677::RULES.activated, enters: authored::card_0677::RULES.enters, entry: authored::card_0677::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_0677::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Celestial Armor", oracle_id: Some("4f1b68d7-7d16-4bb1-af5f-f1708b23ca73"), mana_cost: "{2}{W}", type_line: "Artifact — Equipment", oracle_keywords: &["Equip", "Flash"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0695::RULES.keywords, deck_limit: authored::card_0695::RULES.deck_limit }, effect: authored::card_0695::RULES.checked_effect(), triggers: authored::card_0695::RULES.triggers, activated: authored::card_0695::RULES.activated, enters: authored::card_0695::RULES.enters, entry: authored::card_0695::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_0695::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Cemetery Recruitment", oracle_id: Some("07792d72-d5db-41b9-871c-e720e12ce659"), mana_cost: "{1}{B}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0697::RULES.keywords, deck_limit: authored::card_0697::RULES.deck_limit }, effect: authored::card_0697::RULES.checked_effect(), triggers: authored::card_0697::RULES.triggers, activated: authored::card_0697::RULES.activated, enters: authored::card_0697::RULES.enters, entry: authored::card_0697::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 1 }, rules: &authored::card_0697::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Cephalid Inkmage", oracle_id: Some("8d979a0b-ba04-4abb-8f78-c707ce15351a"), mana_cost: "{2}{U}", type_line: "Creature — Octopus Wizard", oracle_keywords: &["Surveil", "Threshold"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0700::RULES.keywords, deck_limit: authored::card_0700::RULES.deck_limit }, effect: authored::card_0700::RULES.checked_effect(), triggers: authored::card_0700::RULES.triggers, activated: authored::card_0700::RULES.activated, enters: authored::card_0700::RULES.enters, entry: authored::card_0700::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_0700::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Chandra, Flameshaper", oracle_id: Some("818f908c-9af6-4edc-bb0e-193bc830e2e4"), mana_cost: "{5}{R}{R}", type_line: "Legendary Planeswalker — Chandra", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_0715::RULES.keywords, deck_limit: authored::card_0715::RULES.deck_limit }, effect: authored::card_0715::RULES.checked_effect(), triggers: authored::card_0715::RULES.triggers, activated: authored::card_0715::RULES.activated, enters: authored::card_0715::RULES.enters, entry: authored::card_0715::RULES.entry, kind: 64, basic: false, mana_color: None, colored: [0, 0, 0, 2, 0, 0], generic: 5 }, rules: &authored::card_0715::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Charming Prince", oracle_id: Some("c48d844c-3976-4fa5-8e0d-3f0e535e7619"), mana_cost: "{1}{W}", type_line: "Creature — Human Noble", oracle_keywords: &["Scry"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0726::RULES.keywords, deck_limit: authored::card_0726::RULES.deck_limit }, effect: authored::card_0726::RULES.checked_effect(), triggers: authored::card_0726::RULES.triggers, activated: authored::card_0726::RULES.activated, enters: authored::card_0726::RULES.enters, entry: authored::card_0726::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_0726::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Chart a Course", oracle_id: Some("05878e49-93ad-4144-9c50-a0bb86126c2e"), mana_cost: "{1}{U}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0729::RULES.keywords, deck_limit: authored::card_0729::RULES.deck_limit }, effect: authored::card_0729::RULES.checked_effect(), triggers: authored::card_0729::RULES.triggers, activated: authored::card_0729::RULES.activated, enters: authored::card_0729::RULES.enters, entry: authored::card_0729::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_0729::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Circuitous Route", oracle_id: Some("30afacd9-4680-4aac-8c22-584f9418822d"), mana_cost: "{3}{G}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0755::RULES.keywords, deck_limit: authored::card_0755::RULES.deck_limit }, effect: authored::card_0755::RULES.checked_effect(), triggers: authored::card_0755::RULES.triggers, activated: authored::card_0755::RULES.activated, enters: authored::card_0755::RULES.enters, entry: authored::card_0755::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 3 }, rules: &authored::card_0755::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Claws Out", oracle_id: Some("499d76c1-97ff-4006-922c-be1684a7754c"), mana_cost: "{3}{W}{W}", type_line: "Instant", oracle_keywords: &["Affinity"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0766::RULES.keywords, deck_limit: authored::card_0766::RULES.deck_limit }, effect: authored::card_0766::RULES.checked_effect(), triggers: authored::card_0766::RULES.triggers, activated: authored::card_0766::RULES.activated, enters: authored::card_0766::RULES.enters, entry: authored::card_0766::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [2, 0, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_0766::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Clinquant Skymage", oracle_id: Some("d9a3207a-a330-4517-967e-f37c9a9f1355"), mana_cost: "{3}{U}", type_line: "Creature — Bird Wizard", oracle_keywords: &["Flying"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0770::RULES.keywords, deck_limit: authored::card_0770::RULES.deck_limit }, effect: authored::card_0770::RULES.checked_effect(), triggers: authored::card_0770::RULES.triggers, activated: authored::card_0770::RULES.activated, enters: authored::card_0770::RULES.enters, entry: authored::card_0770::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_0770::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Cloudblazer", oracle_id: Some("f84d1291-1f82-4b67-a26e-b79624b4ce1d"), mana_cost: "{3}{W}{U}", type_line: "Creature — Human Scout", oracle_keywords: &["Flying"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0779::RULES.keywords, deck_limit: authored::card_0779::RULES.deck_limit }, effect: authored::card_0779::RULES.checked_effect(), triggers: authored::card_0779::RULES.triggers, activated: authored::card_0779::RULES.activated, enters: authored::card_0779::RULES.enters, entry: authored::card_0779::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 1, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_0779::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Confiscate", oracle_id: Some("d599a38c-7719-443e-bb59-55bd43a8fee6"), mana_cost: "{4}{U}{U}", type_line: "Enchantment — Aura", oracle_keywords: &["Enchant"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0828::RULES.keywords, deck_limit: authored::card_0828::RULES.deck_limit }, effect: authored::card_0828::RULES.checked_effect(), triggers: authored::card_0828::RULES.triggers, activated: authored::card_0828::RULES.activated, enters: authored::card_0828::RULES.enters, entry: authored::card_0828::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 2, 0, 0, 0, 0], generic: 4 }, rules: &authored::card_0828::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Corsair Captain", oracle_id: Some("a7ec13c6-7ade-433a-b5a2-047854eef486"), mana_cost: "{2}{U}", type_line: "Creature — Human Pirate", oracle_keywords: &["Treasure"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0857::RULES.keywords, deck_limit: authored::card_0857::RULES.deck_limit }, effect: authored::card_0857::RULES.checked_effect(), triggers: authored::card_0857::RULES.triggers, activated: authored::card_0857::RULES.activated, enters: authored::card_0857::RULES.enters, entry: authored::card_0857::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_0857::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Courageous Goblin", oracle_id: Some("c107e271-5741-4a84-bfc4-3bc322972d0d"), mana_cost: "{1}{R}", type_line: "Creature — Goblin", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0871::RULES.keywords, deck_limit: authored::card_0871::RULES.deck_limit }, effect: authored::card_0871::RULES.checked_effect(), triggers: authored::card_0871::RULES.triggers, activated: authored::card_0871::RULES.activated, enters: authored::card_0871::RULES.enters, entry: authored::card_0871::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 1 }, rules: &authored::card_0871::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Crackling Cyclops", oracle_id: Some("16ab9451-f3da-4f61-8e2b-d9c84e0e5428"), mana_cost: "{2}{R}", type_line: "Creature — Cyclops Wizard", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0876::RULES.keywords, deck_limit: authored::card_0876::RULES.deck_limit }, effect: authored::card_0876::RULES.checked_effect(), triggers: authored::card_0876::RULES.triggers, activated: authored::card_0876::RULES.activated, enters: authored::card_0876::RULES.enters, entry: authored::card_0876::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 2 }, rules: &authored::card_0876::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Crash Through", oracle_id: Some("41c15160-ee8c-42e3-bc6c-593b8c8ae335"), mana_cost: "{R}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0878::RULES.keywords, deck_limit: authored::card_0878::RULES.deck_limit }, effect: authored::card_0878::RULES.checked_effect(), triggers: authored::card_0878::RULES.triggers, activated: authored::card_0878::RULES.activated, enters: authored::card_0878::RULES.enters, entry: authored::card_0878::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 0 }, rules: &authored::card_0878::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Crawling Barrens", oracle_id: Some("dfe1a112-97aa-4e81-8431-81552ba2cdcf"), mana_cost: "", type_line: "Land", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0883::RULES.keywords, deck_limit: authored::card_0883::RULES.deck_limit }, effect: authored::card_0883::RULES.checked_effect(), triggers: authored::card_0883::RULES.triggers, activated: authored::card_0883::RULES.activated, enters: authored::card_0883::RULES.enters, entry: authored::card_0883::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_0883::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Crow of Dark Tidings", oracle_id: Some("a4ffe297-5e82-43f9-91a4-7aa3d8dd3b4a"), mana_cost: "{2}{B}", type_line: "Creature — Zombie Bird", oracle_keywords: &["Flying", "Mill"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0896::RULES.keywords, deck_limit: authored::card_0896::RULES.deck_limit }, effect: authored::card_0896::RULES.checked_effect(), triggers: authored::card_0896::RULES.triggers, activated: authored::card_0896::RULES.activated, enters: authored::card_0896::RULES.enters, entry: authored::card_0896::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 2 }, rules: &authored::card_0896::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Crusader of Odric", oracle_id: Some("ea384b0d-3091-4d50-b15f-1f6763647b7c"), mana_cost: "{2}{W}", type_line: "Creature — Human Soldier", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0907::RULES.keywords, deck_limit: authored::card_0907::RULES.deck_limit }, effect: authored::card_0907::RULES.checked_effect(), triggers: authored::card_0907::RULES.triggers, activated: authored::card_0907::RULES.activated, enters: authored::card_0907::RULES.enters, entry: authored::card_0907::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_0907::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Crypt Feaster", oracle_id: Some("4c22257e-8a00-40b7-a29b-55fcec7ddbd4"), mana_cost: "{3}{B}", type_line: "Creature — Zombie", oracle_keywords: &["Threshold", "Menace"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0912::RULES.keywords, deck_limit: authored::card_0912::RULES.deck_limit }, effect: authored::card_0912::RULES.checked_effect(), triggers: authored::card_0912::RULES.triggers, activated: authored::card_0912::RULES.activated, enters: authored::card_0912::RULES.enters, entry: authored::card_0912::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 3 }, rules: &authored::card_0912::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Cryptic Caves", oracle_id: Some("b2eb7a64-a307-4a78-a25d-63fb3ae1e237"), mana_cost: "", type_line: "Land", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0915::RULES.keywords, deck_limit: authored::card_0915::RULES.deck_limit }, effect: authored::card_0915::RULES.checked_effect(), triggers: authored::card_0915::RULES.triggers, activated: authored::card_0915::RULES.activated, enters: authored::card_0915::RULES.enters, entry: authored::card_0915::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_0915::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Crystal Barricade", oracle_id: Some("291c7abf-9d64-4ba0-ae21-e3459edc9a95"), mana_cost: "{1}{W}", type_line: "Artifact Creature — Wall", oracle_keywords: &["Defender"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0918::RULES.keywords, deck_limit: authored::card_0918::RULES.deck_limit }, effect: authored::card_0918::RULES.checked_effect(), triggers: authored::card_0918::RULES.triggers, activated: authored::card_0918::RULES.activated, enters: authored::card_0918::RULES.enters, entry: authored::card_0918::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_0918::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Cultivator's Caravan", oracle_id: Some("c1eb530c-dd36-40ae-8617-6bb6969565e1"), mana_cost: "{3}", type_line: "Artifact — Vehicle", oracle_keywords: &["Crew"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0923::RULES.keywords, deck_limit: authored::card_0923::RULES.deck_limit }, effect: authored::card_0923::RULES.checked_effect(), triggers: authored::card_0923::RULES.triggers, activated: authored::card_0923::RULES.activated, enters: authored::card_0923::RULES.enters, entry: authored::card_0923::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_0923::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Curator of Destinies", oracle_id: Some("9938d178-0ce6-45c0-b317-fd5c54231579"), mana_cost: "{4}{U}{U}", type_line: "Creature — Sphinx", oracle_keywords: &["Flying"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0927::RULES.keywords, deck_limit: authored::card_0927::RULES.deck_limit }, effect: authored::card_0927::RULES.checked_effect(), triggers: authored::card_0927::RULES.triggers, activated: authored::card_0927::RULES.activated, enters: authored::card_0927::RULES.enters, entry: authored::card_0927::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 2, 0, 0, 0, 0], generic: 4 }, rules: &authored::card_0927::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Darksteel Colossus", oracle_id: Some("1b09d0cf-403c-4a15-aeee-602a1bdaf0c1"), mana_cost: "{11}", type_line: "Artifact Creature — Golem", oracle_keywords: &["Indestructible", "Trample"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0973::RULES.keywords, deck_limit: authored::card_0973::RULES.deck_limit }, effect: authored::card_0973::RULES.checked_effect(), triggers: authored::card_0973::RULES.triggers, activated: authored::card_0973::RULES.activated, enters: authored::card_0973::RULES.enters, entry: authored::card_0973::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 11 }, rules: &authored::card_0973::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Dauntless Veteran", oracle_id: Some("97b2fb8b-0c69-4ed2-b2d0-8a780f9b5b97"), mana_cost: "{1}{W}{W}", type_line: "Creature — Human Soldier", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0977::RULES.keywords, deck_limit: authored::card_0977::RULES.deck_limit }, effect: authored::card_0977::RULES.checked_effect(), triggers: authored::card_0977::RULES.triggers, activated: authored::card_0977::RULES.activated, enters: authored::card_0977::RULES.enters, entry: authored::card_0977::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [2, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_0977::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Dawnwing Marshal", oracle_id: Some("6640c234-8863-4670-a61f-ba6f59b60d69"), mana_cost: "{1}{W}", type_line: "Creature — Cat Soldier", oracle_keywords: &["Flying"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0985::RULES.keywords, deck_limit: authored::card_0985::RULES.deck_limit }, effect: authored::card_0985::RULES.checked_effect(), triggers: authored::card_0985::RULES.triggers, activated: authored::card_0985::RULES.activated, enters: authored::card_0985::RULES.enters, entry: authored::card_0985::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_0985::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Day of Judgment", oracle_id: Some("d057289d-5e28-43d5-8ff3-4a1bc723477d"), mana_cost: "{2}{W}{W}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0987::RULES.keywords, deck_limit: authored::card_0987::RULES.deck_limit }, effect: authored::card_0987::RULES.checked_effect(), triggers: authored::card_0987::RULES.triggers, activated: authored::card_0987::RULES.activated, enters: authored::card_0987::RULES.enters, entry: authored::card_0987::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [2, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_0987::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Dazzling Angel", oracle_id: Some("7de464e3-fae3-44cc-8233-776fc727c00a"), mana_cost: "{2}{W}", type_line: "Creature — Angel", oracle_keywords: &["Flying"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0989::RULES.keywords, deck_limit: authored::card_0989::RULES.deck_limit }, effect: authored::card_0989::RULES.checked_effect(), triggers: authored::card_0989::RULES.triggers, activated: authored::card_0989::RULES.activated, enters: authored::card_0989::RULES.enters, entry: authored::card_0989::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_0989::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Deadly Plot", oracle_id: Some("6178715c-870d-4710-b758-66e080804ee3"), mana_cost: "{3}{B}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_0998::RULES.keywords, deck_limit: authored::card_0998::RULES.deck_limit }, effect: authored::card_0998::RULES.checked_effect(), triggers: authored::card_0998::RULES.triggers, activated: authored::card_0998::RULES.activated, enters: authored::card_0998::RULES.enters, entry: authored::card_0998::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 3 }, rules: &authored::card_0998::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Deadly Riposte", oracle_id: Some("e341478e-c3f8-45d9-9430-abe3a58fa2fc"), mana_cost: "{1}{W}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1000::RULES.keywords, deck_limit: authored::card_1000::RULES.deck_limit }, effect: authored::card_1000::RULES.checked_effect(), triggers: authored::card_1000::RULES.triggers, activated: authored::card_1000::RULES.activated, enters: authored::card_1000::RULES.enters, entry: authored::card_1000::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_1000::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Death Baron", oracle_id: Some("99024aa8-5687-4d38-8a4b-feef42d6c1ff"), mana_cost: "{1}{B}{B}", type_line: "Creature — Zombie Wizard", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1001::RULES.keywords, deck_limit: authored::card_1001::RULES.deck_limit }, effect: authored::card_1001::RULES.checked_effect(), triggers: authored::card_1001::RULES.triggers, activated: authored::card_1001::RULES.activated, enters: authored::card_1001::RULES.enters, entry: authored::card_1001::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 2, 0, 0, 0], generic: 1 }, rules: &authored::card_1001::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Deathmark", oracle_id: Some("cf09b0af-3cf1-4486-8f65-8cfd2410314a"), mana_cost: "{B}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1008::RULES.keywords, deck_limit: authored::card_1008::RULES.deck_limit }, effect: authored::card_1008::RULES.checked_effect(), triggers: authored::card_1008::RULES.triggers, activated: authored::card_1008::RULES.activated, enters: authored::card_1008::RULES.enters, entry: authored::card_1008::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 0 }, rules: &authored::card_1008::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Demolition Field", oracle_id: Some("93953926-a644-49bb-9b5a-4c8f19114c7e"), mana_cost: "", type_line: "Land", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1037::RULES.keywords, deck_limit: authored::card_1037::RULES.deck_limit }, effect: authored::card_1037::RULES.checked_effect(), triggers: authored::card_1037::RULES.triggers, activated: authored::card_1037::RULES.activated, enters: authored::card_1037::RULES.enters, entry: authored::card_1037::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_1037::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Demonic Pact", oracle_id: Some("19a2f0a0-9e68-4982-a5f5-b77d805befd7"), mana_cost: "{2}{B}{B}", type_line: "Enchantment", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1041::RULES.keywords, deck_limit: authored::card_1041::RULES.deck_limit }, effect: authored::card_1041::RULES.checked_effect(), triggers: authored::card_1041::RULES.triggers, activated: authored::card_1041::RULES.activated, enters: authored::card_1041::RULES.enters, entry: authored::card_1041::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 0, 2, 0, 0, 0], generic: 2 }, rules: &authored::card_1041::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Desecration Demon", oracle_id: Some("ef75be00-1a88-47a1-a1ae-fe2c9881a798"), mana_cost: "{2}{B}{B}", type_line: "Creature — Demon", oracle_keywords: &["Flying"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1050::RULES.keywords, deck_limit: authored::card_1050::RULES.deck_limit }, effect: authored::card_1050::RULES.checked_effect(), triggers: authored::card_1050::RULES.triggers, activated: authored::card_1050::RULES.activated, enters: authored::card_1050::RULES.enters, entry: authored::card_1050::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 2, 0, 0, 0], generic: 2 }, rules: &authored::card_1050::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Devout Decree", oracle_id: Some("92f8c8b1-1fd5-4e80-b8a9-51671937aaf2"), mana_cost: "{1}{W}", type_line: "Sorcery", oracle_keywords: &["Scry"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1066::RULES.keywords, deck_limit: authored::card_1066::RULES.deck_limit }, effect: authored::card_1066::RULES.checked_effect(), triggers: authored::card_1066::RULES.triggers, activated: authored::card_1066::RULES.activated, enters: authored::card_1066::RULES.enters, entry: authored::card_1066::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_1066::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Diamond Mare", oracle_id: Some("9a440122-a015-4af2-b270-37f019884458"), mana_cost: "{2}", type_line: "Artifact Creature — Horse", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1068::RULES.keywords, deck_limit: authored::card_1068::RULES.deck_limit }, effect: authored::card_1068::RULES.checked_effect(), triggers: authored::card_1068::RULES.triggers, activated: authored::card_1068::RULES.activated, enters: authored::card_1068::RULES.enters, entry: authored::card_1068::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_1068::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Dictate of Kruphix", oracle_id: Some("be74a4c1-d569-4203-b28a-3e2ac6a82990"), mana_cost: "{1}{U}{U}", type_line: "Enchantment", oracle_keywords: &["Flash"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1072::RULES.keywords, deck_limit: authored::card_1072::RULES.deck_limit }, effect: authored::card_1072::RULES.checked_effect(), triggers: authored::card_1072::RULES.triggers, activated: authored::card_1072::RULES.activated, enters: authored::card_1072::RULES.enters, entry: authored::card_1072::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 2, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_1072::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Dimir Guildgate", oracle_id: Some("52d14717-0cbc-4d7e-b546-54ea91580338"), mana_cost: "", type_line: "Land — Gate", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1080::RULES.keywords, deck_limit: authored::card_1080::RULES.deck_limit }, effect: authored::card_1080::RULES.checked_effect(), triggers: authored::card_1080::RULES.triggers, activated: authored::card_1080::RULES.activated, enters: authored::card_1080::RULES.enters, entry: authored::card_1080::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_1080::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Diregraf Ghoul", oracle_id: Some("6048fc70-0dcc-4b54-977d-16e240225f82"), mana_cost: "{B}", type_line: "Creature — Zombie", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1087::RULES.keywords, deck_limit: authored::card_1087::RULES.deck_limit }, effect: authored::card_1087::RULES.checked_effect(), triggers: authored::card_1087::RULES.triggers, activated: authored::card_1087::RULES.activated, enters: authored::card_1087::RULES.enters, entry: authored::card_1087::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 0 }, rules: &authored::card_1087::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Disenchant", oracle_id: Some("a7e97fa9-4b72-4548-b854-5be5f18a6f1a"), mana_cost: "{1}{W}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1093::RULES.keywords, deck_limit: authored::card_1093::RULES.deck_limit }, effect: authored::card_1093::RULES.checked_effect(), triggers: authored::card_1093::RULES.triggers, activated: authored::card_1093::RULES.activated, enters: authored::card_1093::RULES.enters, entry: authored::card_1093::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_1093::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Dive Down", oracle_id: Some("2f990b54-fbf3-4949-85bb-9ba39710e72a"), mana_cost: "{U}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1104::RULES.keywords, deck_limit: authored::card_1104::RULES.deck_limit }, effect: authored::card_1104::RULES.checked_effect(), triggers: authored::card_1104::RULES.triggers, activated: authored::card_1104::RULES.activated, enters: authored::card_1104::RULES.enters, entry: authored::card_1104::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_1104::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Divine Resilience", oracle_id: Some("4f6e2e47-34df-4bf3-a546-e06b42840167"), mana_cost: "{W}", type_line: "Instant", oracle_keywords: &["Kicker"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1109::RULES.keywords, deck_limit: authored::card_1109::RULES.deck_limit }, effect: authored::card_1109::RULES.checked_effect(), triggers: authored::card_1109::RULES.triggers, activated: authored::card_1109::RULES.activated, enters: authored::card_1109::RULES.enters, entry: authored::card_1109::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_1109::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Doubling Season", oracle_id: Some("01546b7d-a233-4176-8843-d732074dc5b6"), mana_cost: "{4}{G}", type_line: "Enchantment", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1140::RULES.keywords, deck_limit: authored::card_1140::RULES.deck_limit }, effect: authored::card_1140::RULES.checked_effect(), triggers: authored::card_1140::RULES.triggers, activated: authored::card_1140::RULES.activated, enters: authored::card_1140::RULES.enters, entry: authored::card_1140::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 4 }, rules: &authored::card_1140::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Dragon Fodder", oracle_id: Some("d0d2c45b-b6e3-4999-bdab-976e8f0d6617"), mana_cost: "{1}{R}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1152::RULES.keywords, deck_limit: authored::card_1152::RULES.deck_limit }, effect: authored::card_1152::RULES.checked_effect(), triggers: authored::card_1152::RULES.triggers, activated: authored::card_1152::RULES.activated, enters: authored::card_1152::RULES.enters, entry: authored::card_1152::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 1 }, rules: &authored::card_1152::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Dragon Mage", oracle_id: Some("eab71e4e-27c3-4c41-b95f-259c1d14b97a"), mana_cost: "{5}{R}{R}", type_line: "Creature — Dragon Wizard", oracle_keywords: &["Flying"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1153::RULES.keywords, deck_limit: authored::card_1153::RULES.deck_limit }, effect: authored::card_1153::RULES.checked_effect(), triggers: authored::card_1153::RULES.triggers, activated: authored::card_1153::RULES.activated, enters: authored::card_1153::RULES.enters, entry: authored::card_1153::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 2, 0, 0], generic: 5 }, rules: &authored::card_1153::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Dragon Trainer", oracle_id: Some("328a0691-9453-4f2d-a9c3-5c69da252d20"), mana_cost: "{3}{R}{R}", type_line: "Creature — Human", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1155::RULES.keywords, deck_limit: authored::card_1155::RULES.deck_limit }, effect: authored::card_1155::RULES.checked_effect(), triggers: authored::card_1155::RULES.triggers, activated: authored::card_1155::RULES.activated, enters: authored::card_1155::RULES.enters, entry: authored::card_1155::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 2, 0, 0], generic: 3 }, rules: &authored::card_1155::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Drake Hatcher", oracle_id: Some("4de0ccab-bb5d-4c7c-827e-36aa1d00b182"), mana_cost: "{1}{U}", type_line: "Creature — Human Wizard", oracle_keywords: &["Prowess", "Vigilance"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1171::RULES.keywords, deck_limit: authored::card_1171::RULES.deck_limit }, effect: authored::card_1171::RULES.checked_effect(), triggers: authored::card_1171::RULES.triggers, activated: authored::card_1171::RULES.activated, enters: authored::card_1171::RULES.enters, entry: authored::card_1171::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_1171::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Drakuseth, Maw of Flames", oracle_id: Some("060deaff-44d6-4f03-9568-bcb7add80255"), mana_cost: "{4}{R}{R}{R}", type_line: "Legendary Creature — Dragon", oracle_keywords: &["Flying"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_1172::RULES.keywords, deck_limit: authored::card_1172::RULES.deck_limit }, effect: authored::card_1172::RULES.checked_effect(), triggers: authored::card_1172::RULES.triggers, activated: authored::card_1172::RULES.activated, enters: authored::card_1172::RULES.enters, entry: authored::card_1172::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 3, 0, 0], generic: 4 }, rules: &authored::card_1172::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Dread Summons", oracle_id: Some("642fa01d-025e-44dd-8360-5325e5a28282"), mana_cost: "{X}{B}{B}", type_line: "Sorcery", oracle_keywords: &["Mill"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1174::RULES.keywords, deck_limit: authored::card_1174::RULES.deck_limit }, effect: authored::card_1174::RULES.checked_effect(), triggers: authored::card_1174::RULES.triggers, activated: authored::card_1174::RULES.activated, enters: authored::card_1174::RULES.enters, entry: authored::card_1174::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 2, 0, 0, 0], generic: 0 }, rules: &authored::card_1174::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Dreadwing Scavenger", oracle_id: Some("bbfd0de0-3558-4833-bae4-8efae44b7fff"), mana_cost: "{1}{U}{B}", type_line: "Creature — Nightmare Bird", oracle_keywords: &["Flying", "Threshold"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1177::RULES.keywords, deck_limit: authored::card_1177::RULES.deck_limit }, effect: authored::card_1177::RULES.checked_effect(), triggers: authored::card_1177::RULES.triggers, activated: authored::card_1177::RULES.activated, enters: authored::card_1177::RULES.enters, entry: authored::card_1177::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 1, 0, 0, 0], generic: 1 }, rules: &authored::card_1177::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Driver of the Dead", oracle_id: Some("7c179e54-3beb-4761-bddf-41fa98b082db"), mana_cost: "{3}{B}", type_line: "Creature — Vampire", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1188::RULES.keywords, deck_limit: authored::card_1188::RULES.deck_limit }, effect: authored::card_1188::RULES.checked_effect(), triggers: authored::card_1188::RULES.triggers, activated: authored::card_1188::RULES.activated, enters: authored::card_1188::RULES.enters, entry: authored::card_1188::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 3 }, rules: &authored::card_1188::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Drogskol Reaver", oracle_id: Some("68175b49-f1e6-4b34-aad9-20d61a43d427"), mana_cost: "{5}{W}{U}", type_line: "Creature — Spirit", oracle_keywords: &["Flying", "Lifelink", "Double strike"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1190::RULES.keywords, deck_limit: authored::card_1190::RULES.deck_limit }, effect: authored::card_1190::RULES.checked_effect(), triggers: authored::card_1190::RULES.triggers, activated: authored::card_1190::RULES.activated, enters: authored::card_1190::RULES.enters, entry: authored::card_1190::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 1, 0, 0, 0, 0], generic: 5 }, rules: &authored::card_1190::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Dropkick Bomber", oracle_id: Some("c394d4b6-cfb8-4639-90cb-a322a25c0a2f"), mana_cost: "{2}{R}", type_line: "Creature — Goblin Warrior", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1191::RULES.keywords, deck_limit: authored::card_1191::RULES.deck_limit }, effect: authored::card_1191::RULES.checked_effect(), triggers: authored::card_1191::RULES.triggers, activated: authored::card_1191::RULES.activated, enters: authored::card_1191::RULES.enters, entry: authored::card_1191::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 2 }, rules: &authored::card_1191::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Druid of the Cowl", oracle_id: Some("b1793c3b-25d6-4fae-a99d-cfdd2210ca67"), mana_cost: "{1}{G}", type_line: "Creature — Elf Druid", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1193::RULES.keywords, deck_limit: authored::card_1193::RULES.deck_limit }, effect: authored::card_1193::RULES.checked_effect(), triggers: authored::card_1193::RULES.triggers, activated: authored::card_1193::RULES.activated, enters: authored::card_1193::RULES.enters, entry: authored::card_1193::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 1 }, rules: &authored::card_1193::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Dryad Militant", oracle_id: Some("b8ca5877-ac9e-4b15-8c23-c70f61b01895"), mana_cost: "{G/W}", type_line: "Creature — Dryad Soldier", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1195::RULES.keywords, deck_limit: authored::card_1195::RULES.deck_limit }, effect: authored::card_1195::RULES.checked_effect(), triggers: authored::card_1195::RULES.triggers, activated: authored::card_1195::RULES.activated, enters: authored::card_1195::RULES.enters, entry: authored::card_1195::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 0 }, rules: &authored::card_1195::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Duress", oracle_id: Some("33d405ea-7a9a-4970-b70f-9c05d90dd6f0"), mana_cost: "{B}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1204::RULES.keywords, deck_limit: authored::card_1204::RULES.deck_limit }, effect: authored::card_1204::RULES.checked_effect(), triggers: authored::card_1204::RULES.triggers, activated: authored::card_1204::RULES.activated, enters: authored::card_1204::RULES.enters, entry: authored::card_1204::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 0 }, rules: &authored::card_1204::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Dwynen's Elite", oracle_id: Some("3d4a7dd3-9258-4bd1-adc4-08f0205e196a"), mana_cost: "{1}{G}", type_line: "Creature — Elf Warrior", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1218::RULES.keywords, deck_limit: authored::card_1218::RULES.deck_limit }, effect: authored::card_1218::RULES.checked_effect(), triggers: authored::card_1218::RULES.triggers, activated: authored::card_1218::RULES.activated, enters: authored::card_1218::RULES.enters, entry: authored::card_1218::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 1 }, rules: &authored::card_1218::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Dwynen, Gilt-Leaf Daen", oracle_id: Some("30d0d75f-e94c-460b-b957-9f1d655c0f65"), mana_cost: "{2}{G}{G}", type_line: "Legendary Creature — Elf Warrior", oracle_keywords: &["Reach"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_1219::RULES.keywords, deck_limit: authored::card_1219::RULES.deck_limit }, effect: authored::card_1219::RULES.checked_effect(), triggers: authored::card_1219::RULES.triggers, activated: authored::card_1219::RULES.activated, enters: authored::card_1219::RULES.enters, entry: authored::card_1219::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 2, 0], generic: 2 }, rules: &authored::card_1219::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Eager Trufflesnout", oracle_id: Some("7f2ba562-8d5e-4915-967e-54916c3c5876"), mana_cost: "{2}{G}", type_line: "Creature — Boar", oracle_keywords: &["Food", "Trample"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1227::RULES.keywords, deck_limit: authored::card_1227::RULES.deck_limit }, effect: authored::card_1227::RULES.checked_effect(), triggers: authored::card_1227::RULES.triggers, activated: authored::card_1227::RULES.activated, enters: authored::card_1227::RULES.enters, entry: authored::card_1227::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 2 }, rules: &authored::card_1227::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Eaten by Piranhas", oracle_id: Some("c37eae65-1afe-4242-b677-24dd40e6f401"), mana_cost: "{1}{U}", type_line: "Enchantment — Aura", oracle_keywords: &["Enchant", "Flash"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1248::RULES.keywords, deck_limit: authored::card_1248::RULES.deck_limit }, effect: authored::card_1248::RULES.checked_effect(), triggers: authored::card_1248::RULES.triggers, activated: authored::card_1248::RULES.activated, enters: authored::card_1248::RULES.enters, entry: authored::card_1248::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_1248::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Electroduplicate", oracle_id: Some("112f2b3f-32e7-40b1-b80e-0b99184a840c"), mana_cost: "{2}{R}", type_line: "Sorcery", oracle_keywords: &["Flashback"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1278::RULES.keywords, deck_limit: authored::card_1278::RULES.deck_limit }, effect: authored::card_1278::RULES.checked_effect(), triggers: authored::card_1278::RULES.triggers, activated: authored::card_1278::RULES.activated, enters: authored::card_1278::RULES.enters, entry: authored::card_1278::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 2 }, rules: &authored::card_1278::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Elementalist Adept", oracle_id: Some("cf402270-ebc6-481b-b478-0e3acab7002a"), mana_cost: "{1}{U}", type_line: "Creature — Human Wizard", oracle_keywords: &["Prowess", "Flash"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1284::RULES.keywords, deck_limit: authored::card_1284::RULES.deck_limit }, effect: authored::card_1284::RULES.checked_effect(), triggers: authored::card_1284::RULES.triggers, activated: authored::card_1284::RULES.activated, enters: authored::card_1284::RULES.enters, entry: authored::card_1284::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_1284::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Elenda, Saint of Dusk", oracle_id: Some("bfd614d5-4c7f-416b-9cb2-bb167c0893a0"), mana_cost: "{2}{W}{B}", type_line: "Legendary Creature — Vampire Knight", oracle_keywords: &["Lifelink", "Hexproof from", "Hexproof"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_1285::RULES.keywords, deck_limit: authored::card_1285::RULES.deck_limit }, effect: authored::card_1285::RULES.checked_effect(), triggers: authored::card_1285::RULES.triggers, activated: authored::card_1285::RULES.activated, enters: authored::card_1285::RULES.enters, entry: authored::card_1285::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 1, 0, 0, 0], generic: 2 }, rules: &authored::card_1285::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Elfsworn Giant", oracle_id: Some("f6abdf34-cfdc-465c-b067-cdebb659406e"), mana_cost: "{3}{G}{G}", type_line: "Creature — Giant", oracle_keywords: &["Reach", "Landfall"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1286::RULES.keywords, deck_limit: authored::card_1286::RULES.deck_limit }, effect: authored::card_1286::RULES.checked_effect(), triggers: authored::card_1286::RULES.triggers, activated: authored::card_1286::RULES.activated, enters: authored::card_1286::RULES.enters, entry: authored::card_1286::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 2, 0], generic: 3 }, rules: &authored::card_1286::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Elvish Regrower", oracle_id: Some("adac526d-1a85-431a-a302-0a4b8f4a0c44"), mana_cost: "{2}{G}{G}", type_line: "Creature — Elf Druid", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1302::RULES.keywords, deck_limit: authored::card_1302::RULES.deck_limit }, effect: authored::card_1302::RULES.checked_effect(), triggers: authored::card_1302::RULES.triggers, activated: authored::card_1302::RULES.activated, enters: authored::card_1302::RULES.enters, entry: authored::card_1302::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 2, 0], generic: 2 }, rules: &authored::card_1302::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Empyrean Eagle", oracle_id: Some("270d14b2-07bc-46bc-918f-658102265ccf"), mana_cost: "{1}{W}{U}", type_line: "Creature — Bird Spirit", oracle_keywords: &["Flying"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1323::RULES.keywords, deck_limit: authored::card_1323::RULES.deck_limit }, effect: authored::card_1323::RULES.checked_effect(), triggers: authored::card_1323::RULES.triggers, activated: authored::card_1323::RULES.activated, enters: authored::card_1323::RULES.enters, entry: authored::card_1323::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 1, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_1323::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Enigma Drake", oracle_id: Some("62007675-8a0d-4211-83b2-0daf3641dedc"), mana_cost: "{1}{U}{R}", type_line: "Creature — Drake", oracle_keywords: &["Flying"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1340::RULES.keywords, deck_limit: authored::card_1340::RULES.deck_limit }, effect: authored::card_1340::RULES.checked_effect(), triggers: authored::card_1340::RULES.triggers, activated: authored::card_1340::RULES.activated, enters: authored::card_1340::RULES.enters, entry: authored::card_1340::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 1, 0, 0], generic: 1 }, rules: &authored::card_1340::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Erudite Wizard", oracle_id: Some("7d4b6613-9000-4f53-9737-fe0338f3f629"), mana_cost: "{2}{U}", type_line: "Creature — Human Wizard", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1364::RULES.keywords, deck_limit: authored::card_1364::RULES.deck_limit }, effect: authored::card_1364::RULES.checked_effect(), triggers: authored::card_1364::RULES.triggers, activated: authored::card_1364::RULES.activated, enters: authored::card_1364::RULES.enters, entry: authored::card_1364::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_1364::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Etali, Primal Storm", oracle_id: Some("078def07-ae5d-4591-8db6-d156834aab97"), mana_cost: "{4}{R}{R}", type_line: "Legendary Creature — Elder Dinosaur", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_1377::RULES.keywords, deck_limit: authored::card_1377::RULES.deck_limit }, effect: authored::card_1377::RULES.checked_effect(), triggers: authored::card_1377::RULES.triggers, activated: authored::card_1377::RULES.activated, enters: authored::card_1377::RULES.enters, entry: authored::card_1377::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 2, 0, 0], generic: 4 }, rules: &authored::card_1377::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Exclusion Mage", oracle_id: Some("fd28fe55-36cd-4242-8dd4-edb58fbb9895"), mana_cost: "{2}{U}", type_line: "Creature — Human Wizard", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1394::RULES.keywords, deck_limit: authored::card_1394::RULES.deck_limit }, effect: authored::card_1394::RULES.checked_effect(), triggers: authored::card_1394::RULES.triggers, activated: authored::card_1394::RULES.activated, enters: authored::card_1394::RULES.enters, entry: authored::card_1394::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_1394::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Exemplar of Light", oracle_id: Some("9ad730b5-8950-4215-9319-387e2970dd22"), mana_cost: "{2}{W}{W}", type_line: "Creature — Angel", oracle_keywords: &["Flying"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1396::RULES.keywords, deck_limit: authored::card_1396::RULES.deck_limit }, effect: authored::card_1396::RULES.checked_effect(), triggers: authored::card_1396::RULES.triggers, activated: authored::card_1396::RULES.activated, enters: authored::card_1396::RULES.enters, entry: authored::card_1396::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [2, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_1396::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Expedition Map", oracle_id: Some("8fcf50cd-e6d0-4516-850f-d42ee75dcc3a"), mana_cost: "{1}", type_line: "Artifact", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1402::RULES.keywords, deck_limit: authored::card_1402::RULES.deck_limit }, effect: authored::card_1402::RULES.checked_effect(), triggers: authored::card_1402::RULES.triggers, activated: authored::card_1402::RULES.activated, enters: authored::card_1402::RULES.enters, entry: authored::card_1402::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_1402::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Exsanguinate", oracle_id: Some("8164b1e8-3350-465e-8a17-75f57d326344"), mana_cost: "{X}{B}{B}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1411::RULES.keywords, deck_limit: authored::card_1411::RULES.deck_limit }, effect: authored::card_1411::RULES.checked_effect(), triggers: authored::card_1411::RULES.triggers, activated: authored::card_1411::RULES.activated, enters: authored::card_1411::RULES.enters, entry: authored::card_1411::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 2, 0, 0, 0], generic: 0 }, rules: &authored::card_1411::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Extravagant Replication", oracle_id: Some("3a646245-b8b7-4f91-a312-d5eea9a9e49a"), mana_cost: "{4}{U}{U}", type_line: "Enchantment", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1417::RULES.keywords, deck_limit: authored::card_1417::RULES.deck_limit }, effect: authored::card_1417::RULES.checked_effect(), triggers: authored::card_1417::RULES.triggers, activated: authored::card_1417::RULES.activated, enters: authored::card_1417::RULES.enters, entry: authored::card_1417::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 2, 0, 0, 0, 0], generic: 4 }, rules: &authored::card_1417::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Faebloom Trick", oracle_id: Some("903d114b-1899-4e67-bee3-af0673850388"), mana_cost: "{2}{U}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1425::RULES.keywords, deck_limit: authored::card_1425::RULES.deck_limit }, effect: authored::card_1425::RULES.checked_effect(), triggers: authored::card_1425::RULES.triggers, activated: authored::card_1425::RULES.activated, enters: authored::card_1425::RULES.enters, entry: authored::card_1425::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_1425::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Fake Your Own Death", oracle_id: Some("ad01df89-29fe-44c7-a133-91425f8ff09c"), mana_cost: "{1}{B}", type_line: "Instant", oracle_keywords: &["Treasure"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1431::RULES.keywords, deck_limit: authored::card_1431::RULES.deck_limit }, effect: authored::card_1431::RULES.checked_effect(), triggers: authored::card_1431::RULES.triggers, activated: authored::card_1431::RULES.activated, enters: authored::card_1431::RULES.enters, entry: authored::card_1431::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 1 }, rules: &authored::card_1431::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Fanatical Firebrand", oracle_id: Some("d36e11f1-6ab3-4273-8114-a8fbbe21c1c3"), mana_cost: "{R}", type_line: "Creature — Goblin Pirate", oracle_keywords: &["Haste"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1438::RULES.keywords, deck_limit: authored::card_1438::RULES.deck_limit }, effect: authored::card_1438::RULES.checked_effect(), triggers: authored::card_1438::RULES.triggers, activated: authored::card_1438::RULES.activated, enters: authored::card_1438::RULES.enters, entry: authored::card_1438::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 0 }, rules: &authored::card_1438::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Feldon's Cane", oracle_id: Some("9b884dfd-59f4-45c0-bf1e-6ad9f5b58895"), mana_cost: "{1}", type_line: "Artifact", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1484::RULES.keywords, deck_limit: authored::card_1484::RULES.deck_limit }, effect: authored::card_1484::RULES.checked_effect(), triggers: authored::card_1484::RULES.triggers, activated: authored::card_1484::RULES.activated, enters: authored::card_1484::RULES.enters, entry: authored::card_1484::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_1484::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Felidar Cub", oracle_id: Some("cae60cf8-cd64-4595-a4dd-946694cf2bb1"), mana_cost: "{1}{W}", type_line: "Creature — Cat Beast", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1485::RULES.keywords, deck_limit: authored::card_1485::RULES.deck_limit }, effect: authored::card_1485::RULES.checked_effect(), triggers: authored::card_1485::RULES.triggers, activated: authored::card_1485::RULES.activated, enters: authored::card_1485::RULES.enters, entry: authored::card_1485::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_1485::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Felidar Retreat", oracle_id: Some("16629f59-bae8-4c19-bf50-443eb0ed6856"), mana_cost: "{3}{W}", type_line: "Enchantment", oracle_keywords: &["Landfall"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1486::RULES.keywords, deck_limit: authored::card_1486::RULES.deck_limit }, effect: authored::card_1486::RULES.checked_effect(), triggers: authored::card_1486::RULES.triggers, activated: authored::card_1486::RULES.activated, enters: authored::card_1486::RULES.enters, entry: authored::card_1486::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_1486::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Felidar Savior", oracle_id: Some("2bee2508-cd5d-4c62-a205-12d8bbb3143d"), mana_cost: "{3}{W}", type_line: "Creature — Cat Beast", oracle_keywords: &["Lifelink"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1487::RULES.keywords, deck_limit: authored::card_1487::RULES.deck_limit }, effect: authored::card_1487::RULES.checked_effect(), triggers: authored::card_1487::RULES.triggers, activated: authored::card_1487::RULES.activated, enters: authored::card_1487::RULES.enters, entry: authored::card_1487::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_1487::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Felling Blow", oracle_id: Some("fbfa19ca-98bb-4c7d-a04e-920a8aafb031"), mana_cost: "{2}{G}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1491::RULES.keywords, deck_limit: authored::card_1491::RULES.deck_limit }, effect: authored::card_1491::RULES.checked_effect(), triggers: authored::card_1491::RULES.triggers, activated: authored::card_1491::RULES.activated, enters: authored::card_1491::RULES.enters, entry: authored::card_1491::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 2 }, rules: &authored::card_1491::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Fiendish Panda", oracle_id: Some("b5d64271-4478-4c55-8c71-ba8f02b47fac"), mana_cost: "{2}{W}{B}", type_line: "Creature — Bear Demon", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1503::RULES.keywords, deck_limit: authored::card_1503::RULES.deck_limit }, effect: authored::card_1503::RULES.checked_effect(), triggers: authored::card_1503::RULES.triggers, activated: authored::card_1503::RULES.activated, enters: authored::card_1503::RULES.enters, entry: authored::card_1503::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 1, 0, 0, 0], generic: 2 }, rules: &authored::card_1503::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Fierce Empath", oracle_id: Some("5104053a-d394-4b00-82a4-60fd9f051a6e"), mana_cost: "{2}{G}", type_line: "Creature — Elf", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1504::RULES.keywords, deck_limit: authored::card_1504::RULES.deck_limit }, effect: authored::card_1504::RULES.checked_effect(), triggers: authored::card_1504::RULES.triggers, activated: authored::card_1504::RULES.activated, enters: authored::card_1504::RULES.enters, entry: authored::card_1504::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 2 }, rules: &authored::card_1504::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Fiery Annihilation", oracle_id: Some("d44f3724-17b7-48f0-885d-75292669f971"), mana_cost: "{2}{R}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1505::RULES.keywords, deck_limit: authored::card_1505::RULES.deck_limit }, effect: authored::card_1505::RULES.checked_effect(), triggers: authored::card_1505::RULES.triggers, activated: authored::card_1505::RULES.activated, enters: authored::card_1505::RULES.enters, entry: authored::card_1505::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 2 }, rules: &authored::card_1505::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Finale of Revelation", oracle_id: Some("755bd5d8-67f1-4f24-a4e8-d98edf2f2e03"), mana_cost: "{X}{U}{U}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1511::RULES.keywords, deck_limit: authored::card_1511::RULES.deck_limit }, effect: authored::card_1511::RULES.checked_effect(), triggers: authored::card_1511::RULES.triggers, activated: authored::card_1511::RULES.activated, enters: authored::card_1511::RULES.enters, entry: authored::card_1511::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 2, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_1511::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Fire Elemental", oracle_id: Some("3912d21e-1ebc-4a81-9dc9-f404248d564a"), mana_cost: "{3}{R}{R}", type_line: "Creature — Elemental", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1515::RULES.keywords, deck_limit: authored::card_1515::RULES.deck_limit }, effect: authored::card_1515::RULES.checked_effect(), triggers: authored::card_1515::RULES.triggers, activated: authored::card_1515::RULES.activated, enters: authored::card_1515::RULES.enters, entry: authored::card_1515::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 2, 0, 0], generic: 3 }, rules: &authored::card_1515::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Firebrand Archer", oracle_id: Some("2e9289d6-dbc6-456d-88cf-d1f534e731d6"), mana_cost: "{1}{R}", type_line: "Creature — Human Archer", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1531::RULES.keywords, deck_limit: authored::card_1531::RULES.deck_limit }, effect: authored::card_1531::RULES.checked_effect(), triggers: authored::card_1531::RULES.triggers, activated: authored::card_1531::RULES.activated, enters: authored::card_1531::RULES.enters, entry: authored::card_1531::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 1 }, rules: &authored::card_1531::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Fireshrieker", oracle_id: Some("a02e1ca7-23c5-41e3-a744-72fc9e9dd8ba"), mana_cost: "{3}", type_line: "Artifact — Equipment", oracle_keywords: &["Equip"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1533::RULES.keywords, deck_limit: authored::card_1533::RULES.deck_limit }, effect: authored::card_1533::RULES.checked_effect(), triggers: authored::card_1533::RULES.triggers, activated: authored::card_1533::RULES.activated, enters: authored::card_1533::RULES.enters, entry: authored::card_1533::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_1533::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Firespitter Whelp", oracle_id: Some("1cad92ce-55c8-4b78-8d8c-56645ef8e6ee"), mana_cost: "{2}{R}", type_line: "Creature — Dragon", oracle_keywords: &["Flying"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1534::RULES.keywords, deck_limit: authored::card_1534::RULES.deck_limit }, effect: authored::card_1534::RULES.checked_effect(), triggers: authored::card_1534::RULES.triggers, activated: authored::card_1534::RULES.activated, enters: authored::card_1534::RULES.enters, entry: authored::card_1534::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 2 }, rules: &authored::card_1534::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Fishing Pole", oracle_id: Some("2c1d3e9e-213e-492a-ad5a-91139d70bc0f"), mana_cost: "{1}", type_line: "Artifact — Equipment", oracle_keywords: &["Equip"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1537::RULES.keywords, deck_limit: authored::card_1537::RULES.deck_limit }, effect: authored::card_1537::RULES.checked_effect(), triggers: authored::card_1537::RULES.triggers, activated: authored::card_1537::RULES.activated, enters: authored::card_1537::RULES.enters, entry: authored::card_1537::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_1537::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Flamewake Phoenix", oracle_id: Some("9e43805d-3576-47d4-9331-524869ab99d0"), mana_cost: "{1}{R}{R}", type_line: "Creature — Phoenix", oracle_keywords: &["Flying", "Ferocious", "Haste"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1546::RULES.keywords, deck_limit: authored::card_1546::RULES.deck_limit }, effect: authored::card_1546::RULES.checked_effect(), triggers: authored::card_1546::RULES.triggers, activated: authored::card_1546::RULES.activated, enters: authored::card_1546::RULES.enters, entry: authored::card_1546::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 2, 0, 0], generic: 1 }, rules: &authored::card_1546::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Flashfreeze", oracle_id: Some("eaf98e03-729b-4145-b2af-c910c415c15d"), mana_cost: "{1}{U}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1550::RULES.keywords, deck_limit: authored::card_1550::RULES.deck_limit }, effect: authored::card_1550::RULES.checked_effect(), triggers: authored::card_1550::RULES.triggers, activated: authored::card_1550::RULES.activated, enters: authored::card_1550::RULES.enters, entry: authored::card_1550::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_1550::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Fleeting Distraction", oracle_id: Some("b36271cb-cda2-434a-a580-6050dc460409"), mana_cost: "{U}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1551::RULES.keywords, deck_limit: authored::card_1551::RULES.deck_limit }, effect: authored::card_1551::RULES.checked_effect(), triggers: authored::card_1551::RULES.triggers, activated: authored::card_1551::RULES.activated, enters: authored::card_1551::RULES.enters, entry: authored::card_1551::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_1551::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Fleeting Flight", oracle_id: Some("6336401f-4e2d-4ebe-8c5b-24aa6f516abf"), mana_cost: "{W}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1553::RULES.keywords, deck_limit: authored::card_1553::RULES.deck_limit }, effect: authored::card_1553::RULES.checked_effect(), triggers: authored::card_1553::RULES.triggers, activated: authored::card_1553::RULES.activated, enters: authored::card_1553::RULES.enters, entry: authored::card_1553::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_1553::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Fog Bank", oracle_id: Some("3a2bb47d-228d-499e-a34b-9e10d99e9b2c"), mana_cost: "{1}{U}", type_line: "Creature — Wall", oracle_keywords: &["Flying", "Defender"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1574::RULES.keywords, deck_limit: authored::card_1574::RULES.deck_limit }, effect: authored::card_1574::RULES.checked_effect(), triggers: authored::card_1574::RULES.triggers, activated: authored::card_1574::RULES.activated, enters: authored::card_1574::RULES.enters, entry: authored::card_1574::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_1574::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Frenzied Goblin", oracle_id: Some("6ac470a1-c2be-4971-a5ca-10bb189ebe4d"), mana_cost: "{R}", type_line: "Creature — Goblin Berserker", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1621::RULES.keywords, deck_limit: authored::card_1621::RULES.deck_limit }, effect: authored::card_1621::RULES.checked_effect(), triggers: authored::card_1621::RULES.triggers, activated: authored::card_1621::RULES.activated, enters: authored::card_1621::RULES.enters, entry: authored::card_1621::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 0 }, rules: &authored::card_1621::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Fynn, the Fangbearer", oracle_id: Some("c0b1fba1-4338-4671-934b-098689ad2085"), mana_cost: "{1}{G}", type_line: "Legendary Creature — Human Warrior", oracle_keywords: &["Deathtouch"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_1656::RULES.keywords, deck_limit: authored::card_1656::RULES.deck_limit }, effect: authored::card_1656::RULES.checked_effect(), triggers: authored::card_1656::RULES.triggers, activated: authored::card_1656::RULES.activated, enters: authored::card_1656::RULES.enters, entry: authored::card_1656::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 1 }, rules: &authored::card_1656::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Garna, Bloodfist of Keld", oracle_id: Some("3fb83218-e18d-405d-91c8-46b5e4e672a9"), mana_cost: "{1}{B}{R}{R}", type_line: "Legendary Creature — Human Berserker", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_1683::RULES.keywords, deck_limit: authored::card_1683::RULES.deck_limit }, effect: authored::card_1683::RULES.checked_effect(), triggers: authored::card_1683::RULES.triggers, activated: authored::card_1683::RULES.activated, enters: authored::card_1683::RULES.enters, entry: authored::card_1683::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 2, 0, 0], generic: 1 }, rules: &authored::card_1683::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Gate Colossus", oracle_id: Some("386c5895-9713-4c28-9ba8-8ca0caa06c13"), mana_cost: "{8}", type_line: "Artifact Creature — Construct", oracle_keywords: &["Affinity"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1694::RULES.keywords, deck_limit: authored::card_1694::RULES.deck_limit }, effect: authored::card_1694::RULES.checked_effect(), triggers: authored::card_1694::RULES.triggers, activated: authored::card_1694::RULES.activated, enters: authored::card_1694::RULES.enters, entry: authored::card_1694::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 8 }, rules: &authored::card_1694::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Gatekeeper of Malakir", oracle_id: Some("6781f8ae-2a86-4e3d-bc43-48809c9d6c26"), mana_cost: "{B}{B}", type_line: "Creature — Vampire Warrior", oracle_keywords: &["Kicker"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1695::RULES.keywords, deck_limit: authored::card_1695::RULES.deck_limit }, effect: authored::card_1695::RULES.checked_effect(), triggers: authored::card_1695::RULES.triggers, activated: authored::card_1695::RULES.activated, enters: authored::card_1695::RULES.enters, entry: authored::card_1695::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 2, 0, 0, 0], generic: 0 }, rules: &authored::card_1695::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Gateway Sneak", oracle_id: Some("66c227f2-0e74-43e2-ab24-3866d15c5eef"), mana_cost: "{2}{U}", type_line: "Creature — Vedalken Rogue", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1696::RULES.keywords, deck_limit: authored::card_1696::RULES.deck_limit }, effect: authored::card_1696::RULES.checked_effect(), triggers: authored::card_1696::RULES.triggers, activated: authored::card_1696::RULES.activated, enters: authored::card_1696::RULES.enters, entry: authored::card_1696::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_1696::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Genesis Wave", oracle_id: Some("e2487868-f386-438e-a73f-b494f6d35fac"), mana_cost: "{X}{G}{G}{G}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1709::RULES.keywords, deck_limit: authored::card_1709::RULES.deck_limit }, effect: authored::card_1709::RULES.checked_effect(), triggers: authored::card_1709::RULES.triggers, activated: authored::card_1709::RULES.activated, enters: authored::card_1709::RULES.enters, entry: authored::card_1709::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 0, 0, 3, 0], generic: 0 }, rules: &authored::card_1709::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Ghalta, Primal Hunger", oracle_id: Some("b0b6be0c-41cf-4757-9f0e-87227b6ba6b3"), mana_cost: "{10}{G}{G}", type_line: "Legendary Creature — Elder Dinosaur", oracle_keywords: &["Trample"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_1728::RULES.keywords, deck_limit: authored::card_1728::RULES.deck_limit }, effect: authored::card_1728::RULES.checked_effect(), triggers: authored::card_1728::RULES.triggers, activated: authored::card_1728::RULES.activated, enters: authored::card_1728::RULES.enters, entry: authored::card_1728::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 2, 0], generic: 10 }, rules: &authored::card_1728::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Ghitu Lavarunner", oracle_id: Some("8e3f4987-6f31-4bc5-8bcd-6adb0440ee7c"), mana_cost: "{R}", type_line: "Creature — Human Wizard", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1731::RULES.keywords, deck_limit: authored::card_1731::RULES.deck_limit }, effect: authored::card_1731::RULES.checked_effect(), triggers: authored::card_1731::RULES.triggers, activated: authored::card_1731::RULES.activated, enters: authored::card_1731::RULES.enters, entry: authored::card_1731::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 0 }, rules: &authored::card_1731::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Giada, Font of Hope", oracle_id: Some("48e6d3d8-2f27-4017-acdd-40bce8cdbc02"), mana_cost: "{1}{W}", type_line: "Legendary Creature — Angel", oracle_keywords: &["Flying", "Vigilance"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_1736::RULES.keywords, deck_limit: authored::card_1736::RULES.deck_limit }, effect: authored::card_1736::RULES.checked_effect(), triggers: authored::card_1736::RULES.triggers, activated: authored::card_1736::RULES.activated, enters: authored::card_1736::RULES.enters, entry: authored::card_1736::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_1736::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Giant Cindermaw", oracle_id: Some("bd0c0e85-ff24-46c9-a547-4f633355c131"), mana_cost: "{2}{R}", type_line: "Creature — Dinosaur Beast", oracle_keywords: &["Trample"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1738::RULES.keywords, deck_limit: authored::card_1738::RULES.deck_limit }, effect: authored::card_1738::RULES.checked_effect(), triggers: authored::card_1738::RULES.triggers, activated: authored::card_1738::RULES.activated, enters: authored::card_1738::RULES.enters, entry: authored::card_1738::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 2 }, rules: &authored::card_1738::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Gleaming Barrier", oracle_id: Some("1eb9e401-8975-447b-839d-f7cd23897465"), mana_cost: "{2}", type_line: "Artifact Creature — Wall", oracle_keywords: &["Treasure", "Defender"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1770::RULES.keywords, deck_limit: authored::card_1770::RULES.deck_limit }, effect: authored::card_1770::RULES.checked_effect(), triggers: authored::card_1770::RULES.triggers, activated: authored::card_1770::RULES.activated, enters: authored::card_1770::RULES.enters, entry: authored::card_1770::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_1770::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Gnarlback Rhino", oracle_id: Some("1ae339fd-8e64-4e3a-8e27-e4993932ba62"), mana_cost: "{2}{G}{G}", type_line: "Creature — Rhino", oracle_keywords: &["Trample"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1793::RULES.keywords, deck_limit: authored::card_1793::RULES.deck_limit }, effect: authored::card_1793::RULES.checked_effect(), triggers: authored::card_1793::RULES.triggers, activated: authored::card_1793::RULES.activated, enters: authored::card_1793::RULES.enters, entry: authored::card_1793::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 2, 0], generic: 2 }, rules: &authored::card_1793::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Gnarlid Colony", oracle_id: Some("e5481431-c952-4ad9-94fe-355076ef632b"), mana_cost: "{1}{G}", type_line: "Creature — Beast", oracle_keywords: &["Kicker"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1795::RULES.keywords, deck_limit: authored::card_1795::RULES.deck_limit }, effect: authored::card_1795::RULES.checked_effect(), triggers: authored::card_1795::RULES.triggers, activated: authored::card_1795::RULES.activated, enters: authored::card_1795::RULES.enters, entry: authored::card_1795::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 1 }, rules: &authored::card_1795::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Goblin Boarders", oracle_id: Some("4c64feae-95d7-426d-9e42-d3b0841d2054"), mana_cost: "{2}{R}", type_line: "Creature — Goblin Pirate", oracle_keywords: &["Raid"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1801::RULES.keywords, deck_limit: authored::card_1801::RULES.deck_limit }, effect: authored::card_1801::RULES.checked_effect(), triggers: authored::card_1801::RULES.triggers, activated: authored::card_1801::RULES.activated, enters: authored::card_1801::RULES.enters, entry: authored::card_1801::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 2 }, rules: &authored::card_1801::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Goblin Firebomb", oracle_id: Some("2625fa9c-638c-46d3-966b-1166f440186f"), mana_cost: "{1}", type_line: "Artifact", oracle_keywords: &["Flash"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1802::RULES.keywords, deck_limit: authored::card_1802::RULES.deck_limit }, effect: authored::card_1802::RULES.checked_effect(), triggers: authored::card_1802::RULES.triggers, activated: authored::card_1802::RULES.activated, enters: authored::card_1802::RULES.enters, entry: authored::card_1802::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_1802::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Goblin Negotiation", oracle_id: Some("79824266-bffa-476c-bdd3-976e9d789a42"), mana_cost: "{X}{R}{R}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1805::RULES.keywords, deck_limit: authored::card_1805::RULES.deck_limit }, effect: authored::card_1805::RULES.checked_effect(), triggers: authored::card_1805::RULES.triggers, activated: authored::card_1805::RULES.activated, enters: authored::card_1805::RULES.enters, entry: authored::card_1805::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 0, 2, 0, 0], generic: 0 }, rules: &authored::card_1805::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Goblin Oriflamme", oracle_id: Some("836bd011-2da5-443a-a814-19a664b98a1a"), mana_cost: "{1}{R}", type_line: "Enchantment", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1806::RULES.keywords, deck_limit: authored::card_1806::RULES.deck_limit }, effect: authored::card_1806::RULES.checked_effect(), triggers: authored::card_1806::RULES.triggers, activated: authored::card_1806::RULES.activated, enters: authored::card_1806::RULES.enters, entry: authored::card_1806::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 1 }, rules: &authored::card_1806::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Goblin Smuggler", oracle_id: Some("eae15f7d-1f6e-4dfc-afea-7698b93cd145"), mana_cost: "{2}{R}", type_line: "Creature — Goblin Rogue", oracle_keywords: &["Haste"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1808::RULES.keywords, deck_limit: authored::card_1808::RULES.deck_limit }, effect: authored::card_1808::RULES.checked_effect(), triggers: authored::card_1808::RULES.triggers, activated: authored::card_1808::RULES.activated, enters: authored::card_1808::RULES.enters, entry: authored::card_1808::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 2 }, rules: &authored::card_1808::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Goblin Surprise", oracle_id: Some("79cd09e8-d738-47c0-9473-b6281bec6935"), mana_cost: "{2}{R}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1809::RULES.keywords, deck_limit: authored::card_1809::RULES.deck_limit }, effect: authored::card_1809::RULES.checked_effect(), triggers: authored::card_1809::RULES.triggers, activated: authored::card_1809::RULES.activated, enters: authored::card_1809::RULES.enters, entry: authored::card_1809::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 2 }, rules: &authored::card_1809::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Goldvein Pick", oracle_id: Some("c1624d10-8838-4af8-aea1-a96c0fe6fd6b"), mana_cost: "{2}", type_line: "Artifact — Equipment", oracle_keywords: &["Treasure", "Equip"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1824::RULES.keywords, deck_limit: authored::card_1824::RULES.deck_limit }, effect: authored::card_1824::RULES.checked_effect(), triggers: authored::card_1824::RULES.triggers, activated: authored::card_1824::RULES.activated, enters: authored::card_1824::RULES.enters, entry: authored::card_1824::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_1824::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Golgari Guildgate", oracle_id: Some("fa2da325-6859-45bb-b185-35526b01bcc1"), mana_cost: "", type_line: "Land — Gate", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1825::RULES.keywords, deck_limit: authored::card_1825::RULES.deck_limit }, effect: authored::card_1825::RULES.checked_effect(), triggers: authored::card_1825::RULES.triggers, activated: authored::card_1825::RULES.activated, enters: authored::card_1825::RULES.enters, entry: authored::card_1825::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_1825::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Good-Fortune Unicorn", oracle_id: Some("1e0cdff3-3ec5-41fc-8053-f072bae156b3"), mana_cost: "{1}{G}{W}", type_line: "Creature — Unicorn", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1833::RULES.keywords, deck_limit: authored::card_1833::RULES.deck_limit }, effect: authored::card_1833::RULES.checked_effect(), triggers: authored::card_1833::RULES.triggers, activated: authored::card_1833::RULES.activated, enters: authored::card_1833::RULES.enters, entry: authored::card_1833::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 1, 0], generic: 1 }, rules: &authored::card_1833::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Gorehorn Raider", oracle_id: Some("41699d92-c209-4fcb-bbd6-685b3437d0c2"), mana_cost: "{4}{R}", type_line: "Creature — Minotaur Pirate", oracle_keywords: &["Raid"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1834::RULES.keywords, deck_limit: authored::card_1834::RULES.deck_limit }, effect: authored::card_1834::RULES.checked_effect(), triggers: authored::card_1834::RULES.triggers, activated: authored::card_1834::RULES.activated, enters: authored::card_1834::RULES.enters, entry: authored::card_1834::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 4 }, rules: &authored::card_1834::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Grappling Kraken", oracle_id: Some("1ba99ade-df8d-4654-882a-a5f4256ada3d"), mana_cost: "{4}{U}{U}", type_line: "Creature — Kraken", oracle_keywords: &["Landfall"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1848::RULES.keywords, deck_limit: authored::card_1848::RULES.deck_limit }, effect: authored::card_1848::RULES.checked_effect(), triggers: authored::card_1848::RULES.triggers, activated: authored::card_1848::RULES.activated, enters: authored::card_1848::RULES.enters, entry: authored::card_1848::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 2, 0, 0, 0, 0], generic: 4 }, rules: &authored::card_1848::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Gratuitous Violence", oracle_id: Some("7c340a39-4ee0-4ba1-bb66-6674f8020fda"), mana_cost: "{2}{R}{R}{R}", type_line: "Enchantment", oracle_keywords: &["Double"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1851::RULES.keywords, deck_limit: authored::card_1851::RULES.deck_limit }, effect: authored::card_1851::RULES.checked_effect(), triggers: authored::card_1851::RULES.triggers, activated: authored::card_1851::RULES.activated, enters: authored::card_1851::RULES.enters, entry: authored::card_1851::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 0, 0, 3, 0, 0], generic: 2 }, rules: &authored::card_1851::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Grow from the Ashes", oracle_id: Some("645ab3c7-ee1d-4dd0-811f-2dc7f7c7e792"), mana_cost: "{2}{G}", type_line: "Sorcery", oracle_keywords: &["Kicker"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1888::RULES.keywords, deck_limit: authored::card_1888::RULES.deck_limit }, effect: authored::card_1888::RULES.checked_effect(), triggers: authored::card_1888::RULES.triggers, activated: authored::card_1888::RULES.activated, enters: authored::card_1888::RULES.enters, entry: authored::card_1888::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 2 }, rules: &authored::card_1888::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Gruul Guildgate", oracle_id: Some("d38476e9-2e47-4c0c-8129-483c0bd09ec0"), mana_cost: "", type_line: "Land — Gate", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1895::RULES.keywords, deck_limit: authored::card_1895::RULES.deck_limit }, effect: authored::card_1895::RULES.checked_effect(), triggers: authored::card_1895::RULES.triggers, activated: authored::card_1895::RULES.activated, enters: authored::card_1895::RULES.enters, entry: authored::card_1895::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_1895::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Guarded Heir", oracle_id: Some("63e79a6f-ed98-480a-a740-d44e111046f7"), mana_cost: "{5}{W}", type_line: "Creature — Human Noble", oracle_keywords: &["Lifelink"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1898::RULES.keywords, deck_limit: authored::card_1898::RULES.deck_limit }, effect: authored::card_1898::RULES.checked_effect(), triggers: authored::card_1898::RULES.triggers, activated: authored::card_1898::RULES.activated, enters: authored::card_1898::RULES.enters, entry: authored::card_1898::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 5 }, rules: &authored::card_1898::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Gutless Plunderer", oracle_id: Some("31265357-5ecb-4d83-80ff-1795b1dfdecb"), mana_cost: "{2}{B}", type_line: "Creature — Skeleton Pirate", oracle_keywords: &["Raid", "Deathtouch"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1913::RULES.keywords, deck_limit: authored::card_1913::RULES.deck_limit }, effect: authored::card_1913::RULES.checked_effect(), triggers: authored::card_1913::RULES.triggers, activated: authored::card_1913::RULES.activated, enters: authored::card_1913::RULES.enters, entry: authored::card_1913::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 2 }, rules: &authored::card_1913::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Halana and Alena, Partners", oracle_id: Some("b347be2d-9aa1-42a9-b652-8fb032832f09"), mana_cost: "{2}{R}{G}", type_line: "Legendary Creature — Human Ranger", oracle_keywords: &["Reach", "First strike"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_1927::RULES.keywords, deck_limit: authored::card_1927::RULES.deck_limit }, effect: authored::card_1927::RULES.checked_effect(), triggers: authored::card_1927::RULES.triggers, activated: authored::card_1927::RULES.activated, enters: authored::card_1927::RULES.enters, entry: authored::card_1927::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 1, 0], generic: 2 }, rules: &authored::card_1927::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Harbinger of the Tides", oracle_id: Some("9f5277e8-1cbe-478c-a8bc-ac028f13305e"), mana_cost: "{U}{U}", type_line: "Creature — Merfolk Wizard", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1939::RULES.keywords, deck_limit: authored::card_1939::RULES.deck_limit }, effect: authored::card_1939::RULES.checked_effect(), triggers: authored::card_1939::RULES.triggers, activated: authored::card_1939::RULES.activated, enters: authored::card_1939::RULES.enters, entry: authored::card_1939::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 2, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_1939::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Hare Apparent", oracle_id: Some("3c1619bd-db5e-4df6-a196-0a9d62374f6d"), mana_cost: "{1}{W}", type_line: "Creature — Rabbit Noble", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1947::RULES.keywords, deck_limit: authored::card_1947::RULES.deck_limit }, effect: authored::card_1947::RULES.checked_effect(), triggers: authored::card_1947::RULES.triggers, activated: authored::card_1947::RULES.activated, enters: authored::card_1947::RULES.enters, entry: authored::card_1947::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_1947::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Harmless Offering", oracle_id: Some("f5d32e5f-066b-42cf-8dd1-072a96275313"), mana_cost: "{2}{R}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1948::RULES.keywords, deck_limit: authored::card_1948::RULES.deck_limit }, effect: authored::card_1948::RULES.checked_effect(), triggers: authored::card_1948::RULES.triggers, activated: authored::card_1948::RULES.activated, enters: authored::card_1948::RULES.enters, entry: authored::card_1948::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 2 }, rules: &authored::card_1948::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Healer's Hawk", oracle_id: Some("28a52ba1-95da-44e1-8ac5-0dc23c902394"), mana_cost: "{W}", type_line: "Creature — Bird", oracle_keywords: &["Flying", "Lifelink"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1975::RULES.keywords, deck_limit: authored::card_1975::RULES.deck_limit }, effect: authored::card_1975::RULES.checked_effect(), triggers: authored::card_1975::RULES.triggers, activated: authored::card_1975::RULES.activated, enters: authored::card_1975::RULES.enters, entry: authored::card_1975::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_1975::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Heartfire Immolator", oracle_id: Some("1e414a4f-6e76-4cbe-9c69-d52852cd5bc0"), mana_cost: "{1}{R}", type_line: "Creature — Human Wizard", oracle_keywords: &["Prowess"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1978::RULES.keywords, deck_limit: authored::card_1978::RULES.deck_limit }, effect: authored::card_1978::RULES.checked_effect(), triggers: authored::card_1978::RULES.triggers, activated: authored::card_1978::RULES.activated, enters: authored::card_1978::RULES.enters, entry: authored::card_1978::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 1 }, rules: &authored::card_1978::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Hedron Archive", oracle_id: Some("32263baa-d3f0-463f-92b3-4e9938476add"), mana_cost: "{4}", type_line: "Artifact", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_1990::RULES.keywords, deck_limit: authored::card_1990::RULES.deck_limit }, effect: authored::card_1990::RULES.checked_effect(), triggers: authored::card_1990::RULES.triggers, activated: authored::card_1990::RULES.activated, enters: authored::card_1990::RULES.enters, entry: authored::card_1990::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 4 }, rules: &authored::card_1990::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Helpful Hunter", oracle_id: Some("c0864adb-e9aa-40b6-91ff-a0646193e887"), mana_cost: "{1}{W}", type_line: "Creature — Cat", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2002::RULES.keywords, deck_limit: authored::card_2002::RULES.deck_limit }, effect: authored::card_2002::RULES.checked_effect(), triggers: authored::card_2002::RULES.triggers, activated: authored::card_2002::RULES.activated, enters: authored::card_2002::RULES.enters, entry: authored::card_2002::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_2002::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Herald of Eternal Dawn", oracle_id: Some("4080f7e3-06d3-4d3d-9929-4e826cb66713"), mana_cost: "{4}{W}{W}{W}", type_line: "Creature — Angel", oracle_keywords: &["Flying", "Flash"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2006::RULES.keywords, deck_limit: authored::card_2006::RULES.deck_limit }, effect: authored::card_2006::RULES.checked_effect(), triggers: authored::card_2006::RULES.triggers, activated: authored::card_2006::RULES.activated, enters: authored::card_2006::RULES.enters, entry: authored::card_2006::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [3, 0, 0, 0, 0, 0], generic: 4 }, rules: &authored::card_2006::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Herald of Faith", oracle_id: Some("cb97da84-1d13-4795-b68a-2bf111a50067"), mana_cost: "{3}{W}{W}", type_line: "Creature — Angel", oracle_keywords: &["Flying"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2007::RULES.keywords, deck_limit: authored::card_2007::RULES.deck_limit }, effect: authored::card_2007::RULES.checked_effect(), triggers: authored::card_2007::RULES.triggers, activated: authored::card_2007::RULES.activated, enters: authored::card_2007::RULES.enters, entry: authored::card_2007::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [2, 0, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_2007::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Heraldic Banner", oracle_id: Some("3525e263-e29a-49bf-a29f-fb3ce43bbd33"), mana_cost: "{3}", type_line: "Artifact", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2008::RULES.keywords, deck_limit: authored::card_2008::RULES.deck_limit }, effect: authored::card_2008::RULES.checked_effect(), triggers: authored::card_2008::RULES.triggers, activated: authored::card_2008::RULES.activated, enters: authored::card_2008::RULES.enters, entry: authored::card_2008::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_2008::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Hero's Downfall", oracle_id: Some("03df6a57-37c9-46d3-83b3-4a6240100714"), mana_cost: "{1}{B}{B}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2015::RULES.keywords, deck_limit: authored::card_2015::RULES.deck_limit }, effect: authored::card_2015::RULES.checked_effect(), triggers: authored::card_2015::RULES.triggers, activated: authored::card_2015::RULES.activated, enters: authored::card_2015::RULES.enters, entry: authored::card_2015::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 2, 0, 0, 0], generic: 1 }, rules: &authored::card_2015::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Heroes' Bane", oracle_id: Some("bccd722a-b610-4017-93bd-313f08448f5e"), mana_cost: "{3}{G}{G}", type_line: "Creature — Hydra", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2016::RULES.keywords, deck_limit: authored::card_2016::RULES.deck_limit }, effect: authored::card_2016::RULES.checked_effect(), triggers: authored::card_2016::RULES.triggers, activated: authored::card_2016::RULES.activated, enters: authored::card_2016::RULES.enters, entry: authored::card_2016::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 2, 0], generic: 3 }, rules: &authored::card_2016::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Heroic Reinforcements", oracle_id: Some("c442e8b0-a0a4-4839-8049-125b91b15c99"), mana_cost: "{2}{R}{W}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2019::RULES.keywords, deck_limit: authored::card_2019::RULES.deck_limit }, effect: authored::card_2019::RULES.checked_effect(), triggers: authored::card_2019::RULES.triggers, activated: authored::card_2019::RULES.activated, enters: authored::card_2019::RULES.enters, entry: authored::card_2019::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [1, 0, 0, 1, 0, 0], generic: 2 }, rules: &authored::card_2019::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Hidetsugu's Second Rite", oracle_id: Some("749683c9-8cc0-4e8e-8d90-adda5e5f4e93"), mana_cost: "{3}{R}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2034::RULES.keywords, deck_limit: authored::card_2034::RULES.deck_limit }, effect: authored::card_2034::RULES.checked_effect(), triggers: authored::card_2034::RULES.triggers, activated: authored::card_2034::RULES.activated, enters: authored::card_2034::RULES.enters, entry: authored::card_2034::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 3 }, rules: &authored::card_2034::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "High Fae Trickster", oracle_id: Some("28c1a4a0-f58c-4d07-a918-51ff8a74f050"), mana_cost: "{3}{U}", type_line: "Creature — Faerie Wizard", oracle_keywords: &["Flying", "Flash"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2036::RULES.keywords, deck_limit: authored::card_2036::RULES.deck_limit }, effect: authored::card_2036::RULES.checked_effect(), triggers: authored::card_2036::RULES.triggers, activated: authored::card_2036::RULES.activated, enters: authored::card_2036::RULES.enters, entry: authored::card_2036::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_2036::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "High-Society Hunter", oracle_id: Some("0dc158ba-7ec8-4558-91f5-0a87ef8380d4"), mana_cost: "{3}{B}{B}", type_line: "Creature — Vampire Noble", oracle_keywords: &["Flying"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2041::RULES.keywords, deck_limit: authored::card_2041::RULES.deck_limit }, effect: authored::card_2041::RULES.checked_effect(), triggers: authored::card_2041::RULES.triggers, activated: authored::card_2041::RULES.activated, enters: authored::card_2041::RULES.enters, entry: authored::card_2041::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 2, 0, 0, 0], generic: 3 }, rules: &authored::card_2041::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Hinterland Sanctifier", oracle_id: Some("d812fc6d-b96d-4986-b171-9f3feee603dc"), mana_cost: "{W}", type_line: "Creature — Rabbit Cleric", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2046::RULES.keywords, deck_limit: authored::card_2046::RULES.deck_limit }, effect: authored::card_2046::RULES.checked_effect(), triggers: authored::card_2046::RULES.triggers, activated: authored::card_2046::RULES.activated, enters: authored::card_2046::RULES.enters, entry: authored::card_2046::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_2046::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Hoarding Dragon", oracle_id: Some("d36d3c93-ed24-4447-9d15-3071f5c0989b"), mana_cost: "{3}{R}{R}", type_line: "Creature — Dragon", oracle_keywords: &["Flying"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2052::RULES.keywords, deck_limit: authored::card_2052::RULES.deck_limit }, effect: authored::card_2052::RULES.checked_effect(), triggers: authored::card_2052::RULES.triggers, activated: authored::card_2052::RULES.activated, enters: authored::card_2052::RULES.enters, entry: authored::card_2052::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 2, 0, 0], generic: 3 }, rules: &authored::card_2052::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Homunculus Horde", oracle_id: Some("b3c69bf4-880f-4198-a67c-87d23d9c3d41"), mana_cost: "{3}{U}", type_line: "Creature — Homunculus", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2062::RULES.keywords, deck_limit: authored::card_2062::RULES.deck_limit }, effect: authored::card_2062::RULES.checked_effect(), triggers: authored::card_2062::RULES.triggers, activated: authored::card_2062::RULES.activated, enters: authored::card_2062::RULES.enters, entry: authored::card_2062::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_2062::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Hungry Ghoul", oracle_id: Some("4a7a55a3-568c-4c31-9460-5b94dc0121d8"), mana_cost: "{1}{B}", type_line: "Creature — Zombie", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2101::RULES.keywords, deck_limit: authored::card_2101::RULES.deck_limit }, effect: authored::card_2101::RULES.checked_effect(), triggers: authored::card_2101::RULES.triggers, activated: authored::card_2101::RULES.activated, enters: authored::card_2101::RULES.enters, entry: authored::card_2101::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 1 }, rules: &authored::card_2101::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Icewind Elemental", oracle_id: Some("5001bf21-9dbb-4229-aeef-25805e71b832"), mana_cost: "{4}{U}", type_line: "Creature — Elemental", oracle_keywords: &["Flying"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2126::RULES.keywords, deck_limit: authored::card_2126::RULES.deck_limit }, effect: authored::card_2126::RULES.checked_effect(), triggers: authored::card_2126::RULES.triggers, activated: authored::card_2126::RULES.activated, enters: authored::card_2126::RULES.enters, entry: authored::card_2126::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 4 }, rules: &authored::card_2126::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Immersturm Predator", oracle_id: Some("fcb6c6d4-ea8d-49cf-b6df-377084bdb757"), mana_cost: "{2}{B}{R}", type_line: "Creature — Vampire Dragon", oracle_keywords: &["Flying"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2142::RULES.keywords, deck_limit: authored::card_2142::RULES.deck_limit }, effect: authored::card_2142::RULES.checked_effect(), triggers: authored::card_2142::RULES.triggers, activated: authored::card_2142::RULES.activated, enters: authored::card_2142::RULES.enters, entry: authored::card_2142::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 1, 0, 0], generic: 2 }, rules: &authored::card_2142::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Impact Tremors", oracle_id: Some("9242cd3e-1a71-4700-8182-9c1005616033"), mana_cost: "{1}{R}", type_line: "Enchantment", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2145::RULES.keywords, deck_limit: authored::card_2145::RULES.deck_limit }, effect: authored::card_2145::RULES.checked_effect(), triggers: authored::card_2145::RULES.triggers, activated: authored::card_2145::RULES.activated, enters: authored::card_2145::RULES.enters, entry: authored::card_2145::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 1 }, rules: &authored::card_2145::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Imperious Perfect", oracle_id: Some("3fa71348-fa4d-4f39-a451-cf1570591991"), mana_cost: "{2}{G}", type_line: "Creature — Elf Warrior", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2147::RULES.keywords, deck_limit: authored::card_2147::RULES.deck_limit }, effect: authored::card_2147::RULES.checked_effect(), triggers: authored::card_2147::RULES.triggers, activated: authored::card_2147::RULES.activated, enters: authored::card_2147::RULES.enters, entry: authored::card_2147::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 2 }, rules: &authored::card_2147::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Incinerating Blast", oracle_id: Some("49f0da4a-e046-4220-b90e-e105a5f2ef00"), mana_cost: "{4}{R}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2156::RULES.keywords, deck_limit: authored::card_2156::RULES.deck_limit }, effect: authored::card_2156::RULES.checked_effect(), triggers: authored::card_2156::RULES.triggers, activated: authored::card_2156::RULES.activated, enters: authored::card_2156::RULES.enters, entry: authored::card_2156::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 4 }, rules: &authored::card_2156::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Infernal Vessel", oracle_id: Some("0e13c283-f462-441f-a8a5-0c75a59ff1d9"), mana_cost: "{2}{B}", type_line: "Creature — Human Cleric", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2160::RULES.keywords, deck_limit: authored::card_2160::RULES.deck_limit }, effect: authored::card_2160::RULES.checked_effect(), triggers: authored::card_2160::RULES.triggers, activated: authored::card_2160::RULES.activated, enters: authored::card_2160::RULES.enters, entry: authored::card_2160::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 2 }, rules: &authored::card_2160::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Infestation Sage", oracle_id: Some("d398dca3-f3a5-44c9-998f-b4fce9e5c2bf"), mana_cost: "{B}", type_line: "Creature — Elf Warlock", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2161::RULES.keywords, deck_limit: authored::card_2161::RULES.deck_limit }, effect: authored::card_2161::RULES.checked_effect(), triggers: authored::card_2161::RULES.triggers, activated: authored::card_2161::RULES.activated, enters: authored::card_2161::RULES.enters, entry: authored::card_2161::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 0 }, rules: &authored::card_2161::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Ingenious Leonin", oracle_id: Some("99853d2e-2223-4c05-bbd7-0a746f8f810d"), mana_cost: "{4}{W}", type_line: "Creature — Cat Soldier", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2166::RULES.keywords, deck_limit: authored::card_2166::RULES.deck_limit }, effect: authored::card_2166::RULES.checked_effect(), triggers: authored::card_2166::RULES.triggers, activated: authored::card_2166::RULES.activated, enters: authored::card_2166::RULES.enters, entry: authored::card_2166::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 4 }, rules: &authored::card_2166::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Inspiration from Beyond", oracle_id: Some("be87e410-4302-43ee-b880-56eedf2b3648"), mana_cost: "{2}{U}", type_line: "Sorcery", oracle_keywords: &["Flashback", "Mill"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2185::RULES.keywords, deck_limit: authored::card_2185::RULES.deck_limit }, effect: authored::card_2185::RULES.checked_effect(), triggers: authored::card_2185::RULES.triggers, activated: authored::card_2185::RULES.activated, enters: authored::card_2185::RULES.enters, entry: authored::card_2185::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_2185::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Inspiring Overseer", oracle_id: Some("d646e42b-5635-4798-b633-29c093b66a55"), mana_cost: "{2}{W}", type_line: "Creature — Angel Cleric", oracle_keywords: &["Flying"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2188::RULES.keywords, deck_limit: authored::card_2188::RULES.deck_limit }, effect: authored::card_2188::RULES.checked_effect(), triggers: authored::card_2188::RULES.triggers, activated: authored::card_2188::RULES.activated, enters: authored::card_2188::RULES.enters, entry: authored::card_2188::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_2188::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Inspiring Paladin", oracle_id: Some("95efbb81-2418-42ed-ae34-5759efff8253"), mana_cost: "{2}{W}", type_line: "Creature — Human Knight", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2189::RULES.keywords, deck_limit: authored::card_2189::RULES.deck_limit }, effect: authored::card_2189::RULES.checked_effect(), triggers: authored::card_2189::RULES.triggers, activated: authored::card_2189::RULES.activated, enters: authored::card_2189::RULES.enters, entry: authored::card_2189::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_2189::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Into the Roil", oracle_id: Some("c2898bbd-82a4-4d26-b6ee-169b0ebe71b4"), mana_cost: "{1}{U}", type_line: "Instant", oracle_keywords: &["Kicker"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2202::RULES.keywords, deck_limit: authored::card_2202::RULES.deck_limit }, effect: authored::card_2202::RULES.checked_effect(), triggers: authored::card_2202::RULES.triggers, activated: authored::card_2202::RULES.activated, enters: authored::card_2202::RULES.enters, entry: authored::card_2202::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_2202::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Involuntary Employment", oracle_id: Some("7e86fb1e-9987-43ba-b59e-0cb03bbdedb6"), mana_cost: "{3}{R}", type_line: "Sorcery", oracle_keywords: &["Treasure"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2217::RULES.keywords, deck_limit: authored::card_2217::RULES.deck_limit }, effect: authored::card_2217::RULES.checked_effect(), triggers: authored::card_2217::RULES.triggers, activated: authored::card_2217::RULES.activated, enters: authored::card_2217::RULES.enters, entry: authored::card_2217::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 3 }, rules: &authored::card_2217::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Izzet Guildgate", oracle_id: Some("bf75a3d1-f184-4b48-a913-21caee1db084"), mana_cost: "", type_line: "Land — Gate", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2245::RULES.keywords, deck_limit: authored::card_2245::RULES.deck_limit }, effect: authored::card_2245::RULES.checked_effect(), triggers: authored::card_2245::RULES.triggers, activated: authored::card_2245::RULES.activated, enters: authored::card_2245::RULES.enters, entry: authored::card_2245::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_2245::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Joraga Invocation", oracle_id: Some("216d3eed-cb07-40c5-869a-962e074e757d"), mana_cost: "{4}{G}{G}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2289::RULES.keywords, deck_limit: authored::card_2289::RULES.deck_limit }, effect: authored::card_2289::RULES.checked_effect(), triggers: authored::card_2289::RULES.triggers, activated: authored::card_2289::RULES.activated, enters: authored::card_2289::RULES.enters, entry: authored::card_2289::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 0, 0, 2, 0], generic: 4 }, rules: &authored::card_2289::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Joust Through", oracle_id: Some("33c21ef1-33eb-447a-a318-a352f163a8a0"), mana_cost: "{W}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2291::RULES.keywords, deck_limit: authored::card_2291::RULES.deck_limit }, effect: authored::card_2291::RULES.checked_effect(), triggers: authored::card_2291::RULES.triggers, activated: authored::card_2291::RULES.activated, enters: authored::card_2291::RULES.enters, entry: authored::card_2291::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_2291::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Juggernaut", oracle_id: Some("4ac9116f-36bc-4d71-b696-d6ee064e1d58"), mana_cost: "{4}", type_line: "Artifact Creature — Juggernaut", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2295::RULES.keywords, deck_limit: authored::card_2295::RULES.deck_limit }, effect: authored::card_2295::RULES.checked_effect(), triggers: authored::card_2295::RULES.triggers, activated: authored::card_2295::RULES.activated, enters: authored::card_2295::RULES.enters, entry: authored::card_2295::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 4 }, rules: &authored::card_2295::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Kaito, Cunning Infiltrator", oracle_id: Some("f6d3f145-00e0-47cd-91d8-0381c19a000d"), mana_cost: "{1}{U}{U}", type_line: "Legendary Planeswalker — Kaito", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2307::RULES.keywords, deck_limit: authored::card_2307::RULES.deck_limit }, effect: authored::card_2307::RULES.checked_effect(), triggers: authored::card_2307::RULES.triggers, activated: authored::card_2307::RULES.activated, enters: authored::card_2307::RULES.enters, entry: authored::card_2307::RULES.entry, kind: 64, basic: false, mana_color: None, colored: [0, 2, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_2307::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Kalastria Highborn", oracle_id: Some("77e319da-d782-4258-bf0e-626755e5adc9"), mana_cost: "{B}{B}", type_line: "Creature — Vampire Shaman", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2309::RULES.keywords, deck_limit: authored::card_2309::RULES.deck_limit }, effect: authored::card_2309::RULES.checked_effect(), triggers: authored::card_2309::RULES.triggers, activated: authored::card_2309::RULES.activated, enters: authored::card_2309::RULES.enters, entry: authored::card_2309::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 2, 0, 0, 0], generic: 0 }, rules: &authored::card_2309::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Kargan Dragonrider", oracle_id: Some("df366871-8ed7-4183-8b0c-414ca361a711"), mana_cost: "{1}{R}", type_line: "Creature — Human Warrior", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2317::RULES.keywords, deck_limit: authored::card_2317::RULES.deck_limit }, effect: authored::card_2317::RULES.checked_effect(), triggers: authored::card_2317::RULES.triggers, activated: authored::card_2317::RULES.activated, enters: authored::card_2317::RULES.enters, entry: authored::card_2317::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 1 }, rules: &authored::card_2317::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Kellan, Planar Trailblazer", oracle_id: Some("e8fd428f-7521-4e6a-86b7-835f20ee3c5c"), mana_cost: "{R}", type_line: "Legendary Creature — Human Faerie Scout", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2341::RULES.keywords, deck_limit: authored::card_2341::RULES.deck_limit }, effect: authored::card_2341::RULES.checked_effect(), triggers: authored::card_2341::RULES.triggers, activated: authored::card_2341::RULES.activated, enters: authored::card_2341::RULES.enters, entry: authored::card_2341::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 0 }, rules: &authored::card_2341::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Kindled Fury", oracle_id: Some("ebeee06e-3345-4f02-896f-cb0b2cfa5548"), mana_cost: "{R}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2357::RULES.keywords, deck_limit: authored::card_2357::RULES.deck_limit }, effect: authored::card_2357::RULES.checked_effect(), triggers: authored::card_2357::RULES.triggers, activated: authored::card_2357::RULES.activated, enters: authored::card_2357::RULES.enters, entry: authored::card_2357::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 0 }, rules: &authored::card_2357::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Kiora, the Rising Tide", oracle_id: Some("5d748bce-dff8-46fa-a3d1-633863b7bbff"), mana_cost: "{2}{U}", type_line: "Legendary Creature — Merfolk Noble", oracle_keywords: &["Threshold"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2368::RULES.keywords, deck_limit: authored::card_2368::RULES.deck_limit }, effect: authored::card_2368::RULES.checked_effect(), triggers: authored::card_2368::RULES.triggers, activated: authored::card_2368::RULES.activated, enters: authored::card_2368::RULES.enters, entry: authored::card_2368::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_2368::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Kitesail Corsair", oracle_id: Some("6103f294-f920-4307-af80-d44cc71167fd"), mana_cost: "{1}{U}", type_line: "Creature — Human Pirate", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2374::RULES.keywords, deck_limit: authored::card_2374::RULES.deck_limit }, effect: authored::card_2374::RULES.checked_effect(), triggers: authored::card_2374::RULES.triggers, activated: authored::card_2374::RULES.activated, enters: authored::card_2374::RULES.enters, entry: authored::card_2374::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_2374::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Knight of Grace", oracle_id: Some("7b51a979-47e0-4b98-8ec5-8064c83422e1"), mana_cost: "{1}{W}", type_line: "Creature — Human Knight", oracle_keywords: &["Hexproof from", "Hexproof", "First strike"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2385::RULES.keywords, deck_limit: authored::card_2385::RULES.deck_limit }, effect: authored::card_2385::RULES.checked_effect(), triggers: authored::card_2385::RULES.triggers, activated: authored::card_2385::RULES.activated, enters: authored::card_2385::RULES.enters, entry: authored::card_2385::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_2385::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Knight of Malice", oracle_id: Some("b89d912a-65a0-4bc9-9e1d-ff8317458a72"), mana_cost: "{1}{B}", type_line: "Creature — Human Knight", oracle_keywords: &["Hexproof from", "Hexproof", "First strike"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2386::RULES.keywords, deck_limit: authored::card_2386::RULES.deck_limit }, effect: authored::card_2386::RULES.checked_effect(), triggers: authored::card_2386::RULES.triggers, activated: authored::card_2386::RULES.activated, enters: authored::card_2386::RULES.enters, entry: authored::card_2386::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 1 }, rules: &authored::card_2386::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Koma, World-Eater", oracle_id: Some("1ebf2142-f171-40cd-8143-15f50084b707"), mana_cost: "{3}{G}{G}{U}{U}", type_line: "Legendary Creature — Serpent", oracle_keywords: &["Trample", "Ward"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2393::RULES.keywords, deck_limit: authored::card_2393::RULES.deck_limit }, effect: authored::card_2393::RULES.checked_effect(), triggers: authored::card_2393::RULES.triggers, activated: authored::card_2393::RULES.activated, enters: authored::card_2393::RULES.enters, entry: authored::card_2393::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 2, 0, 0, 2, 0], generic: 3 }, rules: &authored::card_2393::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Krenko, Mob Boss", oracle_id: Some("68418069-f615-40ef-ae0d-764192acae00"), mana_cost: "{2}{R}{R}", type_line: "Legendary Creature — Goblin Warrior", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2416::RULES.keywords, deck_limit: authored::card_2416::RULES.deck_limit }, effect: authored::card_2416::RULES.checked_effect(), triggers: authored::card_2416::RULES.triggers, activated: authored::card_2416::RULES.activated, enters: authored::card_2416::RULES.enters, entry: authored::card_2416::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 2, 0, 0], generic: 2 }, rules: &authored::card_2416::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Kykar, Zephyr Awakener", oracle_id: Some("0156c94b-e64c-49de-a46c-a203c5e05e98"), mana_cost: "{2}{W}{U}", type_line: "Legendary Creature — Bird Wizard", oracle_keywords: &["Flying"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2426::RULES.keywords, deck_limit: authored::card_2426::RULES.deck_limit }, effect: authored::card_2426::RULES.checked_effect(), triggers: authored::card_2426::RULES.triggers, activated: authored::card_2426::RULES.activated, enters: authored::card_2426::RULES.enters, entry: authored::card_2426::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 1, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_2426::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Lathril, Blade of the Elves", oracle_id: Some("9b568c34-ba0a-44a7-a0ee-bee3a37b3396"), mana_cost: "{2}{B}{G}", type_line: "Legendary Creature — Elf Noble", oracle_keywords: &["Menace"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2454::RULES.keywords, deck_limit: authored::card_2454::RULES.deck_limit }, effect: authored::card_2454::RULES.checked_effect(), triggers: authored::card_2454::RULES.triggers, activated: authored::card_2454::RULES.activated, enters: authored::card_2454::RULES.enters, entry: authored::card_2454::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 1, 0], generic: 2 }, rules: &authored::card_2454::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Leyline Axe", oracle_id: Some("4f597675-0f6d-438c-990c-337171927a5e"), mana_cost: "{4}", type_line: "Artifact — Equipment", oracle_keywords: &["Equip"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2481::RULES.keywords, deck_limit: authored::card_2481::RULES.deck_limit }, effect: authored::card_2481::RULES.checked_effect(), triggers: authored::card_2481::RULES.triggers, activated: authored::card_2481::RULES.activated, enters: authored::card_2481::RULES.enters, entry: authored::card_2481::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 4 }, rules: &authored::card_2481::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Lightshell Duo", oracle_id: Some("ef8fab12-d7d2-4c21-8b8c-41627048ac8b"), mana_cost: "{3}{U}", type_line: "Creature — Rat Otter", oracle_keywords: &["Surveil", "Prowess"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2500::RULES.keywords, deck_limit: authored::card_2500::RULES.deck_limit }, effect: authored::card_2500::RULES.checked_effect(), triggers: authored::card_2500::RULES.triggers, activated: authored::card_2500::RULES.activated, enters: authored::card_2500::RULES.enters, entry: authored::card_2500::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_2500::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Liliana, Dreadhorde General", oracle_id: Some("5e958212-6a5b-4288-8d31-f1572619d7dc"), mana_cost: "{4}{B}{B}", type_line: "Legendary Planeswalker — Liliana", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2508::RULES.keywords, deck_limit: authored::card_2508::RULES.deck_limit }, effect: authored::card_2508::RULES.checked_effect(), triggers: authored::card_2508::RULES.triggers, activated: authored::card_2508::RULES.activated, enters: authored::card_2508::RULES.enters, entry: authored::card_2508::RULES.entry, kind: 64, basic: false, mana_color: None, colored: [0, 0, 2, 0, 0, 0], generic: 4 }, rules: &authored::card_2508::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Linden, the Steadfast Queen", oracle_id: Some("dbc56f3b-83b3-41c8-a8ae-d3774fe6ee01"), mana_cost: "{W}{W}{W}", type_line: "Legendary Creature — Human Noble", oracle_keywords: &["Vigilance"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2513::RULES.keywords, deck_limit: authored::card_2513::RULES.deck_limit }, effect: authored::card_2513::RULES.checked_effect(), triggers: authored::card_2513::RULES.triggers, activated: authored::card_2513::RULES.activated, enters: authored::card_2513::RULES.enters, entry: authored::card_2513::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [3, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_2513::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Loot, Exuberant Explorer", oracle_id: Some("f6476884-f73f-465f-8e7f-ea312a5b306d"), mana_cost: "{2}{G}", type_line: "Legendary Creature — Beast Noble", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2549::RULES.keywords, deck_limit: authored::card_2549::RULES.deck_limit }, effect: authored::card_2549::RULES.checked_effect(), triggers: authored::card_2549::RULES.triggers, activated: authored::card_2549::RULES.activated, enters: authored::card_2549::RULES.enters, entry: authored::card_2549::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 2 }, rules: &authored::card_2549::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Luminous Rebuke", oracle_id: Some("d96141af-313f-4ea9-a249-d9eb22cd2431"), mana_cost: "{4}{W}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2580::RULES.keywords, deck_limit: authored::card_2580::RULES.deck_limit }, effect: authored::card_2580::RULES.checked_effect(), triggers: authored::card_2580::RULES.triggers, activated: authored::card_2580::RULES.activated, enters: authored::card_2580::RULES.enters, entry: authored::card_2580::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 4 }, rules: &authored::card_2580::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Lunar Insight", oracle_id: Some("56b4a2f7-2d35-4d05-8d26-4f9f16c5f9f5"), mana_cost: "{2}{U}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2583::RULES.keywords, deck_limit: authored::card_2583::RULES.deck_limit }, effect: authored::card_2583::RULES.checked_effect(), triggers: authored::card_2583::RULES.triggers, activated: authored::card_2583::RULES.activated, enters: authored::card_2583::RULES.enters, entry: authored::card_2583::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_2583::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Lyra Dawnbringer", oracle_id: Some("592c91fc-6430-4c76-9460-65f047350f67"), mana_cost: "{3}{W}{W}", type_line: "Legendary Creature — Angel", oracle_keywords: &["Flying", "Lifelink", "First strike"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2591::RULES.keywords, deck_limit: authored::card_2591::RULES.deck_limit }, effect: authored::card_2591::RULES.checked_effect(), triggers: authored::card_2591::RULES.triggers, activated: authored::card_2591::RULES.activated, enters: authored::card_2591::RULES.enters, entry: authored::card_2591::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [2, 0, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_2591::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Maalfeld Twins", oracle_id: Some("dda4b515-49c0-43fe-9f6a-36defa326bb1"), mana_cost: "{5}{B}", type_line: "Creature — Zombie", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2597::RULES.keywords, deck_limit: authored::card_2597::RULES.deck_limit }, effect: authored::card_2597::RULES.checked_effect(), triggers: authored::card_2597::RULES.triggers, activated: authored::card_2597::RULES.activated, enters: authored::card_2597::RULES.enters, entry: authored::card_2597::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 5 }, rules: &authored::card_2597::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Macabre Waltz", oracle_id: Some("cf75dbd8-b65a-48d9-b96b-4afb43f336d1"), mana_cost: "{1}{B}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2603::RULES.keywords, deck_limit: authored::card_2603::RULES.deck_limit }, effect: authored::card_2603::RULES.checked_effect(), triggers: authored::card_2603::RULES.triggers, activated: authored::card_2603::RULES.activated, enters: authored::card_2603::RULES.enters, entry: authored::card_2603::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 1 }, rules: &authored::card_2603::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Magnigoth Sentry", oracle_id: Some("b52824f9-7897-46b7-a4e5-a0261861cb5b"), mana_cost: "{3}{G}", type_line: "Creature — Treefolk", oracle_keywords: &["Reach"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2628::RULES.keywords, deck_limit: authored::card_2628::RULES.deck_limit }, effect: authored::card_2628::RULES.checked_effect(), triggers: authored::card_2628::RULES.triggers, activated: authored::card_2628::RULES.activated, enters: authored::card_2628::RULES.enters, entry: authored::card_2628::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 3 }, rules: &authored::card_2628::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Make a Stand", oracle_id: Some("531f78d5-5004-4b02-99c7-b390cb342fd9"), mana_cost: "{2}{W}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2634::RULES.keywords, deck_limit: authored::card_2634::RULES.deck_limit }, effect: authored::card_2634::RULES.checked_effect(), triggers: authored::card_2634::RULES.triggers, activated: authored::card_2634::RULES.activated, enters: authored::card_2634::RULES.enters, entry: authored::card_2634::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_2634::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Marauding Blight-Priest", oracle_id: Some("814b87fe-2a75-4ff2-8637-7e69e3fb285b"), mana_cost: "{2}{B}", type_line: "Creature — Vampire Cleric", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2655::RULES.keywords, deck_limit: authored::card_2655::RULES.deck_limit }, effect: authored::card_2655::RULES.checked_effect(), triggers: authored::card_2655::RULES.triggers, activated: authored::card_2655::RULES.activated, enters: authored::card_2655::RULES.enters, entry: authored::card_2655::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 2 }, rules: &authored::card_2655::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Massacre Wurm", oracle_id: Some("93cf50cf-0ecc-4d3e-abea-778c1ebacec4"), mana_cost: "{3}{B}{B}{B}", type_line: "Creature — Phyrexian Wurm", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2678::RULES.keywords, deck_limit: authored::card_2678::RULES.deck_limit }, effect: authored::card_2678::RULES.checked_effect(), triggers: authored::card_2678::RULES.triggers, activated: authored::card_2678::RULES.activated, enters: authored::card_2678::RULES.enters, entry: authored::card_2678::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 3, 0, 0, 0], generic: 3 }, rules: &authored::card_2678::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Maze's End", oracle_id: Some("49479778-c4c0-43ba-a7b7-45f00d067462"), mana_cost: "", type_line: "Land", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2691::RULES.keywords, deck_limit: authored::card_2691::RULES.deck_limit }, effect: authored::card_2691::RULES.checked_effect(), triggers: authored::card_2691::RULES.triggers, activated: authored::card_2691::RULES.activated, enters: authored::card_2691::RULES.enters, entry: authored::card_2691::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_2691::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Mazemind Tome", oracle_id: Some("800fb917-8898-4eba-9f94-aec6e9d75236"), mana_cost: "{2}", type_line: "Artifact — Book", oracle_keywords: &["Scry"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2692::RULES.keywords, deck_limit: authored::card_2692::RULES.deck_limit }, effect: authored::card_2692::RULES.checked_effect(), triggers: authored::card_2692::RULES.triggers, activated: authored::card_2692::RULES.activated, enters: authored::card_2692::RULES.enters, entry: authored::card_2692::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_2692::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Micromancer", oracle_id: Some("d2605782-09f1-4d52-b3e7-5ec1442bd9b5"), mana_cost: "{3}{U}", type_line: "Creature — Human Wizard", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2743::RULES.keywords, deck_limit: authored::card_2743::RULES.deck_limit }, effect: authored::card_2743::RULES.checked_effect(), triggers: authored::card_2743::RULES.triggers, activated: authored::card_2743::RULES.activated, enters: authored::card_2743::RULES.enters, entry: authored::card_2743::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_2743::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Midnight Reaper", oracle_id: Some("e8c7566d-7cc0-48af-a986-83223ec7e06c"), mana_cost: "{2}{B}", type_line: "Creature — Zombie Knight", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2747::RULES.keywords, deck_limit: authored::card_2747::RULES.deck_limit }, effect: authored::card_2747::RULES.checked_effect(), triggers: authored::card_2747::RULES.triggers, activated: authored::card_2747::RULES.activated, enters: authored::card_2747::RULES.enters, entry: authored::card_2747::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 2 }, rules: &authored::card_2747::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Midnight Snack", oracle_id: Some("3ddcaee9-a11e-477a-bd19-f5ee10512549"), mana_cost: "{2}{B}", type_line: "Enchantment", oracle_keywords: &["Food", "Raid"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2748::RULES.keywords, deck_limit: authored::card_2748::RULES.deck_limit }, effect: authored::card_2748::RULES.checked_effect(), triggers: authored::card_2748::RULES.triggers, activated: authored::card_2748::RULES.activated, enters: authored::card_2748::RULES.enters, entry: authored::card_2748::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 2 }, rules: &authored::card_2748::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Mild-Mannered Librarian", oracle_id: Some("fb7070ab-c234-4d54-8cdc-caf106bf24a6"), mana_cost: "{G}", type_line: "Creature — Human", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2757::RULES.keywords, deck_limit: authored::card_2757::RULES.deck_limit }, effect: authored::card_2757::RULES.checked_effect(), triggers: authored::card_2757::RULES.triggers, activated: authored::card_2757::RULES.activated, enters: authored::card_2757::RULES.enters, entry: authored::card_2757::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 0 }, rules: &authored::card_2757::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Mindsparker", oracle_id: Some("54866c25-b332-46c8-a2c5-1a71b702ddc0"), mana_cost: "{1}{R}{R}", type_line: "Creature — Elemental", oracle_keywords: &["First strike"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2769::RULES.keywords, deck_limit: authored::card_2769::RULES.deck_limit }, effect: authored::card_2769::RULES.checked_effect(), triggers: authored::card_2769::RULES.triggers, activated: authored::card_2769::RULES.activated, enters: authored::card_2769::RULES.enters, entry: authored::card_2769::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 2, 0, 0], generic: 1 }, rules: &authored::card_2769::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Mischievous Mystic", oracle_id: Some("542831c6-6279-4977-9b29-0fb9c0ebf04d"), mana_cost: "{1}{U}", type_line: "Creature — Human Wizard", oracle_keywords: &["Flying"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2787::RULES.keywords, deck_limit: authored::card_2787::RULES.deck_limit }, effect: authored::card_2787::RULES.checked_effect(), triggers: authored::card_2787::RULES.triggers, activated: authored::card_2787::RULES.activated, enters: authored::card_2787::RULES.enters, entry: authored::card_2787::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_2787::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Mischievous Pup", oracle_id: Some("e75b187d-0e3b-4d94-a940-8b7e4e7ed1ca"), mana_cost: "{2}{W}", type_line: "Creature — Dog", oracle_keywords: &["Flash"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2788::RULES.keywords, deck_limit: authored::card_2788::RULES.deck_limit }, effect: authored::card_2788::RULES.checked_effect(), triggers: authored::card_2788::RULES.triggers, activated: authored::card_2788::RULES.activated, enters: authored::card_2788::RULES.enters, entry: authored::card_2788::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_2788::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Mocking Sprite", oracle_id: Some("013ca531-1ff7-4efc-a280-4bc3ab574072"), mana_cost: "{2}{U}", type_line: "Creature — Faerie Rogue", oracle_keywords: &["Flying"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2806::RULES.keywords, deck_limit: authored::card_2806::RULES.deck_limit }, effect: authored::card_2806::RULES.checked_effect(), triggers: authored::card_2806::RULES.triggers, activated: authored::card_2806::RULES.activated, enters: authored::card_2806::RULES.enters, entry: authored::card_2806::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_2806::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Mold Adder", oracle_id: Some("2091eb7e-2357-4ef8-b15e-1e6f82176340"), mana_cost: "{G}", type_line: "Creature — Fungus Snake", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2809::RULES.keywords, deck_limit: authored::card_2809::RULES.deck_limit }, effect: authored::card_2809::RULES.checked_effect(), triggers: authored::card_2809::RULES.triggers, activated: authored::card_2809::RULES.activated, enters: authored::card_2809::RULES.enters, entry: authored::card_2809::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 0 }, rules: &authored::card_2809::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Mortify", oracle_id: Some("faa01ed1-ccfa-4e58-951f-cd81f9068027"), mana_cost: "{1}{W}{B}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2856::RULES.keywords, deck_limit: authored::card_2856::RULES.deck_limit }, effect: authored::card_2856::RULES.checked_effect(), triggers: authored::card_2856::RULES.triggers, activated: authored::card_2856::RULES.activated, enters: authored::card_2856::RULES.enters, entry: authored::card_2856::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [1, 0, 1, 0, 0, 0], generic: 1 }, rules: &authored::card_2856::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Mossborn Hydra", oracle_id: Some("84122d81-9634-4d3f-85a5-f6cf4303691a"), mana_cost: "{2}{G}", type_line: "Creature — Elemental Hydra", oracle_keywords: &["Trample", "Double", "Landfall"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2858::RULES.keywords, deck_limit: authored::card_2858::RULES.deck_limit }, effect: authored::card_2858::RULES.checked_effect(), triggers: authored::card_2858::RULES.triggers, activated: authored::card_2858::RULES.activated, enters: authored::card_2858::RULES.enters, entry: authored::card_2858::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 2 }, rules: &authored::card_2858::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Myojin of Night's Reach", oracle_id: Some("5ac65688-0a74-4dc4-b6cb-12bdf8aed01e"), mana_cost: "{5}{B}{B}{B}", type_line: "Legendary Creature — Spirit", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2894::RULES.keywords, deck_limit: authored::card_2894::RULES.deck_limit }, effect: authored::card_2894::RULES.checked_effect(), triggers: authored::card_2894::RULES.triggers, activated: authored::card_2894::RULES.activated, enters: authored::card_2894::RULES.enters, entry: authored::card_2894::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 3, 0, 0, 0], generic: 5 }, rules: &authored::card_2894::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Mystic Archaeologist", oracle_id: Some("b77dfe2a-ebc9-46b0-9134-2ecb2abdd8be"), mana_cost: "{1}{U}", type_line: "Creature — Human Wizard", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2898::RULES.keywords, deck_limit: authored::card_2898::RULES.deck_limit }, effect: authored::card_2898::RULES.checked_effect(), triggers: authored::card_2898::RULES.triggers, activated: authored::card_2898::RULES.activated, enters: authored::card_2898::RULES.enters, entry: authored::card_2898::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_2898::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Mystical Teachings", oracle_id: Some("81d72511-88b4-45ec-a5a9-e722caf55577"), mana_cost: "{3}{U}", type_line: "Instant", oracle_keywords: &["Flashback"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2900::RULES.keywords, deck_limit: authored::card_2900::RULES.deck_limit }, effect: authored::card_2900::RULES.checked_effect(), triggers: authored::card_2900::RULES.triggers, activated: authored::card_2900::RULES.activated, enters: authored::card_2900::RULES.enters, entry: authored::card_2900::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_2900::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Needletooth Pack", oracle_id: Some("ac68fe30-a4a5-45f6-8782-fed57718c42e"), mana_cost: "{3}{G}{G}", type_line: "Creature — Dinosaur", oracle_keywords: &["Morbid"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2913::RULES.keywords, deck_limit: authored::card_2913::RULES.deck_limit }, effect: authored::card_2913::RULES.checked_effect(), triggers: authored::card_2913::RULES.triggers, activated: authored::card_2913::RULES.activated, enters: authored::card_2913::RULES.enters, entry: authored::card_2913::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 2, 0], generic: 3 }, rules: &authored::card_2913::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Nessian Hornbeetle", oracle_id: Some("ab097f46-a7bf-4cf1-a96c-36b681fe79da"), mana_cost: "{1}{G}", type_line: "Creature — Insect", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2919::RULES.keywords, deck_limit: authored::card_2919::RULES.deck_limit }, effect: authored::card_2919::RULES.checked_effect(), triggers: authored::card_2919::RULES.triggers, activated: authored::card_2919::RULES.activated, enters: authored::card_2919::RULES.enters, entry: authored::card_2919::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 1 }, rules: &authored::card_2919::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "New Horizons", oracle_id: Some("8834d3c9-cdb5-423b-bc78-94a94add5e74"), mana_cost: "{2}{G}", type_line: "Enchantment — Aura", oracle_keywords: &["Enchant"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2925::RULES.keywords, deck_limit: authored::card_2925::RULES.deck_limit }, effect: authored::card_2925::RULES.checked_effect(), triggers: authored::card_2925::RULES.triggers, activated: authored::card_2925::RULES.activated, enters: authored::card_2925::RULES.enters, entry: authored::card_2925::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 2 }, rules: &authored::card_2925::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Nine-Lives Familiar", oracle_id: Some("5049da27-f38b-4fe6-a387-51510bd1f44e"), mana_cost: "{1}{B}{B}", type_line: "Creature — Cat", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2944::RULES.keywords, deck_limit: authored::card_2944::RULES.deck_limit }, effect: authored::card_2944::RULES.checked_effect(), triggers: authored::card_2944::RULES.triggers, activated: authored::card_2944::RULES.activated, enters: authored::card_2944::RULES.enters, entry: authored::card_2944::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 2, 0, 0, 0], generic: 1 }, rules: &authored::card_2944::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Niv-Mizzet, Visionary", oracle_id: Some("d9f1dc61-831e-4244-a2da-aa9da6fd5b50"), mana_cost: "{4}{U}{R}", type_line: "Legendary Creature — Dragon Wizard", oracle_keywords: &["Flying"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_2950::RULES.keywords, deck_limit: authored::card_2950::RULES.deck_limit }, effect: authored::card_2950::RULES.checked_effect(), triggers: authored::card_2950::RULES.triggers, activated: authored::card_2950::RULES.activated, enters: authored::card_2950::RULES.enters, entry: authored::card_2950::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 1, 0, 0], generic: 4 }, rules: &authored::card_2950::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Nullpriest of Oblivion", oracle_id: Some("c1111228-ef88-4b64-91e0-66ee32a1f5e4"), mana_cost: "{1}{B}", type_line: "Creature — Vampire Cleric", oracle_keywords: &["Lifelink", "Menace", "Kicker"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2977::RULES.keywords, deck_limit: authored::card_2977::RULES.deck_limit }, effect: authored::card_2977::RULES.checked_effect(), triggers: authored::card_2977::RULES.triggers, activated: authored::card_2977::RULES.activated, enters: authored::card_2977::RULES.enters, entry: authored::card_2977::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 1 }, rules: &authored::card_2977::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Obliterating Bolt", oracle_id: Some("57fe941c-a830-4570-afe2-18f93c7a7b84"), mana_cost: "{1}{R}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_2985::RULES.keywords, deck_limit: authored::card_2985::RULES.deck_limit }, effect: authored::card_2985::RULES.checked_effect(), triggers: authored::card_2985::RULES.triggers, activated: authored::card_2985::RULES.activated, enters: authored::card_2985::RULES.enters, entry: authored::card_2985::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 1 }, rules: &authored::card_2985::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Omniscience", oracle_id: Some("730e39e6-c61d-48b5-8827-bfd952bf1be7"), mana_cost: "{7}{U}{U}{U}", type_line: "Enchantment", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3020::RULES.keywords, deck_limit: authored::card_3020::RULES.deck_limit }, effect: authored::card_3020::RULES.checked_effect(), triggers: authored::card_3020::RULES.triggers, activated: authored::card_3020::RULES.activated, enters: authored::card_3020::RULES.enters, entry: authored::card_3020::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 3, 0, 0, 0, 0], generic: 7 }, rules: &authored::card_3020::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Ordeal of Nylea", oracle_id: Some("5fff2bf4-9e26-4815-9bf6-672ed7e55b96"), mana_cost: "{1}{G}", type_line: "Enchantment — Aura", oracle_keywords: &["Enchant"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3034::RULES.keywords, deck_limit: authored::card_3034::RULES.deck_limit }, effect: authored::card_3034::RULES.checked_effect(), triggers: authored::card_3034::RULES.triggers, activated: authored::card_3034::RULES.activated, enters: authored::card_3034::RULES.enters, entry: authored::card_3034::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 1 }, rules: &authored::card_3034::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Orzhov Guildgate", oracle_id: Some("57b37df5-fee4-4720-931f-f0cb0a8b338c"), mana_cost: "", type_line: "Land — Gate", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3045::RULES.keywords, deck_limit: authored::card_3045::RULES.deck_limit }, effect: authored::card_3045::RULES.checked_effect(), triggers: authored::card_3045::RULES.triggers, activated: authored::card_3045::RULES.activated, enters: authored::card_3045::RULES.enters, entry: authored::card_3045::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_3045::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Overrun", oracle_id: Some("204f9afe-c20b-4933-b5cd-aa572784762a"), mana_cost: "{2}{G}{G}{G}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3076::RULES.keywords, deck_limit: authored::card_3076::RULES.deck_limit }, effect: authored::card_3076::RULES.checked_effect(), triggers: authored::card_3076::RULES.triggers, activated: authored::card_3076::RULES.activated, enters: authored::card_3076::RULES.enters, entry: authored::card_3076::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 0, 0, 3, 0], generic: 2 }, rules: &authored::card_3076::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Ovika, Enigma Goliath", oracle_id: Some("b05a2b5f-1b5c-4f1b-8a6b-525c5c8b8ba1"), mana_cost: "{5}{U}{R}", type_line: "Legendary Creature — Phyrexian Nightmare", oracle_keywords: &["Flying", "Ward"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_3080::RULES.keywords, deck_limit: authored::card_3080::RULES.deck_limit }, effect: authored::card_3080::RULES.checked_effect(), triggers: authored::card_3080::RULES.triggers, activated: authored::card_3080::RULES.activated, enters: authored::card_3080::RULES.enters, entry: authored::card_3080::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 1, 0, 0], generic: 5 }, rules: &authored::card_3080::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Painful Quandary", oracle_id: Some("c37051cc-6683-4dbb-b5ff-5c3a5bdab1df"), mana_cost: "{3}{B}{B}", type_line: "Enchantment", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3091::RULES.keywords, deck_limit: authored::card_3091::RULES.deck_limit }, effect: authored::card_3091::RULES.checked_effect(), triggers: authored::card_3091::RULES.triggers, activated: authored::card_3091::RULES.activated, enters: authored::card_3091::RULES.enters, entry: authored::card_3091::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 0, 2, 0, 0, 0], generic: 3 }, rules: &authored::card_3091::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Pelakka Wurm", oracle_id: Some("d36075c2-de66-4202-9217-b1102a2bc14b"), mana_cost: "{4}{G}{G}{G}", type_line: "Creature — Wurm", oracle_keywords: &["Trample"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3123::RULES.keywords, deck_limit: authored::card_3123::RULES.deck_limit }, effect: authored::card_3123::RULES.checked_effect(), triggers: authored::card_3123::RULES.triggers, activated: authored::card_3123::RULES.activated, enters: authored::card_3123::RULES.enters, entry: authored::card_3123::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 3, 0], generic: 4 }, rules: &authored::card_3123::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Perforating Artist", oracle_id: Some("a3643f66-dfa2-4275-a093-23e162496354"), mana_cost: "{1}{B}{R}", type_line: "Creature — Devil", oracle_keywords: &["Raid", "Deathtouch"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3128::RULES.keywords, deck_limit: authored::card_3128::RULES.deck_limit }, effect: authored::card_3128::RULES.checked_effect(), triggers: authored::card_3128::RULES.triggers, activated: authored::card_3128::RULES.activated, enters: authored::card_3128::RULES.enters, entry: authored::card_3128::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 1, 0, 0], generic: 1 }, rules: &authored::card_3128::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Phyrexian Arena", oracle_id: Some("ee579a32-a048-4335-b966-231ba731cdea"), mana_cost: "{1}{B}{B}", type_line: "Enchantment", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3151::RULES.keywords, deck_limit: authored::card_3151::RULES.deck_limit }, effect: authored::card_3151::RULES.checked_effect(), triggers: authored::card_3151::RULES.triggers, activated: authored::card_3151::RULES.activated, enters: authored::card_3151::RULES.enters, entry: authored::card_3151::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 0, 2, 0, 0, 0], generic: 1 }, rules: &authored::card_3151::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Pilfer", oracle_id: Some("d13f0907-de39-4a90-940a-831740d7aa9b"), mana_cost: "{1}{B}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3162::RULES.keywords, deck_limit: authored::card_3162::RULES.deck_limit }, effect: authored::card_3162::RULES.checked_effect(), triggers: authored::card_3162::RULES.triggers, activated: authored::card_3162::RULES.activated, enters: authored::card_3162::RULES.enters, entry: authored::card_3162::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 1 }, rules: &authored::card_3162::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Pirate's Cutlass", oracle_id: Some("ac77591c-39d7-4fd1-a6e9-805f57c6614a"), mana_cost: "{3}", type_line: "Artifact — Equipment", oracle_keywords: &["Equip"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3172::RULES.keywords, deck_limit: authored::card_3172::RULES.deck_limit }, effect: authored::card_3172::RULES.checked_effect(), triggers: authored::card_3172::RULES.triggers, activated: authored::card_3172::RULES.activated, enters: authored::card_3172::RULES.enters, entry: authored::card_3172::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_3172::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Prayer of Binding", oracle_id: Some("4505ff9f-97f1-45d1-aea9-5590fefb4a4d"), mana_cost: "{3}{W}", type_line: "Enchantment", oracle_keywords: &["Flash"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3217::RULES.keywords, deck_limit: authored::card_3217::RULES.deck_limit }, effect: authored::card_3217::RULES.checked_effect(), triggers: authored::card_3217::RULES.triggers, activated: authored::card_3217::RULES.activated, enters: authored::card_3217::RULES.enters, entry: authored::card_3217::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_3217::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Predator Ooze", oracle_id: Some("a51ec70d-1356-4403-a858-b99445bac54a"), mana_cost: "{G}{G}{G}", type_line: "Creature — Ooze", oracle_keywords: &["Indestructible"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3220::RULES.keywords, deck_limit: authored::card_3220::RULES.deck_limit }, effect: authored::card_3220::RULES.checked_effect(), triggers: authored::card_3220::RULES.triggers, activated: authored::card_3220::RULES.activated, enters: authored::card_3220::RULES.enters, entry: authored::card_3220::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 3, 0], generic: 0 }, rules: &authored::card_3220::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Preposterous Proportions", oracle_id: Some("1e03aad2-6426-4d18-a0e5-bc3d8e6b7b11"), mana_cost: "{5}{G}{G}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3223::RULES.keywords, deck_limit: authored::card_3223::RULES.deck_limit }, effect: authored::card_3223::RULES.checked_effect(), triggers: authored::card_3223::RULES.triggers, activated: authored::card_3223::RULES.activated, enters: authored::card_3223::RULES.enters, entry: authored::card_3223::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 0, 0, 2, 0], generic: 5 }, rules: &authored::card_3223::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Prideful Parent", oracle_id: Some("a90d1d35-6d67-43b3-8256-b0318b3a4b07"), mana_cost: "{2}{W}", type_line: "Creature — Cat", oracle_keywords: &["Vigilance"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3230::RULES.keywords, deck_limit: authored::card_3230::RULES.deck_limit }, effect: authored::card_3230::RULES.checked_effect(), triggers: authored::card_3230::RULES.triggers, activated: authored::card_3230::RULES.activated, enters: authored::card_3230::RULES.enters, entry: authored::card_3230::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_3230::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Primal Might", oracle_id: Some("58d7b8b3-ac47-4805-8259-4ba8e70d7dc3"), mana_cost: "{X}{G}", type_line: "Sorcery", oracle_keywords: &["Fight"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3231::RULES.keywords, deck_limit: authored::card_3231::RULES.deck_limit }, effect: authored::card_3231::RULES.checked_effect(), triggers: authored::card_3231::RULES.triggers, activated: authored::card_3231::RULES.activated, enters: authored::card_3231::RULES.enters, entry: authored::card_3231::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 0 }, rules: &authored::card_3231::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Prime Speaker Zegana", oracle_id: Some("311e9368-696a-47e7-aa2f-3ef1b1a92e2b"), mana_cost: "{2}{G}{G}{U}{U}", type_line: "Legendary Creature — Merfolk Wizard", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_3234::RULES.keywords, deck_limit: authored::card_3234::RULES.deck_limit }, effect: authored::card_3234::RULES.checked_effect(), triggers: authored::card_3234::RULES.triggers, activated: authored::card_3234::RULES.activated, enters: authored::card_3234::RULES.enters, entry: authored::card_3234::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 2, 0, 0, 2, 0], generic: 2 }, rules: &authored::card_3234::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Primeval Bounty", oracle_id: Some("f633c16d-d943-421c-ad18-af35db9ec9fc"), mana_cost: "{5}{G}", type_line: "Enchantment", oracle_keywords: &["Landfall"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3235::RULES.keywords, deck_limit: authored::card_3235::RULES.deck_limit }, effect: authored::card_3235::RULES.checked_effect(), triggers: authored::card_3235::RULES.triggers, activated: authored::card_3235::RULES.activated, enters: authored::card_3235::RULES.enters, entry: authored::card_3235::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 5 }, rules: &authored::card_3235::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Progenitus", oracle_id: Some("2d7e00b6-12f0-4b03-82a6-50e3d1b5395e"), mana_cost: "{W}{W}{U}{U}{B}{B}{R}{R}{G}{G}", type_line: "Legendary Creature — Hydra Avatar", oracle_keywords: &["Protection"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_3254::RULES.keywords, deck_limit: authored::card_3254::RULES.deck_limit }, effect: authored::card_3254::RULES.checked_effect(), triggers: authored::card_3254::RULES.triggers, activated: authored::card_3254::RULES.activated, enters: authored::card_3254::RULES.enters, entry: authored::card_3254::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [2, 2, 2, 2, 2, 0], generic: 0 }, rules: &authored::card_3254::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Pulse Tracker", oracle_id: Some("f916f689-a314-4952-8107-41c4c2f2e811"), mana_cost: "{B}", type_line: "Creature — Vampire Rogue", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3278::RULES.keywords, deck_limit: authored::card_3278::RULES.deck_limit }, effect: authored::card_3278::RULES.checked_effect(), triggers: authored::card_3278::RULES.triggers, activated: authored::card_3278::RULES.activated, enters: authored::card_3278::RULES.enters, entry: authored::card_3278::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 0 }, rules: &authored::card_3278::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Pyromancer's Goggles", oracle_id: Some("f76bcbfe-483f-4e63-8425-76feca1abf3e"), mana_cost: "{5}", type_line: "Legendary Artifact", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_3295::RULES.keywords, deck_limit: authored::card_3295::RULES.deck_limit }, effect: authored::card_3295::RULES.checked_effect(), triggers: authored::card_3295::RULES.triggers, activated: authored::card_3295::RULES.activated, enters: authored::card_3295::RULES.enters, entry: authored::card_3295::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 5 }, rules: &authored::card_3295::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Quakestrider Ceratops", oracle_id: Some("e56629ba-82d4-470f-9a6b-40c29d835af2"), mana_cost: "{3}{G}{G}{G}", type_line: "Creature — Dinosaur", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3302::RULES.keywords, deck_limit: authored::card_3302::RULES.deck_limit }, effect: authored::card_3302::RULES.checked_effect(), triggers: authored::card_3302::RULES.triggers, activated: authored::card_3302::RULES.activated, enters: authored::card_3302::RULES.enters, entry: authored::card_3302::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 3, 0], generic: 3 }, rules: &authored::card_3302::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Quick-Draw Katana", oracle_id: Some("604ef241-df8f-48f8-8680-f9ed77ff30ae"), mana_cost: "{2}", type_line: "Artifact — Equipment", oracle_keywords: &["Equip"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3313::RULES.keywords, deck_limit: authored::card_3313::RULES.deck_limit }, effect: authored::card_3313::RULES.checked_effect(), triggers: authored::card_3313::RULES.triggers, activated: authored::card_3313::RULES.activated, enters: authored::card_3313::RULES.enters, entry: authored::card_3313::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_3313::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Quilled Greatwurm", oracle_id: Some("01635d21-aeda-4754-9d94-68fcd0cd6556"), mana_cost: "{4}{G}{G}", type_line: "Creature — Wurm", oracle_keywords: &["Trample"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3318::RULES.keywords, deck_limit: authored::card_3318::RULES.deck_limit }, effect: authored::card_3318::RULES.checked_effect(), triggers: authored::card_3318::RULES.triggers, activated: authored::card_3318::RULES.activated, enters: authored::card_3318::RULES.enters, entry: authored::card_3318::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 2, 0], generic: 4 }, rules: &authored::card_3318::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Raging Redcap", oracle_id: Some("679841ad-f3ac-48e2-ba11-2cf6d78642b4"), mana_cost: "{2}{R}", type_line: "Creature — Goblin Knight", oracle_keywords: &["Double strike"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3343::RULES.keywords, deck_limit: authored::card_3343::RULES.deck_limit }, effect: authored::card_3343::RULES.checked_effect(), triggers: authored::card_3343::RULES.triggers, activated: authored::card_3343::RULES.activated, enters: authored::card_3343::RULES.enters, entry: authored::card_3343::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 2 }, rules: &authored::card_3343::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Raise the Past", oracle_id: Some("a69a24d0-ca58-4a44-8af1-a3bd1608d2f9"), mana_cost: "{2}{W}{W}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3349::RULES.keywords, deck_limit: authored::card_3349::RULES.deck_limit }, effect: authored::card_3349::RULES.checked_effect(), triggers: authored::card_3349::RULES.triggers, activated: authored::card_3349::RULES.activated, enters: authored::card_3349::RULES.enters, entry: authored::card_3349::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [2, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_3349::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Rakdos Guildgate", oracle_id: Some("361f534b-39d1-4421-b5a8-d3813c62f86d"), mana_cost: "", type_line: "Land — Gate", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3350::RULES.keywords, deck_limit: authored::card_3350::RULES.deck_limit }, effect: authored::card_3350::RULES.checked_effect(), triggers: authored::card_3350::RULES.triggers, activated: authored::card_3350::RULES.activated, enters: authored::card_3350::RULES.enters, entry: authored::card_3350::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_3350::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Ramos, Dragon Engine", oracle_id: Some("3ed41d2d-211b-4013-8562-8c64d54cc43a"), mana_cost: "{6}", type_line: "Legendary Artifact Creature — Dragon", oracle_keywords: &["Flying"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_3361::RULES.keywords, deck_limit: authored::card_3361::RULES.deck_limit }, effect: authored::card_3361::RULES.checked_effect(), triggers: authored::card_3361::RULES.triggers, activated: authored::card_3361::RULES.activated, enters: authored::card_3361::RULES.enters, entry: authored::card_3361::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 6 }, rules: &authored::card_3361::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Ravenous Amulet", oracle_id: Some("87424e20-00ab-4f6d-a905-1955897b358e"), mana_cost: "{2}", type_line: "Artifact", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3398::RULES.keywords, deck_limit: authored::card_3398::RULES.deck_limit }, effect: authored::card_3398::RULES.checked_effect(), triggers: authored::card_3398::RULES.triggers, activated: authored::card_3398::RULES.activated, enters: authored::card_3398::RULES.enters, entry: authored::card_3398::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_3398::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Ravenous Giant", oracle_id: Some("46c7a552-4a98-4e1a-8651-8fb0e7153aec"), mana_cost: "{2}{R}{R}", type_line: "Creature — Giant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3399::RULES.keywords, deck_limit: authored::card_3399::RULES.deck_limit }, effect: authored::card_3399::RULES.checked_effect(), triggers: authored::card_3399::RULES.triggers, activated: authored::card_3399::RULES.activated, enters: authored::card_3399::RULES.enters, entry: authored::card_3399::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 2, 0, 0], generic: 2 }, rules: &authored::card_3399::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Reclamation Sage", oracle_id: Some("032ec6e2-6cc3-4a97-9cc7-3233f5e11904"), mana_cost: "{2}{G}", type_line: "Creature — Elf Shaman", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3424::RULES.keywords, deck_limit: authored::card_3424::RULES.deck_limit }, effect: authored::card_3424::RULES.checked_effect(), triggers: authored::card_3424::RULES.triggers, activated: authored::card_3424::RULES.activated, enters: authored::card_3424::RULES.enters, entry: authored::card_3424::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 2 }, rules: &authored::card_3424::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Redcap Gutter-Dweller", oracle_id: Some("ada48353-4c86-44d6-83a5-e7e3c703b948"), mana_cost: "{2}{R}{R}", type_line: "Creature — Goblin Warrior", oracle_keywords: &["Menace"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3432::RULES.keywords, deck_limit: authored::card_3432::RULES.deck_limit }, effect: authored::card_3432::RULES.checked_effect(), triggers: authored::card_3432::RULES.triggers, activated: authored::card_3432::RULES.activated, enters: authored::card_3432::RULES.enters, entry: authored::card_3432::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 2, 0, 0], generic: 2 }, rules: &authored::card_3432::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Refute", oracle_id: Some("eb1dfa29-7371-4cb6-bfa2-16f7820b69be"), mana_cost: "{1}{U}{U}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3442::RULES.keywords, deck_limit: authored::card_3442::RULES.deck_limit }, effect: authored::card_3442::RULES.checked_effect(), triggers: authored::card_3442::RULES.triggers, activated: authored::card_3442::RULES.activated, enters: authored::card_3442::RULES.enters, entry: authored::card_3442::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 2, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_3442::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Regal Caracal", oracle_id: Some("aa571676-b390-4b8a-baea-4c4cd60c01f6"), mana_cost: "{3}{W}{W}", type_line: "Creature — Cat", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3445::RULES.keywords, deck_limit: authored::card_3445::RULES.deck_limit }, effect: authored::card_3445::RULES.checked_effect(), triggers: authored::card_3445::RULES.triggers, activated: authored::card_3445::RULES.activated, enters: authored::card_3445::RULES.enters, entry: authored::card_3445::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [2, 0, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_3445::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Resolute Reinforcements", oracle_id: Some("eaaed253-9b11-47e2-8624-0340ab2207f6"), mana_cost: "{1}{W}", type_line: "Creature — Human Soldier", oracle_keywords: &["Flash"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3479::RULES.keywords, deck_limit: authored::card_3479::RULES.deck_limit }, effect: authored::card_3479::RULES.checked_effect(), triggers: authored::card_3479::RULES.triggers, activated: authored::card_3479::RULES.activated, enters: authored::card_3479::RULES.enters, entry: authored::card_3479::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_3479::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Revenge of the Rats", oracle_id: Some("29e3a3d9-4839-44d0-ad48-159c01645a6a"), mana_cost: "{2}{B}{B}", type_line: "Sorcery", oracle_keywords: &["Flashback"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3507::RULES.keywords, deck_limit: authored::card_3507::RULES.deck_limit }, effect: authored::card_3507::RULES.checked_effect(), triggers: authored::card_3507::RULES.triggers, activated: authored::card_3507::RULES.activated, enters: authored::card_3507::RULES.enters, entry: authored::card_3507::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 2, 0, 0, 0], generic: 2 }, rules: &authored::card_3507::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Rise of the Dark Realms", oracle_id: Some("e5223a09-f732-4747-8914-e6546ab0ef4c"), mana_cost: "{7}{B}{B}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3538::RULES.keywords, deck_limit: authored::card_3538::RULES.deck_limit }, effect: authored::card_3538::RULES.checked_effect(), triggers: authored::card_3538::RULES.triggers, activated: authored::card_3538::RULES.activated, enters: authored::card_3538::RULES.enters, entry: authored::card_3538::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 2, 0, 0, 0], generic: 7 }, rules: &authored::card_3538::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Rite of the Dragoncaller", oracle_id: Some("5097f4e6-50af-4641-909f-db44abf0ce32"), mana_cost: "{4}{R}{R}", type_line: "Enchantment", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3546::RULES.keywords, deck_limit: authored::card_3546::RULES.deck_limit }, effect: authored::card_3546::RULES.checked_effect(), triggers: authored::card_3546::RULES.triggers, activated: authored::card_3546::RULES.activated, enters: authored::card_3546::RULES.enters, entry: authored::card_3546::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 0, 0, 2, 0, 0], generic: 4 }, rules: &authored::card_3546::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "River's Rebuke", oracle_id: Some("c52cfb41-18f3-4e73-b5e7-d75baf74e578"), mana_cost: "{4}{U}{U}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3550::RULES.keywords, deck_limit: authored::card_3550::RULES.deck_limit }, effect: authored::card_3550::RULES.checked_effect(), triggers: authored::card_3550::RULES.triggers, activated: authored::card_3550::RULES.activated, enters: authored::card_3550::RULES.enters, entry: authored::card_3550::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 2, 0, 0, 0, 0], generic: 4 }, rules: &authored::card_3550::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Ruby, Daring Tracker", oracle_id: Some("5d1b0eee-3a7a-4f22-a40d-7658a368962a"), mana_cost: "{R}{G}", type_line: "Legendary Creature — Human Scout", oracle_keywords: &["Haste"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_3609::RULES.keywords, deck_limit: authored::card_3609::RULES.deck_limit }, effect: authored::card_3609::RULES.checked_effect(), triggers: authored::card_3609::RULES.triggers, activated: authored::card_3609::RULES.activated, enters: authored::card_3609::RULES.enters, entry: authored::card_3609::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 1, 0], generic: 0 }, rules: &authored::card_3609::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Rune-Scarred Demon", oracle_id: Some("14aefe99-fb60-4f1c-a71f-7ffbe94c8b13"), mana_cost: "{5}{B}{B}", type_line: "Creature — Demon", oracle_keywords: &["Flying"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3622::RULES.keywords, deck_limit: authored::card_3622::RULES.deck_limit }, effect: authored::card_3622::RULES.checked_effect(), triggers: authored::card_3622::RULES.triggers, activated: authored::card_3622::RULES.activated, enters: authored::card_3622::RULES.enters, entry: authored::card_3622::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 2, 0, 0, 0], generic: 5 }, rules: &authored::card_3622::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Rune-Sealed Wall", oracle_id: Some("74d31e99-ab48-483a-a62e-5b424b27d017"), mana_cost: "{2}{U}", type_line: "Artifact Creature — Wall", oracle_keywords: &["Surveil", "Defender"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3623::RULES.keywords, deck_limit: authored::card_3623::RULES.deck_limit }, effect: authored::card_3623::RULES.checked_effect(), triggers: authored::card_3623::RULES.triggers, activated: authored::card_3623::RULES.activated, enters: authored::card_3623::RULES.enters, entry: authored::card_3623::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_3623::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Sanguine Indulgence", oracle_id: Some("010ec11a-0484-486d-b7c0-de65593bf457"), mana_cost: "{3}{B}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3684::RULES.keywords, deck_limit: authored::card_3684::RULES.deck_limit }, effect: authored::card_3684::RULES.checked_effect(), triggers: authored::card_3684::RULES.triggers, activated: authored::card_3684::RULES.activated, enters: authored::card_3684::RULES.enters, entry: authored::card_3684::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 3 }, rules: &authored::card_3684::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Sanguine Syphoner", oracle_id: Some("b81a748b-f7a6-4ec4-8b18-40a1819745f6"), mana_cost: "{1}{B}", type_line: "Creature — Vampire Warlock", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3686::RULES.keywords, deck_limit: authored::card_3686::RULES.deck_limit }, effect: authored::card_3686::RULES.checked_effect(), triggers: authored::card_3686::RULES.triggers, activated: authored::card_3686::RULES.activated, enters: authored::card_3686::RULES.enters, entry: authored::card_3686::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 1 }, rules: &authored::card_3686::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Savage Ventmaw", oracle_id: Some("f743d85f-754e-4e70-9907-af54acb9e8d4"), mana_cost: "{4}{R}{G}", type_line: "Creature — Dragon", oracle_keywords: &["Flying"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3694::RULES.keywords, deck_limit: authored::card_3694::RULES.deck_limit }, effect: authored::card_3694::RULES.checked_effect(), triggers: authored::card_3694::RULES.triggers, activated: authored::card_3694::RULES.activated, enters: authored::card_3694::RULES.enters, entry: authored::card_3694::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 1, 0], generic: 4 }, rules: &authored::card_3694::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Savannah Lions", oracle_id: Some("60ba93eb-39e6-4af2-9c66-cd38f72daff2"), mana_cost: "{W}", type_line: "Creature — Cat", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3695::RULES.keywords, deck_limit: authored::card_3695::RULES.deck_limit }, effect: authored::card_3695::RULES.checked_effect(), triggers: authored::card_3695::RULES.triggers, activated: authored::card_3695::RULES.activated, enters: authored::card_3695::RULES.enters, entry: authored::card_3695::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_3695::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Scavenging Ooze", oracle_id: Some("1ff25f67-36a7-4cfa-a2b1-2135b5b6fb67"), mana_cost: "{1}{G}", type_line: "Creature — Ooze", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3719::RULES.keywords, deck_limit: authored::card_3719::RULES.deck_limit }, effect: authored::card_3719::RULES.checked_effect(), triggers: authored::card_3719::RULES.triggers, activated: authored::card_3719::RULES.activated, enters: authored::card_3719::RULES.enters, entry: authored::card_3719::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 1 }, rules: &authored::card_3719::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Scorching Dragonfire", oracle_id: Some("d14f313c-fea6-49c4-8197-5b74ee584a6b"), mana_cost: "{1}{R}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3725::RULES.keywords, deck_limit: authored::card_3725::RULES.deck_limit }, effect: authored::card_3725::RULES.checked_effect(), triggers: authored::card_3725::RULES.triggers, activated: authored::card_3725::RULES.activated, enters: authored::card_3725::RULES.enters, entry: authored::card_3725::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 1 }, rules: &authored::card_3725::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Scrawling Crawler", oracle_id: Some("1d60c80d-9a95-4051-bbf7-02ca51ee1be2"), mana_cost: "{3}", type_line: "Artifact Creature — Phyrexian Construct", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3737::RULES.keywords, deck_limit: authored::card_3737::RULES.deck_limit }, effect: authored::card_3737::RULES.checked_effect(), triggers: authored::card_3737::RULES.triggers, activated: authored::card_3737::RULES.activated, enters: authored::card_3737::RULES.enters, entry: authored::card_3737::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_3737::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Searslicer Goblin", oracle_id: Some("86ec0b56-497f-481c-a03a-c4e64e8e7404"), mana_cost: "{1}{R}", type_line: "Creature — Goblin Warrior", oracle_keywords: &["Raid"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3748::RULES.keywords, deck_limit: authored::card_3748::RULES.deck_limit }, effect: authored::card_3748::RULES.checked_effect(), triggers: authored::card_3748::RULES.triggers, activated: authored::card_3748::RULES.activated, enters: authored::card_3748::RULES.enters, entry: authored::card_3748::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 1 }, rules: &authored::card_3748::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Seeker's Folly", oracle_id: Some("223794f2-67a3-4ab5-81c5-046c63307c6c"), mana_cost: "{2}{B}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3769::RULES.keywords, deck_limit: authored::card_3769::RULES.deck_limit }, effect: authored::card_3769::RULES.checked_effect(), triggers: authored::card_3769::RULES.triggers, activated: authored::card_3769::RULES.activated, enters: authored::card_3769::RULES.enters, entry: authored::card_3769::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 2 }, rules: &authored::card_3769::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Seismic Rupture", oracle_id: Some("3b1f1a57-d83a-443f-b190-03df63d2bfe6"), mana_cost: "{2}{R}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3772::RULES.keywords, deck_limit: authored::card_3772::RULES.deck_limit }, effect: authored::card_3772::RULES.checked_effect(), triggers: authored::card_3772::RULES.triggers, activated: authored::card_3772::RULES.activated, enters: authored::card_3772::RULES.enters, entry: authored::card_3772::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 2 }, rules: &authored::card_3772::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Selesnya Guildgate", oracle_id: Some("75b235d3-595a-4859-be45-9559d8445db5"), mana_cost: "", type_line: "Land — Gate", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3778::RULES.keywords, deck_limit: authored::card_3778::RULES.deck_limit }, effect: authored::card_3778::RULES.checked_effect(), triggers: authored::card_3778::RULES.triggers, activated: authored::card_3778::RULES.activated, enters: authored::card_3778::RULES.enters, entry: authored::card_3778::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_3778::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Self-Reflection", oracle_id: Some("5c68853e-4fb8-465e-b262-e6dbaf972533"), mana_cost: "{4}{U}{U}", type_line: "Sorcery", oracle_keywords: &["Flashback"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3780::RULES.keywords, deck_limit: authored::card_3780::RULES.deck_limit }, effect: authored::card_3780::RULES.checked_effect(), triggers: authored::card_3780::RULES.triggers, activated: authored::card_3780::RULES.activated, enters: authored::card_3780::RULES.enters, entry: authored::card_3780::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 2, 0, 0, 0, 0], generic: 4 }, rules: &authored::card_3780::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Serra Angel", oracle_id: Some("4b7ac066-e5c7-43e6-9e7e-2739b24a905d"), mana_cost: "{3}{W}{W}", type_line: "Creature — Angel", oracle_keywords: &["Flying", "Vigilance"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3797::RULES.keywords, deck_limit: authored::card_3797::RULES.deck_limit }, effect: authored::card_3797::RULES.checked_effect(), triggers: authored::card_3797::RULES.triggers, activated: authored::card_3797::RULES.activated, enters: authored::card_3797::RULES.enters, entry: authored::card_3797::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [2, 0, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_3797::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Shipwreck Dowser", oracle_id: Some("dc3073fc-dc23-426e-b867-fee5dd99fe25"), mana_cost: "{3}{U}{U}", type_line: "Creature — Merfolk Wizard", oracle_keywords: &["Prowess"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3835::RULES.keywords, deck_limit: authored::card_3835::RULES.deck_limit }, effect: authored::card_3835::RULES.checked_effect(), triggers: authored::card_3835::RULES.triggers, activated: authored::card_3835::RULES.activated, enters: authored::card_3835::RULES.enters, entry: authored::card_3835::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 2, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_3835::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Shivan Dragon", oracle_id: Some("711eea87-0fa3-46e0-a42b-fa5a86455f04"), mana_cost: "{4}{R}{R}", type_line: "Creature — Dragon", oracle_keywords: &["Flying"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3838::RULES.keywords, deck_limit: authored::card_3838::RULES.deck_limit }, effect: authored::card_3838::RULES.checked_effect(), triggers: authored::card_3838::RULES.triggers, activated: authored::card_3838::RULES.activated, enters: authored::card_3838::RULES.enters, entry: authored::card_3838::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 2, 0, 0], generic: 4 }, rules: &authored::card_3838::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Simic Guildgate", oracle_id: Some("e8705df9-6439-4930-91b6-229f818559af"), mana_cost: "", type_line: "Land — Gate", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3876::RULES.keywords, deck_limit: authored::card_3876::RULES.deck_limit }, effect: authored::card_3876::RULES.checked_effect(), triggers: authored::card_3876::RULES.triggers, activated: authored::card_3876::RULES.activated, enters: authored::card_3876::RULES.enters, entry: authored::card_3876::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_3876::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Sire of Seven Deaths", oracle_id: Some("a15547d0-bbcb-41ee-a9ae-8325a9e7fe1f"), mana_cost: "{7}", type_line: "Creature — Eldrazi", oracle_keywords: &["Lifelink", "Reach", "Vigilance", "First strike", "Trample", "Menace", "Ward"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3886::RULES.keywords, deck_limit: authored::card_3886::RULES.deck_limit }, effect: authored::card_3886::RULES.checked_effect(), triggers: authored::card_3886::RULES.triggers, activated: authored::card_3886::RULES.activated, enters: authored::card_3886::RULES.enters, entry: authored::card_3886::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 7 }, rules: &authored::card_3886::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Skeleton Archer", oracle_id: Some("3792763c-7aa0-4055-b8a1-3257e0443ef1"), mana_cost: "{3}{B}", type_line: "Creature — Skeleton Archer", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3890::RULES.keywords, deck_limit: authored::card_3890::RULES.deck_limit }, effect: authored::card_3890::RULES.checked_effect(), triggers: authored::card_3890::RULES.triggers, activated: authored::card_3890::RULES.activated, enters: authored::card_3890::RULES.enters, entry: authored::card_3890::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 3 }, rules: &authored::card_3890::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Skyknight Squire", oracle_id: Some("7e497f6b-69e1-4fab-bb04-f687c99a8310"), mana_cost: "{1}{W}", type_line: "Creature — Cat Scout", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3902::RULES.keywords, deck_limit: authored::card_3902::RULES.deck_limit }, effect: authored::card_3902::RULES.checked_effect(), triggers: authored::card_3902::RULES.triggers, activated: authored::card_3902::RULES.activated, enters: authored::card_3902::RULES.enters, entry: authored::card_3902::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_3902::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Skyraker Giant", oracle_id: Some("d481630c-83c4-47d2-a770-db331154d2b9"), mana_cost: "{2}{R}{R}", type_line: "Creature — Giant", oracle_keywords: &["Reach"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3903::RULES.keywords, deck_limit: authored::card_3903::RULES.deck_limit }, effect: authored::card_3903::RULES.checked_effect(), triggers: authored::card_3903::RULES.triggers, activated: authored::card_3903::RULES.activated, enters: authored::card_3903::RULES.enters, entry: authored::card_3903::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 2, 0, 0], generic: 2 }, rules: &authored::card_3903::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Skyship Buccaneer", oracle_id: Some("5951f734-05d3-41ad-94ee-19bd310077d2"), mana_cost: "{3}{U}{U}", type_line: "Creature — Human Pirate", oracle_keywords: &["Flying", "Raid"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3906::RULES.keywords, deck_limit: authored::card_3906::RULES.deck_limit }, effect: authored::card_3906::RULES.checked_effect(), triggers: authored::card_3906::RULES.triggers, activated: authored::card_3906::RULES.activated, enters: authored::card_3906::RULES.enters, entry: authored::card_3906::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 2, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_3906::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Slagstorm", oracle_id: Some("2ed1879a-83b8-48df-a6d2-fe28e912b4c5"), mana_cost: "{1}{R}{R}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3912::RULES.keywords, deck_limit: authored::card_3912::RULES.deck_limit }, effect: authored::card_3912::RULES.checked_effect(), triggers: authored::card_3912::RULES.triggers, activated: authored::card_3912::RULES.activated, enters: authored::card_3912::RULES.enters, entry: authored::card_3912::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 0, 2, 0, 0], generic: 1 }, rules: &authored::card_3912::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Slumbering Cerberus", oracle_id: Some("39cae244-96e1-4ea3-899a-3aa871cca31c"), mana_cost: "{1}{R}", type_line: "Creature — Dog", oracle_keywords: &["Morbid"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3929::RULES.keywords, deck_limit: authored::card_3929::RULES.deck_limit }, effect: authored::card_3929::RULES.checked_effect(), triggers: authored::card_3929::RULES.triggers, activated: authored::card_3929::RULES.activated, enters: authored::card_3929::RULES.enters, entry: authored::card_3929::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 1 }, rules: &authored::card_3929::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Sorcerous Spyglass", oracle_id: Some("b2187f45-80ae-4ac4-9f83-5eb7a00978e2"), mana_cost: "{2}", type_line: "Artifact", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3970::RULES.keywords, deck_limit: authored::card_3970::RULES.deck_limit }, effect: authored::card_3970::RULES.checked_effect(), triggers: authored::card_3970::RULES.triggers, activated: authored::card_3970::RULES.activated, enters: authored::card_3970::RULES.enters, entry: authored::card_3970::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_3970::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Soul-Shackled Zombie", oracle_id: Some("f81ca3a5-356d-4b5d-afc2-a33aa774816b"), mana_cost: "{3}{B}", type_line: "Creature — Zombie", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3976::RULES.keywords, deck_limit: authored::card_3976::RULES.deck_limit }, effect: authored::card_3976::RULES.checked_effect(), triggers: authored::card_3976::RULES.triggers, activated: authored::card_3976::RULES.activated, enters: authored::card_3976::RULES.enters, entry: authored::card_3976::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 3 }, rules: &authored::card_3976::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Soulstone Sanctuary", oracle_id: Some("3c0f99b8-0222-4fac-932a-eb5d77826564"), mana_cost: "", type_line: "Land", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3980::RULES.keywords, deck_limit: authored::card_3980::RULES.deck_limit }, effect: authored::card_3980::RULES.checked_effect(), triggers: authored::card_3980::RULES.triggers, activated: authored::card_3980::RULES.activated, enters: authored::card_3980::RULES.enters, entry: authored::card_3980::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_3980::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Sower of Chaos", oracle_id: Some("2ec1860b-89cc-4529-8b8c-fb47f79f2b4b"), mana_cost: "{3}{R}", type_line: "Creature — Devil", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_3988::RULES.keywords, deck_limit: authored::card_3988::RULES.deck_limit }, effect: authored::card_3988::RULES.checked_effect(), triggers: authored::card_3988::RULES.triggers, activated: authored::card_3988::RULES.activated, enters: authored::card_3988::RULES.enters, entry: authored::card_3988::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 3 }, rules: &authored::card_3988::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Spectral Sailor", oracle_id: Some("a8fdbcdf-479d-4582-9ad5-9fbd4c740c29"), mana_cost: "{U}", type_line: "Creature — Spirit Pirate", oracle_keywords: &["Flying", "Flash"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4001::RULES.keywords, deck_limit: authored::card_4001::RULES.deck_limit }, effect: authored::card_4001::RULES.checked_effect(), triggers: authored::card_4001::RULES.triggers, activated: authored::card_4001::RULES.activated, enters: authored::card_4001::RULES.enters, entry: authored::card_4001::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_4001::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Sphinx of Forgotten Lore", oracle_id: Some("ddb7e868-8f1d-4d57-9ee2-ebadb17b398a"), mana_cost: "{2}{U}{U}", type_line: "Creature — Sphinx", oracle_keywords: &["Flying", "Flash"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4014::RULES.keywords, deck_limit: authored::card_4014::RULES.deck_limit }, effect: authored::card_4014::RULES.checked_effect(), triggers: authored::card_4014::RULES.triggers, activated: authored::card_4014::RULES.activated, enters: authored::card_4014::RULES.enters, entry: authored::card_4014::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 2, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_4014::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Sphinx of the Final Word", oracle_id: Some("d4246e4d-390d-4925-a5a8-89cd096a237c"), mana_cost: "{5}{U}{U}", type_line: "Creature — Sphinx", oracle_keywords: &["Flying", "Hexproof"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4015::RULES.keywords, deck_limit: authored::card_4015::RULES.deck_limit }, effect: authored::card_4015::RULES.checked_effect(), triggers: authored::card_4015::RULES.triggers, activated: authored::card_4015::RULES.activated, enters: authored::card_4015::RULES.enters, entry: authored::card_4015::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 2, 0, 0, 0, 0], generic: 5 }, rules: &authored::card_4015::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Spinner of Souls", oracle_id: Some("6883e79e-0b4f-4fb8-aee3-a3905246a408"), mana_cost: "{2}{G}", type_line: "Creature — Spider Spirit", oracle_keywords: &["Reach"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4050::RULES.keywords, deck_limit: authored::card_4050::RULES.deck_limit }, effect: authored::card_4050::RULES.checked_effect(), triggers: authored::card_4050::RULES.triggers, activated: authored::card_4050::RULES.activated, enters: authored::card_4050::RULES.enters, entry: authored::card_4050::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 2 }, rules: &authored::card_4050::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Spitfire Lagac", oracle_id: Some("7215ca87-bbff-4c14-a1f5-de5ddd97c875"), mana_cost: "{3}{R}", type_line: "Creature — Lizard", oracle_keywords: &["Landfall"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4059::RULES.keywords, deck_limit: authored::card_4059::RULES.deck_limit }, effect: authored::card_4059::RULES.checked_effect(), triggers: authored::card_4059::RULES.triggers, activated: authored::card_4059::RULES.activated, enters: authored::card_4059::RULES.enters, entry: authored::card_4059::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 3 }, rules: &authored::card_4059::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Springbloom Druid", oracle_id: Some("788a4896-3414-40bf-b391-d2efea2fb5f9"), mana_cost: "{2}{G}", type_line: "Creature — Elf Druid", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4073::RULES.keywords, deck_limit: authored::card_4073::RULES.deck_limit }, effect: authored::card_4073::RULES.checked_effect(), triggers: authored::card_4073::RULES.triggers, activated: authored::card_4073::RULES.activated, enters: authored::card_4073::RULES.enters, entry: authored::card_4073::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 2 }, rules: &authored::card_4073::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Squad Rallier", oracle_id: Some("e0b15374-b4f6-48e0-8477-4eb5d5c2a938"), mana_cost: "{3}{W}", type_line: "Creature — Human Scout", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4077::RULES.keywords, deck_limit: authored::card_4077::RULES.deck_limit }, effect: authored::card_4077::RULES.checked_effect(), triggers: authored::card_4077::RULES.triggers, activated: authored::card_4077::RULES.activated, enters: authored::card_4077::RULES.enters, entry: authored::card_4077::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_4077::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Stab", oracle_id: Some("b4ee6843-bb79-4665-9763-83f656863dc5"), mana_cost: "{B}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4083::RULES.keywords, deck_limit: authored::card_4083::RULES.deck_limit }, effect: authored::card_4083::RULES.checked_effect(), triggers: authored::card_4083::RULES.triggers, activated: authored::card_4083::RULES.activated, enters: authored::card_4083::RULES.enters, entry: authored::card_4083::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 0 }, rules: &authored::card_4083::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Starlight Snare", oracle_id: Some("f0691d46-5bf4-4617-8a0a-19d17afb7219"), mana_cost: "{2}{U}", type_line: "Enchantment — Aura", oracle_keywords: &["Enchant"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4105::RULES.keywords, deck_limit: authored::card_4105::RULES.deck_limit }, effect: authored::card_4105::RULES.checked_effect(), triggers: authored::card_4105::RULES.triggers, activated: authored::card_4105::RULES.activated, enters: authored::card_4105::RULES.enters, entry: authored::card_4105::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_4105::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Stasis Snare", oracle_id: Some("6a83768e-672e-4fec-8931-853f5e96d43d"), mana_cost: "{1}{W}{W}", type_line: "Enchantment", oracle_keywords: &["Flash"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4117::RULES.keywords, deck_limit: authored::card_4117::RULES.deck_limit }, effect: authored::card_4117::RULES.checked_effect(), triggers: authored::card_4117::RULES.triggers, activated: authored::card_4117::RULES.activated, enters: authored::card_4117::RULES.enters, entry: authored::card_4117::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [2, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_4117::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Storm Fleet Spy", oracle_id: Some("42081a57-b521-45c2-928d-0a0a4641818d"), mana_cost: "{2}{U}", type_line: "Creature — Human Pirate", oracle_keywords: &["Raid"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4168::RULES.keywords, deck_limit: authored::card_4168::RULES.deck_limit }, effect: authored::card_4168::RULES.checked_effect(), triggers: authored::card_4168::RULES.triggers, activated: authored::card_4168::RULES.activated, enters: authored::card_4168::RULES.enters, entry: authored::card_4168::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_4168::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Strix Lookout", oracle_id: Some("5a6d01c8-47b6-43e5-a6e9-c48088286007"), mana_cost: "{1}{U}", type_line: "Creature — Bird", oracle_keywords: &["Flying", "Vigilance"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4188::RULES.keywords, deck_limit: authored::card_4188::RULES.deck_limit }, effect: authored::card_4188::RULES.checked_effect(), triggers: authored::card_4188::RULES.triggers, activated: authored::card_4188::RULES.activated, enters: authored::card_4188::RULES.enters, entry: authored::card_4188::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_4188::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Stromkirk Bloodthief", oracle_id: Some("c85987b4-090e-472e-b9af-1114cce56ffb"), mana_cost: "{2}{B}", type_line: "Creature — Vampire Rogue", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4191::RULES.keywords, deck_limit: authored::card_4191::RULES.deck_limit }, effect: authored::card_4191::RULES.checked_effect(), triggers: authored::card_4191::RULES.triggers, activated: authored::card_4191::RULES.activated, enters: authored::card_4191::RULES.enters, entry: authored::card_4191::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 2 }, rules: &authored::card_4191::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Stromkirk Noble", oracle_id: Some("c55909d3-1f67-4a4a-9d53-8513d6cf96d8"), mana_cost: "{R}", type_line: "Creature — Vampire Noble", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4192::RULES.keywords, deck_limit: authored::card_4192::RULES.deck_limit }, effect: authored::card_4192::RULES.checked_effect(), triggers: authored::card_4192::RULES.triggers, activated: authored::card_4192::RULES.activated, enters: authored::card_4192::RULES.enters, entry: authored::card_4192::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 0 }, rules: &authored::card_4192::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Strongbox Raider", oracle_id: Some("ba5a2689-3578-4b1a-99d6-6b6d2356330e"), mana_cost: "{2}{R}{R}", type_line: "Creature — Orc Pirate", oracle_keywords: &["Raid"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4193::RULES.keywords, deck_limit: authored::card_4193::RULES.deck_limit }, effect: authored::card_4193::RULES.checked_effect(), triggers: authored::card_4193::RULES.triggers, activated: authored::card_4193::RULES.activated, enters: authored::card_4193::RULES.enters, entry: authored::card_4193::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 2, 0, 0], generic: 2 }, rules: &authored::card_4193::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Sun-Blessed Healer", oracle_id: Some("1729c6f2-2452-4fbc-a1bc-3e625f2ad8e1"), mana_cost: "{1}{W}", type_line: "Creature — Human Cleric", oracle_keywords: &["Lifelink", "Kicker"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4231::RULES.keywords, deck_limit: authored::card_4231::RULES.deck_limit }, effect: authored::card_4231::RULES.checked_effect(), triggers: authored::card_4231::RULES.triggers, activated: authored::card_4231::RULES.activated, enters: authored::card_4231::RULES.enters, entry: authored::card_4231::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_4231::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Sure Strike", oracle_id: Some("fb694e7e-f66e-4958-b6ed-aa74bc9ac43e"), mana_cost: "{1}{R}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4266::RULES.keywords, deck_limit: authored::card_4266::RULES.deck_limit }, effect: authored::card_4266::RULES.checked_effect(), triggers: authored::card_4266::RULES.triggers, activated: authored::card_4266::RULES.activated, enters: authored::card_4266::RULES.enters, entry: authored::card_4266::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 1 }, rules: &authored::card_4266::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Surrak, the Hunt Caller", oracle_id: Some("fa56a5fa-ef96-404c-8fb6-d1f5fcebb52e"), mana_cost: "{2}{G}{G}", type_line: "Legendary Creature — Human Warrior", oracle_keywords: &["Formidable"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_4272::RULES.keywords, deck_limit: authored::card_4272::RULES.deck_limit }, effect: authored::card_4272::RULES.checked_effect(), triggers: authored::card_4272::RULES.triggers, activated: authored::card_4272::RULES.activated, enters: authored::card_4272::RULES.enters, entry: authored::card_4272::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 2, 0], generic: 2 }, rules: &authored::card_4272::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Suspicious Shambler", oracle_id: Some("ea6ee33f-0cba-4e96-817f-dc89c9064ead"), mana_cost: "{3}{B}", type_line: "Creature — Zombie", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4279::RULES.keywords, deck_limit: authored::card_4279::RULES.deck_limit }, effect: authored::card_4279::RULES.checked_effect(), triggers: authored::card_4279::RULES.triggers, activated: authored::card_4279::RULES.activated, enters: authored::card_4279::RULES.enters, entry: authored::card_4279::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 3 }, rules: &authored::card_4279::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Swab Goblin", oracle_id: Some("343ecf2f-0bbc-4840-8fb3-f5f61b26e76f"), mana_cost: "{1}{R}", type_line: "Creature — Goblin Pirate", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4283::RULES.keywords, deck_limit: authored::card_4283::RULES.deck_limit }, effect: authored::card_4283::RULES.checked_effect(), triggers: authored::card_4283::RULES.triggers, activated: authored::card_4283::RULES.activated, enters: authored::card_4283::RULES.enters, entry: authored::card_4283::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 1 }, rules: &authored::card_4283::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Swiftblade Vindicator", oracle_id: Some("44240b1e-c104-4eb4-8ab9-0d56607ba241"), mana_cost: "{R}{W}", type_line: "Creature — Human Soldier", oracle_keywords: &["Vigilance", "Trample", "Double strike"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4291::RULES.keywords, deck_limit: authored::card_4291::RULES.deck_limit }, effect: authored::card_4291::RULES.checked_effect(), triggers: authored::card_4291::RULES.triggers, activated: authored::card_4291::RULES.activated, enters: authored::card_4291::RULES.enters, entry: authored::card_4291::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 1, 0, 0], generic: 0 }, rules: &authored::card_4291::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Sylvan Scavenging", oracle_id: Some("398afdf2-4a12-430e-85c9-c68847230939"), mana_cost: "{1}{G}{G}", type_line: "Enchantment", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4300::RULES.keywords, deck_limit: authored::card_4300::RULES.deck_limit }, effect: authored::card_4300::RULES.checked_effect(), triggers: authored::card_4300::RULES.triggers, activated: authored::card_4300::RULES.activated, enters: authored::card_4300::RULES.enters, entry: authored::card_4300::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 0, 0, 0, 2, 0], generic: 1 }, rules: &authored::card_4300::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Syr Alin, the Lion's Claw", oracle_id: Some("5d9a4905-4a26-410e-936c-56de849e9ca3"), mana_cost: "{3}{W}{W}", type_line: "Legendary Creature — Human Knight", oracle_keywords: &["First strike"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_4307::RULES.keywords, deck_limit: authored::card_4307::RULES.deck_limit }, effect: authored::card_4307::RULES.checked_effect(), triggers: authored::card_4307::RULES.triggers, activated: authored::card_4307::RULES.activated, enters: authored::card_4307::RULES.enters, entry: authored::card_4307::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [2, 0, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_4307::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Tajuru Pathwarden", oracle_id: Some("d8c7cd66-d8bf-4a99-add5-1d5651d4e025"), mana_cost: "{4}{G}", type_line: "Creature — Elf Warrior Ally", oracle_keywords: &["Vigilance", "Trample"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4319::RULES.keywords, deck_limit: authored::card_4319::RULES.deck_limit }, effect: authored::card_4319::RULES.checked_effect(), triggers: authored::card_4319::RULES.triggers, activated: authored::card_4319::RULES.activated, enters: authored::card_4319::RULES.enters, entry: authored::card_4319::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 4 }, rules: &authored::card_4319::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Tatyova, Benthic Druid", oracle_id: Some("0715e860-3b3b-4331-9718-207973e94fee"), mana_cost: "{3}{G}{U}", type_line: "Legendary Creature — Merfolk Druid", oracle_keywords: &["Landfall"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_4345::RULES.keywords, deck_limit: authored::card_4345::RULES.deck_limit }, effect: authored::card_4345::RULES.checked_effect(), triggers: authored::card_4345::RULES.triggers, activated: authored::card_4345::RULES.activated, enters: authored::card_4345::RULES.enters, entry: authored::card_4345::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 1, 0], generic: 3 }, rules: &authored::card_4345::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Teach by Example", oracle_id: Some("d85ba214-fa05-44ca-a7f9-ab2c5dc07962"), mana_cost: "{U/R}{U/R}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4348::RULES.keywords, deck_limit: authored::card_4348::RULES.deck_limit }, effect: authored::card_4348::RULES.checked_effect(), triggers: authored::card_4348::RULES.triggers, activated: authored::card_4348::RULES.activated, enters: authored::card_4348::RULES.enters, entry: authored::card_4348::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 0, 2, 0, 0], generic: 0 }, rules: &authored::card_4348::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Tempest Djinn", oracle_id: Some("b94c233e-11d3-4556-a1da-bd193e58b663"), mana_cost: "{U}{U}{U}", type_line: "Creature — Djinn", oracle_keywords: &["Flying"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4358::RULES.keywords, deck_limit: authored::card_4358::RULES.deck_limit }, effect: authored::card_4358::RULES.checked_effect(), triggers: authored::card_4358::RULES.triggers, activated: authored::card_4358::RULES.activated, enters: authored::card_4358::RULES.enters, entry: authored::card_4358::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 3, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_4358::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Temple of Deceit", oracle_id: Some("33b9b3bd-33ca-46f3-b8bb-a978bc3d1085"), mana_cost: "", type_line: "Land", oracle_keywords: &["Scry"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4363::RULES.keywords, deck_limit: authored::card_4363::RULES.deck_limit }, effect: authored::card_4363::RULES.checked_effect(), triggers: authored::card_4363::RULES.triggers, activated: authored::card_4363::RULES.activated, enters: authored::card_4363::RULES.enters, entry: authored::card_4363::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_4363::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Temple of Malice", oracle_id: Some("7c439c18-31dc-41fe-b03d-3fca06e6fc0b"), mana_cost: "", type_line: "Land", oracle_keywords: &["Scry"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4367::RULES.keywords, deck_limit: authored::card_4367::RULES.deck_limit }, effect: authored::card_4367::RULES.checked_effect(), triggers: authored::card_4367::RULES.triggers, activated: authored::card_4367::RULES.activated, enters: authored::card_4367::RULES.enters, entry: authored::card_4367::RULES.entry, kind: 2, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_4367::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Terror of Mount Velus", oracle_id: Some("8161248c-16a3-47db-a3ae-cdfd878d49e0"), mana_cost: "{5}{R}{R}", type_line: "Creature — Dragon", oracle_keywords: &["Flying", "Double strike"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4399::RULES.keywords, deck_limit: authored::card_4399::RULES.deck_limit }, effect: authored::card_4399::RULES.checked_effect(), triggers: authored::card_4399::RULES.triggers, activated: authored::card_4399::RULES.activated, enters: authored::card_4399::RULES.enters, entry: authored::card_4399::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 2, 0, 0], generic: 5 }, rules: &authored::card_4399::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Thornweald Archer", oracle_id: Some("b5ba87f5-3589-4a8b-ad65-2ae2523e98e8"), mana_cost: "{1}{G}", type_line: "Creature — Elf Archer", oracle_keywords: &["Reach", "Deathtouch"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4546::RULES.keywords, deck_limit: authored::card_4546::RULES.deck_limit }, effect: authored::card_4546::RULES.checked_effect(), triggers: authored::card_4546::RULES.triggers, activated: authored::card_4546::RULES.activated, enters: authored::card_4546::RULES.enters, entry: authored::card_4546::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 1 }, rules: &authored::card_4546::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Thousand-Year Storm", oracle_id: Some("dd4cf149-2fae-40e5-b50b-639f6bcec65e"), mana_cost: "{4}{U}{R}", type_line: "Enchantment", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4556::RULES.keywords, deck_limit: authored::card_4556::RULES.deck_limit }, effect: authored::card_4556::RULES.checked_effect(), triggers: authored::card_4556::RULES.triggers, activated: authored::card_4556::RULES.activated, enters: authored::card_4556::RULES.enters, entry: authored::card_4556::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 1, 0, 1, 0, 0], generic: 4 }, rules: &authored::card_4556::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Thrashing Brontodon", oracle_id: Some("60bc63dc-ac9f-4a2f-aef5-c90d0aa31553"), mana_cost: "{1}{G}{G}", type_line: "Creature — Dinosaur", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4561::RULES.keywords, deck_limit: authored::card_4561::RULES.deck_limit }, effect: authored::card_4561::RULES.checked_effect(), triggers: authored::card_4561::RULES.triggers, activated: authored::card_4561::RULES.activated, enters: authored::card_4561::RULES.enters, entry: authored::card_4561::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 2, 0], generic: 1 }, rules: &authored::card_4561::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Three Tree Mascot", oracle_id: Some("6cb9d153-4eaa-4f32-8930-bd68cd99c128"), mana_cost: "{2}", type_line: "Artifact Creature — Shapeshifter", oracle_keywords: &["Changeling"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4568::RULES.keywords, deck_limit: authored::card_4568::RULES.deck_limit }, effect: authored::card_4568::RULES.checked_effect(), triggers: authored::card_4568::RULES.triggers, activated: authored::card_4568::RULES.activated, enters: authored::card_4568::RULES.enters, entry: authored::card_4568::RULES.entry, kind: 17, basic: false, mana_color: None, colored: [0, 0, 0, 0, 0, 0], generic: 2 }, rules: &authored::card_4568::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Thrill of Possibility", oracle_id: Some("1cb0610b-a731-42c2-b93f-0a29f63cebf4"), mana_cost: "{1}{R}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4572::RULES.keywords, deck_limit: authored::card_4572::RULES.deck_limit }, effect: authored::card_4572::RULES.checked_effect(), triggers: authored::card_4572::RULES.triggers, activated: authored::card_4572::RULES.activated, enters: authored::card_4572::RULES.enters, entry: authored::card_4572::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 1 }, rules: &authored::card_4572::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Time Stop", oracle_id: Some("55bc6a7e-2356-48f0-a93a-8bf19044ee4b"), mana_cost: "{4}{U}{U}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4601::RULES.keywords, deck_limit: authored::card_4601::RULES.deck_limit }, effect: authored::card_4601::RULES.checked_effect(), triggers: authored::card_4601::RULES.triggers, activated: authored::card_4601::RULES.activated, enters: authored::card_4601::RULES.enters, entry: authored::card_4601::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 2, 0, 0, 0, 0], generic: 4 }, rules: &authored::card_4601::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Tinybones, Bauble Burglar", oracle_id: Some("fd335b2f-e6a2-45ac-949e-a67278b21cb6"), mana_cost: "{1}{B}", type_line: "Legendary Creature — Skeleton Rogue", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_4607::RULES.keywords, deck_limit: authored::card_4607::RULES.deck_limit }, effect: authored::card_4607::RULES.checked_effect(), triggers: authored::card_4607::RULES.triggers, activated: authored::card_4607::RULES.activated, enters: authored::card_4607::RULES.enters, entry: authored::card_4607::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 1 }, rules: &authored::card_4607::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Tolarian Terror", oracle_id: Some("b9d0f2e1-62c2-44fd-ad38-471daf17bb0a"), mana_cost: "{6}{U}", type_line: "Creature — Serpent", oracle_keywords: &["Ward"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4620::RULES.keywords, deck_limit: authored::card_4620::RULES.deck_limit }, effect: authored::card_4620::RULES.checked_effect(), triggers: authored::card_4620::RULES.triggers, activated: authored::card_4620::RULES.activated, enters: authored::card_4620::RULES.enters, entry: authored::card_4620::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 6 }, rules: &authored::card_4620::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Tragic Banshee", oracle_id: Some("888f58c0-1860-4df4-ad75-53e634150680"), mana_cost: "{4}{B}", type_line: "Creature — Spirit", oracle_keywords: &["Morbid"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4650::RULES.keywords, deck_limit: authored::card_4650::RULES.deck_limit }, effect: authored::card_4650::RULES.checked_effect(), triggers: authored::card_4650::RULES.triggers, activated: authored::card_4650::RULES.activated, enters: authored::card_4650::RULES.enters, entry: authored::card_4650::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 4 }, rules: &authored::card_4650::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Treetop Snarespinner", oracle_id: Some("2f57a220-9f64-4143-979a-d6aaf63d49b2"), mana_cost: "{3}{G}", type_line: "Creature — Spider", oracle_keywords: &["Reach", "Deathtouch"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4675::RULES.keywords, deck_limit: authored::card_4675::RULES.deck_limit }, effect: authored::card_4675::RULES.checked_effect(), triggers: authored::card_4675::RULES.triggers, activated: authored::card_4675::RULES.activated, enters: authored::card_4675::RULES.enters, entry: authored::card_4675::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 3 }, rules: &authored::card_4675::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Tribute to Hunger", oracle_id: Some("dc5a501c-e16c-4d4b-a510-70a14272473e"), mana_cost: "{2}{B}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4679::RULES.keywords, deck_limit: authored::card_4679::RULES.deck_limit }, effect: authored::card_4679::RULES.checked_effect(), triggers: authored::card_4679::RULES.triggers, activated: authored::card_4679::RULES.activated, enters: authored::card_4679::RULES.enters, entry: authored::card_4679::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 2 }, rules: &authored::card_4679::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Trygon Predator", oracle_id: Some("c744b5f4-fbcf-48b8-9d60-5e9c6ac297e0"), mana_cost: "{1}{G}{U}", type_line: "Creature — Beast", oracle_keywords: &["Flying"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4695::RULES.keywords, deck_limit: authored::card_4695::RULES.deck_limit }, effect: authored::card_4695::RULES.checked_effect(), triggers: authored::card_4695::RULES.triggers, activated: authored::card_4695::RULES.activated, enters: authored::card_4695::RULES.enters, entry: authored::card_4695::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 1, 0], generic: 1 }, rules: &authored::card_4695::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Twinblade Blessing", oracle_id: Some("b3e1674f-dba1-4570-9990-e5dc643e237d"), mana_cost: "{1}{W}{W}", type_line: "Enchantment — Aura", oracle_keywords: &["Enchant", "Flash"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4718::RULES.keywords, deck_limit: authored::card_4718::RULES.deck_limit }, effect: authored::card_4718::RULES.checked_effect(), triggers: authored::card_4718::RULES.triggers, activated: authored::card_4718::RULES.activated, enters: authored::card_4718::RULES.enters, entry: authored::card_4718::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [2, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_4718::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Twinblade Paladin", oracle_id: Some("d373dc71-5196-43a0-8cc5-67e2a591aad0"), mana_cost: "{3}{W}", type_line: "Creature — Human Knight", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4719::RULES.keywords, deck_limit: authored::card_4719::RULES.deck_limit }, effect: authored::card_4719::RULES.checked_effect(), triggers: authored::card_4719::RULES.triggers, activated: authored::card_4719::RULES.activated, enters: authored::card_4719::RULES.enters, entry: authored::card_4719::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_4719::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Twinflame Tyrant", oracle_id: Some("321b5cc6-8df6-4292-97a0-a6a3a22f3b55"), mana_cost: "{3}{R}{R}", type_line: "Creature — Dragon", oracle_keywords: &["Flying", "Double"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4721::RULES.keywords, deck_limit: authored::card_4721::RULES.deck_limit }, effect: authored::card_4721::RULES.checked_effect(), triggers: authored::card_4721::RULES.triggers, activated: authored::card_4721::RULES.activated, enters: authored::card_4721::RULES.enters, entry: authored::card_4721::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 2, 0, 0], generic: 3 }, rules: &authored::card_4721::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Uncharted Voyage", oracle_id: Some("1d6796ce-9316-430a-8126-8b0414cf181c"), mana_cost: "{3}{U}", type_line: "Instant", oracle_keywords: &["Surveil"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4757::RULES.keywords, deck_limit: authored::card_4757::RULES.deck_limit }, effect: authored::card_4757::RULES.checked_effect(), triggers: authored::card_4757::RULES.triggers, activated: authored::card_4757::RULES.activated, enters: authored::card_4757::RULES.enters, entry: authored::card_4757::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_4757::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Undying Malice", oracle_id: Some("8b061702-1bc3-45aa-a758-446b25034801"), mana_cost: "{B}", type_line: "Instant", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4773::RULES.keywords, deck_limit: authored::card_4773::RULES.deck_limit }, effect: authored::card_4773::RULES.checked_effect(), triggers: authored::card_4773::RULES.triggers, activated: authored::card_4773::RULES.activated, enters: authored::card_4773::RULES.enters, entry: authored::card_4773::RULES.entry, kind: 8, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 0 }, rules: &authored::card_4773::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Unflinching Courage", oracle_id: Some("0e969a27-1609-4ab4-b0db-46b1af8066a9"), mana_cost: "{1}{G}{W}", type_line: "Enchantment — Aura", oracle_keywords: &["Enchant"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4779::RULES.keywords, deck_limit: authored::card_4779::RULES.deck_limit }, effect: authored::card_4779::RULES.checked_effect(), triggers: authored::card_4779::RULES.triggers, activated: authored::card_4779::RULES.activated, enters: authored::card_4779::RULES.enters, entry: authored::card_4779::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [1, 0, 0, 0, 1, 0], generic: 1 }, rules: &authored::card_4779::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Valkyrie's Call", oracle_id: Some("27400927-9aba-4fb3-90c4-490d01969e60"), mana_cost: "{3}{W}{W}", type_line: "Enchantment", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4828::RULES.keywords, deck_limit: authored::card_4828::RULES.deck_limit }, effect: authored::card_4828::RULES.checked_effect(), triggers: authored::card_4828::RULES.triggers, activated: authored::card_4828::RULES.activated, enters: authored::card_4828::RULES.enters, entry: authored::card_4828::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [2, 0, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_4828::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Vampire Gourmand", oracle_id: Some("3386414c-32fa-40ae-9b30-dab976a158c1"), mana_cost: "{1}{B}", type_line: "Creature — Vampire", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4837::RULES.keywords, deck_limit: authored::card_4837::RULES.deck_limit }, effect: authored::card_4837::RULES.checked_effect(), triggers: authored::card_4837::RULES.triggers, activated: authored::card_4837::RULES.activated, enters: authored::card_4837::RULES.enters, entry: authored::card_4837::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 1 }, rules: &authored::card_4837::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Vampire Interloper", oracle_id: Some("b753d7dd-4c6b-4472-a009-671572f927eb"), mana_cost: "{1}{B}", type_line: "Creature — Vampire Scout", oracle_keywords: &["Flying"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4838::RULES.keywords, deck_limit: authored::card_4838::RULES.deck_limit }, effect: authored::card_4838::RULES.checked_effect(), triggers: authored::card_4838::RULES.triggers, activated: authored::card_4838::RULES.activated, enters: authored::card_4838::RULES.enters, entry: authored::card_4838::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 1 }, rules: &authored::card_4838::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Vampire Nighthawk", oracle_id: Some("feb244f8-bcb1-44cf-9940-2719221a7309"), mana_cost: "{1}{B}{B}", type_line: "Creature — Vampire Shaman", oracle_keywords: &["Deathtouch", "Flying", "Lifelink"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4840::RULES.keywords, deck_limit: authored::card_4840::RULES.deck_limit }, effect: authored::card_4840::RULES.checked_effect(), triggers: authored::card_4840::RULES.triggers, activated: authored::card_4840::RULES.activated, enters: authored::card_4840::RULES.enters, entry: authored::card_4840::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 2, 0, 0, 0], generic: 1 }, rules: &authored::card_4840::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Vampire Soulcaller", oracle_id: Some("bbea8ade-a641-4132-b306-f39fe2f14311"), mana_cost: "{4}{B}", type_line: "Creature — Vampire Warlock", oracle_keywords: &["Flying"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4841::RULES.keywords, deck_limit: authored::card_4841::RULES.deck_limit }, effect: authored::card_4841::RULES.checked_effect(), triggers: authored::card_4841::RULES.triggers, activated: authored::card_4841::RULES.activated, enters: authored::card_4841::RULES.enters, entry: authored::card_4841::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 4 }, rules: &authored::card_4841::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Vampiric Rites", oracle_id: Some("660de988-b6fb-4f36-8006-42af3e7f908d"), mana_cost: "{B}", type_line: "Enchantment", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4843::RULES.keywords, deck_limit: authored::card_4843::RULES.deck_limit }, effect: authored::card_4843::RULES.checked_effect(), triggers: authored::card_4843::RULES.triggers, activated: authored::card_4843::RULES.activated, enters: authored::card_4843::RULES.enters, entry: authored::card_4843::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 0 }, rules: &authored::card_4843::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Vanguard Seraph", oracle_id: Some("3250466f-82a4-45e1-9722-ff247ddf8c15"), mana_cost: "{3}{W}", type_line: "Creature — Angel Warrior", oracle_keywords: &["Flying", "Surveil"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4844::RULES.keywords, deck_limit: authored::card_4844::RULES.deck_limit }, effect: authored::card_4844::RULES.checked_effect(), triggers: authored::card_4844::RULES.triggers, activated: authored::card_4844::RULES.activated, enters: authored::card_4844::RULES.enters, entry: authored::card_4844::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_4844::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Vengeful Bloodwitch", oracle_id: Some("4cdbc466-42fc-471f-beab-397caec18101"), mana_cost: "{1}{B}", type_line: "Creature — Vampire Warlock", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4862::RULES.keywords, deck_limit: authored::card_4862::RULES.deck_limit }, effect: authored::card_4862::RULES.checked_effect(), triggers: authored::card_4862::RULES.triggers, activated: authored::card_4862::RULES.activated, enters: authored::card_4862::RULES.enters, entry: authored::card_4862::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 1 }, rules: &authored::card_4862::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Venom Connoisseur", oracle_id: Some("03ebc100-68ac-4d53-adc2-d537212f7d31"), mana_cost: "{1}{G}", type_line: "Creature — Human Druid", oracle_keywords: &["Alliance"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4868::RULES.keywords, deck_limit: authored::card_4868::RULES.deck_limit }, effect: authored::card_4868::RULES.checked_effect(), triggers: authored::card_4868::RULES.triggers, activated: authored::card_4868::RULES.activated, enters: authored::card_4868::RULES.enters, entry: authored::card_4868::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 1 }, rules: &authored::card_4868::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Viashino Pyromancer", oracle_id: Some("5e78e6e0-60f0-4c3c-b8dd-bd673f5152b8"), mana_cost: "{1}{R}", type_line: "Creature — Lizard Wizard", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4881::RULES.keywords, deck_limit: authored::card_4881::RULES.deck_limit }, effect: authored::card_4881::RULES.checked_effect(), triggers: authored::card_4881::RULES.triggers, activated: authored::card_4881::RULES.activated, enters: authored::card_4881::RULES.enters, entry: authored::card_4881::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 1 }, rules: &authored::card_4881::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Vile Entomber", oracle_id: Some("556dd333-4066-41f7-98f6-41794754de71"), mana_cost: "{2}{B}{B}", type_line: "Creature — Zombie Warlock", oracle_keywords: &["Deathtouch"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4892::RULES.keywords, deck_limit: authored::card_4892::RULES.deck_limit }, effect: authored::card_4892::RULES.checked_effect(), triggers: authored::card_4892::RULES.triggers, activated: authored::card_4892::RULES.activated, enters: authored::card_4892::RULES.enters, entry: authored::card_4892::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 2, 0, 0, 0], generic: 2 }, rules: &authored::card_4892::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Vivien Reid", oracle_id: Some("b5f20a6d-8c3e-452f-9d98-4886f0fa052a"), mana_cost: "{3}{G}{G}", type_line: "Legendary Planeswalker — Vivien", oracle_keywords: &[], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_4924::RULES.keywords, deck_limit: authored::card_4924::RULES.deck_limit }, effect: authored::card_4924::RULES.checked_effect(), triggers: authored::card_4924::RULES.triggers, activated: authored::card_4924::RULES.activated, enters: authored::card_4924::RULES.enters, entry: authored::card_4924::RULES.entry, kind: 64, basic: false, mana_color: None, colored: [0, 0, 0, 0, 2, 0], generic: 3 }, rules: &authored::card_4924::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Vizier of the Menagerie", oracle_id: Some("4f890d42-e4f1-4eed-aa6c-718865de8927"), mana_cost: "{3}{G}", type_line: "Creature — Snake Cleric", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4925::RULES.keywords, deck_limit: authored::card_4925::RULES.deck_limit }, effect: authored::card_4925::RULES.checked_effect(), triggers: authored::card_4925::RULES.triggers, activated: authored::card_4925::RULES.activated, enters: authored::card_4925::RULES.enters, entry: authored::card_4925::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 3 }, rules: &authored::card_4925::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Volley Veteran", oracle_id: Some("bf4268e6-170a-445b-b215-718f7494ae28"), mana_cost: "{3}{R}", type_line: "Creature — Goblin Warrior", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4934::RULES.keywords, deck_limit: authored::card_4934::RULES.deck_limit }, effect: authored::card_4934::RULES.checked_effect(), triggers: authored::card_4934::RULES.triggers, activated: authored::card_4934::RULES.activated, enters: authored::card_4934::RULES.enters, entry: authored::card_4934::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 1, 0, 0], generic: 3 }, rules: &authored::card_4934::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Voracious Greatshark", oracle_id: Some("5d96840e-f9b3-4c86-8f59-f12e5736318f"), mana_cost: "{3}{U}{U}", type_line: "Creature — Shark", oracle_keywords: &["Flash"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4935::RULES.keywords, deck_limit: authored::card_4935::RULES.deck_limit }, effect: authored::card_4935::RULES.checked_effect(), triggers: authored::card_4935::RULES.triggers, activated: authored::card_4935::RULES.activated, enters: authored::card_4935::RULES.enters, entry: authored::card_4935::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 2, 0, 0, 0, 0], generic: 3 }, rules: &authored::card_4935::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Wardens of the Cycle", oracle_id: Some("fa7d09aa-2ebc-4571-81da-836cd80cf554"), mana_cost: "{1}{B}{G}{G}", type_line: "Creature — Elf Warlock", oracle_keywords: &["Morbid"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4976::RULES.keywords, deck_limit: authored::card_4976::RULES.deck_limit }, effect: authored::card_4976::RULES.checked_effect(), triggers: authored::card_4976::RULES.triggers, activated: authored::card_4976::RULES.activated, enters: authored::card_4976::RULES.enters, entry: authored::card_4976::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 2, 0], generic: 1 }, rules: &authored::card_4976::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Wary Thespian", oracle_id: Some("448cc829-1596-4aec-b180-c0c1e0566f6b"), mana_cost: "{1}{G}", type_line: "Creature — Cat Druid", oracle_keywords: &["Surveil"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_4989::RULES.keywords, deck_limit: authored::card_4989::RULES.deck_limit }, effect: authored::card_4989::RULES.checked_effect(), triggers: authored::card_4989::RULES.triggers, activated: authored::card_4989::RULES.activated, enters: authored::card_4989::RULES.enters, entry: authored::card_4989::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 1 }, rules: &authored::card_4989::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Wildborn Preserver", oracle_id: Some("55cec4bc-3de6-4c0d-a4cc-c5a30849fbda"), mana_cost: "{1}{G}", type_line: "Creature — Elf Archer", oracle_keywords: &["Reach", "Flash"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_5062::RULES.keywords, deck_limit: authored::card_5062::RULES.deck_limit }, effect: authored::card_5062::RULES.checked_effect(), triggers: authored::card_5062::RULES.triggers, activated: authored::card_5062::RULES.activated, enters: authored::card_5062::RULES.enters, entry: authored::card_5062::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 1 }, rules: &authored::card_5062::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Wildheart Invoker", oracle_id: Some("ff8fb796-af5d-4d64-ab3c-7f2469dfa6bf"), mana_cost: "{2}{G}{G}", type_line: "Creature — Elf Shaman", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_5067::RULES.keywords, deck_limit: authored::card_5067::RULES.deck_limit }, effect: authored::card_5067::RULES.checked_effect(), triggers: authored::card_5067::RULES.triggers, activated: authored::card_5067::RULES.activated, enters: authored::card_5067::RULES.enters, entry: authored::card_5067::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 2, 0], generic: 2 }, rules: &authored::card_5067::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Wildwood Scourge", oracle_id: Some("b9dec104-c636-4770-a7fc-7a3331face15"), mana_cost: "{X}{G}", type_line: "Creature — Hydra", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_5070::RULES.keywords, deck_limit: authored::card_5070::RULES.deck_limit }, effect: authored::card_5070::RULES.checked_effect(), triggers: authored::card_5070::RULES.triggers, activated: authored::card_5070::RULES.activated, enters: authored::card_5070::RULES.enters, entry: authored::card_5070::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 1, 0], generic: 0 }, rules: &authored::card_5070::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Wilt-Leaf Liege", oracle_id: Some("2441696b-a9ba-4813-ba2e-e71f85281d05"), mana_cost: "{1}{G/W}{G/W}{G/W}", type_line: "Creature — Elf Knight", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_5074::RULES.keywords, deck_limit: authored::card_5074::RULES.deck_limit }, effect: authored::card_5074::RULES.checked_effect(), triggers: authored::card_5074::RULES.triggers, activated: authored::card_5074::RULES.activated, enters: authored::card_5074::RULES.enters, entry: authored::card_5074::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 0, 0, 3, 0], generic: 1 }, rules: &authored::card_5074::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Wishclaw Talisman", oracle_id: Some("81c70ae7-3c18-4c9b-8505-e4db9e0e6518"), mana_cost: "{1}{B}", type_line: "Artifact", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_5091::RULES.keywords, deck_limit: authored::card_5091::RULES.deck_limit }, effect: authored::card_5091::RULES.checked_effect(), triggers: authored::card_5091::RULES.triggers, activated: authored::card_5091::RULES.activated, enters: authored::card_5091::RULES.enters, entry: authored::card_5091::RULES.entry, kind: 16, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 1 }, rules: &authored::card_5091::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Witness Protection", oracle_id: Some("a5f5e71e-83a7-468d-829a-6392b764c76e"), mana_cost: "{U}", type_line: "Enchantment — Aura", oracle_keywords: &["Enchant"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_5102::RULES.keywords, deck_limit: authored::card_5102::RULES.deck_limit }, effect: authored::card_5102::RULES.checked_effect(), triggers: authored::card_5102::RULES.triggers, activated: authored::card_5102::RULES.activated, enters: authored::card_5102::RULES.enters, entry: authored::card_5102::RULES.entry, kind: 32, basic: false, mana_color: None, colored: [0, 1, 0, 0, 0, 0], generic: 0 }, rules: &authored::card_5102::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Youthful Valkyrie", oracle_id: Some("6d37ba4b-ff56-4eec-9dc2-2d7f357dc9c9"), mana_cost: "{1}{W}", type_line: "Creature — Angel", oracle_keywords: &["Flying"], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_5144::RULES.keywords, deck_limit: authored::card_5144::RULES.deck_limit }, effect: authored::card_5144::RULES.checked_effect(), triggers: authored::card_5144::RULES.triggers, activated: authored::card_5144::RULES.activated, enters: authored::card_5144::RULES.enters, entry: authored::card_5144::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [1, 0, 0, 0, 0, 0], generic: 1 }, rules: &authored::card_5144::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Zimone, Paradox Sculptor", oracle_id: Some("9dd674a7-becf-4106-b53f-bca88426d92d"), mana_cost: "{2}{G}{U}", type_line: "Legendary Creature — Human Wizard", oracle_keywords: &["Double"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_5165::RULES.keywords, deck_limit: authored::card_5165::RULES.deck_limit }, effect: authored::card_5165::RULES.checked_effect(), triggers: authored::card_5165::RULES.triggers, activated: authored::card_5165::RULES.activated, enters: authored::card_5165::RULES.enters, entry: authored::card_5165::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 1, 0, 0, 1, 0], generic: 2 }, rules: &authored::card_5165::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Zombify", oracle_id: Some("bb95db4d-5017-4121-bf79-d68476602d8c"), mana_cost: "{3}{B}", type_line: "Sorcery", oracle_keywords: &[], standard_legal: true, legendary: false, implemented: true, keywords: authored::card_5169::RULES.keywords, deck_limit: authored::card_5169::RULES.deck_limit }, effect: authored::card_5169::RULES.checked_effect(), triggers: authored::card_5169::RULES.triggers, activated: authored::card_5169::RULES.activated, enters: authored::card_5169::RULES.enters, entry: authored::card_5169::RULES.entry, kind: 4, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 3 }, rules: &authored::card_5169::RULES },
CatalogueEntry { definition: Definition { info: CardInfo { name: "Zul Ashur, Lich Lord", oracle_id: Some("a7c6c9b3-91f2-4ecf-b2b0-b0edd96974d9"), mana_cost: "{1}{B}", type_line: "Legendary Creature — Zombie Warlock", oracle_keywords: &["Ward"], standard_legal: true, legendary: true, implemented: true, keywords: authored::card_5179::RULES.keywords, deck_limit: authored::card_5179::RULES.deck_limit }, effect: authored::card_5179::RULES.checked_effect(), triggers: authored::card_5179::RULES.triggers, activated: authored::card_5179::RULES.activated, enters: authored::card_5179::RULES.enters, entry: authored::card_5179::RULES.entry, kind: 1, basic: false, mana_color: None, colored: [0, 0, 1, 0, 0, 0], generic: 1 }, rules: &authored::card_5179::RULES },
];
}
pub use cards::ENTRIES;
