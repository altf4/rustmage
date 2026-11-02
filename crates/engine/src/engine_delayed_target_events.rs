//! Delayed clauses keep their original controller and source independently of
//! the watched creature's control, abilities, or exact source's later departure.
use super::*;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, Serialize)]
pub struct DelayedTargetTriggerView {
    pub subject: ObjectRef,
    pub event: DelayedTargetEvent,
    pub expires_turn: u32,
    pub packet: SpellView,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn delay_target_event(
        &mut self,
        item: &SpellView,
        event: DelayedTargetEvent,
        ability: u16,
    ) {
        let definition = item.trigger_definition();
        let trigger = &definition.triggers()[usize::from(ability)];
        assert_eq!(trigger.condition, TriggerCondition::DelayedTargetEvent);
        assert!(
            !trigger.targets.has_targets(),
            "delayed target template must be untargeted"
        );
        let mut subjects = Vec::new();
        for &bound in &item.targets {
            if let Target::Card(id) = bound.target
                && self.same_target_object(bound)
                && self.on_battlefield(id)
            {
                let subject = self.object_ref(id);
                if !subjects.contains(&subject) {
                    subjects.push(subject);
                }
            }
        }
        for subject in subjects {
            let mut packet = item.clone();
            packet.id = self.next_stack_id();
            packet.kind = StackKind::TriggeredAbility { ability };
            packet.targets.clear();
            packet.crime_qualifying_target = false;
            packet.target_type_slots = None;
            packet.mode = None;
            packet.cast = None;
            packet.related_stack = None;
            packet.is_copy = false;
            packet.additional_triggers = 0;
            self.state
                .delayed_target_events
                .get_or_insert_default()
                .push(DelayedTargetTriggerView {
                    subject,
                    event,
                    expires_turn: self.state.turn,
                    packet,
                });
        }
    }
    pub(super) fn delayed_target_departure(&mut self, event: &ZoneChange) {
        if !event.moved || event.before.zone != Zone::Battlefield {
            return;
        }
        let Some(rows) = self.state.delayed_target_events.take() else {
            return;
        };
        let dies = event.destination == Zone::Graveyard && event.before.is_creature();
        let mut retained = Vec::new();
        for row in *rows {
            if row.expires_turn != self.state.turn {
                continue;
            }
            if row.subject != event.before.reference {
                retained.push(row);
                continue;
            }
            if row.event == DelayedTargetEvent::DiesOnce && dies {
                let mut packet = row.packet;
                packet.id = self.next_stack_id();
                self.state.pending_triggers.push(packet);
            }
            // Every departure ends tracking this exact incarnation. Neither a
            // returned card nor a token's brief graveyard interval can retrigger it.
        }
        if !retained.is_empty() {
            self.state.delayed_target_events = Some(Box::new(retained));
        }
    }
    pub(super) fn delayed_target_combat_damage(&mut self, events: &[DamageEvent]) {
        let Some(rows) = self.state.delayed_target_events.as_ref() else {
            return;
        };
        let hits = events
            .iter()
            .filter_map(|event| {
                if event.amount > 0
                    && event.combat
                    && let Target::Player(player) = event.target
                {
                    Some((event.source.reference, player))
                } else {
                    None
                }
            })
            .collect::<std::collections::BTreeSet<_>>();
        let packets = rows
            .iter()
            .filter(|row| {
                row.event == DelayedTargetEvent::CombatDamageToPlayer
                    && row.expires_turn == self.state.turn
            })
            .flat_map(|row| {
                hits.iter()
                    .filter(move |(source, _)| *source == row.subject)
                    .map(move |_| row.packet.clone())
            })
            .collect::<Vec<_>>();
        for mut packet in packets {
            packet.id = self.next_stack_id();
            self.state.pending_triggers.push(packet);
        }
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn owned_bytes(rows: &Option<Box<Vec<DelayedTargetTriggerView>>>) -> usize {
    rows.as_ref().map_or(0, |rows| {
        std::mem::size_of::<Vec<DelayedTargetTriggerView>>()
            + rows
                .iter()
                .map(|row| {
                    std::mem::size_of::<DelayedTargetTriggerView>() + row.packet.owned_bytes()
                })
                .sum::<usize>()
    })
}
