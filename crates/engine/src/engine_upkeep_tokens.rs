//! Actual hand-return events create one source-independent next-upkeep token trigger.
use super::*;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn return_for_upkeep_tokens(&mut self, item: &SpellView, token: Card) {
        let ids = item
            .targets
            .iter()
            .filter(|b| self.same_target_object(**b))
            .filter_map(|b| b.object)
            .filter(|r| self.on_battlefield(r.card))
            .map(|r| r.card)
            .collect::<Vec<_>>();
        // The raw batch freezes every pre-move object before any departure.
        let events = self.move_objects_raw(&ids, Zone::Hand);
        let amount = u64::try_from(
            events
                .iter()
                .filter(|e| {
                    e.moved
                        && e.destination == Zone::Hand
                        && e.before.owner == item.controller
                        && e.before.is_creature()
                })
                .count(),
        )
        .expect("arena count fits u64");
        let mut delayed = item.clone();
        delayed.id = self.next_stack_id();
        delayed.kind = StackKind::DelayedTokenCreation { token, amount };
        delayed.targets.clear();
        delayed.crime_qualifying_target = false;
        delayed.target_type_slots = None;
        delayed.mode = None;
        delayed.cast = None;
        delayed.is_copy = false;
        delayed.related_stack = None;
        delayed.additional_triggers = 0;
        self.state
            .delayed_upkeep_tokens
            .get_or_insert_default()
            .push(delayed);
        // Save the result before linked returns can suspend on an entry choice.
        // Token cease-to-exist SBAs occur later and do not undo a successful return.
        self.finish_zone_events(&events, &[]);
    }
}
