//! Persistent public designations are sparse and bind exact battlefield incarnations.
use super::*;

impl Game {
    pub(super) fn is_suspected(&self, id: CardId) -> bool {
        self.on_battlefield(id)
            && self.state.suspected.as_ref().is_some_and(|objects| {
                objects
                    .binary_search_by_key(&id, |r| r.card)
                    .is_ok_and(|i| objects[i] == self.object_ref(id))
            })
    }
    pub(super) fn set_suspected(&mut self, object: ObjectRef, suspected: bool) {
        // Pronouns in return-and-suspect effects can identify a permanent which
        // is no longer a creature. The designation remains if its types change.
        if !self.on_battlefield(object.card) || self.object_ref(object.card) != object {
            return;
        }
        if !suspected {
            self.clear_suspected_reference(object);
            return;
        }
        let objects = self
            .state
            .suspected
            .get_or_insert_with(|| Box::new(Vec::new()));
        match objects.binary_search_by_key(&object.card, |r| r.card) {
            Ok(index) => objects[index] = object,
            Err(index) => objects.insert(index, object),
        }
    }
    pub(super) fn clear_suspected_reference(&mut self, object: ObjectRef) {
        if let Some(objects) = &mut self.state.suspected {
            if let Ok(index) = objects.binary_search_by_key(&object.card, |r| r.card)
                && objects[index] == object
            {
                objects.remove(index);
            }
            if objects.is_empty() {
                self.state.suspected = None;
            }
        }
    }
    pub(super) fn suspect_subject(&mut self, item: &SpellView, subject: Subject) {
        let objects: Vec<_> = match subject {
            Subject::Source => vec![item.object],
            Subject::Entering => self
                .entering_subject(item)
                .map(|id| self.object_ref(id))
                .into_iter()
                .collect(),
            Subject::Attached => self
                .attached_subject(item)
                .map(|id| self.object_ref(id))
                .into_iter()
                .collect(),
            Subject::Targets => item
                .targets
                .iter()
                .filter(|b| self.same_target_object(**b))
                .filter_map(|b| match b.target {
                    Target::Card(id) => Some(self.object_ref(id)),
                    _ => None,
                })
                .collect(),
        };
        for object in objects {
            self.set_suspected(object, true);
        }
    }
    pub(super) fn unsuspect_creatures(
        &mut self,
        item: &SpellView,
        filter: crate::cards::CreatureFilter,
    ) {
        let objects: Vec<_> = self
            .state
            .suspected
            .as_ref()
            .map_or(&[][..], |v| v.as_slice())
            .iter()
            .copied()
            .filter(|r| {
                self.on_battlefield(r.card)
                    && self.object_ref(r.card) == *r
                    && self.creature_matches(filter, item.controller, item.object, r.card)
            })
            .collect();
        for object in objects {
            self.clear_suspected_reference(object);
        }
    }
    pub(super) fn return_granted_source_suspected(&mut self, item: &SpellView, tapped: bool) {
        if let Some(mut movement) = self.granted_source_entry(item, None, None) {
            movement.tapped =
                tapped || self.enters_tapped_for(movement.before.id, movement.controller);
            movement.suspect_after = true;
            self.stage_entries(vec![movement], None);
        }
    }
    pub(super) fn suspected_bytes(&self) -> usize {
        self.state.suspected.as_ref().map_or(0, |objects| {
            std::mem::size_of::<Vec<ObjectRef>>() + objects.len() * std::mem::size_of::<ObjectRef>()
        })
    }
}
