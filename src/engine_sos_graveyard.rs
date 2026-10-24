//! Graveyard departure observes one owner batch and the pre-event battlefield (CR603.10).
use super::*;
impl Game {
    pub(super) fn freeze_graveyard_observers(&self) -> Option<std::sync::Arc<[ObjectView]>> {
        let sources: Vec<_> = self.state.objects.iter().enumerate()
            .filter(|(_,o)| o.zone == Zone::Battlefield && o.card.triggers().iter().any(|a| a.condition.event() == TriggerCondition::CardsLeaveGraveyard))
            .map(|(id,_)| self.object_view(id as CardId)).collect();
        (!sources.is_empty()).then(|| sources.into())
    }
    pub(super) fn graveyard_departure_triggers(&mut self, events: &[ZoneChange], _before: &[ObjectView]) {
        let departed = std::array::from_fn::<_, 2, _>(|seat| events.iter().any(|e|
            e.moved && e.before.zone == Zone::Graveyard && e.destination != Zone::Graveyard
            && e.before.owner.index() == seat && !e.before.is_token() && !e.before.card.is_emblem()));
        if !departed.iter().any(|left| *left) { return; }
        let Some(observers) = events.iter().find_map(|e| e.graveyard_observers.clone()) else { return; };
        for source in observers.iter() {
            if !departed[source.controller.index()] { continue; }
            for (index, ability) in source.card.triggers().iter().enumerate() {
                if ability.condition.event() == TriggerCondition::CardsLeaveGraveyard { self.queue_trigger(source, index); }
            }
        }
    }
}
