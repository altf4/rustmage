//! Filter actual battlefield departures against frozen pre-event characteristics.
use super::*;
use crate::cards::{DepartureCounters, DepartureFilter};
impl Game {
    pub(super) fn departure_matches(
        &self,
        filter: DepartureFilter,
        source: &ObjectView,
        event: &ZoneChange,
    ) -> bool {
        event.before.zone == Zone::Battlefield
            && (!filter.graveyard_only || event.destination == Zone::Graveyard)
            && (filter.kinds == 0 || event.before.kinds & filter.kinds != 0)
            && (!filter.other || source.reference != event.before.reference)
            && match filter.relation {
                CastRelation::You => source.controller == event.before.controller,
                CastRelation::Opponent => source.controller != event.before.controller,
                CastRelation::Any => true,
            }
            && match filter.counters {
                DepartureCounters::Unrestricted => true,
                DepartureCounters::Any => event.before.counters.has_any(),
                DepartureCounters::Kind(kind) => event.before.counters.get(kind) > 0,
            }
    }
    pub(super) fn program_uses_departure_counters(program: &[Instruction]) -> bool {
        program.iter().any(|i| {
            matches!(
                i,
                Instruction::PutDepartureCounters { .. }
                    | Instruction::CreateTokensScaled {
                        amount: AmountValue::DepartureCounterCount(_),
                        ..
                    }
            )
        })
    }
    pub(super) fn capture_departure_counters(&mut self, event: &ZoneChange) {
        self.state.pending_triggers.last_mut().unwrap().death_event =
            Some(Box::new(DeathEventView {
                object: event.before.reference,
                counter_total: event.before.counters.total(),
                power: event.before.power,
                counter_snapshot: Some(Box::new(event.before.counters.clone())),
            }));
    }
}
