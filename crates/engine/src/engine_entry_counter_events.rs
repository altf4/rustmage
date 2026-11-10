//! Counter events qualify recipients, independently of the player placing them.
use super::*;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn return_targets_with_counter_events(
        &mut self,
        item: &SpellView,
        amounts: &'static [(CounterKind, u64)],
        origin: Zone,
        tapped: bool,
        after_entry: bool,
    ) {
        let mut moves = Vec::new();
        for bound in &item.targets {
            if !self.same_target_object(*bound) {
                continue;
            }
            let Target::Card(id) = bound.target else {
                continue;
            };
            if self.state.objects[id as usize].zone != origin
                || moves
                    .iter()
                    .any(|m: &entries::EntryMove| m.before.reference == bound.object.unwrap())
            {
                continue;
            }
            let mut movement = self.entry_move(id, Zone::Battlefield, tapped);
            movement.controller = item.controller;
            if after_entry {
                movement.post_entry_counters = Some((item.controller, amounts));
            } else {
                movement.effect_counters = amounts;
            }
            moves.push(movement);
        }
        self.stage_entries(moves, None);
    }

    /// An event is first for an exact recipient, regardless of kind, controller,
    /// creature status, placing player, or whether an observer existed earlier.
    /// Entry commits already recorded their fresh incarnation before ETB facts.
    pub(super) fn controlled_creature_counter_events<'a>(
        &mut self,
        placements: impl Iterator<Item = (ObjectRef, &'a Counters, Player)> + Clone,
        fresh_entry: bool,
    ) {
        if !self.state.counter_sources.iter().any(|&id| {
            self.state.objects[id as usize]
                .card
                .triggers()
                .iter()
                .any(|ability| {
                    matches!(
                        ability.condition.event(),
                        TriggerCondition::ControlledCreatureCountersPlaced { .. }
                    )
                })
        }) {
            return;
        }
        let mut recipients = Vec::new();
        for (object, amounts, _) in placements.clone() {
            if amounts.has_any()
                && self.on_battlefield(object.card)
                && self.object_ref(object.card) == object
                && !recipients.iter().any(|(prior, _)| *prior == object)
            {
                let first = fresh_entry
                    || !counter_placement_history::any_player_placed(
                        &self.state.counter_placement_history,
                        object,
                    );
                recipients.push((object, first));
            }
        }
        // Commit all history before any observer can queue a recursive event.
        for (object, amounts, player) in placements {
            if amounts.has_any()
                && self.on_battlefield(object.card)
                && self.object_ref(object.card) == object
            {
                self.record_counter_placement(
                    object,
                    player,
                    amounts.plus_one > 0,
                    amounts.lore > 0,
                );
            }
        }
        let sources: Vec<_> = self
            .state
            .counter_sources
            .iter()
            .copied()
            .map(|id| self.object_view(id))
            .collect();
        for (object, first) in recipients {
            let recipient = self.object_view(object.card);
            if !recipient.is_creature() {
                continue;
            }
            for source in &sources {
                if source.controller != recipient.controller {
                    continue;
                }
                for (index, ability) in source.card.triggers().iter().enumerate() {
                    if let TriggerCondition::ControlledCreatureCountersPlaced { first_only } =
                        ability.condition.event()
                        && (!first_only || first)
                        && self.queue_trigger(source, index)
                    {
                        // This is an untargeted event-bound exact recipient. The
                        // historical first qualification is never rechecked.
                        self.state
                            .pending_triggers
                            .last_mut()
                            .unwrap()
                            .entering_object = Some(object);
                    }
                }
            }
        }
    }
}
