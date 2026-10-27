//! Batch death observers retain pre-event source facts; return conditions commit
//! their program branch before linked zone events can suspend the resolution.
use super::*;
impl Game {
    pub(super) fn creature_death_batch_triggers(
        &mut self,
        events: &[ZoneChange],
        sources: &[ObjectView],
    ) {
        for source in sources {
            for (index, ability) in source.card.triggers().iter().enumerate() {
                let TriggerCondition::CreatureDiesBatch { filter } = ability.condition.event()
                else {
                    continue;
                };
                let event = events.iter().find(|event| {
                    event.moved
                        && event.before.zone == Zone::Battlefield
                        && event.destination == Zone::Graveyard
                        && event.before.is_creature()
                        && filter.matches_characteristics(
                            event.before.card,
                            event.before.is_token(),
                            event.before.kinds,
                            event.before.granted_subtypes,
                            event.before.controller == source.controller,
                            event.before.reference == source.reference,
                            event.before.attacking,
                        )
                        && event.before.keywords.contains(filter.keyword)
                });
                if let Some(event) = event {
                    // This queues one annotated original. Its frozen multiplicity
                    // witness expands only after all event annotations are complete.
                    self.queue_zone_trigger_from(source, index, event, None);
                }
            }
        }
    }
    pub(super) fn return_targets_to_hand_or_skip(&mut self, item: &SpellView, skip: usize) {
        let mut ids = Vec::new();
        for bound in &item.targets {
            if self.same_target_object(*bound)
                && let Target::Card(id) = bound.target
                && self.on_battlefield(id)
                && !ids.contains(&id)
            {
                ids.push(id);
            }
        }
        let events = self.move_objects_raw(&ids, Zone::Hand);
        if !events.iter().any(|event| {
            event.moved && event.before.zone == Zone::Battlefield && event.destination == Zone::Hand
        }) {
            self.state.resolving.as_mut().unwrap().pc += skip;
        }
        // Exact success is already committed, including returned tokens. A linked
        // return can now pause for entry choices without repeating this movement.
        self.finish_zone_events(&events, &[]);
    }
}
