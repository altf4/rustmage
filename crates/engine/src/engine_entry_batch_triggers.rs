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
    pub(super) fn capture_owned_event_object_departures(&mut self, before: &[&ObjectView]) {
        let mut capture = |item: &mut SpellView| {
            item.capture_entry_group_departure(before);
            item.capture_chosen_cost_creature_departure(before);
        };
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
    fn queue_entry_batch_without_spent_mana(
        &mut self,
        source: &ObjectView,
        index: usize,
        events: &[ZoneChange],
        suppression: u16,
    ) {
        let mut entrants = Vec::new();
        for event in events {
            if !event.moved
                || event.destination != Zone::Battlefield
                || !self.on_battlefield(event.after.card)
                || self.object_ref(event.after.card) != event.after
            {
                continue;
            }
            let entrant = self.object_view(event.after.card);
            if entrant.controller != source.controller
                || entrant.kinds & suppression != 0
                || (entrant.reference != source.reference
                    && (!entrant.is_creature() || entrant.is_token()))
            {
                continue;
            }
            if event
                .cast_event
                .as_ref()
                .is_some_and(|cast| cast.payment != [0; 6])
            {
                return;
            }
            entrants.push(entrant.reference);
        }
        if !entrants.is_empty() && self.queue_trigger(source, index) {
            self.add_entry_trigger_multiplicity(source, &entrants);
            self.state
                .pending_triggers
                .last_mut()
                .unwrap()
                .refer_objects(entrants);
        }
    }

    pub(super) fn queue_entry_copy_trigger(
        &mut self,
        source: &ObjectView,
        index: usize,
        entrants: &[ObjectRef],
    ) {
        if entrants.is_empty() || !self.queue_trigger(source, index) {
            return;
        }
        self.add_entry_trigger_multiplicity(source, entrants);
        let rows = entrants
            .iter()
            .copied()
            .map(|object| EntryBatchCopyView {
                object,
                recipe: self.copiable_recipe(object),
            })
            .collect();
        let packet = self.state.pending_triggers.last_mut().unwrap();
        packet.set_event_object(EventObjectView {
            object: entrants[0],
            facts: EventObjectFacts::CopiableGroup(rows),
            semantic_objects: None,
            conniver_creature: None,
            subtype_lki: None,
        });
        packet.refer_objects(entrants.iter().copied());
    }

    pub(super) fn entry_predicate_matches(
        &self,
        predicate: crate::cards::EntryPredicate,
        source: &ObjectView,
        entered: &ObjectView,
    ) -> bool {
        use crate::cards::EntryPredicate;
        let same = entered.reference == source.reference;
        match predicate {
            EntryPredicate::SourceOr(other) => {
                same || self.entry_predicate_matches(*other, source, entered)
            }
            EntryPredicate::CreatureWithKeyword(keyword) => {
                entered.is_creature() && entered.keywords.contains(keyword)
            }
            EntryPredicate::CreatureToughnessGreaterThanPower => {
                entered.is_creature() && entered.toughness > entered.power
            }
            EntryPredicate::SourceChosenCreatureType => {
                matches!(source.entry_choice, Some(EntryChoiceValue::CreatureType(typ)) if entered.has_subtype(typ.name()))
            }
            EntryPredicate::SourceOrSubtypeAny(subtypes) => {
                same || subtypes.iter().any(|typ| entered.has_subtype(typ))
            }
            EntryPredicate::Token => entered.is_token(),
            EntryPredicate::FaceDown => self.is_face_down(entered.reference),
            EntryPredicate::NontokenArtifact => !entered.is_token() && entered.kinds & 16 != 0,
            EntryPredicate::NontokenEquipment => {
                !entered.is_token() && entered.kinds & 16 != 0 && entered.has_subtype("Equipment")
            }
            EntryPredicate::LegendaryCreature => entered.is_creature() && entered.is_legendary(),
            EntryPredicate::LegendarySubtype(subtype) => {
                entered.is_legendary() && entered.has_subtype(subtype)
            }
            EntryPredicate::ArtifactManaValueAtLeast(minimum) => {
                entered.kinds & 16 != 0 && self.object_mana_value(entered.id) >= u64::from(minimum)
            }
            EntryPredicate::CreatureManaValueAtLeast(minimum) => {
                entered.is_creature() && self.object_mana_value(entered.id) >= u64::from(minimum)
            }
            EntryPredicate::CreatureWithoutSubtype(subtype) => {
                entered.is_creature() && !entered.has_subtype(subtype)
            }
            EntryPredicate::CreaturePowerAtLeast(minimum) => {
                entered.is_creature() && entered.power >= i128::from(minimum)
            }
            EntryPredicate::CreaturePowerAtMost(maximum) => {
                entered.is_creature() && entered.power <= i128::from(maximum)
            }
            EntryPredicate::SubtypeAny(subtypes) => {
                subtypes.iter().any(|subtype| entered.has_subtype(subtype))
            }
            EntryPredicate::KindOrSubtype { kinds, subtype } => {
                entered.kinds & kinds != 0 || entered.has_subtype(subtype)
            }
            EntryPredicate::NoncreatureNonland => entered.kinds & (1 | 2) == 0,
        }
    }

    pub(super) fn batch_entry_triggers(
        &mut self,
        events: &[ZoneChange],
        sources: &[ObjectView],
        suppression: u16,
    ) {
        for source in sources {
            for (index, ability) in source.card.triggers().iter().enumerate() {
                let (other, predicate) = match ability.condition.event() {
                    TriggerCondition::CreaturesFromGraveyardEnterYou { other } => (other, None),
                    TriggerCondition::FilteredEntersBatchYou { predicate, other }
                        if source.zone == Zone::Battlefield =>
                    {
                        (other, Some(predicate))
                    }
                    TriggerCondition::CreaturesEnterYouWithoutManaSpent
                        if source.zone == Zone::Battlefield =>
                    {
                        self.queue_entry_batch_without_spent_mana(
                            source,
                            index,
                            events,
                            suppression,
                        );
                        continue;
                    }
                    _ => continue,
                };
                let entrants: Vec<_> = events
                    .iter()
                    .filter_map(|event| {
                        if !event.moved
                            || event.destination != Zone::Battlefield
                            || (predicate.is_none()
                                && event.before.zone != Zone::Graveyard
                                && event.cast_origin != Some(Zone::Graveyard))
                            || (other && event.after == source.reference)
                            || !self.on_battlefield(event.after.card)
                            || self.object_ref(event.after.card) != event.after
                        {
                            return None;
                        }
                        let entered = self.object_view(event.after.card);
                        (entered.controller == source.controller
                            && entered.kinds & suppression == 0
                            && predicate.map_or_else(
                                || entered.is_creature(),
                                |predicate| {
                                    self.entry_predicate_matches(predicate, source, &entered)
                                },
                            ))
                        .then_some(event.after)
                    })
                    .collect();
                if predicate.is_none() {
                    self.queue_entry_copy_trigger(source, index, &entrants);
                } else if !entrants.is_empty() && self.queue_trigger(source, index) {
                    self.add_entry_trigger_multiplicity(source, &entrants);
                    self.state
                        .pending_triggers
                        .last_mut()
                        .unwrap()
                        .refer_objects(entrants);
                }
            }
        }
    }
}
