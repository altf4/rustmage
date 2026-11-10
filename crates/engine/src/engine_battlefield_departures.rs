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
        let before = &event.before;
        let related = match self.relation {
            CastRelation::You => source.controller == before.controller,
            CastRelation::Opponent => source.controller != before.controller,
            CastRelation::Any => true,
        };
        let subject_matches = match self.subject {
            DepartureSubject::Any => true,
            DepartureSubject::Other => source.reference != before.reference,
            DepartureSubject::SelfOnly => source.reference == before.reference,
        };
        let counters_match = match self.counters {
            DepartureCounters::Unrestricted => true,
            DepartureCounters::Any => before.counters.has_any(),
            DepartureCounters::Kind(kind) => before.counters.get(kind) != 0,
        };
        related
            && subject_matches
            && counters_match
            && (self.destinations.is_empty() || self.destinations.contains(&event.destination))
            && (!self.nontoken || !before.is_token())
            && (self.kinds == 0 || before.kinds & self.kinds != 0)
            && self
                .minimum_power
                .is_none_or(|power| before.is_creature() && before.power >= i128::from(power))
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
    ) -> Option<Arc<[DepartureObserver]>> {
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
                (chosen_creature.is_some()
                    || triggers.iter().any(|ability| {
                        matches!(
                            ability.condition.event(),
                            TriggerCondition::PermanentLeaves(_)
                                | TriggerCondition::BattlefieldDeparture(_)
                                | TriggerCondition::SelfOrControlledPermanentDies(_)
                        )
                    }))
                .then(|| DepartureObserver {
                    source: self.object_view(id),
                    chosen_creature,
                })
            })
            .collect();
        (!sources.is_empty()).then(|| sources.into())
    }

    pub(super) fn battlefield_departure_triggers(&mut self, event: &ZoneChange) {
        let Some(sources) = &event.battlefield_departure_observers else {
            return;
        };
        let before = &event.before;
        for observer in sources.iter() {
            let source = &observer.source;
            for (index, ability) in source.card.triggers().iter().enumerate() {
                let matched = match ability.condition.event() {
                    TriggerCondition::PermanentLeaves(filter) => {
                        DepartureRule::from_legacy(filter).matches(source, event)
                    }
                    TriggerCondition::BattlefieldDeparture(rule) => rule.matches(source, event),
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
            )
        }) {
            EventObjectFacts::DeathCounters(event.before.counters.clone())
        } else if program.iter().any(|instruction| {
            matches!(
                instruction,
                Instruction::GainLifeScaled(
                    _,
                    AmountValue::DeathEventToughness | AmountValue::DeathEventPower
                ) | Instruction::DrawScaled(_, AmountValue::DeathEventPower)
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
