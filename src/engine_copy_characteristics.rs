//! Copiable token exceptions, applied before later characteristic layers.
use super::*;

impl Game {
    pub(super) fn copiable_recipe(&self, object: ObjectRef) -> CopyRecipe {
        self.copy_recipe(object)
            .unwrap_or_else(|| CopyRecipe::new(self.state.objects[object.card as usize].card))
    }
    pub(super) fn copied_keywords(&self, object: ObjectRef) -> Keywords {
        if self
            .copy_recipe(object)
            .is_some_and(|r| r.exceptions.haste_and_sacrifice)
        {
            Keywords::HASTE
        } else {
            Keywords::NONE
        }
    }
    pub(super) fn recipe_for_subject(
        &self,
        item: &SpellView,
        subject: Subject,
    ) -> Option<CopyRecipe> {
        match subject {
            Subject::Source => Some(
                if self.on_battlefield(item.card) && self.object_ref(item.card) == item.object {
                    self.copiable_recipe(item.object)
                } else {
                    item.source_copy_recipe
                        .as_deref()
                        .copied()
                        .unwrap_or_else(|| CopyRecipe::new(item.source_card))
                },
            ),
            Subject::Targets => item.targets.iter().find_map(|bound| {
                if self.same_target_object(*bound)
                    && let Target::Card(id) = bound.target
                {
                    Some(self.copiable_recipe(self.object_ref(id)))
                } else {
                    None
                }
            }),
            _ => unreachable!("copy subject"),
        }
    }
    pub(super) fn queue_copy_end_step_abilities(&mut self) {
        let sources: Vec<_> = self
            .state
            .copy_recipes
            .as_ref()
            .into_iter()
            .flat_map(|r| r.iter())
            .filter(|r| {
                r.recipe.exceptions.haste_and_sacrifice
                    && self.on_battlefield(r.object.card)
                    && self.object_ref(r.object.card) == r.object
                    && self.printed_abilities_available(r.object.card)
            })
            .map(|r| r.object.card)
            .collect();
        for source in sources {
            // Reuse the normal source-facts constructor; this is the token's
            // own intrinsic trigger, not a source-independent delayed effect.
            let mut item =
                self.activation_item(self.state.objects[source as usize].controller, source, 0);
            item.kind = StackKind::CopyEndStepSacrifice;
            self.state.pending_triggers.push(item);
        }
    }
    pub(super) fn sacrifice_exact_source(&mut self, item: &SpellView) {
        if self.on_battlefield(item.card)
            && self.object_ref(item.card) == item.object
            && self.state.objects[item.card as usize].controller == item.controller
        {
            self.sacrifice_batch(&[(item.controller, item.card)]);
        }
    }
}
