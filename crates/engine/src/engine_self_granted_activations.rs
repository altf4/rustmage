//! Actual activated instances bestowed by a resolution, distinct from printed slots.
use super::ability_world::CharacteristicWorld;
use super::*;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct SelfGrantedActivationContext {
    pub grant_timestamp: u64,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn visit_self_granted_activations(
        &self,
        card: CardId,
        actor: Player,
        sorcery: bool,
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        let Some(rows) = &self.state.self_granted_abilities else {
            return false;
        };
        if !rows.iter().any(|row| row.recipient.card == card) {
            return false;
        }
        if CharacteristicWorld::activation_prohibited(self, actor, card) {
            return false;
        }
        let query = self.characteristic_query();
        for row in rows.iter().filter(|row| {
            row.recipient.card == card
                && row.expires_turn.is_none_or(|turn| turn == self.state.turn)
        }) {
            let SelfGrantedAbilityKind::Activated { ability } = row.kind else {
                continue;
            };
            let rules = row.granting_card.activation(ability);
            if rules.mana_production().is_none()
                && (self.nonmana_actions_prohibited() || self.named_activation_prohibited(card))
            {
                continue;
            }
            if self.self_grant_activation_used(
                row.recipient,
                row.timestamp,
                row.granting_card,
                ability,
                actor,
            ) {
                continue;
            }
            let mut readiness = self.activation_readiness(actor, rules);
            readiness.once_per_turn = false;
            readiness.exhaust = false;
            let mut ready_object = self.state.objects[card as usize].clone();
            ready_object.used_abilities = 0;
            if activations::pure_loyalty_placement_prohibited(self, row.recipient, &readiness)
                || (rules.cost.sacrifice && CharacteristicWorld::sacrifice_prohibited(self, card))
                || !query.self_grant_available(row.recipient, row.timestamp)
                || !self.activation_guard_available(rules.guard, card, actor)
                || !Self::object_activation_available_with_kinds(
                    &ready_object,
                    self.counter_amounts(card),
                    self.current_kinds(card),
                    || CharacteristicWorld::activation_keywords(self, card),
                    self.state.players[actor.index()].life,
                    &readiness,
                    usize::from(ability),
                    actor,
                    self.loyalty_timing(card, rules, actor, sorcery),
                )
            {
                continue;
            }
            if let Some(production) = rules.mana_production() {
                let packet = self.self_granted_activation_packet(*row);
                if self.visit_activation_counter_payments(
                    actor,
                    rules,
                    self.counter_amounts(card),
                    |counters| {
                        self.visit_activation_mana_outputs(
                            production,
                            card,
                            row.granting_card,
                            ability,
                            |production| {
                                self.mana_packet_fundable(&packet, counters.clone(), production)
                                    && emit(Action::AnnounceSelfGrantedMana {
                                        source: row.recipient,
                                        grant_timestamp: row.timestamp,
                                        counter_payment: counters.clone(),
                                        production,
                                    })
                            },
                        )
                    },
                ) {
                    return true;
                }
                continue;
            }
            let packet = self.self_granted_activation_packet(*row);
            if self.targets_available(rules.targets, actor, row.recipient)
                && self.visit_activation_counter_payments(
                    actor,
                    rules,
                    self.counter_amounts(card),
                    |counter_payment| {
                        self.initial_activation_packet_target_fundable(&packet, &counter_payment)
                            && emit(Action::AnnounceSelfGrantedAbility {
                                source: row.recipient,
                                grant_timestamp: row.timestamp,
                                counter_payment,
                            })
                    },
                )
            {
                return true;
            }
        }
        false
    }
    pub(super) fn announce_self_granted_ability(
        &mut self,
        source: ObjectRef,
        grant_timestamp: u64,
        counter_payment: Counters,
    ) {
        let row = *self
            .state
            .self_granted_abilities
            .as_ref()
            .unwrap()
            .iter()
            .find(|row| row.recipient == source && row.timestamp == grant_timestamp)
            .unwrap();
        let mut item = self.self_granted_activation_packet(row);
        item.id = self.next_stack_id();
        self.state.passes = 0;
        self.open_payment_announcement(item, counter_payment, None);
    }
    pub(super) fn self_granted_activation_packet(&self, row: self_granted::Row) -> SpellView {
        let source = row.recipient;
        let SelfGrantedAbilityKind::Activated { ability } = row.kind else {
            unreachable!()
        };
        let actor = self.state.objects[source.card as usize].controller;
        let mut item = self.activation_packet(actor, source.card, ability);
        item.set_source_definition(row.granting_card);
        item.source_ability_origin = Some(row.timestamp);
        item.self_granted_activation = Some(Box::new(SelfGrantedActivationContext {
            grant_timestamp: row.timestamp,
        }));
        item.source_transform = None;
        item.source_counter_lki = None;
        item.source_quantity_lki = None;
        item.attachment_trigger = None;
        item
    }
}
