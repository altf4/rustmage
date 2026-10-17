//! Immutable as-though casting permissions, indexed with battlefield statics.
use super::*;
impl Game {
    pub(super) fn cast_timing_granted(&self, actor: Player, chosen: Card) -> bool {
        self.state.static_sources.iter().any(|&source| {
            let object = &self.state.objects[source as usize];
            object.zone == Zone::Battlefield && object.owner == actor
                && object.card.cast_timing_grants().iter().any(|filter| filter.matches(chosen))
        })
    }
}
