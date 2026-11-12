//! Untap prohibitions are external continuous rules, distinct from abilities on
//! the affected permanent. Prohibitions apply before stun replacements.
use super::*;
use crate::cards::{StaticAbilityFragment, UntapRestriction};

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy)]
pub(super) enum UntapCause {
    Effect,
    ControllerStep(Player),
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    fn has_untap_rules(&self) -> bool {
        if self.state.exertions.is_some() {
            return true;
        }
        if self.state.roles.as_ref().is_some_and(|state| {
            state
                .source_linked_effects
                .iter()
                .any(|row| row.kind == SourceLinkedEffectKind::PreventActivationsUntilUntapped)
        }) {
            return true;
        }
        if self.state.roles.as_ref().is_some_and(|state| {
            state
                .source_linked_effects
                .iter()
                .any(|row| row.kind == SourceLinkedEffectKind::PreventUntap)
        }) {
            return true;
        }
        if self.state.attachment_sources.iter().any(|&id| {
            self.state.objects[id as usize]
                .card
                .triggers()
                .iter()
                .any(|a| a.condition.event() == TriggerCondition::AttachedBecomesUntapped)
        }) {
            return true;
        }
        self.state
            .static_sources
            .iter()
            .chain(&self.state.attachment_sources)
            .any(|&id| {
                self.state.objects[id as usize]
                    .card
                    .static_ability_groups()
                    .iter()
                    .any(|group| {
                        group.fragments.iter().any(|fragment| {
                            matches!(
                                fragment,
                                StaticAbilityFragment::AttachedUntap(_)
                                    | StaticAbilityFragment::SelfUntap(_)
                            )
                        })
                    })
            })
    }
    pub(super) fn untap_allowed(&self, id: CardId, cause: UntapCause) -> bool {
        let recipient = self.object_ref(id);
        if let UntapCause::ControllerStep(player) = cause
            && self.state.exertions.as_ref().is_some_and(|rows| {
                rows.iter()
                    .any(|row| row.object == recipient && row.player == player)
            })
        {
            return false;
        }
        if self.state.roles.as_ref().is_some_and(|state| {
            state.source_linked_effects.iter().any(|row| {
                row.recipient == recipient
                    && row.kind == SourceLinkedEffectKind::PreventUntap
                    && source_linked_effects::active(self, row)
            })
        }) {
            return false;
        }
        let query = self.characteristic_query();
        let object = &self.state.objects[id as usize];
        for (index, group) in object.card.static_ability_groups().iter().enumerate() {
            if !query.continuous_group_available(recipient, index, group) {
                continue;
            }
            for fragment in group.fragments {
                if matches!(
                    fragment,
                    StaticAbilityFragment::SelfUntap(UntapRestriction::All)
                ) || matches!(
                    fragment,
                    StaticAbilityFragment::SelfUntap(UntapRestriction::ControllerStep)
                ) && matches!(cause, UntapCause::ControllerStep(player) if player == object.controller)
                {
                    return false;
                }
                if let StaticAbilityFragment::SelfUntap(UntapRestriction::ControllerStepUnless(
                    value,
                )) = *fragment
                    && matches!(cause, UntapCause::ControllerStep(player) if player == object.controller)
                    && self.stat_value(value, recipient, object.controller) == 0
                {
                    return false;
                }
            }
        }
        for &source in &self.state.attachment_sources {
            let object = &self.state.objects[source as usize];
            if object.zone != Zone::Battlefield || object.attached_to != Some(recipient) {
                continue;
            }
            for (index, group) in object.card.static_ability_groups().iter().enumerate() {
                if !query.continuous_group_available(self.object_ref(source), index, group) {
                    continue;
                }
                for fragment in group.fragments {
                    match fragment {
                        StaticAbilityFragment::AttachedUntap(UntapRestriction::All) => {
                            return false;
                        }
                        StaticAbilityFragment::AttachedUntap(UntapRestriction::ControllerStep)
                            if matches!(cause, UntapCause::ControllerStep(player)
                                if player == self.state.objects[id as usize].controller) =>
                        {
                            return false;
                        }
                        StaticAbilityFragment::AttachedUntap(
                            UntapRestriction::ControllerStepUnless(value),
                        ) if matches!(cause, UntapCause::ControllerStep(player)
                                if player == self.state.objects[id as usize].controller)
                            && self.stat_value(
                                *value,
                                self.object_ref(source),
                                object.controller,
                            ) == 0 =>
                        {
                            return false;
                        }
                        _ => (),
                    }
                }
            }
        }
        true
    }
    /// Return actual untaps controlled by the player taking this untap step.
    /// Effect untaps never contribute to this step receipt.
    pub(super) fn untap_objects(&mut self, ids: &[CardId], cause: UntapCause) -> u64 {
        let step_player = match cause {
            UntapCause::Effect => None,
            UntapCause::ControllerStep(player) => Some(player),
        };
        if !self.has_untap_rules() {
            let mut count = 0;
            for &id in ids {
                if self.on_battlefield(id) {
                    let object = &mut self.state.objects[id as usize];
                    if object.untap() && Some(object.controller) == step_player {
                        count += 1;
                    }
                }
            }
            return count;
        }
        // Decide the entire group before removing any stun counter. Untaps
        // caused by the turn-based action or one effect occur simultaneously.
        let allowed: Vec<_> = ids
            .iter()
            .copied()
            .filter(|&id| self.on_battlefield(id) && self.untap_allowed(id, cause))
            .collect();
        let mut count = 0;
        let mut untapped = Vec::new();
        for id in allowed {
            let object = &mut self.state.objects[id as usize];
            if object.untap() {
                if Some(object.controller) == step_player {
                    count += 1;
                }
                untapped.push(self.object_ref(id));
            }
        }
        self.forget_untapped_activation_locks(&untapped);
        self.attached_untap_triggers(&untapped);
        count
    }

