//! Graveyard departure observes one owner batch and the pre-event battlefield (CR603.10).
use super::*;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    /// One triggered packet for the entire simultaneous exile event.
    pub(super) fn creatures_exiled_batch_triggers(&mut self, events: &[ZoneChange]) {
        self.cards_exiled_from_public_zones_triggers(events);
        if !events
            .iter()
            .any(|event| event.moved && event.destination == Zone::Exile)
        {
            return;
        }
        let Some(observers) = events
            .iter()
            .find_map(|event| event.graveyard_observers.as_ref())
        else {
            return;
        };
        // CR603.10a: both battlefield and graveyard departures look back.
        // A source entering or regaining its ability during this event cannot
        // observe it; a departing or newly blanked source can.
        for source in observers.iter().filter(|source| {
            source.printed_abilities_available()
                && source.card.triggers().iter().any(|ability| {
                    ability.condition.event()
                        == TriggerCondition::ControlledCreaturesOrGraveyardCardsExiled
                })
        }) {
            let qualifying: Vec<_> = events
                .iter()
                .filter(|event| {
                    event.moved
                        && event.destination == Zone::Exile
                        && ((event.before.zone == Zone::Battlefield
                            && event.before.controller == source.controller
                            && event.before.is_creature())
                            || (event.before.zone == Zone::Graveyard
                                && event.before.owner == source.controller
                                && event.was_card
                                && event.before.is_creature()))
                })
                .collect();
            if qualifying.is_empty() {
                continue;
            }
            let cards: Vec<_> = qualifying
                .iter()
                .filter_map(|event| {
                    (event.was_card
                        && self.object_ref(event.after.card) == event.after
                        && self.state.objects[event.after.card as usize].zone == Zone::Exile
                        && self.state.objects[event.after.card as usize].is_card()
                        && self.current_kinds(event.after.card) & 1 != 0)
                        .then_some(event.after)
                })
                .collect();
            for (index, ability) in source.card.triggers().iter().enumerate() {
                if ability.condition.event()
                    == TriggerCondition::ControlledCreaturesOrGraveyardCardsExiled
                    && self.queue_trigger(source, index)
                {
                    self.state
                        .pending_triggers
                        .last_mut()
                        .unwrap()
                        .exiled_trigger_cards = Some(Box::new(cards.clone()));
                }
            }
        }
    }
    /// Both public-zone departures use the same pre-event battlefield witnesses.
    fn cards_exiled_from_public_zones_triggers(&mut self, events: &[ZoneChange]) {
        if !events.iter().any(|event| {
            event.moved
                && event.was_card
                && event.destination == Zone::Exile
                && matches!(event.before.zone, Zone::Battlefield | Zone::Graveyard)
        }) {
            return;
        }
        let Some(observers) = events
            .iter()
            .find_map(|event| event.graveyard_observers.as_ref())
        else {
            return;
        };
        for source in observers
            .iter()
            .filter(|source| source.printed_abilities_available())
        {
            for (index, ability) in source.card.triggers().iter().enumerate() {
                if ability.condition.event()
                    == TriggerCondition::CardsExiledFromBattlefieldOrGraveyard
                {
                    self.queue_trigger(source, index);
                }
            }
        }
    }
    /// Freeze only when a battlefield observer needs an exile-batch witness.
    pub(super) fn freeze_exile_batch_observers(&self) -> Option<std::sync::Arc<[ObjectView]>> {
        self.state
            .death_sources
            .iter()
            .any(|&id| {
                self.state.objects[id as usize]
                    .card
                    .triggers()
                    .iter()
                    .any(|ability| {
                        matches!(
                            ability.condition.event(),
                            TriggerCondition::ControlledCreaturesOrGraveyardCardsExiled
                                | TriggerCondition::CardsExiledFromBattlefieldOrGraveyard
                        )
                    })
            })
            .then(|| self.freeze_graveyard_observers())
            .flatten()
    }
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
                                | TriggerCondition::CardsLeaveGraveyardKinds(_)
                                | TriggerCondition::CardsExiledFromBattlefieldOrGraveyard
                                | TriggerCondition::CreatureCardLeavesYourGraveyard
                                | TriggerCondition::ControlledCreaturesOrGraveyardCardsExiled
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
                    TriggerCondition::CardsLeaveGraveyardKinds(kinds) => {
                        if events.iter().any(|event| {
                            event.moved
                                && event.before.zone == Zone::Graveyard
                                && event.destination != Zone::Graveyard
                                && event.before.owner == source.controller
                                && event.was_card
                                && event.before.kinds & kinds != 0
                        }) {
                            self.queue_trigger(source, index);
                        }
                    }
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

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
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
    pub(super) fn permanent_cards_enter_graveyard_triggers(&mut self, events: &[ZoneChange]) {
        let entered = std::array::from_fn::<_, 2, _>(|seat| {
            events.iter().any(|e| {
                e.moved
                    && e.destination == Zone::Graveyard
                    && e.before.zone != Zone::Graveyard
                    && e.before.owner.index() == seat
                    && e.was_card
                    && self.object_ref(e.after.card) == e.after
                    && self.state.objects[e.after.card as usize].zone == Zone::Graveyard
                    && self.current_kinds(e.after.card) & (1 | 2 | 16 | 32 | 64 | 256) != 0
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
                if ability.condition.event() == TriggerCondition::PermanentCardsEnterYourGraveyard {
                    self.queue_trigger_from(&source, index, grant.map(Box::new));
                }
            }
        }
    }
}
