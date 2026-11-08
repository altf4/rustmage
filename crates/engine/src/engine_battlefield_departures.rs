//! Battlefield leave observers use the exact pre-event source and recipient.
use super::*;
use crate::cards::DepartureCounters;
use std::sync::Arc;

impl Game {
    pub(super) fn freeze_battlefield_departure_observers(&self) -> Option<Arc<[ObjectView]>> {
        let sources: Vec<_> = self
            .state
            .death_sources
            .iter()
            .copied()
            .filter(|&id| {
                let object = &self.state.objects[id as usize];
                object.zone == Zone::Battlefield
                    && object.card.triggers().iter().any(|ability| {
                        matches!(
                            ability.condition.event(),
                            TriggerCondition::PermanentLeaves(_)
                        )
                    })
            })
            .map(|id| self.object_view(id))
            .collect();
        (!sources.is_empty()).then(|| sources.into())
    }

    pub(super) fn battlefield_departure_triggers(&mut self, event: &ZoneChange) {
        let Some(sources) = &event.battlefield_departure_observers else {
            return;
        };
        let before = &event.before;
        for source in sources.iter() {
            for (index, ability) in source.card.triggers().iter().enumerate() {
                let TriggerCondition::PermanentLeaves(filter) = ability.condition.event() else {
                    continue;
                };
                let related = match filter.relation {
                    CastRelation::You => source.controller == before.controller,
                    CastRelation::Opponent => source.controller != before.controller,
                    CastRelation::Any => true,
                };
                let counters_match = match filter.counters {
                    DepartureCounters::Unrestricted => true,
                    DepartureCounters::Any => before.counters.has_any(),
                    DepartureCounters::Kind(kind) => before.counters.get(kind) != 0,
                };
                if related
                    && counters_match
                    && (!filter.other || source.reference != before.reference)
                    && (!filter.graveyard_only || event.destination == Zone::Graveyard)
                    && (filter.kinds == 0 || before.kinds & filter.kinds != 0)
                    && self.queue_zone_trigger(source, index, event)
                {
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
                Instruction::PutEventCountersOnTargets
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
                Instruction::GainLifeScaled(_, AmountValue::DeathEventToughness)
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
