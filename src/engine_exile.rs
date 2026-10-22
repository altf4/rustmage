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
    /// Expires immediately before this player's next actual end step begins.
    pub next_end_step: Option<Player>,
    pub cast_only: bool,
    pub without_mana: bool,
    pub source: Option<ObjectRef>,
    pub any_mana: bool,
}
impl Game {
    pub(super) fn exile_top_until_next_end(&mut self, player: Player, amount: u128) {
        // Bound by the actual owned library, never narrow a potentially huge counter total.
        let ids: Vec<_> = self.state.players[player.index()]
            .library
            .iter()
            .rev()
            .copied()
            .take(usize::try_from(amount).unwrap_or(usize::MAX))
            .collect();
        let events = self.move_objects_raw(&ids, Zone::Exile);
        for event in &events {
            if event.moved && event.destination == Zone::Exile {
                let row = ExilePlayPermission {
                    object: event.after,
                    player,
                    expires_turn: u32::MAX,
                    next_end_step: Some(player),
                    cast_only: false,
                    without_mana: false,

                    source: None,

                    any_mana: false,
                };
                let at = self
                    .state
                    .exile_permissions
                    .partition_point(|p| p.object.card < row.object.card);
                self.state.exile_permissions.insert(at, row);
            }
        }
        self.finish_zone_events(&events, &[]);
    }
    pub(super) fn end_source_exile_permissions(&mut self, source: ObjectRef) {
        self.state
            .exile_permissions
            .retain(|permission| permission.source != Some(source));
    }
    pub(super) fn exile_cast_spends_any_mana(&self, player: Player, card: CardId) -> bool {
        self.state.objects[card as usize].zone == Zone::Exile
            && self.state.exile_permissions.iter().any(|permission| {
                permission.player == player
                    && permission.object == self.object_ref(card)
                    && permission.any_mana
                    && permission.expires_turn >= self.state.turn
            })
    }
    pub(super) fn grant_source_bound_exile_cast(&mut self, item: &SpellView, object: ObjectRef) {
        if !self.on_battlefield(item.object.card)
            || self.object_ref(item.object.card) != item.object
            || self.state.objects[item.object.card as usize].controller != item.controller
            || self.object_ref(object.card) != object
            || self.state.objects[object.card as usize].zone != Zone::Exile
            || !crate::cards::CastFilter::InstantSorcery
                .matches(self.state.objects[object.card as usize].card)
        {
            return;
        }
        let at = self
            .state
            .exile_permissions
            .partition_point(|permission| permission.object.card < object.card);
        self.state.exile_permissions.insert(
            at,
            ExilePlayPermission {
                object,
                player: item.controller,
                expires_turn: u32::MAX,
                cast_only: true,
                without_mana: false,
                source: Some(item.object),
                any_mana: true,

                next_end_step: None,
            },
        );
    }

    pub(super) fn exile_until_mana_value_free_cast(
        &mut self,
        owner: Player,
        player: Player,
        minimum: u16,
    ) {
        let cards: Vec<_> = self.state.players[owner.index()]
            .library
            .iter()
            .rev()
            .copied()
            .collect();
        let mut value = 0u128;
        for id in cards {
            if value >= u128::from(minimum) {
                break;
            }
            if self.state.objects[id as usize].zone != Zone::Library {
                continue;
            }
            let event = self.move_object(id, Zone::Exile);
            if event.moved && event.destination == Zone::Exile {
                value = value
                    .checked_add(u128::from(event.before.copy_recipe.as_ref().map_or_else(
                        || event.before.card.fixed_mana_value(),
                        |recipe| recipe.mana_value(),
                    )))
                    .capacity("exiled mana value exhausted");
                self.grant_exile_free_cast_permission(player, event.after);
            }
        }
    }

    pub(super) fn can_play_exiled(&self, player: Player, card: CardId) -> bool {
        if self.can_play_foreign_stash(player, card)
            || self.can_play_adventure(player, card)
            || self.can_cast_warped(player, card)
        {
            return true;
        }
        self.state.exile_permissions.iter().any(|p| {
            !p.without_mana
                && !p.cast_only
                && p.player == player
                && p.object == self.object_ref(card)
                && p.expires_turn >= self.state.turn
        })
    }
    pub(super) fn can_cast_exiled(&self, player: Player, card: CardId) -> bool {
        self.can_play_exiled(player, card)
            || self.state.exile_permissions.iter().any(|p| {
                !p.without_mana
                    && p.cast_only
                    && p.player == player
                    && p.object == self.object_ref(card)
                    && p.expires_turn >= self.state.turn
            })
    }
    pub(super) fn can_cast_exiled_free(&self, player: Player, card: CardId) -> bool {
        self.state.objects[card as usize].zone == Zone::Exile
            && self.state.exile_permissions.iter().any(|p| {
                p.without_mana
                    && p.player == player
                    && p.object == self.object_ref(card)
                    && p.expires_turn >= self.state.turn
            })
    }
    pub(super) fn grant_exile_free_cast_permission(&mut self, player: Player, object: ObjectRef) {
        self.grant_exile_permission(player, object, false, true, true);
    }
    pub(super) fn grant_exile_cast_permission(&mut self, player: Player, object: ObjectRef) {
        self.grant_exile_permission(player, object, false, true, false);
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
        let events = self.move_objects_raw(&ids, Zone::Exile);
        for event in &events {
            if event.moved && event.destination == Zone::Exile {
                self.grant_exile_play_permission(player, event.after, next_turn);
            }
        }
        self.finish_zone_events(&events, &[]);
    }

