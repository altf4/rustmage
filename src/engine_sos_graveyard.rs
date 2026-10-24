//! One trigger per owner and simultaneous physical-card graveyard departure batch.
use super::*;
impl Game {
    pub(super) fn graveyard_departure_triggers(&mut self, events: &[ZoneChange], before: &[ObjectView]) {
        let departed = std::array::from_fn::<_, 2, _>(|seat| events.iter().any(|e|
            e.moved && e.before.zone == Zone::Graveyard && e.destination != Zone::Graveyard
            && e.before.owner.index() == seat && !e.before.is_token() && !e.before.card.is_emblem()));
        if !departed.iter().any(|left| *left) { return; }
        // CR603.10: graveyard-departure abilities look back. Exclude newcomers
        // and retain a source that departed in this same simultaneous movement.
        let mut observers: Vec<_> = self.state.objects.iter().enumerate()
            .filter(|(_, o)| o.zone == Zone::Battlefield && !o.card.triggers().is_empty())
            .map(|(id, _)| self.object_view(id as CardId))
            .filter(|o| !events.iter().any(|e| e.moved && e.destination == Zone::Battlefield && e.after == o.reference))
            .collect();
        for source in before.iter().chain(events.iter().filter(|e| e.moved && e.before.zone == Zone::Battlefield).map(|e| &e.before)) {
            if !observers.iter().any(|o| o.reference == source.reference) { observers.push(source.clone()); }
        }
        for source in observers {
            if !departed[source.controller.index()] { continue; }
            for (index, ability) in source.card.triggers().iter().enumerate() {
                if ability.condition.event() == TriggerCondition::CardsLeaveGraveyard {
                    self.queue_trigger(&source, index);
                }
            }
        }
    }
}
