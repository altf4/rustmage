//! Public entry facts captured after all members of a simultaneous batch exist.
use super::*;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct CreatureEntryHistoryView {
    pub object: ObjectRef,
    pub controller: Player,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn record_creature_entry_history(&mut self, events: &[ZoneChange]) {
        let entries: Vec<_> = events
            .iter()
            .filter(|event| {
                event.moved
                    && event.destination == Zone::Battlefield
                    && self.on_battlefield(event.after.card)
                    && self.object_ref(event.after.card) == event.after
                    && self.is_creature(event.after.card)
            })
            .map(|event| CreatureEntryHistoryView {
                object: event.after,
                controller: self.state.objects[event.after.card as usize].controller,
            })
            .collect();
        if !entries.is_empty() {
            self.state
                .creature_entry_history
                .get_or_insert_default()
                .extend(entries);
        }
    }
    pub(super) fn creature_entries_this_turn(
        &self,
        player: Player,
        source: ObjectRef,
        other: bool,
    ) -> i128 {
        self.state
            .creature_entry_history
            .as_ref()
            .map_or(0, |entries| {
                entries
                    .iter()
                    .filter(|entry| {
                        entry.controller == player && (!other || entry.object != source)
                    })
                    .count() as i128
            })
    }
}
