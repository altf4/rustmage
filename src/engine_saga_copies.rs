//! Exact linked graveyard cards and immutable copiable exceptions.
use super::*;
impl Game {
    pub(super) fn linked_copy_candidate(&self, item: &SpellView, object: ObjectRef) -> bool {
        self.object_ref(object.card) == object
            && self.state.objects[object.card as usize].zone == Zone::Exile
            && self.state.linked_choices.as_ref().is_some_and(|data| {
                data.links.iter().any(|link| {
                    link.source == item.object
                        && link.ability_origin == item.ability_origin()
                        && link.exiled == object
                })
            })
    }
    pub(super) fn exile_graveyard_and_copy(
        &mut self,
        item: &SpellView,
        exceptions: CopyExceptions,
    ) {
        let Some(id) = item.targets.iter().find_map(|bound| match bound.target {
            Target::Card(id) if self.same_target_object(*bound) => Some(id),
            _ => None,
        }) else {
            return;
        };
        let mut recipe = CopyRecipe::new(self.state.objects[id as usize].card);
        recipe.exceptions = recipe.exceptions.union(exceptions);
        let event = self.move_object(id, Zone::Exile);
        if event.moved && event.destination == Zone::Exile {
            self.remember_linked_exile_for_origin(item.object, item.ability_origin(), &[id]);
        }
        self.stage_copy_creation(item.controller, recipe, 1);
    }
    pub(super) fn return_graveyard_name_tapped(&mut self, item: &SpellView) {
        let Some(id) = item.targets.iter().find_map(|bound| match bound.target {
            Target::Card(id) if self.same_target_object(*bound) => Some(id),
            _ => None,
        }) else {
            return;
        };
        let name = self.object_name(id);
        let ids: Vec<_> = self
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(id, object)| {
                object.zone == Zone::Graveyard
                    && object.owner == item.controller
                    && !object.is_token()
                    && crate::card_names::same_name(self.object_name(*id as CardId), name)
            })
            .map(|(id, _)| id as CardId)
            .collect();
        let moves = ids
            .into_iter()
            .map(|id| self.entry_move(id, Zone::Battlefield, true))
            .collect();
        self.stage_entries(moves, None);
    }
}
