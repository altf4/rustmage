use super::*;
use super::*;
use crate::cards::StaticAbilityFragment;
use ability_world::CharacteristicWorld;
use cast_cost_planning::CostContext;
use funding_entries::{EntryProjection, PaymentContinuation};
pub use rustmage_core::engine::funding::*;
use std::collections::HashSet;
#[cfg(test)]
#[path = "assurance/hob_beorn_funding.rs"]
mod beorn_tests;
#[path = "engine_cast_cost_planning.rs"]
mod cast_cost_planning;
#[cfg(test)]
#[path = "assurance/hob_fili_funding.rs"]
mod fili_tests;
#[path = "engine_funding_entries.rs"]
mod funding_entries;
#[cfg(test)]
#[path = "assurance/hob_supper_funding.rs"]
mod hob_supper_funding;
#[cfg(test)]
#[path = "assurance/hob_thranduil_funding.rs"]
mod hob_thranduil_funding;
#[cfg(test)]
#[path = "assurance/hob_hone_funding.rs"]
mod hone_tests;
#[cfg(test)]
#[path = "assurance/hob_notary_funding.rs"]
mod notary_tests;
#[cfg(test)]
#[path = "engine_requested_case_funding_tests.rs"]
mod requested_case_funding_tests;
#[cfg(test)]
#[path = "engine_requested_curator_funding_tests.rs"]
mod requested_curator_funding_tests;
#[cfg(test)]
#[path = "engine_requested_esper_funding_tests.rs"]
mod requested_esper_funding_tests;
#[cfg(test)]
#[path = "engine_requested_essence_funding_tests.rs"]
mod requested_essence_funding_tests;
#[cfg(test)]
#[path = "engine_requested_ghost_projection_tests.rs"]
mod requested_ghost_projection_tests;
#[cfg(test)]
#[path = "engine_funding_sacrifice_tests.rs"]
mod sacrifice_tests;
#[cfg(test)]
#[path = "engine_funding_storied_tests.rs"]
mod storied_tests;
#[cfg(test)]
mod fra_subtype_grant_tests {
    use super::*;
    use crate::engine::assurance::big_helpers::{named, setup};

    #[test]
    fn funding_world_respects_controlled_thopter_haste() {
        for p in [Player::P0, Player::P1] {
            let mut g = setup(p);
            // The definition is still gated, so install it into this isolated fixture.
            let saheeli = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            g.state.objects[saheeli as usize].card =
                Card::from_name("Saheeli, Jewel of Avishkar").unwrap();
            g.register_battlefield_sources(saheeli);
            assert!(g.state.static_sources.contains(&saheeli));
            g.create_tokens(p, Card::test_index(5266), 1, false);
            let thopter = g
                .state
                .objects
                .iter()
                .enumerate()
                .find_map(|(i, o)| {
                    (o.zone == Zone::Battlefield
                        && o.card == Card::test_index(5266)
                        && o.controller == p)
                        .then_some(i as CardId)
                })
                .expect("created Thopter");
            let goal = FundingGoal {
                actor: p,
                required: [0; 6],
                generic: 1,
                reserve: None,
                source: None,
            };
            // Ensure this creature is represented in the projected funding graph.
            let object = g.object_ref(thopter);
            let goal = FundingGoal {
                reserve: Some((
                    object,
                    AdditionalCost {
                        kind: EffectChoiceKind::Sacrifice,
                        filter: SacrificeFilter {
                            keywords: Keywords::NONE,
                            source_only: false,
                            kinds: 1,
                            nonland: false,
                            other: false,
                            tokens: None,
                            artifact_enchantment_or_token: false,
                            subtype: None,
                        },
                    },
                )),
                ..goal
            };
            let world = FundingState::from_game(&g, &goal);
            let resource = world.resources.iter().find(|r| r.id == thopter).unwrap();
            assert!(world.keywords(resource).contains(Keywords::HASTE));
            let mut without_source = world.clone();
            let provider = without_source
                .resources
                .iter_mut()
                .find(|r| r.id == saheeli)
                .unwrap();
            provider.object.zone = Zone::Graveyard;
            let resource = without_source
                .resources
                .iter()
                .find(|r| r.id == thopter)
                .unwrap();
            assert!(!without_source.keywords(resource).contains(Keywords::HASTE));
        }
    }
}
#[cfg(test)]
#[path = "engine_fra_behold_funding_tests.rs"]
mod engine_fra_behold_funding_tests;
#[cfg(test)]
#[path = "engine_fra_jace_funding_tests.rs"]
mod engine_fra_jace_funding_tests;
#[cfg(test)]
#[path = "engine_fra_manifest_cost_departures_tests.rs"]
mod engine_fra_manifest_cost_departures_tests;
#[cfg(test)]
#[path = "engine_fra_ruric_fast_projection_tests.rs"]
mod engine_fra_ruric_fast_projection_tests;
#[cfg(test)]
#[path = "engine_fra_slow_departure_tests.rs"]
mod engine_fra_slow_departure_tests;
