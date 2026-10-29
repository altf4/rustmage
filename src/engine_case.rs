//! Case designations bind to exact battlefield incarnations.
use super::*;

impl Game {
    pub(super) fn case_solved(&self, object: ObjectRef) -> bool {
        self.on_battlefield(object.card)
            && self.object_ref(object.card) == object
            && self
                .state
                .solved_cases
                .as_ref()
                .is_some_and(|v| v.contains(&object))
    }
    pub(super) fn solve_case(&mut self, object: ObjectRef) {
        if self.on_battlefield(object.card)
            && self.object_ref(object.card) == object
            && !self.case_solved(object)
        {
            self.state
                .solved_cases
                .get_or_insert_with(|| Box::new(Vec::new()))
                .push(object);
        }
    }
    pub(super) fn forget_solved_case(&mut self, object: ObjectRef) {
        if let Some(v) = &mut self.state.solved_cases {
            v.retain(|r| *r != object);
            if v.is_empty() {
                self.state.solved_cases = None;
            }
        }
    }
    pub(super) fn grant_creature_graveyard_cast(&mut self, player: Player) {
        let cards: Vec<_> = self
            .state
            .objects
            .iter()
            .enumerate()
            .filter_map(|(id, o)| {
                (o.zone == Zone::Graveyard
                    && o.owner == player
                    && o.is_card()
                    && o.card.kind_mask() & 1 != 0)
                    .then_some(self.object_ref(id as CardId))
            })
            .collect();
        for card in cards {
            self.grant_graveyard_permission(player, card, GraveyardCastKind::Cast);
        }
    }
}
