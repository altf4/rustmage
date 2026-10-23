//! Return-and-animate completion belongs only to an actually returned incarnation.
use super::*;
impl Game {
    pub(super) fn return_artifact_as_robot(&mut self, item: &SpellView) {
        let moves = item
            .targets
            .iter()
            .filter(|b| self.same_target_object(**b))
            .filter_map(|bound| {
                let Target::Card(id) = bound.target else {
                    return None;
                };
                if self.state.objects[id as usize].zone != Zone::Graveyard {
                    return None;
                }
                let mut movement = self.entry_move(id, Zone::Battlefield, false);
                movement.controller = item.controller;
                if self.current_kinds(id) & 1 == 0 {
                    movement.return_modification =
                        Some((crate::cards::ReturnKind::FlyingRobot, item.object));
                }
                Some(movement)
            })
            .collect();
        self.stage_entries(moves, None);
    }
}
