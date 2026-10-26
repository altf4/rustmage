//! Actual activated instances bestowed by a resolution, distinct from printed slots.
use super::ability_world::CharacteristicWorld;
use super::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct SelfGrantedActivationContext {
    pub grant_timestamp: u64,
}
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
        if CharacteristicWorld::activation_prohibited(self, actor, card)
            || self.named_activation_prohibited(card)
        {
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
            let readiness = self.activation_readiness(actor, rules);
            if !query.self_grant_available(row.recipient, row.timestamp)
                || !self.activation_guard_available(rules.guard, card, actor)
                || !Self::object_activation_available_with_kinds(
                    &self.state.objects[card as usize],
                    self.counter_amounts(card),
                    self.current_kinds(card),
                    || CharacteristicWorld::activation_keywords(self, card),
                    self.state.players[actor.index()].life,
                    &readiness,
                    usize::from(ability),
                    actor,
                    sorcery,
                )
            {
                continue;
            }
            if self.targets_available(rules.targets, actor, row.recipient)
                && self.initial_activation_target_fundable(
                    FundingGoal {
                        reserve: None,
                        actor,
                        required: rules.cost.colored,
                        generic: rules.cost.generic,
                        source: Some((
                            row.recipient,
                            ability,
                            Counters::default(),
                            row.granting_card,
                            None,
                        )),
                    },
                    rules.targets,
                )
                && emit(Action::AnnounceSelfGrantedAbility {
                    source: row.recipient,
                    grant_timestamp: row.timestamp,
                })
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
    ) {
        let row = *self
            .state
            .self_granted_abilities
            .as_ref()
            .unwrap()
            .iter()
            .find(|row| row.recipient == source && row.timestamp == grant_timestamp)
            .unwrap();
        let SelfGrantedAbilityKind::Activated { ability } = row.kind else {
            unreachable!()
        };
        let actor = self.state.objects[source.card as usize].controller;
        let mut item = self.activation_item(actor, source.card, ability);
        item.source_card = row.granting_card;
        item.source_ability_origin = Some(row.timestamp);
        item.self_granted_activation = Some(Box::new(SelfGrantedActivationContext {
            grant_timestamp: row.timestamp,
        }));
        item.source_transform = None;
        item.source_counter_lki = None;
        item.source_quantity_lki = None;
        self.open_payment_announcement(item, Counters::default(), None);
    }
}
