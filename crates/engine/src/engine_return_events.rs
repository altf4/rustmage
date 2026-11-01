//! Actual battlefield-to-owner-hand events use pre-whole-event observer facts.
use super::*;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn freeze_returned_hand_observers(&self) -> Option<std::sync::Arc<[ObjectView]>> {
        let sources: Vec<_> = self
            .state
            .death_sources
            .iter()
            .filter(|&&id| {
                self.state.objects[id as usize]
                    .card
                    .triggers()
                    .iter()
                    .any(|a| {
                        a.condition.event()
                            == TriggerCondition::OtherControlledNonlandReturnedToOwnersHand
                    })
            })
            .map(|&id| self.object_view(id))
            .collect();
        (!sources.is_empty()).then(|| sources.into())
    }
    pub(super) fn returned_hand_triggers(&mut self, events: &[ZoneChange]) {
        for event in events {
            if !event.moved
                || event.before.zone != Zone::Battlefield
                || event.destination != Zone::Hand
                || event.before.kinds & 2 != 0
            {
                continue;
            }
            let Some(sources) = &event.returned_hand_observers else {
                continue;
            };
            for source in sources.iter() {
                if source.reference == event.before.reference
                    || source.controller != event.before.controller
                {
                    continue;
                }
                for (index, ability) in source.card.triggers().iter().enumerate() {
                    if ability.condition.event()
                        == TriggerCondition::OtherControlledNonlandReturnedToOwnersHand
                    {
                        self.queue_zone_trigger_from(source, index, event, None);
                    }
                }
            }
        }
    }
}
