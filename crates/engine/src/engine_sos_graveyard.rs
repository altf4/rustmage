//! Graveyard departure observes one owner batch and the pre-event battlefield (CR603.10).
use super::*;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    /// One triggered packet for the entire simultaneous exile event.
    pub(super) fn creatures_exiled_batch_triggers(&mut self, events: &[ZoneChange]) {
        if !events
            .iter()
            .any(|event| event.moved && event.destination == Zone::Exile)
        {
            return;
        }
        let before = events
            .iter()
            .find_map(|event| event.graveyard_observers.clone());
        if before.is_none()
            && !self.state.death_sources.iter().any(|&id| {
                self.state.objects[id as usize]
                    .card
                    .triggers()
                    .iter()
                    .any(|ability| {
                        ability.condition.event()
                            == TriggerCondition::ControlledCreaturesOrGraveyardCardsExiled
                    })
            })
        {
            return;
        }
        let mut observers: Vec<(ObjectView, bool, bool)> = before
            .as_ref()
            .into_iter()
            .flat_map(|rows| rows.iter())
            .filter(|source| {
                source.printed_abilities_available()
                    && source.card.triggers().iter().any(|ability| {
                        ability.condition.event()
                            == TriggerCondition::ControlledCreaturesOrGraveyardCardsExiled
                    })
            })
            .map(|source| {
                let current = self.on_battlefield(source.id)
                    && self.object_ref(source.id) == source.reference
                    && self.state.objects[source.id as usize].controller == source.controller
                    && self.object_view(source.id).printed_abilities_available();
                (source.clone(), true, current)
            })
            .collect();
        // Graveyard-to-exile triggers use the post-event world, including a
        // source that entered or changed controller during this same batch.
        for source in self.death_trigger_sources() {
            if !source.printed_abilities_available()
                || !source.card.triggers().iter().any(|ability| {
                    ability.condition.event()
                        == TriggerCondition::ControlledCreaturesOrGraveyardCardsExiled
                })
            {
                continue;
            }
            if !observers.iter().any(|(before, _, graveyard)| {
                *graveyard
                    && before.reference == source.reference
                    && before.controller == source.controller
            }) {
                observers.push((source, false, true));
            }
        }
        for (source, battlefield, graveyard) in observers {
            let qualifying: Vec<_> = events
                .iter()
                .filter(|event| {
                    event.moved
                        && event.destination == Zone::Exile
                        && ((battlefield
                            && event.before.zone == Zone::Battlefield
                            && event.before.controller == source.controller
                            && event.before.is_creature())
                            || (graveyard
                                && event.before.zone == Zone::Graveyard
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
                    && self.queue_trigger(&source, index)
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
                        ability.condition.event()
                            == TriggerCondition::ControlledCreaturesOrGraveyardCardsExiled
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
