//! Actual committed activations, distinct from resolution, copies and funding.
use super::*;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn register_activation_observer(&mut self, id: CardId) {
        if self.state.objects[id as usize]
            .card
            .triggers()
            .iter()
            .any(|ability| {
                matches!(
                    ability.condition.event(),
                    TriggerCondition::CreatureAbilityActivated
                        | TriggerCondition::NonmanaAbilityActivated
                        | TriggerCondition::LoyaltyAbilityActivated
                        | TriggerCondition::LoyaltyAbilityActivatedRemoving(_)
                        | TriggerCondition::OpponentLoyaltyAbilityActivated
                )
            })
        {
            let sources = self.state.activation_observers.get_or_insert_default();
            if !sources.contains(&id) {
                sources.push(id);
            }
        }
    }
    pub(super) fn forget_activation_observer(&mut self, id: CardId) {
        if let Some(sources) = &mut self.state.activation_observers {
            sources.retain(|&source| source != id);
            if sources.is_empty() {
                self.state.activation_observers = None;
            }
        }
    }
    pub(super) fn creature_ability_activated(&mut self, actor: Player, creature: bool) {
        if !creature
            || self
                .state
                .cast_rollback
                .as_ref()
                .is_some_and(|r| r.replay.is_some())
        {
            return;
        }
        let Some(ids) = &self.state.activation_observers else {
            return;
        };
        let observers: Vec<_> = ids
            .iter()
            .filter(|&&id| {
                self.on_battlefield(id) && self.state.objects[id as usize].controller == actor
            })
            .map(|&id| self.object_view(id))
            .collect();
        for source in observers {
            for (index, ability) in source.card.triggers().iter().enumerate() {
                if ability.condition.event() == TriggerCondition::CreatureAbilityActivated {
                    self.queue_trigger(&source, index);
                }
            }
        }
    }
    pub(super) fn committed_activation_observers(&mut self, item: &SpellView) {
        self.commit_activation_use(item);
        let Some(identity) = item.source else {
            return;
        };
        if let StackKind::ActivatedAbility { ability } = item.kind
            && !item.is_copy
            && identity.card.activation(ability).cost.loyalty.is_some()
        {
            let source_zone = identity.card.activation(ability).source_zone();
            if let Some(object) = self.state.objects.get_mut(identity.object.card as usize)
                && object.generation == identity.object.generation
                && object.zone == source_zone
            {
                object.loyalty_activated_this_turn = true;
            }
        }
        if let StackKind::ActivatedAbility { ability } = item.kind
            && !item.is_copy
            && identity.card.activation(ability).cost.loyalty.is_some()
            && !self
                .state
                .cast_rollback
                .as_ref()
                .is_some_and(|r| r.replay.is_some())
        {
            self.state.players[item.controller.index()].loyalty_ability_activated_this_turn = true;
        }
        if self.state.activation_observers.is_none() {
            return;
        }
        let StackKind::ActivatedAbility {
            ability: ability_index,
        } = item.kind
        else {
            return;
        };
        if item.is_copy {
            return;
        }
        if identity
            .card
            .activation(ability_index)
            .mana_production()
            .is_none()
            && !self
                .state
                .cast_rollback
                .as_ref()
                .is_some_and(|r| r.replay.is_some())
        {
            let observers: Vec<_> = self
                .state
                .activation_observers
                .as_ref()
                .unwrap()
                .iter()
                .filter(|&&id| {
                    self.on_battlefield(id)
                        && self.state.objects[id as usize].controller == item.controller
                })
                .map(|&id| self.object_view(id))
                .collect();
            for source in observers {
                for (index, ability) in source.card.triggers().iter().enumerate() {
                    if ability.condition.event() == TriggerCondition::NonmanaAbilityActivated
                        && self.queue_trigger(&source, index)
                    {
                        self.state.pending_triggers.last_mut().unwrap().copy_event =
                            Some(Box::new(CopyEvent {
                                spell: item.clone(),
                                amount: 1,
                            }));
                    }
                }
            }
        }
        // Borrowing changes the ability definition, not the actual activating object.
        // A creature CARD cycling from hand is not a creature permanent.
        let creature = if self.on_battlefield(identity.object.card)
            && self.object_ref(identity.object.card) == identity.object
        {
            self.is_creature(identity.object.card)
        } else {
            identity.card.activation(ability_index).source_zone() == Zone::Battlefield
                && identity.facts.is_creature()
        };
        self.creature_ability_activated(item.controller, creature);
        if identity
            .card
            .activation(ability_index)
            .cost
            .loyalty
            .is_some()
            && !self
                .state
                .cast_rollback
                .as_ref()
                .is_some_and(|r| r.replay.is_some())
        {
            let observers: Vec<_> = self
                .state
                .activation_observers
                .as_ref()
                .unwrap()
                .iter()
                .filter(|&&id| self.on_battlefield(id))
                .map(|&id| self.object_view(id))
                .collect();
            for source in observers {
                for (index, ability) in source.card.triggers().iter().enumerate() {
                    let triggered = match ability.condition.event() {
                        TriggerCondition::LoyaltyAbilityActivated => {
                            source.controller == item.controller
                        }
                        TriggerCondition::LoyaltyAbilityActivatedRemoving(minimum) => {
                            source.controller == item.controller
                                && item.loyalty_counters_paid >= minimum
                        }
                        TriggerCondition::OpponentLoyaltyAbilityActivated => {
                            source.controller != item.controller
                        }
                        _ => false,
                    };
                    if triggered
                        && self.queue_trigger(&source, index)
                        && ability.condition.event()
                            == TriggerCondition::OpponentLoyaltyAbilityActivated
                    {
                        self.state.pending_triggers.last_mut().unwrap().event_player =
                            Some(item.controller);
                    }
                }
            }
        }
    }
    pub(super) fn restore_retained_trigger_use(&mut self, item: &SpellView) {
        let StackKind::TriggeredAbility { ability } = item.kind else {
            return;
        };
        let Some(identity) = item.source else {
            return;
        };
        let condition = item.trigger_definition().triggers()[usize::from(ability)].condition;
        if condition.once_per_turn() {
            let used = TriggerUse {
                source: identity.object,
                ability,
                origin: item.source_ability_origin,
            };
            if !self.state.triggered_this_turn.contains(&used) {
                self.state.triggered_this_turn.push(used);
            }
        }
    }
}
