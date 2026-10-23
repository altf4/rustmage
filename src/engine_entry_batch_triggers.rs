//! Entry qualification belongs to one whole committed event, not later objects.
use super::*;
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct EntryBatchEventView {
    pub objects: Vec<ObjectRef>,
    /// Last battlefield copiable values, keyed to the departed event incarnation.
    /// Live members use current values instead (CR608.2h/400.7/707.2).
    pub last_known: Vec<EntryBatchCopyView>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct EntryBatchCopyView {
    pub object: ObjectRef,
    pub recipe: CopyRecipe,
}
impl SpellView {
    fn capture_entry_batch_departure(&mut self, object: ObjectRef, recipe: CopyRecipe) {
        if let Some(event) = &mut self.entry_batch_event
            && event.objects.contains(&object)
            && !event.last_known.iter().any(|row| row.object == object)
        {
            event.last_known.push(EntryBatchCopyView { object, recipe });
        }
        if let Some(event) = &mut self.copy_event {
            event.spell.capture_entry_batch_departure(object, recipe);
        }
    }
}
impl Game {
    pub(super) fn entry_batch_copy_recipe(&self, object: ObjectRef) -> Option<CopyRecipe> {
        let event = self
            .state
            .resolving
            .as_ref()?
            .item
            .entry_batch_event
            .as_ref()?;
        if !event.objects.contains(&object) {
            return None;
        }
        if self.on_battlefield(object.card) && self.object_ref(object.card) == object {
            Some(self.copiable_recipe(object))
        } else {
            event
                .last_known
                .iter()
                .find(|row| row.object == object)
                .map(|row| row.recipe)
        }
    }
    pub(super) fn capture_entry_batch_copy_departure(&mut self, before: &ObjectView) {
        if before.zone != Zone::Battlefield {
            return;
        }
        let object = before.reference;
        let recipe = before
            .copy_recipe
            .unwrap_or_else(|| CopyRecipe::new(before.card));
        let mut capture = |item: &mut SpellView| item.capture_entry_batch_departure(object, recipe);
        self.state.stack.iter_mut().for_each(&mut capture);
        self.state
            .pending_triggers
            .iter_mut()
            .for_each(&mut capture);
        self.state
            .ordering_triggers
            .iter_mut()
            .for_each(&mut capture);
        if let Some(a) = &mut self.state.announcing {
            capture(&mut a.item);
        }
        if let Some(r) = &mut self.state.resolving {
            capture(&mut r.item);
        }
        if let Some(copy) = &mut self.state.stack_copy_data {
            copy.visit_owned_packets(&mut capture);
        }
        if let Some(replacements) = &mut self.state.replacements {
            replacements.visit_owned_packets(&mut capture);
        }
        if let Some(replacements) = self
            .state
            .entry_batch
            .as_mut()
            .and_then(|batch| batch.replacement_resume.as_deref_mut())
        {
            replacements.visit_owned_packets(&mut capture);
        }
        if let Some(draws) = &mut self.state.delayed_combat_draws {
            draws.iter_mut().for_each(&mut capture);
        }
        if let Some(returns) = &mut self.state.delayed_returns {
            for row in returns.iter_mut() {
                capture(&mut row.item);
            }
        }
        if let Some(roles) = &mut self.state.roles {
            for row in &mut roles.delayed_deaths {
                capture(&mut row.item);
            }
        }
    }

    pub(super) fn batch_graveyard_entry_triggers(
        &mut self,
        events: &[ZoneChange],
        sources: &[ObjectView],
        suppression: u16,
    ) {
        for source in sources {
            for (index, ability) in source.card.triggers().iter().enumerate() {
                let TriggerCondition::CreaturesFromGraveyardEnterYou { other } =
                    ability.condition.event()
                else {
                    continue;
                };
                let objects: Vec<_> = events
                    .iter()
                    .filter_map(|event| {
                        if !event.moved
                            || event.destination != Zone::Battlefield
                            || (event.before.zone != Zone::Graveyard
                                && event.cast_origin != Some(Zone::Graveyard))
                            || (other && event.after == source.reference)
                            || !self.on_battlefield(event.after.card)
                            || self.object_ref(event.after.card) != event.after
                        {
                            return None;
                        }
                        let entered = self.object_view(event.after.card);
                        (entered.is_creature()
                            && entered.controller == source.controller
                            && entered.kinds & suppression == 0)
                            .then_some(event.after)
                    })
                    .collect();
                if !objects.is_empty() && self.queue_trigger(source, index) {
                    self.state
                        .pending_triggers
                        .last_mut()
                        .unwrap()
                        .entry_batch_event = Some(Box::new(EntryBatchEventView {
                        objects,
                        last_known: Vec::new(),
                    }));
                }
            }
        }
    }
}
