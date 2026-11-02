//! Exact-incarnation exert costs and next-untap obligations (CR701.43).
use super::*;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct ExertionView {
    pub object: ObjectRef,
    /// The player who exerted it, independent of subsequent control changes.
    pub player: Player,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) type Rows = Option<SharedVec<ExertionView>>;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn owned_bytes(rows: &Rows) -> usize {
    rows.as_ref().map_or(0, |rows| {
        std::mem::size_of::<Vec<ExertionView>>() + rows.len() * std::mem::size_of::<ExertionView>()
    })
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn exert_source(&mut self, object: ObjectRef, player: Player) {
        assert!(
            self.on_battlefield(object.card) && self.object_ref(object.card) == object,
            "exert is a cost of the exact live permanent"
        );
        let rows = self
            .state
            .exertions
            .get_or_insert_with(|| Vec::new().into());
        // Repeated exertions for the same next step overlap, not stack across
        // future turns. Keep rows canonical for deterministic observations.
        if let Err(index) =
            rows.binary_search_by_key(&(object, player), |row| (row.object, row.player))
        {
            rows.try_reserve(1).capacity("exertion capacity exhausted");
            rows.insert(index, ExertionView { object, player });
        }
    }

    pub(super) fn forget_exertions(&mut self, object: ObjectRef) {
        if let Some(rows) = &mut self.state.exertions
            && rows.iter().any(|row| row.object == object)
        {
            rows.retain(|row| row.object != object);
            if rows.is_empty() {
                self.state.exertions = None;
            }
        }
    }

    pub(super) fn expire_exertions(&mut self, player: Player) {
        if let Some(rows) = &mut self.state.exertions
            && rows.iter().any(|row| row.player == player)
        {
            rows.retain(|row| row.player != player);
            if rows.is_empty() {
                self.state.exertions = None;
            }
        }
    }
}
