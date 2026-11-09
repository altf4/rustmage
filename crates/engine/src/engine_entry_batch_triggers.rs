//! Exact entry-event cohorts with last battlefield copiable values, never new incarnations.
use super::*;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct EntryBatchCopyView {
    pub object: ObjectRef,
    pub recipe: CopyRecipe,
}
impl SpellView {
    fn capture_entry_group_departure(&mut self, before: &[&ObjectView]) {
        if let Some(event) = &mut self.event_object
            && let EventObjectFacts::CopiableGroup(rows) = &mut event.facts
        {
            for row in rows {
                if let Some(view) = before
                    .iter()
                    .find(|view| view.zone == Zone::Battlefield && view.reference == row.object)
                {
                    row.recipe = view
                        .copy_recipe
                        .unwrap_or_else(|| CopyRecipe::new(view.card));
                }
            }
        }
        if let Some(event) = &mut self.copy_event {
            event.spell.capture_entry_group_departure(before);
        }
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn entry_batch_copy_options(&self) -> Vec<EntryBatchCopyView> {
        let Some(event) = self
            .state
            .resolving
            .as_ref()
            .and_then(|r| r.item.event_object.as_ref())
        else {
            return Vec::new();
        };
        let EventObjectFacts::CopiableGroup(rows) = &event.facts else {
            return Vec::new();
        };
        rows.iter()
            .map(|row| {
                if self
                    .state
                    .objects
                    .get(row.object.card as usize)
                    .is_some_and(|object| {
                        object.zone == Zone::Battlefield
                            && object.generation == row.object.generation
                    })
                {
                    EntryBatchCopyView {
                        object: row.object,
                        recipe: self.copiable_recipe(row.object),
                    }
                } else {
                    *row
                }
            })
            .collect()
    }
    pub(super) fn choose_entry_batch_copy(&mut self, object: ObjectRef) {
        let recipe = self
            .entry_batch_copy_options()
            .iter()
            .find(|row| row.object == object)
            .unwrap()
            .recipe;
        let controller = self.state.resolving.as_ref().unwrap().item.controller;
        self.stage_copy_creation(controller, recipe, 1);
        self.run_program();
    }
    pub(super) fn capture_entry_batch_copy_departures(&mut self, before: &[&ObjectView]) {
        let mut capture = |item: &mut SpellView| item.capture_entry_group_departure(before);
        self.state.stack.iter_mut().for_each(&mut capture);
        self.state
            .pending_triggers
            .iter_mut()
            .for_each(&mut capture);
        self.state
            .ordering_triggers
            .iter_mut()
            .for_each(&mut capture);
        if let Some(row) = &mut self.state.announcing {
            capture(&mut row.item);
        }
        if let Some(row) = &mut self.state.resolving {
            capture(&mut row.item);
        }
        if let Some(row) = &mut self.state.activation_payment {
            capture(&mut row.item);
        }
        if let Some(rows) = &mut self.state.mana_announcement_stack {
            for row in rows.iter_mut() {
                capture(&mut row.announcement.item);
            }
        }
        if let Some(copy) = &mut self.state.stack_copy_data {
            copy.visit_owned_packets(&mut capture);
        }
        if let Some(rows) = &mut self.state.replacements {
            rows.visit_owned_packets(&mut capture);
        }
        if let Some(rows) = self
            .state
            .entry_batch
            .as_mut()
            .and_then(|batch| batch.replacement_resume.as_deref_mut())
        {
            rows.visit_owned_packets(&mut capture);
        }
        if let Some(rows) = &mut self.state.delayed_returns {
            for row in rows.iter_mut() {
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
                let rows: Vec<_> = events
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
                            .then(|| EntryBatchCopyView {
                                object: event.after,
                                recipe: self.copiable_recipe(event.after),
                            })
                    })
                    .collect();
                if !rows.is_empty() && self.queue_trigger(source, index) {
                    let objects: Vec<_> = rows.iter().map(|row| row.object).collect();
                    self.add_entry_trigger_multiplicity(source, &objects);
                    let packet = self.state.pending_triggers.last_mut().unwrap();
                    packet.set_event_object(EventObjectView {
                        object: objects[0],
                        facts: EventObjectFacts::CopiableGroup(rows),
                        semantic_objects: None,
                        conniver_creature: None,
                        subtype_lki: None,
                    });
                    packet.refer_objects(objects);
                }
            }
        }
    }
}