    pub(super) fn untap_active_player(&mut self) {
        let active = self.state.active;
        let also_untap = |fragment: &StaticAbilityFragment| {
            matches!(fragment, StaticAbilityFragment::UntapDuringOtherUntapSteps)
        };
        let also_untap_creatures = |fragment: &StaticAbilityFragment| {
            matches!(
                fragment,
                StaticAbilityFragment::UntapControlledCreaturesDuringOtherUntapSteps
            )
        };
        let mut other = Vec::new();
        let mut other_creatures = false;
        for &id in &self.state.static_sources {
            let object = &self.state.objects[id as usize];
            if object.zone != Zone::Battlefield || object.controller == active {
                continue;
            }
            let groups = object.card.static_ability_groups();
            if object.tapped
                && groups
                    .iter()
                    .any(|group| group.fragments.iter().any(also_untap))
                && self.continuous_fragment_available(id, also_untap)
            {
                other.push(id);
            }
            if !other_creatures
                && groups
                    .iter()
                    .any(|group| group.fragments.iter().any(also_untap_creatures))
                && self.continuous_fragment_available(id, also_untap_creatures)
            {
                other_creatures = true;
            }
        }
        if other_creatures {
            other.extend(
                self.state
                    .objects
                    .iter()
                    .enumerate()
                    .filter_map(|(id, object)| {
                        let id = id as CardId;
                        (object.zone == Zone::Battlefield
                            && object.controller != active
                            && object.tapped
                            && self.is_creature(id))
                        .then_some(id)
                    }),
            );
            // One simultaneous untap attempt per permanent, even when it has
            // an individual off-turn untap ability and a controller-wide one.
            other.sort_unstable();
            other.dedup();
        }
        let count = if other.is_empty() && !self.has_untap_rules() {
            let mut count = 0;
            for object in &mut self.state.objects {
                if object.zone == Zone::Battlefield && object.controller == active && object.untap()
                {
                    count += 1;
                }
            }
            count
        } else {
            let ids: Vec<_> = self
                .state
                .objects
                .iter()
                .enumerate()
                .filter_map(|(id, object)| {
                    (object.zone == Zone::Battlefield && object.controller == active)
                        .then_some(id as CardId)
                })
                .chain(other)
                .collect();
            self.untap_objects(&ids, UntapCause::ControllerStep(active))
        };
        // CR701.43: all overlapping exertions for this player's next actual
        // untap expire together, even if another player now controls a recipient.
        self.expire_exertions(active);
        self.queue_untap_step_triggers(active, count);
    }

    fn queue_untap_step_triggers(&mut self, active: Player, amount: u64) {
        if amount == 0
            || !self.state.step_sources.iter().any(|&id| {
                self.state.objects[id as usize]
                    .card
                    .triggers()
                    .iter()
                    .any(|ability| {
                        ability.condition.event() == TriggerCondition::OwnUntapStepUntaps
                    })
            })
        {
            return;
        }
        // CR502.4 and 503.1a: enqueue now and publish with upkeep triggers.
        for (source, grant) in self.indexed_trigger_sources(&self.state.step_sources) {
            if source.controller != active {
                continue;
            }
            let definition = grant.as_ref().map_or(source.card, |row| row.granting_card);
            for (index, ability) in definition.triggers().iter().enumerate() {
                if ability.condition.event() == TriggerCondition::OwnUntapStepUntaps
                    && self.queue_trigger_from(&source, index, grant.map(Box::new))
                {
                    self.state
                        .pending_triggers
                        .last_mut()
                        .unwrap()
                        .set_event_object(EventObjectView {
                            object: source.reference,
                            facts: EventObjectFacts::UntapStepCount(amount),
                            semantic_objects: None,
                            conniver_creature: None,
                            subtype_lki: None,
                        });
                }
            }
        }
    }
}
