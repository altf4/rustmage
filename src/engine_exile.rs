//! Face-up exile and the paired one-shot returns of CR610.3.
use super::*;
impl Game {
    pub(super) fn exile_until_source_leaves(&mut self, item: &SpellView) {
        if !self.on_battlefield(item.object.card)
            || self.object_ref(item.object.card) != item.object
        {
            return;
        }
        let ids: Vec<_> = item
            .targets
            .iter()
            .copied()
            .filter(|&bound| self.same_target_object(bound))
            .filter_map(|bound| {
                if let Target::Card(id) = bound.target {
                    Some(id)
                } else {
                    None
                }
            })
            .collect();
        let events = self.move_objects_raw(&ids, Zone::Exile);
        for event in &events {
            if event.moved {
                self.state.exile_links.push(ExileLink {
                    source: item.object,
                    exiled: event.after,
                    return_zone: event.before.zone,
                });
            }
        }
        self.finish_zone_events(&events, &[]);
    }

    pub(super) fn finish_zone_events(
        &mut self,
        events: &[ZoneChange],
        death_sources: &[ObjectView],
    ) {
        // No zone changes occur while observers are queued. Freeze their
        // post-event views once instead of rescanning the arena per newcomer.
        let sources: Vec<_> = if events
            .iter()
            .any(|e| e.moved && e.destination == Zone::Battlefield)
        {
            self.state
                .objects
                .iter()
                .enumerate()
                .filter(|(_, o)| o.zone == Zone::Battlefield && !o.card.triggers().is_empty())
                .map(|(i, _)| self.object_view(i as CardId))
                .collect()
        } else {
            Vec::new()
        };
        for event in events {
            self.zone_triggers(event, death_sources, &sources);
        }
        if self.state.exile_links.is_empty()
            || !events
                .iter()
                .any(|event| event.moved && event.before.zone == Zone::Battlefield)
        {
            return;
        }
        let mut returning = Vec::new();
        let mut index = 0;
        while index < self.state.exile_links.len() {
            let link = self.state.exile_links[index];
            let valid = self.object_ref(link.exiled.card) == link.exiled
                && self.state.objects[link.exiled.card as usize].zone == Zone::Exile;
            if valid
                && self.on_battlefield(link.source.card)
                && self.object_ref(link.source.card) == link.source
            {
                index += 1;
            } else {
                self.state.exile_links.swap_remove(index);
                if valid
                    && !self.state.objects[link.exiled.card as usize]
                        .card
                        .is_token()
                {
                    returning.push((link.exiled.card, link.return_zone));
                }
            }
        }
        // All effects created by the same departure batch return simultaneously.
        // They use no stack and offer no intervening choice or priority.
        let moves = returning
            .into_iter()
            .map(|(id, destination)| {
                self.entry_move(
                    id,
                    destination,
                    destination == Zone::Battlefield && self.enters_tapped(id),
                )
            })
            .collect();
        self.stage_entries(moves, None);
    }
}
