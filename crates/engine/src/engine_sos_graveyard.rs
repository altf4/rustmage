//! Graveyard arrivals use committed objects. Departures observe the pre-event battlefield.
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
                && source.trigger_templates().any(|(_, ability)| {
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
                .filter(|event| event.was_card)
                .flat_map(|event| event.objects_after())
                .filter(|&object| {
                    self.object_ref(object.card) == object
                        && self.state.objects[object.card as usize].zone == Zone::Exile
                        && self.state.objects[object.card as usize].is_card()
                        && !self.exiled_face_down(object)
                        && self.current_kinds(object.card) & 1 != 0
                })
                .collect();
            for (index, ability) in source.trigger_templates() {
                if ability.condition.event()
                    == TriggerCondition::ControlledCreaturesOrGraveyardCardsExiled
                    && self.queue_trigger(source, index)
                {
                    let packet = self.state.pending_triggers.last_mut().unwrap();
                    for event in &qualifying {
                        packet.refer_zone_event(event);
                    }
                    packet.exiled_trigger_cards = Some(Box::new(cards.clone()));
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
            for (index, ability) in source.trigger_templates() {
                if ability.condition.event()
                    == TriggerCondition::CardsExiledFromBattlefieldOrGraveyard
                    && self.queue_trigger(source, index)
                {
                    let packet = self.state.pending_triggers.last_mut().unwrap();
                    for event in events.iter().filter(|event| {
                        event.moved
                            && event.was_card
                            && event.destination == Zone::Exile
                            && matches!(event.before.zone, Zone::Battlefield | Zone::Graveyard)
                    }) {
                        packet.refer_zone_event(event);
                    }
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
                self.has_trigger(id, |ability| {
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
            .filter(|(id, o)| {
                o.zone == Zone::Battlefield
                    && self.has_trigger(*id as CardId, |a| {
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
            for (index, ability) in source.trigger_templates() {
                match ability.condition.event() {
                    TriggerCondition::CardsLeaveGraveyardKinds(_)
                    | TriggerCondition::CardsLeaveGraveyard => {
                        let kinds = match ability.condition.event() {
                            TriggerCondition::CardsLeaveGraveyardKinds(kinds) => kinds,
                            _ => u16::MAX,
                        };
                        let matching: Vec<_> = events
                            .iter()
                            .filter(|event| {
                                event.moved
                                    && event.before.zone == Zone::Graveyard
                                    && event.destination != Zone::Graveyard
                                    && event.before.owner == source.controller
                                    && event.was_card
                                    && event.before.kinds & kinds != 0
                            })
                            .collect();
                        if !matching.is_empty() && self.queue_trigger(source, index) {
                            let packet = self.state.pending_triggers.last_mut().unwrap();
                            for event in matching {
                                packet.refer_zone_event(event);
                            }
                        }
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
        self.card_arrivals_in_graveyard(
            events,
            1,
            TriggerCondition::CreatureCardsEnterYourGraveyard,
            None,
        );
    }

    pub(super) fn permanent_cards_enter_graveyard_triggers(&mut self, events: &[ZoneChange]) {
        self.card_arrivals_in_graveyard(
            events,
            crate::cards::PERMANENT_KINDS,
            TriggerCondition::PermanentCardsEnterYourGraveyard,
            None,
        );
    }

    pub(super) fn land_cards_enter_graveyard_from_library_triggers(
        &mut self,
        events: &[ZoneChange],
    ) {
        self.card_arrivals_in_graveyard(
            events,
            2,
            TriggerCondition::LandCardsEnterYourGraveyardFromLibrary,
            Some(Zone::Library),
        );
    }

    fn card_arrivals_in_graveyard(
        &mut self,
        events: &[ZoneChange],
        kinds: u16,
        condition: TriggerCondition,
        from: Option<Zone>,
    ) {
        let entered: [bool; 2] = [Player::P0, Player::P1].map(|owner| {
            self.graveyard_arrivals(events, owner, kinds, from)
                .next()
                .is_some()
        });
        if !entered.into_iter().any(|value| value) {
            return;
        }
        let sources = self.indexed_trigger_sources(&self.state.death_sources);
        for (source, grant) in sources {
            if source.zone != Zone::Battlefield || !entered[source.controller.index()] {
                continue;
            }
            for (index, ability) in source.trigger_templates_from(grant.as_ref()) {
                if ability.condition.event() != condition {
                    continue;
                }
                let matching = self
                    .graveyard_arrivals(events, source.controller, kinds, from)
                    .collect::<Vec<_>>();
                if self.queue_trigger_from(&source, index, grant.map(Box::new)) {
                    if source.additional_creature_death_triggers != 0
                        && events.iter().any(|event| {
                            event.is_creature_death()
                                && event
                                    .objects_after()
                                    .any(|object| matching.contains(&object))
                        })
                    {
                        self.add_event_trigger_multiplicity(
                            source.additional_creature_death_triggers,
                        );
                    }
                    self.state
                        .pending_triggers
                        .last_mut()
                        .unwrap()
                        .refer_objects(matching);
                }
            }
        }
    }
    fn graveyard_arrivals<'a>(
        &'a self,
        events: &'a [ZoneChange],
        owner: Player,
        kinds: u16,
        from: Option<Zone>,
    ) -> impl Iterator<Item = ObjectRef> + 'a {
        events
            .iter()
            .filter(move |event| {
                event.moved
                    && event.destination == Zone::Graveyard
                    && event.before.zone != Zone::Graveyard
                    && from.is_none_or(|zone| event.before.zone == zone)
                    && event.before.owner == owner
                    && event.was_card
            })
            .flat_map(ZoneChange::objects_after)
            .filter(move |&object| {
                self.object_ref(object.card) == object
                    && self.state.objects[object.card as usize].zone == Zone::Graveyard
                    && self.current_kinds(object.card) & kinds != 0
            })
    }

    pub(super) fn return_graveyard_arrival_batch(&mut self, item: &SpellView, tapped: bool) {
        let Some(event) = &item.event_object else {
            return;
        };
        let arrivals = std::iter::once(event.object).chain(event.semantic_objects.iter().flat_map(
            |objects| {
                objects.iter().filter_map(|source| match source {
                    DamageSourceRef::Object(object) => Some(*object),
                    DamageSourceRef::StackObject { .. } => None,
                })
            },
        ));
        let moves = arrivals
            .filter_map(|reference| {
                let object = self.state.objects.get(reference.card as usize)?;
                (object.zone == Zone::Graveyard
                    && object.owner == item.controller
                    && object.is_card()
                    && self.object_ref(reference.card) == reference
                    && self.current_kinds(reference.card) & crate::cards::PERMANENT_KINDS != 0)
                    .then(|| self.entry_move(reference.card, Zone::Battlefield, tapped))
            })
            .collect();
        self.stage_entries(moves, None);
    }
}
