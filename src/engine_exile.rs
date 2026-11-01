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
    /// This permission requires life instead of the chosen spell face's mana cost.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub life_instead_of_mana: bool,
    /// This origin permission requires the free alternative, not a paid cast.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub without_paying_mana_cost: bool,
    /// A distinct permission to cast for {2} instead of its mana cost.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub airbend: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requires_subtype: Option<&'static str>,
}
/// Deadline boundaries refer to actual turns, never to skipped turn candidates.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum ExilePlayExpiry {
    WhileExiled,
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
pub(super) fn top_library_cards(game: &Game, player: Player, amount: u128) -> Vec<CardId> {
    let library = &game.state.players[player.index()].library;
    let count = usize::try_from(amount.min(library.len() as u128)).unwrap();
    library
        .iter()
        .rev()
        .copied()
        .filter(|&id| game.state.objects[id as usize].is_card())
        .take(count)
        .collect()
}
impl Game {
    /// The linked permission is controlled by the source's current controller
    /// and follows its ability and threshold, not the entry trigger's controller.
    pub(super) fn threshold_linked_source(&self, card: CardId) -> Option<(ObjectRef, Option<u64>)> {
        let reference = self.object_ref(card);
        self.state
            .linked_choices
            .as_ref()?
            .null_cast_links
            .iter()
            .find(|link| link.exiled == reference)
            .map(|link| (link.source, link.ability_origin))
    }
    fn can_cast_threshold_linked_exile(&self, player: Player, card: CardId) -> bool {
        let Some(object) = self.state.objects.get(card as usize) else {
            return false;
        };
        object.zone == Zone::Exile
            && !object.card.is_land()
            && self
                .threshold_linked_source(card)
                .is_some_and(|source| threshold_linked_source_available(self, player, source))
    }
    pub(super) fn can_play_exiled(&self, player: Player, card: CardId) -> bool {
        if self.can_cast_threshold_linked_exile(player, card)
            || self.can_cast_prepared(player, card)
            || self.can_play_foreign_stash(player, card)
            || self.can_play_adventure(player, card)
            || self.can_cast_warped(player, card)
        {
            return true;
        }
        self.state.exile_permissions.iter().any(|p| {
            !p.life_instead_of_mana
                && !p.without_paying_mana_cost
                && !p.airbend
                && p.player == player
                && p.object == self.object_ref(card)
                && p.expiry.active(self.state.turn)
                && self.exile_permission_guard(p)
        })
    }
    pub(super) fn can_play_exiled_land(&self, player: Player, card: CardId) -> bool {
        self.can_play_foreign_stash(player, card)
            || self.can_play_adventure(player, card)
            || self.state.exile_permissions.iter().any(|p| {
                !p.without_paying_mana_cost
                    && !p.airbend
                    && p.player == player
                    && p.object == self.object_ref(card)
                    && p.expiry.active(self.state.turn)
                    && self.exile_permission_guard(p)
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
                    && self.exile_permission_guard(row)
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
            if event.moved && event.destination == Zone::Exile && event.was_card {
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
        let ids = top_library_cards(self, library_owner, amount);
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
                life_instead_of_mana: false,
                without_paying_mana_cost: false,
                airbend: false,
                requires_subtype: None,
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
            .filter(|&&id| self.state.objects[id as usize].is_card())
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
        let Some(source) = item.source else {
            return;
        };
        let objects = item
            .targets
            .iter()
            .copied()
            .filter(|&b| self.same_target_object(b))
            .filter_map(|b| b.object)
            .collect::<Vec<_>>();
        self.exile_objects_until_source_leaves(source.object, &objects);
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
        self.queue_earthbend_returns(events);
        self.record_entry_turn_facts(events);
        self.queue_manifest_deaths(events);
        self.creature_death_batch_triggers(events, death_sources);
        self.creature_cards_enter_graveyard_triggers(events);
        self.permanent_cards_enter_graveyard_triggers(events);
        self.creatures_exiled_batch_triggers(events);
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
                    (matches!(o.zone, Zone::Battlefield | Zone::Command)
                        && (o.zone == Zone::Battlefield || o.card.is_emblem())
                        && !o.card.triggers().is_empty())
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
                && event.was_card
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

impl Game {
    pub(super) fn configure_exile_free_cast(item: &mut SpellView, flash_surcharge: bool) {
        let cast = item.cast.as_mut().unwrap();
        cast.method = CastMethod::Free;
        cast.flash_surcharge = flash_surcharge;
        cast.spend_any_mana = false;
        cast.exile_on_graveyard = false;
        cast.stash_origin = false;
        if item.source.expect("cast spell has source").card.x_symbols() > 0 {
            item.x = Some(0);
        }
    }
    pub(super) fn has_exile_free_permission(&self, player: Player, card: CardId) -> bool {
        self.state.objects[card as usize].zone == Zone::Exile
            && self.state.exile_permissions.iter().any(|p| {
                p.without_paying_mana_cost
                    && p.player == player
                    && p.object == self.object_ref(card)
                    && p.expiry.active(self.state.turn)
                    && self.exile_permission_guard(p)
            })
    }
    pub(super) fn grant_exile_free_permission(&mut self, player: Player, object: ObjectRef) {
        self.grant_exile_free_permission_with_expiry(player, object, ExilePlayExpiry::WhileExiled);
    }
    pub(super) fn grant_exile_free_permission_until_end(
        &mut self,
        player: Player,
        object: ObjectRef,
    ) {
        self.grant_exile_free_permission_with_expiry(
            player,
            object,
            ExilePlayExpiry::Turn(self.state.turn),
        );
    }
    fn grant_exile_free_permission_with_expiry(
        &mut self,
        player: Player,
        object: ObjectRef,
        expiry: ExilePlayExpiry,
    ) {
        if self.object_ref(object.card) != object
            || self.state.objects[object.card as usize].zone != Zone::Exile
            || self.state.objects[object.card as usize].is_token()
        {
            return;
        }
        if self.state.exile_permissions.iter().any(|p| {
            p.object == object
                && p.player == player
                && p.without_paying_mana_cost
                && p.expiry == expiry
        }) {
            return;
        }
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
                cast_only: true,
                exile_on_graveyard: false,
                life_instead_of_mana: false,
                without_paying_mana_cost: true,
                airbend: false,
                requires_subtype: None,
            },
        );
    }
    pub(super) fn visit_exile_free_casts(
        &self,
        player: Player,
        card: CardId,
        sorcery: bool,
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        if !self.has_exile_free_permission(player, card)
            || self.state.objects[card as usize].is_token()
        {
            return false;
        }
        for face in [
            CastFace::Normal,
            CastFace::Adventure,
            CastFace::Omen,
            CastFace::Back,
        ] {
            let physical = self.state.objects[card as usize].card;
            let definition = match face {
                CastFace::Normal => Some(physical),
                CastFace::Adventure => physical.adventure_spell(),
                CastFace::Omen => physical.omen_spell(),
                CastFace::Back => physical.back_face(),
            };
            if !definition.is_some_and(|d| !d.is_land()) {
                continue;
            }
            for flash_surcharge in [false, true] {
                if self.permitted_exile_free_cast_available(
                    player,
                    card,
                    face,
                    sorcery,
                    flash_surcharge,
                ) && emit(Action::AnnounceExileFree {
                    card,
                    face,
                    flash_surcharge,
                }) {
                    return true;
                }
            }
        }
        false
    }
}

impl Game {
    pub(super) fn exile_permission_guard(&self, permission: &ExilePlayPermission) -> bool {
        permission.requires_subtype.is_none_or(|subtype| {
            self.state.objects.iter().enumerate().any(|(id, o)| {
                o.zone == Zone::Battlefield
                    && o.controller == permission.player
                    && self.has_subtype(id as CardId, subtype)
            })
        })
    }
    pub(super) fn exile_top_face_down_play(
        &mut self,
        item: &SpellView,
        amount: u16,
        subtype: &'static str,
    ) {
        let Some(source) = item.source else {
            return;
        };
        let ids = top_library_cards(self, item.controller, u128::from(amount));
        let events = self.move_objects_raw(&ids, Zone::Exile);
        for event in &events {
            if !event.moved || event.destination != Zone::Exile {
                continue;
            }
            // Look access belongs to the captured player, independently of the
            // source and the condition on future plays. Reuse exact hidden rows.
            self.state
                .linked_choices
                .get_or_insert_default()
                .hidden
                .push((source.object, event.after, 1 << item.controller.index()));
            let at = self
                .state
                .exile_permissions
                .partition_point(|p| p.object.card < event.after.card);
            self.state.exile_permissions.insert(
                at,
                ExilePlayPermission {
                    object: event.after,
                    player: item.controller,
                    expiry: ExilePlayExpiry::WhileExiled,
                    spend_any_mana: false,
                    cast_only: false,
                    exile_on_graveyard: false,
                    life_instead_of_mana: false,
                    without_paying_mana_cost: false,
                    airbend: false,
                    requires_subtype: Some(subtype),
                },
            );
        }
        self.finish_zone_events(&events, &[]);
    }
}

/// Re-evaluate the continuous permission in the live or speculative world.
pub(super) fn threshold_linked_source_available<W: ability_world::CharacteristicWorld + ?Sized>(
    world: &W,
    player: Player,
    linked: (ObjectRef, Option<u64>),
) -> bool {
    let (source, ability_origin) = linked;
    let object = world.object(source.card);
    object.zone == Zone::Battlefield
        && world.reference(source.card) == source
        && object.controller == player
        && world.ability_origin(source.card) == ability_origin
        && activations::graveyard_cards_at_least(world, player, 0, 7)
        && object.card.static_ability_groups().iter().any(|group| {
            group.fragments.iter().any(|fragment| {
                matches!(
                    fragment,
                    crate::cards::StaticAbilityFragment::CastLinkedExileAtThreshold
                )
            })
        })
        && characteristic_query::CharacteristicQuery::in_world(world).continuous_fragment_available(
            source.card,
            |fragment| {
                matches!(
                    fragment,
                    crate::cards::StaticAbilityFragment::CastLinkedExileAtThreshold
                )
            },
        )
}

impl Game {
    pub(super) fn configure_airbend_cast(item: &mut SpellView, flash_surcharge: bool) {
        let cast = item.cast.as_mut().expect("cast information");
        cast.method = CastMethod::Airbend;
        cast.flash_surcharge = flash_surcharge;
        cast.spend_any_mana = false;
        cast.exile_on_graveyard = false;
        cast.stash_origin = false;
        if item.source.expect("cast source").card.x_symbols() > 0 {
            item.x = Some(0);
        }
    }
    pub(super) fn has_airbend_permission(&self, player: Player, card: CardId) -> bool {
        self.state.objects[card as usize].zone == Zone::Exile
            && self.state.exile_permissions.iter().any(|row| {
                row.airbend
                    && row.player == player
                    && row.object == self.object_ref(card)
                    && row.expiry.active(self.state.turn)
                    && self.exile_permission_guard(row)
            })
    }
    pub(super) fn grant_airbend_permission(&mut self, player: Player, object: ObjectRef) {
        if self.object_ref(object.card) != object
            || self.state.objects[object.card as usize].zone != Zone::Exile
            || self.state.objects[object.card as usize].is_token()
            || self.state.objects[object.card as usize].is_spell_copy()
        {
            return;
        }
        if self.has_airbend_permission(player, object.card) {
            return;
        }
        let at = self
            .state
            .exile_permissions
            .partition_point(|row| row.object.card < object.card);
        self.state.exile_permissions.insert(
            at,
            ExilePlayPermission {
                object,
                player,
                expiry: ExilePlayExpiry::WhileExiled,
                spend_any_mana: false,
                cast_only: true,
                exile_on_graveyard: false,
                life_instead_of_mana: false,
                without_paying_mana_cost: false,
                airbend: true,
                requires_subtype: None,
            },
        );
    }
    pub(super) fn visit_airbend_casts(
        &self,
        player: Player,
        card: CardId,
        sorcery: bool,
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        if !self.has_airbend_permission(player, card)
            || self.state.objects[card as usize].is_token()
        {
            return false;
        }
        let physical = self.state.objects[card as usize].card;
        for face in [
            CastFace::Normal,
            CastFace::Adventure,
            CastFace::Omen,
            CastFace::Back,
        ] {
            let definition = match face {
                CastFace::Normal => Some(physical),
                CastFace::Adventure => physical.adventure_spell(),
                CastFace::Omen => physical.omen_spell(),
                CastFace::Back => physical.back_face(),
            };
            if !definition.is_some_and(|d| !d.is_land()) {
                continue;
            }
            for flash_surcharge in [false, true] {
                if self.permitted_cast_available(
                    player,
                    card,
                    CastMethod::Airbend,
                    face,
                    sorcery,
                    flash_surcharge,
                ) && emit(Action::AnnounceAirbend {
                    card,
                    face,
                    flash_surcharge,
                }) {
                    return true;
                }
            }
        }
        false
    }
}
