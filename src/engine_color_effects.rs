//! Exact-object color replacement effects, independent of the source after resolution.
use super::*;
use crate::cards::{AnimationDuration, ChoiceColor};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct ColorEffectView {
    pub object: ObjectRef,
    pub colors: u8,
    pub timestamp: u64,
    pub expires_turn: Option<u32>,
}
impl Game {
    pub(super) fn apply_mana_source_color(
        &mut self,
        source: ObjectRef,
        definition: ManaProduction,
        production: Mana,
    ) {
        if self.on_battlefield(source.card)
            && self.object_ref(source.card) == source
            && let Some(colors) = definition.resulting_source_color(production)
        {
            self.set_color_effects(&[source.card], colors, AnimationDuration::EndOfTurn);
        }
    }
    pub(super) fn resolution_chosen_color(&self) -> Option<ChoiceColor> {
        self.state
            .resolving
            .as_ref()
            .and_then(|resolution| resolution.chosen_color)
    }
    pub(super) fn choose_resolution_color(&mut self, color: ChoiceColor) {
        self.state.resolving.as_mut().unwrap().chosen_color = Some(color);
        self.run_program();
    }
    pub(super) fn set_color_effects(
        &mut self,
        recipients: &[CardId],
        colors: u8,
        duration: AnimationDuration,
    ) {
        assert_eq!(colors & !31, 0, "noncolor bit in color effect");
        let recipients: Vec<_> = recipients
            .iter()
            .copied()
            .filter(|&id| self.on_battlefield(id))
            .map(|id| self.object_ref(id))
            .collect();
        if recipients.is_empty() {
            return;
        }
        let timestamp = self.next_role_timestamp();
        let expires_turn = match duration {
            AnimationDuration::EndOfTurn => Some(self.state.turn),
            AnimationDuration::Permanent => None,
        };
        let rows = &mut self
            .state
            .roles
            .get_or_insert_with(Default::default)
            .color_effects;
        for object in recipients {
            rows.push(ColorEffectView {
                object,
                colors,
                timestamp,
                expires_turn,
            });
        }
    }
    pub(super) fn forget_color_effects(&mut self, object: ObjectRef) {
        if let Some(roles) = &mut self.state.roles {
            roles.color_effects.retain(|row| row.object != object);
        }
    }
    pub(super) fn expire_color_effects(&mut self) -> bool {
        let Some(roles) = &mut self.state.roles else {
            return false;
        };
        let before = roles.color_effects.len();
        roles.color_effects.retain(|row| row.expires_turn.is_none());
        roles.color_effects.len() != before
    }
}
