//! Opt-in entrant copy facts and post-entry continuous effects on exact results.
use super::*;
use crate::cards::AnimationSpec;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn bind_entering_copy(&mut self, entering: &ObjectView, program: &[Instruction]) {
        if program.iter().any(|i| {
            matches!(
                i,
                Instruction::CopyEnteringPermanentThenAnimate(_)
                    | Instruction::CopyEnteringWithEquipDiscount(_)
            )
        }) {
            let recipe = self.copiable_recipe(entering.reference);
            self.state
                .pending_triggers
                .last_mut()
                .unwrap()
                .set_event_object(EventObjectView {
                    object: entering.reference,
                    facts: EventObjectFacts::Copiable(recipe),
                    semantic_objects: None,
                    conniver_creature: None,
                    subtype_lki: None,
                });
        }
    }
    fn entering_copy_recipe(&self, item: &SpellView) -> CopyRecipe {
        let event = item.event_object.as_ref().expect("copiable entrant facts");
        let EventObjectFacts::Copiable(saved) = event.facts else {
            unreachable!()
        };
        if self.on_battlefield(event.object.card)
            && self.object_ref(event.object.card) == event.object
        {
            self.copiable_recipe(event.object)
        } else {
            saved
        }
    }
    pub(super) fn copy_entering_then_animate(&mut self, item: &SpellView, spec: AnimationSpec) {
        self.stage_copy_creation_inner(
            item.controller,
            self.entering_copy_recipe(item),
            1,
            replacements::TokenCompletion::AnimateNoncreatures(spec),
        );
    }
    pub(super) fn copy_entering_with_equip_discount(&mut self, item: &SpellView, generic: u32) {
        let recipe = self
            .entering_copy_recipe(item)
            .with_exceptions(CopyExceptions {
                equip_generic_reduction: generic,
                ..CopyExceptions::NONE
            });
        self.stage_copy_creation_inner(
            item.controller,
            recipe,
            1,
            replacements::TokenCompletion::TemporaryTokens(Box::new(
                temporary_tokens::TokenInstructions {
                    item: item.clone(),
                    keywords: Keywords::NONE,
                    attacking: false,
                    cleanup: temporary_tokens::TokenCleanup::NextUpkeepSacrifice,
                },
            )),
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
