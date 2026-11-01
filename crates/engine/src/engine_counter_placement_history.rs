//! Public exact-object placement events for this turn; removals preserve history.
use super::*;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct CounterPlacementHistoryView {
    pub object: ObjectRef,
    pub placed_by: Player,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub plus_one: bool,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl std::hash::Hash for CounterPlacementHistoryView {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (self.object, self.placed_by.index(), self.plus_one).hash(state);
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) type Rows = Option<SharedVec<CounterPlacementHistoryView>>;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn owned_bytes(rows: &Rows) -> usize {
    rows.as_ref().map_or(0, |r| {
        std::mem::size_of::<Vec<CounterPlacementHistoryView>>()
            + r.len() * std::mem::size_of::<CounterPlacementHistoryView>()
    })
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn record(rows: &mut Rows, object: ObjectRef, placed_by: Player, plus_one: bool) {
    if let Some(index) = rows.as_ref().and_then(|r| {
        r.iter()
            .position(|row| row.object == object && row.placed_by == placed_by)
    }) {
        if plus_one && !rows.as_ref().unwrap()[index].plus_one {
            rows.as_mut().unwrap()[index].plus_one = true;
        }
        return;
    }
    rows.get_or_insert_default()
        .push(CounterPlacementHistoryView {
            object,
            placed_by,
            plus_one,
        });
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn any_placed(rows: &Rows, object: ObjectRef, player: Player) -> bool {
    rows.as_ref().is_some_and(|r| {
        r.iter()
            .any(|row| row.object == object && row.placed_by == player)
    })
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn plus_one_placed(rows: &Rows, object: ObjectRef, player: Player) -> bool {
    rows.as_ref().is_some_and(|r| {
        r.iter()
            .any(|row| row.object == object && row.placed_by == player && row.plus_one)
    })
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn record_counter_placement(
        &mut self,
        object: ObjectRef,
        placed_by: Player,
        plus_one: bool,
    ) {
        record(
            &mut self.state.counter_placement_history,
            object,
            placed_by,
            plus_one,
        );
    }
    pub(super) fn put_counter_on_source_this_turn(
        &self,
        object: ObjectRef,
        player: Player,
    ) -> bool {
        any_placed(&self.state.counter_placement_history, object, player)
    }
}
