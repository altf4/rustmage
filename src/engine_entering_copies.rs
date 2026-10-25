//! Opt-in entrant copy facts and post-entry continuous effects on exact results.
use super::*;
use crate::cards::AnimationSpec;
impl Game {
    pub(super) fn bind_entering_copy(&mut self, entering: &ObjectView, program: &[Instruction]) {
        if program
            .iter()
            .any(|i| matches!(i, Instruction::CopyEnteringPermanentThenAnimate(_)))
        {
            self.state.pending_triggers.last_mut().unwrap().event_object =
                Some(Box::new(EventObjectView {
                    object: entering.reference,
                    facts: EventObjectFacts::Copiable(self.copiable_recipe(entering.reference)),
                }));
        }
    }
    pub(super) fn copy_entering_then_animate(&mut self, item: &SpellView, spec: AnimationSpec) {
        let event = item.event_object.as_ref().expect("copiable entrant facts");
        let EventObjectFacts::Copiable(saved) = event.facts else {
            unreachable!()
        };
        let recipe = if self.on_battlefield(event.object.card)
            && self.object_ref(event.object.card) == event.object
        {
            self.copiable_recipe(event.object)
        } else {
            saved
        };
        self.stage_copy_creation_inner(
            item.controller,
            recipe,
            1,
            replacements::TokenCompletion::AnimateNoncreatures(spec),
        );
    }
    pub(super) fn animate_created_noncreatures(
        &mut self,
        created: &[ObjectRef],
        spec: AnimationSpec,
    ) {
        for &object in created {
            if self.on_battlefield(object.card)
                && self.object_ref(object.card) == object
                && !self.is_creature(object.card)
            {
                self.change_object_form(
                    object,
                    spec,
                    characteristic_layers::AnimationTypeMode::AddCreature,
                );
            }
        }
    }
}
