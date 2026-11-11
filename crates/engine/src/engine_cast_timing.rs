//! Immutable as-though casting permissions, indexed with battlefield statics.
use super::*;
use crate::cards::SelfCastTiming;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn self_cast_timing_applies<W: ability_world::CharacteristicWorld + ?Sized>(
    world: &W,
    actor: Player,
    condition: SelfCastTiming,
) -> bool {
    match condition {
        SelfCastTiming::ControlSubtype(subtype)
        | SelfCastTiming::FlashWhileControlSubtype(subtype) => {
            (0..world.game().state.objects.len()).any(|id| {
                let id = id as CardId;
                let object = world.object(id);
                object.zone == Zone::Battlefield
                    && object.controller == actor
                    && world.subtypes(id).has(world.definition(id), subtype)
            })
        }
        SelfCastTiming::FlashAfterCrime => {
            world.game().state.players[actor.index()].committed_crime_this_turn
        }
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn cast_timing_granted(&self, actor: Player, chosen: Card) -> bool {
        self.cast_timing_granted_with_types(actor, chosen, &[])
    }
    pub(super) fn cast_timing_granted_with_types(
        &self,
        actor: Player,
        chosen: Card,
        additional_types: &[&str],
    ) -> bool {
        // CR 601.3d checks conditional flash before beginning the announcement.
        let own_permission = chosen
            .self_cast_timing()
            .is_some_and(|condition| self_cast_timing_applies(self, actor, condition));
        own_permission
            || self.state.static_sources.iter().any(|&source| {
                let object = &self.state.objects[source as usize];
                object.zone == Zone::Battlefield
                    && self.printed_abilities_available(source)
                    && object.controller == actor
                    && (!object.card.cast_timing_requires_opponent_spell()
                        || self.state.players[actor.other().index()].spells_cast_this_turn > 0)
                    && self.step_guard(
                        object.card.cast_timing_guard(),
                        actor,
                        self.object_ref(source),
                        object.tapped,
                    )
                    && object.card.cast_timing_grants().iter().any(|filter| {
                        filter.matches_mana_value_with(
                            chosen,
                            chosen.fixed_mana_value(),
                            |subtype| {
                                additional_types.contains(&subtype) || chosen.has_subtype(subtype)
                            },
                        )
                    })
            })
    }
}
