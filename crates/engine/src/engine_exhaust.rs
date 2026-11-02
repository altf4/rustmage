//! Exhaust is an actual activation event, an exact ability use and a player history.
use super::*;
use crate::cards::StaticAbilityFragment;
use ability_world::CharacteristicWorld;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn reuse_available<W: CharacteristicWorld + ?Sized>(world: &W, actor: Player) -> bool {
    world.game().state.active == actor
        && !world.exhaust_activated_this_turn(actor)
        && world.static_sources().iter().any(|&id| {
            let source = world.object(id);
            source.zone == Zone::Battlefield
                && source.controller == actor
                && source.card.static_ability_groups().iter().any(|group| {
                    group.fragments.iter().any(|f| {
                        matches!(
                            f,
                            StaticAbilityFragment::ReuseExhaustBeforeFirstActivationOwnTurn
                        )
                    })
                })
                && world.fragment_available(id, |f| {
                    matches!(
                        f,
                        StaticAbilityFragment::ReuseExhaustBeforeFirstActivationOwnTurn
                    )
                })
        })
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn generic_discount<W: CharacteristicWorld + ?Sized>(
    world: &W,
    source: CardId,
    definition: Card,
    ability: u16,
) -> u32 {
    if !definition.activation(ability).exhaust || world.object(source).zone != Zone::Battlefield {
        return 0;
    }
    let controller = world.object(source).controller;
    world
        .static_sources()
        .iter()
        .copied()
        .filter(|&id| {
            id != source
                && world.object(id).zone == Zone::Battlefield
                && world.object(id).controller == controller
        })
        .flat_map(|id| {
            world
                .object(id)
                .card
                .static_ability_groups()
                .iter()
                .flat_map(move |group| {
                    group.fragments.iter().filter_map(move |f| {
                        let StaticAbilityFragment::OtherControlledExhaustGenericReduction(amount) =
                            *f
                        else {
                            return None;
                        };
                        world
                            .fragment_available(id, |candidate| std::ptr::eq(candidate, f))
                            .then_some(amount)
                    })
                })
        })
        .fold(0u32, u32::saturating_add)
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct GrantedExhaustUseView {
    pub source: ObjectRef,
    pub grantor: ObjectRef,
    pub definition: Card,
    pub ability: u16,
    pub origin: Option<u64>,
    pub turn: u32,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) type GrantUses = Option<SharedVec<GrantedExhaustUseView>>;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn grant_use_bytes(rows: &GrantUses) -> usize {
    rows.as_ref().map_or(0, |rows| {
        std::mem::size_of::<Vec<GrantedExhaustUseView>>()
            + rows.len() * std::mem::size_of::<GrantedExhaustUseView>()
    })
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    #[allow(
        clippy::too_many_arguments,
        reason = "An acquired ability is qualified by both exact objects and its immutable origin"
    )]
    pub(super) fn external_exhaust_readiness(
        &self,
        source: ObjectRef,
        grantor: ObjectRef,
        definition: Card,
        index: u16,
        actor: Player,
        ability: &ActivatedAbility,
    ) -> Option<ActivatedAbility> {
        let mut readiness = self.activation_readiness(actor, ability);
        if !ability.exhaust || source == grantor {
            return Some(readiness);
        }
        let origin = self.copied_grant_origin(grantor);
        if let Some(used) = self.state.granted_exhaust_uses.as_ref().and_then(|rows| {
            rows.iter().find(|row| {
                row.source == source
                    && row.grantor == grantor
                    && row.definition == definition
                    && row.ability == index
                    && row.origin == origin
            })
        }) && (readiness.exhaust || ability.once_per_turn && used.turn == self.state.turn)
        {
            return None;
        }
        // Recipient printed-use bits belong to independent abilities, not this grant.
        readiness.exhaust = false;
        readiness.once_per_turn = false;
        Some(readiness)
    }
    pub(super) fn record_external_exhaust_use(&mut self, item: &SpellView, ability: u16) -> bool {
        let Some(source) = item.source else {
            return false;
        };
        let Some(grant) = item
            .attachment_trigger
            .as_ref()
            .filter(|grant| grant.granted && grant.grantor != source.object)
        else {
            return false;
        };
        if !source.card.activation(ability).exhaust {
            return false;
        }
        if !self.on_battlefield(source.object.card)
            || self.object_ref(source.object.card) != source.object
            || !self.on_battlefield(grant.grantor.card)
            || self.object_ref(grant.grantor.card) != grant.grantor
        {
            return true;
        }
        let rows = self.state.granted_exhaust_uses.get_or_insert_default();
        if let Some(row) = rows.iter_mut().find(|row| {
            row.source == source.object
                && row.grantor == grant.grantor
                && row.definition == source.card
                && row.ability == ability
                && row.origin == item.source_ability_origin
        }) {
            row.turn = self.state.turn;
        } else {
            rows.push(GrantedExhaustUseView {
                source: source.object,
                grantor: grant.grantor,
                definition: source.card,
                ability,
                origin: item.source_ability_origin,
                turn: self.state.turn,
            });
        }
        true
    }
    pub(super) fn forget_granted_exhaust_uses(&mut self, source: ObjectRef) {
        if let Some(rows) = &mut self.state.granted_exhaust_uses {
            rows.retain(|row| row.source != source && row.grantor != source);
            if rows.is_empty() {
                self.state.granted_exhaust_uses = None;
            }
        }
    }

    pub(super) fn item_exhaust_reuse_available(&self, item: &SpellView) -> bool {
        matches!(item.kind, StackKind::ActivatedAbility { ability }
            if item.source.is_some_and(|source| source.card.activation(ability).exhaust))
            && reuse_available(self, item.controller)
    }
    pub(super) fn exhaust_reuse_authorized_for_goal(&self, goal: &funding::FundingGoal) -> bool {
        let Some((source, ability, _, definition, slot)) = goal.source.as_ref() else {
            return false;
        };
        if !definition.activation(*ability).exhaust {
            return false;
        }
        let matching = |a: &targeting::Announcement| {
            a.item.controller == goal.actor
                && a.item.kind == (StackKind::ActivatedAbility { ability: *ability })
                && a.item
                    .source
                    .is_some_and(|s| s.object == *source && s.card == *definition)
                && a.item.borrowed_slot == *slot
        };
        self.state
            .announcing
            .iter()
            .chain(
                self.state
                    .mana_announcement_stack
                    .iter()
                    .flat_map(|frames| frames.iter().map(|frame| &frame.announcement)),
            )
            .find(|a| matching(a))
            .map_or_else(
                || reuse_available(self, goal.actor),
                |a| a.exhaust_reuse_authorized,
            )
    }
    pub(super) fn committed_exhaust_activation(&mut self, item: &SpellView) {
        let StackKind::ActivatedAbility { ability } = item.kind else {
            return;
        };
        let Some(source) = item.source else {
            return;
        };
        if item.is_copy
            || !source.card.activation(ability).exhaust
            || self
                .state
                .cast_rollback
                .as_ref()
                .is_some_and(|r| r.replay.is_some())
        {
            return;
        }
        self.state.players[item.controller.index()]
            .turn_facts
            .exhaust_activated_this_turn = true;
        // The event is committed after all costs, including suspended placements,
        // without requiring the activating permanent to have survived those costs.
        let observers: Vec<_> = self
            .state
            .activation_observers
            .as_deref()
            .into_iter()
            .flatten()
            .copied()
            .filter(|&id| {
                let object = &self.state.objects[id as usize];
                matches!(object.zone, Zone::Battlefield | Zone::Graveyard)
                    && object.controller == item.controller
            })
            .map(|id| self.object_view(id))
            .collect();
        for observer in observers {
            for (index, trigger) in observer.card.triggers().iter().enumerate() {
                if trigger.condition.event() == TriggerCondition::ExhaustAbilityActivated {
                    self.queue_trigger(&observer, index);
                }
            }
        }
        if source.card.activation(ability).mana_production().is_none() {
            self.capture_exhaust_copies(item);
        }
    }
}