    pub(super) fn grant_exile_play_permission(
        &mut self,
        player: Player,
        object: ObjectRef,
        next_turn: bool,
    ) {
        self.grant_exile_permission(player, object, next_turn, false, false);
    }
    fn grant_exile_permission(
        &mut self,
        player: Player,
        object: ObjectRef,
        next_turn: bool,
        cast_only: bool,
        without_mana: bool,
    ) {
        if self.object_ref(object.card) != object
            || self.state.objects[object.card as usize].zone != Zone::Exile
        {
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
        let at = self
            .state
            .exile_permissions
            .partition_point(|p| p.object.card < object.card);
        self.state.exile_permissions.insert(
            at,
            ExilePlayPermission {
                object,
                player,
                expires_turn,
                next_end_step: None,
                cast_only,
                without_mana,
                source: None,
                any_mana: false,
            },
        );
    }

    pub(super) fn exile_top_choose_play(
        &mut self,
        player: Player,
        amount: u16,
        next_turn: bool,
    ) -> bool {
        let ids: Vec<_> = self.state.players[player.index()]
            .library
            .iter()
            .rev()
            .take(usize::from(amount))
            .copied()
            .collect();
        let events = self.move_objects_raw(&ids, Zone::Exile);
        self.state.resolving.as_mut().unwrap().milled = events
            .iter()
            .filter(|e| e.moved && e.destination == Zone::Exile)
            .map(|e| e.after)
            .collect();
        self.finish_zone_events(&events, &[]);
        self.start_milled_selection(MilledSelectionSpec {
            filter: MilledFilter::Any,
            count: MilledCount::Range {
                minimum: 1,
                maximum: 1,
            },
            destination: MilledDestination::PlayPermission { next_turn },
        })
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

    fn is_permanent_card_graveyard_entry(&self, event: &ZoneChange) -> bool {
        if !event.moved || event.destination != Zone::Graveyard || event.before.is_token() {
            return false;
        }
        let object = &self.state.objects[event.after.card as usize];
        object.zone == Zone::Graveyard
            && self.object_ref(event.after.card) == event.after
            && !object.is_token()
            && !object.is_spell_copy()
            // Use the card's new graveyard definition, not a departing animated
            // or transformed permanent's last-known types (CR603.6c).
            && object.card.kind_mask() & (1 | 2 | 16 | 32 | 64 | 256) != 0
    }
    fn batch_graveyard_triggers(&mut self, events: &[ZoneChange]) {
        if !events
            .iter()
            .any(|event| self.is_permanent_card_graveyard_entry(event))
        {
            return;
        }
        // This sparse observer list contains only live post-event sources.
        // Anywhere-to-graveyard abilities never use leaving-battlefield lookback.
        for cursor in 0..self.state.death_sources.len() {
            let id = self.state.death_sources[cursor];
            if !self.on_battlefield(id) {
                continue;
            }
            if !self.state.objects[id as usize]
                .card
                .triggers()
                .iter()
                .any(|a| {
                    matches!(
                        a.condition.event(),
                        TriggerCondition::PermanentCardsToGraveyard { .. }
                    )
                })
            {
                continue;
            }
            let source = self.object_view(id);
            for (index, ability) in source.card.triggers().iter().enumerate() {
                let TriggerCondition::PermanentCardsToGraveyard { relation } =
                    ability.condition.event()
                else {
                    continue;
                };
                if events.iter().any(|event| {
                    self.is_permanent_card_graveyard_entry(event)
                        && match relation {
                            CastRelation::You => event.before.owner == source.controller,
                            CastRelation::Opponent => event.before.owner != source.controller,
                            CastRelation::Any => true,
                        }
                }) {
                    self.queue_trigger(&source, index);
                }
            }
        }
    }
    pub(super) fn finish_zone_events(
        &mut self,
        events: &[ZoneChange],
        death_sources: &[ObjectView],
    ) {
        self.record_creature_entry_history(events);
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
                .filter(|(_, o)| {
                    (o.zone == Zone::Battlefield && !o.card.triggers().is_empty())
                        || (o.zone == Zone::Graveyard
                            && o.card
                                .triggers()
                                .iter()
                                .any(|a| a.condition.origin() == Some(TargetZone::Graveyard)))
                })
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
                && !event.before.is_token()
                && !event.before.card.is_emblem()
                && self.object_ref(event.after.card) == event.after
                && self.state.objects[event.after.card as usize].zone == Zone::Hand
                && !self.state.known_hand.contains(&event.after)
            {
                self.state.known_hand.push(event.after);
            }
            self.zone_triggers(event, death_sources, &sources, entry_suppression);
            if event.moved && event.destination == Zone::Battlefield {
                // Entering with counters is a placement event on every entry path,
                // including raw returns and tokens, exactly once after the whole batch.
                // CR122.6a: entering with counters is attributed to the
                // entering permanent's controller, including controlled returns.
                let placed_by = self.state.objects[event.after.card as usize].controller;
                let counters = self.counter_amounts(event.after.card).to_owned();
                for kind in CounterKind::ALL {
                    self.self_counters_added(event.after, kind, counters.get(kind), placed_by);
                }
                self.lore_counters_added(event.after, 0);
            }
        }
        self.batch_graveyard_entry_triggers(events, &sources, entry_suppression);
        self.batch_graveyard_triggers(events);
        self.prune_linked_exiles();
        self.bind_first_public_source_moves(events);
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
                if valid && !self.state.objects[link.exiled.card as usize].is_token() {
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
