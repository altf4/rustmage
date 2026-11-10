//! Public exact-object placement events for this turn; removals preserve history.
use super::*;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct CounterPlacementHistoryView {
    pub object: ObjectRef,
    pub placed_by: Player,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub plus_one: bool,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub lore: bool,
    /// The recipient was a creature when at least one counter was placed.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub on_creature: bool,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl std::hash::Hash for CounterPlacementHistoryView {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (
            self.object,
            self.placed_by.index(),
            self.plus_one,
            self.lore,
            self.on_creature,
        )
            .hash(state);
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
pub(super) fn record(
    rows: &mut Rows,
    object: ObjectRef,
    placed_by: Player,
    plus_one: bool,
    lore: bool,
    on_creature: bool,
) {
    if let Some(index) = rows.as_ref().and_then(|r| {
        r.iter()
            .position(|row| row.object == object && row.placed_by == placed_by)
    }) {
        let prior = rows.as_ref().unwrap()[index];
        if (plus_one && !prior.plus_one)
            || (lore && !prior.lore)
            || (on_creature && !prior.on_creature)
        {
            let row = &mut rows.as_mut().unwrap()[index];
            row.plus_one |= plus_one;
            row.lore |= lore;
            row.on_creature |= on_creature;
        }
        return;
    }
    rows.get_or_insert_default()
        .push(CounterPlacementHistoryView {
            object,
            placed_by,
            plus_one,
            lore,
            on_creature,
        });
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn placed_on_creature(rows: &Rows, player: Player) -> bool {
    rows.as_ref().is_some_and(|rows| {
        rows.iter()
            .any(|row| row.placed_by == player && row.on_creature)
    })
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn mark_creature_entry(rows: &mut Rows, object: ObjectRef) {
    if rows.as_ref().is_some_and(|rows| {
        rows.iter()
            .any(|row| row.object == object && !row.on_creature)
    }) {
        for row in rows
            .as_mut()
            .unwrap()
            .iter_mut()
            .filter(|row| row.object == object)
        {
            row.on_creature = true;
        }
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn any_placed(rows: &Rows, object: ObjectRef, player: Player) -> bool {
    rows.as_ref().is_some_and(|r| {
        r.iter()
            .any(|row| row.object == object && row.placed_by == player)
    })
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn any_player_placed(rows: &Rows, object: ObjectRef) -> bool {
    rows.as_ref()
        .is_some_and(|rows| rows.iter().any(|row| row.object == object))
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn plus_one_placed(rows: &Rows, object: ObjectRef, player: Player) -> bool {
    rows.as_ref().is_some_and(|r| {
        r.iter()
            .any(|row| row.object == object && row.placed_by == player && row.plus_one)
    })
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn lore_placed(rows: &Rows, object: ObjectRef, player: Player) -> bool {
    rows.as_ref().is_some_and(|r| {
        r.iter()
            .any(|row| row.object == object && row.placed_by == player && row.lore)
    })
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn record_counter_placement(
        &mut self,
        object: ObjectRef,
        placed_by: Player,
        plus_one: bool,
        lore: bool,
    ) {
        let on_creature = self.is_creature(object.card);
        record(
            &mut self.state.counter_placement_history,
            object,
            placed_by,
            plus_one,
            lore,
            on_creature,
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
