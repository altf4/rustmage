//! Exact public damage facts and state-trigger latching derive from owned packets.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct CreatureDamageHistoryView {
    pub source: ObjectRef,
    pub recipient: ObjectRef,
}
impl Game {
    pub(super) fn creature_damage_events(&mut self, events: &[DamageEvent]) {
        for event in events {
            if event.amount <= 0 {
                continue;
            }
            let Target::Card(id) = event.target else {
                continue;
            };
            if !self.on_battlefield(id) || !self.is_creature(id) {
                continue;
            }
            let row = CreatureDamageHistoryView {
                source: event.source.reference,
                recipient: self.object_ref(id),
            };
            let rows = self
                .state
                .creature_damage_history
                .get_or_insert_with(|| Vec::new().into());
            if !rows.contains(&row) {
                rows.push(row);
            }
        }
        // Each individual granted ability triggers once per simultaneous damage
        // batch, including damage to creatures; multiple grants stay independent.
        let grants: Vec<_> = self
            .state
            .temporary_abilities
            .as_ref()
            .map_or(&[][..], |rows| rows.as_slice())
            .iter()
            .copied()
            .filter(|grant| {
                grant.expires_turn == self.state.turn
                    && self.on_battlefield(grant.recipient.card)
                    && self.object_ref(grant.recipient.card) == grant.recipient
                    && grant.granting_card.triggers()[usize::from(grant.ability)]
                        .condition
                        .event()
                        == TriggerCondition::SourceDealtCombatDamage
                    && events.iter().any(|event| {
                        event.amount > 0
                            && event.combat
                            && event.source.reference == grant.recipient
                    })
            })
            .collect();
        for grant in grants {
            let source = self.object_view(grant.recipient.card);
            if !source.grant_available(
                grant.grantor,
                grant.granting_card,
                grant.ability,
                Some(grant.timestamp),
            ) {
                continue;
            }
            let context = AttachmentTriggerView {
                grantor: grant.grantor,
                granting_card: grant.granting_card,
                recipient: grant.recipient,
                defender: None,
                granted: true,
            };
            self.queue_trigger_from(&source, usize::from(grant.ability), Some(Box::new(context)));
        }
    }

    pub(super) fn queue_state_counter_triggers(&mut self) {
        // Use the existing counter observer index. A state trigger is latched
        // while its exact ability is pending, announcing, stacked or resolving.
        let sources: Vec<_> = self
            .state
            .counter_sources
            .iter()
            .map(|&id| self.object_view(id))
            .collect();
        for source in sources {
            for (index, ability) in source.card.triggers().iter().enumerate() {
                let TriggerCondition::StateCounterAtLeast { kind, minimum } =
                    ability.condition.event()
                else {
                    continue;
                };
                if self.counter_count(source.reference.card, kind) < minimum {
                    continue;
                }
                let matches = |item: &SpellView| {
                    item.object == source.reference
                        && item.source_card == source.card
                        && item.kind
                            == (StackKind::TriggeredAbility {
                                ability: index as u16,
                            })
                };
                let queued = self
                    .state
                    .pending_triggers
                    .iter()
                    .chain(&self.state.ordering_triggers)
                    .chain(&self.state.stack)
                    .any(matches)
                    || self
                        .state
                        .announcing
                        .as_ref()
                        .is_some_and(|a| matches(&a.item))
                    || self
                        .state
                        .resolving
                        .as_ref()
                        .is_some_and(|r| matches(&r.item));
                if !queued {
                    self.queue_trigger(&source, index);
                }
            }
        }
    }
}
