//! Sparse exact-incarnation grants reuse static programs and owned trigger packets.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct TemporaryAbilityView {
    pub recipient: ObjectRef,
    /// Historical spell/ability source; departure never revokes the grant.
    pub grantor: ObjectRef,
    pub granting_card: Card,
    pub ability: u16,
    pub grantor_controller: Player,
    pub expires_turn: u32,
    pub permanent: bool,
    pub timestamp: u64,
}
impl Game {
    pub(super) fn grant_temporary_ability(&mut self, item: &SpellView, ability: u16) {
        let ids = item
            .targets
            .iter()
            .filter(|&&bound| self.same_target_object(bound))
            .filter_map(|bound| {
                if let Target::Card(id) = bound.target {
                    self.on_battlefield(id).then_some(id)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        self.grant_temporary_ability_to(item, ability, &ids);
    }
    pub(super) fn grant_temporary_ability_to(
        &mut self,
        item: &SpellView,
        ability: u16,
        ids: &[CardId],
    ) {
        let granting_card = item.trigger_definition();
        assert!(
            granting_card.triggers()[usize::from(ability)]
                .condition
                .is_granted()
        );
        let timestamp = self.next_role_timestamp();
        for &id in ids {
            let grant = TemporaryAbilityView {
                recipient: self.object_ref(id),
                grantor: item.object,
                granting_card,
                ability,
                grantor_controller: item.controller,
                expires_turn: self.state.turn,
                permanent: false,
                timestamp,
            };
            self.state
                .temporary_abilities
                .get_or_insert_with(|| Box::new(Vec::new()))
                .push(grant);
        }
    }
    pub(super) fn temporary_attack_triggers(&mut self, actor: Player) {
        let grants = self
            .state
            .temporary_abilities
            .as_deref()
            .into_iter()
            .flatten()
            .copied()
            .filter(|grant| {
                (grant.permanent || grant.expires_turn == self.state.turn)
                    && self.on_battlefield(grant.recipient.card)
                    && self.object_ref(grant.recipient.card) == grant.recipient
                    && self.state.objects[grant.recipient.card as usize].controller == actor
                    && self
                        .state
                        .attackers
                        .iter()
                        .any(|a| a.card == grant.recipient.card)
                    && grant.granting_card.triggers()[usize::from(grant.ability)]
                        .condition
                        .event()
                        == TriggerCondition::SelfAttacks
            })
            .collect::<Vec<_>>();
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
            self.queue_trigger_from(
                &source,
                usize::from(grant.ability),
                Some(Box::new(AttachmentTriggerView {
                    grantor: grant.grantor,
                    granting_card: grant.granting_card,
                    recipient: grant.recipient,
                    defender: Some(actor.other()),
                    granted: true,
                })),
            );
        }
    }
    pub(super) fn temporary_departure_triggers(&mut self, event: &ZoneChange) {
        if event.before.zone != Zone::Battlefield || !event.moved {
            return;
        }
        if event.destination == Zone::Graveyard {
            let matching: Vec<_> = self
                .state
                .temporary_abilities
                .as_ref()
                .map_or(&[][..], |grants| grants.as_slice())
                .iter()
                .copied()
                .filter(|grant| {
                    grant.recipient == event.before.reference
                        && (grant.permanent || grant.expires_turn == self.state.turn)
                        && event.before.grant_available(
                            grant.grantor,
                            grant.granting_card,
                            grant.ability,
                            Some(grant.timestamp),
                        )
                })
                .collect();
            for grant in matching {
                let ability = grant.granting_card.triggers()[usize::from(grant.ability)];
                if ability.condition.event() != TriggerCondition::SelfDies {
                    continue;
                }
                // 'You' belongs to the dying creature's actual controller, not
                // the caster that originally bestowed the ability.
                let context = AttachmentTriggerView {
                    grantor: grant.grantor,
                    granting_card: grant.granting_card,
                    recipient: grant.recipient,
                    defender: None,
                    granted: true,
                };
                self.queue_zone_trigger_from(
                    &event.before,
                    usize::from(grant.ability),
                    event,
                    Some(Box::new(context)),
                );
            }
        }
        // Dispatch before erasing the exact old incarnation, including each
        // member of a simultaneous departure batch. Other recipients survive.
        if let Some(grants) = &mut self.state.temporary_abilities {
            grants.retain(|grant| grant.recipient != event.before.reference);
        }
        if self
            .state
            .temporary_abilities
            .as_ref()
            .is_some_and(|grants| grants.is_empty())
        {
            self.state.temporary_abilities = None;
        }
    }
    pub(super) fn return_granted_source(
        &mut self,
        item: &SpellView,
        counter: Option<(CounterKind, u64)>,
        role: Option<Card>,
    ) {
        if let Some(movement) = self.granted_source_entry(item, counter, role) {
            self.stage_entries(vec![movement], None);
        }
    }
    pub(super) fn granted_source_entry(
        &self,
        item: &SpellView,
        counter: Option<(CounterKind, u64)>,
        role: Option<Card>,
    ) -> Option<entries::EntryMove> {
        let expected = item.target_source();
        let object = self.state.objects.get(expected.card as usize)?;
        if object.zone != Zone::Graveyard || self.object_ref(expected.card) != expected {
            return None;
        }
        let owner = object.owner;
        let mut movement = self.entry_move(expected.card, Zone::Battlefield, true);
        movement.controller = owner;
        movement.effect_counter = counter;
        movement.role_after = role.map(|token| (token, item.controller));
        Some(movement)
    }
    pub(super) fn temporary_ability_bytes(&self) -> usize {
        self.state.temporary_abilities.as_ref().map_or(0, |grants| {
            std::mem::size_of::<Vec<TemporaryAbilityView>>()
                + grants.len() * std::mem::size_of::<TemporaryAbilityView>()
        })
    }
}
