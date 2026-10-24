//! Public counter-placement events for this turn; removals do not erase history.
use super::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct CounterPlacementHistoryView {
    pub object: ObjectRef,
    pub placed_by: Player,
}
pub(super) type Rows = Option<SharedVec<CounterPlacementHistoryView>>;
pub(super) fn owned_bytes(rows: &Rows) -> usize {
    rows.as_ref().map_or(0, |r| {
        std::mem::size_of::<Vec<CounterPlacementHistoryView>>()
            + r.len() * std::mem::size_of::<CounterPlacementHistoryView>()
    })
}
impl Game {
    pub(super) fn record_counter_placement(&mut self, object: ObjectRef, placed_by: Player) {
        let rows = self
            .state
            .counter_placement_history
            .get_or_insert_with(SharedVec::default);
        let row = CounterPlacementHistoryView { object, placed_by };
        if !rows.contains(&row) {
            rows.push(row);
        }
    }
    pub(super) fn put_counter_on_source_this_turn(
        &self,
        object: ObjectRef,
        player: Player,
    ) -> bool {
        self.state
            .counter_placement_history
            .as_ref()
            .is_some_and(|rows| {
                rows.contains(&CounterPlacementHistoryView {
                    object,
                    placed_by: player,
                })
            })
    }
}
