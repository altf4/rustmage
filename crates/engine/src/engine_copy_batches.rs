//! Freeze every original before one simultaneous token-creation action (CR608.2f).
use super::*;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn exile_graveyard_targets_copy_until_own_end_step(&mut self, item: &SpellView) {
        let targets: Vec<_> = item
            .targets
            .iter()
            .filter_map(|bound| {
                let Target::Card(id) = bound.target else {
                    return None;
                };
                (self.same_target_object(*bound)
                    && self.state.objects[id as usize].zone == Zone::Graveyard)
                    .then_some(id)
            })
            .collect();
        if targets.is_empty() {
            return;
        }
        let events = self.move_objects_raw(&targets, Zone::Exile);
        let recipes = events
            .iter()
            .filter(|event| event.moved && event.was_card && event.destination == Zone::Exile)
            .flat_map(ZoneChange::objects_after)
            .filter(|&object| {
                let exiled = &self.state.objects[object.card as usize];
                exiled.zone == Zone::Exile
                    && self.object_ref(object.card) == object
                    && exiled.is_card()
                    && exiled.card.is_creature()
                    && !self.exiled_face_down(object)
            })
            .map(|object| {
                CopyRecipe::new(self.state.objects[object.card as usize].card.front_face())
            })
            .collect();
        self.finish_zone_events(&events, &[]);
        self.stage_copy_recipes_until_own_end_step(item, recipes);
    }

    pub(super) fn copy_each_target(&mut self, item: &SpellView, amount: u128) {
        let recipes = item
            .targets
            .iter()
            .filter_map(|bound| {
                let Target::Card(id) = bound.target else {
                    return None;
                };
                let object = bound.object?;
                (self.on_battlefield(id) && self.object_ref(id) == object)
                    .then(|| self.copiable_recipe(object))
            })
            .collect();
        self.stage_copy_recipes(item.controller, recipes, amount, false);
    }

    pub(super) fn copy_other_controlled_nonlands(&mut self, item: &SpellView) {
        let source = item.source.map(|source| source.object);
        let recipes = self
            .state
            .objects
            .iter()
            .enumerate()
            .filter_map(|(id, object)| {
                let id = id as CardId;
                let reference = self.object_ref(id);
                (object.zone == Zone::Battlefield
                    && object.controller == item.controller
                    && Some(reference) != source
                    && self.current_kinds(id) & 2 == 0)
                    .then(|| self.copiable_recipe(reference))
            })
            .collect();
        self.stage_copy_recipes(item.controller, recipes, 1, false);
    }
    /// Freeze all originals before one simultaneous token creation event.
    pub(super) fn copy_target_players_creatures_with_conditional_sacrifice(
        &mut self,
        item: &SpellView,
    ) {
        let Some(player) = item.targets.iter().find_map(|bound| match bound.target {
            Target::Player(player) => Some(player),
            _ => None,
        }) else {
            return;
        };
        let recipes =
            self.state
                .objects
                .iter()
                .enumerate()
                .filter_map(|(id, object)| {
                    let id = id as CardId;
                    if object.zone != Zone::Battlefield
                        || object.controller != player
                        || !self.is_creature(id)
                    {
                        return None;
                    }
                    Some(self.copiable_recipe(self.object_ref(id)).with_exceptions(
                        CopyExceptions {
                            conditional_sacrifice_abilities: 1,
                            ..CopyExceptions::NONE
                        },
                    ))
                })
                .collect();
        self.stage_copy_recipes(item.controller, recipes, 1, false);
    }
    pub(super) fn copy_controlled_nontoken_creatures_nonlegendary(&mut self, player: Player) {
        let recipes = self
            .state
            .objects
            .iter()
            .enumerate()
            .filter_map(|(id, object)| {
                let id = id as CardId;
                if object.zone != Zone::Battlefield
                    || object.controller != player
                    || object.is_token()
                    || !self.is_creature(id)
                {
                    return None;
                }
                let mut recipe = self.copiable_recipe(self.object_ref(id));
                recipe.overrides = Some(CopyOverrides {
                    legendary: Some(false),
                    ..recipe.overrides.unwrap_or(CopyOverrides::NONE)
                });
                Some(recipe)
            })
            .collect();
        self.stage_copy_recipes(player, recipes, 1, false);
    }
}
