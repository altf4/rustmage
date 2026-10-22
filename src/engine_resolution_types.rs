//! Public type choices owned by the active resolution through entry continuations.
use super::*;
impl Game {
    pub(super) fn resolution_chosen_creature_type(&self) -> Option<crate::cards::CreatureType> {
        self.state
            .resolving
            .as_ref()
            .and_then(|resolution| resolution.chosen_creature_type)
    }
    pub(super) fn choose_resolution_creature_type(&mut self, kind: crate::cards::CreatureType) {
        self.state.resolving.as_mut().unwrap().chosen_creature_type = Some(kind);
        self.run_program();
    }
    pub(super) fn return_resolution_chosen_type(&mut self, controller: Player) {
        let kind = self
            .resolution_chosen_creature_type()
            .expect("preceding creature type choice");
        let moves = self
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(id, object)| {
                object.zone == Zone::Graveyard
                    && object.owner == controller
                    && !object.is_token()
                    && object.card.kind_mask() & 1 != 0
                    && self.has_subtype(*id as CardId, kind.name())
            })
            .map(|(id, _)| {
                let mut movement = self.entry_move(id as CardId, Zone::Battlefield, false);
                movement.controller = controller;
                movement
            })
            .collect();
        self.stage_entries(moves, None);
    }
}
