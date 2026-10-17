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
}
impl Game {
    pub(super) fn grant_temporary_ability(&mut self, item: &SpellView, ability: u16) {
        let granting_card = item.trigger_definition();
        assert!(
            granting_card.triggers()[usize::from(ability)]
                .condition
                .is_granted()
        );
        for bound in &item.targets {
            if !self.same_target_object(*bound) {
                continue;
            }
            let Target::Card(id) = bound.target else {
                continue;
            };
            if !self.on_battlefield(id) || !self.is_creature(id) {
                continue;
            }
            let grant = TemporaryAbilityView {
                recipient: self.object_ref(id),
                grantor: item.object,
                granting_card,
                ability,
                grantor_controller: item.controller,
                expires_turn: self.state.turn,
            };
            self.state
                .temporary_abilities
                .get_or_insert_with(|| Box::new(Vec::new()))
                .push(grant);
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
                        && grant.expires_turn == self.state.turn
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
        let expected = item.target_source();
        let Some(object) = self.state.objects.get(expected.card as usize) else {
            return;
        };
        if object.zone != Zone::Graveyard || self.object_ref(expected.card) != expected {
            return;
        }
        let owner = object.owner;
        let mut movement = self.entry_move(expected.card, Zone::Battlefield, true);
        movement.controller = owner;
        movement.effect_counter = counter;
        movement.role_after = role.map(|token| (token, item.controller));
        self.stage_entries(vec![movement], None);
    }
    pub(super) fn temporary_ability_bytes(&self) -> usize {
        self.state.temporary_abilities.as_ref().map_or(0, |grants| {
            std::mem::size_of::<Vec<TemporaryAbilityView>>()
                + grants.len() * std::mem::size_of::<TemporaryAbilityView>()
        })
    }
}
