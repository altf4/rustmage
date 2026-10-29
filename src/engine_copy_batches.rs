//! Freeze every original before one simultaneous token-creation action (CR608.2f).
use super::*;
impl Game {
    /// Freeze all originals before one simultaneous token creation event.
    pub(super) fn copy_target_players_creatures_with_conditional_sacrifice(&mut self, item: &SpellView) {
        let Some(player) = item.targets.iter().find_map(|bound| match bound.target {
            Target::Player(player) => Some(player), _ => None,
        }) else { return };
        let recipes = self.state.objects.iter().enumerate().filter_map(|(id, object)| {
            let id = id as CardId;
            if object.zone != Zone::Battlefield || object.controller != player || !self.is_creature(id) {
                return None;
            }
            Some(self.copiable_recipe(self.object_ref(id)).with_exceptions(CopyExceptions {
                conditional_sacrifice_abilities: 1, ..CopyExceptions::NONE
            }))
        }).collect();
        self.stage_copy_recipes(item.controller, recipes);
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
        self.stage_copy_recipes(player, recipes);
    }
}
