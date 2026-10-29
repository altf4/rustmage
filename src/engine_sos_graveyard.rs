//! Graveyard departure observes one owner batch and the pre-event battlefield (CR603.10).
use super::*;
impl Game {
    pub(super) fn freeze_graveyard_observers(&self) -> Option<std::sync::Arc<[ObjectView]>> {
        let sources: Vec<_> = self
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(_, o)| {
                o.zone == Zone::Battlefield
                    && o.card.triggers().iter().any(|a| {
                        matches!(
                            a.condition.event(),
                            TriggerCondition::CardsLeaveGraveyard
                                | TriggerCondition::CreatureCardLeavesYourGraveyard
                        )
                    })
            })
            .map(|(id, _)| self.object_view(id as CardId))
            .collect();
        (!sources.is_empty()).then(|| sources.into())
    }
    pub(super) fn graveyard_departure_triggers(
        &mut self,
        events: &[ZoneChange],
        _before: &[ObjectView],
    ) {
        let departed = std::array::from_fn::<_, 2, _>(|seat| {
            events.iter().any(|e| {
                e.moved
                    && e.before.zone == Zone::Graveyard
                    && e.destination != Zone::Graveyard
                    && e.before.owner.index() == seat
                    && e.was_card
            })
        });
        if !departed.iter().any(|left| *left) {
            return;
        }
        let Some(observers) = events.iter().find_map(|e| e.graveyard_observers.clone()) else {
            return;
        };
        for source in observers.iter() {
            if !departed[source.controller.index()] {
                continue;
            }
            for (index, ability) in source.card.triggers().iter().enumerate() {
                match ability.condition.event() {
                    TriggerCondition::CardsLeaveGraveyard => {
                        self.queue_trigger(source, index);
                    }
                    TriggerCondition::CreatureCardLeavesYourGraveyard => {
                        for event in events.iter().filter(|event| {
                            event.moved
                                && event.before.zone == Zone::Graveyard
                                && event.destination != Zone::Graveyard
                                && event.before.owner == source.controller
                                && event.was_card
                                && event.before.is_creature()
                        }) {
                            // Preserve the original pre-event source and its multiplier
                            // witness; annotate each original before prepare expands it.
                            self.queue_zone_trigger_from(source, index, event, None);
                        }
                    }
                    _ => (),
                }
            }
        }
    }
}

impl Game {
    /// Ordinary from-anywhere triggers inspect the committed new zone, not dying
    /// characteristics or pre-event observers. Tokens never become creature cards.
    pub(super) fn creature_cards_enter_graveyard_triggers(&mut self, events: &[ZoneChange]) {
        let entered = std::array::from_fn::<_, 2, _>(|seat| {
            events.iter().any(|e| {
                e.moved
                    && e.destination == Zone::Graveyard
                    && e.before.zone != Zone::Graveyard
                    && e.before.owner.index() == seat
                    && e.was_card
                    && self.object_ref(e.after.card) == e.after
                    && self.state.objects[e.after.card as usize].zone == Zone::Graveyard
                    && self.current_kinds(e.after.card) & 1 != 0
            })
        });
        if !entered.iter().any(|v| *v) {
            return;
        }
        let sources = self.indexed_trigger_sources(&self.state.death_sources);
        for (source, grant) in sources {
            if source.zone != Zone::Battlefield || !entered[source.controller.index()] {
                continue;
            }
            let definition = grant.as_ref().map_or(source.card, |g| g.granting_card);
            for (index, ability) in definition.triggers().iter().enumerate() {
                if ability.condition.event() == TriggerCondition::CreatureCardsEnterYourGraveyard {
                    self.queue_trigger_from(&source, index, grant.map(Box::new));
                }
            }
        }
    }
}
