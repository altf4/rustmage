//! Resolution-created, chosen-color hexproof is fixed independently of color changes.
use super::*;
impl Game {
    pub(super) fn grant_chosen_color_hexproof(&mut self, item: &SpellView) {
        if !self.on_battlefield(item.card) || self.object_ref(item.card) != item.object {
            return;
        }
        let color = self
            .resolution_chosen_color()
            .expect("chosen-color instruction precedes grant");
        let timestamp = self.next_role_timestamp();
        let turn = self.state.turn;
        self.state
            .roles
            .get_or_insert_with(Default::default)
            .keyword_effects
            .push(KeywordEffectView {
                recipient: item.object,
                origin: KeywordEffectOrigin::Resolution {
                    source: item.object,
                },
                keywords: Keywords::NONE,
                hexproof_colors: 1 << color.index(),
                rules: Keywords::NONE,
                timestamp,
                expires_turn: Some(turn),
                expires_before_turn: None,
            });
    }
}
