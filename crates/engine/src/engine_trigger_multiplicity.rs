//! Extra trigger instances freeze at the same characteristic boundary as their
//! source view, then expand only after callers finish capturing event metadata.
use super::*;
use crate::cards::{AdditionalTriggerScope, StaticAbilityFragment};
use ability_world::CharacteristicWorld;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct TriggerMultiplicityState {
    providers: Vec<CardId>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl TriggerMultiplicityState {
    pub(super) fn providers(game: &Game) -> &[CardId] {
        game.state
            .trigger_multiplicity
            .as_deref()
            .map_or(&[], |state| &state.providers)
    }

    pub(super) fn bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.providers.len() * std::mem::size_of::<CardId>()
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Card {
    fn has_additional_triggers(self) -> bool {
        self.static_ability_groups().iter().any(|group| {
            group.fragments.iter().any(|fragment| {
                matches!(fragment, StaticAbilityFragment::AdditionalTriggers { .. })
            })
        })
    }
}

#[derive(Clone, Copy)]
enum TriggerCause<'a> {
    Ordinary,
    Attack,
    Entry(&'a [ObjectRef]),
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn additional_triggers<W: CharacteristicWorld + ?Sized>(
    query: &characteristic_query::CharacteristicQuery<'_, W>,
    id: CardId,
) -> usize {
    additional_triggers_for_event(query, id, TriggerCause::Ordinary)
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn additional_attack_triggers<W: CharacteristicWorld + ?Sized>(
    query: &characteristic_query::CharacteristicQuery<'_, W>,
    id: CardId,
) -> usize {
    additional_triggers_for_event(query, id, TriggerCause::Attack)
}
fn additional_triggers_for_event<W: CharacteristicWorld + ?Sized>(
    query: &characteristic_query::CharacteristicQuery<'_, W>,
    id: CardId,
    cause: TriggerCause<'_>,
) -> usize {
    let providers = query.trigger_multiplier_sources();
    if providers.is_empty() {
        return 0;
    }
    let recipient = query.object(id);
    if recipient.zone != Zone::Battlefield {
        return 0;
    }
    let mut extra = 0usize;
    for &provider in providers {
        let object = query.object(provider);
        if object.zone != Zone::Battlefield {
            continue;
        }
        for group in object.card.static_ability_groups() {
            for fragment in group.fragments {
                let StaticAbilityFragment::AdditionalTriggers { scope, condition } = *fragment
                else {
                    continue;
                };
                let matches_event = match scope {
                    AdditionalTriggerScope::ControlledPermanentAttack => {
                        matches!(cause, TriggerCause::Attack)
                    }
                    AdditionalTriggerScope::ControlledLandOrSubtypeEntry(_) => {
                        matches!(cause, TriggerCause::Entry(_))
                    }
                    _ => matches!(cause, TriggerCause::Ordinary),
                };
                if !matches_event {
                    continue;
                }
                if query.value(condition, provider) == 0
                    || !query.continuous_fragment_available(provider, |f| matches!(f, StaticAbilityFragment::AdditionalTriggers { scope: s, condition: c } if *s == scope && *c == condition))
                { continue; }
                let applies = match scope {
                    AdditionalTriggerScope::ControlledSubtype(subtype)
                    | AdditionalTriggerScope::OtherControlledSubtype(subtype)
                    | AdditionalTriggerScope::ControlledCreatureSubtype(subtype) => {
                        recipient.controller == object.controller
                            && (!matches!(scope, AdditionalTriggerScope::OtherControlledSubtype(_))
                                || provider != id)
                            && query.subtypes(id).has(recipient.card, subtype)
                            && (!matches!(
                                scope,
                                AdditionalTriggerScope::ControlledCreatureSubtype(_)
                            ) || query.kinds(id) & 1 != 0)
                    }
                    AdditionalTriggerScope::ControlledPermanentAttack => {
                        recipient.controller == object.controller
                    }
                    AdditionalTriggerScope::EquippedCreature => {
                        query.is_equipment(provider)
                            && query.attached(query.reference(provider), id)
                    }
                    AdditionalTriggerScope::SourceAndAttachedEquipment => {
                        let equipped = |equipment| {
                            query.object(equipment).attached_to == Some(query.reference(provider))
                                && query.is_equipment(equipment)
                                && query.attached(query.reference(equipment), provider)
                        };
                        if id == provider {
                            (0..query.state.objects.len())
                                .any(|equipment| equipped(equipment as CardId))
                        } else {
                            equipped(id)
                        }
                    }
                    AdditionalTriggerScope::ControlledLandOrSubtypeEntry(subtype) => {
                        let TriggerCause::Entry(entrants) = cause else {
                            unreachable!("entry trigger cause")
                        };
                        recipient.controller == object.controller
                            && entrants.iter().any(|&entrant| {
                                let entrant_object = query.object(entrant.card);
                                entrant_object.zone == Zone::Battlefield
                                    && query.reference(entrant.card) == entrant
                                    && entrant_object.controller == object.controller
                                    && (query.kinds(entrant.card) & 2 != 0
                                        || query
                                            .subtypes(entrant.card)
                                            .has(entrant_object.card, subtype))
                            })
                    }
                };
                if applies {
                    extra = extra
                        .checked_add(1)
                        .capacity("trigger multiplicity exhausted");
                }
            }
        }
    }
    extra
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn register_trigger_multiplier_source(&mut self, id: CardId) {
        if !self.state.objects[id as usize]
            .card
            .has_additional_triggers()
        {
            return;
        }
        let state = self
            .state
            .trigger_multiplicity
            .get_or_insert_with(Default::default);
        if !state.providers.contains(&id) {
            state.providers.push(id);
        }
    }
    pub(super) fn forget_trigger_multiplier_source(&mut self, id: CardId) {
        if let Some(state) = &mut self.state.trigger_multiplicity {
            state.providers.retain(|&provider| provider != id);
            if state.providers.is_empty() {
                self.state.trigger_multiplicity = None;
            }
        }
    }
    pub(super) fn add_entry_trigger_multiplicity(
        &mut self,
        source: &ObjectView,
        entrants: &[ObjectRef],
    ) {
        if TriggerMultiplicityState::providers(self).is_empty() || source.zone != Zone::Battlefield
        {
            return;
        }
        let packet = self
            .state
            .pending_triggers
            .last()
            .expect("queued entry trigger");
        let StackKind::TriggeredAbility { ability } = packet.kind else {
            unreachable!("entry trigger kind")
        };
        if packet.trigger_definition().triggers()[usize::from(ability)]
            .condition
            .once_per_turn()
        {
            return;
        }
        let extra = additional_triggers_for_event(
            &self.characteristic_query(),
            source.id,
            TriggerCause::Entry(entrants),
        );
        let packet = self.state.pending_triggers.last_mut().unwrap();
        packet.additional_triggers = packet
            .additional_triggers
            .checked_add(extra)
            .capacity("trigger multiplicity exhausted");
    }

    pub(super) fn expand_trigger_multiplicity(&mut self) {
        if !self
            .state
            .pending_triggers
            .iter()
            .any(|item| item.additional_triggers != 0)
        {
            return;
        }
        let originals = std::mem::take(&mut self.state.pending_triggers);
        for mut item in originals {
            let extra = std::mem::take(&mut item.additional_triggers);
            for _ in 0..extra {
                let mut additional = item.clone();
                additional.id = self.next_stack_id();
                debug_assert!(!additional.is_copy);
                self.state.pending_triggers.push(additional);
            }
            self.state.pending_triggers.push(item);
        }
    }
}
