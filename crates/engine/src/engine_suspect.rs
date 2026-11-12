//! Persistent public designations are sparse and bind exact battlefield incarnations.
use super::ability_world::CharacteristicWorld;
use super::*;
use crate::cards::StaticAbilityFragment;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn is_suspected(&self, id: CardId) -> bool {
        CharacteristicWorld::suspected(self, id)
    }
    pub(super) fn suspected_reference(&self, object: ObjectRef) -> bool {
        self.state.suspected.as_ref().is_some_and(|objects| {
            objects
                .binary_search_by_key(&object.card, |r| r.card)
                .is_ok_and(|i| objects[i] == object)
        })
    }
    fn can_become_suspected(&self, object: ObjectRef) -> bool {
        if !self.on_battlefield(object.card)
            || self.object_ref(object.card) != object
            || self.suspected_reference(object)
        {
            return false;
        }
        let prohibits = |fragment: &StaticAbilityFragment| {
            matches!(fragment, StaticAbilityFragment::AttachedSuspectProhibited)
        };
        let mut query = None;
        !self.attachment_sources().iter().copied().any(|source| {
            let provider = self.object(source);
            provider.zone == Zone::Battlefield
                && provider.attached_to == Some(object)
                && self
                    .definition(source)
                    .static_ability_groups()
                    .iter()
                    .any(|group| group.fragments.iter().any(prohibits))
                && self.attached(self.object_ref(source), object.card)
                && query
                    .get_or_insert_with(|| self.characteristic_query())
                    .continuous_fragment_available(source, prohibits)
        })
    }
    pub(super) fn set_suspected(&mut self, object: ObjectRef, suspected: bool) -> bool {
        if !suspected {
            let changed = self.suspected_reference(object);
            if changed {
                self.clear_suspected_reference(object);
            }
            return changed;
        }
        // Pronouns in return-and-suspect effects can identify a permanent which
        // is no longer a creature. The designation remains if its types change.
        if !self.can_become_suspected(object) {
            return false;
        }
        self.record_suspect_keywords(object);
        let objects = self
            .state
            .suspected
            .get_or_insert_with(|| Box::new(Vec::new()));
        match objects.binary_search_by_key(&object.card, |r| r.card) {
            Ok(index) => objects[index] = object,
            Err(index) => objects.insert(index, object),
        }
        true
    }
    pub(super) fn clear_suspected_reference(&mut self, object: ObjectRef) {
        self.forget_suspect_keywords(object);
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
    pub(super) fn set_suspected_subject(
        &mut self,
        item: &SpellView,
        subject: Subject,
        suspected: bool,
    ) {
        for id in self.keyword_subjects(item, subject) {
            self.set_suspected(self.object_ref(id), suspected);
        }
    }
    pub(super) fn can_transfer_suspect(&self, item: &SpellView, recipient: ObjectRef) -> bool {
        // The trigger has already identified these untargeted entrants (CR 608.2k).
        item.source
            .is_some_and(|source| self.suspected_reference(source.object))
            && item.event_object.as_ref().is_some_and(|event| {
                event.object == recipient
                    || event.semantic_objects.as_ref().is_some_and(|objects| {
                        objects.contains(&DamageSourceRef::Object(recipient))
                    })
            })
            && self.can_become_suspected(recipient)
    }
    pub(super) fn transfer_suspect(&mut self, source: ObjectRef, recipient: ObjectRef) {
        if self.suspected_reference(source) && self.set_suspected(recipient, true) {
            self.set_suspected(source, false);
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
                    && item.source.is_some_and(|source| {
                        self.creature_matches(filter, item.controller, source.object, r.card)
                    })
            })
            .collect();
        for object in objects {
            self.clear_suspected_reference(object);
        }
    }
    pub(super) fn return_granted_source_suspected(&mut self, item: &SpellView, tapped: bool) {
        if let Some(mut movement) = self.granted_source_entry(item) {
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
