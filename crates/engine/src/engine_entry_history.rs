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
        for event in events {
            let object = event.after;
            if !event.moved
                || event.destination != Zone::Battlefield
                || !self.on_battlefield(object.card)
                || self.object_ref(object.card) != object
                || !self.is_creature(object.card)
            {
                continue;
            }
            let controller = self.state.objects[object.card as usize].controller;
            counter_placement_history::mark_creature_entry(
                &mut self.state.counter_placement_history,
                object,
            );
            self.state
                .creature_entry_history
                .get_or_insert_default()
                .push(CreatureEntryHistoryView { object, controller });
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
