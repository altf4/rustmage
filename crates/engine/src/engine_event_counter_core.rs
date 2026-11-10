//! Exact public damage facts and state-trigger latching derive from owned packets.
use super::*;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct CreatureDamageHistoryView {
    pub source: ObjectRef,
    pub recipient: ObjectRef,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct PermanentDamageHistoryView {
    pub source: ObjectRef,
    pub recipient: ObjectRef,
    pub amount: u128,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct PermanentDamageRecord {
    pub(super) view: PermanentDamageHistoryView,
    /// The existing dealt-damage-to-a-creature predicate checks the type at the event.
    pub(super) was_creature: bool,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn damage_received_this_turn(&self, object: ObjectRef) -> u128 {
        self.state
            .permanent_damage_history
            .as_ref()
            .into_iter()
            .flat_map(|rows| rows.iter())
            .filter(|row| row.view.recipient == object)
            .fold(0, |total, row| {
                total
                    .checked_add(row.view.amount)
                    .capacity("permanent damage amount exhausted")
            })
    }

    pub(super) fn creature_damage_events(&mut self, events: &[DamageEvent]) {
        self.combat_turn_callback_events(events);
        self.controlled_combat_damage_triggers(events);
        let sources = self.state.damage_sources.clone();
        for id in sources {
            let source = self.object_view(id);
            let Some(recipient) = source.attached_to else {
                continue;
            };
            if !self.on_battlefield(recipient.card)
                || self.object_ref(recipient.card) != recipient
                || !self.is_creature(recipient.card)
            {
                continue;
            }
            let amount = events
                .iter()
                .filter(|e| e.amount > 0 && e.combat && e.source.reference == recipient)
                .fold(0u128, |total, e| {
                    total
                        .checked_add(e.amount as u128)
                        .capacity("equipped combat damage exhausted")
                });
            if amount == 0 {
                continue;
            }
            for (index, ability) in source.card.triggers().iter().enumerate() {
                if ability.condition.event() == TriggerCondition::AttachedDealtCombatDamage
                    && self.queue_trigger(&source, index)
                {
                    self.state
                        .pending_triggers
                        .last_mut()
                        .unwrap()
                        .refer_objects([recipient]);
                    if ability.program.iter().any(|instruction| {
                        matches!(
                            instruction,
                            Instruction::PutCountersScaled {
                                amount: AmountValue::CreatureCombatDamageAmount,
                                ..
                            }
                        )
                    }) {
                        self.state
                            .pending_triggers
                            .last_mut()
                            .unwrap()
                            .creature_combat_event = Some(Box::new(CreatureCombatEventView {
                            dealer: recipient,
                            amount,
                        }));
                    }
                }
            }
        }

        for event in events {
            if event.amount <= 0 {
                continue;
            }
            let Target::Card(id) = event.target else {
                continue;
            };
            if !self.on_battlefield(id) {
                continue;
            }
            let view = PermanentDamageHistoryView {
                source: event.source.reference,
                recipient: self.object_ref(id),
                amount: event.amount as u128,
            };
            let was_creature = self.is_creature(id);
            let rows = self
                .state
                .permanent_damage_history
                .get_or_insert_with(|| Vec::new().into());
            if let Some(index) = rows.iter().position(|row| {
                row.view.source == view.source && row.view.recipient == view.recipient
            }) {
                let row = &mut rows[index];
                row.view.amount = row
                    .view
                    .amount
                    .checked_add(view.amount)
                    .capacity("permanent damage amount exhausted");
                row.was_creature |= was_creature;
            } else {
                rows.push(PermanentDamageRecord { view, was_creature });
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
                (grant.permanent || grant.expires_turn == self.state.turn)
                    && self.on_battlefield(grant.recipient.card)
                    && self.object_ref(grant.recipient.card) == grant.recipient
                    && grant.granting_card.triggers()[usize::from(grant.ability)]
                        .condition
                        .event()
                        .is_growth_damage_event()
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
                origin: grant.origin,
            };
            let event = grant.granting_card.triggers()[usize::from(grant.ability)]
                .condition
                .event();
            if event == TriggerCondition::SourceCombatDamagePlayerOrPlaneswalker {
                let hits: std::collections::BTreeSet<_> = events
                    .iter()
                    .filter(|event| {
                        event.amount > 0
                            && event.combat
                            && event.source.reference == grant.recipient
                            && self.is_player_or_planeswalker_damage(event)
                    })
                    .map(|event| event.target)
                    .collect();
                for _ in hits {
                    self.queue_trigger_from(
                        &source,
                        usize::from(grant.ability),
                        Some(Box::new(context)),
                    );
                }
            } else if let TriggerCondition::PlayerDamaged { .. } = event {
                let mut hits = std::collections::BTreeMap::<Player, u128>::new();
                for e in events {
                    if e.amount > 0
                        && e.combat
                        && e.source.reference == grant.recipient
                        && let Target::Player(player) = e.target
                    {
                        let n = hits.entry(player).or_default();
                        *n = n
                            .checked_add(e.amount as u128)
                            .capacity("damage amount exhausted");
                    }
                }
                for (player, amount) in hits {
                    if self.queue_trigger_from(
                        &source,
                        usize::from(grant.ability),
                        Some(Box::new(context)),
                    ) {
                        self.state.pending_triggers.last_mut().unwrap().damage_event =
                            Some(DamageTriggerView {
                                dealer: Some(grant.recipient),
                                player,
                                amount,
                                combat: true,
                            });
                    }
                }
            } else {
                self.queue_trigger_from(
                    &source,
                    usize::from(grant.ability),
                    Some(Box::new(context)),
                );
            }
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
                    item.source.is_some_and(|identity| {
                        identity.object == source.reference && identity.card == source.card
                    }) && item.kind
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
