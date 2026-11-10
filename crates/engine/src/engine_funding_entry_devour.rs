//! Project the same owned Devour choices during nested mana payments.
use super::*;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl FundingState<'_> {
    pub(super) fn visit_entry_devour_choices(
        &self,
        moves: Vec<entries::EntryMove>,
        emit: &mut dyn FnMut(&Self, Vec<entries::EntryMove>) -> bool,
    ) -> bool {
        if self.entry_projection.is_none() {
            let mut world = self.clone();
            world.prepare_entry_projection();
            return world.visit_entry_devour_choices(moves, emit);
        }
        let Some(index) = entry_devour::selection_index(&moves, self.active) else {
            let selected: Vec<_> = entry_devour::selections(&moves)
                .map(|(_, object)| object)
                .collect();
            if selected.is_empty() {
                let mut moves = moves;
                entry_devour::record_results(&mut moves, |_| false);
                return emit(self, moves);
            }
            return self.cost_departures(&selected, Zone::Graveyard, &mut |world| {
                let mut moves = moves.clone();
                entry_devour::record_results(&mut moves, |object| {
                    world.reference(object.card) != object
                });
                moves.retain(|movement| {
                    world.reference(movement.before.id) == movement.before.reference
                });
                emit(world, moves)
            });
        };
        let mut candidates = Vec::new();
        if self
            .preview_entry_view(&moves[index].replacement_preview())
            .is_some_and(|view| view.printed_abilities_available())
        {
            candidates.extend(
                self.resources
                    .iter()
                    .filter(|r| entry_devour::is_candidate(self, &moves, index, r.id))
                    .map(|r| self.reference(r.id)),
            );
        }
        self.visit_entry_devour_subsets(moves, index, &candidates, emit)
    }

    fn visit_entry_devour_subsets(
        &self,
        moves: Vec<entries::EntryMove>,
        index: usize,
        candidates: &[ObjectRef],
        emit: &mut dyn FnMut(&Self, Vec<entries::EntryMove>) -> bool,
    ) -> bool {
        let mut finished = moves.clone();
        finished[index]
            .devour_preparation
            .get_or_insert_default()
            .closed = true;
        if self.visit_entry_devour_choices(finished, emit) {
            return true;
        }
        for (i, &object) in candidates.iter().enumerate() {
            let mut next = moves.clone();
            next[index]
                .devour_preparation
                .get_or_insert_default()
                .selected
                .push(object);
            if self.visit_entry_devour_subsets(next, index, &candidates[i + 1..], emit) {
                return true;
            }
        }
        false
    }
}
