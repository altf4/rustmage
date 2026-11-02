//! Exact immediate exile results in speculative as-entry choices.
use super::*;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl FundingState<'_> {
    pub(super) fn visit_entry_exile_choices(
        &self,
        moves: Vec<entries::EntryMove>,
        emit: &mut dyn FnMut(&Self, Vec<entries::EntryMove>) -> bool,
    ) -> bool {
        let Some(index) = moves
            .iter()
            .enumerate()
            .filter(|(_, m)| entry_exiles::needs_exile(m))
            .min_by_key(|(i, m)| (m.controller != self.active, *i))
            .map(|(i, _)| i)
        else {
            return emit(self, moves);
        };
        let m = &moves[index];
        let spec = m.before.card.entry_exile_counters().unwrap();
        let selected = m
            .exile_preparation
            .as_ref()
            .map_or_else(Vec::new, |p| p.selected.clone());
        let closed = m
            .exile_preparation
            .as_ref()
            .is_some_and(|p| p.selection_closed);
        let candidates = self
            .resources
            .iter()
            .filter(|r| {
                r.object.zone == Zone::Graveyard
                    && r.object.owner == m.controller
                    && r.object.is_card()
                    && entry_exiles::filter_matches(self, spec.filter, r.id)
                    && !selected.contains(&self.reference(r.id))
                    && !moves.iter().any(|other| {
                        !other.skipped
                            && other.destination == Zone::Battlefield
                            && other.before.reference == self.reference(r.id)
                    })
            })
            .map(|r| self.reference(r.id))
            .collect::<Vec<_>>();
        self.visit_entry_exile_subsets(moves, index, &candidates, 0, selected, closed, emit)
    }
    fn visit_entry_exile_subsets(
        &self,
        moves: Vec<entries::EntryMove>,
        index: usize,
        candidates: &[ObjectRef],
        at: usize,
        selected: Vec<ObjectRef>,
        closed: bool,
        emit: &mut dyn FnMut(&Self, Vec<entries::EntryMove>) -> bool,
    ) -> bool {
        let mut seed = self.clone();
        seed.prepare_entry_projection();
        if seed.cost_departures(&selected, Zone::Exile, &mut |world| {
            let exiled = selected
                .iter()
                .filter(|r| {
                    world.object(r.card).zone == Zone::Exile && world.reference(r.card) != **r
                })
                .map(|r| world.reference(r.card))
                .collect();
            let mut branch = moves.clone();
            branch[index].exile_preparation = Some(Box::new(entry_exiles::Preparation {
                done: true,
                selection_closed: true,
                selected: selected.clone(),
                exiled,
            }));
            world.visit_entry_exile_choices(branch, emit)
        }) {
            return true;
        }
        if closed || selected.len() >= moves[index].x as usize {
            return false;
        }
        for next in at..candidates.len() {
            let mut chosen = selected.clone();
            chosen.push(candidates[next]);
            if self.visit_entry_exile_subsets(
                moves.clone(),
                index,
                candidates,
                next + 1,
                chosen,
                false,
                emit,
            ) {
                return true;
            }
        }
        false
    }
}
