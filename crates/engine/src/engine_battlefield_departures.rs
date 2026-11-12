//! Battlefield leave observers use the exact pre-event source and recipient.
use super::*;
use crate::cards::{DepartureCounters, DepartureFilter, DepartureRule, DepartureSubject};
use std::sync::Arc;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct ChosenCreatureLinkView {
    pub source: ObjectRef,
    pub definition: Card,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<u64>,
    pub chosen: ObjectRef,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct DepartureObserver {
    source: ObjectView,
    chosen_creature: Option<ObjectRef>,
    greatest_opposing_power: Option<i128>,
}

impl DepartureRule {
    fn from_legacy(filter: DepartureFilter) -> Self {
        Self {
            kinds: filter.kinds,
            relation: filter.relation,
            subject: if filter.other {
                DepartureSubject::Other
            } else {
                DepartureSubject::Any
            },
            destinations: if filter.graveyard_only {
                &[Zone::Graveyard]
            } else {
                &[]
            },
            counters: filter.counters,
            ..Self::ANY
        }
    }

    fn matches(self, source: &ObjectView, event: &ZoneChange) -> bool {
        if !self.destinations.is_empty() && !self.destinations.contains(&event.destination) {
            return false;
        }
        let before = &event.before;
        let counters_match = match self.counters {
            DepartureCounters::Unrestricted => true,
            DepartureCounters::Any => before.counters.has_any(),
            DepartureCounters::Kind(kind) => before.counters.get(kind) != 0,
        };
        if !counters_match {
            return false;
        }
        if self.subject == DepartureSubject::SelfOrOtherMatching
            && source.reference == before.reference
        {
            return true;
        }
        let related = match self.relation {
            CastRelation::You => source.controller == before.controller,
            CastRelation::Opponent => source.controller != before.controller,
            CastRelation::Any => true,
        };
        let subject_matches = match self.subject {
            DepartureSubject::Any | DepartureSubject::SelfOrOtherMatching => true,
            DepartureSubject::Other => source.reference != before.reference,
            DepartureSubject::SelfOnly => source.reference == before.reference,
        };
        related
            && subject_matches
            && before.keywords.contains(self.keywords)
            && (!self.nontoken || !before.is_token())
            && (!self.modified || before.modified)
            && (self.kinds == 0 || before.kinds & self.kinds != 0)
            && self
                .subtype
                .is_none_or(|subtype| before.has_subtype(subtype))
            && self
                .minimum_power
                .is_none_or(|power| before.is_creature() && before.power >= i128::from(power))
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Debug, PartialEq, Eq)]
pub(super) struct DepartureObservers {
    sources: Vec<DepartureObserver>,
    mana_values: Vec<(ObjectRef, u64)>,
}

impl DepartureObservers {
    pub(super) fn mana_value(&self, object: ObjectRef) -> Option<u64> {
        self.mana_values
            .iter()
            .find_map(|&(reference, value)| (object == reference).then_some(value))
    }
}

impl Game {
    pub(super) fn finish_chosen_creature(&mut self, chosen: Option<ObjectRef>, stats: Stats) {
        let item = &self.state.resolving.as_ref().unwrap().item;
        let source = item.source.expect("linked creature choice source").object;
        let recipients = self.player_creature_recipients(PlayerSet::Each, item);
        if let Some(chosen) = chosen
            && self.on_battlefield(source.card)
            && self.object_ref(source.card) == source
        {
            let definition = item.trigger_definition();
            let origin = item.source_ability_origin;
            let links = &mut self
                .state
                .roles
                .get_or_insert_default()
                .chosen_creature_links;
            if let Some(link) = links.iter_mut().find(|link| {
                link.source == source && link.definition == definition && link.origin == origin
            }) {
                link.chosen = chosen;
            } else {
                links.push(ChosenCreatureLinkView {
                    source,
                    definition,
                    origin,
                    chosen,
                });
            }
        }
        for id in recipients {
            let reference = self.object_ref(id);
            if reference != source && Some(reference) != chosen {
                self.state.objects[id as usize].growth += stats;
            }
        }
    }

    pub(super) fn freeze_battlefield_departure_observers(
        &self,
        departing: impl IntoIterator<Item = ObjectRef>,
    ) -> Option<Arc<DepartureObservers>> {
        let mut greatest_power = None;
        let sources: Vec<_> = self
            .state
            .death_sources
            .iter()
            .copied()
            .filter_map(|id| {
                let object = &self.state.objects[id as usize];
                if object.zone != Zone::Battlefield {
                    return None;
                }
                let triggers = object.card.triggers();
                let chosen_creature = if triggers.iter().any(|ability| {
                    ability.condition.event() == TriggerCondition::ChosenCreatureLeaves
                }) {
                    let source = self.object_ref(id);
                    self.state.roles.as_ref().and_then(|roles| {
                        roles.chosen_creature_links.iter().find_map(|link| {
                            (link.source == source
                                && link.definition == object.card
                                && link.origin == self.copy_ability_origin(source))
                            .then_some(link.chosen)
                        })
                    })
                } else {
                    None
                };
                let observes_greatest_power = triggers.iter().any(|ability| {
                    ability.condition.event() == TriggerCondition::OpponentHighestPowerCreatureDies
                });
                let greatest_opposing_power = observes_greatest_power
                    .then(|| {
                        greatest_power.get_or_insert_with(|| {
                            let mut power: [Option<i128>; 2] = [None; 2];
                            let query = self.characteristic_query();
                            for (id, creature) in self.state.objects.iter().enumerate() {
                                let id = id as CardId;
                                if creature.zone == Zone::Battlefield
                                    && ability_world::CharacteristicWorld::kinds(&query, id) & 1
                                        != 0
                                {
                                    let candidate = query.stats(id).power;
                                    let highest = &mut power[creature.controller.index()];
                                    *highest =
                                        Some(highest.map_or(candidate, |old| old.max(candidate)));
                                }
                            }
                            power
                        })[object.controller.other().index()]
                    })
                    .flatten();
                (chosen_creature.is_some()
                    || observes_greatest_power
                    || triggers.iter().any(|ability| {
                        matches!(
                            ability.condition.event(),
                            TriggerCondition::PermanentLeaves(_)
                                | TriggerCondition::BattlefieldDeparture(_)
                                | TriggerCondition::BattlefieldDepartureBatch(_)
                                | TriggerCondition::SelfOrControlledPermanentDies(_)
                        )
                    }))
                .then(|| DepartureObserver {
                    source: self.object_view(id),
                    chosen_creature,
                    greatest_opposing_power,
                })
            })
            .collect();
        if sources.is_empty() {
            return None;
        }
        let reads_mana_value = sources.iter().any(|observer| {
            observer.source.card.triggers().iter().any(|trigger| {
                matches!(
                    trigger.condition.event(),
                    TriggerCondition::BattlefieldDeparture(_)
                        | TriggerCondition::BattlefieldDepartureBatch(_)
                ) && trigger.targets.reads_event_mana_value()
            })
        });
        let mana_values = if reads_mana_value {
            departing
                .into_iter()
                .filter(|&object| {
                    self.on_battlefield(object.card) && self.object_ref(object.card) == object
                })
                .map(|object| (object, self.object_mana_value(object.card)))
                .collect()
        } else {
            Vec::new()
        };
        Some(Arc::new(DepartureObservers {
            sources,
            mana_values,
        }))
    }

    pub(super) fn battlefield_departure_triggers(&mut self, event: &ZoneChange) {
        let Some(sources) = &event.battlefield_departure_observers else {
            return;
        };
        let before = &event.before;
        for observer in &sources.sources {
            let source = &observer.source;
            for (index, ability) in source.card.triggers().iter().enumerate() {
                let matched = match ability.condition.event() {
                    TriggerCondition::PermanentLeaves(filter) => {
                        DepartureRule::from_legacy(filter).matches(source, event)
                    }
                    TriggerCondition::BattlefieldDeparture(rule) => rule.matches(source, event),
                    TriggerCondition::OpponentHighestPowerCreatureDies => {
                        event.destination == Zone::Graveyard
                            && before.controller != source.controller
                            && before.is_creature()
                            && Some(before.power) == observer.greatest_opposing_power
                    }
                    TriggerCondition::SelfOrControlledPermanentDies(kinds) => {
                        event.destination == Zone::Graveyard
                            && (source.reference == before.reference
                                || (source.controller == before.controller
                                    && before.kinds & kinds != 0))
                    }
                    TriggerCondition::ChosenCreatureLeaves => {
                        observer.chosen_creature == Some(before.reference)
                    }
                    _ => false,
                };
                if matched && self.queue_zone_trigger(source, index, event) {
                    self.capture_battlefield_departure_facts(event, ability.program);
                }
            }
        }
    }

    pub(super) fn battlefield_departure_batch_triggers(&mut self, events: &[ZoneChange]) {
        let mut dispatched: Vec<&Arc<DepartureObservers>> = Vec::new();
        for observers in events
            .iter()
            .filter_map(|event| event.battlefield_departure_observers.as_ref())
        {
            // A combined report may contain moves with different pre-event worlds.
            if dispatched.iter().any(|group| Arc::ptr_eq(group, observers)) {
                continue;
            }
            dispatched.push(observers);

            for observer in &observers.sources {
                let source = &observer.source;
                for (index, ability) in source.card.triggers().iter().enumerate() {
                    let TriggerCondition::BattlefieldDepartureBatch(rule) =
                        ability.condition.event()
                    else {
                        continue;
                    };
                    let mut matching = events.iter().filter(|event| {
                        event.moved
                            && event.before.zone == Zone::Battlefield
                            && event
                                .battlefield_departure_observers
                                .as_ref()
                                .is_some_and(|group| Arc::ptr_eq(group, observers))
                            && rule.matches(source, event)
                    });
                    let Some(first) = matching.next() else {
                        continue;
                    };
                    if !self.queue_zone_trigger(source, index, first) {
                        continue;
                    }
                    let mut includes_death = first.is_creature_death();
                    let packet = self.state.pending_triggers.last_mut().unwrap();
                    for event in matching {
                        includes_death |= event.is_creature_death();
                        packet.refer_zone_event(event);
                    }
                    if includes_death && !first.is_creature_death() {
                        self.add_event_trigger_multiplicity(
                            source.additional_creature_death_triggers,
                        );
                    }
                }
            }
        }
    }

    pub(super) fn capture_battlefield_departure_facts(
        &mut self,
        event: &ZoneChange,
        program: &[Instruction],
    ) {
        let facts = if program.iter().any(|instruction| {
            matches!(
                instruction,
                Instruction::ExileDeathCounterTotalUntilNextEnd
                    | Instruction::PutEventCountersOnTargets
                    | Instruction::CreateTokensScaled {
                        amount: AmountValue::DepartureCounterCount(_),
                        ..
                    }
                    | Instruction::CreateTokensWithSourceDeparture {
                        amount: AmountValue::DepartureCounterCount(_),
                        ..
                    }
            )
        }) {
            EventObjectFacts::DeathCounters(event.before.counters.clone())
        } else if event.sacrificed_by.is_some()
            && program.iter().any(|instruction| {
                matches!(
                    instruction,
                    Instruction::ExileTopLibraryRemember {
                        amount: AmountValue::SacrificeEventManaValue,
                        ..
                    }
                )
            })
        {
            EventObjectFacts::DepartureManaValue(event.granted_departure_mana_value)
        } else if program.iter().any(|instruction| {
            matches!(
                instruction,
                Instruction::GainLifeScaled(
                    _,
                    AmountValue::DeathEventToughness | AmountValue::DeathEventPower
                ) | Instruction::DrawScaled(_, AmountValue::DeathEventPower)
                    | Instruction::DamageScaled(AmountValue::DeathEventPower)
                    | Instruction::DamageRememberPlayersScaled(AmountValue::DeathEventPower)
                    | Instruction::PutCountersScaled {
                        amount: AmountValue::DeathEventPower,
                        ..
                    }
                    | Instruction::ExileTopChoosePlayScaled {
                        amount: AmountValue::DeathEventPower,
                        ..
                    }
            )
        }) {
            EventObjectFacts::CreatureComparison(CreatureComparisonFacts::from_view(&event.before))
        } else {
            return;
        };
        self.state
            .pending_triggers
            .last_mut()
            .unwrap()
            .set_event_object(EventObjectView {
                object: event.before.reference,
                facts,
                semantic_objects: None,
                conniver_creature: None,
                subtype_lki: None,
            });
    }
}
