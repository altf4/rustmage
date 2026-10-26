//! Face-up exile and the paired one-shot returns of CR610.3.
use super::*;
/// Independent paid play permission for one exact face-up exile incarnation.
/// The deadline is inclusive through priority in that turn, and expires during
/// cleanup alongside other until-end-of-turn effects (CR514.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct ExilePlayPermission {
    pub object: ObjectRef,
    pub player: Player,
    pub expiry: ExilePlayExpiry,
    pub spend_any_mana: bool,
    /// A cast permission never authorizes a land play.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub cast_only: bool,
    pub exile_on_graveyard: bool,
}
/// Deadline boundaries refer to actual turns, never to skipped turn candidates.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum ExilePlayExpiry {
    Turn(u32),
    EndOfNextTurn,
    StartOfNextTurn(Player),
}
impl ExilePlayExpiry {
    pub(super) fn active(self, turn: u32) -> bool {
        match self {
            Self::Turn(deadline) => deadline >= turn,
            _ => true,
        }
    }
}
/// Exact top-card selection, bounded only by the finite library's length.
/// Keep the full rules scalar until that bound makes conversion address-safe.
pub(super) fn top_library_cards(library: &[CardId], amount: u128) -> Vec<CardId> {
    let count = usize::try_from(amount.min(library.len() as u128)).unwrap();
    library.iter().rev().take(count).copied().collect()
}
impl Game {
    pub(super) fn can_play_exiled(&self, player: Player, card: CardId) -> bool {
        if self.can_cast_prepared(player, card)
            || self.can_play_foreign_stash(player, card)
            || self.can_play_adventure(player, card)
            || self.can_cast_warped(player, card)
        {
            return true;
        }
        self.state.exile_permissions.iter().any(|p| {
            p.player == player
                && p.object == self.object_ref(card)
                && p.expiry.active(self.state.turn)
        })
    }
    pub(super) fn can_play_exiled_land(&self, player: Player, card: CardId) -> bool {
        self.can_play_foreign_stash(player, card)
            || self.can_play_adventure(player, card)
            || self.state.exile_permissions.iter().any(|p| {
                p.player == player
                    && p.object == self.object_ref(card)
                    && p.expiry.active(self.state.turn)
                    && !p.cast_only
            })
    }
    pub(super) fn exile_cast_modifiers(&self, player: Player, card: CardId) -> (bool, bool) {
        if self.state.objects[card as usize].zone != Zone::Exile {
            return (false, false);
        }
        self.state
            .exile_permissions
            .iter()
            .filter(|row| {
                row.player == player
                    && row.object == self.object_ref(card)
                    && row.expiry.active(self.state.turn)
            })
            .fold((false, false), |flags, row| {
                (
                    flags.0 || row.spend_any_mana,
                    flags.1 || row.exile_on_graveyard,
                )
            })
    }
    pub(super) fn exile_targets_for_modified_cast(&mut self, item: &SpellView) {
        let ids = item
            .targets
            .iter()
            .filter(|&&bound| self.same_target_object(bound))
            .filter_map(|bound| {
                if let Target::Card(id) = bound.target {
                    Some(id)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        let events = self.move_objects_raw(&ids, Zone::Exile);
        for event in &events {
            if event.moved && event.destination == Zone::Exile && !event.before.is_token() {
                self.grant_exile_play_permission(item.controller, event.after, false);
                for row in &mut self.state.exile_permissions {
                    if row.player == item.controller && row.object == event.after {
                        row.spend_any_mana = true;
                        row.exile_on_graveyard = true;
                    }
                }
            }
        }
        self.finish_zone_events(&events, &[]);
    }
    pub(super) fn exile_top_play(&mut self, player: Player, amount: u128, next_turn: bool) {
        self.exile_top_play_from(player, player, amount, next_turn);
    }

    pub(super) fn exile_top_play_from(
        &mut self,
        library_owner: Player,
        player: Player,
        amount: u128,
        next_turn: bool,
    ) {
        let ids = top_library_cards(&self.state.players[library_owner.index()].library, amount);
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
        if self.object_ref(object.card) != object
            || self.state.objects[object.card as usize].zone != Zone::Exile
        {
            return;
        }
        let expiry = if next_turn {
            ExilePlayExpiry::EndOfNextTurn
        } else {
            ExilePlayExpiry::Turn(self.state.turn)
        };
        let at = self
            .state
            .exile_permissions
            .partition_point(|p| p.object.card < object.card);
        self.state.exile_permissions.insert(
            at,
            ExilePlayPermission {
                object,
                player,
                expiry,
                spend_any_mana: false,
                cast_only: false,
                exile_on_graveyard: false,
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
        let objects = item
            .targets
            .iter()
            .copied()
            .filter(|&b| self.same_target_object(b))
            .filter_map(|b| b.object)
            .collect::<Vec<_>>();
        self.exile_objects_until_source_leaves(item.object, &objects);
    }
    /// A non-target hand choice and an ordinary bound target share the same exact
    /// movement/link transaction. Only an actual arrival in exile creates a link.
    pub(super) fn exile_objects_until_source_leaves(
        &mut self,
        source: ObjectRef,
        objects: &[ObjectRef],
    ) {
        if !self.on_battlefield(source.card) || self.object_ref(source.card) != source {
            return;
        }
        let ids: Vec<_> = objects
            .iter()
            .filter(|r| self.object_ref(r.card) == **r)
            .map(|r| r.card)
            .collect();
        let events = self.move_objects_raw(&ids, Zone::Exile);
        for event in &events {
            if event.moved && event.destination == Zone::Exile {
                self.state.exile_links.push(ExileLink {
                    source,
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
        self.refresh_enduring_stories();
        if events
            .iter()
            .any(|event| event.moved && event.before.zone == Zone::Battlefield)
        {
            self.prune_source_bound_copies();
        }
        self.record_entry_turn_facts(events);
        self.creature_cards_enter_graveyard_triggers(events);
        self.graveyard_departure_triggers(events, death_sources);
        self.returned_hand_triggers(events);
        for event in events {
            if event.moved
                && event.destination == Zone::Battlefield
                && self
                    .copiable_recipe(event.after)
                    .definition
                    .enters_prepared()
                && self.printed_abilities_available(event.after.card)
            {
                self.prepare_permanent(event.after);
            }
        }
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
        // One trigger per observing source for the entire simultaneous entry event.
        let mut token_entered = [false; 2];
        for event in events {
            if event.moved && event.destination == Zone::Battlefield {
                let entered = self.object_view(event.after.card);
                if entered.is_token() && entered.kinds & entry_suppression == 0 {
                    token_entered[entered.controller.index()] = true;
                }
            }
        }
        for source in &sources {
            if source.zone != Zone::Battlefield || !token_entered[source.controller.index()] {
                continue;
            }
            for (index, ability) in source.card.triggers().iter().enumerate() {
                if ability.condition.event() == TriggerCondition::TokensEnterYou {
                    self.queue_trigger(source, index);
                }
            }
        }
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
                let placed_by = event
                    .entry_counter_player
                    .expect("battlefield entry controller");
                let counters = self.counter_amounts(event.after.card).to_owned();
                for kind in CounterKind::ALL {
                    self.self_counters_added(event.after, kind, counters.get(kind), placed_by);
                }
                self.lore_counters_added(event.after, 0);
            }
        }
        if self.has_counter_batch_observers() {
            let placements: Vec<_> = events
                .iter()
                .filter(|event| event.moved && event.destination == Zone::Battlefield)
                .map(|event| {
                    (
                        event.after,
                        self.counter_amounts(event.after.card).to_owned(),
                        event
                            .entry_counter_player
                            .expect("battlefield entry controller"),
                    )
                })
                .collect();
            self.counter_batch_added(
                placements
                    .iter()
                    .map(|(object, amount, player)| (*object, amount, *player)),
            );
        }
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
