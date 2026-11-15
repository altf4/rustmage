//! CR 701.72: one owned keyword action for battlefield and card-zone objects.
use super::characteristic_layers::AnimationTypeMode;
use super::linked_exile_effects::LinkedExileMode;
use super::*;
use crate::cards::{AnimationCreatureTypes, AnimationDuration, AnimationSpec};

const BORG_FORM: AnimationSpec = AnimationSpec {
    artifact: true,
    creature_types: AnimationCreatureTypes::Named(&["Borg"]),
    base: None,
    keywords: Keywords::NONE,
    duration: AnimationDuration::Permanent,
};

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    /// Apply the action to one exact subject. Battlefield counters are an event
    /// between control and layer-four changes, including replacement choices.
    pub(super) fn assimilate(&mut self, item: &SpellView, subject: Subject) {
        let Some(id) = self.keyword_subjects(item, subject).into_iter().next() else {
            return;
        };
        let object = self.object_ref(id);
        let state = &self.state.objects[id as usize];
        if state.zone == Zone::Battlefield && self.is_creature(id) {
            self.record_fixed_control_batch([(object, item.controller)]);
            self.state
                .resolving
                .as_mut()
                .unwrap()
                .pending_assimilated_form = Some(object);
            self.effect_counters(
                vec![(object, Counters::of(CounterKind::PlusOne, 1))],
                replacements::ResumeMode::Program,
            );
        } else if matches!(state.zone, Zone::Graveyard | Zone::Exile)
            && state.is_card()
            && self.current_kinds(id) & 1 != 0
            && (state.zone != Zone::Exile || !self.exiled_face_down(object))
        {
            self.stage_assimilated_entry(object, item.controller);
        }
    }

    pub(super) fn stage_assimilated_entry(&mut self, object: ObjectRef, controller: Player) {
        if self.object_ref(object.card) != object {
            return;
        }
        let state = &self.state.objects[object.card as usize];
        if !matches!(state.zone, Zone::Graveyard | Zone::Exile)
            || !state.is_card()
            || self.current_kinds(object.card) & 1 == 0
            || (state.zone == Zone::Exile && self.exiled_face_down(object))
        {
            return;
        }
        let mut movement = self.entry_move(object.card, Zone::Battlefield, false);
        movement.controller = controller;
        movement.effect_counter = Some((CounterKind::PlusOne, 1));
        movement.animation = Some(&BORG_FORM);
        self.stage_entries(vec![movement], None);
    }

    pub(super) fn finish_pending_assimilated_form(&mut self) {
        let Some(object) = self
            .state
            .resolving
            .as_mut()
            .and_then(|resolution| resolution.pending_assimilated_form.take())
        else {
            return;
        };
        if self.on_battlefield(object.card) && self.object_ref(object.card) == object {
            self.change_object_form(object, BORG_FORM, AnimationTypeMode::PreserveCardTypes);
        }
    }

    /// The fallback target was declared with the trigger. If all chosen
    /// targets are illegal, ordinary resolution legality prevents this call.
    pub(super) fn assimilate_linked_or_exile_target(&mut self, item: &SpellView) -> bool {
        if self.start_linked_exile_effect(item, LinkedExileMode::AssimilateCreature) {
            return true;
        }
        self.exile_targets_linked(item, false);
        false
    }
}
