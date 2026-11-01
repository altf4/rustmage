//! Equip cost modifiers and committed player history, separate from attachment effects.
use super::*;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    /// Cost readiness may use a declared replacement, while timing, guards and
    /// usage limits still belong to the actual acquired ability instance.
    pub(super) fn activation_readiness(
        &self,
        actor: Player,
        ability: &ActivatedAbility,
    ) -> ActivatedAbility {
        let mut readiness = *ability;
        if self.first_equip_zero_available(actor, ability) {
            readiness.cost = ActivationCost::ZERO;
        }
        readiness
    }
    pub(super) fn visit_activation_counter_payments(
        &self,
        actor: Player,
        ability: &ActivatedAbility,
        available: CounterAmounts<'_>,
        mut emit: impl FnMut(Counters) -> bool,
    ) -> bool {
        // A waived printed fixed-counter cost must not prevent declaration.
        (ability.cost.remove.is_some()
            && self.first_equip_zero_available(actor, ability)
            && emit(Counters::default()))
            || activations::counter_payments(ability.cost, available, emit)
    }
    pub(super) fn activation_target_sensitive(definition: Card, ability: u16) -> bool {
        definition.activation(ability).is_equip()
            || definition.activation_target_reduction(ability).is_some()
            || definition
                .activation_target_counter_discount(ability)
                .is_some()
    }
    pub(super) fn first_equip_zero_available(
        &self,
        actor: Player,
        ability: &ActivatedAbility,
    ) -> bool {
        ability.is_equip()
            && !self.state.players[actor.index()].equip_activated_this_turn
            && self.state.static_sources.iter().any(|&id| {
                let object = &self.state.objects[id as usize];
                object.zone == Zone::Battlefield
                    && object.controller == actor
                    && object.card.static_ability_groups().iter().any(|group| {
                        group.fragments.iter().any(|fragment| {
                            if let crate::cards::StaticAbilityFragment::FirstEquipZeroCost {
                                condition,
                            } = fragment
                            {
                                self.continuous_fragment_available(id, |f| matches!(f, crate::cards::StaticAbilityFragment::FirstEquipZeroCost { condition: c } if c == condition))
                                    && self.stat_value(*condition, self.object_ref(id), actor) != 0
                            } else {
                                false
                            }
                        })
                    })
            })
    }
    /// Keep the ordinary payment-major action order when no new target-side
    /// mechanic can apply. The existing static index is sparse and incarnation-safe.
    pub(super) fn active_target_equip_reducer(
        &self,
        actor: Player,
        definition: Card,
        ability: u16,
    ) -> bool {
        definition.activation(ability).is_equip()
            && self.state.static_sources.iter().any(|&id| {
                self.equip_target_reduction(actor, definition, ability, Target::Card(id)) != 0
            })
    }
    pub(super) fn item_first_equip_zero_available(&self, item: &SpellView) -> bool {
        if let StackKind::ActivatedAbility { ability } = item.kind {
            !item.is_copy
                && self.first_equip_zero_available(
                    item.controller,
                    item.source
                        .expect("activated ability has source")
                        .card
                        .activation(ability),
                )
        } else {
            false
        }
    }
    pub(super) fn equip_target_reduction(
        &self,
        actor: Player,
        definition: Card,
        ability: u16,
        target: Target,
    ) -> u32 {
        if !definition.activation(ability).is_equip() {
            return 0;
        }
        let Target::Card(id) = target else {
            return 0;
        };
        let Some(object) = self.state.objects.get(id as usize) else {
            return 0;
        };
        if object.zone != Zone::Battlefield || object.controller != actor {
            return 0;
        }
        object
            .card
            .static_ability_groups()
            .iter()
            .flat_map(|g| g.fragments)
            .filter_map(|fragment| {
                if let crate::cards::StaticAbilityFragment::EquipTargetReduction(generic) = fragment
                {
                    self.continuous_fragment_available(id, |f| matches!(f, crate::cards::StaticAbilityFragment::EquipTargetReduction(g) if g == generic))
                        .then_some(*generic)
                } else {
                    None
                }
            })
            .fold(0u32, u32::saturating_add)
    }
    pub(super) fn announcement_cost_choice_fundable(&self, a: &Announcement, index: usize) -> bool {
        let Some(cost) = a.cost_options().nth(index) else {
            return false;
        };
        if matches!(a.item.kind, StackKind::ActivatedAbility { .. }) {
            let mut proposal = a.clone();
            proposal.cost = Some(cost);
            proposal.activation_cost_route = a.activation_route_for_index(index);
            if a.equip_zero_available
                && proposal.activation_cost_route == ActivationCostRoute::Printed
                && let StackKind::ActivatedAbility { ability } = proposal.item.kind
                && proposal
                    .item
                    .source
                    .expect("object-backed announcement")
                    .card
                    .activation(ability)
                    .cost
                    .remove
                    .is_some()
                && !activations::counter_payments(
                    proposal
                        .item
                        .source
                        .expect("cost announcement has object source")
                        .card
                        .activation(ability)
                        .cost,
                    self.counter_amounts(
                        proposal
                            .item
                            .source
                            .expect("cost announcement has object source")
                            .id,
                    ),
                    |payment| payment == proposal.counter_payment,
                )
            {
                return false;
            }
            self.announcement_cost_fundable(&proposal, cost)
        } else {
            self.announcement_cost_fundable(a, cost)
        }
    }
}
