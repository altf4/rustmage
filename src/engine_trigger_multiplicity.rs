//! Additive trigger counts frozen at the event, expanded only after packet capture.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct TriggerMultiplicityView {
    pub trigger: u64,
    pub additional: u64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct TriggerProvider {
    source: ObjectRef,
    controller: Player,
    subtype: &'static str,
    other: bool,
    creature_only: bool,
}
impl Game {
    pub(super) fn freeze_trigger_providers(&self) -> Option<Box<[TriggerProvider]>> {
        let mut providers = Vec::new();
        for &id in &self.state.static_sources {
            let card = self.state.objects[id as usize].card;
            for fragment in card
                .static_ability_groups()
                .iter()
                .flat_map(|g| g.fragments)
            {
                if let Some((subtype, other, creature_only)) =
                    fragment.additional_subtype_triggers()
                    && self.continuous_fragment_available(id, |f| {
                        f.additional_subtype_triggers() == Some((subtype, other, creature_only))
                    })
                {
                    providers.push(TriggerProvider {
                        source: self.object_ref(id),
                        controller: self.state.objects[id as usize].controller,
                        subtype,
                        other,
                        creature_only,
                    });
                }
            }
        }
        (!providers.is_empty()).then(|| providers.into_boxed_slice())
    }
    pub(super) fn capture_trigger_multiplicity(
        &mut self,
        source: &ObjectView,
        index: usize,
        providers: Option<&[TriggerProvider]>,
    ) {
        let item = self.state.pending_triggers.last().unwrap();
        let condition = item.trigger_definition().triggers()[index].condition;
        self.capture_intrinsic_trigger_multiplicity(
            source,
            providers,
            condition.once_per_turn() || condition.event() == TriggerCondition::Reflexive,
        );
    }
    pub(super) fn capture_intrinsic_trigger_multiplicity(
        &mut self,
        source: &ObjectView,
        providers: Option<&[TriggerProvider]>,
        limited_or_reflexive: bool,
    ) {
        let trigger = self.state.pending_triggers.last().unwrap().id;
        // CR603.2d excludes delayed and reflexive abilities. Trigger limits apply
        // to actual occurrences, unlike a copy already on the stack.
        let additional = if source.zone != Zone::Battlefield || limited_or_reflexive {
            0
        } else {
            providers
                .unwrap_or_default()
                .iter()
                .fold(0u64, |n, provider| {
                    if provider.controller == source.controller
                        && (!provider.other || provider.source != source.reference)
                        && source.has_subtype(provider.subtype)
                        && (!provider.creature_only || source.is_creature())
                    {
                        n.checked_add(1).capacity("trigger multiplicity exhausted")
                    } else {
                        n
                    }
                })
        };
        if let Some(roles) = &mut self.state.roles {
            roles
                .trigger_multiplicities
                .retain(|r| r.trigger != trigger);
        }
        if additional != 0 {
            self.state
                .roles
                .get_or_insert_with(Default::default)
                .trigger_multiplicities
                .push(TriggerMultiplicityView {
                    trigger,
                    additional,
                });
        }
    }
    pub(super) fn expand_pending_trigger_multiplicities(&mut self) {
        let Some(roles) = &mut self.state.roles else {
            return;
        };
        let counts = std::mem::take(&mut roles.trigger_multiplicities);
        if counts.is_empty() {
            return;
        }
        let originals = std::mem::take(&mut self.state.pending_triggers);
        for item in originals {
            let additional = counts
                .iter()
                .find(|r| r.trigger == item.id)
                .map_or(0, |r| r.additional);
            for _ in 0..additional {
                let mut extra = item.clone();
                extra.id = self.next_stack_id();
                extra.is_copy = false;
                self.state.pending_triggers.push(extra);
            }
            self.state.pending_triggers.push(item);
        }
    }
}
