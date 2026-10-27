//! Extra trigger instances freeze at the same characteristic boundary as their
//! source view, then expand only after callers finish capturing event metadata.
use super::*;
use crate::cards::{AdditionalTriggerScope, StaticAbilityFragment};
use ability_world::CharacteristicWorld;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct TriggerMultiplicityState {
    providers: Vec<CardId>,
}
impl TriggerMultiplicityState {
    pub(super) fn bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.providers.len() * std::mem::size_of::<CardId>()
    }
}
impl Card {
    fn has_additional_triggers(self) -> bool {
        self.static_ability_groups().iter().any(|group| {
            group.fragments.iter().any(|fragment| {
                matches!(fragment, StaticAbilityFragment::AdditionalTriggers { .. })
            })
        })
    }
}

pub(super) fn additional_triggers<W: CharacteristicWorld + ?Sized>(
    query: &characteristic_query::CharacteristicQuery<'_, W>,
    id: CardId,
) -> usize {
    let Some(state) = &query.game().state.trigger_multiplicity else {
        return 0;
    };
    let recipient = query.object(id);
    if recipient.zone != Zone::Battlefield {
        return 0;
    }
    let mut extra = 0usize;
    for &provider in &state.providers {
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
                if query.value(condition, provider) == 0
                    || !query.continuous_fragment_available(provider, |f| matches!(f, StaticAbilityFragment::AdditionalTriggers { scope: s, condition: c } if *s == scope && *c == condition))
                { continue; }
                let applies = match scope {
                    AdditionalTriggerScope::ControlledSubtype(subtype) => {
                        recipient.controller == object.controller
                            && query.subtypes(id).has(recipient.card, subtype)
                    }
                    AdditionalTriggerScope::EquippedCreature => {
                        query.is_equipment(provider)
                            && query.attached(query.reference(provider), id)
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
