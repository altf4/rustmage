//! Effective base-power consumers use the same layer boundary as ordinary stats.
use super::*;

impl Game {
    pub(super) fn static_casting_prohibited(&self, actor: Player) -> bool {
        self.state.static_sources.iter().any(|&id| {
            let object = &self.state.objects[id as usize];
            object.zone == Zone::Battlefield
                && self.printed_abilities_available(id)
                && object.controller == self.state.active
                && object.controller != actor
                && object.card.static_cast_restriction() & 1 != 0
        })
    }
    pub(super) fn put_power_excess_counters(&mut self, item: &SpellView, filter: CreatureFilter) {
        // Capture the entire simultaneous batch before any counter replacement
        // or placement can change a later recipient's characteristics.
        let changes = self
            .state
            .objects
            .iter()
            .enumerate()
            .filter_map(|(index, _)| {
                let id = index as CardId;
                if !self.creature_matches(filter, item.controller, item.object, id) {
                    return None;
                }
                let (base, current) = self.layered_stats_without_hand(id, None);
                if current.power <= base.power {
                    return None;
                }
                let amount = current
                    .power
                    .checked_sub(base.power)
                    .capacity("power difference exhausted");
                let amount = u64::try_from(amount).capacity("counter amount exhausted");
                let counters = Counters {
                    plus_one: amount,
                    ..Counters::default()
                };
                Some((self.object_ref(id), counters))
            })
            .collect();
        self.effect_counters(changes, replacements::ResumeMode::Program);
    }
}
