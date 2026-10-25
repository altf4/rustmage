//! Freeze every original before one simultaneous token-creation action (CR608.2f).
use super::*;
impl Game {
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
