//! Immutable as-though casting permissions, indexed with battlefield statics.
use super::*;
impl Game {
    pub(super) fn cast_timing_granted(&self, actor: Player, chosen: Card) -> bool {
        // This is an announcement permission, never a keyword and never
        // rechecked during mana abilities/payment after legality is approved.
        let own_permission = chosen
            .self_cast_timing()
            .is_some_and(|condition| match condition {
                crate::cards::SelfCastTiming::ControlSubtype(subtype) => {
                    self.state.objects.iter().enumerate().any(|(id, object)| {
                        object.zone == Zone::Battlefield
                            && object.controller == actor
                            && self
                                .granted_subtypes(id as CardId)
                                .has(object.card, subtype)
                    })
                }
            });
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
                    && object
                        .card
                        .cast_timing_grants()
                        .iter()
                        .any(|filter| filter.matches(chosen))
            })
    }
}
