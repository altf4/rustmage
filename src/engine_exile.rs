//! Face-up exile and the paired one-shot returns of CR610.3.
use super::*;
/// Independent paid play permission for one exact face-up exile incarnation.
/// The deadline is inclusive through priority in that turn, and expires during
/// cleanup alongside other until-end-of-turn effects (CR514.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct ExilePlayPermission {
    pub object: ObjectRef,
    pub player: Player,
    pub expires_turn: u32,
}
impl Game {
    pub(super) fn can_play_exiled(&self, player: Player, card: CardId) -> bool {
        if self.can_play_adventure(player, card) || self.can_cast_warped(player, card) {
            return true;
        }
        self.state.exile_permissions.iter().any(|p| {
            p.player == player
                && p.object == self.object_ref(card)
                && p.expires_turn >= self.state.turn
        })
    }
    pub(super) fn exile_top_play(&mut self, player: Player, amount: u16, next_turn: bool) {
        self.exile_top_play_from(player, player, amount, next_turn);
    }

    pub(super) fn exile_top_play_from(
        &mut self,
        library_owner: Player,
        player: Player,
        amount: u16,
        next_turn: bool,
    ) {
        let ids: Vec<_> = self.state.players[library_owner.index()]
            .library
            .iter()
            .rev()
            .take(usize::from(amount))
            .copied()
            .collect();
        if ids.is_empty() {
            return;
        }
        let expires_turn = self
            .state
            .turn
            .checked_add(if next_turn {
                if self.state.active == player { 2 } else { 1 }
            } else {
                0
            })
            .capacity("exile permission deadline exhausted");
        let events = self.move_objects_raw(&ids, Zone::Exile);
        for event in &events {
            if event.moved && event.destination == Zone::Exile {
                let at = self
                    .state
                    .exile_permissions
                    .partition_point(|p| p.object.card < event.after.card);
                self.state.exile_permissions.insert(
                    at,
                    ExilePlayPermission {
                        object: event.after,
                        player,
                        expires_turn,
                    },
                );
            }
        }
        self.finish_zone_events(&events, &[]);
    }

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
        // Static sources are already registered for every member of the batch.
        // No moves occur during dispatch, so compute this union just once.
        let entry_suppression = if sources.is_empty() {
            0
        } else {
            self.state.static_sources.iter().fold(0, |mask, &id| {
                if self.printed_abilities_available(id) {
                    mask | self.state.objects[id as usize]
                        .card
                        .entry_trigger_suppression()
                } else {
                    mask
                }
            })
        };
        for event in events {
            // These zones are public in the supported rules subset (all exile
            // is face up). Preserve the exact new hand incarnation for both
            // observers; library draws and private searches reveal nothing.
            if event.moved
                && event.destination == Zone::Hand
                && matches!(
                    event.before.zone,
                    Zone::Battlefield | Zone::Graveyard | Zone::Stack | Zone::Exile
                )
                && !event.before.card.is_token()
                && !event.before.card.is_emblem()
                && self.object_ref(event.after.card) == event.after
                && self.state.objects[event.after.card as usize].zone == Zone::Hand
                && !self.state.known_hand.contains(&event.after)
            {
                self.state.known_hand.push(event.after);
            }
            self.zone_triggers(event, death_sources, &sources, entry_suppression);
            if event.moved && event.destination == Zone::Battlefield {
                self.lore_counters_added(event.after, 0);
            }
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
            .map(|(id, destination)| self.entry_move(id, destination, false))
            .collect();
        self.stage_entries(moves, None);
    }
}
