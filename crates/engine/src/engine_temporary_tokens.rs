//! Creation instructions are carried through replacement and actual entry results.
use super::*;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct TokenInstructions {
    pub item: SpellView,
    pub keywords: Keywords,
    pub attacking: bool,
    pub own_end_step: bool,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn create_linked_exile_attacking_copies(&mut self, item: &SpellView) {
        let Some(source) = item.source else {
            return;
        };
        // CR607.3: copied exile triggers can link multiple cards. Copy EACH,
        // rather than selecting one as the upstream Java implementation does.
        let recipes = self
            .state
            .exile_links
            .iter()
            .filter(|link| {
                link.source == source.object
                    && link.ability_origin == item.source_ability_origin
                    && self.object_ref(link.exiled.card) == link.exiled
                    && self.state.objects[link.exiled.card as usize].zone == Zone::Exile
                    && self.state.objects[link.exiled.card as usize].is_card()
            })
            .map(|link| CopyRecipe {
                definition: self.state.objects[link.exiled.card as usize]
                    .card
                    .front_face(),
                exceptions: CopyExceptions::NONE,
                overrides: None,
                faces: None,
                sampled_size: None,
            })
            .collect();
        self.stage_linked_attacking_recipes(item, recipes);
    }
}
