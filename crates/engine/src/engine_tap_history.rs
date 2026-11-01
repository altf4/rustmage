//! Exact-object tap history persists through control changes and observer entry.
use super::*;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct TapHistory {
    object: ObjectRef,
    repeated: bool,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) type Rows = Option<SharedVec<TapHistory>>;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn owned_bytes(rows: &Rows) -> usize {
    rows.as_ref().map_or(0, |rows| {
        std::mem::size_of::<Vec<TapHistory>>() + rows.len() * std::mem::size_of::<TapHistory>()
    })
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn record_tap(&mut self, object: ObjectRef) {
        let rows = self
            .state
            .tap_history
            .get_or_insert_with(|| Vec::new().into());
        if let Some(index) = rows.iter().position(|row| row.object == object) {
            if !rows[index].repeated {
                rows[index].repeated = true;
            }
        } else {
            rows.push(TapHistory {
                object,
                repeated: false,
            });
        }
    }
    pub(super) fn first_tap(&self, object: ObjectRef) -> bool {
        self.state
            .tap_history
            .as_ref()
            .is_some_and(|rows| rows.iter().any(|row| row.object == object && !row.repeated))
    }
    /// The first-tap clause is checked again at resolution, for the event object,
    /// independently of the observer's current controller, abilities or location.
    pub(super) fn tap_event_guard(&self, item: &SpellView) -> bool {
        let StackKind::TriggeredAbility { ability } = item.kind else {
            return true;
        };
        item.trigger_definition().triggers()[usize::from(ability)]
            .condition
            .event()
            != TriggerCondition::ControlledCreatureFirstTapOwnTurn
            || item
                .entering_object
                .is_some_and(|object| self.first_tap(object))
    }
}
