//! Public creature entry facts captured once after a simultaneous movement batch.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct CreatureEntryHistoryView {
    pub object: ObjectRef,
    pub controller: Player,
}
impl Game {
    pub(super) fn record_creature_entry_history(&mut self, events: &[ZoneChange]) {
        for event in events {
            if event.moved
                && event.destination == Zone::Battlefield
                && self.on_battlefield(event.after.card)
                && self.object_ref(event.after.card) == event.after
            {
                self.state
                    .permanent_entry_history
                    .get_or_insert_with(|| Vec::new().into())
                    .push(event.after);
            }
            if event.moved
                && event.destination == Zone::Battlefield
                && self.on_battlefield(event.after.card)
                && self.object_ref(event.after.card) == event.after
                && self.current_kinds(event.after.card) & 16 != 0
            {
                let controller = self.state.objects[event.after.card as usize].controller;
                self.state.players[controller.index()].artifact_entered_this_turn = true;
            }
        }
        // All newcomers and continuous effects exist before an entry is classified.
        // Retain entry-time controller/type even after departure or control changes.
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
                .get_or_insert_with(|| Vec::new().into())
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
